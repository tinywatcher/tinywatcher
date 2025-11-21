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

    fn get_service_path(&self) -> PathBuf {
        // Use user systemd services directory
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".config/systemd/user").join(format!("{}.service", self.service_name))
    }

    fn create_service_content(&self, config_path: Option<PathBuf>) -> Result<String> {
        let exe_path = super::get_executable_path()?;
        let exe_path_str = exe_path.to_str().context("Invalid executable path")?;
        
        let mut exec_start = format!("{} watch", exe_path_str);
        
        if let Some(config) = config_path {
            exec_start.push_str(&format!(" --config {}", config.to_str().unwrap_or("")));
        }
        
        let service_content = format!(r#"[Unit]
Description=TinyWatcher - Zero-infrastructure observability tool
After=network.target

[Service]
Type=simple
ExecStart={}
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=default.target
"#, exec_start);
        
        Ok(service_content)
    }
}

impl ServiceManager for SystemdManager {
    fn install(&self, config_path: Option<PathBuf>) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Installing")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher as a systemd user service...")?;
        
        let service_path = self.get_service_path();
        
        // Create directory if it doesn't exist
        if let Some(parent) = service_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create systemd user directory")?;
        }
        
        // Create service file
        let service_content = self.create_service_content(config_path.clone())?;
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
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "  ✓")?;
        stdout.reset()?;
        writeln!(&mut stdout, " Reloaded systemd daemon")?;
        
        // Enable the service (start on boot)
        let output = Command::new("systemctl")
            .arg("--user")
            .arg("enable")
            .arg(&self.service_name)
            .output()
            .context("Failed to enable service")?;
        
        if output.status.success() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service enabled (will start on login)")?;
        }
        
        // Start the service
        let output = Command::new("systemctl")
            .arg("--user")
            .arg("start")
            .arg(&self.service_name)
            .output()
            .context("Failed to start service")?;
        
        if output.status.success() {
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
            writeln!(&mut stdout, "  View logs: journalctl --user -u {} -f", self.service_name)?;
            stdout.reset()?;
            
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to start service: {}", error);
        }
    }

    fn uninstall(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
        write!(&mut stdout, "Uninstalling")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        let service_path = self.get_service_path();
        
        if service_path.exists() {
            // Stop the service
            let _ = Command::new("systemctl")
                .arg("--user")
                .arg("stop")
                .arg(&self.service_name)
                .output();
            
            // Disable the service
            let _ = Command::new("systemctl")
                .arg("--user")
                .arg("disable")
                .arg(&self.service_name)
                .output();
            
            // Remove service file
            fs::remove_file(&service_path)
                .context("Failed to remove service file")?;
            
            // Reload daemon
            let _ = Command::new("systemctl")
                .arg("--user")
                .arg("daemon-reload")
                .output();
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service uninstalled")?;
            
            writeln!(&mut stdout)?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
            writeln!(&mut stdout, "SUCCESS")?;
            stdout.reset()?;
            writeln!(&mut stdout, "TinyWatcher service removed!")?;
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(&mut stdout, "  ℹ")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service not installed")?;
        }
        
        Ok(())
    }

    fn start(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Starting")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        let service_path = self.get_service_path();
        
        if !service_path.exists() {
            anyhow::bail!("Service not installed. Run 'tinywatcher start --config <path>' first.");
        }
        
        let output = Command::new("systemctl")
            .arg("--user")
            .arg("start")
            .arg(&self.service_name)
            .output()
            .context("Failed to start service")?;
        
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
            writeln!(&mut stdout, "  View logs: journalctl --user -u {} -f", self.service_name)?;
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
        
        let service_path = self.get_service_path();
        
        if !service_path.exists() {
            anyhow::bail!("Service not installed");
        }
        
        let output = Command::new("systemctl")
            .arg("--user")
            .arg("stop")
            .arg(&self.service_name)
            .output()
            .context("Failed to stop service")?;
        
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
        let service_path = self.get_service_path();
        
        if !service_path.exists() {
            return Ok(ServiceStatus::NotInstalled);
        }
        
        let output = Command::new("systemctl")
            .arg("--user")
            .arg("is-active")
            .arg(&self.service_name)
            .output()
            .context("Failed to check service status")?;
        
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
