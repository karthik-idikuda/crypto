//! P2P networking primitives.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Unique identifier for a peer (BLAKE3 hash of their public key).
pub type PeerId = [u8; 32];

/// Information about a connected peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub address: String,
    pub port: u16,
    pub version: String,
    pub chain_id: u64,
    pub shard_ids: Vec<u16>,
    pub connected_at: u64,
    pub last_seen: u64,
    pub latency_ms: u64,
}

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub listen_port: u16,
    pub max_peers: usize,
    pub chain_id: u64,
    pub boot_nodes: Vec<String>,
    pub shard_ids: Vec<u16>,
    pub gossip_interval_ms: u64,
    pub peer_timeout_secs: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            listen_addr: "0.0.0.0".into(),
            listen_port: 30303,
            max_peers: 50,
            chain_id: 20240101,
            boot_nodes: Vec::new(),
            shard_ids: vec![0],
            gossip_interval_ms: 100,
            peer_timeout_secs: 30,
        }
    }
}

/// Peer manager tracking connected peers.
pub struct PeerManager {
    pub peers: HashMap<PeerId, PeerInfo>,
    pub config: NetworkConfig,
}

impl PeerManager {
    pub fn new(config: NetworkConfig) -> Self {
        PeerManager {
            peers: HashMap::new(),
            config,
        }
    }

    /// Add a peer.
    pub fn add_peer(&mut self, info: PeerInfo) -> bool {
        if self.peers.len() >= self.config.max_peers {
            return false;
        }
        if info.chain_id != self.config.chain_id {
            return false;
        }
        self.peers.insert(info.peer_id, info);
        true
    }

    /// Remove a peer.
    pub fn remove_peer(&mut self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.remove(peer_id)
    }

    /// Get a peer by ID.
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }

    /// Get peers that serve a specific shard.
    pub fn peers_for_shard(&self, shard_id: u16) -> Vec<&PeerInfo> {
        self.peers.values()
            .filter(|p| p.shard_ids.contains(&shard_id))
            .collect()
    }

    /// Number of connected peers.
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    /// Check if we have capacity for more peers.
    pub fn has_capacity(&self) -> bool {
        self.peers.len() < self.config.max_peers
    }

    /// Get all connected peer IDs.
    pub fn connected_peer_ids(&self) -> Vec<PeerId> {
        self.peers.keys().copied().collect()
    }

    /// Prune stale peers based on last_seen timestamp.
    pub fn prune_stale_peers(&mut self, current_time: u64) {
        let timeout = self.config.peer_timeout_secs;
        self.peers.retain(|_, p| {
            current_time.saturating_sub(p.last_seen) < timeout
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer(id: u8, shard: u16) -> PeerInfo {
        let mut peer_id = [0u8; 32];
        peer_id[0] = id;
        PeerInfo {
            peer_id,
            address: "127.0.0.1".into(),
            port: 30303,
            version: "0.1.0".into(),
            chain_id: 20240101,
            shard_ids: vec![shard],
            connected_at: 1000,
            last_seen: 1000,
            latency_ms: 10,
        }
    }

    #[test]
    fn test_add_peer() {
        let mut pm = PeerManager::new(NetworkConfig::default());
        assert!(pm.add_peer(make_peer(1, 0)));
        assert_eq!(pm.peer_count(), 1);
    }

    #[test]
    fn test_max_peers() {
        let config = NetworkConfig { max_peers: 1, ..NetworkConfig::default() };
        let mut pm = PeerManager::new(config);
        assert!(pm.add_peer(make_peer(1, 0)));
        assert!(!pm.add_peer(make_peer(2, 0)));
    }

    #[test]
    fn test_wrong_chain_id() {
        let mut pm = PeerManager::new(NetworkConfig::default());
        let mut peer = make_peer(1, 0);
        peer.chain_id = 999;
        assert!(!pm.add_peer(peer));
    }

    #[test]
    fn test_peers_for_shard() {
        let mut pm = PeerManager::new(NetworkConfig::default());
        pm.add_peer(make_peer(1, 0));
        pm.add_peer(make_peer(2, 1));
        pm.add_peer(make_peer(3, 0));
        assert_eq!(pm.peers_for_shard(0).len(), 2);
        assert_eq!(pm.peers_for_shard(1).len(), 1);
    }

    #[test]
    fn test_prune_stale() {
        let mut pm = PeerManager::new(NetworkConfig::default());
        let mut peer = make_peer(1, 0);
        peer.last_seen = 100;
        pm.add_peer(peer);
        pm.prune_stale_peers(200); // 200 - 100 = 100 > 30 timeout
        assert_eq!(pm.peer_count(), 0);
    }
}
