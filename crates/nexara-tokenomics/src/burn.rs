//! Token burn engine — deflationary mechanics.

use serde::{Deserialize, Serialize};
use crate::error::TokenomicsError;

/// Fee burn rate in basis points (50% = 5000 BPS).
pub const FEE_BURN_RATE_BPS: u32 = 5000;

/// Burn event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRecord {
    pub amount: u128,
    pub reason: BurnReason,
    pub block_height: u64,
    pub timestamp: u64,
}

/// Reason for burning tokens.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BurnReason {
    TransactionFee,
    SlashingPenalty,
    GovernanceBurn,
    ContractExecution,
    Manual,
}

/// Engine that tracks all burns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnEngine {
    pub total_burned: u128,
    pub records: Vec<BurnRecord>,
    pub fee_burn_rate_bps: u32,
}

impl BurnEngine {
    pub fn new() -> Self {
        BurnEngine {
            total_burned: 0,
            records: Vec::new(),
            fee_burn_rate_bps: FEE_BURN_RATE_BPS,
        }
    }

    /// Calculate the burn amount from a transaction fee.
    pub fn fee_burn_amount(&self, fee: u128) -> u128 {
        fee * (self.fee_burn_rate_bps as u128) / 10_000
    }

    /// Burn tokens and record the event.
    pub fn burn(
        &mut self,
        amount: u128,
        reason: BurnReason,
        block_height: u64,
        timestamp: u64,
        circulating: u128,
    ) -> Result<(), TokenomicsError> {
        if amount > circulating {
            return Err(TokenomicsError::BurnExceedsSupply {
                burn: amount,
                supply: circulating,
            });
        }
        self.total_burned = self.total_burned.saturating_add(amount);
        self.records.push(BurnRecord {
            amount,
            reason,
            block_height,
            timestamp,
        });
        Ok(())
    }

    /// Process a transaction fee: burn portion, return remainder.
    pub fn process_fee(
        &mut self,
        fee: u128,
        block_height: u64,
        timestamp: u64,
        circulating: u128,
    ) -> Result<(u128, u128), TokenomicsError> {
        let to_burn = self.fee_burn_amount(fee);
        let remainder = fee.saturating_sub(to_burn);
        self.burn(to_burn, BurnReason::TransactionFee, block_height, timestamp, circulating)?;
        Ok((to_burn, remainder))
    }

    /// Number of burn events.
    pub fn burn_count(&self) -> usize {
        self.records.len()
    }

    /// Average burn per event.
    pub fn average_burn(&self) -> u128 {
        if self.records.is_empty() {
            0
        } else {
            self.total_burned / (self.records.len() as u128)
        }
    }
}

impl Default for BurnEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_burn_amount() {
        let engine = BurnEngine::new();
        assert_eq!(engine.fee_burn_amount(1000), 500); // 50%
    }

    #[test]
    fn test_burn_record() {
        let mut engine = BurnEngine::new();
        engine.burn(1000, BurnReason::Manual, 1, 100, 10_000).unwrap();
        assert_eq!(engine.total_burned, 1000);
        assert_eq!(engine.burn_count(), 1);
    }

    #[test]
    fn test_burn_exceeds_supply() {
        let mut engine = BurnEngine::new();
        let result = engine.burn(10_000, BurnReason::Manual, 1, 100, 5_000);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_fee() {
        let mut engine = BurnEngine::new();
        let (burned, remainder) = engine.process_fee(2000, 1, 100, 1_000_000).unwrap();
        assert_eq!(burned, 1000);
        assert_eq!(remainder, 1000);
        assert_eq!(engine.total_burned, 1000);
    }

    #[test]
    fn test_average_burn() {
        let mut engine = BurnEngine::new();
        engine.burn(1000, BurnReason::TransactionFee, 1, 100, 1_000_000).unwrap();
        engine.burn(3000, BurnReason::SlashingPenalty, 2, 200, 1_000_000).unwrap();
        assert_eq!(engine.average_burn(), 2000);
    }
}
