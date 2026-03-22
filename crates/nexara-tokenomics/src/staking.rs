//! Staking pool and reward distribution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::TokenomicsError;

/// Minimum stake amount (1000 NXR in base units, 18 decimals).
pub const MIN_STAKE: u128 = 1_000_000_000_000_000_000_000;

/// Cooldown period in seconds (7 days).
pub const COOLDOWN_PERIOD: u64 = 7 * 24 * 3600;

/// Individual stake info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub staker: String,
    pub amount: u128,
    pub reward_debt: u128,
    pub staked_at: u64,
    pub unstake_requested_at: Option<u64>,
}

/// Staking pool managing all stakes and rewards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingPool {
    pub total_staked: u128,
    pub reward_per_token: u128,
    pub accumulated_rewards: u128,
    pub stakes: HashMap<String, StakeInfo>,
}

impl StakingPool {
    pub fn new() -> Self {
        StakingPool {
            total_staked: 0,
            reward_per_token: 0,
            accumulated_rewards: 0,
            stakes: HashMap::new(),
        }
    }

    /// Stake tokens.
    pub fn stake(&mut self, staker: String, amount: u128, now: u64) -> Result<(), TokenomicsError> {
        if amount < MIN_STAKE {
            return Err(TokenomicsError::InvalidStakeAmount {
                amount,
                minimum: MIN_STAKE,
            });
        }

        let entry = self.stakes.entry(staker.clone()).or_insert(StakeInfo {
            staker,
            amount: 0,
            reward_debt: 0,
            staked_at: now,
            unstake_requested_at: None,
        });

        entry.amount = entry.amount.saturating_add(amount);
        entry.reward_debt = entry.amount.saturating_mul(self.reward_per_token) / 1_000_000_000_000;
        self.total_staked = self.total_staked.saturating_add(amount);
        Ok(())
    }

    /// Request unstake. Initiates cooldown.
    pub fn request_unstake(&mut self, staker: &str, now: u64) -> Result<(), TokenomicsError> {
        let entry = self.stakes.get_mut(staker)
            .ok_or(TokenomicsError::StakeNotFound)?;

        if entry.unstake_requested_at.is_some() {
            return Err(TokenomicsError::CooldownActive);
        }
        entry.unstake_requested_at = Some(now);
        Ok(())
    }

    /// Complete unstake after cooldown.
    pub fn complete_unstake(&mut self, staker: &str, now: u64) -> Result<u128, TokenomicsError> {
        let entry = self.stakes.get(staker)
            .ok_or(TokenomicsError::StakeNotFound)?;

        let requested = entry.unstake_requested_at
            .ok_or(TokenomicsError::StakeNotFound)?;

        if now < requested + COOLDOWN_PERIOD {
            return Err(TokenomicsError::CooldownActive);
        }

        let amount = entry.amount;
        self.total_staked = self.total_staked.saturating_sub(amount);
        self.stakes.remove(staker);
        Ok(amount)
    }

    /// Distribute rewards proportionally to stakers.
    pub fn distribute_rewards(&mut self, reward_amount: u128) {
        if self.total_staked == 0 {
            return;
        }
        self.accumulated_rewards = self.accumulated_rewards.saturating_add(reward_amount);
        // Scale by 1e12 for precision
        self.reward_per_token = self.reward_per_token.saturating_add(
            reward_amount.saturating_mul(1_000_000_000_000) / self.total_staked,
        );
    }

    /// Calculate pending rewards for a staker.
    pub fn pending_rewards(&self, staker: &str) -> u128 {
        match self.stakes.get(staker) {
            Some(info) => {
                let gross = info.amount.saturating_mul(self.reward_per_token) / 1_000_000_000_000;
                gross.saturating_sub(info.reward_debt)
            }
            None => 0,
        }
    }

    /// Claim pending rewards.
    pub fn claim_rewards(&mut self, staker: &str) -> Result<u128, TokenomicsError> {
        let pending = self.pending_rewards(staker);
        let entry = self.stakes.get_mut(staker)
            .ok_or(TokenomicsError::StakeNotFound)?;
        entry.reward_debt = entry.amount.saturating_mul(self.reward_per_token) / 1_000_000_000_000;
        Ok(pending)
    }

    /// Get number of active stakers.
    pub fn staker_count(&self) -> usize {
        self.stakes.len()
    }
}

impl Default for StakingPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_below_minimum() {
        let mut pool = StakingPool::new();
        assert!(pool.stake("alice".into(), 100, 0).is_err());
    }

    #[test]
    fn test_stake_and_total() {
        let mut pool = StakingPool::new();
        pool.stake("alice".into(), MIN_STAKE, 0).unwrap();
        pool.stake("bob".into(), MIN_STAKE * 2, 0).unwrap();
        assert_eq!(pool.total_staked, MIN_STAKE * 3);
        assert_eq!(pool.staker_count(), 2);
    }

    #[test]
    fn test_unstake_cooldown() {
        let mut pool = StakingPool::new();
        pool.stake("alice".into(), MIN_STAKE, 0).unwrap();
        pool.request_unstake("alice", 100).unwrap();
        // Too early
        assert!(pool.complete_unstake("alice", 100 + COOLDOWN_PERIOD - 1).is_err());
        // After cooldown
        let amount = pool.complete_unstake("alice", 100 + COOLDOWN_PERIOD).unwrap();
        assert_eq!(amount, MIN_STAKE);
        assert_eq!(pool.total_staked, 0);
    }

    #[test]
    fn test_rewards_distribution() {
        let mut pool = StakingPool::new();
        pool.stake("alice".into(), MIN_STAKE, 0).unwrap();
        pool.stake("bob".into(), MIN_STAKE, 0).unwrap();
        let reward = MIN_STAKE * 2; // 2000 NXR in rewards
        pool.distribute_rewards(reward);
        // Each should get ~MIN_STAKE
        let alice_r = pool.pending_rewards("alice");
        let bob_r = pool.pending_rewards("bob");
        assert!(alice_r > 0);
        assert_eq!(alice_r, bob_r);
    }

    #[test]
    fn test_claim_rewards() {
        let mut pool = StakingPool::new();
        pool.stake("alice".into(), MIN_STAKE, 0).unwrap();
        let reward = MIN_STAKE; // 1000 NXR in rewards
        pool.distribute_rewards(reward);
        let claimed = pool.claim_rewards("alice").unwrap();
        assert_eq!(claimed, reward);
        // After claim, pending should be 0
        assert_eq!(pool.pending_rewards("alice"), 0);
    }
}
