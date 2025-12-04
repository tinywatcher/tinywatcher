#[cfg(test)]
mod tests {
    use crate::log_monitor::LogMonitor;
    use crate::alerts::{AlertHandler, AlertManager};
    use crate::config::{Rule, RuleSources, SourceType};
    use anyhow::Result;
    use async_trait::async_trait;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

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

    fn create_test_monitor() -> (LogMonitor, Arc<AtomicUsize>, Arc<tokio::sync::Mutex<String>>) {
        let mut alert_manager = AlertManager::new("test-server".to_string());
        let (handler, call_count, last_message) = MockAlertHandler::new("test-alert");
        alert_manager.register("test-alert".to_string(), Arc::new(handler));
        
        let rules = vec![Rule {
            name: "error_rule".to_string(),
            text: Some("ERROR".to_string()),
            pattern: None,
            alert: vec!["test-alert".to_string()],
            cooldown: 1,
            sources: None,
        }];
        
        let monitor = LogMonitor::new(rules, Arc::new(alert_manager)).unwrap();
        (monitor, call_count, last_message)
    }

    #[tokio::test]
    async fn test_log_monitor_creation() {
        let alert_manager = Arc::new(AlertManager::new("test".to_string()));
        let rules = vec![Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        }];
        
        let result = LogMonitor::new(rules, alert_manager);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_log_monitor_invalid_regex() {
        let alert_manager = Arc::new(AlertManager::new("test".to_string()));
        let rules = vec![Rule {
            name: "test".to_string(),
            text: None,
            pattern: Some("[invalid regex(".to_string()),
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        }];
        
        let result = LogMonitor::new(rules, alert_manager);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_line_text_match() {
        let (monitor, call_count, last_message) = create_test_monitor();
        
        let source = SourceType::Container("test".to_string());
        monitor.process_line("This is an ERROR message", &source).await;
        
        // Give it a moment to process
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        let msg = last_message.lock().await;
        assert_eq!(*msg, "This is an ERROR message");
    }

    #[tokio::test]
    async fn test_process_line_text_no_match() {
        let (monitor, call_count, _) = create_test_monitor();
        
        let source = SourceType::Container("test".to_string());
        monitor.process_line("This is a normal message", &source).await;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        assert_eq!(call_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_process_line_regex_match() {
        let mut alert_manager = AlertManager::new("test-server".to_string());
        let (handler, call_count, last_message) = MockAlertHandler::new("test-alert");
        alert_manager.register("test-alert".to_string(), Arc::new(handler));
        
        let rules = vec![Rule {
            name: "error_or_warn".to_string(),
            text: None,
            pattern: Some("ERROR|WARN".to_string()),
            alert: vec!["test-alert".to_string()],
            cooldown: 1,
            sources: None,
        }];
        
        let monitor = LogMonitor::new(rules, Arc::new(alert_manager)).unwrap();
        let source = SourceType::File(PathBuf::from("/var/log/app.log"));
        
        monitor.process_line("This is a WARN message", &source).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        let msg = last_message.lock().await;
        assert_eq!(*msg, "This is a WARN message");
    }

    #[tokio::test]
    async fn test_process_line_case_insensitive() {
        let mut alert_manager = AlertManager::new("test-server".to_string());
        let (handler, call_count, _) = MockAlertHandler::new("test-alert");
        alert_manager.register("test-alert".to_string(), Arc::new(handler));
        
        let rules = vec![Rule {
            name: "error_rule".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["test-alert".to_string()],
            cooldown: 1,
            sources: None,
        }];
        
        let monitor = LogMonitor::new(rules, Arc::new(alert_manager)).unwrap();
        let source = SourceType::Container("test".to_string());
        
        // Text matching is case-sensitive by default in Rust
        monitor.process_line("This is an ERROR message", &source).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Should not match because "error" != "ERROR"
        assert_eq!(call_count.load(Ordering::SeqCst), 0);
        
        // But should match exact case
        monitor.process_line("This is an error message", &source).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_rule_applies_to_source_no_filter() {
        let alert_manager = Arc::new(AlertManager::new("test".to_string()));
        let rules = vec![Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        }];
        
        let monitor = LogMonitor::new(rules, alert_manager).unwrap();
        let rule = &monitor.rules[0];
        
        // Should apply to all sources when no filter
        assert!(monitor.rule_applies_to_source(rule, &SourceType::File(PathBuf::from("/var/log/app.log"))));
        assert!(monitor.rule_applies_to_source(rule, &SourceType::Container("nginx".to_string())));
        assert!(monitor.rule_applies_to_source(rule, &SourceType::Stream("websocket".to_string())));
    }

    #[tokio::test]
    async fn test_rule_applies_to_source_file_filter() {
        let alert_manager = Arc::new(AlertManager::new("test".to_string()));
        let rules = vec![Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: Some(RuleSources {
                files: vec![PathBuf::from("/var/log/app.log")],
                containers: vec![],
                streams: vec![],
            }),
        }];
        
        let monitor = LogMonitor::new(rules, alert_manager).unwrap();
        let rule = &monitor.rules[0];
        
        // Should match specified file
        assert!(monitor.rule_applies_to_source(rule, &SourceType::File(PathBuf::from("/var/log/app.log"))));
        
        // Should not match other files
        assert!(!monitor.rule_applies_to_source(rule, &SourceType::File(PathBuf::from("/var/log/other.log"))));
        
        // Should not match containers or streams
        assert!(!monitor.rule_applies_to_source(rule, &SourceType::Container("nginx".to_string())));
        assert!(!monitor.rule_applies_to_source(rule, &SourceType::Stream("websocket".to_string())));
    }

    #[tokio::test]
    async fn test_rule_applies_to_source_container_filter() {
        let alert_manager = Arc::new(AlertManager::new("test".to_string()));
        let rules = vec![Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: Some(RuleSources {
                files: vec![],
                containers: vec!["nginx".to_string(), "api".to_string()],
                streams: vec![],
            }),
        }];
        
        let monitor = LogMonitor::new(rules, alert_manager).unwrap();
        let rule = &monitor.rules[0];
        
        // Should match specified containers
        assert!(monitor.rule_applies_to_source(rule, &SourceType::Container("nginx".to_string())));
        assert!(monitor.rule_applies_to_source(rule, &SourceType::Container("api".to_string())));
        
        // Should not match other containers
        assert!(!monitor.rule_applies_to_source(rule, &SourceType::Container("postgres".to_string())));
    }

    #[tokio::test]
    async fn test_multiple_rules() {
        let mut alert_manager = AlertManager::new("test-server".to_string());
        let (handler1, count1, _) = MockAlertHandler::new("alert1");
        let (handler2, count2, _) = MockAlertHandler::new("alert2");
        alert_manager.register("alert1".to_string(), Arc::new(handler1));
        alert_manager.register("alert2".to_string(), Arc::new(handler2));
        
        let rules = vec![
            Rule {
                name: "error_rule".to_string(),
                text: Some("ERROR".to_string()),
                pattern: None,
                alert: vec!["alert1".to_string()],
                cooldown: 1,
                sources: None,
            },
            Rule {
                name: "warn_rule".to_string(),
                text: Some("WARN".to_string()),
                pattern: None,
                alert: vec!["alert2".to_string()],
                cooldown: 1,
                sources: None,
            },
        ];
        
        let monitor = LogMonitor::new(rules, Arc::new(alert_manager)).unwrap();
        let source = SourceType::Container("test".to_string());
        
        // Should trigger first rule
        monitor.process_line("This is an ERROR", &source).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 0);
        
        // Should trigger second rule
        monitor.process_line("This is a WARN", &source).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
        
        // Wait for cooldown to expire (1 second cooldown + buffer)
        tokio::time::sleep(Duration::from_millis(1100)).await;
        
        // Should trigger both rules
        monitor.process_line("ERROR and WARN together", &source).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(count1.load(Ordering::SeqCst), 2);
        assert_eq!(count2.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_multi_destination_alert() {
        let mut alert_manager = AlertManager::new("test-server".to_string());
        let (handler1, count1, _) = MockAlertHandler::new("alert1");
        let (handler2, count2, _) = MockAlertHandler::new("alert2");
        alert_manager.register("alert1".to_string(), Arc::new(handler1));
        alert_manager.register("alert2".to_string(), Arc::new(handler2));
        
        let rules = vec![Rule {
            name: "critical".to_string(),
            text: Some("CRITICAL".to_string()),
            pattern: None,
            alert: vec!["alert1".to_string(), "alert2".to_string()],
            cooldown: 1,
            sources: None,
        }];
        
        let monitor = LogMonitor::new(rules, Arc::new(alert_manager)).unwrap();
        let source = SourceType::Container("test".to_string());
        
        monitor.process_line("CRITICAL error occurred", &source).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Both alerts should be triggered
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }
}
