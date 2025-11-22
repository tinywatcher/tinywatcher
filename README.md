# TinyWatcher 

**A tiny, cross-platform observability tool for logs and system resources. Zero infrastructure required.**

> "Finally, observability without the dashboards, agents, or cloud lock-in."

---

## **What is TinyWatcher?**

TinyWatcher is a **single binary tool** that watches your logs and system metrics in real time and triggers alerts when things go wrong.

It's designed for:

* **Small projects & MVPs**
* **Solo developers or small teams**
* **Docker or bare-metal deployments**
* **Anyone tired of Prometheus, Datadog, or ELK for one VPS**

With TinyWatcher, you get **actionable alerts** — not dashboards.

### **Production-Ready & Robust**

Despite its tiny footprint, TinyWatcher is built for reliability:

*  **Automatic reconnection** — Network hiccups? Container restarts? TinyWatcher reconnects automatically
*  **Zero zombie processes** — Proper cleanup of all child processes, no resource leaks
*  **DoS protection** — Line length limits prevent regex attacks from pathological logs
*  **Exponential backoff** — Smart retry logic that doesn't hammer your systems
*  **Clean shutdown** — Graceful termination with proper resource cleanup
*  **Memory safe** — Bounded memory usage, no unbounded buffers

**~5-20 KB memory per monitor** • **Sub-millisecond regex matching** • **Production-tested**

---

## **Features (v1 MVP)**

### **Log Monitoring**

* Tail local log files (`/var/log/nginx/error.log`)
* Stream logs from Docker containers (`docker logs -f`)
* **⭐ Real-time log streaming** (WebSocket, HTTP, TCP) — Azure, AWS, K8s, and more!
* **⭐ Source-specific rules** — apply rules only to specific files, containers, or streams
* Regex-based rules for pattern matching
* Cooldown per rule to prevent alert spam
* **⭐ Auto-reconnection** — monitors never die, automatic retry with exponential backoff

### **Rule Testing**

* **Check command** — test rules against recent logs without real-time monitoring
* Highlights matched patterns in terminal output
* Shows which rules triggered on which lines
* Perfect for debugging rules before deployment

### **Background Service Mode**

* **⭐ Run as a system service** — persistent monitoring with auto-start on boot
* **⭐ Cross-platform** — systemd (Linux), launchd (macOS), Windows Service
* Automatic restart
* Simple management: `start`, `stop`, `restart`, `status` commands
* Perfect for production deployments

### **Resource Monitoring**

* CPU usage alerts
* Memory usage alerts
* Disk usage alerts
* Configurable thresholds and intervals

### **Alerts**

* **Named alerts** — define multiple alerts of the same type with custom names
* **Multi-destination rules** — send one rule to multiple alert destinations
* **Identity tracking** — all alerts include the instance/hostname for easy identification
* stdout (immediate feedback)
* Webhook (send JSON to any endpoint)
* Slack (via webhook)
* Email (via sendmail on Unix/macOS or SMTP)

### **Configuration**

* **YAML-based config** — familiar and editable by anyone
* **Identity management** — set custom instance names or auto-detect hostname
* One file can define log inputs, resource thresholds, and alert rules
* Support for both single alert or array of alerts per rule
* Minimal setup: drop in your YAML and run

---

## **Example YAML Configuration**

```yaml
# Optional: Set a custom identity for this instance
# If not set, hostname is auto-detected
identity:
  name: my-api-server-1

inputs:
  files:
    - /var/log/nginx/error.log
  containers:
    - nginx
    - api

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
    alert: [oncall_slack, team_slack, webhook]  # send to all channels
    cooldown: 10

resources:
  interval: 10   # seconds
  thresholds:
    cpu_percent: 85
    memory_percent: 80
    disk_percent: 90
    alert: team_slack  # references alert name
```

---

## **🌐 Log Streaming (NEW!)**

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

**📖 See [STREAMING.md](STREAMING.md) for complete documentation and examples!**

---

## **🎯 Source-Specific Rules (NEW!)**

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

**📖 See [SOURCE_FILTERING.md](SOURCE_FILTERING.md) for complete documentation!**

---

## **🏷️ Identity Management**

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

### Watch files and containers in real-time:

```bash
# Watch a specific file
tinywatcher watch --file /var/log/nginx/error.log

# Watch Docker containers
tinywatcher watch --container nginx --container api

# Use a configuration file
tinywatcher watch --config config.yaml
```

### Check rules against recent logs (with highlighted matches):

```bash
# Check last 100 lines (default) from configured sources
tinywatcher check --config config.yaml

# Check last 50 lines
tinywatcher check --config config.yaml -n 50

# Override and check specific container
tinywatcher check --config config.yaml --container tinyetl-mysql

# Check specific file
tinywatcher check --config config.yaml --file /var/log/app.log
```

### Test configuration validity:

```bash
tinywatcher test --config config.yaml
```

### Enable verbose logging:

```bash
tinywatcher watch --config config.yaml --verbose
```

---

## **🔄 Background Service Mode**

Run TinyWatcher as a persistent background service that starts automatically on boot and restarts on crashes. **Fully cross-platform** — works seamlessly on Linux (systemd), macOS (launchd), and Windows (Windows Service).

### Install and start as a background service:

```bash
# First time: Install and start the service with your config
tinywatcher start --config config.yaml
```

This will:
- ✅ Install TinyWatcher as a system service
- ✅ Start it immediately
- ✅ Configure it to start automatically on boot
- ✅ Auto-restart on crashes or failures

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

## **Why TinyWatcher?**

* **Zero infrastructure** — No DB, no agents, no cloud required
* **Truly cross-platform** — Linux, macOS, Windows (same binary, same config)
* **Background service mode** — Run as systemd/launchd/Windows Service with auto-start
* **Minimal setup** — Drop in a YAML file and run, zero ceremony
* **Single binary** — Easy to deploy in Docker, VMs, or bare metal
* **Built for small teams** — Perfect for deployments that don't need enterprise observability

---

## **Installation**

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/tinywatcher
cd tinywatcher

# Build the binary
cargo build --release

# The binary will be at target/release/tinywatcher
./target/release/tinywatcher --help
```

### From Releases

Download the latest release for your platform:

```bash
# Linux / macOS
curl -L https://github.com/yourusername/tinywatcher/releases/latest/download/tinywatcher.tar.gz | tar xz
./tinywatcher --help
```

### Docker

Run as a Docker container:

```bash
docker run -v /var/run/docker.sock:/var/run/docker.sock \
  -v $(pwd)/config.yaml:/config.yaml \
  tinywatcher watch --config /config.yaml
```

---

## **Building**

Requires Rust 1.70 or later.

```bash
cargo build --release
```

---

## **Examples**

### 1. Test Rules Before Deploying

```yaml
# config.yaml
inputs:
  containers:
    - my-app

alerts:
  console:
    type: stdout

rules:
  - name: errors
    pattern: "ERROR|FATAL"
    alert: console
    cooldown: 60
```

```bash
# First, check if your rules match any recent logs
tinywatcher check --config config.yaml -n 200

# Output shows highlighted matches:
# 📋 Testing 1 rules:
#   • errors (pattern: ERROR|FATAL)
#
# 🐳 Checking container: my-app
#   ✓ [errors]
#     2024-11-20 10:15:23 - ERROR: Connection timeout
#                           ^^^^^
#   ✓ [errors]  
#     2024-11-20 10:16:45 - FATAL: Database unavailable
#                           ^^^^^
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# ✅ Found 2 total matches

# Once satisfied, start real-time monitoring
tinywatcher watch --config config.yaml
```

### 2. Monitor Nginx Logs for Errors

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
  - name: nginx_error
    pattern: "\\[error\\]"
    alert: slack
    cooldown: 300
```

```bash
tinywatcher watch --config config.yaml
```

### 3. Monitor Docker Container + System Resources

```yaml
# config.yaml
identity:
  name: production-server-1  # Custom name for easy identification

inputs:
  containers:
    - my-app
    - postgres

alerts:
  webhook:
    type: webhook
    url: "https://api.example.com/alerts"

rules:
  - name: app_error
    pattern: "ERROR|FATAL"
    alert: webhook
    cooldown: 60

resources:
  interval: 30
  thresholds:
    cpu_percent: 80
    memory_percent: 85
    alert: webhook
```

```bash
tinywatcher watch --config config.yaml
```

### 4. Quick Test Without Config File

```bash
# Just watch a file (note: no rules means no alerts)
tinywatcher watch --file /var/log/app.log
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

---

## **Planned Enhancements (Post-MVP)**

* JSON/structured log support
* Multi-channel alerts (PagerDuty, Telegram, Email)
* Auto-discover containers and logs
* Simple local dashboard
* Anomaly detection for spikes in logs or resources

---

## **Philosophy**

TinyWatcher exists because most small deployments **don't need enterprise observability** — they need **actionable alerts with zero setup**.

It's inspired by TinyETL: **small, focused, reliable, and practical.**

---

## **License**

MIT

---

## **Contributing**

Contributions are welcome! Please feel free to submit a Pull Request.

---

## **Support**

If you encounter any issues or have questions, please file an issue on GitHub.
