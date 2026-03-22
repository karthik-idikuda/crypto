//! Peer discovery using bootstrap nodes and mDNS.

use crate::p2p::{PeerId, PeerInfo, NetworkConfig};
use std::collections::HashMap;

/// Peer discovery service.
pub struct PeerDiscovery {
    pub known_peers: HashMap<PeerId, PeerInfo>,
    pub boot_nodes: Vec<String>,
    pub max_peers: usize,
}

impl PeerDiscovery {
    /// Create a new peer discovery service.
    pub fn new(config: &NetworkConfig) -> Self {
        PeerDiscovery {
            known_peers: HashMap::new(),
            boot_nodes: config.boot_nodes.clone(),
            max_peers: config.max_peers,
        }
    }

    /// Register a discovered peer.
    pub fn register_peer(&mut self, info: PeerInfo) -> bool {
        if self.known_peers.len() >= self.max_peers {
            return false;
        }
        self.known_peers.insert(info.peer_id, info);
        true
    }

    /// Remove a peer from discovery.
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.known_peers.remove(peer_id);
    }

    /// Get peers closest to a target hash (for Kademlia-like routing).
    pub fn closest_peers(&self, target: &[u8; 32], count: usize) -> Vec<&PeerInfo> {
        let mut peers: Vec<_> = self.known_peers.values().collect();
        peers.sort_by_key(|p| xor_distance(&p.peer_id, target));
        peers.truncate(count);
        peers
    }

    /// Number of known peers.
    pub fn known_peer_count(&self) -> usize {
        self.known_peers.len()
    }

    /// Get all known peer IDs.
    pub fn all_peer_ids(&self) -> Vec<PeerId> {
        self.known_peers.keys().copied().collect()
    }
}

/// Compute XOR distance between two 32-byte IDs.
fn xor_distance(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..32 {
        result[i] = a[i] ^ b[i];
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer(id: u8) -> PeerInfo {
        let mut peer_id = [0u8; 32];
        peer_id[0] = id;
        PeerInfo {
            peer_id,
            address: "127.0.0.1".into(),
            port: 30303,
            version: "0.1.0".into(),
            chain_id: 20240101,
            shard_ids: vec![0],
            connected_at: 1000,
            last_seen: 1000,
            latency_ms: 10,
        }
    }

    #[test]
    fn test_register_peer() {
        let config = NetworkConfig::default();
        let mut disc = PeerDiscovery::new(&config);
        assert!(disc.register_peer(make_peer(1)));
        assert_eq!(disc.known_peer_count(), 1);
    }

    #[test]
    fn test_closest_peers() {
        let config = NetworkConfig::default();
        let mut disc = PeerDiscovery::new(&config);
        disc.register_peer(make_peer(1));
        disc.register_peer(make_peer(2));
        disc.register_peer(make_peer(3));
        let target = [1u8; 32];
        let closest = disc.closest_peers(&target, 2);
        assert_eq!(closest.len(), 2);
    }

    #[test]
    fn test_xor_distance() {
        let a = [0u8; 32];
        let b = [0u8; 32];
        assert_eq!(xor_distance(&a, &b), [0u8; 32]);
    }
}
