mod alerts;
mod cli;
mod config;
mod log_monitor;
mod resource_monitor;
mod stream_monitor;

use alerts::AlertManager;
use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use config::Config;
use log_monitor::LogMonitor;
use regex::Regex;
use resource_monitor::ResourceMonitor;
use stream_monitor::StreamMonitor;
use std::io::Write;
use std::sync::Arc;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
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
        Commands::Check {
            config,
            lines,
            file,
            container,
        } => {
            handle_check(config, lines, file, container).await?;
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
            alerts: std::collections::HashMap::new(),
            rules: Vec::new(),
            resources: None,
            identity: config::Identity::default(),
        }
    };

    // Merge CLI arguments
    config.merge_with_cli(files, containers);

    // Check if we have anything to watch
    if config.inputs.files.is_empty()
        && config.inputs.containers.is_empty()
        && config.inputs.streams.is_empty()
        && (no_resources || config.resources.is_none())
    {
        anyhow::bail!("Nothing to watch! Provide --file, --container, --stream, or configure resources.");
    }

    let identity = config.identity.get_name();
    tracing::info!("🚀 Starting TinyWatcher (identity: {})...", identity);

    // Create alert manager and register handlers
    let mut alert_manager = AlertManager::new(identity);
    
    for (name, alert) in &config.alerts {
        use crate::config::{AlertOptions, AlertType};
        
        let handler: Arc<dyn alerts::AlertHandler> = match alert.alert_type {
            AlertType::Stdout => Arc::new(alerts::StdoutAlert::new(name.clone())),
            AlertType::Slack => {
                if let AlertOptions::Slack { url } = &alert.options {
                    Arc::new(alerts::SlackAlert::new(name.clone(), url.clone()))
                } else {
                    tracing::error!("Invalid Slack alert configuration for '{}'", name);
                    continue;
                }
            }
            AlertType::Webhook => {
                if let AlertOptions::Webhook { url } = &alert.options {
                    Arc::new(alerts::WebhookAlert::new(name.clone(), url.clone()))
                } else {
                    tracing::error!("Invalid Webhook alert configuration for '{}'", name);
                    continue;
                }
            }
            AlertType::Email => {
                #[cfg(unix)]
                {
                    if let AlertOptions::Email { from, to, smtp_server: _ } = &alert.options {
                        Arc::new(alerts::EmailAlert::new(name.clone(), from.clone(), to.clone()))
                    } else {
                        tracing::error!("Invalid Email alert configuration for '{}'", name);
                        continue;
                    }
                }
                
                #[cfg(not(unix))]
                {
                    if let AlertOptions::Email { from, to, smtp_server } = &alert.options {
                        Arc::new(alerts::EmailAlert::new(name.clone(), from.clone(), to.clone(), smtp_server.clone()))
                    } else {
                        tracing::error!("Invalid Email alert configuration for '{}'", name);
                        continue;
                    }
                }
            }
        };
        
        alert_manager.register(name.clone(), handler);
        tracing::debug!("Registered alert handler: {}", name);
    }
    
    let alert_manager = Arc::new(alert_manager);

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

        // Watch streams
        for stream_config in config.inputs.streams.clone() {
            let stream_monitor = Arc::new(
                StreamMonitor::new(config.rules.clone(), alert_manager.clone())
                    .context("Failed to create stream monitor")?,
            );
            tasks.push(tokio::spawn(async move {
                if let Err(e) = stream_monitor.watch_stream(stream_config.clone()).await {
                    tracing::error!("Error watching stream {}: {}", stream_config.get_name(), e);
                }
            }));
        }
    } else if !config.inputs.files.is_empty() || !config.inputs.containers.is_empty() || !config.inputs.streams.is_empty() {
        tracing::warn!("Log sources configured but no rules defined!");
        tracing::info!("Tip: Add a --config file with rules, or the logs will be monitored but no alerts will be triggered.");
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
        tracing::error!("No monitoring tasks started!");
        tracing::error!("You need to either:");
        tracing::error!("   - Provide a --config file with rules and inputs");
        tracing::error!("   - Or use --file/--container with a config file that has rules");
        tracing::error!("   - Or configure resource monitoring in your config file");
        anyhow::bail!("Nothing to monitor");
    }

    tracing::info!(" TinyWatcher is running. Press Ctrl+C to stop.");

    // Wait for any task to complete (which shouldn't happen unless there's an error)
    let (result, _, _) = futures::future::select_all(tasks).await;
    result?;

    Ok(())
}

async fn handle_test(config_path: std::path::PathBuf) -> Result<()> {
    tracing::info!("Testing configuration: {}", config_path.display());

    let config = Config::from_file(config_path.to_str().context("Invalid config path")?)?;
    validate_config(&config)?;

    Ok(())
}

fn validate_config(config: &Config) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    // Show identity
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(&mut stdout, "\nIDENTITY")?;
    stdout.reset()?;
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
    write!(&mut stdout, "  Name: ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(&mut stdout, "{}", config.identity.get_name())?;
    stdout.reset()?;
    
    if config.identity.name.is_none() {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_dimmed(true))?;
        writeln!(&mut stdout, "  (auto-detected from hostname)")?;
        stdout.reset()?;
    }

    // Validate inputs
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(&mut stdout, "\nINPUTS")?;
    stdout.reset()?;
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
    write!(&mut stdout, "  Files: ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(&mut stdout, "{}", config.inputs.files.len())?;
    stdout.reset()?;
    
    for file in &config.inputs.files {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        writeln!(&mut stdout, "    • {}", file.display())?;
        stdout.reset()?;
        if !file.exists() {
            write!(&mut stdout, "    ")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
            write!(&mut stdout, "[ERROR]")?;
            stdout.reset()?;
            writeln!(&mut stdout, " File does not exist")?;
        }
    }
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
    write!(&mut stdout, "  Containers: ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(&mut stdout, "{}", config.inputs.containers.len())?;
    stdout.reset()?;
    
    for container in &config.inputs.containers {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        writeln!(&mut stdout, "    • {}", container)?;
        stdout.reset()?;
    }
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
    write!(&mut stdout, "  Streams: ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(&mut stdout, "{}", config.inputs.streams.len())?;
    stdout.reset()?;
    
    for stream in &config.inputs.streams {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        writeln!(&mut stdout, "    • {} ({:?})", stream.get_name(), stream.stream_type)?;
        writeln!(&mut stdout, "      URL: {}", stream.url)?;
        if let Some(headers) = &stream.headers {
            writeln!(&mut stdout, "      Headers: {} configured", headers.len())?;
        }
        writeln!(&mut stdout, "      Reconnect delay: {}s", stream.get_reconnect_delay())?;
        stdout.reset()?;
    }

    // Validate alerts
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(&mut stdout, "\nALERTS")?;
    stdout.reset()?;
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
    write!(&mut stdout, "  Total: ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(&mut stdout, "{}", config.alerts.len())?;
    stdout.reset()?;
    
    for (name, alert) in &config.alerts {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
        write!(&mut stdout, "  {}", name)?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        write!(&mut stdout, " ({:?})", alert.alert_type)?;
        stdout.reset()?;
        
        match &alert.options {
            crate::config::AlertOptions::Slack { url } => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
                writeln!(&mut stdout, " → {}...", &url.chars().take(30).collect::<String>())?;
                stdout.reset()?;
            }
            crate::config::AlertOptions::Webhook { url } => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
                writeln!(&mut stdout, " → {}...", &url.chars().take(30).collect::<String>())?;
                stdout.reset()?;
            }
            crate::config::AlertOptions::Email { from, to, smtp_server } => {
                writeln!(&mut stdout)?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
                writeln!(&mut stdout, "      From: {}", from)?;
                writeln!(&mut stdout, "      To: [{}]", to.join(", "))?;
                if let Some(server) = smtp_server {
                    writeln!(&mut stdout, "      SMTP: {}", server)?;
                }
                stdout.reset()?;
            }
            crate::config::AlertOptions::Stdout {} => {
                writeln!(&mut stdout)?;
            }
        }
    }

    // Validate rules
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(&mut stdout, "\nRULES")?;
    stdout.reset()?;
    
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
    write!(&mut stdout, "  Total: ")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
    writeln!(&mut stdout, "{}", config.rules.len())?;
    stdout.reset()?;
    
    for rule in &config.rules {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
        writeln!(&mut stdout, "  {}", rule.name)?;
        stdout.reset()?;
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        writeln!(&mut stdout, "    Pattern: {}", rule.pattern)?;
        stdout.reset()?;
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        if rule.alert.len() == 1 {
            write!(&mut stdout, "    Alert: ")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            writeln!(&mut stdout, "{}", rule.alert[0])?;
        } else {
            write!(&mut stdout, "    Alerts: ")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            writeln!(&mut stdout, "[{}]", rule.alert.join(", "))?;
        }
        stdout.reset()?;
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        write!(&mut stdout, "    Cooldown: ")?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        writeln!(&mut stdout, "{}s", rule.cooldown)?;
        stdout.reset()?;

        // Show source filtering
        if let Some(sources) = &rule.sources {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            writeln!(&mut stdout, "    Sources:")?;
            if !sources.files.is_empty() {
                writeln!(&mut stdout, "      Files: [{}]", sources.files.iter()
                    .map(|f| f.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "))?;
            }
            if !sources.containers.is_empty() {
                writeln!(&mut stdout, "      Containers: [{}]", sources.containers.join(", "))?;
            }
            if !sources.streams.is_empty() {
                writeln!(&mut stdout, "      Streams: [{}]", sources.streams.join(", "))?;
            }
            stdout.reset()?;
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            writeln!(&mut stdout, "    Sources: all (no filter)")?;
            stdout.reset()?;
        }

        // Check if all alerts exist
        for alert_name in &rule.alert {
            if !config.alerts.contains_key(alert_name) {
                write!(&mut stdout, "    ")?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                write!(&mut stdout, "[ERROR]")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Alert '{}' not found in configuration", alert_name)?;
                anyhow::bail!("Rule '{}' references undefined alert '{}'", rule.name, alert_name);
            }
        }

        // Test regex compilation
        match Regex::new(&rule.pattern) {
            Ok(_) => {
                write!(&mut stdout, "    ")?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(&mut stdout, "[OK]")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Pattern is valid")?;
            }
            Err(e) => {
                write!(&mut stdout, "    ")?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                write!(&mut stdout, "[ERROR]")?;
                stdout.reset()?;
                writeln!(&mut stdout, " Pattern is invalid: {}", e)?;
                anyhow::bail!("Invalid regex pattern in rule: {}", rule.name);
            }
        }
    }

    // Validate resource monitoring
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(&mut stdout, "\nRESOURCE MONITORING")?;
    stdout.reset()?;
    
    if let Some(resources) = &config.resources {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        write!(&mut stdout, "  Interval: ")?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        writeln!(&mut stdout, "{}s", resources.interval)?;
        stdout.reset()?;
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        writeln!(&mut stdout, "  Thresholds:")?;
        stdout.reset()?;
        
        if let Some(cpu) = resources.thresholds.cpu_percent {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            write!(&mut stdout, "    CPU: ")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            writeln!(&mut stdout, "{}%", cpu)?;
            stdout.reset()?;
        }
        if let Some(memory) = resources.thresholds.memory_percent {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            write!(&mut stdout, "    Memory: ")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            writeln!(&mut stdout, "{}%", memory)?;
            stdout.reset()?;
        }
        if let Some(disk) = resources.thresholds.disk_percent {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
            write!(&mut stdout, "    Disk: ")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            writeln!(&mut stdout, "{}%", disk)?;
            stdout.reset()?;
        }
        
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        write!(&mut stdout, "    Alert: ")?;
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        writeln!(&mut stdout, "{}", resources.thresholds.alert)?;
        stdout.reset()?;
        
        // Check if alert exists
        if !config.alerts.contains_key(&resources.thresholds.alert) {
            write!(&mut stdout, "    ")?;
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
            write!(&mut stdout, "[ERROR]")?;
            stdout.reset()?;
            writeln!(&mut stdout, " Alert '{}' not found in configuration", resources.thresholds.alert)?;
            anyhow::bail!("Resource monitoring references undefined alert '{}'", resources.thresholds.alert);
        }
    } else {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        writeln!(&mut stdout, "  Not configured")?;
        stdout.reset()?;
    }

    // Final success message
    writeln!(&mut stdout)?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
    writeln!(&mut stdout, "Configuration is valid!")?;
    stdout.reset()?;

    Ok(())
}

async fn handle_check(
    config_path: std::path::PathBuf,
    lines: usize,
    cli_files: Vec<std::path::PathBuf>,
    cli_containers: Vec<String>,
) -> Result<()> {
    use tokio::process::Command;

    let mut config = Config::from_file(config_path.to_str().context("Invalid config path")?)?;

    // Override with CLI args if provided
    if !cli_files.is_empty() {
        config.inputs.files = cli_files;
    }
    if !cli_containers.is_empty() {
        config.inputs.containers = cli_containers;
    }

    // First, validate the configuration
    validate_config(&config)?;

    if config.rules.is_empty() {
        tracing::error!("No rules defined in configuration!");
        anyhow::bail!("Cannot check logs without rules");
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(" Checking last {} lines of logs...\n", lines);
    tracing::info!("Starting log check...");

    // Compile rules (validation already checked they compile)
    let compiled_rules: Vec<(String, Regex)> = config
        .rules
        .iter()
        .map(|rule| {
            Ok((
                rule.name.clone(),
                Regex::new(&rule.pattern).unwrap(), // Safe because validate_config already checked
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut total_matches = 0;

    // Check files
    for file in &config.inputs.files {
        println!(" Checking file: {}", file.display());
        
        if !file.exists() {
            println!("    File does not exist, skipping...\n");
            continue;
        }

        let output = Command::new("tail")
            .arg("-n")
            .arg(lines.to_string())
            .arg(file)
            .output()
            .await
            .context(format!("Failed to tail file: {}", file.display()))?;

        let log_content = String::from_utf8_lossy(&output.stdout);
        let matches = check_logs_for_rules(&log_content, &compiled_rules);
        total_matches += matches;
        println!();
    }

    // Check containers
    for container in &config.inputs.containers {
        println!(" Checking container: {}", container);

        let output = Command::new("docker")
            .arg("logs")
            .arg("--tail")
            .arg(lines.to_string())
            .arg(container)
            .output()
            .await;

        match output {
            Ok(output) => {
                // Check both stdout and stderr
                let stdout_content = String::from_utf8_lossy(&output.stdout);
                let stderr_content = String::from_utf8_lossy(&output.stderr);
                
                let matches = check_logs_for_rules(&stdout_content, &compiled_rules)
                    + check_logs_for_rules(&stderr_content, &compiled_rules);
                total_matches += matches;
            }
            Err(e) => {
                println!("    Failed to get logs: {}\n", e);
                continue;
            }
        }
        println!();
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if total_matches > 0 {
        println!(" Found {} total matches", total_matches);
    } else {
        println!("  No matches found in the checked logs");
    }

    Ok(())
}

fn check_logs_for_rules(log_content: &str, rules: &[(String, Regex)]) -> usize {
    let mut match_count = 0;

    for line in log_content.lines() {
        for (rule_name, regex) in rules {
            if let Some(mat) = regex.find(line) {
                match_count += 1;
                
                // Highlight the match
                let before = &line[..mat.start()];
                let matched = &line[mat.start()..mat.end()];
                let after = &line[mat.end()..];
                
                println!("  ✓ [{}]", rule_name);
                println!("    {}\x1b[1;33m{}\x1b[0m{}", before, matched, after);
            }
        }
    }

    match_count
}
