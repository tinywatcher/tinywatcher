use crate::alerts::AlertManager;
use crate::config::{MatchType, Rule, SourceType};
use anyhow::{Context, Result};
use regex::Regex;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// Maximum line length to prevent regex DoS
const MAX_LINE_LENGTH: usize = 10_000;

/// Initial retry delay
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);

/// Maximum retry delay
const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);

pub struct LogMonitor {
    rules: Vec<CompiledRule>,
    alert_manager: Arc<AlertManager>,
}

struct CompiledRule {
    name: String,
    matcher: RuleMatcher,
    alert_names: Vec<String>,
    cooldown: u64,
    sources: Option<crate::config::RuleSources>,
}

enum RuleMatcher {
    Text(String),
    Regex(Regex),
}

impl LogMonitor {
    pub fn new(rules: Vec<Rule>, alert_manager: Arc<AlertManager>) -> Result<Self> {
        let compiled_rules = rules
            .into_iter()
            .map(|rule| {
                let matcher = match rule.match_type() {
                    MatchType::Text(text) => RuleMatcher::Text(text),
                    MatchType::Regex(pattern) => {
                        let regex = Regex::new(&pattern)
                            .context(format!("Invalid regex pattern in rule: {}", rule.name))?;
                        RuleMatcher::Regex(regex)
                    }
                };

                Ok(CompiledRule {
                    name: rule.name.clone(),
                    matcher,
                    alert_names: rule.alert,
                    cooldown: rule.cooldown,
                    sources: rule.sources,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            rules: compiled_rules,
            alert_manager,
        })
    }

    /// Watch a file with automatic retry and reconnection
    pub async fn watch_file(&self, path: PathBuf) -> Result<()> {
        let mut retry_delay = INITIAL_RETRY_DELAY;
        
        loop {
            match self.watch_file_once(path.clone()).await {
                Ok(_) => {
                    tracing::warn!("File watcher exited cleanly for: {}", path.display());
                    // Reset retry delay on successful connection
                    retry_delay = INITIAL_RETRY_DELAY;
                }
                Err(e) => {
                    tracing::error!(
                        "File watch failed for {}: {}. Retrying in {:?}...",
                        path.display(),
                        e,
                        retry_delay
                    );
                }
            }
            
            tokio::time::sleep(retry_delay).await;
            retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
        }
    }

    /// Watch a file once (internal, no retry)
    async fn watch_file_once(&self, path: PathBuf) -> Result<()> {
        tracing::info!("Starting file watch: {}", path.display());

        let mut cmd = Command::new("tail")
            .arg("-F")  // Follow by name, handles log rotation
            .arg("-n")
            .arg("0")
            .arg(&path)
            .stdout(Stdio::piped())
            .spawn()
            .context("Failed to spawn tail command")?;

        let stdout = cmd.stdout.take().context("Failed to capture stdout")?;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let source = SourceType::File(path.clone());
        
        loop {
            tokio::select! {
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            // Enforce line length limit
                            if line.len() > MAX_LINE_LENGTH {
                                tracing::warn!(
                                    "Skipping line longer than {} bytes in {}",
                                    MAX_LINE_LENGTH,
                                    path.display()
                                );
                                continue;
                            }
                            self.process_line(&line, &source).await;
                        }
                        Ok(None) => {
                            tracing::debug!("EOF reached for {}", path.display());
                            break;
                        }
                        Err(e) => {
                            // Kill child process before returning error
                            let _ = cmd.kill().await;
                            return Err(e.into());
                        }
                    }
                }
                status = cmd.wait() => {
                    match status {
                        Ok(exit_status) => {
                            tracing::warn!(
                                "tail process exited with status: {} for {}",
                                exit_status,
                                path.display()
                            );
                            return Err(anyhow::anyhow!("tail process exited: {}", exit_status));
                        }
                        Err(e) => {
                            return Err(e).context("Failed to wait on tail process");
                        }
                    }
                }
            }
        }

        // Ensure child process is killed
        let _ = cmd.kill().await;
        Ok(())
    }

    /// Watch a container with automatic retry and reconnection
    pub async fn watch_container(&self, container_name: String) -> Result<()> {
        let mut retry_delay = INITIAL_RETRY_DELAY;
        
        loop {
            match self.watch_container_once(container_name.clone()).await {
                Ok(_) => {
                    tracing::warn!("Container watcher exited cleanly for: {}", container_name);
                    // Reset retry delay on successful connection
                    retry_delay = INITIAL_RETRY_DELAY;
                }
                Err(e) => {
                    tracing::error!(
                        "Container watch failed for {}: {}. Retrying in {:?}...",
                        container_name,
                        e,
                        retry_delay
                    );
                }
            }
            
            tokio::time::sleep(retry_delay).await;
            retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
        }
    }

    /// Watch a container once (internal, no retry)
    async fn watch_container_once(&self, container_name: String) -> Result<()> {
        tracing::info!("Starting container watch: {}", container_name);

        let mut cmd = Command::new("docker")
            .arg("logs")
            .arg("-f")
            .arg("--tail")
            .arg("0")
            .arg(&container_name)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn docker logs command")?;

        // Read both stdout and stderr
        let stdout = cmd.stdout.take().context("Failed to capture stdout")?;
        let stderr = cmd.stderr.take().context("Failed to capture stderr")?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let self_clone = Arc::new(self.clone_monitor());
        let source = SourceType::Container(container_name.clone());

        // Spawn tasks to read both streams
        let stdout_task = {
            let monitor = self_clone.clone();
            let source = source.clone();
            tokio::spawn(async move {
                let mut lines = stdout_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Enforce line length limit
                    if line.len() > MAX_LINE_LENGTH {
                        tracing::warn!(
                            "Skipping line longer than {} bytes in container {:?}",
                            MAX_LINE_LENGTH,
                            source
                        );
                        continue;
                    }
                    monitor.process_line(&line, &source).await;
                }
            })
        };

        let stderr_task = {
            let monitor = self_clone;
            let source = source.clone();
            tokio::spawn(async move {
                let mut lines = stderr_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Enforce line length limit
                    if line.len() > MAX_LINE_LENGTH {
                        tracing::warn!(
                            "Skipping line longer than {} bytes in container {:?}",
                            MAX_LINE_LENGTH,
                            source
                        );
                        continue;
                    }
                    monitor.process_line(&line, &source).await;
                }
            })
        };

        // Wait for both tasks to complete and the process to exit
        tokio::select! {
            result = async {
                tokio::try_join!(stdout_task, stderr_task)
            } => {
                // Kill process if streams finish
                let _ = cmd.kill().await;
                result?;
            }
            status = cmd.wait() => {
                let exit_status = status.context("Failed to wait on docker logs process")?;
                tracing::warn!(
                    "docker logs process exited with status: {} for {}",
                    exit_status,
                    container_name
                );
                return Err(anyhow::anyhow!("docker logs process exited: {}", exit_status));
            }
        }

        Ok(())
    }

    async fn process_line(&self, line: &str, source: &SourceType) {
        for rule in &self.rules {
            // Check if rule applies to this source
            if !self.rule_applies_to_source(rule, source) {
                continue;
            }

            let matched = match &rule.matcher {
                RuleMatcher::Text(text) => line.contains(text),
                RuleMatcher::Regex(regex) => regex.is_match(line),
            };

            if matched {
                tracing::debug!("Rule '{}' matched line from {:?}: {}", rule.name, source, line);
                
                // Send alert to all configured destinations
                if let Err(e) = self
                    .alert_manager
                    .send_alert_multi(&rule.alert_names, &rule.name, line, rule.cooldown)
                    .await
                {
                    tracing::error!("Failed to send alert for rule '{}': {}", rule.name, e);
                }
            }
        }
    }

    fn rule_applies_to_source(&self, rule: &CompiledRule, source: &SourceType) -> bool {
        // If no sources filter is specified, rule applies to all sources
        let Some(ref sources) = rule.sources else {
            return true;
        };

        match source {
            SourceType::File(path) => {
                if sources.files.is_empty() {
                    return false;
                }
                sources.files.iter().any(|f| f == path)
            }
            SourceType::Container(name) => {
                if sources.containers.is_empty() {
                    return false;
                }
                sources.containers.iter().any(|c| c == name)
            }
            SourceType::Stream(name) => {
                if sources.streams.is_empty() {
                    return false;
                }
                sources.streams.iter().any(|s| s == name)
            }
        }
    }

    fn clone_monitor(&self) -> Self {
        Self {
            rules: self.rules.iter().map(|r| CompiledRule {
                name: r.name.clone(),
                matcher: match &r.matcher {
                    RuleMatcher::Text(text) => RuleMatcher::Text(text.clone()),
                    RuleMatcher::Regex(regex) => RuleMatcher::Regex(regex.clone()),
                },
                alert_names: r.alert_names.clone(),
                cooldown: r.cooldown,
                sources: r.sources.clone(),
            }).collect(),
            alert_manager: self.alert_manager.clone(),
        }
    }
}
