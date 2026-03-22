//! Tokenomics error types.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TokenomicsError {
    #[error("Insufficient balance: need {need}, have {have}")]
    InsufficientBalance { need: u128, have: u128 },

    #[error("Vesting not yet unlocked")]
    VestingLocked,

    #[error("Invalid stake amount: {amount}, minimum: {minimum}")]
    InvalidStakeAmount { amount: u128, minimum: u128 },

    #[error("Stake not found")]
    StakeNotFound,

    #[error("Burn exceeds supply: burn {burn}, supply {supply}")]
    BurnExceedsSupply { burn: u128, supply: u128 },

    #[error("Cooldown period still active")]
    CooldownActive,

    #[error("Maximum supply reached")]
    MaxSupplyReached,
}
