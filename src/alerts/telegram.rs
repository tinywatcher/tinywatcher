use super::AlertHandler;
use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

pub struct TelegramAlert {
    name: String,
    bot_token: String,
    chat_id: String,
    client: reqwest::Client,
}

impl TelegramAlert {
    pub fn new(name: String, bot_token: String, chat_id: String) -> Self {
        Self {
            name,
            bot_token,
            chat_id,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AlertHandler for TelegramAlert {
    async fn send(&self, identity: &str, rule_name: &str, message: &str) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        
        let text = format!(
            "🚨 *Alert: {}*\n\n*Host:* `{}`\n\n```\n{}\n```",
            rule_name, identity, message
        );
        
        let payload = json!({
            "chat_id": self.chat_id,
            "text": text,
            "parse_mode": "Markdown"
        });

        self.client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        
        tracing::info!("Sent Telegram alert '{}' for rule: {} (from {})", self.name, rule_name, identity);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
