# Privilege Detection and Elevated Service Installation

## Overview

TinyWatcher now automatically detects when monitoring root/admin-owned log files and installs services with appropriate elevated privileges to ensure proper file access permissions.

## Features

### 1. Automatic Privilege Detection

- **Unix (Linux/macOS)**: Checks file ownership and permissions to determine if files are owned by root and not readable by the current user
- **Windows**: Attempts to open files and detects permission-denied errors to identify protected files

### 2. Platform-Specific Service Installation

#### Linux (systemd)

- **User Service** (default): Installed in `~/.config/systemd/user/` for user-owned logs
- **System Service** (elevated): Installed in `/etc/systemd/system/` for root-owned logs
  - Requires `sudo` for installation
  - Runs as root to access protected logs
  - Starts on boot (multi-user.target)
  - View logs: `journalctl -u tinywatcher -f`

#### macOS (launchd)

- **LaunchAgent** (default): Installed in `~/Library/LaunchAgents/` for user-owned logs
- **LaunchDaemon** (elevated): Installed in `/Library/LaunchDaemons/` for root-owned logs
  - Requires `sudo` for installation
  - Runs as root to access protected logs
  - Starts on boot
  - Logs stored in `/var/log/tinywatcher.log` and `/var/log/tinywatcher.err`

#### Windows (Service Manager)

- **User Service** (default): Standard Windows service for accessible logs
- **SYSTEM Service** (elevated): Service running as LocalSystem for protected logs
  - Requires Administrator privileges
  - Full system access for log monitoring
  - Manage via `services.msc`

## Usage

### Installation with Root-Owned Logs

```bash
# The service will automatically detect root-owned logs
tinywatcher start --config /path/to/config.yaml
```

Example output when root-owned logs are detected:

```
Installing tinywatcher as a systemd system service...
  ⚠ Detected root-owned log files:
    - /var/log/syslog
    - /var/log/auth.log

  ℹ Service will be installed with elevated privileges (sudo required)

[sudo] password for user: 
  ✓ Created system service file at: /etc/systemd/system/tinywatcher.service
  ✓ Reloaded systemd daemon
  ✓ Service enabled (will start on boot)
  ✓ Service started
  ℹ Using config: /path/to/config.yaml

SUCCESS
TinyWatcher agent installed and started!
  View logs: journalctl -u tinywatcher -f
```

### Configuration Example

```yaml
inputs:
  files:
    - /var/log/syslog        # Root-owned - triggers elevated service
    - /var/log/auth.log       # Root-owned - triggers elevated service
    - /var/log/nginx/error.log # May be root-owned depending on setup

alerts:
  stdout:
    type: stdout

rules:
  - name: "Authentication Failures"
    pattern: "authentication failure"
    alert: ["stdout"]
```

## Implementation Details

### Privilege Detection Module (`src/daemon/privilege.rs`)

Key functions:
- `is_elevated()`: Check if current process has elevated privileges
- `file_needs_elevation(path)`: Check if a specific file requires elevated access
- `any_file_needs_elevation(paths)`: Check if any files in a list need elevation
- `get_files_needing_elevation(paths)`: Get list of files requiring elevation

### Service Manager Updates

All service managers now accept a `needs_elevation: bool` parameter in the `install()` method to determine the appropriate installation mode.

### Pre-Flight Checks

Before service installation, the CLI:
1. Loads the configuration file
2. Checks permissions on all configured log files
3. Displays a warning if root-owned files are detected
4. Installs the service with appropriate privileges

## Benefits

1. **Automatic Detection**: No manual configuration needed - the tool detects privilege requirements automatically
2. **User Feedback**: Clear warnings inform users when elevated privileges will be used
3. **Cross-Platform**: Works consistently across Linux, macOS, and Windows
4. **Secure**: Only requests elevated privileges when actually needed
5. **Transparent**: Shows which files require elevated access before installation

## Notes

- Elevated services run with root/SYSTEM privileges - ensure your configuration is trusted
- On Unix systems, `sudo` will prompt for password during installation
- On Windows, the command prompt/terminal must be run as Administrator
- You can check service status at any time with: `tinywatcher status`
- To reinstall with different privileges, first uninstall: `tinywatcher stop` (if running), then reinstall

## Future Enhancements

Potential improvements:
- Support for running as a specific user (instead of root) with file ACLs
- Docker container privilege detection
- More granular permission checking (e.g., group-based access)
- Automatic privilege escalation prompts in interactive mode
