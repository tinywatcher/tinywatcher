use crate::alerts::AlertManager;
use crate::config::Rule;
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
    alert_type: crate::config::AlertType,
    cooldown: u64,
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
                    alert_type: rule.alert,
                    cooldown: rule.cooldown,
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

        while let Some(line) = lines.next_line().await? {
            self.process_line(&line).await;
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

        // Spawn tasks to read both streams
        let stdout_task = {
            let monitor = self_clone.clone();
            tokio::spawn(async move {
                let mut lines = stdout_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    monitor.process_line(&line).await;
                }
            })
        };

        let stderr_task = {
            let monitor = self_clone;
            tokio::spawn(async move {
                let mut lines = stderr_reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    monitor.process_line(&line).await;
                }
            })
        };

        tokio::try_join!(stdout_task, stderr_task)?;

        Ok(())
    }

    async fn process_line(&self, line: &str) {
        for rule in &self.rules {
            if rule.regex.is_match(line) {
                tracing::debug!("Rule '{}' matched line: {}", rule.name, line);
                
                if let Err(e) = self
                    .alert_manager
                    .send_alert(&rule.alert_type, &rule.name, line, rule.cooldown)
                    .await
                {
                    tracing::error!("Failed to send alert: {}", e);
                }
            }
        }
    }

    fn clone_monitor(&self) -> Self {
        Self {
            rules: self.rules.iter().map(|r| CompiledRule {
                name: r.name.clone(),
                regex: r.regex.clone(),
                alert_type: r.alert_type.clone(),
                cooldown: r.cooldown,
            }).collect(),
            alert_manager: self.alert_manager.clone(),
        }
    }
}
