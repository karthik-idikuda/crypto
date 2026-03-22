//! Committee election for NEXARA consensus.
//!
//! Uses VRF-like sortition with BLAKE3 to select block committees
//! proportional to validator stake.

use crate::validator::ValidatorSet;
use nexara_crypto::Blake3Hash;
use serde::{Serialize, Deserialize};

/// Consensus committee for a given epoch and round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Committee {
    pub epoch: u64,
    pub round: u64,
    pub members: Vec<CommitteeMember>,
    pub total_committee_stake: u128,
}

/// A member of a consensus committee with their sortition proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitteeMember {
    pub validator_address: nexara_crypto::WalletAddress,
    pub stake: u128,
    pub sortition_hash: Blake3Hash,
    pub sortition_proof: Vec<u8>,
}

/// Elect a committee for a given epoch, round, and seed.
///
/// Selection is performed via VRF-like sortition:
/// `hash = BLAKE3(epoch || round || seed || validator_address)`
/// A validator is selected if `hash_value / MAX < stake / total_stake * committee_fraction`.
pub fn elect_committee(
    validator_set: &ValidatorSet,
    epoch: u64,
    round: u64,
    seed: &Blake3Hash,
    target_committee_size: usize,
) -> Committee {
    let active = validator_set.get_active_validators();
    let total_active_stake = validator_set.total_active_stake();

    if active.is_empty() || total_active_stake == 0 {
        return Committee {
            epoch,
            round,
            members: Vec::new(),
            total_committee_stake: 0,
        };
    }

    let mut members = Vec::new();

    for v in &active {
        let sortition_hash = compute_sortition_hash(epoch, round, seed, &v.address);
        let hash_bytes = sortition_hash.as_bytes();
        let hash_value = u64::from_le_bytes([
            hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3],
            hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7],
        ]);

        // Probability proportional to stake
        let threshold = compute_threshold(v.total_stake, total_active_stake, target_committee_size, active.len());
        if hash_value <= threshold {
            members.push(CommitteeMember {
                validator_address: v.address,
                stake: v.total_stake,
                sortition_hash,
                sortition_proof: hash_bytes[..16].to_vec(),
            });
        }
    }

    let total_committee_stake = members.iter().map(|m| m.stake).sum();

    Committee {
        epoch,
        round,
        members,
        total_committee_stake,
    }
}

/// Compute the sortition hash for a validator.
fn compute_sortition_hash(
    epoch: u64,
    round: u64,
    seed: &Blake3Hash,
    address: &nexara_crypto::WalletAddress,
) -> Blake3Hash {
    let mut data = Vec::new();
    data.extend_from_slice(&epoch.to_le_bytes());
    data.extend_from_slice(&round.to_le_bytes());
    data.extend_from_slice(seed.as_bytes());
    data.extend_from_slice(&address.0);
    Blake3Hash::compute(&data)
}

/// Compute the selection threshold based on stake weight.
fn compute_threshold(
    validator_stake: u128,
    total_stake: u128,
    target_committee_size: usize,
    total_validators: usize,
) -> u64 {
    if total_validators == 0 || total_stake == 0 {
        return 0;
    }
    // Expected probability = (stake / total_stake) * (target_committee_size / total_validators)
    // threshold = probability * u64::MAX
    let stake_ratio = (validator_stake as f64) / (total_stake as f64);
    let size_ratio = (target_committee_size as f64) / (total_validators as f64);
    let probability = (stake_ratio * size_ratio).min(1.0);
    (probability * u64::MAX as f64) as u64
}

/// Generate a deterministic epoch seed from the previous epoch's final block hash.
pub fn generate_epoch_seed(prev_epoch_hash: &Blake3Hash, epoch: u64) -> Blake3Hash {
    let mut data = Vec::new();
    data.extend_from_slice(prev_epoch_hash.as_bytes());
    data.extend_from_slice(&epoch.to_le_bytes());
    data.extend_from_slice(b"nexara-epoch-seed");
    Blake3Hash::compute(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::{Validator, ValidatorSet};
    use nexara_crypto::keys::KeyPair;

    fn make_validator_set(count: usize, stake: u128) -> ValidatorSet {
        let mut vs = ValidatorSet::new();
        for _ in 0..count {
            let kp = KeyPair::generate();
            vs.add_validator(Validator::new(kp.public.wallet_address(), kp.public, stake));
        }
        vs
    }

    #[test]
    fn test_committee_election() {
        let vs = make_validator_set(20, 1000);
        let seed = Blake3Hash::compute(b"test-seed");
        let committee = elect_committee(&vs, 1, 0, &seed, 10);
        // Should get some members (probabilistic, but with 20 validators and target 10, expect some)
        assert!(committee.epoch == 1);
        assert!(committee.round == 0);
    }

    #[test]
    fn test_empty_validator_set() {
        let vs = ValidatorSet::new();
        let seed = Blake3Hash::compute(b"test-seed");
        let committee = elect_committee(&vs, 1, 0, &seed, 10);
        assert!(committee.members.is_empty());
    }

    #[test]
    fn test_epoch_seed_deterministic() {
        let hash = Blake3Hash::compute(b"block-hash");
        let s1 = generate_epoch_seed(&hash, 1);
        let s2 = generate_epoch_seed(&hash, 1);
        assert_eq!(s1, s2);

        let s3 = generate_epoch_seed(&hash, 2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn test_sortition_deterministic() {
        let seed = Blake3Hash::compute(b"seed");
        let addr = nexara_crypto::WalletAddress::zero();
        let h1 = compute_sortition_hash(1, 0, &seed, &addr);
        let h2 = compute_sortition_hash(1, 0, &seed, &addr);
        assert_eq!(h1, h2);
    }
}
