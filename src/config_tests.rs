#[cfg(test)]
mod tests {
    use super::super::*;
    use std::env;
    use std::path::PathBuf;

    #[test]
    fn test_identity_default() {
        let identity = Identity::default();
        assert!(identity.name.is_none());
        
        // get_name should return hostname when name is None
        let name = identity.get_name();
        assert!(!name.is_empty());
        assert_ne!(name, "unknown"); // Should have a real hostname in test env
    }

    #[test]
    fn test_identity_custom_name() {
        let identity = Identity {
            name: Some("test-server-1".to_string()),
        };
        assert_eq!(identity.get_name(), "test-server-1");
    }

    #[test]
    fn test_rule_validate_no_pattern_or_text() {
        let rule = Rule {
            name: "test".to_string(),
            text: None,
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        };
        
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_rule_validate_both_pattern_and_text() {
        let rule = Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: Some("ERROR".to_string()),
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        };
        
        assert!(rule.validate().is_err());
    }

    #[test]
    fn test_rule_validate_text_only() {
        let rule = Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        };
        
        assert!(rule.validate().is_ok());
    }

    #[test]
    fn test_rule_validate_pattern_only() {
        let rule = Rule {
            name: "test".to_string(),
            text: None,
            pattern: Some("ERROR|WARN".to_string()),
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        };
        
        assert!(rule.validate().is_ok());
    }

    #[test]
    fn test_rule_match_type_text() {
        let rule = Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        };
        
        match rule.match_type() {
            MatchType::Text(text) => assert_eq!(text, "error"),
            _ => panic!("Expected Text match type"),
        }
    }

    #[test]
    fn test_rule_match_type_regex() {
        let rule = Rule {
            name: "test".to_string(),
            text: None,
            pattern: Some("ERROR|WARN".to_string()),
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        };
        
        match rule.match_type() {
            MatchType::Regex(pattern) => assert_eq!(pattern, "ERROR|WARN"),
            _ => panic!("Expected Regex match type"),
        }
    }

    #[test]
    fn test_rule_applies_to_source_no_filter() {
        let rule = Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: None,
        };
        
        // Should apply to all sources when no filter is specified
        assert!(rule.applies_to_source(&SourceType::File(PathBuf::from("/var/log/app.log"))));
        assert!(rule.applies_to_source(&SourceType::Container("nginx".to_string())));
        assert!(rule.applies_to_source(&SourceType::Stream("websocket".to_string())));
    }

    #[test]
    fn test_rule_applies_to_source_file_filter() {
        let rule = Rule {
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
        };
        
        // Should match the specified file
        assert!(rule.applies_to_source(&SourceType::File(PathBuf::from("/var/log/app.log"))));
        
        // Should not match other files
        assert!(!rule.applies_to_source(&SourceType::File(PathBuf::from("/var/log/other.log"))));
        
        // Should not match containers or streams (empty filter)
        assert!(!rule.applies_to_source(&SourceType::Container("nginx".to_string())));
        assert!(!rule.applies_to_source(&SourceType::Stream("websocket".to_string())));
    }

    #[test]
    fn test_rule_applies_to_source_container_filter() {
        let rule = Rule {
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
        };
        
        // Should match specified containers
        assert!(rule.applies_to_source(&SourceType::Container("nginx".to_string())));
        assert!(rule.applies_to_source(&SourceType::Container("api".to_string())));
        
        // Should not match other containers
        assert!(!rule.applies_to_source(&SourceType::Container("postgres".to_string())));
        
        // Should not match files or streams
        assert!(!rule.applies_to_source(&SourceType::File(PathBuf::from("/var/log/app.log"))));
        assert!(!rule.applies_to_source(&SourceType::Stream("websocket".to_string())));
    }

    #[test]
    fn test_rule_applies_to_source_stream_filter() {
        let rule = Rule {
            name: "test".to_string(),
            text: Some("error".to_string()),
            pattern: None,
            alert: vec!["slack".to_string()],
            cooldown: 60,
            sources: Some(RuleSources {
                files: vec![],
                containers: vec![],
                streams: vec!["azure_webapp".to_string()],
            }),
        };
        
        // Should match specified stream
        assert!(rule.applies_to_source(&SourceType::Stream("azure_webapp".to_string())));
        
        // Should not match other streams
        assert!(!rule.applies_to_source(&SourceType::Stream("k8s_logs".to_string())));
        
        // Should not match files or containers
        assert!(!rule.applies_to_source(&SourceType::File(PathBuf::from("/var/log/app.log"))));
        assert!(!rule.applies_to_source(&SourceType::Container("nginx".to_string())));
    }

    #[test]
    fn test_stream_config_get_name_with_name() {
        let stream = StreamConfig {
            name: Some("my-stream".to_string()),
            stream_type: StreamType::Websocket,
            url: "wss://example.com/logs".to_string(),
            headers: None,
            reconnect_delay: None,
        };
        
        assert_eq!(stream.get_name(), "my-stream");
    }

    #[test]
    fn test_stream_config_get_name_without_name() {
        let stream = StreamConfig {
            name: None,
            stream_type: StreamType::Websocket,
            url: "wss://example.com/logs".to_string(),
            headers: None,
            reconnect_delay: None,
        };
        
        let name = stream.get_name();
        assert!(name.contains("Websocket"));
        assert!(name.contains("wss://example.com/logs"));
    }

    #[test]
    fn test_stream_config_reconnect_delay_default() {
        let stream = StreamConfig {
            name: None,
            stream_type: StreamType::Tcp,
            url: "localhost:514".to_string(),
            headers: None,
            reconnect_delay: None,
        };
        
        assert_eq!(stream.get_reconnect_delay(), 5);
    }

    #[test]
    fn test_stream_config_reconnect_delay_custom() {
        let stream = StreamConfig {
            name: None,
            stream_type: StreamType::Tcp,
            url: "localhost:514".to_string(),
            headers: None,
            reconnect_delay: Some(10),
        };
        
        assert_eq!(stream.get_reconnect_delay(), 10);
    }

    #[test]
    fn test_env_var_expansion() {
        // Set test environment variable
        env::set_var("TEST_WEBHOOK_URL", "https://hooks.slack.com/test");
        
        let yaml = r#"
alerts:
  slack:
    type: slack
    url: "${TEST_WEBHOOK_URL}"
rules: []
"#;
        
        let mut config: Config = serde_yaml::from_str(yaml).unwrap();
        config.expand_env_vars();
        
        if let Some(alert) = config.alerts.get("slack") {
            if let AlertOptions::Slack { url } = &alert.options {
                assert_eq!(url, "https://hooks.slack.com/test");
            } else {
                panic!("Expected Slack alert");
            }
        } else {
            panic!("Expected slack alert to exist");
        }
        
        env::remove_var("TEST_WEBHOOK_URL");
    }

    #[test]
    fn test_env_var_expansion_missing() {
        let yaml = r#"
alerts:
  slack:
    type: slack
    url: "${NONEXISTENT_VAR}"
rules: []
"#;
        
        let mut config: Config = serde_yaml::from_str(yaml).unwrap();
        config.expand_env_vars();
        
        if let Some(alert) = config.alerts.get("slack") {
            if let AlertOptions::Slack { url } = &alert.options {
                // Should be empty string when var doesn't exist
                assert_eq!(url, "");
            }
        }
    }

    #[test]
    fn test_single_alert_deserialization() {
        let yaml = r#"
name: test_rule
text: "error"
alert: slack
cooldown: 60
"#;
        
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.alert.len(), 1);
        assert_eq!(rule.alert[0], "slack");
    }

    #[test]
    fn test_multiple_alerts_deserialization() {
        let yaml = r#"
name: test_rule
text: "error"
alert: [slack, pagerduty, discord]
cooldown: 60
"#;
        
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.alert.len(), 3);
        assert_eq!(rule.alert[0], "slack");
        assert_eq!(rule.alert[1], "pagerduty");
        assert_eq!(rule.alert[2], "discord");
    }

    #[test]
    fn test_default_cooldown() {
        let yaml = r#"
name: test_rule
text: "error"
alert: slack
"#;
        
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.cooldown, 60); // Default cooldown
    }

    #[test]
    fn test_custom_cooldown() {
        let yaml = r#"
name: test_rule
text: "error"
alert: slack
cooldown: 300
"#;
        
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.cooldown, 300);
    }

    #[test]
    fn test_alert_type_parsing() {
        let yaml = r#"
slack:
  type: slack
  url: "https://hooks.slack.com/test"
discord:
  type: discord
  url: "https://discord.com/api/webhooks/test"
telegram:
  type: telegram
  bot_token: "123456:ABC"
  chat_id: "123456"
stdout:
  type: stdout
"#;
        
        let alerts: HashMap<String, Alert> = serde_yaml::from_str(yaml).unwrap();
        
        assert_eq!(alerts.get("slack").unwrap().alert_type, AlertType::Slack);
        assert_eq!(alerts.get("discord").unwrap().alert_type, AlertType::Discord);
        assert_eq!(alerts.get("telegram").unwrap().alert_type, AlertType::Telegram);
        assert_eq!(alerts.get("stdout").unwrap().alert_type, AlertType::Stdout);
    }

    #[test]
    fn test_resource_thresholds_multiple_alerts() {
        let yaml = r#"
interval: 30
thresholds:
  cpu_percent: 90
  memory_percent: 85
  alert: [slack, pagerduty]
"#;
        
        let resource: ResourceConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(resource.thresholds.alert.len(), 2);
        assert_eq!(resource.thresholds.alert[0], "slack");
        assert_eq!(resource.thresholds.alert[1], "pagerduty");
    }

    #[test]
    fn test_system_check_defaults() {
        let yaml = r#"
name: api_health
type: http
url: "http://localhost:8080/health"
alert: slack
"#;
        
        let check: SystemCheck = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(check.interval, 30); // default
        assert_eq!(check.timeout, 5); // default
        assert_eq!(check.missed_threshold, 2); // default
    }

    #[test]
    fn test_system_check_custom_values() {
        let yaml = r#"
name: api_health
type: http
url: "http://localhost:8080/health"
interval: 60
timeout: 10
missed_threshold: 3
alert: slack
"#;
        
        let check: SystemCheck = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(check.interval, 60);
        assert_eq!(check.timeout, 10);
        assert_eq!(check.missed_threshold, 3);
    }
}
