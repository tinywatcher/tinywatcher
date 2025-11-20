# TinyWatcher 🚀

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
* Regex-based rules for pattern matching
* Cooldown per rule to prevent alert spam

### **Resource Monitoring**

* CPU usage alerts
* Memory usage alerts
* Disk usage alerts
* Configurable thresholds and intervals

### **Alerts**

* stdout (immediate feedback)
* Webhook (send JSON to any endpoint)
* Slack (via webhook)

### **Configuration**

* **YAML-based config** — familiar and editable by anyone
* One file can define log inputs, resource thresholds, and alert rules
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

alerts:
  slack: "https://hooks.slack.com/services/XXX"
  webhook: "https://myapi.com/alert"

rules:
  - name: server_500
    pattern: "500"
    alert: slack
    cooldown: 60

  - name: db_down
    pattern: "OperationalError"
    alert: webhook
    cooldown: 30

resources:
  interval: 10   # seconds
  thresholds:
    cpu_percent: 85
    memory_percent: 80
    disk_percent: 90
    alert: slack
```

---

## **Usage**

### Watch files:

```bash
tinywatcher watch --file /var/log/nginx/error.log
```

### Watch Docker containers:

```bash
tinywatcher watch --container nginx --container api
```

### Use a configuration file:

```bash
tinywatcher watch --config config.yaml
```

### Test rules without watching:

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

### 1. Monitor Nginx Logs for Errors

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

### 2. Monitor Docker Container + System Resources

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

### 3. Quick Test Without Config File

```bash
# Just watch a file and print matches to stdout
tinywatcher watch --file /var/log/app.log
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
