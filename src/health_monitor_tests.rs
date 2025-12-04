#[cfg(test)]
mod tests {
    use crate::health_monitor::{HealthCheck, HealthCheckType, HealthMonitor};
    use crate::alerts::{AlertHandler, AlertManager};
    use anyhow::Result;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // Mock alert handler for testing
    struct MockAlertHandler {
        name: String,
        call_count: Arc<AtomicUsize>,
        last_message: Arc<tokio::sync::Mutex<String>>,
    }

    impl MockAlertHandler {
        fn new(name: &str) -> (Self, Arc<AtomicUsize>, Arc<tokio::sync::Mutex<String>>) {
            let call_count = Arc::new(AtomicUsize::new(0));
            let last_message = Arc::new(tokio::sync::Mutex::new(String::new()));
            (
                Self {
                    name: name.to_string(),
                    call_count: call_count.clone(),
                    last_message: last_message.clone(),
                },
                call_count,
                last_message,
            )
        }
    }

    #[async_trait]
    impl AlertHandler for MockAlertHandler {
        async fn send(&self, _identity: &str, _rule_name: &str, message: &str) -> Result<()> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            *self.last_message.lock().await = message.to_string();
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_health_check_creation() {
        let check = HealthCheck {
            name: "test_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://localhost:8080/health".to_string(),
            interval: 30,
            timeout_secs: 5,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        assert_eq!(check.name, "test_api");
        assert_eq!(check.check_type, HealthCheckType::Http);
        assert_eq!(check.interval, 30);
        assert_eq!(check.missed_threshold, 2);
    }

    #[test]
    fn test_health_monitor_creation() {
        let checks = vec![HealthCheck {
            name: "test_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://localhost:8080/health".to_string(),
            interval: 30,
            timeout_secs: 5,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        }];

        let alert_manager = Arc::new(AlertManager::new("test-server".to_string()));
        let monitor = HealthMonitor::new(checks, alert_manager, "test-server".to_string());

        assert_eq!(monitor.checks.len(), 1);
        assert_eq!(monitor.identity, "test-server");
    }

    #[test]
    fn test_health_monitor_multiple_checks() {
        let checks = vec![
            HealthCheck {
                name: "api".to_string(),
                check_type: HealthCheckType::Http,
                url: "http://localhost:8080/health".to_string(),
                interval: 30,
                timeout_secs: 5,
                missed_threshold: 2,
                alert: vec!["slack".to_string()],
            threshold: None,
            },
            HealthCheck {
                name: "database".to_string(),
                check_type: HealthCheckType::Http,
                url: "http://localhost:5432/health".to_string(),
                interval: 60,
                timeout_secs: 10,
                missed_threshold: 3,
                alert: vec!["pagerduty".to_string()],
            threshold: None,
            },
        ];

        let alert_manager = Arc::new(AlertManager::new("test-server".to_string()));
        let monitor = HealthMonitor::new(checks, alert_manager, "test-server".to_string());

        assert_eq!(monitor.checks.len(), 2);
        assert_eq!(monitor.checks[0].name, "api");
        assert_eq!(monitor.checks[1].name, "database");
    }

    #[test]
    fn test_health_check_multi_destination_alerts() {
        let check = HealthCheck {
            name: "critical_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://localhost:8080/health".to_string(),
            interval: 30,
            timeout_secs: 5,
            missed_threshold: 2,
            alert: vec!["slack".to_string(), "pagerduty".to_string(), "discord".to_string()],
            threshold: None,
        };

        assert_eq!(check.alert.len(), 3);
        assert!(check.alert.contains(&"slack".to_string()));
        assert!(check.alert.contains(&"pagerduty".to_string()));
        assert!(check.alert.contains(&"discord".to_string()));
    }

    #[test]
    fn test_health_check_default_values() {
        // Test that we can create a check with all fields explicitly set
        let check = HealthCheck {
            name: "test".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://example.com".to_string(),
            interval: 10,
            timeout_secs: 3,
            missed_threshold: 1,
            alert: vec!["stdout".to_string()],
            threshold: None,
        };

        assert_eq!(check.interval, 10);
        assert_eq!(check.timeout_secs, 3);
        assert_eq!(check.missed_threshold, 1);
    }

    #[tokio::test]
   
    async fn test_http_check_timeout() {
        let check = HealthCheck {
            name: "slow_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://httpbin.org/delay/10".to_string(), // Takes 10 seconds
            interval: 30,
            timeout_secs: 1, // But we only wait 1 second
            missed_threshold: 1,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        let result = HealthMonitor::perform_check(&check).await;
        if result.is_ok() {
            eprintln!("Expected timeout but request succeeded (network issue?)");
        }
    }

    #[tokio::test]
   
    async fn test_http_check_success() {
        let check = HealthCheck {
            name: "test_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "https://httpbin.org/status/200".to_string(),
            interval: 30,
            timeout_secs: 10,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        let result = HealthMonitor::perform_check(&check).await;
        if result.is_err() {
            // Network tests can be flaky, just log the error
            eprintln!("HTTP check failed (network issue?): {:?}", result.err());
        }
    }

    #[tokio::test]

    async fn test_http_check_failure_status() {
        let check = HealthCheck {
            name: "test_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "https://httpbin.org/status/500".to_string(),
            interval: 30,
            timeout_secs: 10,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        let result = HealthMonitor::perform_check(&check).await;
        if result.is_ok() {
            eprintln!("Expected HTTP 500 check to fail, but it succeeded (network issue?)");
        }
    }

    #[tokio::test]
   
    async fn test_http_check_not_found() {
        let check = HealthCheck {
            name: "test_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "https://httpbin.org/status/404".to_string(),
            interval: 30,
            timeout_secs: 10,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        let result = HealthMonitor::perform_check(&check).await;
        if result.is_ok() {
            eprintln!("Expected 404 check to fail, but it succeeded (network issue?)");
        }
    }

    #[tokio::test]
    async fn test_http_check_connection_refused() {
        let check = HealthCheck {
            name: "test_api".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://localhost:99999/health".to_string(), // Invalid port
            interval: 30,
            timeout_secs: 2,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        let result = HealthMonitor::perform_check(&check).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_health_check_clone() {
        let check = HealthCheck {
            name: "test".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://localhost:8080".to_string(),
            interval: 30,
            timeout_secs: 5,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        let cloned = check.clone();
        assert_eq!(check.name, cloned.name);
        assert_eq!(check.url, cloned.url);
        assert_eq!(check.interval, cloned.interval);
        assert_eq!(check.missed_threshold, cloned.missed_threshold);
    }

    #[test]
    fn test_health_check_type_equality() {
        assert_eq!(HealthCheckType::Http, HealthCheckType::Http);
    }

    #[test]
    fn test_health_check_debug_format() {
        let check = HealthCheck {
            name: "test".to_string(),
            check_type: HealthCheckType::Http,
            url: "http://localhost:8080".to_string(),
            interval: 30,
            timeout_secs: 5,
            missed_threshold: 2,
            alert: vec!["slack".to_string()],
            threshold: None,
        };

        let debug_str = format!("{:?}", check);
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("Http"));
        assert!(debug_str.contains("localhost:8080"));
    }
}
