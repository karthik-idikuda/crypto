//! Wallet management for NEXARA accounts.
//!
//! A wallet bundles an ML-DSA-65 signing keypair, an ML-KEM-1024 key exchange keypair,
//! and the derived on-chain address.

use serde::{Serialize, Deserialize};
use sha3::{Digest, Sha3_256};
use crate::keys::{KeyPair, WalletAddress, MlDsaSignature};
use crate::kem::KemKeyPair;
use crate::signing::Signer;

/// A complete NEXARA wallet with signing and encryption keys.
pub struct Wallet {
    pub address: WalletAddress,
    pub keypair: KeyPair,
    pub kem_keypair: KemKeyPair,
    pub nonce: u64,
}

/// Exported wallet information (safe to share publicly).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletExport {
    pub address: String,
    pub public_key_hex: String,
    pub nonce: u64,
}

impl Wallet {
    /// Create a new wallet with random keys.
    pub fn create_new() -> Self {
        let keypair = KeyPair::generate();
        let kem_keypair = KemKeyPair::generate();
        let address = keypair.public.wallet_address();
        Wallet {
            address,
            keypair,
            kem_keypair,
            nonce: 0,
        }
    }

    /// Create a wallet from a seed phrase (deterministic).
    ///
    /// The phrase is hashed with SHA3-256 to produce a 32-byte seed,
    /// which is then used for deterministic key generation.
    pub fn from_seed_phrase(phrase: &str) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(phrase.as_bytes());
        let result = hasher.finalize();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&result);

        let keypair = KeyPair::from_seed(&seed);
        let kem_keypair = KemKeyPair::generate(); // KEM doesn't need determinism for wallet
        let address = keypair.public.wallet_address();

        Wallet {
            address,
            keypair,
            kem_keypair,
            nonce: 0,
        }
    }

    /// Get the wallet address.
    pub fn address(&self) -> &WalletAddress {
        &self.address
    }

    /// Sign transaction bytes.
    pub fn sign_transaction(&self, tx: &[u8]) -> MlDsaSignature {
        let signer = Signer::new(self.keypair.clone());
        signer.sign(tx)
    }

    /// Get and increment the nonce (for transaction ordering).
    pub fn next_nonce(&mut self) -> u64 {
        let n = self.nonce;
        self.nonce += 1;
        n
    }

    /// Export public wallet information.
    pub fn export_keys(&self) -> WalletExport {
        WalletExport {
            address: self.address.to_bech32(),
            public_key_hex: self.keypair.public.to_hex(),
            nonce: self.nonce,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_new_wallet() {
        let wallet = Wallet::create_new();
        assert_eq!(wallet.nonce, 0);
        assert_ne!(wallet.address, WalletAddress::zero());
    }

    #[test]
    fn test_from_seed_phrase_deterministic() {
        let w1 = Wallet::from_seed_phrase("test phrase for NEXARA wallet");
        let w2 = Wallet::from_seed_phrase("test phrase for NEXARA wallet");
        assert_eq!(w1.address, w2.address);
        assert_eq!(w1.keypair.public.0, w2.keypair.public.0);
    }

    #[test]
    fn test_different_phrases_different_wallets() {
        let w1 = Wallet::from_seed_phrase("phrase one");
        let w2 = Wallet::from_seed_phrase("phrase two");
        assert_ne!(w1.address, w2.address);
    }

    #[test]
    fn test_nonce_increment() {
        let mut wallet = Wallet::create_new();
        assert_eq!(wallet.next_nonce(), 0);
        assert_eq!(wallet.next_nonce(), 1);
        assert_eq!(wallet.next_nonce(), 2);
    }

    #[test]
    fn test_sign_transaction() {
        let wallet = Wallet::create_new();
        let sig = wallet.sign_transaction(b"tx data");
        assert_eq!(sig.0.len(), crate::keys::SIGNATURE_SIZE);
    }

    #[test]
    fn test_export_keys() {
        let wallet = Wallet::create_new();
        let export = wallet.export_keys();
        assert!(export.address.starts_with("nxr1"));
        assert!(!export.public_key_hex.is_empty());
    }
}
