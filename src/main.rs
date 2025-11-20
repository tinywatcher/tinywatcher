mod alerts;
mod cli;
mod config;
mod log_monitor;
mod resource_monitor;

use alerts::AlertManager;
use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use log_monitor::LogMonitor;
use resource_monitor::ResourceMonitor;
use std::sync::Arc;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    match cli.command {
        Commands::Watch {
            config: config_path,
            file,
            container,
            no_resources,
        } => {
            handle_watch(config_path, file, container, no_resources).await?;
        }
        Commands::Test { config } => {
            handle_test(config).await?;
        }
    }

    Ok(())
}

async fn handle_watch(
    config_path: Option<std::path::PathBuf>,
    files: Vec<std::path::PathBuf>,
    containers: Vec<String>,
    no_resources: bool,
) -> Result<()> {
    // Load or create config
    let mut config = if let Some(path) = config_path {
        Config::from_file(path.to_str().context("Invalid config path")?)?
    } else {
        Config {
            inputs: config::Inputs::default(),
            alerts: config::Alerts::default(),
            rules: Vec::new(),
            resources: None,
        }
    };

    // Merge CLI arguments
    config.merge_with_cli(files, containers);

    // Check if we have anything to watch
    if config.inputs.files.is_empty()
        && config.inputs.containers.is_empty()
        && (no_resources || config.resources.is_none())
    {
        anyhow::bail!("Nothing to watch! Provide --file, --container, or configure resources.");
    }

    tracing::info!("🚀 Starting TinyWatcher...");

    // Create alert manager
    let alert_manager = Arc::new(AlertManager::new(config.alerts.clone()));

    // Spawn log monitoring tasks
    let mut tasks = Vec::new();

    if !config.rules.is_empty() {
        let log_monitor = Arc::new(
            LogMonitor::new(config.rules.clone(), alert_manager.clone())
                .context("Failed to create log monitor")?,
        );

        // Watch files
        for file in config.inputs.files {
            let monitor = log_monitor.clone();
            let file_clone = file.clone();
            tasks.push(tokio::spawn(async move {
                if let Err(e) = monitor.watch_file(file_clone.clone()).await {
                    tracing::error!("Error watching file {}: {}", file_clone.display(), e);
                }
            }));
        }

        // Watch containers
        for container in config.inputs.containers {
            let monitor = log_monitor.clone();
            let container_clone = container.clone();
            tasks.push(tokio::spawn(async move {
                if let Err(e) = monitor.watch_container(container_clone.clone()).await {
                    tracing::error!("Error watching container {}: {}", container_clone, e);
                }
            }));
        }
    } else if !config.inputs.files.is_empty() || !config.inputs.containers.is_empty() {
        tracing::warn!("Log sources configured but no rules defined!");
    }

    // Start resource monitoring
    if !no_resources {
        if let Some(resource_config) = config.resources {
            let resource_monitor = ResourceMonitor::new(resource_config, alert_manager.clone());
            tasks.push(tokio::spawn(async move {
                resource_monitor.start().await;
            }));
        }
    }

    // Wait for all tasks
    if tasks.is_empty() {
        anyhow::bail!("No monitoring tasks started!");
    }

    tracing::info!("✅ TinyWatcher is running. Press Ctrl+C to stop.");

    // Wait for any task to complete (which shouldn't happen unless there's an error)
    let (result, _, _) = futures::future::select_all(tasks).await;
    result?;

    Ok(())
}

async fn handle_test(config_path: std::path::PathBuf) -> Result<()> {
    tracing::info!("Testing configuration: {}", config_path.display());

    let config = Config::from_file(config_path.to_str().context("Invalid config path")?)?;

    // Validate inputs
    println!("\n📁 Inputs:");
    println!("  Files: {}", config.inputs.files.len());
    for file in &config.inputs.files {
        println!("    - {}", file.display());
        if !file.exists() {
            println!("      ⚠️  File does not exist!");
        }
    }
    println!("  Containers: {}", config.inputs.containers.len());
    for container in &config.inputs.containers {
        println!("    - {}", container);
    }

    // Validate alerts
    println!("\n🔔 Alerts:");
    if let Some(slack) = &config.alerts.slack {
        println!("  Slack: configured ({}...)", &slack.chars().take(30).collect::<String>());
    } else {
        println!("  Slack: not configured");
    }
    if let Some(webhook) = &config.alerts.webhook {
        println!("  Webhook: configured ({}...)", &webhook.chars().take(30).collect::<String>());
    } else {
        println!("  Webhook: not configured");
    }

    // Validate rules
    println!("\n📋 Rules: {}", config.rules.len());
    for rule in &config.rules {
        println!("  - {}", rule.name);
        println!("    Pattern: {}", rule.pattern);
        println!("    Alert: {:?}", rule.alert);
        println!("    Cooldown: {}s", rule.cooldown);

        // Test regex compilation
        match regex::Regex::new(&rule.pattern) {
            Ok(_) => println!("    ✅ Pattern is valid"),
            Err(e) => println!("    ❌ Pattern is invalid: {}", e),
        }
    }

    // Validate resource monitoring
    if let Some(resources) = &config.resources {
        println!("\n💻 Resource Monitoring:");
        println!("  Interval: {}s", resources.interval);
        println!("  Thresholds:");
        if let Some(cpu) = resources.thresholds.cpu_percent {
            println!("    CPU: {}%", cpu);
        }
        if let Some(memory) = resources.thresholds.memory_percent {
            println!("    Memory: {}%", memory);
        }
        if let Some(disk) = resources.thresholds.disk_percent {
            println!("    Disk: {}%", disk);
        }
        println!("    Alert: {:?}", resources.thresholds.alert);
    } else {
        println!("\n💻 Resource Monitoring: not configured");
    }

    println!("\n✅ Configuration is valid!");

    Ok(())
}

