use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use regex::Regex;

/// Threshold configuration for rate-based alerting
/// Example: "5 in 2s" means trigger alert if event occurs 5 times within 2 seconds
#[derive(Debug, Clone, PartialEq)]
pub struct Threshold {
    pub count: u32,
    pub window: Duration,
}

impl Threshold {
    /// Parse a threshold string like "5 in 2s"
    /// Supported formats:
    /// - "5 in 2s"   - 5 occurrences in 2 seconds
    /// - "10 in 1m"  - 10 occurrences in 1 minute
    /// - "3 in 500ms" - 3 occurrences in 500 milliseconds
    /// - "100 in 1h" - 100 occurrences in 1 hour
    pub fn parse(s: &str) -> Result<Self, String> {
        let re = Regex::new(r"^\s*(?P<count>\d+)\s+in\s+(?P<value>\d+)(?P<unit>ms|s|m|h)\s*$")
            .unwrap();
        
        let caps = re.captures(s)
            .ok_or_else(|| format!("Invalid threshold format: '{}'. Expected format like '5 in 2s'", s))?;
        
        let count: u32 = caps["count"].parse()
            .map_err(|_| format!("Invalid count in threshold: '{}'", &caps["count"]))?;
        
        let value: u64 = caps["value"].parse()
            .map_err(|_| format!("Invalid value in threshold: '{}'", &caps["value"]))?;
        
        let window = match &caps["unit"] {
            "ms" => Duration::from_millis(value),
            "s"  => Duration::from_secs(value),
            "m"  => Duration::from_secs(value * 60),
            "h"  => Duration::from_secs(value * 3600),
            _ => return Err(format!("Invalid time unit in '{}'", s)),
        };
        
        Ok(Threshold { count, window })
    }
}

// Custom serde deserialization for Threshold
impl<'de> Deserialize<'de> for Threshold {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Threshold::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Threshold {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize back to string format
        let secs = self.window.as_secs();
        let millis = self.window.as_millis();
        
        let s = if millis < 1000 {
            format!("{} in {}ms", self.count, millis)
        } else if secs < 60 {
            format!("{} in {}s", self.count, secs)
        } else if secs < 3600 {
            format!("{} in {}m", self.count, secs / 60)
        } else {
            format!("{} in {}h", self.count, secs / 3600)
        };
        
        serializer.serialize_str(&s)
    }
}

// Helper function to expand environment variables in strings
fn expand_env_vars(value: &str) -> String {
    let re = Regex::new(r"\$\{([^}]+)\}|\$([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    
    re.replace_all(value, |caps: &regex::Captures| {
        let var_name = caps.get(1).or_else(|| caps.get(2)).unwrap().as_str();
        std::env::var(var_name).unwrap_or_else(|_| {
            eprintln!("Warning: Environment variable '{}' not found, using empty string", var_name);
            String::new()
        })
    }).to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub inputs: Inputs,
    #[serde(default)]
    pub alerts: HashMap<String, Alert>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    pub resources: Option<ResourceConfig>,
    #[serde(default)]
    pub identity: Identity,
    #[serde(default)]
    pub system_checks: Vec<SystemCheck>,
    pub heartbeat: Option<HeartbeatConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeartbeatConfig {
    pub url: String,
    #[serde(default = "default_heartbeat_interval")]
    pub interval: u64,  // seconds
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemCheck {
    pub name: String,
    #[serde(rename = "type")]
    pub check_type: SystemCheckType,
    pub url: String,
    #[serde(default = "default_check_interval")]
    pub interval: u64,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_missed_threshold")]
    pub missed_threshold: u32,
    #[serde(deserialize_with = "string_or_seq_string")]
    pub alert: Vec<String>,
    /// Optional threshold for rate-based health check alerting (e.g., "3 in 1m")
    /// If specified, alert only when failures occur this many times within the window
    /// This provides an alternative to missed_threshold for more sophisticated failure detection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<Threshold>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SystemCheckType {
    Http,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Identity {
    pub name: Option<String>,
}

impl Default for Identity {
    fn default() -> Self {
        Self {
            name: None,
        }
    }
}

impl Identity {
    /// Get the identity name, using hostname as fallback if not specified
    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string())
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Inputs {
    #[serde(default)]
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub containers: Vec<String>,
    #[serde(default)]
    pub streams: Vec<StreamConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamConfig {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub stream_type: StreamType,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconnect_delay: Option<u64>,  // seconds
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Websocket,
    Http,
    Tcp,
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
    Discord { url: String },
    Telegram { 
        bot_token: String, 
        chat_id: String 
    },
    PagerDuty { 
        routing_key: String 
    },
    Ntfy { 
        topic: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        server: Option<String>,
    },
    Email { 
        from: String, 
        to: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        smtp_server: Option<String>,
    },
    SendGrid {
        api_key: String,
        from: String,
        to: Vec<String>,
    },
    Stdout {},
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    Stdout,
    Slack,
    Webhook,
    Discord,
    Telegram,
    PagerDuty,
    Ntfy,
    Email,
    SendGrid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(deserialize_with = "string_or_seq_string")]
    pub alert: Vec<String>,  // Can be a single alert name or list of alert names
    #[serde(default = "default_cooldown")]
    pub cooldown: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<RuleSources>,
    /// Optional threshold for rate-based alerting (e.g., "5 in 2s")
    /// If specified, alert only when the pattern matches this many times within the window
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<Threshold>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Text(String),
    Regex(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleSources {
    #[serde(default)]
    pub containers: Vec<String>,
    #[serde(default)]
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub streams: Vec<String>,
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
    #[serde(deserialize_with = "string_or_seq_string")]
    pub alert: Vec<String>,  // Now references alert names (can be multiple)
}

fn default_cooldown() -> u64 {
    60
}

fn default_interval() -> u64 {
    10
}

fn default_check_interval() -> u64 {
    30
}

fn default_timeout() -> u64 {
    5
}

fn default_missed_threshold() -> u32 {
    2
}

fn default_heartbeat_interval() -> u64 {
    60
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&content)?;
        config.expand_env_vars();
        Ok(config)
    }

    /// Expand glob patterns in file paths
    /// Returns a new list of files with all globs expanded
    pub fn expand_file_globs(&self) -> anyhow::Result<Vec<PathBuf>> {
        let mut expanded_files = Vec::new();
        
        for file_pattern in &self.inputs.files {
            let pattern_str = file_pattern.to_string_lossy();
            
            // Check if the pattern contains glob characters
            if pattern_str.contains('*') || pattern_str.contains('?') || pattern_str.contains('[') {
                // This is a glob pattern, expand it
                match glob::glob(&pattern_str) {
                    Ok(paths) => {
                        let mut found_any = false;
                        for entry in paths {
                            match entry {
                                Ok(path) => {
                                    // Only include files, not directories
                                    if path.is_file() {
                                        expanded_files.push(path);
                                        found_any = true;
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!("Error reading glob entry for '{}': {}", pattern_str, e);
                                }
                            }
                        }
                        
                        if !found_any {
                            tracing::warn!("Glob pattern '{}' matched no files", pattern_str);
                        } else {
                            tracing::info!("Glob pattern '{}' matched {} file(s)", pattern_str, expanded_files.len());
                        }
                    }
                    Err(e) => {
                        tracing::error!("Invalid glob pattern '{}': {}", pattern_str, e);
                        anyhow::bail!("Invalid glob pattern '{}': {}", pattern_str, e);
                    }
                }
            } else {
                // Not a glob pattern, use as-is
                expanded_files.push(file_pattern.clone());
            }
        }
        
        Ok(expanded_files)
    }

    /// Expand environment variables in all string fields
    fn expand_env_vars(&mut self) {
        // Expand in alerts
        for alert in self.alerts.values_mut() {
            match &mut alert.options {
                AlertOptions::Slack { url } => {
                    *url = expand_env_vars(url);
                }
                AlertOptions::Webhook { url } => {
                    *url = expand_env_vars(url);
                }
                AlertOptions::Discord { url } => {
                    *url = expand_env_vars(url);
                }
                AlertOptions::Telegram { bot_token, chat_id } => {
                    *bot_token = expand_env_vars(bot_token);
                    *chat_id = expand_env_vars(chat_id);
                }
                AlertOptions::PagerDuty { routing_key } => {
                    *routing_key = expand_env_vars(routing_key);
                }
                AlertOptions::Ntfy { topic, server } => {
                    *topic = expand_env_vars(topic);
                    if let Some(srv) = server {
                        *srv = expand_env_vars(srv);
                    }
                }
                AlertOptions::Email { from, to, smtp_server } => {
                    *from = expand_env_vars(from);
                    for email in to.iter_mut() {
                        *email = expand_env_vars(email);
                    }
                    if let Some(server) = smtp_server {
                        *server = expand_env_vars(server);
                    }
                }
                AlertOptions::SendGrid { api_key, from, to } => {
                    *api_key = expand_env_vars(api_key);
                    *from = expand_env_vars(from);
                    for email in to.iter_mut() {
                        *email = expand_env_vars(email);
                    }
                }
                AlertOptions::Stdout {} => {}
            }
        }

        // Expand in streams
        for stream in &mut self.inputs.streams {
            stream.url = expand_env_vars(&stream.url);
            if let Some(name) = &mut stream.name {
                *name = expand_env_vars(name);
            }
            if let Some(headers) = &mut stream.headers {
                for value in headers.values_mut() {
                    *value = expand_env_vars(value);
                }
            }
        }

        // Expand in system checks
        for check in &mut self.system_checks {
            check.url = expand_env_vars(&check.url);
        }

        // Expand in heartbeat
        if let Some(heartbeat) = &mut self.heartbeat {
            heartbeat.url = expand_env_vars(&heartbeat.url);
        }

        // Expand in identity
        if let Some(name) = &mut self.identity.name {
            *name = expand_env_vars(name);
        }
    }

    #[allow(dead_code)]
    pub fn merge_with_cli(&mut self, files: Vec<PathBuf>, containers: Vec<String>) {
        if !files.is_empty() {
            self.inputs.files.extend(files);
        }
        if !containers.is_empty() {
            self.inputs.containers.extend(containers);
        }
    }
}

impl StreamConfig {
    pub fn get_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| {
            format!("{:?}:{}", self.stream_type, self.url)
        })
    }

    pub fn get_reconnect_delay(&self) -> u64 {
        self.reconnect_delay.unwrap_or(5)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceType {
    File(PathBuf),
    Container(String),
    Stream(String),
}

impl Rule {
    /// Validate that the rule has exactly one of text or pattern
    pub fn validate(&self) -> anyhow::Result<()> {
        match (&self.text, &self.pattern) {
            (None, None) => anyhow::bail!(
                "Rule '{}' must have either 'text' or 'pattern' field", 
                self.name
            ),
            (Some(_), Some(_)) => anyhow::bail!(
                "Rule '{}' cannot have both 'text' and 'pattern' fields", 
                self.name
            ),
            _ => Ok(()),
        }
    }

    /// Get the match type for this rule
    pub fn match_type(&self) -> MatchType {
        if let Some(ref text) = self.text {
            MatchType::Text(text.clone())
        } else if let Some(ref pattern) = self.pattern {
            MatchType::Regex(pattern.clone())
        } else {
            // This should never happen if validate() was called
            panic!("Rule '{}' has neither text nor pattern", self.name)
        }
    }

    /// Check if this rule applies to the given source
    /// Returns true if the rule has no sources filter (applies to all) or if the source matches
    #[allow(dead_code)]
    pub fn applies_to_source(&self, source: &SourceType) -> bool {
        // If no sources filter is specified, rule applies to all sources
        let Some(ref sources) = self.sources else {
            return true;
        };

        match source {
            SourceType::File(path) => {
                // If no files specified in filter, don't match any files
                if sources.files.is_empty() {
                    return false;
                }
                // Check if the path matches any of the specified files
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
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;

