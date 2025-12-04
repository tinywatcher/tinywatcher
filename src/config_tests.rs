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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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
            threshold: None,
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

    #[test]
    fn test_threshold_parse_seconds() {
        let threshold = Threshold::parse("5 in 2s").unwrap();
        assert_eq!(threshold.count, 5);
        assert_eq!(threshold.window.as_secs(), 2);
    }

    #[test]
    fn test_threshold_parse_milliseconds() {
        let threshold = Threshold::parse("3 in 500ms").unwrap();
        assert_eq!(threshold.count, 3);
        assert_eq!(threshold.window.as_millis(), 500);
    }

    #[test]
    fn test_threshold_parse_minutes() {
        let threshold = Threshold::parse("10 in 1m").unwrap();
        assert_eq!(threshold.count, 10);
        assert_eq!(threshold.window.as_secs(), 60);
    }

    #[test]
    fn test_threshold_parse_hours() {
        let threshold = Threshold::parse("100 in 1h").unwrap();
        assert_eq!(threshold.count, 100);
        assert_eq!(threshold.window.as_secs(), 3600);
    }

    #[test]
    fn test_threshold_parse_with_whitespace() {
        let threshold = Threshold::parse("  5  in  2s  ").unwrap();
        assert_eq!(threshold.count, 5);
        assert_eq!(threshold.window.as_secs(), 2);
    }

    #[test]
    fn test_threshold_parse_large_values() {
        let threshold = Threshold::parse("1000 in 24h").unwrap();
        assert_eq!(threshold.count, 1000);
        assert_eq!(threshold.window.as_secs(), 24 * 3600);
    }

    #[test]
    fn test_threshold_parse_invalid_format() {
        assert!(Threshold::parse("5 2s").is_err());
        assert!(Threshold::parse("5in2s").is_err());
        assert!(Threshold::parse("5 at 2s").is_err());
        assert!(Threshold::parse("five in 2s").is_err());
    }

    #[test]
    fn test_threshold_parse_invalid_unit() {
        assert!(Threshold::parse("5 in 2d").is_err());
        assert!(Threshold::parse("5 in 2x").is_err());
    }

    #[test]
    fn test_threshold_serde_deserialize() {
        let yaml = r#"
name: test_rule
text: "error"
alert: slack
threshold: "5 in 2s"
"#;
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert!(rule.threshold.is_some());
        let threshold = rule.threshold.unwrap();
        assert_eq!(threshold.count, 5);
        assert_eq!(threshold.window.as_secs(), 2);
    }

    #[test]
    fn test_threshold_serde_serialize() {
        let threshold = Threshold {
            count: 5,
            window: std::time::Duration::from_secs(2),
        };
        let serialized = serde_yaml::to_string(&threshold).unwrap();
        assert!(serialized.contains("5 in 2s") || serialized.contains("\"5 in 2s\""));
    }

    #[test]
    fn test_rule_with_threshold() {
        let yaml = r#"
name: rate_limited_error
pattern: "ERROR"
alert: oncall
threshold: "10 in 1m"
cooldown: 300
"#;
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.name, "rate_limited_error");
        assert!(rule.threshold.is_some());
        let threshold = rule.threshold.unwrap();
        assert_eq!(threshold.count, 10);
        assert_eq!(threshold.window.as_secs(), 60);
    }

    #[test]
    fn test_system_check_with_threshold() {
        let yaml = r#"
name: api_health
type: http
url: "http://localhost:8080/health"
threshold: "3 in 1m"
alert: slack
"#;
        let check: SystemCheck = serde_yaml::from_str(yaml).unwrap();
        assert!(check.threshold.is_some());
        let threshold = check.threshold.unwrap();
        assert_eq!(threshold.count, 3);
        assert_eq!(threshold.window.as_secs(), 60);
    }

    #[test]
    fn test_rule_without_threshold() {
        let yaml = r#"
name: simple_error
text: "error"
alert: slack
"#;
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert!(rule.threshold.is_none());
    }

    #[test]
    fn test_expand_file_globs_no_patterns() {
        use std::fs::File;
        use tempfile::TempDir;

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("test1.log");
        let file2_path = temp_dir.path().join("test2.log");
        
        File::create(&file1_path).unwrap();
        File::create(&file2_path).unwrap();

        let config = Config {
            inputs: Inputs {
                files: vec![file1_path.clone(), file2_path.clone()],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let expanded = config.expand_file_globs().unwrap();
        assert_eq!(expanded.len(), 2);
        assert!(expanded.contains(&file1_path));
        assert!(expanded.contains(&file2_path));
    }

    #[test]
    fn test_expand_file_globs_with_wildcard() {
        use std::fs::File;
        use tempfile::TempDir;

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("app1.log");
        let file2_path = temp_dir.path().join("app2.log");
        let file3_path = temp_dir.path().join("other.txt");
        
        File::create(&file1_path).unwrap();
        File::create(&file2_path).unwrap();
        File::create(&file3_path).unwrap();

        let pattern = temp_dir.path().join("*.log");
        
        let config = Config {
            inputs: Inputs {
                files: vec![pattern],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let expanded = config.expand_file_globs().unwrap();
        assert_eq!(expanded.len(), 2);
        assert!(expanded.contains(&file1_path));
        assert!(expanded.contains(&file2_path));
        assert!(!expanded.iter().any(|p| p.ends_with("other.txt")));
    }

    #[test]
    fn test_expand_file_globs_mixed_patterns_and_files() {
        use std::fs::File;
        use tempfile::TempDir;

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("app1.log");
        let file2_path = temp_dir.path().join("app2.log");
        let file3_path = temp_dir.path().join("specific.log");
        
        File::create(&file1_path).unwrap();
        File::create(&file2_path).unwrap();
        File::create(&file3_path).unwrap();

        let pattern = temp_dir.path().join("app*.log");
        
        let config = Config {
            inputs: Inputs {
                files: vec![pattern, file3_path.clone()],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let expanded = config.expand_file_globs().unwrap();
        assert_eq!(expanded.len(), 3);
        assert!(expanded.contains(&file1_path));
        assert!(expanded.contains(&file2_path));
        assert!(expanded.contains(&file3_path));
    }

    #[test]
    fn test_expand_file_globs_no_matches() {
        use tempfile::TempDir;

        // Create a temporary directory with no matching files
        let temp_dir = TempDir::new().unwrap();
        let pattern = temp_dir.path().join("*.log");
        
        let config = Config {
            inputs: Inputs {
                files: vec![pattern],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let expanded = config.expand_file_globs().unwrap();
        assert_eq!(expanded.len(), 0);
    }

    #[test]
    fn test_expand_file_globs_question_mark_pattern() {
        use std::fs::File;
        use tempfile::TempDir;

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("app1.log");
        let file2_path = temp_dir.path().join("app2.log");
        let file3_path = temp_dir.path().join("app10.log");
        
        File::create(&file1_path).unwrap();
        File::create(&file2_path).unwrap();
        File::create(&file3_path).unwrap();

        let pattern = temp_dir.path().join("app?.log");
        
        let config = Config {
            inputs: Inputs {
                files: vec![pattern],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let expanded = config.expand_file_globs().unwrap();
        assert_eq!(expanded.len(), 2);
        assert!(expanded.contains(&file1_path));
        assert!(expanded.contains(&file2_path));
        assert!(!expanded.contains(&file3_path)); // app10.log has two characters
    }

    #[test]
    fn test_expand_file_globs_bracket_pattern() {
        use std::fs::File;
        use tempfile::TempDir;

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.path().join("app1.log");
        let file2_path = temp_dir.path().join("app2.log");
        let file3_path = temp_dir.path().join("app9.log");
        
        File::create(&file1_path).unwrap();
        File::create(&file2_path).unwrap();
        File::create(&file3_path).unwrap();

        let pattern = temp_dir.path().join("app[12].log");
        
        let config = Config {
            inputs: Inputs {
                files: vec![pattern],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let expanded = config.expand_file_globs().unwrap();
        assert_eq!(expanded.len(), 2);
        assert!(expanded.contains(&file1_path));
        assert!(expanded.contains(&file2_path));
        assert!(!expanded.contains(&file3_path));
    }

    #[test]
    fn test_expand_file_globs_ignores_directories() {
        use std::fs::{create_dir, File};
        use tempfile::TempDir;

        // Create a temporary directory with files and subdirectories
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("file.log");
        let dir_path = temp_dir.path().join("dir.log");
        
        File::create(&file_path).unwrap();
        create_dir(&dir_path).unwrap();

        let pattern = temp_dir.path().join("*.log");
        
        let config = Config {
            inputs: Inputs {
                files: vec![pattern],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let expanded = config.expand_file_globs().unwrap();
        // Should only include the file, not the directory
        assert_eq!(expanded.len(), 1);
        assert!(expanded.contains(&file_path));
        assert!(!expanded.contains(&dir_path));
    }

    #[test]
    fn test_expand_file_globs_invalid_pattern() {
        let config = Config {
            inputs: Inputs {
                files: vec![PathBuf::from("[invalid")],
                containers: vec![],
                streams: vec![],
            },
            alerts: std::collections::HashMap::new(),
            rules: vec![],
            resources: None,
            identity: Identity::default(),
            system_checks: vec![],
        };

        let result = config.expand_file_globs();
        assert!(result.is_err());
    }
}
