use super::AlertHandler;
use async_trait::async_trait;
use anyhow::Result;

pub struct NtfyAlert {
    name: String,
    topic: String,
    server: String,
    client: reqwest::Client,
}

impl NtfyAlert {
    pub fn new(name: String, topic: String, server: Option<String>) -> Self {
        Self {
            name,
            topic,
            server: server.unwrap_or_else(|| "https://ntfy.sh".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AlertHandler for NtfyAlert {
    async fn send(&self, identity: &str, rule_name: &str, message: &str) -> Result<()> {
        let url = format!("{}/{}", self.server, self.topic);
        
        let body = format!(
            "Alert: {}\nHost: {}\n\n{}",
            rule_name, identity, message
        );
        
        self.client
            .post(&url)
            .header("Title", format!("TinyWatcher: {}", rule_name))
            .header("Tags", "rotating_light,warning")
            .header("Priority", "high")
            .body(body)
            .send()
            .await?
            .error_for_status()?;
        
        tracing::info!("Sent Ntfy alert '{}' for rule: {} (from {})", self.name, rule_name, identity);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
