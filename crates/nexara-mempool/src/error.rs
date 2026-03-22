use thiserror::Error;

#[derive(Debug, Error)]
pub enum MempoolError {
    #[error("Pool full: {0}")]
    PoolFull(String),
    #[error("Duplicate transaction: {0}")]
    Duplicate(String),
    #[error("Invalid transaction: {0}")]
    Invalid(String),
    #[error("Nonce too low: expected {expected}, got {got}")]
    NonceTooLow { expected: u64, got: u64 },
    #[error("Fee too low: minimum {minimum}")]
    FeeTooLow { minimum: u128 },
}
