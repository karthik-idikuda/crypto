//! Validator-to-shard assignment.

use nexara_crypto::{Blake3Hash, WalletAddress};
use serde::{Serialize, Deserialize};
use crate::shard::NUM_SHARDS;

/// An assignment of a validator to one or more shards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardAssignment {
    pub validator: WalletAddress,
    pub shard_ids: Vec<u16>,
    pub epoch: u64,
    pub primary_shard: u16,
}

/// Assign validators to shards deterministically based on epoch seed.
///
/// Each validator is assigned to `shards_per_validator` shards based on
/// BLAKE3(seed || validator_address || shard_index).
pub fn assign_validators_to_shards(
    validators: &[WalletAddress],
    epoch_seed: &Blake3Hash,
    shards_per_validator: usize,
    epoch: u64,
) -> Vec<ShardAssignment> {
    validators.iter().map(|v| {
        let mut shard_ids = Vec::new();
        for i in 0..shards_per_validator {
            let mut data = Vec::new();
            data.extend_from_slice(epoch_seed.as_bytes());
            data.extend_from_slice(&v.0);
            data.extend_from_slice(&(i as u32).to_le_bytes());
            let hash = Blake3Hash::compute(&data);
            let bytes = hash.as_bytes();
            let shard = u16::from_le_bytes([bytes[0], bytes[1]]) % NUM_SHARDS;
            if !shard_ids.contains(&shard) {
                shard_ids.push(shard);
            }
        }
        // Ensure at least one shard
        if shard_ids.is_empty() {
            shard_ids.push(0);
        }
        let primary_shard = shard_ids[0];
        ShardAssignment {
            validator: *v,
            shard_ids,
            epoch,
            primary_shard,
        }
    }).collect()
}

/// Get all validators assigned to a specific shard.
pub fn validators_for_shard(assignments: &[ShardAssignment], shard_id: u16) -> Vec<WalletAddress> {
    assignments.iter()
        .filter(|a| a.shard_ids.contains(&shard_id))
        .map(|a| a.validator)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment() {
        let validators = vec![WalletAddress::zero()];
        let seed = Blake3Hash::compute(b"epoch-seed");
        let assignments = assign_validators_to_shards(&validators, &seed, 3, 1);
        assert_eq!(assignments.len(), 1);
        assert!(!assignments[0].shard_ids.is_empty());
        for s in &assignments[0].shard_ids {
            assert!(*s < NUM_SHARDS);
        }
    }

    #[test]
    fn test_deterministic_assignment() {
        let validators = vec![WalletAddress::zero()];
        let seed = Blake3Hash::compute(b"epoch-seed");
        let a1 = assign_validators_to_shards(&validators, &seed, 3, 1);
        let a2 = assign_validators_to_shards(&validators, &seed, 3, 1);
        assert_eq!(a1[0].shard_ids, a2[0].shard_ids);
    }

    #[test]
    fn test_validators_for_shard() {
        let validators = vec![WalletAddress::zero()];
        let seed = Blake3Hash::compute(b"seed");
        let assignments = assign_validators_to_shards(&validators, &seed, 3, 1);
        let primary = assignments[0].primary_shard;
        let found = validators_for_shard(&assignments, primary);
        assert!(!found.is_empty());
    }
}
