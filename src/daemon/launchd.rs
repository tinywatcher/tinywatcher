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

    fn get_plist_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join("Library/LaunchAgents").join(format!("{}.plist", self.service_name))
    }

    fn create_plist_content(&self, config_path: Option<PathBuf>) -> Result<String> {
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
    <string>/tmp/tinywatcher.log</string>
    
    <key>StandardErrorPath</key>
    <string>/tmp/tinywatcher.err</string>
    
    <key>WorkingDirectory</key>
    <string>{}</string>
</dict>
</plist>"#,
            self.service_name,
            args.join("\n"),
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
                .to_str()
                .unwrap_or("/tmp")
        );
        
        Ok(plist)
    }
}

impl ServiceManager for LaunchdManager {
    fn install(&self, config_path: Option<PathBuf>) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        write!(&mut stdout, "Installing")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher as a LaunchAgent...")?;
        
        let plist_path = self.get_plist_path();
        
        // Create directory if it doesn't exist
        if let Some(parent) = plist_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create LaunchAgents directory")?;
        }
        
        // Create plist file
        let plist_content = self.create_plist_content(config_path.clone())?;
        fs::write(&plist_path, plist_content)
            .context("Failed to write plist file")?;
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(&mut stdout, "  ✓")?;
        stdout.reset()?;
        writeln!(&mut stdout, " Created plist at: {}", plist_path.display())?;
        
        // Load the service
        let output = Command::new("launchctl")
            .arg("load")
            .arg(&plist_path)
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
            writeln!(&mut stdout, "  Logs: /tmp/tinywatcher.log")?;
            writeln!(&mut stdout, "  Errors: /tmp/tinywatcher.err")?;
            stdout.reset()?;
            
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to load service: {}", error);
        }
    }

    fn uninstall(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
        write!(&mut stdout, "Uninstalling")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher agent...")?;
        
        let plist_path = self.get_plist_path();
        
        if plist_path.exists() {
            // Unload the service
            let output = Command::new("launchctl")
                .arg("unload")
                .arg(&plist_path)
                .output();
            
            if let Ok(output) = output {
                if !output.status.success() {
                    let error = String::from_utf8_lossy(&output.stderr);
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                    write!(&mut stdout, "  ⚠")?;
                    stdout.reset()?;
                    writeln!(&mut stdout, " Warning unloading service: {}", error)?;
                }
            }
            
            // Remove plist file
            fs::remove_file(&plist_path)
                .context("Failed to remove plist file")?;
            
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            write!(&mut stdout, "  ✓")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Service uninstalled")?;
            
            writeln!(&mut stdout)?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
            writeln!(&mut stdout, "SUCCESS")?;
            stdout.reset()?;
            writeln!(&mut stdout, "TinyWatcher agent removed!")?;
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
        writeln!(&mut stdout, " tinywatcher agent...")?;
        
        let plist_path = self.get_plist_path();
        
        if !plist_path.exists() {
            anyhow::bail!("Service not installed. Run 'tinywatcher start --config <path>' first.");
        }
        
        // Try to unload first (in case it's already loaded)
        let _ = Command::new("launchctl")
            .arg("unload")
            .arg(&plist_path)
            .output();
        
        // Load the service
        let output = Command::new("launchctl")
            .arg("load")
            .arg(&plist_path)
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
            anyhow::bail!("Failed to start service: {}", error);
        }
    }

    fn stop(&self) -> Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        write!(&mut stdout, "Stopping")?;
        stdout.reset()?;
        writeln!(&mut stdout, " tinywatcher agent...")?;
        
        let plist_path = self.get_plist_path();
        
        if !plist_path.exists() {
            anyhow::bail!("Service not installed");
        }
        
        let output = Command::new("launchctl")
            .arg("unload")
            .arg(&plist_path)
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
        let plist_path = self.get_plist_path();
        
        if !plist_path.exists() {
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
