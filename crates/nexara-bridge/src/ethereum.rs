//! Ethereum bridge adapter.

use serde::{Deserialize, Serialize};
use crate::error::BridgeError;

/// Ethereum bridge configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumBridge {
    pub contract_address: String,
    pub chain_id: u64,
    pub confirmations_required: u32,
    pub enabled: bool,
}

/// Ethereum deposit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthDepositEvent {
    pub tx_hash: String,
    pub sender: String,
    pub recipient: String,
    pub amount: u128,
    pub token: String,
    pub block_number: u64,
}

impl EthereumBridge {
    pub fn new(contract_address: &str, chain_id: u64) -> Self {
        EthereumBridge {
            contract_address: contract_address.to_string(),
            chain_id,
            confirmations_required: 12,
            enabled: true,
        }
    }

    /// Validate an Ethereum deposit event.
    pub fn validate_deposit(&self, event: &EthDepositEvent) -> Result<bool, BridgeError> {
        if !self.enabled {
            return Err(BridgeError::BridgePaused("Ethereum".into()));
        }
        if event.tx_hash.is_empty() {
            return Err(BridgeError::InvalidProof("Empty tx hash".into()));
        }
        if event.amount == 0 {
            return Err(BridgeError::InvalidMessage("Zero amount".into()));
        }
        // Verify tx hash format (0x + 64 hex chars)
        let valid_hash = event.tx_hash.starts_with("0x") && event.tx_hash.len() == 66;
        Ok(valid_hash)
    }

    /// Generate a Merkle proof for withdrawal (simplified).
    pub fn generate_withdrawal_proof(&self, recipient: &str, amount: u128) -> Vec<u8> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"ETH_WITHDRAWAL");
        hasher.update(recipient.as_bytes());
        hasher.update(&amount.to_le_bytes());
        hasher.finalize().as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_deposit() {
        let bridge = EthereumBridge::new("0x1234567890abcdef1234567890abcdef12345678", 1);
        let event = EthDepositEvent {
            tx_hash: "0x".to_string() + &"ab".repeat(32),
            sender: "0xsender".into(),
            recipient: "nxr_recipient".into(),
            amount: 1000,
            token: "ETH".into(),
            block_number: 100,
        };
        assert!(bridge.validate_deposit(&event).unwrap());
    }

    #[test]
    fn test_invalid_deposit() {
        let bridge = EthereumBridge::new("0xcontract", 1);
        let event = EthDepositEvent {
            tx_hash: "invalid".into(),
            sender: "s".into(),
            recipient: "r".into(),
            amount: 100,
            token: "ETH".into(),
            block_number: 1,
        };
        assert!(!bridge.validate_deposit(&event).unwrap());
    }

    #[test]
    fn test_withdrawal_proof() {
        let bridge = EthereumBridge::new("0x", 1);
        let proof = bridge.generate_withdrawal_proof("recipient", 500);
        assert_eq!(proof.len(), 32);
    }
}
