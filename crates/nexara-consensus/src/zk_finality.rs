//! ZK-SNARK Finality Layer.
//!
//! Provides zero-knowledge proof-based finality confirmation.
//! This is a simulation of ZK proof generation and verification
//! using BLAKE3 as a placeholder for actual ZK circuits.

use nexara_crypto::Blake3Hash;
use serde::{Serialize, Deserialize};

/// A simulated ZK-SNARK finality proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkFinalityProof {
    pub epoch: u64,
    pub block_range_start: u64,
    pub block_range_end: u64,
    pub state_root: Blake3Hash,
    pub proof_data: Vec<u8>,
    pub verification_key_hash: Blake3Hash,
}

/// Result of ZK proof verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkVerifyResult {
    Valid,
    Invalid(String),
}

/// Generate a simulated ZK finality proof for a range of blocks.
///
/// In production, this would use a real ZK-SNARK library (e.g., bellman, arkworks).
/// The simulation creates a deterministic proof from the inputs.
pub fn generate_finality_proof(
    epoch: u64,
    block_range_start: u64,
    block_range_end: u64,
    state_root: &Blake3Hash,
    block_hashes: &[Blake3Hash],
) -> ZkFinalityProof {
    // Build the proof input
    let mut proof_input = Vec::new();
    proof_input.extend_from_slice(&epoch.to_le_bytes());
    proof_input.extend_from_slice(&block_range_start.to_le_bytes());
    proof_input.extend_from_slice(&block_range_end.to_le_bytes());
    proof_input.extend_from_slice(state_root.as_bytes());

    for bh in block_hashes {
        proof_input.extend_from_slice(bh.as_bytes());
    }

    // Simulated proof: BLAKE3 hash of all inputs
    let proof_hash = Blake3Hash::compute(&proof_input);
    let proof_data = proof_hash.as_bytes().to_vec();

    // Verification key is derived from the epoch
    let vk_input = format!("nexara-zk-vk-epoch-{}", epoch);
    let verification_key_hash = Blake3Hash::compute(vk_input.as_bytes());

    ZkFinalityProof {
        epoch,
        block_range_start,
        block_range_end,
        state_root: *state_root,
        proof_data,
        verification_key_hash,
    }
}

/// Verify a ZK finality proof.
///
/// Simulated verification: recompute the proof from expected inputs and compare.
pub fn verify_finality_proof(
    proof: &ZkFinalityProof,
    expected_state_root: &Blake3Hash,
    block_hashes: &[Blake3Hash],
) -> ZkVerifyResult {
    // Check state root matches
    if proof.state_root != *expected_state_root {
        return ZkVerifyResult::Invalid("State root mismatch".into());
    }

    // Check block range is valid
    if proof.block_range_start > proof.block_range_end {
        return ZkVerifyResult::Invalid("Invalid block range".into());
    }

    // Re-derive the proof and compare
    let expected_proof = generate_finality_proof(
        proof.epoch,
        proof.block_range_start,
        proof.block_range_end,
        expected_state_root,
        block_hashes,
    );

    if proof.proof_data != expected_proof.proof_data {
        return ZkVerifyResult::Invalid("Proof data mismatch".into());
    }

    ZkVerifyResult::Valid
}

/// Batch verify multiple ZK finality proofs.
pub fn batch_verify_proofs(
    proofs: &[(ZkFinalityProof, Blake3Hash, Vec<Blake3Hash>)],
) -> Vec<ZkVerifyResult> {
    proofs.iter()
        .map(|(proof, state_root, hashes)| verify_finality_proof(proof, state_root, hashes))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_proof() {
        let state_root = Blake3Hash::compute(b"state");
        let block_hashes = vec![
            Blake3Hash::compute(b"block-1"),
            Blake3Hash::compute(b"block-2"),
        ];
        let proof = generate_finality_proof(1, 0, 1, &state_root, &block_hashes);
        assert_eq!(proof.epoch, 1);
        assert_eq!(proof.block_range_start, 0);
        assert_eq!(proof.block_range_end, 1);
        assert!(!proof.proof_data.is_empty());
    }

    #[test]
    fn test_verify_valid_proof() {
        let state_root = Blake3Hash::compute(b"state");
        let block_hashes = vec![
            Blake3Hash::compute(b"block-1"),
            Blake3Hash::compute(b"block-2"),
        ];
        let proof = generate_finality_proof(1, 0, 1, &state_root, &block_hashes);
        let result = verify_finality_proof(&proof, &state_root, &block_hashes);
        assert_eq!(result, ZkVerifyResult::Valid);
    }

    #[test]
    fn test_verify_wrong_state_root() {
        let state_root = Blake3Hash::compute(b"state");
        let wrong_root = Blake3Hash::compute(b"wrong");
        let block_hashes = vec![Blake3Hash::compute(b"block-1")];
        let proof = generate_finality_proof(1, 0, 0, &state_root, &block_hashes);
        let result = verify_finality_proof(&proof, &wrong_root, &block_hashes);
        assert!(matches!(result, ZkVerifyResult::Invalid(_)));
    }

    #[test]
    fn test_batch_verify() {
        let state_root = Blake3Hash::compute(b"state");
        let block_hashes = vec![Blake3Hash::compute(b"block-1")];
        let proof = generate_finality_proof(1, 0, 0, &state_root, &block_hashes);
        let results = batch_verify_proofs(&[(proof, state_root, block_hashes)]);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], ZkVerifyResult::Valid);
    }

    #[test]
    fn test_proof_deterministic() {
        let state_root = Blake3Hash::compute(b"state");
        let hashes = vec![Blake3Hash::compute(b"b1")];
        let p1 = generate_finality_proof(1, 0, 0, &state_root, &hashes);
        let p2 = generate_finality_proof(1, 0, 0, &state_root, &hashes);
        assert_eq!(p1.proof_data, p2.proof_data);
    }
}
