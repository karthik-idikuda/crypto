//! HybridSync consensus engine.
//!
//! Orchestrates block proposals, attestations, quorum checks, and finalization
//! using DPoQS + AIBFT + ZK-Finality.

use crate::error::ConsensusError;
use crate::validator::ValidatorSet;
use crate::committee::Committee;
use nexara_crypto::{Blake3Hash, WalletAddress, MlDsaSignature};
use nexara_core::block::Block;
use serde::{Serialize, Deserialize};

/// An attestation (vote) from a committee member for a proposed block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub validator: WalletAddress,
    pub block_hash: Blake3Hash,
    pub epoch: u64,
    pub round: u64,
    pub signature: MlDsaSignature,
    pub timestamp: u64,
}

/// Result of a consensus round.
#[derive(Debug, Clone)]
pub enum ConsensusResult {
    /// Block finalized.
    Finalized(Block),
    /// Waiting for more attestations.
    Pending(QuorumStatus),
    /// Round failed – need new proposal.
    RoundFailed(String),
}

/// Quorum progress status.
#[derive(Debug, Clone)]
pub struct QuorumStatus {
    pub block_hash: Blake3Hash,
    pub attestation_count: usize,
    pub attestation_stake: u128,
    pub required_stake: u128,
    pub threshold_pct: u8,
}

impl QuorumStatus {
    /// Check if the quorum threshold is met.
    pub fn is_met(&self) -> bool {
        self.attestation_stake >= self.required_stake
    }
}

/// The HybridSync consensus engine.
pub struct HybridSyncEngine {
    pub epoch: u64,
    pub round: u64,
    pub validator_set: ValidatorSet,
    pub current_committee: Option<Committee>,
    pub pending_attestations: Vec<Attestation>,
    pub quorum_threshold_pct: u8,
    pub proposed_block: Option<Block>,
}

impl HybridSyncEngine {
    /// Create a new HybridSync engine.
    pub fn new(validator_set: ValidatorSet, quorum_threshold_pct: u8) -> Self {
        HybridSyncEngine {
            epoch: 0,
            round: 0,
            validator_set,
            current_committee: None,
            pending_attestations: Vec::new(),
            quorum_threshold_pct,
            proposed_block: None,
        }
    }

    /// Propose a new block for the current round.
    pub fn propose_block(&mut self, block: Block) -> Result<Blake3Hash, ConsensusError> {
        if self.proposed_block.is_some() {
            return Err(ConsensusError::InvalidProposal("Block already proposed for this round".into()));
        }
        let hash = block.hash();
        self.proposed_block = Some(block);
        self.pending_attestations.clear();
        Ok(hash)
    }

    /// Submit an attestation for the current proposed block.
    pub fn submit_attestation(&mut self, attestation: Attestation) -> Result<(), ConsensusError> {
        let proposed = self.proposed_block.as_ref()
            .ok_or_else(|| ConsensusError::InvalidProposal("No block proposed".into()))?;

        let block_hash = proposed.hash();
        if attestation.block_hash != block_hash {
            return Err(ConsensusError::InvalidAttestation("Block hash mismatch".into()));
        }

        // Check for duplicate attestations
        if self.pending_attestations.iter().any(|a| a.validator == attestation.validator) {
            return Err(ConsensusError::InvalidAttestation("Duplicate attestation".into()));
        }

        self.pending_attestations.push(attestation);
        Ok(())
    }

    /// Check whether quorum has been reached.
    pub fn check_quorum(&self) -> QuorumStatus {
        let total_active_stake = self.validator_set.total_active_stake();
        let required_stake = total_active_stake
            .saturating_mul(self.quorum_threshold_pct as u128)
            / 100;

        let attestation_stake: u128 = self.pending_attestations.iter()
            .filter_map(|a| {
                self.validator_set.get_validator(&a.validator)
                    .map(|v| v.total_stake)
            })
            .sum();

        let block_hash = self.proposed_block.as_ref()
            .map(|b| b.hash())
            .unwrap_or_else(|| Blake3Hash::compute(b"none"));

        QuorumStatus {
            block_hash,
            attestation_count: self.pending_attestations.len(),
            attestation_stake,
            required_stake,
            threshold_pct: self.quorum_threshold_pct,
        }
    }

    /// Attempt to finalize the current block if quorum is met.
    pub fn finalize_block(&mut self) -> ConsensusResult {
        let status = self.check_quorum();
        if !status.is_met() {
            return ConsensusResult::Pending(status);
        }

        match self.proposed_block.take() {
            Some(block) => {
                self.pending_attestations.clear();
                self.round += 1;
                ConsensusResult::Finalized(block)
            }
            None => ConsensusResult::RoundFailed("No block to finalize".into()),
        }
    }

    /// Advance to a new epoch.
    pub fn advance_epoch(&mut self) {
        self.epoch += 1;
        self.round = 0;
        self.pending_attestations.clear();
        self.proposed_block = None;
        self.current_committee = None;
    }

    /// Set the committee for the current round.
    pub fn set_committee(&mut self, committee: Committee) {
        self.current_committee = Some(committee);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::Validator;
    use nexara_crypto::keys::KeyPair;
    use nexara_core::block::{Block, BlockHeader};

    fn setup_engine() -> (HybridSyncEngine, Vec<KeyPair>) {
        let mut vs = ValidatorSet::new();
        let mut keys = Vec::new();
        for _ in 0..4 {
            let kp = KeyPair::generate();
            let v = Validator::new(kp.public.wallet_address(), kp.public.clone(), 1000);
            vs.add_validator(v);
            keys.push(kp);
        }
        (HybridSyncEngine::new(vs, 67), keys) // 67% quorum
    }

    fn make_block() -> Block {
        let header = BlockHeader {
            version: 1,
            shard_id: 0,
            height: 1,
            timestamp: 1000,
            parent_hash: Blake3Hash::compute(b"prev"),
            state_root: Blake3Hash::compute(b"state"),
            tx_root: Blake3Hash::compute(b"txroot"),
            validator_set_hash: Blake3Hash::compute(b"valset"),
            proposer_address: WalletAddress::zero(),
            zk_finality_proof: Vec::new(),
            attestation_bitfield: Vec::new(),
        };
        Block::new(header, Vec::new())
    }

    #[test]
    fn test_propose_block() {
        let (mut engine, _) = setup_engine();
        let block = make_block();
        let result = engine.propose_block(block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_double_propose_fails() {
        let (mut engine, _) = setup_engine();
        let block = make_block();
        engine.propose_block(block.clone()).unwrap();
        assert!(engine.propose_block(block).is_err());
    }

    #[test]
    fn test_quorum_not_met() {
        let (mut engine, _) = setup_engine();
        let block = make_block();
        engine.propose_block(block).unwrap();
        let status = engine.check_quorum();
        assert!(!status.is_met());
    }

    #[test]
    fn test_submit_attestation() {
        let (mut engine, keys) = setup_engine();
        let block = make_block();
        let block_hash = block.hash();
        engine.propose_block(block).unwrap();

        let att = Attestation {
            validator: keys[0].public.wallet_address(),
            block_hash,
            epoch: 0,
            round: 0,
            signature: nexara_crypto::MlDsaSignature(vec![0u8; nexara_crypto::keys::SIGNATURE_SIZE]),
            timestamp: 1000,
        };
        assert!(engine.submit_attestation(att).is_ok());
    }

    #[test]
    fn test_advance_epoch() {
        let (mut engine, _) = setup_engine();
        engine.advance_epoch();
        assert_eq!(engine.epoch, 1);
        assert_eq!(engine.round, 0);
    }
}
