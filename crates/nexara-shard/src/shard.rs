//! Shard state and management.

use nexara_crypto::Blake3Hash;
use nexara_core::block::Block;
use nexara_core::state::ChainState;
use serde::{Serialize, Deserialize};

/// Number of shards in the NEXARA network.
pub const NUM_SHARDS: u16 = 100;

/// Configuration for a shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    pub shard_id: u16,
    pub max_transactions_per_block: usize,
    pub target_block_time_ms: u64,
    pub max_validators: usize,
}

impl Default for ShardConfig {
    fn default() -> Self {
        ShardConfig {
            shard_id: 0,
            max_transactions_per_block: 5000,
            target_block_time_ms: 200,
            max_validators: 100,
        }
    }
}

/// The state of a single shard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardState {
    pub shard_id: u16,
    pub latest_block_height: u64,
    pub latest_block_hash: Blake3Hash,
    pub state_root: Blake3Hash,
    pub validator_count: usize,
    pub total_transactions: u64,
    pub pending_cross_shard: u64,
}

/// A shard in the NEXARA network.
pub struct Shard {
    pub config: ShardConfig,
    pub state: ShardState,
    pub chain_state: ChainState,
    pub blocks: Vec<Block>,
}

impl Shard {
    /// Create a new shard.
    pub fn new(shard_id: u16) -> Self {
        let config = ShardConfig {
            shard_id,
            ..Default::default()
        };
        let state = ShardState {
            shard_id,
            latest_block_height: 0,
            latest_block_hash: Blake3Hash::compute(
                format!("genesis-shard-{}", shard_id).as_bytes(),
            ),
            state_root: Blake3Hash::compute(b"empty-state"),
            validator_count: 0,
            total_transactions: 0,
            pending_cross_shard: 0,
        };
        Shard {
            config,
            state,
            chain_state: ChainState::new(),
            blocks: Vec::new(),
        }
    }

    /// Append a block to this shard.
    pub fn append_block(&mut self, block: Block) {
        self.state.total_transactions += block.tx_count() as u64;
        self.state.latest_block_height = block.header.height;
        self.state.latest_block_hash = block.hash();
        self.state.state_root = self.chain_state.state_root();
        self.blocks.push(block);
    }

    /// Get the latest block height.
    pub fn height(&self) -> u64 {
        self.state.latest_block_height
    }

    /// Determine which shard an address belongs to.
    pub fn shard_for_address(address: &[u8]) -> u16 {
        let hash = Blake3Hash::compute(address);
        let bytes = hash.as_bytes();
        let val = u16::from_le_bytes([bytes[0], bytes[1]]);
        val % NUM_SHARDS
    }

    /// Get shard ID.
    pub fn shard_id(&self) -> u16 {
        self.config.shard_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_creation() {
        let shard = Shard::new(0);
        assert_eq!(shard.shard_id(), 0);
        assert_eq!(shard.height(), 0);
    }

    #[test]
    fn test_shard_for_address() {
        let shard_id = Shard::shard_for_address(b"some-address");
        assert!(shard_id < NUM_SHARDS);
    }

    #[test]
    fn test_shard_for_address_deterministic() {
        let s1 = Shard::shard_for_address(b"addr1");
        let s2 = Shard::shard_for_address(b"addr1");
        assert_eq!(s1, s2);
    }

    #[test]
    fn test_num_shards() {
        assert_eq!(NUM_SHARDS, 100);
    }
}
