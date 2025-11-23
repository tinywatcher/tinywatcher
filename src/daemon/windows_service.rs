use super::{ServiceManager, ServiceStatus};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct WindowsServiceManager {
    service_name: String,
}

impl WindowsServiceManager {
    pub fn new() -> Self {
        Self {
            service_name: "TinyWatcher".to_string(),
        }
    }
}

impl ServiceManager for WindowsServiceManager {
    fn install(&self, config_path: Option<PathBuf>, needs_elevation: bool) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Installing")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher as a Windows service...")?;
        
        if needs_elevation && !super::is_elevated() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            write!(&mut stdout, "  ⚠")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Detected protected log files. Service will run as SYSTEM...")?;
        }
        
        let exe_path = super::get_executable_path()?;
        let exe_path_str = exe_path.to_str().context("Invalid executable path")?;
        
        let mut bin_path = format!("\"{}\" watch", exe_path_str);
        
        if let Some(config) = config_path.clone() {
            bin_path.push_str(&format!(" --config \"{}\"", config.to_str().unwrap_or("")));
        }
        
        // Create the service using sc.exe
        // Windows services run as LocalSystem by default, which has full access
        // If needs_elevation is true, we explicitly set the service to run as LocalSystem
        let mut args = vec![
            "create",
            &self.service_name,
            "binPath=",
            &bin_path,
            "start=",
            "auto",
            "DisplayName=",
            "TinyWatcher Agent",
        ];
        
        // Explicitly set to run as LocalSystem if elevated privileges are needed
        let obj_param;
        if needs_elevation {
            obj_param = "obj=LocalSystem".to_string();
            args.push(&obj_param);
        }
        
        let output = Command::new("sc")
            .args(&args)
            .output()
            .context("Failed to create service. Note: Administrator privileges required.")?;
        
        if output.status.success() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service created")?;
            
            // Start the service
            let start_output = Command::new("sc")
                .args(&["start", &self.service_name])
                .output()
                .context("Failed to start service")?;
            
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
                writeln!(&mut stdout, "TinyWatcher service installed and started!")?;
                
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
                writeln!(&mut stdout, "  View in: services.msc")?;
                stdout.reset()?;
                
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&start_output.stderr);
                anyhow::bail!("Failed to start service: {}", error);
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create service: {}. Make sure to run as Administrator.", error);
        }
    }

    fn uninstall(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
        write!(&mut stdout, "Uninstalling")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        // Stop the service first
        let _ = Command::new("sc")
            .args(&["stop", &self.service_name])
            .output();
        
        // Delete the service
        let output = Command::new("sc")
            .args(&["delete", &self.service_name])
            .output()
            .context("Failed to delete service. Note: Administrator privileges required.")?;
        
        if output.status.success() {
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
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("does not exist") || error.contains("1060") {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                write!(&mut stdout, "  ℹ")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Service not installed")?;
                Ok(())
            } else {
                anyhow::bail!("Failed to delete service: {}", error);
            }
        }
    }

    fn start(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Starting")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        let output = Command::new("sc")
            .args(&["start", &self.service_name])
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
            
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("already been started") || error.contains("1056") {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                write!(&mut stdout, "  ℹ")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Service is already running")?;
                Ok(())
            } else {
                anyhow::bail!("Failed to start service: {}", error);
            }
        }
    }

    fn stop(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(&mut stdout, "Stopping")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher service...")?;
        
        let output = Command::new("sc")
            .args(&["stop", &self.service_name])
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
            if error.contains("not started") || error.contains("1062") {
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
        let output = Command::new("sc")
            .args(&["query", &self.service_name])
            .output()
            .context("Failed to query service status")?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("does not exist") || error.contains("1060") {
                return Ok(ServiceStatus::NotInstalled);
            }
            return Ok(ServiceStatus::Unknown);
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        if output_str.contains("RUNNING") {
            Ok(ServiceStatus::Running)
        } else if output_str.contains("STOPPED") {
            Ok(ServiceStatus::Stopped)
        } else {
            Ok(ServiceStatus::Unknown)
        }
    }

    fn service_name(&self) -> &str {
        &self.service_name
    }
}
