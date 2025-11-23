use super::{ServiceManager, ServiceStatus};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct LaunchdManager {
    service_name: String,
}

impl LaunchdManager {
    pub fn new() -> Self {
        Self {
            service_name: "com.tinywatcher.agent".to_string(),
        }
    }

    fn get_plist_path(&self, is_daemon: bool) -> PathBuf {
        if is_daemon {
            // LaunchDaemon - system service running as root
            PathBuf::from("/Library/LaunchDaemons").join(format!("{}.plist", self.service_name))
        } else {
            // LaunchAgent - user service
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join("Library/LaunchAgents").join(format!("{}.plist", self.service_name))
        }
    }

    fn create_plist_content(&self, config_path: Option<PathBuf>, is_daemon: bool) -> Result<String> {
        let exe_path = super::get_executable_path()?;
        let exe_path_str = exe_path.to_str().context("Invalid executable path")?;
        
        let mut args = vec![
            format!("        <string>{}</string>", exe_path_str),
            "        <string>watch</string>".to_string(),
        ];
        
        if let Some(config) = config_path {
            args.push("        <string>--config</string>".to_string());
            args.push(format!("        <string>{}</string>", config.to_str().unwrap_or("")));
        }
        
        // For LaunchDaemons, use /var/log instead of /tmp for logs
        let (log_path, err_path) = if is_daemon {
            ("/var/log/tinywatcher.log", "/var/log/tinywatcher.err")
        } else {
            ("/tmp/tinywatcher.log", "/tmp/tinywatcher.err")
        };
        
        let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{}</string>
    
    <key>ProgramArguments</key>
    <array>
{}
    </array>
    
    <key>RunAtLoad</key>
    <true/>
    
    <key>KeepAlive</key>
    <true/>
    
    <key>StandardOutPath</key>
    <string>{}</string>
    
    <key>StandardErrorPath</key>
    <string>{}</string>
    
    <key>WorkingDirectory</key>
    <string>{}</string>
</dict>
</plist>"#,
            self.service_name,
            args.join("\n"),
            log_path,
            err_path,
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
                .to_str()
                .unwrap_or("/tmp")
        );
        
        Ok(plist)
    }
}

impl ServiceManager for LaunchdManager {
    fn install(&self, config_path: Option<PathBuf>, needs_elevation: bool) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        let is_daemon = needs_elevation;
        let service_type = if is_daemon { "LaunchDaemon (root)" } else { "LaunchAgent" };
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Installing")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher as a {}...", service_type)?;
        
        if is_daemon && !super::is_elevated() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            write!(&mut stdout, "  ⚠")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Detected root-owned log files. Installing as LaunchDaemon (requires sudo)...")?;
        }
        
        let plist_path = self.get_plist_path(is_daemon);
        
        // Create plist content
        let plist_content = self.create_plist_content(config_path.clone(), is_daemon)?;
        
        if is_daemon {
            // Write to temp file first, then use sudo to move it
            let temp_path = std::env::temp_dir().join(format!("{}.plist", self.service_name));
            fs::write(&temp_path, &plist_content)
                .context("Failed to write temporary plist file")?;
            
            // Use sudo to move the file to system location
            let output = Command::new("sudo")
                .args(&["mv", temp_path.to_str().unwrap(), plist_path.to_str().unwrap()])
                .output()
                .context("Failed to install plist file. Sudo required.")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to install plist file: {}", error);
            }
            
            // Set proper ownership and permissions
            let _ = Command::new("sudo")
                .args(&["chown", "root:wheel", plist_path.to_str().unwrap()])
                .output();
            
            let _ = Command::new("sudo")
                .args(&["chmod", "644", plist_path.to_str().unwrap()])
                .output();
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Created plist at: {}", plist_path.display())?;
            
            // Load the service with sudo
            let output = Command::new("sudo")
                .args(&["launchctl", "load", plist_path.to_str().unwrap()])
                .output()
                .context("Failed to load service with launchctl")?;
            
            if output.status.success() {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(&mut stdout, "  ✓")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Service loaded successfully")?;
                
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(&mut stdout, "  ✓")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Service will start automatically on boot")?;
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to load service: {}", error);
            }
        } else {
            // Create directory if it doesn't exist (LaunchAgent)
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create LaunchAgents directory")?;
            }
            
            // Write plist file directly
            fs::write(&plist_path, plist_content)
                .context("Failed to write plist file")?;
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Created plist at: {}", plist_path.display())?;
            
            // Load the service
            let output = Command::new("launchctl")
                .args(&["load", plist_path.to_str().unwrap()])
                .output()
                .context("Failed to load service with launchctl")?;
            
            if output.status.success() {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(&mut stdout, "  ✓")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Service loaded successfully")?;
                
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(&mut stdout, "  ✓")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Service will start automatically on login")?;
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to load service: {}", error);
            }
        }
        
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
        if is_daemon {
            writeln!(&mut stdout, "  Logs: /var/log/tinywatcher.log")?;
            writeln!(&mut stdout, "  Errors: /var/log/tinywatcher.err")?;
        } else {
            writeln!(&mut stdout, "  Logs: /tmp/tinywatcher.log")?;
            writeln!(&mut stdout, "  Errors: /tmp/tinywatcher.err")?;
        }
        stdout.reset()?;
        
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
        write!(&mut stdout, "Uninstalling")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher agent...")?;
        
        // Check both LaunchAgent and LaunchDaemon locations
        let agent_path = self.get_plist_path(false);
        let daemon_path = self.get_plist_path(true);
        
        let (plist_path, is_daemon) = if daemon_path.exists() {
            (daemon_path, true)
        } else if agent_path.exists() {
            (agent_path, false)
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(&mut stdout, "  ℹ")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service not installed")?;
            return Ok(());
        };
        
        if is_daemon {
            // Unload daemon with sudo
            let _ = Command::new("sudo")
                .args(&["launchctl", "unload", plist_path.to_str().unwrap()])
                .output();
            
            // Remove plist file with sudo
            let output = Command::new("sudo")
                .args(&["rm", plist_path.to_str().unwrap()])
                .output();
            
            if let Ok(output) = output {
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                    write!(&mut stdout, "  ⚠")?;
                    stdout.reset()?;
                    writeln!(&mut stdout, " Warning removing plist: {}", error)?;
                }
            }
        } else {
            // Unload agent
            let _ = Command::new("launchctl")
                .args(&["unload", plist_path.to_str().unwrap()])
                .output();
            
            // Remove plist file
            let _ = fs::remove_file(&plist_path);
        }
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "  ✓")?;
        stdout.reset()?;
        writeln!(&mut stdout, " Service uninstalled")?;
        
        writeln!(&mut stdout)?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
        writeln!(&mut stdout, "SUCCESS")?;
        stdout.reset()?;
        writeln!(&mut stdout, "TinyWatcher agent removed!")?;
        
        Ok(())
    }

    fn start(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Starting")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher agent...")?;
        
        // Check both LaunchAgent and LaunchDaemon locations
        let agent_path = self.get_plist_path(false);
        let daemon_path = self.get_plist_path(true);
        
        let (plist_path, is_daemon) = if daemon_path.exists() {
            (daemon_path, true)
        } else if agent_path.exists() {
            (agent_path, false)
        } else {
            anyhow::bail!("Service not installed. Run 'tinywatcher start --config <path>' first.");
        };
        
        if is_daemon {
            // Try to unload first (in case it's already loaded)
            let _ = Command::new("sudo")
                .args(&["launchctl", "unload", plist_path.to_str().unwrap()])
                .output();
            
            // Load the service with sudo
            let output = Command::new("sudo")
                .args(&["launchctl", "load", plist_path.to_str().unwrap()])
                .output()
                .context("Failed to start service")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to start service: {}", error);
            }
        } else {
            // Try to unload first (in case it's already loaded)
            let _ = Command::new("launchctl")
                .args(&["unload", plist_path.to_str().unwrap()])
                .output();
            
            // Load the service
            let output = Command::new("launchctl")
                .args(&["load", plist_path.to_str().unwrap()])
                .output()
                .context("Failed to start service")?;
            
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to start service: {}", error);
            }
        }
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "  ✓")?;
        stdout.reset()?;
        writeln!(&mut stdout, " Service started")?;
        
        writeln!(&mut stdout)?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
        writeln!(&mut stdout, "SUCCESS")?;
        stdout.reset()?;
        writeln!(&mut stdout, "TinyWatcher is running in the background!")?;
        
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(&mut stdout, "Stopping")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher agent...")?;
        
        // Check both LaunchAgent and LaunchDaemon locations
        let agent_path = self.get_plist_path(false);
        let daemon_path = self.get_plist_path(true);
        
        let (plist_path, is_daemon) = if daemon_path.exists() {
            (daemon_path, true)
        } else if agent_path.exists() {
            (agent_path, false)
        } else {
            anyhow::bail!("Service not installed");
        };
        
        let output = if is_daemon {
            Command::new("sudo")
                .args(&["launchctl", "unload", plist_path.to_str().unwrap()])
                .output()
                .context("Failed to stop service")?
        } else {
            Command::new("launchctl")
                .args(&["unload", plist_path.to_str().unwrap()])
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
            // launchctl sometimes reports errors even on success
            if error.contains("Could not find specified service") {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                write!(&mut stdout, "  ℹ")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Service was not running")?;
                Ok(())
            } else {
                anyhow::bail!("Failed to stop service: {}", error);
            }
        }
    }

    fn status(&self) -> Result<ServiceStatus> {
        // Check both LaunchAgent and LaunchDaemon locations
        let agent_path = self.get_plist_path(false);
        let daemon_path = self.get_plist_path(true);
        
        if !agent_path.exists() && !daemon_path.exists() {
            return Ok(ServiceStatus::NotInstalled);
        }
        
        // Check if service is loaded
        let output = Command::new("launchctl")
            .arg("list")
            .output()
            .context("Failed to query launchctl")?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        if output_str.contains(&self.service_name) {
            Ok(ServiceStatus::Running)
        } else {
            Ok(ServiceStatus::Stopped)
        }
    }

    fn service_name(&self) -> &str {
        &self.service_name
    }
}
