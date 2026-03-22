//! Delegated Proof of Quantum Stake (DPoQS) module.
//!
//! Handles stake delegation, quantum-secured validator election,
//! and delegation reward distribution.

use nexara_crypto::{Blake3Hash, WalletAddress};
use serde::{Serialize, Deserialize};

/// A delegation of stake from a delegator to a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator: WalletAddress,
    pub validator: WalletAddress,
    pub amount: u128,
    pub epoch_delegated: u64,
    pub reward_share_pct: u8,
}

/// DPoQS stake management.
pub struct DPoQS {
    pub delegations: Vec<Delegation>,
    pub minimum_stake: u128,
    pub epoch: u64,
}

impl DPoQS {
    /// Create a new DPoQS instance.
    pub fn new(minimum_stake: u128) -> Self {
        DPoQS {
            delegations: Vec::new(),
            minimum_stake,
            epoch: 0,
        }
    }

    /// Delegate stake to a validator.
    pub fn delegate(
        &mut self,
        delegator: WalletAddress,
        validator: WalletAddress,
        amount: u128,
        reward_share_pct: u8,
    ) -> Result<(), String> {
        if amount < self.minimum_stake {
            return Err(format!("Minimum stake is {}", self.minimum_stake));
        }
        if reward_share_pct > 100 {
            return Err("Reward share must be <= 100%".into());
        }
        self.delegations.push(Delegation {
            delegator,
            validator,
            amount,
            epoch_delegated: self.epoch,
            reward_share_pct,
        });
        Ok(())
    }

    /// Undelegate stake from a validator.
    pub fn undelegate(&mut self, delegator: &WalletAddress, validator: &WalletAddress) -> Option<Delegation> {
        if let Some(pos) = self.delegations.iter().position(|d| {
            d.delegator == *delegator && d.validator == *validator
        }) {
            Some(self.delegations.remove(pos))
        } else {
            None
        }
    }

    /// Get total delegated stake for a validator.
    pub fn total_delegated_to(&self, validator: &WalletAddress) -> u128 {
        self.delegations.iter()
            .filter(|d| d.validator == *validator)
            .map(|d| d.amount)
            .sum()
    }

    /// Get all delegations by a delegator.
    pub fn delegations_by(&self, delegator: &WalletAddress) -> Vec<&Delegation> {
        self.delegations.iter()
            .filter(|d| d.delegator == *delegator)
            .collect()
    }

    /// Calculate rewards for delegators of a validator given a block reward.
    pub fn calculate_delegation_rewards(
        &self,
        validator: &WalletAddress,
        block_reward: u128,
        validator_commission_pct: u8,
    ) -> Vec<(WalletAddress, u128)> {
        let total_delegated = self.total_delegated_to(validator);
        if total_delegated == 0 {
            return Vec::new();
        }

        let commission = block_reward.saturating_mul(validator_commission_pct as u128) / 100;
        let distributable = block_reward.saturating_sub(commission);

        self.delegations.iter()
            .filter(|d| d.validator == *validator)
            .map(|d| {
                let share = distributable.saturating_mul(d.amount) / total_delegated;
                (d.delegator, share)
            })
            .collect()
    }

    /// Advance to next epoch.
    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
    }

    /// Select a block proposer weighted by total stake (own + delegated).
    /// Uses BLAKE3-based deterministic selection.
    pub fn select_proposer(
        &self,
        validators: &[(WalletAddress, u128)],
        epoch: u64,
        round: u64,
    ) -> Option<WalletAddress> {
        if validators.is_empty() {
            return None;
        }

        let mut data = Vec::new();
        data.extend_from_slice(&epoch.to_le_bytes());
        data.extend_from_slice(&round.to_le_bytes());
        data.extend_from_slice(b"dpqs-proposer");
        let hash = Blake3Hash::compute(&data);
        let hash_bytes = hash.as_bytes();
        let hash_val = u64::from_le_bytes([
            hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3],
            hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7],
        ]);

        let total: u128 = validators.iter().map(|(_, s)| s).sum();
        if total == 0 {
            return None;
        }

        let target = (hash_val as u128) % total;
        let mut cumulative = 0u128;
        for (addr, stake) in validators {
            cumulative = cumulative.saturating_add(*stake);
            if cumulative > target {
                return Some(*addr);
            }
        }
        validators.last().map(|(addr, _)| *addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegation() {
        let mut dpqs = DPoQS::new(100);
        let delegator = WalletAddress::zero();
        let validator = WalletAddress::zero();
        assert!(dpqs.delegate(delegator, validator, 500, 10).is_ok());
        assert_eq!(dpqs.total_delegated_to(&validator), 500);
    }

    #[test]
    fn test_minimum_stake() {
        let mut dpqs = DPoQS::new(100);
        let result = dpqs.delegate(WalletAddress::zero(), WalletAddress::zero(), 50, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_undelegation() {
        let mut dpqs = DPoQS::new(100);
        let d = WalletAddress::zero();
        let v = WalletAddress::zero();
        dpqs.delegate(d, v, 500, 10).unwrap();
        let removed = dpqs.undelegate(&d, &v);
        assert!(removed.is_some());
        assert_eq!(dpqs.total_delegated_to(&v), 0);
    }

    #[test]
    fn test_reward_calculation() {
        let mut dpqs = DPoQS::new(100);
        let d1 = WalletAddress::zero();
        let v = WalletAddress::zero();
        dpqs.delegate(d1, v, 1000, 10).unwrap();

        let rewards = dpqs.calculate_delegation_rewards(&v, 100, 10);
        // 100 reward, 10% commission = 10, distributable = 90
        // d1 has 100% of delegated stake, gets 90
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].1, 90);
    }

    #[test]
    fn test_proposer_selection() {
        let dpqs = DPoQS::new(100);
        let validators = vec![
            (WalletAddress::zero(), 1000u128),
        ];
        let result = dpqs.select_proposer(&validators, 1, 0);
        assert!(result.is_some());
    }
}
