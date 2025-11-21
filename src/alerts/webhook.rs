use super::AlertHandler;
use async_trait::async_trait;
use anyhow::Result;
use chrono::Utc;
use serde_json::json;

pub struct WebhookAlert {
    name: String,
    webhook_url: String,
    client: reqwest::Client,
}

impl WebhookAlert {
    pub fn new(name: String, webhook_url: String) -> Self {
        Self {
            name,
            webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AlertHandler for WebhookAlert {
    async fn send(&self, identity: &str, rule_name: &str, message: &str) -> Result<()> {
        let payload = json!({
            "identity": identity,
            "rule": rule_name,
            "message": message,
            "timestamp": Utc::now().to_rfc3339(),
            "alert_name": self.name,
        });

        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        tracing::info!("Sent webhook alert '{}' for rule: {} (from {})", self.name, rule_name, identity);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
