//! Beacon chain coordination for NEXARA shards.
//!
//! The beacon chain coordinates all 100 shards, manages epoch transitions,
//! and aggregates cross-shard state roots.

use nexara_crypto::Blake3Hash;
use crate::shard::ShardState;
use serde::{Serialize, Deserialize};

/// A beacon block that coordinates all shards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconBlock {
    pub height: u64,
    pub epoch: u64,
    pub timestamp: u64,
    pub previous_hash: Blake3Hash,
    pub shard_state_roots: Vec<(u16, Blake3Hash)>,
    pub aggregate_state_root: Blake3Hash,
    pub cross_links: Vec<BeaconCrossLink>,
}

/// A cross-link included in a beacon block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconCrossLink {
    pub shard_id: u16,
    pub block_height: u64,
    pub block_hash: Blake3Hash,
    pub state_root: Blake3Hash,
}

/// The beacon chain.
pub struct BeaconChain {
    pub blocks: Vec<BeaconBlock>,
    pub current_epoch: u64,
    pub shard_states: Vec<ShardState>,
}

impl BeaconChain {
    /// Create a new beacon chain.
    pub fn new() -> Self {
        BeaconChain {
            blocks: Vec::new(),
            current_epoch: 0,
            shard_states: Vec::new(),
        }
    }

    /// Update shard state in the beacon chain.
    pub fn update_shard_state(&mut self, state: ShardState) {
        if let Some(existing) = self.shard_states.iter_mut().find(|s| s.shard_id == state.shard_id) {
            *existing = state;
        } else {
            self.shard_states.push(state);
        }
    }

    /// Compute the aggregate state root across all shards.
    pub fn compute_aggregate_root(&self) -> Blake3Hash {
        let mut data = Vec::new();
        let mut sorted: Vec<_> = self.shard_states.iter().collect();
        sorted.sort_by_key(|s| s.shard_id);
        for s in sorted {
            data.extend_from_slice(&s.shard_id.to_le_bytes());
            data.extend_from_slice(s.state_root.as_bytes());
        }
        Blake3Hash::compute(&data)
    }

    /// Create a new beacon block.
    pub fn create_beacon_block(&mut self, timestamp: u64, cross_links: Vec<BeaconCrossLink>) -> BeaconBlock {
        let prev_hash = self.blocks.last()
            .map(|b| Blake3Hash::compute(&bincode::serialize(b).unwrap_or_default()))
            .unwrap_or_else(|| Blake3Hash::compute(b"genesis-beacon"));

        let shard_state_roots: Vec<_> = self.shard_states.iter()
            .map(|s| (s.shard_id, s.state_root))
            .collect();

        let aggregate_state_root = self.compute_aggregate_root();

        let height = self.blocks.len() as u64;
        let block = BeaconBlock {
            height,
            epoch: self.current_epoch,
            timestamp,
            previous_hash: prev_hash,
            shard_state_roots,
            aggregate_state_root,
            cross_links,
        };

        self.blocks.push(block.clone());
        block
    }

    /// Advance to the next epoch.
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
    }

    /// Get the latest beacon block height.
    pub fn height(&self) -> u64 {
        self.blocks.len() as u64
    }
}

impl Default for BeaconChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shard::ShardState;

    fn make_shard_state(id: u16) -> ShardState {
        ShardState {
            shard_id: id,
            latest_block_height: 0,
            latest_block_hash: Blake3Hash::compute(format!("shard-{}", id).as_bytes()),
            state_root: Blake3Hash::compute(format!("state-{}", id).as_bytes()),
            validator_count: 10,
            total_transactions: 0,
            pending_cross_shard: 0,
        }
    }

    #[test]
    fn test_beacon_creation() {
        let bc = BeaconChain::new();
        assert_eq!(bc.height(), 0);
        assert_eq!(bc.current_epoch, 0);
    }

    #[test]
    fn test_update_shard_state() {
        let mut bc = BeaconChain::new();
        bc.update_shard_state(make_shard_state(0));
        bc.update_shard_state(make_shard_state(1));
        assert_eq!(bc.shard_states.len(), 2);
    }

    #[test]
    fn test_create_beacon_block() {
        let mut bc = BeaconChain::new();
        bc.update_shard_state(make_shard_state(0));
        let block = bc.create_beacon_block(1000, Vec::new());
        assert_eq!(block.height, 0);
        assert_eq!(bc.height(), 1);
    }

    #[test]
    fn test_aggregate_root_deterministic() {
        let mut bc = BeaconChain::new();
        bc.update_shard_state(make_shard_state(0));
        bc.update_shard_state(make_shard_state(1));
        let r1 = bc.compute_aggregate_root();
        let r2 = bc.compute_aggregate_root();
        assert_eq!(r1, r2);
    }
}
