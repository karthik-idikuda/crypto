//! Bitcoin bridge adapter (HTLC-based).

use serde::{Deserialize, Serialize};
use crate::error::BridgeError;

/// Bitcoin bridge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinBridge {
    pub network: BitcoinNetwork,
    pub confirmations_required: u32,
    pub htlc_timeout_blocks: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitcoinNetwork {
    Mainnet,
    Testnet,
    Regtest,
}

/// Bitcoin HTLC lock event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcLockEvent {
    pub txid: String,
    pub vout: u32,
    pub amount_sats: u64,
    pub hash_lock: [u8; 32],
    pub timeout_block: u64,
    pub sender_pubkey: String,
    pub recipient: String,
}

impl BitcoinBridge {
    pub fn new(network: BitcoinNetwork) -> Self {
        BitcoinBridge {
            network,
            confirmations_required: 6,
            htlc_timeout_blocks: 144, // ~24 hours
            enabled: true,
        }
    }

    pub fn validate_lock(&self, event: &BtcLockEvent) -> Result<bool, BridgeError> {
        if !self.enabled {
            return Err(BridgeError::BridgePaused("Bitcoin".into()));
        }
        if event.amount_sats == 0 {
            return Err(BridgeError::InvalidMessage("Zero amount".into()));
        }
        if event.txid.is_empty() {
            return Err(BridgeError::InvalidProof("Empty txid".into()));
        }
        // Verify hash lock is non-zero
        Ok(event.hash_lock != [0u8; 32])
    }

    /// Generate a hash lock for HTLC.
    pub fn generate_hash_lock(secret: &[u8]) -> [u8; 32] {
        *blake3::hash(secret).as_bytes()
    }

    /// Verify a preimage against hash lock.
    pub fn verify_preimage(preimage: &[u8], hash_lock: &[u8; 32]) -> bool {
        let computed = blake3::hash(preimage);
        computed.as_bytes() == hash_lock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btc_validate_lock() {
        let bridge = BitcoinBridge::new(BitcoinNetwork::Mainnet);
        let lock = BtcLockEvent {
            txid: "abc123".into(),
            vout: 0,
            amount_sats: 100_000,
            hash_lock: [1u8; 32],
            timeout_block: 800_000,
            sender_pubkey: "pubkey".into(),
            recipient: "nxr_addr".into(),
        };
        assert!(bridge.validate_lock(&lock).unwrap());
    }

    #[test]
    fn test_htlc_hash_lock() {
        let secret = b"my_secret_preimage";
        let hash_lock = BitcoinBridge::generate_hash_lock(secret);
        assert!(BitcoinBridge::verify_preimage(secret, &hash_lock));
        assert!(!BitcoinBridge::verify_preimage(b"wrong", &hash_lock));
    }
}
