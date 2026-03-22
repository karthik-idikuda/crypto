//! Validator management for the NEXARA consensus layer.

use serde::{Serialize, Deserialize};
use nexara_crypto::{Blake3Hash, WalletAddress, MlDsaPublicKey};

/// A NEXARA validator node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: WalletAddress,
    pub pubkey: MlDsaPublicKey,
    pub stake: u128,
    pub delegated_stake: u128,
    pub total_stake: u128,
    pub status: ValidatorStatus,
    pub shard_assignments: Vec<u16>,
    pub performance_score: f64,
    pub last_active_block: u64,
    pub slashing_history: Vec<SlashEvent>,
}

/// Validator operational status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorStatus {
    Active,
    Probation,
    Jailed,
    Exiting,
}

/// Record of a slashing event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashEvent {
    pub block_height: u64,
    pub offense: SlashOffense,
    pub amount_slashed: u128,
    pub evidence_hash: Blake3Hash,
}

/// Types of slashable offenses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashOffense {
    DoubleSign,
    InvalidZkProof,
    Unavailability,
    ByzantineBehavior,
    InvalidAttestation,
}

/// The complete set of validators for the current epoch.
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub epoch: u64,
    pub total_stake: u128,
}

impl Validator {
    /// Create a new validator.
    pub fn new(address: WalletAddress, pubkey: MlDsaPublicKey, stake: u128) -> Self {
        Validator {
            address,
            pubkey,
            stake,
            delegated_stake: 0,
            total_stake: stake,
            status: ValidatorStatus::Active,
            shard_assignments: Vec::new(),
            performance_score: 1.0,
            last_active_block: 0,
            slashing_history: Vec::new(),
        }
    }

    /// Update total stake (own + delegated).
    pub fn update_total_stake(&mut self) {
        self.total_stake = self.stake.saturating_add(self.delegated_stake);
    }

    /// Check if this validator is active.
    pub fn is_active(&self) -> bool {
        self.status == ValidatorStatus::Active
    }
}

impl ValidatorSet {
    /// Create a new empty validator set.
    pub fn new() -> Self {
        ValidatorSet {
            validators: Vec::new(),
            epoch: 0,
            total_stake: 0,
        }
    }

    /// Add a validator and update total stake.
    pub fn add_validator(&mut self, v: Validator) {
        self.total_stake = self.total_stake.saturating_add(v.total_stake);
        self.validators.push(v);
    }

    /// Remove a validator by address.
    pub fn remove_validator(&mut self, addr: &WalletAddress) {
        if let Some(pos) = self.validators.iter().position(|v| v.address == *addr) {
            let removed = self.validators.remove(pos);
            self.total_stake = self.total_stake.saturating_sub(removed.total_stake);
        }
    }

    /// Find a validator by address.
    pub fn get_validator(&self, addr: &WalletAddress) -> Option<&Validator> {
        self.validators.iter().find(|v| v.address == *addr)
    }

    /// Get a mutable reference to a validator.
    pub fn get_validator_mut(&mut self, addr: &WalletAddress) -> Option<&mut Validator> {
        self.validators.iter_mut().find(|v| v.address == *addr)
    }

    /// Get all active validators.
    pub fn get_active_validators(&self) -> Vec<&Validator> {
        self.validators.iter().filter(|v| v.is_active()).collect()
    }

    /// Sum of all active validators' stake.
    pub fn total_active_stake(&self) -> u128 {
        self.validators
            .iter()
            .filter(|v| v.is_active())
            .map(|v| v.total_stake)
            .sum()
    }

    /// Compute a hash of the validator set for block header.
    pub fn set_hash(&self) -> Blake3Hash {
        let mut data = Vec::new();
        data.extend_from_slice(&self.epoch.to_le_bytes());
        for v in &self.validators {
            data.extend_from_slice(&v.address.0);
            data.extend_from_slice(&v.total_stake.to_le_bytes());
        }
        Blake3Hash::compute(&data)
    }

    /// Number of validators.
    pub fn len(&self) -> usize {
        self.validators.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }
}

impl Default for ValidatorSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_crypto::keys::KeyPair;

    fn make_validator(stake: u128) -> Validator {
        let kp = KeyPair::generate();
        Validator::new(kp.public.wallet_address(), kp.public, stake)
    }

    #[test]
    fn test_validator_creation() {
        let v = make_validator(1000);
        assert!(v.is_active());
        assert_eq!(v.total_stake, 1000);
        assert_eq!(v.performance_score, 1.0);
    }

    #[test]
    fn test_validator_set() {
        let mut vs = ValidatorSet::new();
        vs.add_validator(make_validator(100));
        vs.add_validator(make_validator(200));
        assert_eq!(vs.len(), 2);
        assert_eq!(vs.total_active_stake(), 300);
    }

    #[test]
    fn test_validator_removal() {
        let mut vs = ValidatorSet::new();
        let v = make_validator(100);
        let addr = v.address;
        vs.add_validator(v);
        vs.add_validator(make_validator(200));

        vs.remove_validator(&addr);
        assert_eq!(vs.len(), 1);
        assert_eq!(vs.total_stake, 200);
    }

    #[test]
    fn test_set_hash_deterministic() {
        let mut vs = ValidatorSet::new();
        vs.add_validator(make_validator(100));
        let h1 = vs.set_hash();
        let h2 = vs.set_hash();
        assert_eq!(h1, h2);
    }
}
