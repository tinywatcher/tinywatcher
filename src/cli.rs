use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tinywatcher",
    version,
    about = "A tiny, zero-infrastructure observability tool for logs and system resources",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Watch logs and system resources
    Watch {
        /// Configuration file path
        #[arg(long)]
        config: Option<PathBuf>,

        /// Watch specific log files
        #[arg(short, long)]
        file: Vec<PathBuf>,

        /// Watch specific Docker containers
        #[arg(short = 'c', long)]
        container: Vec<String>,

        /// Disable resource monitoring
        #[arg(long)]
        no_resources: bool,
    },

    /// Test configuration and rules without watching
    Test {
        /// Configuration file path
        #[arg(long, required = true)]
        config: PathBuf,
    },

    /// Check rules against recent logs with highlighted matches
    Check {
        /// Configuration file path
        #[arg(long, required = true)]
        config: PathBuf,

        /// Number of lines to tail from each source (default: 100)
        #[arg(short = 'n', long, default_value = "100")]
        lines: usize,

        /// Watch specific log files (overrides config)
        #[arg(short, long)]
        file: Vec<PathBuf>,

        /// Watch specific Docker containers (overrides config)
        #[arg(short = 'c', long)]
        container: Vec<String>,
    },

    /// Start tinywatcher as a background service/daemon
    Start {
        /// Configuration file path (required for first-time setup)
        #[arg(long)]
        config: Option<PathBuf>,
    },

    /// Stop the tinywatcher background service/daemon
    Stop,

    /// Restart the tinywatcher background service/daemon
    Restart,

    /// Show the status of the tinywatcher background service/daemon
    Status,
}
