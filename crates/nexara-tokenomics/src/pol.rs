//! Protocol-Owned Liquidity (POL) management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::TokenomicsError;

/// A liquidity pool pairing NXR with another asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub pair_asset: String,
    pub nxr_amount: u128,
    pub pair_amount: u128,
    pub lp_tokens: u128,
}

/// Protocol-Owned Liquidity manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolOwnedLiquidity {
    pub pools: HashMap<String, LiquidityPool>,
    pub total_nxr_deployed: u128,
    pub treasury_balance: u128,
}

impl ProtocolOwnedLiquidity {
    pub fn new(treasury_balance: u128) -> Self {
        ProtocolOwnedLiquidity {
            pools: HashMap::new(),
            total_nxr_deployed: 0,
            treasury_balance,
        }
    }

    /// Add liquidity to a pool.
    pub fn add_liquidity(
        &mut self,
        pair_asset: &str,
        nxr_amount: u128,
        pair_amount: u128,
    ) -> Result<u128, TokenomicsError> {
        if nxr_amount > self.treasury_balance {
            return Err(TokenomicsError::InsufficientBalance {
                need: nxr_amount,
                have: self.treasury_balance,
            });
        }

        self.treasury_balance = self.treasury_balance.saturating_sub(nxr_amount);
        self.total_nxr_deployed = self.total_nxr_deployed.saturating_add(nxr_amount);

        // Simple LP token calculation: sqrt(nxr * pair) approximated as (nxr + pair) / 2
        let lp_tokens = (nxr_amount + pair_amount) / 2;

        let pool = self.pools.entry(pair_asset.to_string()).or_insert(LiquidityPool {
            pair_asset: pair_asset.to_string(),
            nxr_amount: 0,
            pair_amount: 0,
            lp_tokens: 0,
        });

        pool.nxr_amount = pool.nxr_amount.saturating_add(nxr_amount);
        pool.pair_amount = pool.pair_amount.saturating_add(pair_amount);
        pool.lp_tokens = pool.lp_tokens.saturating_add(lp_tokens);

        Ok(lp_tokens)
    }

    /// Remove liquidity from a pool.
    pub fn remove_liquidity(
        &mut self,
        pair_asset: &str,
        lp_tokens_to_remove: u128,
    ) -> Result<(u128, u128), TokenomicsError> {
        let pool = self.pools.get_mut(pair_asset)
            .ok_or(TokenomicsError::InsufficientBalance { need: lp_tokens_to_remove, have: 0 })?;

        if lp_tokens_to_remove > pool.lp_tokens {
            return Err(TokenomicsError::InsufficientBalance {
                need: lp_tokens_to_remove,
                have: pool.lp_tokens,
            });
        }

        let share = lp_tokens_to_remove as f64 / pool.lp_tokens as f64;
        let nxr_out = (pool.nxr_amount as f64 * share) as u128;
        let pair_out = (pool.pair_amount as f64 * share) as u128;

        pool.nxr_amount = pool.nxr_amount.saturating_sub(nxr_out);
        pool.pair_amount = pool.pair_amount.saturating_sub(pair_out);
        pool.lp_tokens = pool.lp_tokens.saturating_sub(lp_tokens_to_remove);

        self.total_nxr_deployed = self.total_nxr_deployed.saturating_sub(nxr_out);
        self.treasury_balance = self.treasury_balance.saturating_add(nxr_out);

        Ok((nxr_out, pair_out))
    }

    /// Get the total value locked across all pools (in NXR terms, pair counted 1:1).
    pub fn total_value_locked(&self) -> u128 {
        self.pools.values()
            .map(|p| p.nxr_amount.saturating_add(p.pair_amount))
            .sum()
    }

    /// Number of active pools.
    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_liquidity() {
        let mut pol = ProtocolOwnedLiquidity::new(1_000_000);
        let lp = pol.add_liquidity("USDC", 500_000, 500_000).unwrap();
        assert_eq!(lp, 500_000);
        assert_eq!(pol.treasury_balance, 500_000);
        assert_eq!(pol.total_nxr_deployed, 500_000);
        assert_eq!(pol.pool_count(), 1);
    }

    #[test]
    fn test_insufficient_treasury() {
        let mut pol = ProtocolOwnedLiquidity::new(100);
        assert!(pol.add_liquidity("USDC", 500, 500).is_err());
    }

    #[test]
    fn test_remove_liquidity() {
        let mut pol = ProtocolOwnedLiquidity::new(1_000_000);
        let lp = pol.add_liquidity("USDC", 500_000, 500_000).unwrap();
        let (nxr, pair) = pol.remove_liquidity("USDC", lp / 2).unwrap();
        assert!(nxr > 0 && pair > 0);
        assert_eq!(pol.pool_count(), 1);
    }

    #[test]
    fn test_tvl() {
        let mut pol = ProtocolOwnedLiquidity::new(2_000_000);
        pol.add_liquidity("USDC", 500_000, 500_000).unwrap();
        pol.add_liquidity("ETH", 300_000, 300_000).unwrap();
        assert_eq!(pol.total_value_locked(), 1_600_000);
    }

    #[test]
    fn test_multiple_adds_same_pool() {
        let mut pol = ProtocolOwnedLiquidity::new(2_000_000);
        pol.add_liquidity("USDC", 100_000, 100_000).unwrap();
        pol.add_liquidity("USDC", 200_000, 200_000).unwrap();
        let pool = pol.pools.get("USDC").unwrap();
        assert_eq!(pool.nxr_amount, 300_000);
    }
}
