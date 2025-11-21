# TinyWatcher Streaming Feature

## Overview

TinyWatcher now supports **real-time log streaming** from various sources, making it perfect for cloud environments, Kubernetes, and any system where logs aren't just local files or containers.

## Supported Stream Types

### 1. **WebSocket** 
Perfect for Azure App Service, Kubernetes pod logs, and other WebSocket-based log streams.

```yaml
streams:
  - name: azure_app_service
    type: websocket
    url: "wss://myapp.scm.azurewebsites.net/api/logstream/application"
    headers:
      Authorization: "Bearer YOUR_TOKEN_HERE"
    reconnect_delay: 5
```

### 2. **HTTP Streaming** (SSE/Long-polling)
For CloudWatch Logs, HTTP event streams, and Server-Sent Events.

```yaml
streams:
  - name: cloudwatch_stream
    type: http
    url: "https://logs.us-east-1.amazonaws.com/stream"
    headers:
      X-Amz-Target: "Logs_20140328.GetLogEvents"
      Content-Type: "application/x-amz-json-1.1"
    reconnect_delay: 10
```

### 3. **TCP** 
For raw TCP log streams, syslog, or custom log aggregators.

```yaml
streams:
  - name: syslog_tcp
    type: tcp
    url: "localhost:514"
    reconnect_delay: 3
```

## Configuration

All stream configurations support:

- **`name`** (optional): Identifier for the stream in logs/alerts
- **`type`**: One of `websocket`, `http`, or `tcp`
- **`url`**: The endpoint URL/address
- **`headers`** (optional): Custom HTTP headers for authentication
- **`reconnect_delay`** (optional): Seconds to wait before reconnecting (default: 5)

## Usage

### Basic Example

```yaml
inputs:
  streams:
    - type: websocket
      url: "wss://my-log-server.com/stream"

rules:
  - name: error_detection
    pattern: "ERROR|FATAL"
    alert: stdout
    cooldown: 60

alerts:
  stdout:
    type: stdout
```

### Run TinyWatcher

```bash
tinywatcher watch --config stream-config.yaml
```

### Test Configuration

```bash
tinywatcher test --config stream-config.yaml
```

## Real-World Examples

### Azure App Service Logs

```yaml
streams:
  - name: azure_webapp
    type: websocket
    url: "wss://myapp.scm.azurewebsites.net/api/logstream/application"
    headers:
      Authorization: "Bearer YOUR_AZURE_TOKEN"
```

### Kubernetes Pod Logs

```yaml
streams:
  - name: k8s_pod
    type: websocket
    url: "wss://k8s-api:443/api/v1/namespaces/default/pods/my-pod/log?follow=true"
    headers:
      Authorization: "Bearer YOUR_K8S_TOKEN"
```

### Custom HTTP Log Stream

```yaml
streams:
  - name: custom_logs
    type: http
    url: "https://my-service.com/logs/stream"
    headers:
      Accept: "text/event-stream"
      Api-Key: "your-api-key"
```

### Syslog over TCP

```yaml
streams:
  - name: syslog
    type: tcp
    url: "syslog.mycompany.com:514"
```

## Features

✅ **Automatic Reconnection**: Streams automatically reconnect on failure  
✅ **Same Rules Engine**: Uses the same regex patterns and alerts as files/containers  
✅ **Cooldown Support**: Prevent alert spam with per-rule cooldowns  
✅ **Multiple Streams**: Monitor multiple streams simultaneously  
✅ **Custom Headers**: Full support for authentication headers  
✅ **Real-time Processing**: Process log lines as they arrive  

## Combining Sources

You can mix files, containers, and streams in one config:

```yaml
inputs:
  files:
    - /var/log/nginx/error.log
  containers:
    - my-docker-app
  streams:
    - type: websocket
      url: "wss://cloud-logs.example.com/stream"
    - type: tcp
      url: "localhost:514"

rules:
  - name: errors_everywhere
    pattern: "ERROR|Exception"
    alert: slack
    cooldown: 60
```

All sources use the same rules and alerts!

## Benefits

- 🌐 **Cloud-Native**: Perfect for Azure, AWS, GCP
- 🔄 **Real-Time**: No polling, instant log processing
- 🚀 **Ephemeral-Friendly**: Works with serverless/container environments
- 🔌 **Pluggable**: Easy to extend with new protocols
- 📦 **Zero Dependencies**: No log agents or forwarders needed

## Troubleshooting

### Connection Issues

If streams fail to connect, check:
- URL is correct and accessible
- Authentication headers are valid
- Firewall/network allows the connection

Enable debug logging:
```bash
tinywatcher watch --config config.yaml --verbose
```

### High Memory Usage

For high-throughput streams, consider:
- Increasing cooldown values to reduce alert frequency
- Using more specific regex patterns
- Filtering logs at the source if possible

## Architecture

```
┌─────────────────┐
│  Log Sources    │
│  - Files        │
│  - Containers   │
│  - Streams ⭐   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  Rule Engine    │
│  (Regex Match)  │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Alert Manager   │
│ (with Cooldown) │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  Alert Outputs  │
│  - Slack        │
│  - Email        │
│  - Webhook      │
│  - Stdout       │
└─────────────────┘
```

## Next Steps

See the full example configuration in `stream-config.yaml`.

For more information, visit the [main README](README.md).
