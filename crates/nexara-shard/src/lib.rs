//! NEXARA 100-shard architecture.
//!
//! Manages shard state, cross-shard communication, beacon chain coordination,
//! and validator-to-shard assignment.

pub mod shard;
pub mod beacon;
pub mod crosslink;
pub mod assignment;
pub mod error;

pub use error::ShardError;
pub use shard::{Shard, ShardState, ShardConfig};
pub use beacon::BeaconChain;
pub use crosslink::{CrossLink, CrossShardMessage};
pub use assignment::ShardAssignment;
