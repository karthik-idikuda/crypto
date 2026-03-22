//! NEXARA P2P networking layer.
//!
//! Provides gossip-based block/transaction propagation,
//! peer discovery, handshake protocol, and shard-aware messaging.

pub mod p2p;
pub mod gossip;
pub mod handshake;
pub mod discovery;
pub mod error;

pub use error::NetworkError;
pub use p2p::{PeerInfo, PeerId, NetworkConfig};
pub use gossip::{GossipMessage, GossipMessageType};
pub use handshake::{HandshakeRequest, HandshakeResponse};
pub use discovery::PeerDiscovery;
