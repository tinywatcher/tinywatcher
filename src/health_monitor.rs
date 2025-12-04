use crate::alerts::AlertManager;
use crate::config::Threshold;
use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::interval;

/// Initial retry delay
const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(5);

/// Maximum retry delay
const MAX_RETRY_DELAY: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub check_type: HealthCheckType,
    pub url: String,
    pub interval: u64,         // seconds between checks
    pub timeout_secs: u64,     // request timeout
    pub missed_threshold: u32, // how many failures before alert
    pub alert: Vec<String>,    // alert names to trigger
    pub threshold: Option<Threshold>, // optional rate-based threshold (e.g., "3 in 1m")
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
                Self::run_health_check_with_retry(check, alert_manager, identity).await;
            }));
        }

        // Wait for all tasks (they run indefinitely)
        for task in tasks {
            let _ = task.await;
        }
    }

    /// Wrapper that retries health checks if they fail or exit unexpectedly
    async fn run_health_check_with_retry(
        check: HealthCheck,
        alert_manager: Arc<AlertManager>,
        identity: String,
    ) {
        let mut retry_delay = INITIAL_RETRY_DELAY;
        
        loop {
            tracing::info!("Starting health check task: {}", check.name);
            
            // Run the health check (this should run indefinitely)
            Self::run_health_check(check.clone(), alert_manager.clone(), identity.clone()).await;
            
            // If we get here, the check loop exited unexpectedly
            tracing::error!(
                "Health check '{}' exited unexpectedly. Retrying in {:?}...",
                check.name,
                retry_delay
            );
            
            tokio::time::sleep(retry_delay).await;
            retry_delay = (retry_delay * 2).min(MAX_RETRY_DELAY);
        }
    }

    async fn run_health_check(
        check: HealthCheck,
        alert_manager: Arc<AlertManager>,
        identity: String,
    ) {
        let threshold_info = if let Some(ref t) = check.threshold {
            format!("threshold: {} in {:?}", t.count, t.window)
        } else {
            format!("threshold: {} consecutive", check.missed_threshold)
        };

        tracing::info!(
            "Starting health check '{}' for {} (interval: {}s, timeout: {}s, {})",
            check.name,
            check.url,
            check.interval,
            check.timeout_secs,
            threshold_info
        );

        let mut interval_timer = interval(Duration::from_secs(check.interval));
        let mut consecutive_failures = 0u32;
        let mut is_down = false;
        
        // For sliding window threshold tracking
        let failure_history: Arc<Mutex<VecDeque<Instant>>> = Arc::new(Mutex::new(VecDeque::new()));

        loop {
            interval_timer.tick().await;

            match Self::perform_check(&check).await {
                Ok(()) => {
                    // Check succeeded
                    if is_down {
                        // Service recovered
                        tracing::info!("Health check '{}' recovered: {}", check.name, check.url);
                        
                        let message = format!(
                            "Service '{}' is back UP\n\
                            Identity: {}\n\
                            URL: {}\n\
                            Status: Healthy",
                            check.name,
                            identity,
                            check.url
                        );

                        if let Err(e) = alert_manager.send_alert_multi(&check.alert, &check.name, &message, 0).await {
                            tracing::error!("Failed to send recovery alert for '{}': {}", check.name, e);
                        }
                        is_down = false;
                    }
                    consecutive_failures = 0;
                    
                    // Clear failure history on success
                    if check.threshold.is_some() {
                        failure_history.lock().await.clear();
                    }
                    
                    tracing::debug!("Health check '{}' passed", check.name);
                }
                Err(e) => {
                    // Check failed
                    consecutive_failures += 1;
                    
                    // Determine if we should alert based on threshold type
                    let should_alert = if let Some(ref threshold) = check.threshold {
                        // Use sliding window threshold
                        Self::check_failure_threshold(&failure_history, threshold, &check.name).await
                    } else {
                        // Use consecutive failure threshold
                        consecutive_failures >= check.missed_threshold
                    };
                    
                    tracing::warn!(
                        "Health check '{}' failed: {}",
                        check.name,
                        e
                    );

                    if should_alert && !is_down {
                        // Threshold reached, send alert
                        is_down = true;
                        
                        let failure_info = if let Some(ref threshold) = check.threshold {
                            format!("{} failures in {:?}", threshold.count, threshold.window)
                        } else {
                            format!("{} consecutive failures", consecutive_failures)
                        };
                        
                        let message = format!(
                            "Service '{}' is DOWN\n\
                            Identity: {}\n\
                            URL: {}\n\
                            Threshold: {}\n\
                            Error: {}",
                            check.name,
                            identity,
                            check.url,
                            failure_info,
                            e
                        );

                        if let Err(e) = alert_manager.send_alert_multi(&check.alert, &check.name, &message, 0).await {
                            tracing::error!("Failed to send alert for '{}': {}", check.name, e);
                        }
                    }
                }
            }
        }
    }

    /// Check if failure threshold is exceeded using sliding window
    /// Returns true if we should send an alert
    async fn check_failure_threshold(
        failure_history: &Arc<Mutex<VecDeque<Instant>>>,
        threshold: &Threshold,
        check_name: &str,
    ) -> bool {
        let now = Instant::now();
        let mut history = failure_history.lock().await;
        
        // Add current failure
        history.push_back(now);
        
        // Remove old failures outside the time window
        let cutoff = now - threshold.window;
        while let Some(&oldest) = history.front() {
            if oldest < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }
        
        // Check if threshold is exceeded
        let count = history.len();
        if count >= threshold.count as usize {
            tracing::info!(
                "Health check threshold exceeded for '{}': {} failures in {:?}",
                check_name,
                count,
                threshold.window
            );
            // Clear history after alerting to avoid repeated alerts
            history.clear();
            true
        } else {
            tracing::debug!(
                "Health check '{}' failed but threshold not reached: {}/{} in {:?}",
                check_name,
                count,
                threshold.count,
                threshold.window
            );
            false
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
            .connect_timeout(Duration::from_secs(5))
            .tcp_keepalive(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .context("Failed to create HTTP client")?;

        let response = client
            .get(&check.url)
            .send()
            .await
            .context("HTTP request failed")?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("HTTP status: {}", response.status())
        }
    }
}

#[cfg(test)]
#[path = "health_monitor_tests.rs"]
mod tests;
