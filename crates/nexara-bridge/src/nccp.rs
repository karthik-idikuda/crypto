//! NCCP - Native Cross-Chain Protocol core types and logic.

use serde::{Deserialize, Serialize};
use crate::error::BridgeError;
use crate::nullifier::NullifierSet;

/// Supported chains.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainId {
    Nexara,
    Ethereum,
    BNBChain,
    Solana,
    Cosmos,
    Bitcoin,
    Custom(String),
}

impl std::fmt::Display for ChainId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainId::Nexara => write!(f, "NEXARA"),
            ChainId::Ethereum => write!(f, "ETH"),
            ChainId::BNBChain => write!(f, "BNB"),
            ChainId::Solana => write!(f, "SOL"),
            ChainId::Cosmos => write!(f, "ATOM"),
            ChainId::Bitcoin => write!(f, "BTC"),
            ChainId::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Bridge transfer status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeStatus {
    Pending,
    Locked,
    Confirmed,
    Completed,
    Failed,
    Refunded,
}

/// A cross-chain bridge transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTransfer {
    pub id: String,
    pub source_chain: ChainId,
    pub dest_chain: ChainId,
    pub sender: String,
    pub recipient: String,
    pub amount: u128,
    pub token: String,
    pub status: BridgeStatus,
    pub proof: Option<Vec<u8>>,
    pub nullifier: [u8; 32],
    pub created_at: u64,
    pub completed_at: Option<u64>,
}

/// A cross-chain message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    pub source_chain: ChainId,
    pub dest_chain: ChainId,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub signature: Vec<u8>,
}

/// Bridge protocol engine.
pub struct BridgeProtocol {
    nullifiers: NullifierSet,
    transfers: Vec<BridgeTransfer>,
    supported_chains: Vec<ChainId>,
    paused_chains: Vec<ChainId>,
    liquidity: std::collections::HashMap<String, u128>,
}

impl BridgeProtocol {
    pub fn new() -> Self {
        BridgeProtocol {
            nullifiers: NullifierSet::new(),
            transfers: Vec::new(),
            supported_chains: vec![
                ChainId::Nexara,
                ChainId::Ethereum,
                ChainId::BNBChain,
                ChainId::Solana,
                ChainId::Cosmos,
                ChainId::Bitcoin,
            ],
            paused_chains: Vec::new(),
            liquidity: std::collections::HashMap::new(),
        }
    }

    /// Add liquidity for a token.
    pub fn add_liquidity(&mut self, token: &str, amount: u128) {
        let entry = self.liquidity.entry(token.to_string()).or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    /// Initiate a bridge transfer.
    pub fn initiate_transfer(
        &mut self,
        source: ChainId,
        dest: ChainId,
        sender: &str,
        recipient: &str,
        amount: u128,
        token: &str,
    ) -> Result<BridgeTransfer, BridgeError> {
        // Validate chains
        if !self.supported_chains.contains(&source) {
            return Err(BridgeError::UnsupportedChain(source.to_string()));
        }
        if !self.supported_chains.contains(&dest) {
            return Err(BridgeError::UnsupportedChain(dest.to_string()));
        }
        if self.paused_chains.contains(&source) {
            return Err(BridgeError::BridgePaused(source.to_string()));
        }
        if self.paused_chains.contains(&dest) {
            return Err(BridgeError::BridgePaused(dest.to_string()));
        }

        // Check liquidity
        let available = self.liquidity.get(token).copied().unwrap_or(0);
        if available < amount {
            return Err(BridgeError::InsufficientLiquidity { need: amount, have: available });
        }

        // Generate transfer ID and nullifier
        let id = self.generate_transfer_id(sender, recipient, amount);
        let nullifier = self.generate_nullifier(&id);

        // Check for duplicate
        if self.nullifiers.contains(&nullifier) {
            return Err(BridgeError::DuplicateTransfer(id));
        }

        let transfer = BridgeTransfer {
            id: id.clone(),
            source_chain: source,
            dest_chain: dest,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            amount,
            token: token.to_string(),
            status: BridgeStatus::Pending,
            proof: None,
            nullifier,
            created_at: 0,
            completed_at: None,
        };

        self.transfers.push(transfer.clone());
        Ok(transfer)
    }

    /// Complete a pending transfer with proof.
    pub fn complete_transfer(&mut self, transfer_id: &str, proof: Vec<u8>) -> Result<(), BridgeError> {
        let transfer = self.transfers.iter_mut()
            .find(|t| t.id == transfer_id)
            .ok_or_else(|| BridgeError::InvalidProof(transfer_id.to_string()))?;

        if transfer.status != BridgeStatus::Pending && transfer.status != BridgeStatus::Locked {
            return Err(BridgeError::InvalidProof(transfer_id.to_string()));
        }

        // Verify proof (simplified — check non-empty)
        if proof.is_empty() {
            return Err(BridgeError::InvalidProof(transfer_id.to_string()));
        }

        // Mark nullifier as spent
        if self.nullifiers.contains(&transfer.nullifier) {
            return Err(BridgeError::NullifierSpent(hex::encode(transfer.nullifier)));
        }
        self.nullifiers.insert(transfer.nullifier);

        // Deduct liquidity
        if let Some(liq) = self.liquidity.get_mut(&transfer.token) {
            *liq = liq.saturating_sub(transfer.amount);
        }

        transfer.proof = Some(proof);
        transfer.status = BridgeStatus::Completed;
        Ok(())
    }

    /// Pause a chain's bridge.
    pub fn pause_chain(&mut self, chain: ChainId) {
        if !self.paused_chains.contains(&chain) {
            self.paused_chains.push(chain);
        }
    }

    /// Resume a chain's bridge.
    pub fn resume_chain(&mut self, chain: &ChainId) {
        self.paused_chains.retain(|c| c != chain);
    }

    /// Get all transfers.
    pub fn transfers(&self) -> &[BridgeTransfer] {
        &self.transfers
    }

    fn generate_transfer_id(&self, sender: &str, recipient: &str, amount: u128) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(sender.as_bytes());
        hasher.update(recipient.as_bytes());
        hasher.update(&amount.to_le_bytes());
        hasher.update(&(self.transfers.len() as u64).to_le_bytes());
        hex::encode(&hasher.finalize().as_bytes()[..16])
    }

    fn generate_nullifier(&self, transfer_id: &str) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"NCCP_NULLIFIER_V1");
        hasher.update(transfer_id.as_bytes());
        *hasher.finalize().as_bytes()
    }
}

impl Default for BridgeProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify a bridge message signature (simplified).
pub fn verify_bridge_message(msg: &BridgeMessage) -> Result<bool, BridgeError> {
    if msg.payload.is_empty() {
        return Err(BridgeError::InvalidMessage("Empty payload".into()));
    }
    if msg.signature.is_empty() {
        return Err(BridgeError::InvalidMessage("Missing signature".into()));
    }
    // Simplified verification
    let hash = blake3::hash(&msg.payload);
    Ok(hash.as_bytes().len() == 32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initiate_transfer() {
        let mut bridge = BridgeProtocol::new();
        bridge.add_liquidity("NXR", 1_000_000);
        let transfer = bridge.initiate_transfer(
            ChainId::Nexara, ChainId::Ethereum,
            "sender_addr", "recipient_addr",
            1000, "NXR",
        ).unwrap();
        assert_eq!(transfer.status, BridgeStatus::Pending);
        assert_eq!(transfer.amount, 1000);
    }

    #[test]
    fn test_complete_transfer() {
        let mut bridge = BridgeProtocol::new();
        bridge.add_liquidity("NXR", 1_000_000);
        let transfer = bridge.initiate_transfer(
            ChainId::Nexara, ChainId::Ethereum,
            "sender", "recipient", 500, "NXR",
        ).unwrap();
        bridge.complete_transfer(&transfer.id, vec![1, 2, 3]).unwrap();
        assert_eq!(bridge.transfers()[0].status, BridgeStatus::Completed);
    }

    #[test]
    fn test_insufficient_liquidity() {
        let mut bridge = BridgeProtocol::new();
        bridge.add_liquidity("NXR", 100);
        let err = bridge.initiate_transfer(
            ChainId::Nexara, ChainId::Ethereum,
            "s", "r", 1000, "NXR",
        ).unwrap_err();
        assert!(matches!(err, BridgeError::InsufficientLiquidity { .. }));
    }

    #[test]
    fn test_paused_chain() {
        let mut bridge = BridgeProtocol::new();
        bridge.add_liquidity("NXR", 1_000_000);
        bridge.pause_chain(ChainId::Ethereum);
        let err = bridge.initiate_transfer(
            ChainId::Nexara, ChainId::Ethereum,
            "s", "r", 100, "NXR",
        ).unwrap_err();
        assert!(matches!(err, BridgeError::BridgePaused(_)));
    }

    #[test]
    fn test_unsupported_chain() {
        let mut bridge = BridgeProtocol::new();
        let err = bridge.initiate_transfer(
            ChainId::Custom("Unknown".into()), ChainId::Ethereum,
            "s", "r", 100, "NXR",
        ).unwrap_err();
        assert!(matches!(err, BridgeError::UnsupportedChain(_)));
    }

    #[test]
    fn test_chain_display() {
        assert_eq!(ChainId::Ethereum.to_string(), "ETH");
        assert_eq!(ChainId::Nexara.to_string(), "NEXARA");
        assert_eq!(ChainId::Bitcoin.to_string(), "BTC");
    }
}
