//! BLAKE3 and SHA3 hashing primitives for the NEXARA blockchain.
//!
//! BLAKE3 is used as the primary hash function for blocks, transactions, and state roots.
//! SHA3-256 and SHAKE256 are used for address derivation from public keys.

use sha3::{Digest, Sha3_256};
use serde::{Serialize, Deserialize};
use crate::CryptoError;

/// A 32-byte BLAKE3 hash.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Blake3Hash(pub [u8; 32]);

/// A 32-byte SHA3-256 hash.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Sha3Hash(pub [u8; 32]);

impl Blake3Hash {
    /// Compute the BLAKE3 hash of arbitrary data.
    pub fn compute(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Blake3Hash(*hash.as_bytes())
    }

    /// Return the hash as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a BLAKE3 hash from a hex string.
    pub fn from_hex(s: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(s).map_err(|e| CryptoError::InvalidHex(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                expected: 32,
                got: bytes.len(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Blake3Hash(arr))
    }

    /// Verify that this hash matches the given data.
    pub fn verify(&self, data: &[u8]) -> bool {
        let computed = Self::compute(data);
        self.0 == computed.0
    }

    /// Return the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// A zero hash (used as placeholder / genesis parent).
    pub fn zero() -> Self {
        Blake3Hash([0u8; 32])
    }
}

impl std::fmt::Debug for Blake3Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blake3({})", &self.to_hex()[..16])
    }
}

impl std::fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for Blake3Hash {
    fn default() -> Self {
        Self::zero()
    }
}

impl Sha3Hash {
    /// Compute the SHA3-256 hash of arbitrary data.
    pub fn compute(data: &[u8]) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        Sha3Hash(arr)
    }

    /// Return the hash as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl std::fmt::Debug for Sha3Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sha3({})", &self.to_hex()[..16])
    }
}

/// Compute the BLAKE3 hash of a serialized block header.
pub fn block_hash(header_bytes: &[u8]) -> Blake3Hash {
    Blake3Hash::compute(header_bytes)
}

/// Compute the BLAKE3 hash of a serialized transaction.
pub fn transaction_hash(tx_bytes: &[u8]) -> Blake3Hash {
    Blake3Hash::compute(tx_bytes)
}

/// Derive a 32-byte wallet address from a public key using SHA3-256.
///
/// This compresses an arbitrary-length public key (e.g. 1952 bytes for ML-DSA-65)
/// into a fixed 32-byte address suitable for use on-chain.
pub fn wallet_address_from_pubkey(pubkey_bytes: &[u8]) -> [u8; 32] {
    let hash = Sha3Hash::compute(pubkey_bytes);
    hash.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_compute_and_verify() {
        let data = b"NEXARA blockchain";
        let hash = Blake3Hash::compute(data);
        assert!(hash.verify(data));
        assert!(!hash.verify(b"tampered data"));
    }

    #[test]
    fn test_blake3_hex_roundtrip() {
        let data = b"test data";
        let hash = Blake3Hash::compute(data);
        let hex_str = hash.to_hex();
        let recovered = Blake3Hash::from_hex(&hex_str).unwrap();
        assert_eq!(hash, recovered);
    }

    #[test]
    fn test_sha3_compute() {
        let data = b"NEXARA";
        let hash = Sha3Hash::compute(data);
        assert_eq!(hash.0.len(), 32);
        // Deterministic
        assert_eq!(hash, Sha3Hash::compute(data));
    }

    #[test]
    fn test_wallet_address_from_pubkey() {
        let pubkey = vec![42u8; 1952]; // Simulated ML-DSA pubkey
        let addr = wallet_address_from_pubkey(&pubkey);
        assert_eq!(addr.len(), 32);
        // Deterministic
        assert_eq!(addr, wallet_address_from_pubkey(&pubkey));
    }

    #[test]
    fn test_block_and_tx_hash() {
        let header = b"block header bytes";
        let tx = b"transaction bytes";
        let bh = block_hash(header);
        let th = transaction_hash(tx);
        assert_ne!(bh, th);
    }
}
