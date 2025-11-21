use crate::alerts::AlertManager;
use crate::config::{Rule, SourceType};
use anyhow::{Context, Result};
use regex::Regex;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub struct LogMonitor {
    rules: Vec<CompiledRule>,
    alert_manager: Arc<AlertManager>,
}

struct CompiledRule {
    name: String,
    regex: Regex,
    alert_names: Vec<String>,
    cooldown: u64,
    sources: Option<crate::config::RuleSources>,
}

impl LogMonitor {
    pub fn new(rules: Vec<Rule>, alert_manager: Arc<AlertManager>) -> Result<Self> {
        let compiled_rules = rules
            .into_iter()
            .map(|rule| {
                Ok(CompiledRule {
                    name: rule.name.clone(),
                    regex: Regex::new(&rule.pattern)
                        .context(format!("Invalid regex pattern in rule: {}", rule.name))?,
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

    pub async fn watch_file(&self, path: PathBuf) -> Result<()> {
        tracing::info!("Watching file: {}", path.display());

        let mut cmd = Command::new("tail")
            .arg("-f")
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
        while let Some(line) = lines.next_line().await? {
            self.process_line(&line, &source).await;
        }

        Ok(())
    }

    pub async fn watch_container(&self, container_name: String) -> Result<()> {
        tracing::info!("Watching container: {}", container_name);

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
                    monitor.process_line(&line, &source).await;
                }
            })
        };

        tokio::try_join!(stdout_task, stderr_task)?;

        Ok(())
    }

    async fn process_line(&self, line: &str, source: &SourceType) {
        for rule in &self.rules {
            // Check if rule applies to this source
            if !self.rule_applies_to_source(rule, source) {
                continue;
            }

            if rule.regex.is_match(line) {
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
                regex: r.regex.clone(),
                alert_names: r.alert_names.clone(),
                cooldown: r.cooldown,
                sources: r.sources.clone(),
            }).collect(),
            alert_manager: self.alert_manager.clone(),
        }
    }
}
