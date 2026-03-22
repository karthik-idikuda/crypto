//! Node configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default RPC port.
pub const DEFAULT_RPC_PORT: u16 = 9944;
/// Default P2P port.
pub const DEFAULT_P2P_PORT: u16 = 30333;
/// Default data directory.
pub const DEFAULT_DATA_DIR: &str = ".nexara";

/// Full node configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub chain_id: u64,
    pub data_dir: PathBuf,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub validator: bool,
    pub validator_key_path: Option<PathBuf>,
    pub max_peers: usize,
    pub log_level: String,
    pub num_shards: u32,
    pub block_time_ms: u64,
    pub genesis_path: Option<PathBuf>,
    pub bootnodes: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            chain_id: 20240101,
            data_dir: dirs_or_default(),
            rpc_port: DEFAULT_RPC_PORT,
            p2p_port: DEFAULT_P2P_PORT,
            validator: false,
            validator_key_path: None,
            max_peers: 50,
            log_level: "info".to_string(),
            num_shards: 100,
            block_time_ms: 200,
            genesis_path: None,
            bootnodes: Vec::new(),
        }
    }
}

impl NodeConfig {
    /// Load config from a TOML/JSON file, falling back to defaults.
    pub fn load_or_default(path: Option<&str>) -> Self {
        if let Some(p) = path {
            if let Ok(data) = std::fs::read_to_string(p) {
                if let Ok(cfg) = serde_json::from_str::<NodeConfig>(&data) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    /// Validate configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.num_shards == 0 {
            return Err("num_shards must be > 0".into());
        }
        if self.block_time_ms == 0 {
            return Err("block_time_ms must be > 0".into());
        }
        if self.max_peers == 0 {
            return Err("max_peers must be > 0".into());
        }
        Ok(())
    }
}

fn dirs_or_default() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DEFAULT_DATA_DIR)
}

/// Minimal home-dir helper (avoids pulling in the `dirs` crate).
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = NodeConfig::default();
        assert_eq!(cfg.chain_id, 20240101);
        assert_eq!(cfg.rpc_port, DEFAULT_RPC_PORT);
        assert_eq!(cfg.num_shards, 100);
        assert!(!cfg.validator);
    }

    #[test]
    fn test_validate_ok() {
        let cfg = NodeConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_bad_shards() {
        let cfg = NodeConfig { num_shards: 0, ..NodeConfig::default() };
        assert!(cfg.validate().is_err());
    }
}
