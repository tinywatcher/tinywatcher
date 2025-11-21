# Source Filtering Quick Reference

## Syntax

```yaml
rules:
  - name: rule_name
    pattern: "regex"
    sources:          # Optional - omit for "applies to all"
      containers: []  # Optional - list of container names
      files: []       # Optional - list of file paths (absolute)
      streams: []     # Optional - list of stream names
    alert: alert_name
    cooldown: 60
```

## Common Patterns

### Container-Only Rules

```yaml
# PostgreSQL errors
- name: postgres_errors
  pattern: "FATAL|PANIC|deadlock"
  sources:
    containers: ["postgres"]
  alert: database_team
  cooldown: 30

# Multiple containers
- name: api_errors
  pattern: "ERROR"
  sources:
    containers: ["api-service", "worker-service"]
  alert: dev_team
  cooldown: 60
```

### File-Only Rules

```yaml
# Nginx errors
- name: nginx_errors
  pattern: "error"
  sources:
    files: ["/var/log/nginx/error.log"]
  alert: ops_team
  cooldown: 120

# Multiple log files
- name: app_errors
  pattern: "Exception"
  sources:
    files: 
      - "/var/log/app.log"
      - "/var/log/worker.log"
  alert: dev_team
  cooldown: 60
```

### Stream-Only Rules

```yaml
# Azure App Service
- name: azure_errors
  pattern: "Exception|Error"
  sources:
    streams: ["azure_webapp"]
  alert: cloud_team
  cooldown: 60

# Kubernetes pods
- name: k8s_pod_errors
  pattern: "CrashLoopBackOff|OOMKilled"
  sources:
    streams: ["prod-cluster", "staging-cluster"]
  alert: platform_team
  cooldown: 120
```

### Mixed Source Rules

```yaml
# Memory issues across app and container
- name: memory_warnings
  pattern: "OutOfMemoryError|OOM"
  sources:
    containers: ["app"]
    files: ["/var/log/app.log"]
  alert: [dev_team, oncall]
  cooldown: 300

# Security events from multiple sources
- name: security_alerts
  pattern: "unauthorized|forbidden|breach"
  sources:
    files: 
      - "/var/log/nginx/error.log"
      - "/var/log/auth.log"
    containers: ["auth-service"]
    streams: ["waf-logs"]
  alert: security_team
  cooldown: 0  # No cooldown for security
```

### Universal Rules (No Filter)

```yaml
# Critical errors everywhere
- name: critical_everywhere
  pattern: "CRITICAL|FATAL|EMERGENCY"
  # No sources field = applies to all inputs
  alert: oncall_pagerduty
  cooldown: 60

# Panic errors
- name: panic_errors
  pattern: "panic:"
  alert: oncall_slack
  cooldown: 30
```

## Best Practices

### 1. Start Broad, Then Narrow

```yaml
# Start with a universal rule
- name: all_errors
  pattern: "ERROR"
  alert: console
  cooldown: 60

# Then create specific rules as needed
- name: api_errors
  pattern: "ERROR"
  sources:
    containers: ["api"]
  alert: api_team
  cooldown: 60
```

### 2. Use Descriptive Rule Names

```yaml
# ✅ Good
- name: postgres_connection_errors
  sources:
    containers: ["postgres"]
  
# ❌ Less clear
- name: rule1
  sources:
    containers: ["postgres"]
```

### 3. Group by Team Ownership

```yaml
# API Team
- name: api_errors
  sources:
    containers: ["api", "api-worker"]
  alert: api_team_slack

# Database Team  
- name: db_errors
  sources:
    containers: ["postgres", "redis"]
  alert: db_team_slack

# Platform Team
- name: platform_errors
  sources:
    streams: ["k8s-cluster"]
  alert: platform_team_slack
```

### 4. Separate Critical from Non-Critical

```yaml
# Critical - no cooldown, multiple alerts
- name: production_down
  pattern: "service unavailable|500|502|503"
  sources:
    containers: ["prod-api"]
  alert: [oncall_pagerduty, cto_slack]
  cooldown: 0

# Non-critical - longer cooldown
- name: staging_warnings
  pattern: "warning|deprecated"
  sources:
    containers: ["staging-api"]
  alert: dev_slack
  cooldown: 300
```

### 5. Use Different Cooldowns by Source

```yaml
# Production - aggressive alerting
- name: prod_errors
  pattern: "ERROR"
  sources:
    containers: ["prod-api"]
  alert: oncall_slack
  cooldown: 30

# Development - relaxed alerting
- name: dev_errors
  pattern: "ERROR"
  sources:
    containers: ["dev-api"]
  alert: dev_slack
  cooldown: 300
```

## Performance Tips

1. **Use specific sources** - Rules only run against matching sources
2. **Optimize regex** - Simpler patterns = faster matching
3. **Appropriate cooldowns** - Prevent alert spam

## Testing

```bash
# Validate configuration
tinywatcher test --config your-config.yaml

# Check recent logs with rules
tinywatcher check --config your-config.yaml --lines 100

# Watch with verbose output
tinywatcher watch --config your-config.yaml --verbose
```

## Troubleshooting

### Rule not matching?

1. Check source name exactly matches (case-sensitive)
2. For files: use absolute paths
3. For containers: exact container name (not image name)
4. For streams: use the `name` field from stream config

### Too many matches?

1. Add source filters to narrow scope
2. Make regex more specific
3. Increase cooldown

### Not enough matches?

1. Remove sources filter temporarily to test pattern
2. Check logs manually: `tail /path/to/file | grep "pattern"`
3. Use `check` command to see what matches

## Examples by Use Case

### Microservices

```yaml
rules:
  - name: auth_service_errors
    pattern: "authentication.*failed"
    sources: {containers: ["auth-svc"]}
    alert: auth_team

  - name: payment_errors  
    pattern: "payment.*failed"
    sources: {containers: ["payment-svc"]}
    alert: payment_team
```

### Multi-Environment

```yaml
rules:
  - name: prod_critical
    pattern: "CRITICAL"
    sources: 
      containers: ["prod-app"]
    alert: pagerduty
    cooldown: 0

  - name: staging_critical
    pattern: "CRITICAL"
    sources:
      containers: ["staging-app"]
    alert: slack
    cooldown: 60
```

### Security Monitoring

```yaml
rules:
  - name: ssh_failures
    pattern: "Failed password"
    sources:
      files: ["/var/log/auth.log"]
    alert: security_team
    cooldown: 30

  - name: web_attacks
    pattern: "sql injection|xss|csrf"
    sources:
      files: ["/var/log/nginx/access.log"]
    alert: security_team
    cooldown: 0
```
