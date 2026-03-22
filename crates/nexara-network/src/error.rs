//! Network error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Invalid peer: {0}")]
    InvalidPeer(String),
    #[error("Gossip error: {0}")]
    GossipError(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
}
