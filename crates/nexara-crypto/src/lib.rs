//! # NEXARA Cryptography
//!
//! Post-quantum cryptographic primitives for the NEXARA blockchain.
//!
//! ## Algorithms
//! - **ML-DSA-65** (NIST FIPS 204): Digital signatures — 1952-byte public keys, 3309-byte signatures
//! - **ML-KEM-1024** (NIST FIPS 203): Key encapsulation mechanism for encrypted communication
//! - **BLAKE3**: Primary hash function for blocks, transactions, and state
//! - **SHA3-256 / SHAKE256**: Address derivation and auxillary hashing
//!
//! ## SIMULATION NOTICE
//! The ML-DSA and ML-KEM implementations in this crate are **simulations** that match
//! the real algorithm's API and byte sizes but use HMAC/BLAKE3 internally.
//! Replace with the real `pqcrypto` crate for production deployment.

pub mod hash;
pub mod keys;
pub mod signing;
pub mod kem;
pub mod wallet;
pub mod mpc;

mod error;

pub use error::CryptoError;
pub use hash::{Blake3Hash, Sha3Hash, block_hash, transaction_hash, wallet_address_from_pubkey};
pub use keys::{KeyPair, MlDsaPublicKey, MlDsaPrivateKey, MlDsaSignature, WalletAddress};
pub use signing::{Signer, VerificationResult, BatchVerifier};
pub use kem::{KemKeyPair, KemPublicKey, KemPrivateKey, KemCiphertext, SharedSecret};
pub use wallet::{Wallet, WalletExport};
pub use mpc::MpcKeyShare;
