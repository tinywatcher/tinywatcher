# Service Management Guide

## Overview

TinyWatcher can run as either a **user-level service** (LaunchAgent) or a **system-level service** (LaunchDaemon) on macOS.

## Service Types

### LaunchAgent (User-level)
- **Location:** `~/Library/LaunchAgents/com.tinywatcher.agent.plist`
- **Privileges:** Runs with current user privileges
- **Command:** `tinywatcher start` (without sudo)
- **Use case:** Monitoring user-accessible log files

### LaunchDaemon (System-level)
- **Location:** `/Library/LaunchDaemons/com.tinywatcher.agent.plist`
- **Privileges:** Runs with root privileges
- **Command:** `sudo tinywatcher start` (with sudo)
- **Use case:** Monitoring system log files that require root access (e.g., `/var/log/system.log`)

## Fixed Issues

### Problem: Conflict Between User and System Services

Previously, if you:
1. Ran `tinywatcher start` (creating a LaunchAgent)
2. Then ran `sudo tinywatcher start` (attempting to start as LaunchDaemon)

The tool would incorrectly try to use `sudo launchctl` to manage the user-level service, which doesn't work properly.

### Solution

The service manager now:
1. **Detects which service type is installed** (LaunchAgent, LaunchDaemon, or both)
2. **Checks if running with sudo** (`is_elevated()`)
3. **Matches the service type to the privilege level**:
   - Without sudo → manages LaunchAgent only
   - With sudo → manages LaunchDaemon only
4. **Prevents mismatched operations** with clear error messages

## Usage Examples

### Installing User-level Service (No Root Access Needed)

```bash
# Install and start as user service
tinywatcher start --config config.yaml

# Check status
tinywatcher status

# Stop the service
tinywatcher stop

# Uninstall
tinywatcher uninstall
```

### Installing System-level Service (Requires Root Access)

```bash
# Install and start as system service (requires sudo)
sudo tinywatcher start --config config.yaml

# Check status (with sudo to see system service)
sudo tinywatcher status

# Stop the service
sudo tinywatcher stop

# Uninstall
sudo tinywatcher uninstall
```

### If Both Services Are Installed

If you accidentally install both services:

```bash
# This will manage the user-level service
tinywatcher stop
tinywatcher uninstall

# This will manage the system-level service
sudo tinywatcher stop
sudo tinywatcher uninstall
```

The tool will warn you when both services are detected and show you which one is being managed.

## Error Messages

### "User service (LaunchAgent) is installed. Do not use sudo."
- **Cause:** Trying to use `sudo` with a user-level service
- **Fix:** Run the command without `sudo`:
  ```bash
  tinywatcher start
  ```

### "System service (LaunchDaemon) is installed but requires sudo."
- **Cause:** Trying to manage a system-level service without sudo
- **Fix:** Run the command with `sudo`:
  ```bash
  sudo tinywatcher start
  ```

## Best Practices

1. **Choose one service type** - Don't install both unless you have a specific reason
2. **User service is recommended** for most use cases (monitoring application logs)
3. **System service is required** only when monitoring system logs that need root access
4. **Be consistent** - Always use the same privilege level (with or without sudo) for all operations

## Checking What's Installed

```bash
# Check for user-level service
ls -la ~/Library/LaunchAgents/com.tinywatcher.agent.plist

# Check for system-level service (requires sudo)
sudo ls -la /Library/LaunchDaemons/com.tinywatcher.agent.plist
```

## Migration

### From User to System Service

```bash
# Stop and uninstall user service
tinywatcher stop
tinywatcher uninstall

# Install system service
sudo tinywatcher start --config config.yaml
```

### From System to User Service

```bash
# Stop and uninstall system service
sudo tinywatcher stop
sudo tinywatcher uninstall

# Install user service
tinywatcher start --config config.yaml
```
