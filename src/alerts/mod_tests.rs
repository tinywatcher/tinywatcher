#[cfg(test)]
mod tests {
    use crate::alerts::{AlertHandler, AlertManager};
    use anyhow::Result;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use async_trait::async_trait;

    // Mock alert handler for testing
    struct MockAlertHandler {
        name: String,
        call_count: Arc<AtomicUsize>,
    }

    impl MockAlertHandler {
        fn new(name: &str) -> (Self, Arc<AtomicUsize>) {
            let call_count = Arc::new(AtomicUsize::new(0));
            (
                Self {
                    name: name.to_string(),
                    call_count: call_count.clone(),
                },
                call_count,
            )
        }
    }

    #[async_trait]
    impl AlertHandler for MockAlertHandler {
        async fn send(&self, _identity: &str, _rule_name: &str, _message: &str) -> Result<()> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    // Mock failing alert handler
    struct FailingAlertHandler {
        name: String,
    }

    #[async_trait]
    impl AlertHandler for FailingAlertHandler {
        async fn send(&self, _identity: &str, _rule_name: &str, _message: &str) -> Result<()> {
            Err(anyhow::anyhow!("Simulated failure"))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_alert_manager_register() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler, _) = MockAlertHandler::new("test-alert");
        
        manager.register("test-alert".to_string(), Arc::new(handler));
        
        assert_eq!(manager.handlers.len(), 1);
        assert!(manager.handlers.contains_key("test-alert"));
    }

    #[tokio::test]
    async fn test_alert_manager_send_alert() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler, call_count) = MockAlertHandler::new("test-alert");
        
        manager.register("test-alert".to_string(), Arc::new(handler));
        
        // Send an alert
        let result = manager
            .send_alert("test-alert", "test-rule", "test message", 60)
            .await;
        
        assert!(result.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_alert_manager_send_alert_not_found() {
        let manager = AlertManager::new("test-server".to_string());
        
        // Try to send to non-existent alert
        let result = manager
            .send_alert("nonexistent", "test-rule", "test message", 60)
            .await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_alert_manager_cooldown() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler, call_count) = MockAlertHandler::new("test-alert");
        
        manager.register("test-alert".to_string(), Arc::new(handler));
        
        // Send first alert
        manager
            .send_alert("test-alert", "test-rule", "message 1", 5)
            .await
            .unwrap();
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        
        // Try to send second alert immediately (should be blocked by cooldown)
        manager
            .send_alert("test-alert", "test-rule", "message 2", 5)
            .await
            .unwrap();
        
        // Should still be 1 because of cooldown
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_alert_manager_cooldown_expired() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler, call_count) = MockAlertHandler::new("test-alert");
        
        manager.register("test-alert".to_string(), Arc::new(handler));
        
        // Send first alert with 1 second cooldown
        manager
            .send_alert("test-alert", "test-rule", "message 1", 1)
            .await
            .unwrap();
        
        assert_eq!(call_count.load(Ordering::SeqCst), 1);
        
        // Wait for cooldown to expire
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Send second alert (should go through)
        manager
            .send_alert("test-alert", "test-rule", "message 2", 1)
            .await
            .unwrap();
        
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_alert_manager_different_rules_no_cooldown_interference() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler, call_count) = MockAlertHandler::new("test-alert");
        
        manager.register("test-alert".to_string(), Arc::new(handler));
        
        // Send alert for rule1
        manager
            .send_alert("test-alert", "rule1", "message 1", 60)
            .await
            .unwrap();
        
        // Immediately send alert for rule2 (different rule, no cooldown)
        manager
            .send_alert("test-alert", "rule2", "message 2", 60)
            .await
            .unwrap();
        
        // Both should go through
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_alert_manager_send_alert_multi() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler1, count1) = MockAlertHandler::new("alert1");
        let (handler2, count2) = MockAlertHandler::new("alert2");
        let (handler3, count3) = MockAlertHandler::new("alert3");
        
        manager.register("alert1".to_string(), Arc::new(handler1));
        manager.register("alert2".to_string(), Arc::new(handler2));
        manager.register("alert3".to_string(), Arc::new(handler3));
        
        // Send to multiple alerts
        let alerts = vec![
            "alert1".to_string(),
            "alert2".to_string(),
            "alert3".to_string(),
        ];
        
        manager
            .send_alert_multi(&alerts, "test-rule", "test message", 60)
            .await
            .unwrap();
        
        // All three should have been called
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
        assert_eq!(count3.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_alert_manager_send_alert_multi_with_failure() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler1, count1) = MockAlertHandler::new("alert1");
        let failing = FailingAlertHandler {
            name: "failing".to_string(),
        };
        let (handler2, count2) = MockAlertHandler::new("alert2");
        
        manager.register("alert1".to_string(), Arc::new(handler1));
        manager.register("failing".to_string(), Arc::new(failing));
        manager.register("alert2".to_string(), Arc::new(handler2));
        
        let alerts = vec![
            "alert1".to_string(),
            "failing".to_string(),
            "alert2".to_string(),
        ];
        
        // Should not fail even though one handler fails
        let result = manager
            .send_alert_multi(&alerts, "test-rule", "test message", 60)
            .await;
        
        assert!(result.is_ok());
        
        // First and third should have been called despite second failing
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_alert_manager_send_alert_multi_cooldown() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler1, count1) = MockAlertHandler::new("alert1");
        let (handler2, count2) = MockAlertHandler::new("alert2");
        
        manager.register("alert1".to_string(), Arc::new(handler1));
        manager.register("alert2".to_string(), Arc::new(handler2));
        
        let alerts = vec!["alert1".to_string(), "alert2".to_string()];
        
        // Send first multi-alert
        manager
            .send_alert_multi(&alerts, "test-rule", "message 1", 5)
            .await
            .unwrap();
        
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
        
        // Try to send again immediately (should be blocked by cooldown)
        manager
            .send_alert_multi(&alerts, "test-rule", "message 2", 5)
            .await
            .unwrap();
        
        // Counts should still be 1 (cooldown blocked both)
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_alert_manager_send_alert_multi_not_found() {
        let mut manager = AlertManager::new("test-server".to_string());
        let (handler, _) = MockAlertHandler::new("alert1");
        
        manager.register("alert1".to_string(), Arc::new(handler));
        
        let alerts = vec!["alert1".to_string(), "nonexistent".to_string()];
        
        // Should fail when one alert doesn't exist
        let result = manager
            .send_alert_multi(&alerts, "test-rule", "test message", 60)
            .await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_alert_manager_identity() {
        let manager = AlertManager::new("prod-server-1".to_string());
        assert_eq!(manager.identity, "prod-server-1");
    }

    #[tokio::test]
    async fn test_alert_manager_multiple_registrations() {
        let mut manager = AlertManager::new("test-server".to_string());
        
        for i in 1..=10 {
            let (handler, _) = MockAlertHandler::new(&format!("alert{}", i));
            manager.register(format!("alert{}", i), Arc::new(handler));
        }
        
        assert_eq!(manager.handlers.len(), 10);
    }
}
