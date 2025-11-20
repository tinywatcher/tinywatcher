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
        #[arg(short, long)]
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
        #[arg(short, long, required = true)]
        config: PathBuf,
    },
}
