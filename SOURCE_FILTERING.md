# Source-Specific Rule Filtering

## Overview

TinyWatcher now supports assigning rules to specific sources, allowing you to create targeted monitoring rules that only apply to certain files, containers, or streams. This makes your monitoring more efficient and organized.

## How It Works

When a log line comes in, TinyWatcher:

1. **Identifies the source** - Which file, container, or stream produced the log line
2. **Checks rule filters** - Whether each rule has a `sources:` filter defined
3. **Applies matching rules** - Only processes rules that match the source

**Important**: If a rule has **no** `sources:` filter, it applies to **all** inputs.

## Configuration Syntax

```yaml
rules:
  - name: rule_name
    pattern: "regex pattern"
    sources:                    # Optional - omit to apply to all sources
      containers: ["name1"]     # Optional - list of container names
      files: ["/path/to/file"]  # Optional - list of file paths
      streams: ["stream_name"]  # Optional - list of stream names
    alert: alert_name
    cooldown: 60
```

## Examples

### 1. Container-Specific Rule

Monitor only the API container for 500 errors:

```yaml
rules:
  - name: api_500s
    pattern: "500|Internal Server Error"
    sources:
      containers: ["api"]
    alert: team_slack
    cooldown: 60
```

### 2. File-Specific Rule

Monitor authentication failures only in nginx logs:

```yaml
rules:
  - name: nginx_auth_failures
    pattern: "auth.*failed"
    sources:
      files: ["/var/log/nginx/error.log"]
    alert: security_webhook
    cooldown: 120
```

### 3. Stream-Specific Rule

Monitor a specific stream for errors:

```yaml
rules:
  - name: azure_errors
    pattern: "Exception|Error"
    sources:
      streams: ["azure_app_service"]
    alert: team_slack
    cooldown: 60
```

### 4. Multi-Source Rule

Apply a rule to multiple sources:

```yaml
rules:
  - name: memory_warnings
    pattern: "OutOfMemoryError|memory exhausted"
    sources:
      containers: ["api", "worker"]
      files: ["/var/log/app.log"]
    alert: oncall_slack
    cooldown: 300
```

### 5. Universal Rule (No Filter)

Apply a rule to ALL sources by omitting the `sources:` field:

```yaml
rules:
  - name: critical_errors
    pattern: "CRITICAL|FATAL"
    alert: oncall_slack
    cooldown: 60
    # No sources field = applies to all files, containers, and streams
```

## Complete Example

```yaml
inputs:
  files:
    - /var/log/nginx/error.log
    - /var/log/app.log
  containers:
    - api
    - postgres
  streams:
    - name: azure_app_service
      type: http
      url: https://example.com/logs

alerts:
  team_slack:
    type: slack
    url: https://hooks.slack.com/services/YOUR/WEBHOOK
  
  oncall_slack:
    type: slack
    url: https://hooks.slack.com/services/YOUR/ONCALL/WEBHOOK

rules:
  # Only check API container for 500s
  - name: api_500s
    pattern: "500|Internal Server Error"
    sources:
      containers: ["api"]
      streams: ["azure_app_service"]
    alert: team_slack
    cooldown: 60

  # Only check postgres container for database errors
  - name: postgres_errors
    pattern: "FATAL|PANIC|deadlock"
    sources:
      containers: ["postgres"]
    alert: oncall_slack
    cooldown: 30

  # Only check nginx logs for auth failures
  - name: nginx_auth_failures
    pattern: "auth.*failed"
    sources:
      files: ["/var/log/nginx/error.log"]
    alert: oncall_slack
    cooldown: 120

  # Check ALL sources for critical errors
  - name: critical_errors
    pattern: "CRITICAL|FATAL"
    alert: oncall_slack
    cooldown: 60
```

## Testing Your Configuration

Use the `test` command to validate your configuration, including source filters:

```bash
tinywatcher test --config example-sources.yaml
```

This will show:
- Which sources each rule applies to
- Whether the source filters are valid
- A visual breakdown of your monitoring setup

## Use Cases

### Microservices Architecture

```yaml
rules:
  - name: auth_service_errors
    pattern: "authentication failed"
    sources:
      containers: ["auth-service"]
    alert: auth_team_slack
    
  - name: payment_errors
    pattern: "payment.*failed"
    sources:
      containers: ["payment-service"]
    alert: payment_team_slack
```

### Environment Separation

```yaml
rules:
  - name: prod_database_errors
    pattern: "FATAL|PANIC"
    sources:
      containers: ["prod-postgres"]
    alert: oncall_pagerduty
    
  - name: staging_database_errors
    pattern: "FATAL|PANIC"
    sources:
      containers: ["staging-postgres"]
    alert: dev_slack
```

### Log Type Separation

```yaml
rules:
  - name: security_events
    pattern: "unauthorized|forbidden"
    sources:
      files: ["/var/log/nginx/error.log", "/var/log/auth.log"]
    alert: security_team
    
  - name: application_errors
    pattern: "ERROR|Exception"
    sources:
      files: ["/var/log/app.log"]
      containers: ["app-container"]
    alert: dev_team
```

## Benefits

1. **Performance** - Rules only run against relevant sources, reducing CPU usage
2. **Organization** - Clear separation of concerns for different services/components
3. **Flexibility** - Same pattern can trigger different alerts based on source
4. **Reduced Noise** - Avoid false positives from irrelevant sources
5. **Team Ownership** - Route alerts to appropriate teams based on source

## Migration from Non-Filtered Rules

If you have existing rules without `sources:` filters, they will continue to work exactly as before - applying to all inputs. You can gradually migrate to source-specific rules:

```yaml
# Old style (still works)
rules:
  - name: all_errors
    pattern: "ERROR"
    alert: console
    cooldown: 60

# New style with source filtering
rules:
  - name: api_errors
    pattern: "ERROR"
    sources:
      containers: ["api"]
    alert: team_slack
    cooldown: 60
    
  - name: other_errors
    pattern: "ERROR"
    # No sources = applies to everything except sources matched by api_errors
    alert: general_slack
    cooldown: 60
```

**Note**: If multiple rules match a log line, all of them will trigger. There's no exclusion logic - each rule independently evaluates whether it applies.

## Troubleshooting

### Rule Not Triggering

1. Check that the source name exactly matches (case-sensitive):
   - For files: Use absolute paths that match what you configured in `inputs.files`
   - For containers: Use exact container names
   - For streams: Use the `name` field from your stream config

2. Verify the rule pattern matches your logs:
   ```bash
   tinywatcher check --config your-config.yaml --lines 100
   ```

3. Test without source filters to isolate the issue:
   ```yaml
   # Temporarily remove sources to see if pattern matches
   - name: test_rule
     pattern: "your pattern"
     alert: console
     # sources: ...  # commented out
   ```

### Debugging

Enable verbose logging to see which rules are being evaluated:

```bash
tinywatcher watch --config your-config.yaml --verbose
```

This will show debug messages like:
```
Rule 'api_500s' matched line from Container("api"): ...
```
