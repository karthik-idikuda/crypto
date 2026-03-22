use thiserror::Error;

/// Errors in the consensus layer.
#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("invalid attestation: {0}")]
    InvalidAttestation(String),

    #[error("quorum not reached: have {have}, need {need}")]
    QuorumNotReached { have: u128, need: u128 },

    #[error("duplicate attestation from {0}")]
    DuplicateAttestation(String),

    #[error("validator not found: {0}")]
    ValidatorNotFound(String),

    #[error("validator not in committee")]
    NotInCommittee,

    #[error("invalid block proposal: {0}")]
    InvalidProposal(String),

    #[error("slashing error: {0}")]
    SlashingError(String),

    #[error("zk proof verification failed")]
    ZkVerificationFailed,

    #[error("epoch transition error: {0}")]
    EpochError(String),

    #[error("{0}")]
    Other(String),
}
