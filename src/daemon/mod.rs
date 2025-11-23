use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "linux")]
mod systemd;
#[cfg(target_os = "macos")]
mod launchd;
#[cfg(target_os = "windows")]
mod windows_service;

mod privilege;

pub use privilege::{is_elevated, any_file_needs_elevation, get_files_needing_elevation};

/// Determine the service manager for the current platform
pub fn get_service_manager() -> Box<dyn ServiceManager> {
    #[cfg(target_os = "linux")]
    return Box::new(systemd::SystemdManager::new());
    
    #[cfg(target_os = "macos")]
    return Box::new(launchd::LaunchdManager::new());
    
    #[cfg(target_os = "windows")]
    return Box::new(windows_service::WindowsServiceManager::new());
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    compile_error!("Unsupported platform for daemon mode");
}

/// Service manager trait for cross-platform daemon management
pub trait ServiceManager: Send + Sync {
    /// Install the service
    /// If needs_elevation is true, the service will be installed with elevated privileges
    fn install(&self, config_path: Option<PathBuf>, needs_elevation: bool) -> Result<()>;
    
    /// Uninstall the service
    fn uninstall(&self) -> Result<()>;
    
    /// Start the service
    fn start(&self) -> Result<()>;
    
    /// Stop the service
    fn stop(&self) -> Result<()>;
    
    /// Get the status of the service
    fn status(&self) -> Result<ServiceStatus>;
    
    /// Get the service name
    fn service_name(&self) -> &str {
        "tinywatcher"
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    NotInstalled,
    Unknown,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Running => write!(f, "running"),
            ServiceStatus::Stopped => write!(f, "stopped"),
            ServiceStatus::NotInstalled => write!(f, "not installed"),
            ServiceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Get the path to the current executable
pub fn get_executable_path() -> Result<PathBuf> {
    std::env::current_exe().context("Failed to get current executable path")
}

/// Helper to run a command and check if it succeeded
pub fn run_command(command: &str, args: &[&str]) -> Result<bool> {
    let output = Command::new(command)
        .args(args)
        .output()
        .context(format!("Failed to execute: {} {}", command, args.join(" ")))?;
    
    Ok(output.status.success())
}

/// Helper to run a command with sudo
#[cfg(unix)]
pub fn run_command_sudo(command: &str, args: &[&str]) -> Result<bool> {
    let mut sudo_args = vec![command];
    sudo_args.extend_from_slice(args);
    
    let output = Command::new("sudo")
        .args(sudo_args)
        .output()
        .context(format!("Failed to execute with sudo: {} {}", command, args.join(" ")))?;
    
    Ok(output.status.success())
}
