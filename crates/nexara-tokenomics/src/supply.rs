//! Token supply management.

use serde::{Deserialize, Serialize};
use crate::error::TokenomicsError;

/// Total supply of NXR tokens (500 million, 18 decimals).
pub const TOTAL_SUPPLY: u128 = 500_000_000_000_000_000_000_000_000;

/// Allocation percentages (basis points, total = 10000).
pub const VALIDATOR_REWARDS_BPS: u32 = 3500;    // 35%
pub const ECOSYSTEM_FUND_BPS: u32 = 2000;       // 20%
pub const TEAM_BPS: u32 = 1500;                 // 15%
pub const COMMUNITY_BPS: u32 = 1000;            // 10%
pub const TREASURY_BPS: u32 = 1000;             // 10%
pub const LIQUIDITY_BPS: u32 = 500;             // 5%
pub const AIRDROP_BPS: u32 = 500;               // 5%

/// Token supply tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSupply {
    pub total_supply: u128,
    pub circulating_supply: u128,
    pub burned: u128,
    pub staked: u128,
    pub locked_in_vesting: u128,
    pub bridge_reserves: u128,
}

impl TokenSupply {
    /// Create initial supply state.
    pub fn genesis() -> Self {
        TokenSupply {
            total_supply: TOTAL_SUPPLY,
            circulating_supply: 0,
            burned: 0,
            staked: 0,
            locked_in_vesting: 0,
            bridge_reserves: 0,
        }
    }

    /// Calculate allocation for a given BPS.
    pub fn allocation_for_bps(bps: u32) -> u128 {
        TOTAL_SUPPLY * (bps as u128) / 10_000
    }

    /// Release tokens to circulation.
    pub fn release(&mut self, amount: u128) -> Result<(), TokenomicsError> {
        let available = self.total_supply
            .saturating_sub(self.circulating_supply)
            .saturating_sub(self.burned)
            .saturating_sub(self.staked)
            .saturating_sub(self.locked_in_vesting);
        if amount > available {
            return Err(TokenomicsError::InsufficientBalance { need: amount, have: available });
        }
        self.circulating_supply = self.circulating_supply.saturating_add(amount);
        Ok(())
    }

    /// Burn tokens from circulation.
    pub fn burn(&mut self, amount: u128) -> Result<(), TokenomicsError> {
        if amount > self.circulating_supply {
            return Err(TokenomicsError::BurnExceedsSupply {
                burn: amount,
                supply: self.circulating_supply,
            });
        }
        self.circulating_supply = self.circulating_supply.saturating_sub(amount);
        self.burned = self.burned.saturating_add(amount);
        Ok(())
    }

    /// Lock tokens for staking.
    pub fn stake(&mut self, amount: u128) -> Result<(), TokenomicsError> {
        if amount > self.circulating_supply {
            return Err(TokenomicsError::InsufficientBalance {
                need: amount, have: self.circulating_supply,
            });
        }
        self.circulating_supply = self.circulating_supply.saturating_sub(amount);
        self.staked = self.staked.saturating_add(amount);
        Ok(())
    }

    /// Unlock staked tokens.
    pub fn unstake(&mut self, amount: u128) -> Result<(), TokenomicsError> {
        if amount > self.staked {
            return Err(TokenomicsError::InsufficientBalance {
                need: amount, have: self.staked,
            });
        }
        self.staked = self.staked.saturating_sub(amount);
        self.circulating_supply = self.circulating_supply.saturating_add(amount);
        Ok(())
    }

    /// Effective total supply (total - burned).
    pub fn effective_supply(&self) -> u128 {
        self.total_supply.saturating_sub(self.burned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_supply() {
        let supply = TokenSupply::genesis();
        assert_eq!(supply.total_supply, TOTAL_SUPPLY);
        assert_eq!(supply.circulating_supply, 0);
    }

    #[test]
    fn test_allocation_bps() {
        let validator_alloc = TokenSupply::allocation_for_bps(VALIDATOR_REWARDS_BPS);
        assert_eq!(validator_alloc, TOTAL_SUPPLY * 35 / 100);
    }

    #[test]
    fn test_release_and_burn() {
        let mut supply = TokenSupply::genesis();
        supply.release(1_000_000).unwrap();
        assert_eq!(supply.circulating_supply, 1_000_000);
        supply.burn(500_000).unwrap();
        assert_eq!(supply.burned, 500_000);
        assert_eq!(supply.circulating_supply, 500_000);
    }

    #[test]
    fn test_stake_unstake() {
        let mut supply = TokenSupply::genesis();
        supply.release(1_000_000).unwrap();
        supply.stake(400_000).unwrap();
        assert_eq!(supply.staked, 400_000);
        assert_eq!(supply.circulating_supply, 600_000);
        supply.unstake(200_000).unwrap();
        assert_eq!(supply.staked, 200_000);
        assert_eq!(supply.circulating_supply, 800_000);
    }

    #[test]
    fn test_bps_total() {
        let total = VALIDATOR_REWARDS_BPS + ECOSYSTEM_FUND_BPS + TEAM_BPS
            + COMMUNITY_BPS + TREASURY_BPS + LIQUIDITY_BPS + AIRDROP_BPS;
        assert_eq!(total, 10_000);
    }

    #[test]
    fn test_effective_supply() {
        let mut supply = TokenSupply::genesis();
        supply.release(1_000).unwrap();
        supply.burn(500).unwrap();
        assert_eq!(supply.effective_supply(), TOTAL_SUPPLY - 500);
    }
}
