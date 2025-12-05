use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;
use tracing;

/// Response from heartbeat endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct HeartbeatResponse {
    pub status: String,
    pub message: String,
    pub next_ping_in: Option<u64>,
    pub watcher_name: Option<String>,
}

pub struct HeartbeatMonitor {
    url: String,
    interval_secs: u64,
    identity: String,
    client: reqwest::Client,
}

impl HeartbeatMonitor {
    pub fn new(url: String, interval_secs: u64, identity: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            url,
            interval_secs,
            identity,
            client,
        }
    }

    /// Start the heartbeat monitoring loop
    pub async fn start(self) {
        let mut ticker = interval(Duration::from_secs(self.interval_secs));
        
        tracing::info!(
            "Starting heartbeat monitoring (interval: {}s, url: {})", 
            self.interval_secs, 
            self.url
        );

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.send_heartbeat().await {
                tracing::warn!("Failed to send heartbeat: {}", e);
            }
        }
    }

    /// Send a heartbeat ping to the configured endpoint
    async fn send_heartbeat(&self) -> Result<()> {
        tracing::debug!("Sending heartbeat to {}", self.url);

        let response = self.client
            .post(&self.url)
            .json(&serde_json::json!({
                "watcher_name": self.identity,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
            .send()
            .await
            .context("Failed to send heartbeat request")?;

        let status = response.status();
        
        if !status.is_success() {
            tracing::warn!(
                "Heartbeat endpoint returned non-success status: {} ({})", 
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            );
            
            // Still try to read the response body for debugging
            if let Ok(text) = response.text().await {
                tracing::debug!("Response body: {}", text);
            }
            
            anyhow::bail!("Heartbeat failed with status {}", status);
        }

        // Try to parse the response
        let response_text = response.text().await
            .context("Failed to read heartbeat response")?;

        match serde_json::from_str::<HeartbeatResponse>(&response_text) {
            Ok(heartbeat_response) => {
                tracing::debug!(
                    "Heartbeat sent successfully: {} (status: {})", 
                    heartbeat_response.message,
                    heartbeat_response.status
                );

                if let Some(next_ping) = heartbeat_response.next_ping_in {
                    tracing::debug!("Server recommends next ping in {}s", next_ping);
                }

                if let Some(watcher_name) = heartbeat_response.watcher_name {
                    if watcher_name != self.identity {
                        tracing::debug!(
                            "Server recorded watcher name: {} (local: {})",
                            watcher_name,
                            self.identity
                        );
                    }
                }
            }
            Err(e) => {
                // If we can't parse the JSON, that's okay - the ping was recorded
                tracing::debug!("Could not parse heartbeat response as JSON: {}", e);
                tracing::debug!("Response body: {}", response_text);
                tracing::info!("Heartbeat sent successfully (non-JSON response)");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heartbeat_monitor_creation() {
        let monitor = HeartbeatMonitor::new(
            "https://example.com/ping".to_string(),
            60,
            "test-watcher".to_string(),
        );

        assert_eq!(monitor.url, "https://example.com/ping");
        assert_eq!(monitor.interval_secs, 60);
        assert_eq!(monitor.identity, "test-watcher");
    }

    #[test]
    fn test_heartbeat_response_deserialization() {
        let json = r#"{
            "status": "ok",
            "message": "Heartbeat recorded",
            "next_ping_in": 60,
            "watcher_name": "production-server-1"
        }"#;

        let response: HeartbeatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "ok");
        assert_eq!(response.message, "Heartbeat recorded");
        assert_eq!(response.next_ping_in, Some(60));
        assert_eq!(response.watcher_name, Some("production-server-1".to_string()));
    }

    #[test]
    fn test_heartbeat_response_minimal() {
        let json = r#"{
            "status": "ok",
            "message": "Ping received"
        }"#;

        let response: HeartbeatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "ok");
        assert_eq!(response.message, "Ping received");
        assert_eq!(response.next_ping_in, None);
        assert_eq!(response.watcher_name, None);
    }
}
