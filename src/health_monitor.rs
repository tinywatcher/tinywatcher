use crate::alerts::AlertManager;
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, timeout};

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub check_type: HealthCheckType,
    pub url: String,
    pub interval: u64,         // seconds between checks
    pub timeout_secs: u64,     // request timeout
    pub missed_threshold: u32, // how many failures before alert
    pub alert: String,         // alert name to trigger
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthCheckType {
    Http,
    // Future: Tcp, Ping, etc.
}

pub struct HealthMonitor {
    checks: Vec<HealthCheck>,
    alert_manager: Arc<AlertManager>,
    identity: String,
}

impl HealthMonitor {
    pub fn new(checks: Vec<HealthCheck>, alert_manager: Arc<AlertManager>, identity: String) -> Self {
        Self {
            checks,
            alert_manager,
            identity,
        }
    }

    pub async fn start(self) {
        let mut tasks = Vec::new();

        for check in self.checks {
            let alert_manager = self.alert_manager.clone();
            let identity = self.identity.clone();
            
            tasks.push(tokio::spawn(async move {
                Self::run_health_check(check, alert_manager, identity).await;
            }));
        }

        // Wait for all tasks (they run indefinitely)
        for task in tasks {
            let _ = task.await;
        }
    }

    async fn run_health_check(
        check: HealthCheck,
        alert_manager: Arc<AlertManager>,
        identity: String,
    ) {
        tracing::info!(
            "Starting health check '{}' for {} (interval: {}s, timeout: {}s, threshold: {})",
            check.name,
            check.url,
            check.interval,
            check.timeout_secs,
            check.missed_threshold
        );

        let mut interval_timer = interval(Duration::from_secs(check.interval));
        let mut consecutive_failures = 0u32;
        let mut is_down = false;

        loop {
            interval_timer.tick().await;

            match Self::perform_check(&check).await {
                Ok(()) => {
                    // Check succeeded
                    if is_down {
                        // Service recovered
                        tracing::info!("Health check '{}' recovered: {}", check.name, check.url);
                        
                        let message = format!(
                            "✅ Service '{}' is back UP\n\
                            Identity: {}\n\
                            URL: {}\n\
                            Status: Healthy",
                            check.name,
                            identity,
                            check.url
                        );

                        if let Err(e) = alert_manager.send_alert(&check.alert, &check.name, &message, 0).await {
                            tracing::error!("Failed to send recovery alert for '{}': {}", check.name, e);
                        }

                        is_down = false;
                    }
                    consecutive_failures = 0;
                    tracing::debug!("Health check '{}' passed", check.name);
                }
                Err(e) => {
                    // Check failed
                    consecutive_failures += 1;
                    tracing::warn!(
                        "Health check '{}' failed ({}/{}): {}",
                        check.name,
                        consecutive_failures,
                        check.missed_threshold,
                        e
                    );

                    if consecutive_failures >= check.missed_threshold && !is_down {
                        // Threshold reached, send alert
                        is_down = true;
                        
                        let message = format!(
                            "🚨 Service '{}' is DOWN\n\
                            Identity: {}\n\
                            URL: {}\n\
                            Failed checks: {}\n\
                            Error: {}",
                            check.name,
                            identity,
                            check.url,
                            consecutive_failures,
                            e
                        );

                        if let Err(e) = alert_manager.send_alert(&check.alert, &check.name, &message, 0).await {
                            tracing::error!("Failed to send alert for '{}': {}", check.name, e);
                        }
                    }
                }
            }
        }
    }

    async fn perform_check(check: &HealthCheck) -> Result<()> {
        match check.check_type {
            HealthCheckType::Http => Self::http_check(check).await,
        }
    }

    async fn http_check(check: &HealthCheck) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(check.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        let response = timeout(
            Duration::from_secs(check.timeout_secs),
            client.get(&check.url).send(),
        )
        .await
        .context("HTTP request timed out")?
        .context("HTTP request failed")?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("HTTP status: {}", response.status())
        }
    }
}
