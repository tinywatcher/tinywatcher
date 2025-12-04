use super::AlertHandler;
use async_trait::async_trait;
use anyhow::{Result, Context};
use serde_json::json;

pub struct SendGridAlert {
    name: String,
    api_key: String,
    from: String,
    to: Vec<String>,
}

impl SendGridAlert {
    pub fn new(name: String, api_key: String, from: String, to: Vec<String>) -> Self {
        tracing::info!(
            "Created SendGrid alert '{}' - from: {}, to: {:?}",
            name, from, to
        );
        Self {
            name,
            api_key,
            from,
            to,
        }
    }
}

#[async_trait]
impl AlertHandler for SendGridAlert {
    async fn send(&self, identity: &str, rule_name: &str, message: &str) -> Result<()> {
        tracing::info!(
            "SendGrid alert '{}' triggered for rule '{}' - sending to {} recipient(s)",
            self.name, rule_name, self.to.len()
        );
        
        let subject = format!("TinyWatcher Alert: {} ({})", rule_name, identity);
        let body = format!(
            "TinyWatcher Alert\n\
             =================\n\n\
             Host: {}\n\
             Rule: {}\n\
             Time: {}\n\n\
             Message:\n\
             {}\n",
            identity,
            rule_name,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            message
        );

        // Build personalizations for each recipient
        let personalizations: Vec<_> = self.to.iter().map(|email| {
            json!({
                "to": [{"email": email}]
            })
        }).collect();

        // Build SendGrid API request
        let payload = json!({
            "personalizations": personalizations,
            "from": {"email": self.from},
            "subject": subject,
            "content": [{
                "type": "text/plain",
                "value": body
            }]
        });

        tracing::debug!("Sending SendGrid API request");
        
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send SendGrid API request")?;

        if response.status().is_success() {
            tracing::info!(
                "✅ Successfully sent SendGrid alert '{}' to {} recipient(s) for rule: {}",
                self.name, self.to.len(), rule_name
            );
            Ok(())
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!(
                "❌ SendGrid API request failed with status {}: {}",
                status, error_body
            );
            Err(anyhow::anyhow!(
                "SendGrid API request failed with status {}: {}",
                status, error_body
            ))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}
