//! Gossip protocol for block and transaction propagation.

use nexara_crypto::Blake3Hash;
use serde::{Serialize, Deserialize};
use crate::p2p::PeerId;
use std::collections::HashSet;

/// Types of gossip messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GossipMessageType {
    NewBlock,
    NewTransaction,
    Attestation,
    CrossShardReceipt,
    PeerAnnounce,
}

/// A gossip message propagated through the P2P network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipMessage {
    pub msg_type: GossipMessageType,
    pub payload: Vec<u8>,
    pub sender: PeerId,
    pub message_id: Blake3Hash,
    pub shard_id: u16,
    pub ttl: u8,
}

impl GossipMessage {
    /// Create a new gossip message.
    pub fn new(msg_type: GossipMessageType, payload: Vec<u8>, sender: PeerId, shard_id: u16) -> Self {
        let mut id_data = Vec::new();
        id_data.extend_from_slice(&sender);
        id_data.extend_from_slice(&payload);
        id_data.extend_from_slice(&shard_id.to_le_bytes());
        let message_id = Blake3Hash::compute(&id_data);

        GossipMessage {
            msg_type,
            payload,
            sender,
            message_id,
            shard_id,
            ttl: 8,
        }
    }

    /// Decrement TTL and check if message should still propagate.
    pub fn should_propagate(&self) -> bool {
        self.ttl > 0
    }

    /// Create a forwarded copy with decremented TTL.
    pub fn forwarded(&self) -> Self {
        let mut msg = self.clone();
        msg.ttl = msg.ttl.saturating_sub(1);
        msg
    }
}

/// Message deduplication tracker.
pub struct MessageDedup {
    seen: HashSet<Blake3Hash>,
    max_size: usize,
}

impl MessageDedup {
    pub fn new(max_size: usize) -> Self {
        MessageDedup {
            seen: HashSet::new(),
            max_size,
        }
    }

    /// Returns true if the message is new (not seen before).
    pub fn check_and_mark(&mut self, id: &Blake3Hash) -> bool {
        if self.seen.contains(id) {
            return false;
        }
        if self.seen.len() >= self.max_size {
            // Simple eviction: clear half
            let to_remove: Vec<_> = self.seen.iter().take(self.max_size / 2).copied().collect();
            for h in to_remove {
                self.seen.remove(&h);
            }
        }
        self.seen.insert(*id);
        true
    }

    pub fn len(&self) -> usize {
        self.seen.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gossip_message_creation() {
        let sender = [1u8; 32];
        let msg = GossipMessage::new(
            GossipMessageType::NewBlock,
            b"block-data".to_vec(),
            sender,
            0,
        );
        assert_eq!(msg.msg_type, GossipMessageType::NewBlock);
        assert_eq!(msg.ttl, 8);
        assert!(msg.should_propagate());
    }

    #[test]
    fn test_message_forwarding() {
        let msg = GossipMessage::new(
            GossipMessageType::NewTransaction,
            b"tx-data".to_vec(),
            [0u8; 32],
            0,
        );
        let fwd = msg.forwarded();
        assert_eq!(fwd.ttl, 7);
        assert_eq!(fwd.message_id, msg.message_id);
    }

    #[test]
    fn test_dedup() {
        let mut dedup = MessageDedup::new(100);
        let id = Blake3Hash::compute(b"msg1");
        assert!(dedup.check_and_mark(&id));
        assert!(!dedup.check_and_mark(&id)); // duplicate
    }

    #[test]
    fn test_dedup_eviction() {
        let mut dedup = MessageDedup::new(10);
        for i in 0..15u32 {
            let id = Blake3Hash::compute(&i.to_le_bytes());
            dedup.check_and_mark(&id);
        }
        assert!(dedup.len() <= 10);
    }
}
