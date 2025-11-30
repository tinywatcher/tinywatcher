use super::AlertHandler;
use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

pub struct PagerDutyAlert {
    name: String,
    routing_key: String,
    client: reqwest::Client,
}

impl PagerDutyAlert {
    pub fn new(name: String, routing_key: String) -> Self {
        Self {
            name,
            routing_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AlertHandler for PagerDutyAlert {
    async fn send(&self, identity: &str, rule_name: &str, message: &str) -> Result<()> {
        let url = "https://events.pagerduty.com/v2/enqueue";
        
        let payload = json!({
            "routing_key": self.routing_key,
            "event_action": "trigger",
            "payload": {
                "summary": format!("TinyWatcher Alert: {} on {}", rule_name, identity),
                "severity": "error",
                "source": identity,
                "component": "TinyWatcher",
                "group": rule_name,
                "custom_details": {
                    "message": message,
                    "alert_name": self.name,
                    "rule": rule_name
                }
            }
        });

        self.client
            .post(url)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;
        
        tracing::info!("Sent PagerDuty alert '{}' for rule: {} (from {})", self.name, rule_name, identity);
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
