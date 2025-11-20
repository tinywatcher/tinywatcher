use crate::config::{AlertType, Alerts};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct AlertManager {
    alerts: Alerts,
    cooldowns: Arc<Mutex<HashMap<String, Instant>>>,
    client: reqwest::Client,
}

impl AlertManager {
    pub fn new(alerts: Alerts) -> Self {
        Self {
            alerts,
            cooldowns: Arc::new(Mutex::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_alert(
        &self,
        alert_type: &AlertType,
        rule_name: &str,
        message: &str,
        cooldown_secs: u64,
    ) -> anyhow::Result<()> {
        // Check cooldown
        if !self.check_cooldown(rule_name, cooldown_secs).await {
            return Ok(());
        }

        match alert_type {
            AlertType::Stdout => self.send_stdout(rule_name, message).await,
            AlertType::Slack => self.send_slack(rule_name, message).await,
            AlertType::Webhook => self.send_webhook(rule_name, message).await,
        }
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

    async fn send_stdout(&self, rule_name: &str, message: &str) -> anyhow::Result<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}] ALERT [{}]: {}", timestamp, rule_name, message);
        Ok(())
    }

    async fn send_slack(&self, rule_name: &str, message: &str) -> anyhow::Result<()> {
        if let Some(webhook_url) = &self.alerts.slack {
            let payload = json!({
                "text": format!("🚨 *Alert: {}*\n```{}```", rule_name, message),
                "username": "TinyWatcher",
                "icon_emoji": ":eyes:"
            });

            self.client
                .post(webhook_url)
                .json(&payload)
                .send()
                .await?;
            
            tracing::info!("Sent Slack alert for rule: {}", rule_name);
        } else {
            tracing::warn!("Slack webhook not configured, cannot send alert");
        }
        Ok(())
    }

    async fn send_webhook(&self, rule_name: &str, message: &str) -> anyhow::Result<()> {
        if let Some(webhook_url) = &self.alerts.webhook {
            let payload = json!({
                "rule": rule_name,
                "message": message,
                "timestamp": Utc::now().to_rfc3339(),
            });

            self.client
                .post(webhook_url)
                .json(&payload)
                .send()
                .await?;
            
            tracing::info!("Sent webhook alert for rule: {}", rule_name);
        } else {
            tracing::warn!("Webhook URL not configured, cannot send alert");
        }
        Ok(())
    }
}
