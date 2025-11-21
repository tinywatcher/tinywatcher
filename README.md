# TinyWatcher 

**A tiny, zero-infrastructure observability tool for logs and system resources.**

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

---

## **Features (v1 MVP)**

### **Log Monitoring**

* Tail local log files (`/var/log/nginx/error.log`)
* Stream logs from Docker containers (`docker logs -f`)
* **⭐ Real-time log streaming** (WebSocket, HTTP, TCP) — Azure, AWS, K8s, and more!
* Regex-based rules for pattern matching
* Cooldown per rule to prevent alert spam

### **Rule Testing**

* **Check command** — test rules against recent logs without real-time monitoring
* Highlights matched patterns in terminal output
* Shows which rules triggered on which lines
* Perfect for debugging rules before deployment

### **Resource Monitoring**

* CPU usage alerts
* Memory usage alerts
* Disk usage alerts
* Configurable thresholds and intervals

### **Alerts**

* **Named alerts** — define multiple alerts of the same type with custom names
* **Multi-destination rules** — send one rule to multiple alert destinations
* stdout (immediate feedback)
* Webhook (send JSON to any endpoint)
* Slack (via webhook)

### **Configuration**

* **YAML-based config** — familiar and editable by anyone
* One file can define log inputs, resource thresholds, and alert rules
* Support for both single alert or array of alerts per rule
* Minimal setup: drop in your YAML and run

---

## **Example YAML Configuration**

```yaml
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

## **Why TinyWatcher?**

* No DB, no agents, no cloud required
* Cross-platform: Linux, macOS, Windows
* Minimal setup, zero ceremony
* Designed for small deployments that don't need enterprise observability
* Single binary, easy to run in Docker or on host

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

rules:
  - name: errors
    pattern: "ERROR|FATAL"
    alert: stdout
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
  slack: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

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
inputs:
  containers:
    - my-app
    - postgres

alerts:
  webhook: "https://api.example.com/alerts"

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
