# Threshold-Based Alerting

TinyWatcher supports rate-based alerting with an intuitive `"X in Y"` threshold format. This helps prevent alert fatigue by only alerting when events occur at a concerning rate.

## Format

```
<count> in <time_window>
```

### Examples

- `5 in 2s` - 5 occurrences in 2 seconds
- `10 in 1m` - 10 occurrences in 1 minute  
- `3 in 500ms` - 3 occurrences in 500 milliseconds
- `100 in 1h` - 100 occurrences in 1 hour

### Supported Time Units

- `ms` - milliseconds
- `s` - seconds
- `m` - minutes
- `h` - hours

## Usage

### Log Monitoring Rules

Add a `threshold` field to any rule:

```yaml
rules:
  # Alert only if we see 5 errors within 2 seconds
  - name: error_burst
    pattern: "ERROR"
    threshold: "5 in 2s"
    alert: team_slack
    cooldown: 300

  # No threshold = immediate alert (default behavior)
  - name: critical_error
    pattern: "CRITICAL"
    alert: oncall
    cooldown: 60
```

### Health Check Monitoring

Add a `threshold` field to system checks:

```yaml
system_checks:
  # Alert if API fails 3 times within 1 minute
  - name: api_health
    type: http
    url: "http://localhost:8080/health"
    interval: 10
    timeout: 5
    threshold: "3 in 1m"
    alert: oncall

  # You can still use missed_threshold for consecutive failures
  - name: legacy_check
    type: http
    url: "http://localhost:9090/health"
    interval: 30
    timeout: 5
    missed_threshold: 2  # Alert after 2 consecutive failures
    alert: oncall
```

## How It Works

### Sliding Window

TinyWatcher uses a **sliding time window** to track events:

1. When a pattern matches (or health check fails), the timestamp is recorded
2. Old timestamps outside the time window are automatically removed
3. When the count reaches the threshold, an alert is triggered
4. The window is cleared after alerting to prevent repeated alerts

### Example Scenario

With `threshold: "5 in 2s"`:

```
Time    Event       Window Count    Action
------  ----------  ------------    ------
0.0s    ERROR       [0.0]           1 - no alert
0.5s    ERROR       [0.0, 0.5]      2 - no alert
1.0s    ERROR       [0.0, 0.5, 1.0] 3 - no alert
1.5s    ERROR       [0.0, 0.5, 1.0, 1.5] 4 - no alert
1.8s    ERROR       [0.0, 0.5, 1.0, 1.5, 1.8] 5 - ALERT! (threshold reached)
2.0s    ERROR       [2.0]           1 - window cleared after alert
2.5s    ERROR       [2.0, 2.5]      2 - no alert
```

## Benefits

### 1. Prevents Alert Fatigue

Without thresholds, a single error triggers an alert immediately. With thresholds, you only get alerted when errors occur at a rate that indicates a real problem.

### 2. Catches Bursts and Spikes

Perfect for detecting:
- Error bursts
- Authentication attacks
- Rate limit violations
- Service degradation
- Memory leaks (increasing OOM errors)

### 3. Flexible Configuration

Different patterns need different thresholds:

```yaml
rules:
  # Occasional warnings are OK
  - name: warnings
    pattern: "WARN"
    threshold: "50 in 1m"
    alert: team_slack

  # But errors should be rare
  - name: errors
    pattern: "ERROR"
    threshold: "5 in 30s"
    alert: oncall

  # Critical errors = immediate alert
  - name: critical
    pattern: "CRITICAL"
    alert: oncall  # No threshold!
```

## Comparison with Consecutive Failures

### Health Check: Threshold vs missed_threshold

```yaml
# Old style: Alert after 2 consecutive failures
system_checks:
  - name: api
    type: http
    url: "http://localhost:8080/health"
    interval: 30
    missed_threshold: 2  # Must fail 2 times in a row
    alert: oncall

# New style: Alert if 3 failures within 1 minute (more sophisticated)
system_checks:
  - name: api
    type: http
    url: "http://localhost:8080/health"
    interval: 10
    threshold: "3 in 1m"  # Any 3 failures in the window
    alert: oncall
```

**Difference:**
- `missed_threshold: 2` requires 2 **consecutive** failures
- `threshold: "3 in 1m"` requires 3 failures **within the time window** (doesn't have to be consecutive)

## Real-World Examples

### 1. Database Connection Pool Exhaustion

```yaml
- name: db_pool_exhausted
  pattern: "no available database connections|connection pool timeout"
  threshold: "3 in 30s"
  alert: oncall
  cooldown: 300
```

### 2. Brute Force Attack Detection

```yaml
- name: auth_attack
  pattern: "authentication failed|invalid credentials"
  threshold: "20 in 30s"
  alert: [security_team, oncall]
  cooldown: 600
```

### 3. Memory Pressure

```yaml
- name: memory_issues
  pattern: "out of memory|OOM|MemoryError|heap space"
  threshold: "2 in 5m"
  alert: oncall
  cooldown: 300
```

### 4. API Rate Limiting

```yaml
- name: rate_limit_spike
  text: "429 Too Many Requests"
  threshold: "100 in 1m"
  alert: team_slack
  cooldown: 180
```

### 5. Slow Query Detection

```yaml
- name: slow_query_burst
  pattern: "slow query|query took.*ms"
  threshold: "50 in 5m"
  alert: dba_team
  cooldown: 600
```

## Tips and Best Practices

### 1. Start Conservative

Begin with higher thresholds and adjust down based on your observability needs:

```yaml
# Start with this
threshold: "10 in 1m"

# Adjust to this if too noisy
threshold: "20 in 1m"

# Or this if missing issues
threshold: "5 in 30s"
```

### 2. Match Threshold to Severity

```yaml
# High severity = low threshold or no threshold
- name: critical_errors
  pattern: "CRITICAL|FATAL"
  alert: oncall  # No threshold - immediate alert

# Medium severity = moderate threshold
- name: errors
  pattern: "ERROR"
  threshold: "5 in 1m"
  alert: team_slack

# Low severity = high threshold
- name: warnings
  pattern: "WARN"
  threshold: "50 in 5m"
  alert: team_slack
```

### 3. Consider Your Check Interval

For health checks, ensure your threshold window is longer than your check interval:

```yaml
# ✅ Good: 10s interval, 1m window allows 6 checks
- name: api_health
  type: http
  url: "http://localhost:8080/health"
  interval: 10
  threshold: "3 in 1m"
  alert: oncall

# ❌ Bad: 60s interval, 30s window means at most 1 check
- name: bad_example
  type: http
  url: "http://localhost:8080/health"
  interval: 60
  threshold: "2 in 30s"  # This won't work as intended!
  alert: oncall
```

### 4. Use Cooldown to Prevent Spam

Even with thresholds, use cooldown to prevent repeated alerts:

```yaml
- name: error_burst
  pattern: "ERROR"
  threshold: "5 in 2s"
  alert: team_slack
  cooldown: 300  # Don't alert again for 5 minutes
```

## See Also

- [examples/example-thresholds.yaml](examples/example-thresholds.yaml) - Complete working example
- [Configuration Documentation](README.md#configuration) - Full config reference

