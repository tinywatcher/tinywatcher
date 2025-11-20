use super::AlertHandler;
use async_trait::async_trait;
use anyhow::Result;
use chrono::Utc;

pub struct StdoutAlert {
    name: String,
}

impl StdoutAlert {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[async_trait]
impl AlertHandler for StdoutAlert {
    async fn send(&self, rule_name: &str, message: &str) -> Result<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        println!("[{}] ALERT [{}]: {}", timestamp, rule_name, message);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
