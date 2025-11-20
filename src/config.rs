use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub inputs: Inputs,
    #[serde(default)]
    pub alerts: HashMap<String, Alert>,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Alert {
    #[serde(rename = "type")]
    pub alert_type: AlertType,
    #[serde(flatten)]
    pub options: AlertOptions,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AlertOptions {
    Slack { url: String },
    Webhook { url: String },
    Stdout {},
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    Stdout,
    Slack,
    Webhook,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    pub pattern: String,
    #[serde(deserialize_with = "string_or_seq_string")]
    pub alert: Vec<String>,  // Can be a single alert name or list of alert names
    #[serde(default = "default_cooldown")]
    pub cooldown: u64,
}

// Helper function to deserialize either a string or array of strings
fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};

    struct StringOrVec;

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_string()])
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Vec::<String>::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
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
    pub alert: String,  // Now references alert name instead of type
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
