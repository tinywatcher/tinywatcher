use super::AlertHandler;
use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

pub struct DiscordAlert {
    name: String,
    webhook_url: String,
    client: reqwest::Client,
}

impl DiscordAlert {
    pub fn new(name: String, webhook_url: String) -> Self {
        Self {
            name,
            webhook_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AlertHandler for DiscordAlert {
    async fn send(&self, identity: &str, rule_name: &str, message: &str) -> Result<()> {
        let payload = json!({
            "embeds": [{
                "title": format!("🚨 Alert: {}", rule_name),
                "description": message,
                "color": 15158332, // Red color
                "fields": [
                    {
                        "name": "Host",
                        "value": format!("`{}`", identity),
                        "inline": true
                    }
                ],
                "footer": {
                    "text": "TinyWatcher"
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            }]
        });

        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        
        tracing::info!("Sent Discord alert '{}' for rule: {} (from {})", self.name, rule_name, identity);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
