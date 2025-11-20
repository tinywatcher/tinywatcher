mod email;
mod slack;
mod stdout;
mod webhook;

use async_trait::async_trait;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub use email::EmailAlert;
pub use slack::SlackAlert;
pub use stdout::StdoutAlert;
pub use webhook::WebhookAlert;

/// Trait that all alert handlers must implement
#[async_trait]
pub trait AlertHandler: Send + Sync {
    /// Send an alert with the given rule name and message
    async fn send(&self, rule_name: &str, message: &str) -> Result<()>;
    
    /// Get a human-readable name for this alert handler
    fn name(&self) -> &str;
}

/// Manages alert handlers and cooldowns
pub struct AlertManager {
    handlers: HashMap<String, Arc<dyn AlertHandler>>,
    cooldowns: Arc<Mutex<HashMap<String, Instant>>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            cooldowns: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register an alert handler with a unique name
    pub fn register(&mut self, name: String, handler: Arc<dyn AlertHandler>) {
        self.handlers.insert(name, handler);
    }

    /// Send an alert to a specific handler by name
    pub async fn send_alert(
        &self,
        alert_name: &str,
        rule_name: &str,
        message: &str,
        cooldown_secs: u64,
    ) -> Result<()> {
        // Check cooldown
        if !self.check_cooldown(rule_name, cooldown_secs).await {
            return Ok(());
        }

        // Look up the alert handler
        let handler = self.handlers.get(alert_name).ok_or_else(|| {
            anyhow::anyhow!("Alert '{}' not found in configuration", alert_name)
        })?;

        handler.send(rule_name, message).await
    }

    /// Send an alert to multiple handlers
    pub async fn send_alert_multi(
        &self,
        alert_names: &[String],
        rule_name: &str,
        message: &str,
        cooldown_secs: u64,
    ) -> Result<()> {
        // Check cooldown
        if !self.check_cooldown(rule_name, cooldown_secs).await {
            return Ok(());
        }

        // Send to all specified handlers
        for alert_name in alert_names {
            let handler = self.handlers.get(alert_name).ok_or_else(|| {
                anyhow::anyhow!("Alert '{}' not found in configuration", alert_name)
            })?;

            if let Err(e) = handler.send(rule_name, message).await {
                tracing::error!("Failed to send alert to '{}': {}", alert_name, e);
            }
        }

        Ok(())
    }

    async fn check_cooldown(&self, rule_name: &str, cooldown_secs: u64) -> bool {
        let mut cooldowns = self.cooldowns.lock().await;
        
        if let Some(last_alert) = cooldowns.get(rule_name) {
            if last_alert.elapsed() < Duration::from_secs(cooldown_secs) {
                return false;
            }
        }
        
        cooldowns.insert(rule_name.to_string(), Instant::now());
        true
    }
}
