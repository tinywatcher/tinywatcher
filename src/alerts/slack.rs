use super::AlertHandler;
use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

pub struct SlackAlert {
    name: String,
    webhook_url: String,
    client: reqwest::Client,
}

impl SlackAlert {
    pub fn new(name: String, webhook_url: String) -> Self {
        Self {
            name,
            webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AlertHandler for SlackAlert {
    async fn send(&self, identity: &str, rule_name: &str, message: &str) -> Result<()> {
        let payload = json!({
            "text": format!("🚨 *Alert: {}*\n*Host:* `{}`\n```{}```", rule_name, identity, message),
            "username": "TinyWatcher",
            "icon_emoji": ":eyes:"
        });

        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        tracing::info!("Sent Slack alert '{}' for rule: {} (from {})", self.name, rule_name, identity);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
