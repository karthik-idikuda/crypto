//! Cross-shard communication and cross-links.

use nexara_crypto::Blake3Hash;
use serde::{Serialize, Deserialize};

/// A cross-link proving a shard block is finalized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossLink {
    pub source_shard: u16,
    pub target_shard: u16,
    pub block_height: u64,
    pub block_hash: Blake3Hash,
    pub state_root: Blake3Hash,
    pub receipt_root: Blake3Hash,
}

/// A cross-shard message (e.g., for cross-shard transfers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardMessage {
    pub id: Blake3Hash,
    pub source_shard: u16,
    pub target_shard: u16,
    pub payload: CrossShardPayload,
    pub source_block_height: u64,
    pub proof: Vec<u8>,
    pub status: CrossShardStatus,
}

/// The payload of a cross-shard message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossShardPayload {
    Transfer {
        from: nexara_crypto::WalletAddress,
        to: nexara_crypto::WalletAddress,
        amount: u128,
    },
    ContractCall {
        contract: nexara_crypto::WalletAddress,
        data: Vec<u8>,
    },
    Receipt {
        original_msg_id: Blake3Hash,
        success: bool,
        return_data: Vec<u8>,
    },
}

/// Status of a cross-shard message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossShardStatus {
    Pending,
    Delivered,
    Confirmed,
    Failed,
}

impl CrossShardMessage {
    /// Create a new cross-shard transfer message.
    pub fn new_transfer(
        source_shard: u16,
        target_shard: u16,
        from: nexara_crypto::WalletAddress,
        to: nexara_crypto::WalletAddress,
        amount: u128,
        source_block_height: u64,
    ) -> Self {
        let mut id_data = Vec::new();
        id_data.extend_from_slice(&source_shard.to_le_bytes());
        id_data.extend_from_slice(&target_shard.to_le_bytes());
        id_data.extend_from_slice(&from.0);
        id_data.extend_from_slice(&to.0);
        id_data.extend_from_slice(&amount.to_le_bytes());
        id_data.extend_from_slice(&source_block_height.to_le_bytes());
        let id = Blake3Hash::compute(&id_data);

        CrossShardMessage {
            id,
            source_shard,
            target_shard,
            payload: CrossShardPayload::Transfer { from, to, amount },
            source_block_height,
            proof: Vec::new(),
            status: CrossShardStatus::Pending,
        }
    }

    /// Mark as delivered.
    pub fn mark_delivered(&mut self) {
        self.status = CrossShardStatus::Delivered;
    }

    /// Mark as confirmed.
    pub fn mark_confirmed(&mut self) {
        self.status = CrossShardStatus::Confirmed;
    }

    /// Mark as failed.
    pub fn mark_failed(&mut self) {
        self.status = CrossShardStatus::Failed;
    }
}

/// Verify a cross-link.
pub fn verify_cross_link(link: &CrossLink) -> bool {
    // Basic validation
    link.source_shard != link.target_shard
        && link.source_shard < 100
        && link.target_shard < 100
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_crypto::WalletAddress;

    #[test]
    fn test_cross_shard_transfer() {
        let msg = CrossShardMessage::new_transfer(
            0, 1,
            WalletAddress::zero(), WalletAddress::zero(),
            1000,
            100,
        );
        assert_eq!(msg.source_shard, 0);
        assert_eq!(msg.target_shard, 1);
        assert_eq!(msg.status, CrossShardStatus::Pending);
    }

    #[test]
    fn test_status_transitions() {
        let mut msg = CrossShardMessage::new_transfer(
            0, 1,
            WalletAddress::zero(), WalletAddress::zero(),
            1000, 100,
        );
        msg.mark_delivered();
        assert_eq!(msg.status, CrossShardStatus::Delivered);
        msg.mark_confirmed();
        assert_eq!(msg.status, CrossShardStatus::Confirmed);
    }

    #[test]
    fn test_verify_cross_link() {
        let link = CrossLink {
            source_shard: 0,
            target_shard: 1,
            block_height: 100,
            block_hash: Blake3Hash::compute(b"block"),
            state_root: Blake3Hash::compute(b"state"),
            receipt_root: Blake3Hash::compute(b"receipts"),
        };
        assert!(verify_cross_link(&link));
    }

    #[test]
    fn test_invalid_cross_link_same_shard() {
        let link = CrossLink {
            source_shard: 0,
            target_shard: 0,
            block_height: 100,
            block_hash: Blake3Hash::compute(b"block"),
            state_root: Blake3Hash::compute(b"state"),
            receipt_root: Blake3Hash::compute(b"receipts"),
        };
        assert!(!verify_cross_link(&link));
    }
}
