# System Health Checks

TinyWatcher can monitor the liveness of HTTP services and alert you when they become unavailable.

## Features

- **HTTP Health Checks**: Monitor HTTP endpoints for availability and response codes
- **Configurable Intervals**: Set how often to check each service
- **Failure Thresholds**: Only alert after N consecutive failures to avoid false positives
- **Recovery Alerts**: Get notified when services come back online
- **Timeout Control**: Set request timeouts per check

## Configuration

Add `system_checks` to your configuration file:

```yaml
system_checks:
  - name: local_api
    type: http
    url: "http://localhost:8080/health"
    interval: 30        # Check every 30 seconds (default: 30)
    timeout: 5          # Request timeout in seconds (default: 5)
    missed_threshold: 2 # Alert after N failures (default: 2)
    alert: oncall_slack
```

### Parameters

- **name** (required): Friendly name for the health check
- **type** (required): Type of check. Currently only `http` is supported
- **url** (required): The URL to check
- **interval** (optional): Seconds between checks. Default: 30
- **timeout** (optional): Request timeout in seconds. Default: 5
- **missed_threshold** (optional): Number of consecutive failures before alerting. Default: 2
- **alert** (required): Name of the alert handler to use (must be defined in `alerts` section)

## How It Works

1. **Periodic Checks**: TinyWatcher will make an HTTP GET request to the specified URL at the configured interval
2. **Success Detection**: Any HTTP 2xx response code is considered successful
3. **Failure Detection**: Non-2xx responses, timeouts, or connection errors count as failures
4. **Threshold Logic**: The check must fail `missed_threshold` times in a row before triggering an alert
5. **Recovery Detection**: When a service recovers after being down, a recovery alert is sent

## Alert Messages

### Down Alert
```
Service 'local_api' is DOWN
Identity: production-server
URL: http://localhost:8080/health
Failed checks: 2
Error: HTTP status: 503
```

### Recovery Alert
```
Service 'local_api' is back UP
Identity: production-server
URL: http://localhost:8080/health
Status: Healthy
```

## Example Configurations

### Minimal Configuration
```yaml
alerts:
  stdout_alert:
    type: stdout

system_checks:
  - name: my_service
    type: http
    url: "http://localhost:8080/health"
    alert: stdout_alert
```

### Production Configuration
```yaml
alerts:
  pagerduty:
    type: webhook
    url: "https://events.pagerduty.com/v2/enqueue"
  
  slack_ops:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

system_checks:
  # Critical API - alert immediately
  - name: payment_api
    type: http
    url: "https://api.example.com/v1/health"
    interval: 15
    timeout: 3
    missed_threshold: 1  # Alert on first failure
    alert: pagerduty

  # Internal service - more lenient
  - name: background_worker
    type: http
    url: "http://localhost:9090/healthz"
    interval: 60
    timeout: 10
    missed_threshold: 3
    alert: slack_ops

  # Database health endpoint
  - name: postgres_health
    type: http
    url: "http://localhost:5432/health"
    interval: 30
    timeout: 5
    missed_threshold: 2
    alert: pagerduty
```

### Combined with Other Monitoring
```yaml
identity:
  name: web-server-01

alerts:
  ops_slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

# Monitor log files
inputs:
  files:
    - /var/log/app.log

rules:
  - name: error_logs
    text: "ERROR"
    alert: ops_slack
    cooldown: 300

# Monitor system resources
resources:
  interval: 60
  thresholds:
    cpu_percent: 90
    memory_percent: 85
    disk_percent: 90
    alert: ops_slack

# Monitor service health
system_checks:
  - name: web_app
    type: http
    url: "http://localhost:3000/health"
    interval: 30
    timeout: 5
    missed_threshold: 2
    alert: ops_slack
```

## Best Practices

1. **Set Appropriate Intervals**: Balance between quick detection and avoiding excessive checks
   - Critical services: 15-30 seconds
   - Normal services: 30-60 seconds
   - Background jobs: 60-120 seconds

2. **Use Missed Threshold Wisely**: Avoid false positives from temporary network issues
   - Critical services: 1-2 failures
   - Normal services: 2-3 failures
   - Flaky services: 3-5 failures

3. **Set Realistic Timeouts**: Based on expected response times
   - Fast APIs: 3-5 seconds
   - Normal APIs: 5-10 seconds
   - Slow services: 10-30 seconds

4. **Health Check Endpoints**: Implement proper health check endpoints in your services
   - Should be lightweight (no heavy database queries)
   - Return 200 OK when healthy
   - Return 503 Service Unavailable when unhealthy
   - Include basic dependency checks (database, cache, etc.)

## Testing

Use the `test` command to validate your configuration:

```bash
tinywatcher test --config health-checks.yaml
```

This will validate:
- All health check configurations are valid
- Referenced alerts exist
- URLs are properly formatted

## Running

Start monitoring with:

```bash
tinywatcher watch --config health-checks.yaml
```

Or as a background service:

```bash
tinywatcher start --config health-checks.yaml
```

## Future Enhancements

Planned features for system checks:
- TCP port checks
- ICMP ping checks
- DNS resolution checks
- Custom headers for HTTP checks
- POST/PUT request support
- Response body validation
- SSL certificate expiry monitoring
