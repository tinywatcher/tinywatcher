# Changelog

All notable changes to TinyWatcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-12-04

### Added
- **File Glob Pattern Support** - Monitor multiple log files with wildcards (`*.log`, `/var/log/services/*/error.log`)
- **SendGrid Integration** - Send alerts via SendGrid API with support for multiple recipients
- **Threshold-Based Rules** - Configure resource monitoring with CPU, memory, and disk thresholds

### Changed
- Improved documentation with glob pattern examples
- Enhanced alert configuration with SendGrid support

## [0.1.0] - 2024-11-28

### Added
- Initial release of TinyWatcher
- Single binary log and system monitoring tool
- Multiple input sources: local files, Docker containers, log streams (WebSocket, HTTP, TCP)
- Alert integrations: Discord, Telegram, Slack, PagerDuty, Ntfy.sh, Webhooks, Email, Stdout
- Regex pattern matching with cooldown periods
- Resource monitoring (CPU, memory, disk usage)
- Health check monitoring for HTTP endpoints
- Identity management for multi-server deployments
- Daemon mode with systemd/launchd/Windows Service support
- Configuration validation and testing modes
- Environment variable support for secrets
- Cross-platform support (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64/ARM64)

[0.2.0]: https://github.com/tinywatcher/tinywatcher/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/tinywatcher/tinywatcher/releases/tag/v0.1.0
