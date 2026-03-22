//! ML-DSA-65 key generation for the NEXARA blockchain.
//!
//! SIMULATION: This module simulates ML-DSA-65 (NIST FIPS 204) key structures
//! with the correct byte sizes. Replace with real `pqcrypto-dilithium` in production.
//!
//! ## Key Sizes (NIST FIPS 204 ML-DSA-65 parameter set)
//! - Public key: 1952 bytes
//! - Private key: 4032 bytes
//! - Signature: 3309 bytes

use serde::{Serialize, Deserialize};
use rand::Rng;
use crate::hash::wallet_address_from_pubkey;
use crate::CryptoError;

/// ML-DSA-65 public key size in bytes (NIST FIPS 204).
pub const PUBKEY_SIZE: usize = 1952;
/// ML-DSA-65 private key size in bytes (NIST FIPS 204).
pub const PRIVKEY_SIZE: usize = 4032;
/// ML-DSA-65 signature size in bytes (NIST FIPS 204).
pub const SIGNATURE_SIZE: usize = 3309;

/// An ML-DSA-65 public key (1952 bytes).
#[derive(Clone, Serialize, Deserialize)]
pub struct MlDsaPublicKey(pub Vec<u8>);

/// An ML-DSA-65 private key (4032 bytes).
///
/// SECURITY: This key material is sensitive. Zeroized on drop in production.
#[derive(Clone, Serialize, Deserialize)]
pub struct MlDsaPrivateKey(pub Vec<u8>);

/// An ML-DSA-65 digital signature (3309 bytes).
#[derive(Clone, Serialize, Deserialize)]
pub struct MlDsaSignature(pub Vec<u8>);

/// A complete ML-DSA-65 keypair.
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyPair {
    pub public: MlDsaPublicKey,
    pub private: MlDsaPrivateKey,
}

/// A 32-byte wallet address derived from a public key via SHA3.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletAddress(pub [u8; 32]);

impl KeyPair {
    /// Generate a new random ML-DSA-65 keypair.
    ///
    /// SIMULATION: Replace with real pqcrypto crate in production.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let mut pubkey = vec![0u8; PUBKEY_SIZE];
        let mut privkey = vec![0u8; PRIVKEY_SIZE];
        rng.fill(&mut pubkey[..]);
        rng.fill(&mut privkey[..]);
        KeyPair {
            public: MlDsaPublicKey(pubkey),
            private: MlDsaPrivateKey(privkey),
        }
    }

    /// Generate a deterministic keypair from a 32-byte seed.
    ///
    /// SIMULATION: Uses BLAKE3 expansion of seed. In production,
    /// use ML-DSA-65 deterministic key generation.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let mut pubkey = vec![0u8; PUBKEY_SIZE];
        let mut privkey = vec![0u8; PRIVKEY_SIZE];

        // Expand seed into public key bytes using BLAKE3 in keyed mode
        let mut offset = 0;
        let mut counter = 0u64;
        while offset < PUBKEY_SIZE {
            let mut input = Vec::with_capacity(40);
            input.extend_from_slice(seed);
            input.extend_from_slice(&counter.to_le_bytes());
            let hash = blake3::hash(&input);
            let bytes = hash.as_bytes();
            let copy_len = (PUBKEY_SIZE - offset).min(32);
            pubkey[offset..offset + copy_len].copy_from_slice(&bytes[..copy_len]);
            offset += copy_len;
            counter += 1;
        }

        // Expand seed into private key bytes (different domain)
        offset = 0;
        while offset < PRIVKEY_SIZE {
            let mut input = Vec::with_capacity(41);
            input.push(0xFF); // Domain separator
            input.extend_from_slice(seed);
            input.extend_from_slice(&counter.to_le_bytes());
            let hash = blake3::hash(&input);
            let bytes = hash.as_bytes();
            let copy_len = (PRIVKEY_SIZE - offset).min(32);
            privkey[offset..offset + copy_len].copy_from_slice(&bytes[..copy_len]);
            offset += copy_len;
            counter += 1;
        }

        KeyPair {
            public: MlDsaPublicKey(pubkey),
            private: MlDsaPrivateKey(privkey),
        }
    }

    /// Get a reference to the public key.
    pub fn public_key(&self) -> &MlDsaPublicKey {
        &self.public
    }

    /// Get a reference to the private key.
    pub fn private_key(&self) -> &MlDsaPrivateKey {
        &self.private
    }
}

impl MlDsaPublicKey {
    /// Return the raw public key bytes.
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Return the public key as a hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Derive a wallet address from this public key.
    pub fn wallet_address(&self) -> WalletAddress {
        WalletAddress(wallet_address_from_pubkey(&self.0))
    }

    /// Create from raw bytes with validation.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != PUBKEY_SIZE {
            return Err(CryptoError::InvalidKeyLength {
                expected: PUBKEY_SIZE,
                got: bytes.len(),
            });
        }
        Ok(MlDsaPublicKey(bytes.to_vec()))
    }
}

impl std::fmt::Debug for MlDsaPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MlDsaPK({}..)", &self.to_hex()[..16])
    }
}

impl MlDsaPrivateKey {
    /// Return the raw private key bytes.
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Create from raw bytes with validation.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != PRIVKEY_SIZE {
            return Err(CryptoError::InvalidKeyLength {
                expected: PRIVKEY_SIZE,
                got: bytes.len(),
            });
        }
        Ok(MlDsaPrivateKey(bytes.to_vec()))
    }
}

impl MlDsaSignature {
    /// Return the raw signature bytes.
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Create from raw bytes with validation.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != SIGNATURE_SIZE {
            return Err(CryptoError::InvalidSignatureLength {
                expected: SIGNATURE_SIZE,
                got: bytes.len(),
            });
        }
        Ok(MlDsaSignature(bytes.to_vec()))
    }
}

impl std::fmt::Debug for MlDsaSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MlDsaSig({}..)", hex::encode(&self.0[..8]))
    }
}

impl WalletAddress {
    /// Return the address as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse a wallet address from a hex string.
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
        Ok(WalletAddress(arr))
    }

    /// Encode the address in bech32-style format with "nxr1" prefix.
    ///
    /// Note: This is a simplified bech32 encoding for development.
    /// Production should use proper bech32m (BIP-350).
    pub fn to_bech32(&self) -> String {
        // Simple base32 encoding with nxr1 prefix
        let encoded = base32_encode(&self.0);
        format!("nxr1{}", encoded)
    }

    /// The zero address (used as burn address).
    pub fn zero() -> Self {
        WalletAddress([0u8; 32])
    }

    /// Return the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for WalletAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Addr({})", &self.to_hex()[..16])
    }
}

impl std::fmt::Display for WalletAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_bech32())
    }
}

impl Default for WalletAddress {
    fn default() -> Self {
        Self::zero()
    }
}

/// Simple base32 encoding (RFC 4648 lowercase).
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut result = String::new();
    let mut bits: u32 = 0;
    let mut num_bits: u32 = 0;

    for &byte in data {
        bits = (bits << 8) | byte as u32;
        num_bits += 8;
        while num_bits >= 5 {
            num_bits -= 5;
            let idx = ((bits >> num_bits) & 0x1F) as usize;
            result.push(ALPHABET[idx] as char);
        }
    }
    if num_bits > 0 {
        let idx = ((bits << (5 - num_bits)) & 0x1F) as usize;
        result.push(ALPHABET[idx] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        assert_eq!(kp.public.0.len(), PUBKEY_SIZE);
        assert_eq!(kp.private.0.len(), PRIVKEY_SIZE);
    }

    #[test]
    fn test_keypair_from_seed_deterministic() {
        let seed = [42u8; 32];
        let kp1 = KeyPair::from_seed(&seed);
        let kp2 = KeyPair::from_seed(&seed);
        assert_eq!(kp1.public.0, kp2.public.0);
        assert_eq!(kp1.private.0, kp2.private.0);
    }

    #[test]
    fn test_different_seeds_different_keys() {
        let kp1 = KeyPair::from_seed(&[1u8; 32]);
        let kp2 = KeyPair::from_seed(&[2u8; 32]);
        assert_ne!(kp1.public.0, kp2.public.0);
    }

    #[test]
    fn test_wallet_address_derivation() {
        let kp = KeyPair::generate();
        let addr1 = kp.public.wallet_address();
        let addr2 = kp.public.wallet_address();
        assert_eq!(addr1, addr2); // Same key → same address
    }

    #[test]
    fn test_wallet_address_hex_roundtrip() {
        let kp = KeyPair::generate();
        let addr = kp.public.wallet_address();
        let hex_str = addr.to_hex();
        let recovered = WalletAddress::from_hex(&hex_str).unwrap();
        assert_eq!(addr, recovered);
    }

    #[test]
    fn test_bech32_format() {
        let kp = KeyPair::generate();
        let addr = kp.public.wallet_address();
        let bech32 = addr.to_bech32();
        assert!(bech32.starts_with("nxr1"));
    }

    #[test]
    fn test_unique_keypairs() {
        let keypairs: Vec<KeyPair> = (0..100).map(|_| KeyPair::generate()).collect();
        for i in 0..keypairs.len() {
            for j in (i + 1)..keypairs.len() {
                assert_ne!(keypairs[i].public.0, keypairs[j].public.0);
            }
        }
    }
}
