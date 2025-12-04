# TinyWatcher

<p align="center">
  <img src="logo.svg" alt="TinyWatcher Logo" width="128" height="128">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.2.0-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/binary-<10MB-green.svg" alt="Binary Size">
  <img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust">
  <img src="https://img.shields.io/badge/license-MIT-brightgreen.svg" alt="MIT License">
  <img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey.svg" alt="Platform Support">
</p>

<p align="center">
  <strong>A tiny, cross-platform observability tool for logs and system resources. Zero infrastructure required.</strong>
</p>

<p align="center">
  <a href="https://tinywatcher.com">Website</a> •
  <a href="https://tinywatcher.com/docs">Documentation</a> •
  <a href="https://tinywatcher.com/getting-started">Getting Started</a> •
  <a href="https://tinywatcher.com/config-builder">Config Builder</a> •
  <a href="https://tinywatcher.com/heartbeat">Heartbeat</a>
</p>

---

## What is TinyWatcher?

TinyWatcher is a **single binary tool** that watches your logs and system metrics in real time and triggers alerts when things go wrong.

Perfect for:
- Small projects & MVPs
- Solo developers or small teams
- Docker or bare-metal deployments
- Anyone tired of Prometheus, Datadog, or ELK for one VPS

**~5-20 KB memory per monitor** • **Sub-millisecond regex matching** • **Production-tested**

---

## Quick Start

```bash
# Create a config file
cat > config.yaml << EOF
inputs:
  files:
    - /var/log/nginx/error.log
  containers:
    - myapp

alerts:
  slack:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/WEBHOOK"

rules:
  - name: server_errors
    pattern: "500|ERROR"
    alert: slack
    cooldown: 60
EOF

# Start monitoring
tinywatcher watch --config config.yaml

# Or run as a background service
tinywatcher start --config config.yaml
```

---

## Key Features

- **Log Monitoring** — Files, Docker containers, and real-time streams (WebSocket, HTTP, TCP)
- **Health Checks** — HTTP endpoint monitoring with failure detection
- **Resource Monitoring** — CPU, memory, and disk usage alerts
- **Background Service** — Run as systemd/launchd/Windows Service with auto-start
- **Smart Alerts** — Stdout, Slack, Webhook, Email with cooldowns and multi-destination support
- **Auto-Reconnection** — Never miss logs, automatic retry with exponential backoff
- **Source Filtering** — Apply rules only to specific files, containers, or streams

---

## Installation

### From Source

```bash
git clone https://github.com/yourusername/tinywatcher
cd tinywatcher
cargo build --release
./target/release/tinywatcher --help
```

### Docker

```bash
docker run -v /var/run/docker.sock:/var/run/docker.sock \
  -v $(pwd)/config.yaml:/config.yaml \
  tinywatcher watch --config /config.yaml
```

---

## Documentation

- [Full Documentation](https://tinywatcher.com/docs)
- [Getting Started Guide](https://tinywatcher.com/getting-started)
- [Heartbeat Monitoring](https://tinywatcher.com/heartbeat)
- [Complete README](README-FULL.md) (extensive examples and details)

---

## Example Configuration

```yaml
identity:
  name: production-server-1

inputs:
  files:
    - /var/log/app/error.log
  containers:
    - api
  streams:
    - name: azure_logs
      type: websocket
      url: "wss://myapp.scm.azurewebsites.net/api/logstream"

alerts:
  oncall:
    type: slack
    url: "https://hooks.slack.com/services/YOUR/WEBHOOK"

rules:
  - name: critical_errors
    pattern: "CRITICAL|FATAL"
    alert: oncall
    cooldown: 30

resources:
  interval: 30
  thresholds:
    cpu_percent: 90
    memory_percent: 85
    disk_percent: 95
    alert: oncall

system_checks:
  - name: api_health
    type: http
    url: "http://localhost:8080/health"
    interval: 30
    missed_threshold: 2
    alert: oncall
```

---

## Why TinyWatcher?

- **Zero infrastructure** — No DB, no agents, no cloud required
- **Single binary** — Easy to deploy anywhere
- **Cross-platform** — Linux, macOS, Windows
- **Production-ready** — Auto-reconnect, DoS protection, clean shutdown
- **Minimal setup** — Drop in a YAML file and run

---

## License

MIT

---

## Support

- [tinywatcher.com](https://tinywatcher.com)
- [GitHub Issues](https://github.com/yourusername/tinywatcher/issues)
- [Documentation](https://tinywatcher.com/docs)
