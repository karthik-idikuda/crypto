//! Slashing module for the NEXARA consensus layer.
//!
//! Handles evidence validation, slash amount calculation, and slash proposal management.

use crate::validator::{SlashOffense, SlashEvent};
use nexara_crypto::{Blake3Hash, WalletAddress};
use serde::{Serialize, Deserialize};

/// A slash proposal submitted by a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashProposal {
    pub proposer: WalletAddress,
    pub offender: WalletAddress,
    pub offense: SlashOffense,
    pub evidence: SlashEvidence,
    pub evidence_hash: Blake3Hash,
    pub block_height: u64,
}

/// Evidence for a slashable offense.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashEvidence {
    /// Two signed blocks at the same height.
    DoubleSign {
        block_hash_a: Blake3Hash,
        block_hash_b: Blake3Hash,
        height: u64,
    },
    /// An invalid ZK proof submission.
    InvalidZkProof {
        proof_hash: Blake3Hash,
        expected_root: Blake3Hash,
    },
    /// Validator unresponsive for a number of consecutive blocks.
    Unavailability {
        missed_blocks: u64,
        window_start: u64,
        window_end: u64,
    },
    /// General Byzantine behavior evidence.
    Byzantine {
        description: String,
        evidence_data: Vec<u8>,
    },
}

/// Calculate the slash amount based on the offense type and validator stake.
///
/// - DoubleSign: 100% of stake
/// - InvalidZkProof: 50% of stake
/// - Unavailability: 10% of stake
/// - ByzantineBehavior: 75% of stake
/// - InvalidAttestation: 25% of stake
pub fn calculate_slash_amount(offense: SlashOffense, validator_stake: u128) -> u128 {
    let pct = match offense {
        SlashOffense::DoubleSign => 100,
        SlashOffense::InvalidZkProof => 50,
        SlashOffense::Unavailability => 10,
        SlashOffense::ByzantineBehavior => 75,
        SlashOffense::InvalidAttestation => 25,
    };
    validator_stake.saturating_mul(pct) / 100
}

/// Validate the evidence in a slash proposal.
pub fn validate_slash_evidence(proposal: &SlashProposal) -> bool {
    // Basic validation: evidence type must match offense
    match (&proposal.evidence, proposal.offense) {
        (SlashEvidence::DoubleSign { block_hash_a, block_hash_b, .. }, SlashOffense::DoubleSign) => {
            block_hash_a != block_hash_b
        }
        (SlashEvidence::InvalidZkProof { proof_hash, expected_root }, SlashOffense::InvalidZkProof) => {
            proof_hash != expected_root
        }
        (SlashEvidence::Unavailability { missed_blocks, .. }, SlashOffense::Unavailability) => {
            *missed_blocks > 100 // Must have missed more than 100 blocks
        }
        (SlashEvidence::Byzantine { evidence_data, .. }, SlashOffense::ByzantineBehavior) => {
            !evidence_data.is_empty()
        }
        _ => false,
    }
}

/// Create a slash event from a validated proposal.
pub fn create_slash_event(proposal: &SlashProposal, amount: u128) -> SlashEvent {
    SlashEvent {
        block_height: proposal.block_height,
        offense: proposal.offense,
        amount_slashed: amount,
        evidence_hash: proposal.evidence_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slash_amounts() {
        let stake = 1000u128;
        assert_eq!(calculate_slash_amount(SlashOffense::DoubleSign, stake), 1000);
        assert_eq!(calculate_slash_amount(SlashOffense::InvalidZkProof, stake), 500);
        assert_eq!(calculate_slash_amount(SlashOffense::Unavailability, stake), 100);
        assert_eq!(calculate_slash_amount(SlashOffense::ByzantineBehavior, stake), 750);
        assert_eq!(calculate_slash_amount(SlashOffense::InvalidAttestation, stake), 250);
    }

    #[test]
    fn test_validate_double_sign() {
        let proposal = SlashProposal {
            proposer: WalletAddress::zero(),
            offender: WalletAddress::zero(),
            offense: SlashOffense::DoubleSign,
            evidence: SlashEvidence::DoubleSign {
                block_hash_a: Blake3Hash::compute(b"block-a"),
                block_hash_b: Blake3Hash::compute(b"block-b"),
                height: 100,
            },
            evidence_hash: Blake3Hash::compute(b"evidence"),
            block_height: 101,
        };
        assert!(validate_slash_evidence(&proposal));
    }

    #[test]
    fn test_validate_unavailability() {
        let proposal = SlashProposal {
            proposer: WalletAddress::zero(),
            offender: WalletAddress::zero(),
            offense: SlashOffense::Unavailability,
            evidence: SlashEvidence::Unavailability {
                missed_blocks: 200,
                window_start: 0,
                window_end: 200,
            },
            evidence_hash: Blake3Hash::compute(b"evidence"),
            block_height: 201,
        };
        assert!(validate_slash_evidence(&proposal));
    }

    #[test]
    fn test_mismatched_evidence_type() {
        let proposal = SlashProposal {
            proposer: WalletAddress::zero(),
            offender: WalletAddress::zero(),
            offense: SlashOffense::DoubleSign,
            evidence: SlashEvidence::Unavailability {
                missed_blocks: 200,
                window_start: 0,
                window_end: 200,
            },
            evidence_hash: Blake3Hash::compute(b"evidence"),
            block_height: 101,
        };
        assert!(!validate_slash_evidence(&proposal));
    }
}
