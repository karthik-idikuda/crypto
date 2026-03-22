use thiserror::Error;

/// Errors that can occur in the core blockchain layer.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("duplicate transaction: {0}")]
    DuplicateTransaction(String),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },

    #[error("invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("shard out of range: {0} (max 99)")]
    ShardOutOfRange(u16),

    #[error("block too large: {size} bytes (max {max})")]
    BlockTooLarge { size: usize, max: usize },

    #[error("merkle root mismatch")]
    MerkleRootMismatch,

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("genesis error: {0}")]
    GenesisError(String),

    #[error("mempool full")]
    MempoolFull,

    #[error("account not found: {0}")]
    AccountNotFound(String),
}
