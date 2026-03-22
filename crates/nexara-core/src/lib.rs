//! # NEXARA Core
//!
//! Core blockchain primitives: blocks, transactions, state, genesis configuration,
//! and error types for the NEXARA Layer 1 blockchain.

pub mod block;
pub mod transaction;
pub mod state;
pub mod genesis;
pub mod error;

pub use block::{Block, BlockHeader, CrossShardReceipt, calculate_merkle_root};
pub use transaction::{Transaction, TransactionType, TransactionPool};
pub use state::{ChainState, AccountState};
pub use genesis::{GenesisConfig, GenesisValidator, GenesisAllocation, AllocationType, VestingSchedule};
pub use error::CoreError;
