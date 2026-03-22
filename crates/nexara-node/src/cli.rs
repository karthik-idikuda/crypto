//! CLI argument parsing.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// NEXARA blockchain node
#[derive(Parser, Debug)]
#[command(name = "nexara", version, about = "NEXARA Layer 1 Blockchain Node")]
pub struct Cli {
    /// Path to config file (JSON)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Enable validator mode
    #[arg(long)]
    pub validator: bool,

    /// RPC server port
    #[arg(long, default_value_t = 9944)]
    pub rpc_port: u16,

    /// P2P listen port
    #[arg(long, default_value_t = 30333)]
    pub p2p_port: u16,

    /// Data directory
    #[arg(long)]
    pub data_dir: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Bootnode multiaddr(s)
    #[arg(long)]
    pub bootnodes: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new chain with genesis
    Init {
        /// Path to genesis config
        #[arg(long)]
        genesis: Option<PathBuf>,
    },
    /// Export chain data
    Export {
        /// Output path
        #[arg(long)]
        output: PathBuf,
    },
    /// Show node version and configuration
    Info,
}

/// Parse CLI arguments.
pub fn parse_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["nexara"]);
        assert_eq!(cli.rpc_port, 9944);
        assert_eq!(cli.p2p_port, 30333);
        assert!(!cli.validator);
    }

    #[test]
    fn test_cli_validator_flag() {
        let cli = Cli::parse_from(["nexara", "--validator"]);
        assert!(cli.validator);
    }

    #[test]
    fn test_cli_init_subcommand() {
        let cli = Cli::parse_from(["nexara", "init"]);
        assert!(matches!(cli.command, Some(Commands::Init { .. })));
    }
}
