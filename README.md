<p align="center">
  <img src="logo.svg" alt="TinyWatcher Logo" width="150">
</p>

# TinyWatcher 

<p align="center">
  <img src="https://img.shields.io/badge/version-0.2.0-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/binary-<10MB-green.svg" alt="Binary Size">
  <img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust">
  <img src="https://img.shields.io/badge/license-MIT-brightgreen.svg" alt="MIT License">
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg" alt="Platform Support">
</p>



**A free, open-source, lightweight, single-binary log and system monitor designed for developers who need simple, actionable alerts without the complexity or cost of enterprise observability stacks.**

✨ **100% Free Forever** • **MIT Licensed** • **No Vendor Lock-in**

![TinyWatcher Demo](tinywatcher-demo.gif)

*Demo showing real-time Slack alerts from TinyWatcher monitoring a log file, followed by the test suite validating config and rule matching on real logs.*



> "Finally, observability without the dashboards, agents, or cloud lock-in."

---

## **What is TinyWatcher?**

TinyWatcher is a **single binary tool** that watches your logs and system metrics in real time and triggers alerts when things go wrong.

### **Why TinyWatcher?**

Most monitoring solutions are overkill for small projects. TinyWatcher fills the gap between `tail -f` and full-blown observability platforms.

Perfect for:

* **Side projects and MVPs**
* **Small production deployments**
* **Development and staging environments**
* **Quick debugging and incident response**

### **Philosophy**

TinyWatcher believes in **alerts over dashboards**. Instead of building pretty graphs you never look at, TinyWatcher sends you alerts when something goes wrong. It's designed to be set up in minutes and forgotten until it saves your weekend.

**~5-20 KB memory per monitor** • **Sub-millisecond regex matching** 

---

## **Key Features**

### **Single Binary**

No agents, no databases, no complicated setup. Just one binary to deploy!

### **Multiple Inputs**

* Tail local log files (`/var/log/nginx/error.log`)
* **Glob patterns** — monitor multiple files with wildcards (`/var/log/app/*.log`)
* Stream logs from Docker containers (`docker logs -f`)
* **Real-time log streaming** (WebSocket, HTTP, TCP) — Azure, AWS, K8s, and more!
* **Source-specific rules** — apply rules only to specific files, containers, or streams

### **Flexible Alerts**

Send to Discord, Telegram, Slack, PagerDuty, Ntfy.sh, Webhooks, Email, SendGrid, or stdout

* **Named alerts** — define multiple alerts of the same type with custom names
* **Multi-destination rules** — send one rule to multiple alert destinations
* **Identity tracking** — all alerts include the instance/hostname for easy identification

### **Regex Patterns**

* Match any log pattern with regex or exact text matching
* Cooldown per rule to prevent alert spam
* Case-insensitive matching by default

### **Resource Monitoring**

* Track CPU, memory, and disk usage
* Configurable thresholds and intervals
* Get alerted before things break

### **Health Checks**

* Monitor HTTP endpoints for availability
* Configurable check intervals and timeouts
* Failure thresholds to avoid false positives
* Recovery alerts when services come back online
* Perfect for monitoring APIs, databases, and microservices

### **Runs as Service**

* Install as systemd, launchd, or Windows service
* Automatic restart on crashes
* Start automatically on boot
* Simple management: `start`, `stop`, `restart`, `status` commands

<!-- ### **Optional Heartbeat Monitoring**

Get alerted if TinyWatcher itself stops running (paid service) — because who monitors the monitor? -->

### **Production-Ready & Robust**

* **Automatic reconnection** — Network hiccups? Container restarts? TinyWatcher reconnects automatically
* **Zero zombie processes** — Proper cleanup of all child processes, no resource leaks
* **DoS protection** — Line length limits prevent regex attacks from pathological logs
* **Exponential backoff** — Smart retry logic that doesn't hammer your systems
* **Clean shutdown** — Graceful termination with proper resource cleanup
* **Memory safe** — Bounded memory usage, no unbounded buffers

**~5-20 KB memory per monitor** • **Sub-millisecond regex matching** • **Production-tested**

### **Configuration**

* **YAML-based config** — familiar and editable by anyone
* **Identity management** — set custom instance names or auto-detect hostname
* **Environment variable support** — secure handling of secrets and credentials
* One file can define log inputs, resource thresholds, and alert rules
* Support for both single alert or array of alerts per rule
* Minimal setup: drop in your YAML and run

---

## **Installation**

### **Binary Download**

**Linux (x86_64):**
```bash
curl -L https://github.com/tinywatcher/tinywatcher/releases/latest/download/tinywatcher-linux-x86_64.tar.gz -o tinywatcher-linux-x86_64.tar.gz
tar -xzf tinywatcher-linux-x86_64.tar.gz
chmod +x tinywatcher
sudo mv tinywatcher /usr/local/bin/
```

**Linux (ARM64):**
```bash
curl -L https://github.com/tinywatcher/tinywatcher/releases/latest/download/tinywatcher-linux-aarch64.tar.gz -o tinywatcher-linux-aarch64.tar.gz
tar -xzf tinywatcher-linux-aarch64.tar.gz
chmod +x tinywatcher
sudo mv tinywatcher /usr/local/bin/
```

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/tinywatcher/tinywatcher/releases/latest/download/tinywatcher-macos-aarch64.tar.gz -o tinywatcher-macos-aarch64.tar.gz
tar -xzf tinywatcher-macos-aarch64.tar.gz
chmod +x tinywatcher
sudo mv tinywatcher /usr/local/bin/
```

**macOS (Intel):**
```bash
curl -L https://github.com/tinywatcher/tinywatcher/releases/latest/download/tinywatcher-macos-x86_64.tar.gz -o tinywatcher-macos-x86_64.tar.gz
tar -xzf tinywatcher-macos-x86_64.tar.gz
chmod +x tinywatcher
sudo mv tinywatcher /usr/local/bin/
```

**Windows (x86_64) (PowerShell):**
```powershell
Invoke-WebRequest -Uri https://github.com/tinywatcher/tinywatcher/releases/latest/download/tinywatcher-windows-x86_64.exe.tar.gz -OutFile tinywatcher-windows-x86_64.exe.tar.gz
tar -xzf tinywatcher-windows-x86_64.exe.tar.gz
```

**Windows (ARM64) (PowerShell):**
```powershell
Invoke-WebRequest -Uri https://github.com/tinywatcher/tinywatcher/releases/latest/download/tinywatcher-windows-aarch64.exe.tar.gz -OutFile tinywatcher-windows-aarch64.exe.tar.gz
tar -xzf tinywatcher-windows-aarch64.exe.tar.gz
```

### **Docker**

> **Docker support is coming soon!**

### **Build From Source**

Requires Rust and Cargo installed. [Install Rust here](https://rustup.rs/).

```bash
git clone https://github.com/tinywatcher/tinywatcher
cd tinywatcher
cargo build --release
sudo cp target/release/tinywatcher /usr/local/bin/
```

---

## **First 60 Seconds**

Get up and running in under a minute:

```bash
# 1. Create a minimal config
cat > config.yaml << EOF
inputs:
  files:
    - /var/log/nginx/error.log

alerts:
  slack:
    type: slack
    url: "YOUR_SLACK_WEBHOOK_URL"

rules:
  - name: nginx_errors
    pattern: "error|crit"
    alert: slack
    cooldown: 300
EOF

# 2. Test it
tinywatcher check --config config.yaml

# 3. Start watching
tinywatcher watch --config config.yaml
```

---

## **Configuration Guide**

### **Complete Example**

```yaml
# Optional: Set a custom identity for this instance
# If not set, hostname is auto-detected
identity:
  name: my-api-server-1

# Log sources
inputs:
  files:
    - /var/log/nginx/error.log
    - /var/log/app/*.log           # Glob: all .log files
    - /var/log/services/*/error.log # Glob: error.log from all services
  containers:
    - nginx
    - api
  streams:
    - name: azure_webapp
      type: websocket
      url: "wss://myapp.scm.azurewebsites.net/api/logstream"

# Define named alerts - you can have multiple of the same type!
alerts:
  console:
    type: stdout
  
  team_slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/TEAM/WEBHOOK"
  
  oncall_slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/ONCALL/WEBHOOK"
  
  telegram_alerts:
    type: telegram
    bot_token: "${TELEGRAM_BOT_TOKEN}"
    chat_id: "${TELEGRAM_CHAT_ID}"
  
  discord_alerts:
    type: discord
    url: "${DISCORD_WEBHOOK_URL}"
  
  pagerduty_oncall:
    type: pagerduty
    routing_key: "${PAGERDUTY_KEY}"
  
  ntfy_alerts:
    type: ntfy
    topic: "tinywatcher-alerts"
  
  webhook:
    type: webhook
    url: "https://myapi.com/alert"

# Rules reference alert names - single or multiple!
rules:
  - name: server_500
    pattern: "500"
    alert: team_slack  # single alert
    cooldown: 60

  - name: db_down
    pattern: "OperationalError"
    alert: [oncall_slack, webhook]  # multiple alerts!
    cooldown: 30
  
  - name: critical_error
    pattern: "CRITICAL|FATAL"
    alert: [pagerduty_oncall, telegram_alerts, discord_alerts]  # send to all channels
    cooldown: 10
  
  # Use exact text matching instead of regex
  - name: auth_failures
    text: "authentication failed"
    alert: team_slack
    cooldown: 120

# Resource monitoring
resources:
  interval: 10   # seconds
  thresholds:
    cpu_percent: 85
    memory_percent: 80
    disk_percent: 90
    alert: team_slack  # can also be an array

# Health checks
system_checks:
  - name: api_health
    type: http
    url: "http://localhost:8080/health"
    interval: 30        # Check every 30 seconds
    timeout: 5          # Request timeout in seconds
    missed_threshold: 2 # Alert after 2 consecutive failures
    alert: oncall_slack

  - name: database
    type: http
    url: "http://localhost:5432/health"
    interval: 60
    missed_threshold: 3
    alert: [team_slack, pagerduty_oncall]  # multiple destinations

# Optional: Heartbeat monitoring (paid service)
# heartbeat:
#   url: "https://heartbeat.tinywatcher.com/ping/your-unique-id"
#   interval: 60  # Send heartbeat every 60 seconds
```

### **Environment Variables**

All configurations support environment variable expansion for security. This is especially important for sensitive data like API tokens, webhook URLs, and credentials.

```yaml
alerts:
  telegram-alerts:
    type: telegram
    bot_token: "${TELEGRAM_BOT_TOKEN}"
    chat_id: "${TELEGRAM_CHAT_ID}"
  
  discord-alerts:
    type: discord
    url: "${DISCORD_WEBHOOK_URL}"
  
  ntfy-alerts:
    type: ntfy
    topic: "tinywatcher-${HOSTNAME}"
```

Environment variables can be used in any string value throughout your configuration file. Simply use the `${VAR_NAME}` syntax, and TinyWatcher will replace it with the actual value at runtime.

---

## **Log Streaming (NEW!)**

TinyWatcher now supports real-time log streaming from cloud services and custom endpoints!

```yaml
inputs:
  streams:
    # Azure App Service logs
    - name: azure_webapp
      type: websocket
      url: "wss://myapp.scm.azurewebsites.net/api/logstream/application"
      headers:
        Authorization: "Bearer YOUR_TOKEN"
    
    # Kubernetes pod logs
    - name: k8s_pod
      type: websocket
      url: "wss://k8s-api:443/api/v1/namespaces/default/pods/my-pod/log?follow=true"
    
    # Custom TCP syslog
    - name: syslog
      type: tcp
      url: "localhost:514"
```

---

## **Glob Patterns for Files (NEW!)**

Monitor multiple log files with a single pattern — perfect for rotating logs and microservices!

```yaml
inputs:
  files:
    # All .log files in a directory
    - "/var/log/app/*.log"
    
    # Rotating logs with dates
    - "/var/log/myapp-*.log"         # myapp-2024-12-01.log, myapp-2024-12-02.log
    
    # Error logs from all services
    - "/var/log/services/*/error.log"  # auth/error.log, api/error.log, worker/error.log
    
    # Single character wildcard
    - "/var/log/app-?.log"           # app-1.log, app-2.log (but not app-10.log)
    
    # Character classes
    - "/var/log/app-[123].log"       # app-1.log, app-2.log, app-3.log
```

**Supported patterns:**
- `*` — Matches any characters (e.g., `*.log`)
- `?` — Matches exactly one character (e.g., `app-?.log`)
- `[...]` — Matches one character from set (e.g., `[0-9]` or `[abc]`)

**How it works:**
- Patterns are expanded at startup to actual files
- Each matched file gets its own watcher
- Only files (not directories) are monitored
- Logs show: `INFO: Glob pattern '/var/log/app/*.log' matched 3 file(s)`

See [GLOB_PATTERNS.md](GLOB_PATTERNS.md) for detailed documentation and examples.

---

## **Source-Specific Rules (NEW!)**

Apply rules only to specific sources for better organization and performance:

```yaml
rules:
  # Only check API container for 500 errors
  - name: api_500s
    pattern: "500|Internal Server Error"
    sources:
      containers: ["api"]
      streams: ["azure_app_service"]
    alert: team_slack
    cooldown: 60

  # Only check postgres container for database errors
  - name: postgres_errors
    pattern: "FATAL|PANIC"
    sources:
      containers: ["postgres"]
    alert: oncall_slack
    cooldown: 30

  # Only check nginx logs for auth failures
  - name: nginx_auth_failures
    pattern: "auth.*failed"
    sources:
      files: ["/var/log/nginx/error.log"]
    alert: security_webhook
    cooldown: 120

  # Rule with no sources filter - applies to ALL inputs
  - name: critical_errors
    pattern: "CRITICAL"
    alert: oncall_slack
    cooldown: 60
```

---

## **Identity Management**

TinyWatcher automatically identifies which server or instance sent each alert:

```yaml
# Set a custom identity for this instance
identity:
  name: my-api-server-1
```

If you don't set an identity, TinyWatcher will **auto-detect the hostname**.

**How it appears in alerts:**

- **Slack**: Includes host name in the alert message
- **Webhook**: JSON payload includes `"identity": "my-api-server-1"`
- **Email**: Subject and body include the hostname
- **Stdout**: Timestamp shows `[timestamp] [identity] [rule] message`

This is especially useful when monitoring multiple servers with the same config file!

---

## **Usage**

### Watch Mode

Start monitoring with your configuration:

```bash
tinywatcher watch --config config.yaml
```

Disable resource monitoring:

```bash
tinywatcher watch --config config.yaml --no-resources
```

### Check Mode

Test your rules against recent log entries with highlighted matches:

```bash
# Check last 100 lines (default)
tinywatcher check --config config.yaml

# Check last 50 lines
tinywatcher check --config config.yaml -n 50

# Check specific files only
tinywatcher check --config config.yaml --file /var/log/app.log

# Check specific containers only
tinywatcher check --config config.yaml --container myapp
```

### Test Mode

Validate your configuration without starting monitoring:

```bash
tinywatcher test --config config.yaml
```

This will:
- Validate all configuration syntax
- Check that referenced alerts exist
- Verify regex patterns compile
- Display a summary of all rules and alerts

---

## **Daemon Mode**

Run TinyWatcher as a persistent background service that starts automatically on boot and restarts on crashes. **Fully cross-platform** — works seamlessly on Linux (systemd), macOS (launchd), and Windows (Windows Service).

### Start as a background service:

```bash
# Start the daemon
tinywatcher start --config config.yaml
```

This will:
- Install TinyWatcher as a system service
- Start it immediately
- Configure it to start automatically on boot
- Auto-restart on crashes or failures

### Manage the service:

```bash
# Check if the service is running
tinywatcher status

# Stop the service
tinywatcher stop

# Restart the service (stop + start)
tinywatcher restart

# On Linux: View logs
journalctl --user -u tinywatcher -f

# On macOS: View logs
tail -f /tmp/tinywatcher.log
```

### Platform-specific details:

| Platform | Service Manager | Log Location |
|----------|----------------|--------------|
| **Linux** | systemd (user service) | `journalctl --user -u tinywatcher` |
| **macOS** | launchd (LaunchAgent) | `/tmp/tinywatcher.log` |
| **Windows** | Windows Service | Event Viewer or `services.msc` |

**Perfect for:**
- Production servers that need 24/7 monitoring
- Servers that restart frequently
- "Set it and forget it" deployments
- Running on VPS, bare metal, or cloud instances

---

## **Alert Destinations**

TinyWatcher supports multiple alert destinations for maximum flexibility.

### **Discord**

Send alerts to Discord channels using webhooks.

**Setup:**
1. Open Discord server settings
2. Navigate to Integrations → Webhooks
3. Create/copy webhook URL

```yaml
alerts:
  my-discord:
    type: discord
    url: "https://discord.com/api/webhooks/123456789/your-webhook-token"
```

### **Telegram**

Send notifications via Telegram Bot API.

**Setup:**
1. Talk to @BotFather on Telegram: `/newbot`
2. Copy the bot token
3. Get your chat ID from `https://api.telegram.org/bot<BOT_TOKEN>/getUpdates`

```yaml
alerts:
  telegram-alerts:
    type: telegram
    bot_token: "123456789:ABCdefGHIjklMNOpqrsTUVwxyz"
    chat_id: "987654321"
```

### **PagerDuty**

Enterprise incident management using Events API v2.

**Setup:**
1. Go to Services → Service Directory
2. Select/create service → Integrations tab
3. Add Events API V2 integration
4. Copy the Integration Key

```yaml
alerts:
  pagerduty-oncall:
    type: pagerduty
    routing_key: "your-integration-key-here"
```

### **Ntfy.sh**

Simple push notifications with no authentication required.

**Setup:**
1. Choose a unique topic name
2. Subscribe on your device (iOS/Android/Web)
3. Start receiving notifications!

```yaml
alerts:
  ntfy-public:
    type: ntfy
    topic: "tinywatcher-alerts-xyz123"
  
  # Or self-hosted
  ntfy-private:
    type: ntfy
    topic: "alerts"
    server: "https://ntfy.example.com"
```

⚠️ **Security Note:** Choose a unique, hard-to-guess topic name for public ntfy.sh server.

### **Slack**

Send alerts to Slack channels using webhooks.

```yaml
alerts:
  slack-team:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
```

### **Webhook**

Generic webhook for custom integrations.

```yaml
alerts:
  custom-webhook:
    type: webhook
    url: "https://your-service.com/webhook"
```

**Payload Format:**
```json
{
  "identity": "hostname",
  "rule": "rule-name",
  "message": "log message",
  "timestamp": "2025-11-28T12:00:00Z",
  "alert_name": "custom-webhook"
}
```

### **Email**

Send alerts via email using sendmail (Unix) or SMTP.

```yaml
# Unix (sendmail)
alerts:
  email-admin:
    type: email
    from: "alerts@example.com"
    to:
      - "admin@example.com"

# Windows/SMTP
alerts:
  email-admin:
    type: email
    from: "alerts@example.com"
    to:
      - "admin@example.com"
    smtp_server: "smtp.gmail.com:587"
```

### **SendGrid**

Send alerts via SendGrid's API (requires API key).

```yaml
alerts:
  sendgrid-team:
    type: sendgrid
    api_key: "${SENDGRID_API_KEY}"
    from: "alerts@yourdomain.com"  # Must be verified in SendGrid
    to:
      - "team@example.com"
      - "oncall@example.com"
```

**Setup:**
1. Sign up at [SendGrid](https://sendgrid.com)
2. Create an API key in Settings > API Keys
3. Verify your sender email/domain
4. Set `SENDGRID_API_KEY` environment variable

### **Stdout**

Output to console (useful for testing).

```yaml
alerts:
  console:
    type: stdout
```

---

<!-- ## **Heartbeat Monitoring**

💡 **Who monitors the monitor?**

Our optional heartbeat monitoring service ensures TinyWatcher itself is running. If your TinyWatcher instance stops sending heartbeats, you'll receive an alert.

### **Why You Need This**

* **Silent Failures** — If TinyWatcher crashes, you won't get alerts
* **Configuration Errors** — Misconfigurations might prevent startup
* **Server Issues** — Network/resource problems could stop monitoring
* **Peace of Mind** — Know your monitoring is working 24/7

### **Setup**

1. Sign up at [tinywatcher.com/heartbeat](https://tinywatcher.com/heartbeat)
2. Get your unique heartbeat URL
3. Add to config:

```yaml
heartbeat:
  url: "https://heartbeat.tinywatcher.com/ping/your-unique-id"
  interval: 60  # Send heartbeat every 60 seconds
```

4. Configure alert preferences (Discord, Telegram, Slack, Email, etc.)
5. Set your threshold (e.g., alert if no heartbeat for 5 minutes)

### **Pricing**

**Heartbeat monitoring is a paid service** that helps support TinyWatcher's development while keeping the core tool 100% free.

| Plan | Price | Features |
|------|-------|----------|
| **Hobby** | $5/month | Up to 3 instances, 1-min intervals, Discord/Telegram/Email alerts |
| **Pro** | $15/month | Unlimited instances, 30-sec intervals, all alert channels, priority support |

✅ **14-day free trial** • **Supporting open source**

[Get Started with Heartbeat →](https://tinywatcher.com/heartbeat) -->

---

## **Security**

### **Handling Secrets**

Never commit webhook URLs or API tokens to version control. Use environment variables:

```yaml
alerts:
  slack:
    type: slack
    url: "${SLACK_WEBHOOK_URL}"
  
  telegram:
    type: telegram
    bot_token: "${TELEGRAM_BOT_TOKEN}"
    chat_id: "${TELEGRAM_CHAT_ID}"
```

Or store config outside your repo:
```bash
tinywatcher watch --config /etc/tinywatcher/config.yaml
```

### **File Permissions**

Protect your config file:

```bash
chmod 600 config.yaml
chown $USER:$USER config.yaml
```

### **Production Best Practices**

* Run as a dedicated user with minimal permissions
* Only grant Docker socket access if monitoring containers
* Use read-only volume mounts in Docker
* Rotate webhook URLs periodically
<!-- * Monitor TinyWatcher itself with Heartbeat Monitoring -->

---

## **Examples**

### 1. Monitor Rotating Logs with Glob Patterns

```yaml
# config.yaml
inputs:
  files:
    # Monitor all rotating log files
    - "/var/log/myapp/app-*.log"
    # Monitor error logs from all microservices
    - "/var/log/services/*/error.log"

alerts:
  slack:
    type: slack
    url: "${SLACK_WEBHOOK_URL}"

rules:
  - name: errors
    pattern: "ERROR|FATAL"
    alert: slack
    cooldown: 300
```

```bash
# Check which files match your patterns
tinywatcher check --config config.yaml

# Output shows:
# INFO: Glob pattern '/var/log/myapp/app-*.log' matched 5 file(s)
# INFO: Glob pattern '/var/log/services/*/error.log' matched 3 file(s)
# Starting file watch: /var/log/myapp/app-2024-12-01.log
# Starting file watch: /var/log/myapp/app-2024-12-02.log
# ...

# Start monitoring
tinywatcher watch --config config.yaml
```

### 2. Test Rules Before Deploying

```yaml
# config.yaml
inputs:
  containers:
    - my-app

alerts:
  console:
    type: stdout

rules:
  # Regex pattern matching
  - name: errors
    pattern: "ERROR|FATAL"
    alert: console
    cooldown: 60
  
  # Exact text matching
  - name: auth_failure
    text: "authentication failed"
    alert: console
    cooldown: 120
```

```bash
# First, check if your rules match any recent logs
tinywatcher check --config config.yaml -n 200

# Output shows highlighted matches:
# Testing 2 rules:
#   - errors (pattern: ERROR|FATAL)
#   - auth_failure (text: authentication failed)
#
# Checking container: my-app
#   [errors]
#     2024-11-20 10:15:23 - ERROR: Connection timeout
#                           ^^^^^
#   [errors]  
#     2024-11-20 10:16:45 - FATAL: Database unavailable
#                           ^^^^^
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Found 2 total matches

# Once satisfied, start real-time monitoring
tinywatcher watch --config config.yaml
```

### 3. Monitor Nginx Logs for Errors

```yaml
# config.yaml
inputs:
  files:
    - /var/log/nginx/error.log

alerts:
  slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

rules:
  # Use regex pattern
  - name: nginx_error
    pattern: "\\[error\\]|\\[crit\\]"
    alert: slack
    cooldown: 300
```

```bash
tinywatcher watch --config config.yaml
```

### 4. Multi-Destination Critical Alerts

```yaml
# config.yaml
identity:
  name: production-server-1

inputs:
  containers:
    - my-app
    - postgres

alerts:
  pagerduty_oncall:
    type: pagerduty
    routing_key: "${PAGERDUTY_KEY}"
  
  telegram_personal:
    type: telegram
    bot_token: "${TELEGRAM_BOT_TOKEN}"
    chat_id: "${TELEGRAM_CHAT_ID}"
  
  discord_team:
    type: discord
    url: "${DISCORD_WEBHOOK_URL}"

rules:
  # Critical errors go to all channels for redundancy
  - name: critical_error
    pattern: "CRITICAL|FATAL|PANIC"
    alert: [pagerduty_oncall, telegram_personal, discord_team]
    cooldown: 60
  
  # Regular errors only to team chat
  - name: app_error
    pattern: "ERROR"
    alert: discord_team
    cooldown: 300

resources:
  interval: 30
  thresholds:
    cpu_percent: 90
    memory_percent: 85
    alert: [pagerduty_oncall, telegram_personal]  # Critical resource alerts
```

```bash
tinywatcher watch --config config.yaml
```

### 5. Debug Your Rules

```bash
# Check if your regex patterns work correctly
tinywatcher check --config config.yaml --container my-app -n 1000

# The output will show you exactly what matched and where
```

### 6. Production Deployment with Background Service

```yaml
# production-config.yaml
identity:
  name: prod-server-us-east-1

inputs:
  files:
    - /var/log/nginx/error.log
    - /var/log/app/error.log
  containers:
    - api
    - worker

alerts:
  oncall_slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/ONCALL/WEBHOOK"

rules:
  - name: critical_errors
    pattern: "CRITICAL|FATAL|PANIC"
    alert: oncall_slack
    cooldown: 60

resources:
  interval: 30
  thresholds:
    cpu_percent: 90
    memory_percent: 85
    disk_percent: 95
    alert: oncall_slack
```

```bash
# Install and start as a background service
tinywatcher start --config production-config.yaml

# Verify it's running
tinywatcher status

# The service will now:
# - Start automatically on server boot
# - Restart automatically if it crashes
# - Monitor your logs and resources 24/7
# - Send alerts to your configured channels

# View real-time logs (Linux)
journalctl --user -u tinywatcher -f

# View real-time logs (macOS)
tail -f /tmp/tinywatcher.log
```

### 8. Complete Monitoring Stack with Health Checks

```yaml
# complete-monitoring.yaml
identity:
  name: api-server-prod-1

alerts:
  critical_slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/CRITICAL/WEBHOOK"
  
  ops_slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/OPS/WEBHOOK"

# Monitor application logs
inputs:
  files:
    - /var/log/app/error.log
  containers:
    - api-service
    - background-worker

rules:
  - name: critical_errors
    pattern: "CRITICAL|FATAL|PANIC"
    alert: critical_slack
    cooldown: 30
  
  - name: errors
    pattern: "ERROR"
    alert: ops_slack
    cooldown: 300

# Monitor system resources
resources:
  interval: 30
  thresholds:
    cpu_percent: 90
    memory_percent: 85
    disk_percent: 95
    alert: critical_slack

# Monitor service health
system_checks:
  - name: main_api
    type: http
    url: "http://localhost:8080/health"
    interval: 30
    timeout: 5
    missed_threshold: 2
    alert: critical_slack
  
  - name: worker_health
    type: http
    url: "http://localhost:9090/health"
    interval: 60
    timeout: 10
    missed_threshold: 3
    alert: ops_slack
  
  - name: postgres
    type: http
    url: "http://localhost:5432/health"
    interval: 30
    missed_threshold: 2
    alert: critical_slack
```

```bash
# Deploy complete monitoring
tinywatcher start --config complete-monitoring.yaml

# This single config monitors:
# - Application logs for errors
# - System CPU, memory, and disk usage
# - HTTP health of all services
# - Sends alerts to appropriate Slack channels
# - Runs 24/7 with automatic restart
```

---

## **Troubleshooting**

### **Regex patterns not matching**

Use the `check` command to test patterns against real logs:

```bash
tinywatcher check --config config.yaml -n 200
```

Remember that patterns are case-insensitive by default. Escape special regex characters like `[ ] ( ) . * +`

### **Docker permission denied**

Add your user to the docker group:

```bash
sudo usermod -aG docker $USER
# Requires logout/login
```

### **Service won't start**

Check logs:

```bash
# Linux
journalctl --user -u tinywatcher -f

# macOS
tail -f /tmp/tinywatcher.log
```

Verify config:
```bash
tinywatcher test --config config.yaml
```

### **Not receiving alerts**

* Test webhook URL with `curl`
* Verify cooldown period hasn't been triggered
* Check that rule pattern actually matches your logs with `check` mode
* Ensure environment variables are set correctly

### **High CPU usage**

* Reduce number of monitored sources
* Increase cooldown periods
* Avoid overly complex regex patterns
* Use `text` matching instead of `pattern` for exact matches

---

## **Planned Enhancements (Post-MVP)**

* JSON/structured log support
* Auto-discover containers and logs
* Simple local dashboard
* Anomaly detection for spikes in logs or resources

---

## **License**

MIT

---

## **Contributing**

Contributions are welcome! Please feel free to submit a Pull Request.

---

## **Support**

If you encounter any issues or have questions, please file an issue on [GitHub](https://github.com/tinywatcher/tinywatcher).
