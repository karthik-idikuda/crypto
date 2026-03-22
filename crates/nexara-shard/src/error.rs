use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShardError {
    #[error("Invalid shard ID: {0}")]
    InvalidShardId(u16),
    #[error("Shard full: {0}")]
    ShardFull(String),
    #[error("Cross-shard error: {0}")]
    CrossShard(String),
    #[error("Beacon error: {0}")]
    Beacon(String),
    #[error("Assignment error: {0}")]
    Assignment(String),
    #[error("State error: {0}")]
    State(String),
}
