//! NEXARA Tokenomics - token economics engine.

pub mod error;
pub mod supply;
pub mod vesting;
pub mod staking;
pub mod burn;
pub mod pol;

pub use error::TokenomicsError;
pub use supply::{TokenSupply, TOTAL_SUPPLY};
pub use vesting::{VestingSchedule, VestingEntry};
pub use staking::{StakingPool, StakeInfo};
pub use burn::BurnEngine;
pub use pol::ProtocolOwnedLiquidity;
