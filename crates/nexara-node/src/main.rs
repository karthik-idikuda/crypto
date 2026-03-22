//! NEXARA blockchain node entry point.

mod cli;
mod config;
mod rpc;

use anyhow::Result;
use tracing::{info, warn};

fn main() -> Result<()> {
    let args = cli::parse_args();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&args.log_level)),
        )
        .init();

    info!("NEXARA Node v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let mut node_config = config::NodeConfig::load_or_default(
        args.config.as_ref().and_then(|p| p.to_str()),
    );

    // Apply CLI overrides
    node_config.rpc_port = args.rpc_port;
    node_config.p2p_port = args.p2p_port;
    node_config.validator = args.validator;
    if let Some(dir) = args.data_dir {
        node_config.data_dir = dir;
    }
    if !args.bootnodes.is_empty() {
        node_config.bootnodes = args.bootnodes;
    }

    node_config.validate().map_err(|e| anyhow::anyhow!(e))?;

    // Handle subcommands
    match args.command {
        Some(cli::Commands::Init { genesis }) => {
            info!("Initializing chain...");
            if let Some(path) = genesis {
                info!("Using genesis config: {:?}", path);
            }
            info!("Data directory: {:?}", node_config.data_dir);
            std::fs::create_dir_all(&node_config.data_dir)?;
            info!("Chain initialized successfully.");
            return Ok(());
        }
        Some(cli::Commands::Export { output }) => {
            info!("Exporting chain data to {:?}", output);
            warn!("Export not yet implemented.");
            return Ok(());
        }
        Some(cli::Commands::Info) => {
            println!("NEXARA Node v{}", env!("CARGO_PKG_VERSION"));
            println!("Chain ID:    {}", node_config.chain_id);
            println!("Shards:      {}", node_config.num_shards);
            println!("Block time:  {}ms", node_config.block_time_ms);
            println!("RPC port:    {}", node_config.rpc_port);
            println!("P2P port:    {}", node_config.p2p_port);
            println!("Validator:   {}", node_config.validator);
            println!("Data dir:    {:?}", node_config.data_dir);
            return Ok(());
        }
        None => {}
    }

    // Start the node
    info!(
        chain_id = node_config.chain_id,
        shards = node_config.num_shards,
        validator = node_config.validator,
        "Starting NEXARA node"
    );

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        run_node(node_config).await
    })
}

async fn run_node(config: config::NodeConfig) -> Result<()> {
    info!("Initializing RPC server on port {}", config.rpc_port);
    let _rpc = rpc::RpcServer::new(config.rpc_port);

    info!("Initializing {} shards...", config.num_shards);

    if config.validator {
        info!("Running in validator mode");
    } else {
        info!("Running in full node mode");
    }

    info!("NEXARA node started. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down gracefully...");

    Ok(())
}
