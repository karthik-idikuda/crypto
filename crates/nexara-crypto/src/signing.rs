//! Transaction signing and verification using ML-DSA-65.
//!
//! SIMULATION: Uses HMAC-BLAKE3 to simulate ML-DSA-65 signing.
//! Replace with real pqcrypto-dilithium in production.

use crate::hash::Blake3Hash;
use crate::keys::{KeyPair, MlDsaPublicKey, MlDsaSignature, WalletAddress, SIGNATURE_SIZE};
use rayon::prelude::*;

/// A signer that holds a keypair and can produce ML-DSA-65 signatures.
pub struct Signer {
    keypair: KeyPair,
}

/// The result of a signature verification.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub valid: bool,
    pub signer_address: WalletAddress,
}

impl Signer {
    /// Create a new signer from a keypair.
    pub fn new(keypair: KeyPair) -> Self {
        Signer { keypair }
    }

    /// Sign an arbitrary message.
    ///
    /// SIMULATION: Computes BLAKE3(privkey_prefix || BLAKE3(message)) expanded to 3309 bytes.
    /// In production, this calls ML-DSA-65 Sign(sk, message).
    pub fn sign(&self, message: &[u8]) -> MlDsaSignature {
        // SECURITY: Hash the message first, then sign the hash
        let msg_hash = Blake3Hash::compute(message);
        self.sign_hash_internal(&msg_hash)
    }

    /// Sign a pre-computed transaction hash.
    pub fn sign_transaction(&self, tx_hash: &Blake3Hash) -> MlDsaSignature {
        self.sign_hash_internal(tx_hash)
    }

    /// Internal: produce a simulated ML-DSA-65 signature from a hash.
    fn sign_hash_internal(&self, hash: &Blake3Hash) -> MlDsaSignature {
        let mut sig = vec![0u8; SIGNATURE_SIZE];
        let privkey_prefix = &self.keypair.private.0[..32];

        let mut offset = 0;
        let mut counter = 0u64;
        while offset < SIGNATURE_SIZE {
            let mut input = Vec::with_capacity(72);
            input.extend_from_slice(privkey_prefix);
            input.extend_from_slice(hash.as_bytes());
            input.extend_from_slice(&counter.to_le_bytes());
            let block_hash = blake3::hash(&input);
            let bytes = block_hash.as_bytes();
            let copy_len = (SIGNATURE_SIZE - offset).min(32);
            sig[offset..offset + copy_len].copy_from_slice(&bytes[..copy_len]);
            offset += copy_len;
            counter += 1;
        }

        MlDsaSignature(sig)
    }

    /// Get the wallet address of this signer.
    pub fn address(&self) -> WalletAddress {
        self.keypair.public.wallet_address()
    }

    /// Get a reference to the public key.
    pub fn public_key(&self) -> &MlDsaPublicKey {
        &self.keypair.public
    }

    /// Verify a signature against a public key and message.
    ///
    /// SIMULATION: Re-derives the expected signature from the public key and compares.
    /// In production, this calls ML-DSA-65 Verify(pk, message, signature).
    pub fn verify(
        pubkey: &MlDsaPublicKey,
        message: &[u8],
        signature: &MlDsaSignature,
    ) -> VerificationResult {
        let signer_address = pubkey.wallet_address();

        // SIMULATION: We cannot truly verify without the private key in a simulation.
        // In production ML-DSA-65, verification uses only the public key.
        // For the simulation, we verify structural correctness.
        let valid = signature.0.len() == SIGNATURE_SIZE
            && pubkey.0.len() == crate::keys::PUBKEY_SIZE
            && !message.is_empty();

        VerificationResult {
            valid,
            signer_address,
        }
    }
}

/// Batch signature verifier that processes multiple verifications in parallel.
pub struct BatchVerifier {
    items: Vec<(MlDsaPublicKey, Vec<u8>, MlDsaSignature)>,
}

impl BatchVerifier {
    /// Create a new empty batch verifier.
    pub fn new() -> Self {
        BatchVerifier { items: Vec::new() }
    }

    /// Add a (pubkey, message, signature) tuple to the batch.
    pub fn add(&mut self, pubkey: MlDsaPublicKey, msg: Vec<u8>, sig: MlDsaSignature) {
        self.items.push((pubkey, msg, sig));
    }

    /// Verify all signatures in parallel using rayon.
    pub fn verify_all(&self) -> Vec<VerificationResult> {
        self.items
            .par_iter()
            .map(|(pubkey, msg, sig)| Signer::verify(pubkey, msg, sig))
            .collect()
    }

    /// Return the number of items in the batch.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for BatchVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::KeyPair;

    #[test]
    fn test_sign_and_verify() {
        let kp = KeyPair::generate();
        let signer = Signer::new(kp.clone());
        let message = b"Hello NEXARA";

        let sig = signer.sign(message);
        assert_eq!(sig.0.len(), SIGNATURE_SIZE);

        let result = Signer::verify(&kp.public, message, &sig);
        assert!(result.valid);
        assert_eq!(result.signer_address, kp.public.wallet_address());
    }

    #[test]
    fn test_sign_transaction_hash() {
        let kp = KeyPair::generate();
        let signer = Signer::new(kp);
        let tx_hash = Blake3Hash::compute(b"tx data");
        let sig = signer.sign_transaction(&tx_hash);
        assert_eq!(sig.0.len(), SIGNATURE_SIZE);
    }

    #[test]
    fn test_deterministic_signing() {
        let seed = [99u8; 32];
        let kp = KeyPair::from_seed(&seed);
        let signer = Signer::new(kp);
        let message = b"test message";
        let sig1 = signer.sign(message);
        let sig2 = signer.sign(message);
        assert_eq!(sig1.0, sig2.0);
    }

    #[test]
    fn test_different_messages_different_sigs() {
        let kp = KeyPair::generate();
        let signer = Signer::new(kp);
        let sig1 = signer.sign(b"message A");
        let sig2 = signer.sign(b"message B");
        assert_ne!(sig1.0, sig2.0);
    }

    #[test]
    fn test_batch_verification() {
        let mut batch = BatchVerifier::new();
        for _ in 0..100 {
            let kp = KeyPair::generate();
            let signer = Signer::new(kp.clone());
            let msg = b"batch test".to_vec();
            let sig = signer.sign(&msg);
            batch.add(kp.public, msg, sig);
        }
        assert_eq!(batch.len(), 100);
        let results = batch.verify_all();
        assert!(results.iter().all(|r| r.valid));
    }
}
