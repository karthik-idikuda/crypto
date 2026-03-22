use thiserror::Error;

/// Errors that can occur during cryptographic operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid hex string: {0}")]
    InvalidHex(String),

    #[error("invalid key length: expected {expected}, got {got}")]
    InvalidKeyLength { expected: usize, got: usize },

    #[error("invalid signature length: expected {expected}, got {got}")]
    InvalidSignatureLength { expected: usize, got: usize },

    #[error("signature verification failed")]
    VerificationFailed,

    #[error("KEM decapsulation failed")]
    DecapsulationFailed,

    #[error("MPC reconstruction failed: {0}")]
    MpcReconstructionFailed(String),

    #[error("insufficient shares: need {threshold}, got {got}")]
    InsufficientShares { threshold: u8, got: u8 },

    #[error("invalid share index")]
    InvalidShareIndex,

    #[error("serialization error: {0}")]
    Serialization(String),
}
