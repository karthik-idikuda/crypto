//! # NEXARA Consensus — HybridSync
//!
//! A hybrid consensus engine combining Delegated Proof of Quantum Stake (DPoQS),
//! AI-Enhanced BFT, and ZK-SNARK finality proofs.

pub mod validator;
pub mod committee;
pub mod slashing;
pub mod hybridsync;
pub mod dpqs;
pub mod aibft;
pub mod zk_finality;

mod error;

pub use validator::{Validator, ValidatorStatus, ValidatorSet, SlashEvent, SlashOffense};
pub use committee::Committee;
pub use slashing::{SlashProposal, SlashEvidence};
pub use hybridsync::{HybridSyncEngine, Attestation, ConsensusResult, QuorumStatus};
pub use error::ConsensusError;
