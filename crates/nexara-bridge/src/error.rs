//! Bridge error types.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum BridgeError {
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    #[error("Invalid proof for transfer {0}")]
    InvalidProof(String),

    #[error("Transfer already processed: {0}")]
    DuplicateTransfer(String),

    #[error("Nullifier already spent: {0}")]
    NullifierSpent(String),

    #[error("Insufficient bridge liquidity: need {need}, have {have}")]
    InsufficientLiquidity { need: u128, have: u128 },

    #[error("Bridge paused for chain: {0}")]
    BridgePaused(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Timeout: transfer {0} expired")]
    TransferTimeout(String),

    #[error("Relayer error: {0}")]
    RelayerError(String),
}
