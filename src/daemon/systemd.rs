use super::{ServiceManager, ServiceStatus};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct SystemdManager {
    service_name: String,
}

impl SystemdManager {
    pub fn new() -> Self {
        Self {
            service_name: "tinywatcher".to_string(),
        }
    }

    fn get_service_path(&self, system_service: bool) -> PathBuf {
        if system_service {
            // System service path (requires root)
            PathBuf::from("/etc/systemd/system").join(format!("{}.service", self.service_name))
        } else {
            // User service path
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".config/systemd/user").join(format!("{}.service", self.service_name))
        }
    }

    fn create_service_content(&self, config_path: Option<PathBuf>, system_service: bool) -> Result<String> {
        let exe_path = super::get_executable_path()?;
        let exe_path_str = exe_path.to_str().context("Invalid executable path")?;
        
        let mut exec_start = format!("{} watch", exe_path_str);
        
        if let Some(config) = config_path {
            exec_start.push_str(&format!(" --config {}", config.to_str().unwrap_or("")));
        }
        
        let wanted_by = if system_service {
            "multi-user.target"
        } else {
            "default.target"
        };
        
        // For system services, we might want to add User directive if needed
        let user_directive = if system_service {
            // Run as root for system services to access root-owned logs
            ""
        } else {
            ""
        };
        
        let service_content = format!(r#"[Unit]
Description=TinyWatcher - Zero-infrastructure observability tool
After=network.target

[Service]
Type=simple
ExecStart={}{}
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy={}
"#, exec_start, user_directive, wanted_by);
        
        Ok(service_content)
    }
}

impl ServiceManager for SystemdManager {
    fn install(&self, config_path: Option<PathBuf>, needs_elevation: bool) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        let system_service = needs_elevation;
        let service_type = if system_service { "system" } else { "user" };
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Installing")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher as a systemd {} service...", service_type)?;
        
        // Check if opposite service type is already installed
        let opposite_path = self.get_service_path(!system_service);
        if opposite_path.exists() {
            let opposite_type = if system_service { "user" } else { "system" };
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            write!(&mut stdout, "  ⚠")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Note: {} service is already installed at:", opposite_type)?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            writeln!(&mut stdout, "    {}", opposite_path.display())?;
            stdout.reset()?;
            writeln!(&mut stdout, "  Both services will coexist. You can remove the {} service later if not needed.", opposite_type)?;
            writeln!(&mut stdout)?;
        }
        
        if system_service && !super::is_elevated() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            write!(&mut stdout, "  ⚠")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Detected root-owned log files. Installing as system service (requires sudo)...")?;
        }
        
        let service_path = self.get_service_path(system_service);
        
        // Create service file content
        let service_content = self.create_service_content(config_path.clone(), system_service)?;
        
        if system_service {
            // Write to temp file first, then use sudo to move it
            let temp_path = std::env::temp_dir().join(format!("{}.service", self.service_name));
            fs::write(&temp_path, &service_content)
                .context("Failed to write temporary service file")?;
            
            // Use sudo to move the file to system location
            let output = Command::new("sudo")
                .args(&["mv", temp_path.to_str().unwrap(), service_path.to_str().unwrap()])
                .output()
                .context("Failed to install service file. Sudo required.")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to install service file: {}", error);
            }
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Created system service file at: {}", service_path.display())?;
            
            // Reload systemd daemon with sudo
            let output = Command::new("sudo")
                .args(&["systemctl", "daemon-reload"])
                .output()
                .context("Failed to reload systemd daemon")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to reload systemd daemon: {}", error);
            }
        } else {
            // Create directory if it doesn't exist (user service)
            if let Some(parent) = service_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create systemd user directory")?;
            }
            
            // Write service file directly
            fs::write(&service_path, service_content)
                .context("Failed to write service file")?;
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Created service file at: {}", service_path.display())?;
            
            // Reload systemd daemon
            let output = Command::new("systemctl")
                .arg("--user")
                .arg("daemon-reload")
                .output()
                .context("Failed to reload systemd daemon")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to reload systemd daemon: {}", error);
            }
        }
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "  ✓")?;
        stdout.reset()?;
        writeln!(&mut stdout, " Reloaded systemd daemon")?;
        
        // Enable the service (start on boot)
        let enable_output = if system_service {
            Command::new("sudo")
                .args(&["systemctl", "enable", &self.service_name])
                .output()
                .context("Failed to enable service")?
        } else {
            Command::new("systemctl")
                .args(&["--user", "enable", &self.service_name])
                .output()
                .context("Failed to enable service")?
        };
        
        if enable_output.status.success() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            if system_service {
                writeln!(&mut stdout, " Service enabled (will start on boot)")?;
            } else {
                writeln!(&mut stdout, " Service enabled (will start on login)")?;
            }
        }
        
        // Start the service
        let start_output = if system_service {
            Command::new("sudo")
                .args(&["systemctl", "start", &self.service_name])
                .output()
                .context("Failed to start service")?
        } else {
            Command::new("systemctl")
                .args(&["--user", "start", &self.service_name])
                .output()
                .context("Failed to start service")?
        };
        
        if start_output.status.success() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service started")?;
            
            if let Some(cfg) = config_path {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                write!(&mut stdout, "  ℹ")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Using config: {}", cfg.display())?;
            }
            
            writeln!(&mut stdout)?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
            writeln!(&mut stdout, "SUCCESS")?;
            stdout.reset()?;
            writeln!(&mut stdout, "TinyWatcher agent installed and started!")?;
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            if system_service {
                writeln!(&mut stdout, "  View logs: journalctl -u {} -f", self.service_name)?;
            } else {
                writeln!(&mut stdout, "  View logs: journalctl --user -u {} -f", self.service_name)?;
            }
            stdout.reset()?;
            
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&start_output.stderr);
            anyhow::bail!("Failed to start service: {}", error);
        }
    }

    fn uninstall(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
        write!(&mut stdout, "Uninstalling")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        // Check both user and system service locations
        let user_service_path = self.get_service_path(false);
        let system_service_path = self.get_service_path(true);
        let running_as_root = super::is_elevated();
        
        // Handle the case where both services exist
        if system_service_path.exists() && user_service_path.exists() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            write!(&mut stdout, "  ⚠")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Detected both user and system services installed")?;
            
            if running_as_root {
                writeln!(&mut stdout, "  Uninstalling system service...")?;
                writeln!(&mut stdout)?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                write!(&mut stdout, "  ℹ")?;
                stdout.reset()?;
                writeln!(&mut stdout, " To also remove user service, run: tinywatcher uninstall (without sudo)")?;
            } else {
                writeln!(&mut stdout, "  Uninstalling user service...")?;
                writeln!(&mut stdout)?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                write!(&mut stdout, "  ℹ")?;
                stdout.reset()?;
                writeln!(&mut stdout, " To also remove system service, run: sudo tinywatcher uninstall")?;
            }
        }
        
        let (service_path, is_system) = if system_service_path.exists() && running_as_root {
            (system_service_path, true)
        } else if user_service_path.exists() && !running_as_root {
            (user_service_path, false)
        } else if system_service_path.exists() && !running_as_root {
            anyhow::bail!(
                "System service is installed but requires sudo.\n\
                Run: sudo tinywatcher uninstall"
            );
        } else if user_service_path.exists() && running_as_root {
            anyhow::bail!(
                "User service is installed.\n\
                Do not use sudo. Run: tinywatcher uninstall (without sudo)"
            );
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(&mut stdout, "  ℹ")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service not installed")?;
            return Ok(());
        };
        
        if is_system {
            // System service - needs sudo
            let _ = Command::new("sudo")
                .args(&["systemctl", "stop", &self.service_name])
                .output();
            
            let _ = Command::new("sudo")
                .args(&["systemctl", "disable", &self.service_name])
                .output();
            
            let _ = Command::new("sudo")
                .args(&["rm", service_path.to_str().unwrap()])
                .output();
            
            let _ = Command::new("sudo")
                .args(&["systemctl", "daemon-reload"])
                .output();
        } else {
            // User service
            let _ = Command::new("systemctl")
                .args(&["--user", "stop", &self.service_name])
                .output();
            
            let _ = Command::new("systemctl")
                .args(&["--user", "disable", &self.service_name])
                .output();
            
            let _ = fs::remove_file(&service_path);
            
            let _ = Command::new("systemctl")
                .args(&["--user", "daemon-reload"])
                .output();
        }
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "  ✓")?;
        stdout.reset()?;
        writeln!(&mut stdout, " Service uninstalled")?;
        
        writeln!(&mut stdout)?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
        writeln!(&mut stdout, "SUCCESS")?;
        stdout.reset()?;
        writeln!(&mut stdout, "TinyWatcher service removed!")?;
        
        Ok(())
    }

    fn start(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Starting")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        // Check both user and system service locations
        let user_service_path = self.get_service_path(false);
        let system_service_path = self.get_service_path(true);
        let running_as_root = super::is_elevated();
        
        // Determine which service to use based on what's installed and current privileges
        let is_system = if system_service_path.exists() && user_service_path.exists() {
            // Both exist - choose based on current user context
            if running_as_root {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(&mut stdout, "  ⚠")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Detected both user and system services installed")?;
                writeln!(&mut stdout, "  Starting system service since running with sudo...")?;
                true
            } else {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(&mut stdout, "  ⚠")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Detected both user and system services installed")?;
                writeln!(&mut stdout, "  Starting user service...")?;
                false
            }
        } else if system_service_path.exists() {
            if !running_as_root {
                anyhow::bail!(
                    "System service is installed but requires sudo.\n\
                    Run: sudo tinywatcher start"
                );
            }
            true
        } else if user_service_path.exists() {
            if running_as_root {
                anyhow::bail!(
                    "User service is installed.\n\
                    Do not use sudo. Run: tinywatcher start (without sudo)"
                );
            }
            false
        } else {
            anyhow::bail!("Service not installed. Run 'tinywatcher start --config <path>' first.");
        };
        
        let output = if is_system {
            Command::new("sudo")
                .args(&["systemctl", "start", &self.service_name])
                .output()
                .context("Failed to start service")?
        } else {
            Command::new("systemctl")
                .args(&["--user", "start", &self.service_name])
                .output()
                .context("Failed to start service")?
        };
        
        if output.status.success() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service started")?;
            
            writeln!(&mut stdout)?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
            writeln!(&mut stdout, "SUCCESS")?;
            stdout.reset()?;
            writeln!(&mut stdout, "TinyWatcher is running in the background!")?;
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            if is_system {
                writeln!(&mut stdout, "  View logs: journalctl -u {} -f", self.service_name)?;
            } else {
                writeln!(&mut stdout, "  View logs: journalctl --user -u {} -f", self.service_name)?;
            }
            stdout.reset()?;
            
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to start service: {}", error);
        }
    }

    fn stop(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(&mut stdout, "Stopping")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        // Check both user and system service locations
        let user_service_path = self.get_service_path(false);
        let system_service_path = self.get_service_path(true);
        let running_as_root = super::is_elevated();
        
        // Determine which service to stop based on what's installed and current privileges
        let is_system = if system_service_path.exists() && user_service_path.exists() {
            // Both exist - choose based on current user context
            if running_as_root {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(&mut stdout, "  ⚠")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Detected both user and system services installed")?;
                writeln!(&mut stdout, "  Stopping system service since running with sudo...")?;
                true
            } else {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(&mut stdout, "  ⚠")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Detected both user and system services installed")?;
                writeln!(&mut stdout, "  Stopping user service...")?;
                false
            }
        } else if system_service_path.exists() {
            if !running_as_root {
                anyhow::bail!(
                    "System service is installed but requires sudo.\n\
                    Run: sudo tinywatcher stop"
                );
            }
            true
        } else if user_service_path.exists() {
            if running_as_root {
                anyhow::bail!(
                    "User service is installed.\n\
                    Do not use sudo. Run: tinywatcher stop (without sudo)"
                );
            }
            false
        } else {
            anyhow::bail!("Service not installed");
        };
        
        let output = if is_system {
            Command::new("sudo")
                .args(&["systemctl", "stop", &self.service_name])
                .output()
                .context("Failed to stop service")?
        } else {
            Command::new("systemctl")
                .args(&["--user", "stop", &self.service_name])
                .output()
                .context("Failed to stop service")?
        };
        
        if output.status.success() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service stopped")?;
            
            writeln!(&mut stdout)?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
            writeln!(&mut stdout, "SUCCESS")?;
            stdout.reset()?;
            writeln!(&mut stdout, "TinyWatcher has been stopped")?;
            
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to stop service: {}", error);
        }
    }

    fn status(&self) -> Result<ServiceStatus> {
        // Check both user and system service locations
        let user_service_path = self.get_service_path(false);
        let system_service_path = self.get_service_path(true);
        let running_as_root = super::is_elevated();
        
        // Determine which service to check based on what's installed and current privileges
        let service_exists = if system_service_path.exists() && user_service_path.exists() {
            // Both exist - check the one that matches current user context
            true
        } else if system_service_path.exists() {
            true
        } else if user_service_path.exists() {
            true
        } else {
            return Ok(ServiceStatus::NotInstalled);
        };
        
        if !service_exists {
            return Ok(ServiceStatus::NotInstalled);
        }
        
        // Check if service is active in the appropriate context
        let output = if running_as_root && system_service_path.exists() {
            // Check system service
            Command::new("systemctl")
                .args(&["is-active", &self.service_name])
                .output()
                .context("Failed to check service status")?
        } else {
            // Check user service
            Command::new("systemctl")
                .args(&["--user", "is-active", &self.service_name])
                .output()
                .context("Failed to check service status")?
        };
        
        let status_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        match status_str.as_str() {
            "active" => Ok(ServiceStatus::Running),
            "inactive" | "failed" => Ok(ServiceStatus::Stopped),
            _ => Ok(ServiceStatus::Unknown),
        }
    }

    fn service_name(&self) -> &str {
        &self.service_name
    }
}
