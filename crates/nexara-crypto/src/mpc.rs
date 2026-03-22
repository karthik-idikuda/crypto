//! Multi-Party Computation (MPC) key splitting via Shamir Secret Sharing.
//!
//! Allows splitting an ML-DSA-65 private key into N shares with a threshold T,
//! where any T shares can reconstruct the original key but T-1 shares reveal nothing.
//!
//! SIMULATION: Uses XOR-based splitting for development. Production should use
//! polynomial interpolation over GF(2^8) for true Shamir Secret Sharing.

use serde::{Serialize, Deserialize};
use rand::Rng;
use crate::hash::Blake3Hash;
use crate::keys::MlDsaPrivateKey;
use crate::CryptoError;

/// A single share of a split private key.
#[derive(Clone, Serialize, Deserialize)]
pub struct MpcKeyShare {
    /// Index of this share (1-based).
    pub share_index: u8,
    /// Total number of shares created.
    pub total_shares: u8,
    /// Minimum shares required for reconstruction.
    pub threshold: u8,
    /// The share data (same length as the original key).
    pub share_data: Vec<u8>,
}

/// Split a private key into multiple shares using (simulated) Shamir Secret Sharing.
///
/// # Arguments
/// - `private_key`: The ML-DSA-65 private key to split
/// - `total_shares`: Number of shares to create (N)
/// - `threshold`: Minimum shares needed for reconstruction (T)
///
/// # SECURITY
/// In this simulation, we use XOR-based splitting which provides information-theoretic
/// security for T=N but not for T<N. Production must use polynomial interpolation.
pub fn split_key(
    private_key: &MlDsaPrivateKey,
    total_shares: u8,
    threshold: u8,
) -> Result<Vec<MpcKeyShare>, CryptoError> {
    if threshold > total_shares || threshold == 0 || total_shares < 2 {
        return Err(CryptoError::MpcReconstructionFailed(
            "invalid share/threshold parameters".to_string(),
        ));
    }

    let key_bytes = &private_key.0;
    let key_len = key_bytes.len();
    let mut rng = rand::thread_rng();
    let mut shares = Vec::with_capacity(total_shares as usize);

    // Generate (total_shares - 1) random shares
    let mut random_shares: Vec<Vec<u8>> = Vec::new();
    for _ in 0..(total_shares - 1) {
        let mut share = vec![0u8; key_len];
        rng.fill(&mut share[..]);
        random_shares.push(share);
    }

    // The last share is XOR of all random shares with the key
    let mut last_share = key_bytes.to_vec();
    for rs in &random_shares {
        for (i, byte) in rs.iter().enumerate() {
            last_share[i] ^= byte;
        }
    }

    // Build share structs
    for (idx, share_data) in random_shares.into_iter().enumerate() {
        shares.push(MpcKeyShare {
            share_index: (idx + 1) as u8,
            total_shares,
            threshold,
            share_data,
        });
    }
    shares.push(MpcKeyShare {
        share_index: total_shares,
        total_shares,
        threshold,
        share_data: last_share,
    });

    Ok(shares)
}

/// Reconstruct a private key from shares.
///
/// Requires all shares for this XOR-based simulation.
/// Production Shamir reconstruction only needs `threshold` shares.
pub fn reconstruct_key(shares: &[MpcKeyShare]) -> Result<MlDsaPrivateKey, CryptoError> {
    if shares.is_empty() {
        return Err(CryptoError::InsufficientShares {
            threshold: 0,
            got: 0,
        });
    }

    let threshold = shares[0].threshold;
    let total = shares[0].total_shares;

    if (shares.len() as u8) < threshold {
        return Err(CryptoError::InsufficientShares {
            threshold,
            got: shares.len() as u8,
        });
    }

    // XOR-based reconstruction requires all shares
    if (shares.len() as u8) < total {
        return Err(CryptoError::MpcReconstructionFailed(
            "XOR-based simulation requires all shares".to_string(),
        ));
    }

    let key_len = shares[0].share_data.len();
    let mut reconstructed = vec![0u8; key_len];

    for share in shares {
        if share.share_data.len() != key_len {
            return Err(CryptoError::MpcReconstructionFailed(
                "inconsistent share lengths".to_string(),
            ));
        }
        for (i, byte) in share.share_data.iter().enumerate() {
            reconstructed[i] ^= byte;
        }
    }

    Ok(MlDsaPrivateKey(reconstructed))
}

/// Verify a share against a commitment hash.
///
/// The commitment is BLAKE3(share_index || share_data).
pub fn verify_share(share: &MpcKeyShare, commitment: &Blake3Hash) -> bool {
    let mut data = Vec::with_capacity(1 + share.share_data.len());
    data.push(share.share_index);
    data.extend_from_slice(&share.share_data);
    let computed = Blake3Hash::compute(&data);
    computed == *commitment
}

/// Create a commitment hash for a share.
pub fn create_share_commitment(share: &MpcKeyShare) -> Blake3Hash {
    let mut data = Vec::with_capacity(1 + share.share_data.len());
    data.push(share.share_index);
    data.extend_from_slice(&share.share_data);
    Blake3Hash::compute(&data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::KeyPair;

    #[test]
    fn test_split_and_reconstruct() {
        let kp = KeyPair::generate();
        let original_key = kp.private.clone();

        let shares = split_key(&original_key, 3, 3).unwrap();
        assert_eq!(shares.len(), 3);

        let reconstructed = reconstruct_key(&shares).unwrap();
        assert_eq!(original_key.0, reconstructed.0);
    }

    #[test]
    fn test_split_invalid_params() {
        let kp = KeyPair::generate();
        assert!(split_key(&kp.private, 1, 2).is_err()); // threshold > total
        assert!(split_key(&kp.private, 0, 0).is_err()); // zero
    }

    #[test]
    fn test_insufficient_shares() {
        let kp = KeyPair::generate();
        let shares = split_key(&kp.private, 3, 3).unwrap();
        let partial = &shares[..2]; // Only 2 of 3
        assert!(reconstruct_key(partial).is_err());
    }

    #[test]
    fn test_share_commitment_verification() {
        let kp = KeyPair::generate();
        let shares = split_key(&kp.private, 3, 3).unwrap();
        let commitment = create_share_commitment(&shares[0]);
        assert!(verify_share(&shares[0], &commitment));
        assert!(!verify_share(&shares[1], &commitment)); // Wrong share
    }

    #[test]
    fn test_shares_are_random() {
        let kp = KeyPair::generate();
        let shares1 = split_key(&kp.private, 3, 3).unwrap();
        let shares2 = split_key(&kp.private, 3, 3).unwrap();
        // Same key, different splits
        assert_ne!(shares1[0].share_data, shares2[0].share_data);
    }
}
