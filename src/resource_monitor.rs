use crate::alerts::AlertManager;
use crate::config::ResourceConfig;
use std::sync::Arc;
use sysinfo::{System, Disks};
use tokio::time::{interval, Duration};

pub struct ResourceMonitor {
    config: ResourceConfig,
    alert_manager: Arc<AlertManager>,
}

impl ResourceMonitor {
    pub fn new(config: ResourceConfig, alert_manager: Arc<AlertManager>) -> Self {
        Self {
            config,
            alert_manager,
        }
    }

    pub async fn start(&self) {
        let mut interval = interval(Duration::from_secs(self.config.interval));
        let mut sys = System::new_all();
        
        tracing::info!(
            "Starting resource monitoring (interval: {}s)",
            self.config.interval
        );

        loop {
            interval.tick().await;
            sys.refresh_all();

            self.check_cpu(&sys).await;
            self.check_memory(&sys).await;
            self.check_disk().await;
        }
    }

    async fn check_cpu(&self, sys: &System) {
        if let Some(threshold) = self.config.thresholds.cpu_percent {
            let cpu_usage = sys.global_cpu_usage();
            
            if cpu_usage > threshold {
                let message = format!(
                    "CPU usage is {}% (threshold: {}%)",
                    cpu_usage, threshold
                );
                
                if let Err(e) = self
                    .alert_manager
                    .send_alert_multi(
                        &self.config.thresholds.alert,
                        "cpu_threshold",
                        &message,
                        self.config.interval * 6, // 6x interval cooldown
                    )
                    .await
                {
                    tracing::error!("Failed to send CPU alert: {}", e);
                }
            }
        }
    }

    async fn check_memory(&self, sys: &System) {
        if let Some(threshold) = self.config.thresholds.memory_percent {
            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let memory_percent = (used_memory as f32 / total_memory as f32) * 100.0;
            
            if memory_percent > threshold {
                let message = format!(
                    "Memory usage is {:.1}% (threshold: {}%)",
                    memory_percent, threshold
                );
                
                if let Err(e) = self
                    .alert_manager
                    .send_alert_multi(
                        &self.config.thresholds.alert,
                        "memory_threshold",
                        &message,
                        self.config.interval * 6,
                    )
                    .await
                {
                    tracing::error!("Failed to send memory alert: {}", e);
                }
            }
        }
    }

    async fn check_disk(&self) {
        if let Some(threshold) = self.config.thresholds.disk_percent {
            let disks = Disks::new_with_refreshed_list();
            
            for disk in &disks {
                let total_space = disk.total_space();
                let available_space = disk.available_space();
                
                if total_space == 0 {
                    continue;
                }
                
                let used_percent = ((total_space - available_space) as f32 / total_space as f32) * 100.0;
                
                if used_percent > threshold {
                    let message = format!(
                        "Disk usage on {} is {:.1}% (threshold: {}%)",
                        disk.mount_point().display(),
                        used_percent,
                        threshold
                    );
                    
                    if let Err(e) = self
                        .alert_manager
                        .send_alert_multi(
                            &self.config.thresholds.alert,
                            "disk_threshold",
                            &message,
                            self.config.interval * 6,
                        )
                        .await
                    {
                        tracing::error!("Failed to send disk alert: {}", e);
                    }
                }
            }
        }
    }
}
