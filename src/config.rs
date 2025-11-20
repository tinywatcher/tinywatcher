use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub inputs: Inputs,
    #[serde(default)]
    pub alerts: Alerts,
    #[serde(default)]
    pub rules: Vec<Rule>,
    pub resources: Option<ResourceConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Inputs {
    #[serde(default)]
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub containers: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Alerts {
    pub slack: Option<String>,
    pub webhook: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    pub pattern: String,
    pub alert: AlertType,
    #[serde(default = "default_cooldown")]
    pub cooldown: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    Stdout,
    Slack,
    Webhook,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceConfig {
    #[serde(default = "default_interval")]
    pub interval: u64,
    pub thresholds: ResourceThresholds,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceThresholds {
    pub cpu_percent: Option<f32>,
    pub memory_percent: Option<f32>,
    pub disk_percent: Option<f32>,
    pub alert: AlertType,
}

fn default_cooldown() -> u64 {
    60
}

fn default_interval() -> u64 {
    10
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn merge_with_cli(&mut self, files: Vec<PathBuf>, containers: Vec<String>) {
        if !files.is_empty() {
            self.inputs.files.extend(files);
        }
        if !containers.is_empty() {
            self.inputs.containers.extend(containers);
        }
    }
}
