//! Post-quantum handshake protocol.
//!
//! Combines ML-KEM key exchange with chain verification.

use nexara_crypto::{Blake3Hash, MlDsaPublicKey, WalletAddress};
use serde::{Serialize, Deserialize};

/// Handshake request sent to initiate a connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub version: String,
    pub chain_id: u64,
    pub pubkey: MlDsaPublicKey,
    pub node_address: WalletAddress,
    pub genesis_hash: Blake3Hash,
    pub best_block_height: u64,
    pub best_block_hash: Blake3Hash,
    pub shard_ids: Vec<u16>,
    pub timestamp: u64,
    pub nonce: u64,
}

/// Handshake response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub accepted: bool,
    pub version: String,
    pub chain_id: u64,
    pub pubkey: MlDsaPublicKey,
    pub node_address: WalletAddress,
    pub best_block_height: u64,
    pub best_block_hash: Blake3Hash,
    pub shard_ids: Vec<u16>,
    pub reason: Option<String>,
}

/// Validate a handshake request.
pub fn validate_handshake(
    request: &HandshakeRequest,
    our_chain_id: u64,
    our_genesis_hash: &Blake3Hash,
) -> Result<(), String> {
    if request.chain_id != our_chain_id {
        return Err(format!(
            "Chain ID mismatch: expected {}, got {}",
            our_chain_id, request.chain_id
        ));
    }
    if request.genesis_hash != *our_genesis_hash {
        return Err("Genesis hash mismatch".into());
    }
    if request.shard_ids.is_empty() {
        return Err("Peer must serve at least one shard".into());
    }
    Ok(())
}

/// Create a handshake response accepting the connection.
pub fn accept_handshake(
    our_pubkey: MlDsaPublicKey,
    our_address: WalletAddress,
    chain_id: u64,
    best_height: u64,
    best_hash: Blake3Hash,
    shard_ids: Vec<u16>,
) -> HandshakeResponse {
    HandshakeResponse {
        accepted: true,
        version: "0.1.0".into(),
        chain_id,
        pubkey: our_pubkey,
        node_address: our_address,
        best_block_height: best_height,
        best_block_hash: best_hash,
        shard_ids,
        reason: None,
    }
}

/// Create a handshake response rejecting the connection.
pub fn reject_handshake(reason: String) -> HandshakeResponse {
    HandshakeResponse {
        accepted: false,
        version: "0.1.0".into(),
        chain_id: 0,
        pubkey: MlDsaPublicKey(vec![0u8; nexara_crypto::keys::PUBKEY_SIZE]),
        node_address: WalletAddress::zero(),
        best_block_height: 0,
        best_block_hash: Blake3Hash::compute(b""),
        shard_ids: Vec::new(),
        reason: Some(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_crypto::keys::KeyPair;

    #[test]
    fn test_validate_handshake_ok() {
        let kp = KeyPair::generate();
        let genesis = Blake3Hash::compute(b"genesis");
        let req = HandshakeRequest {
            version: "0.1.0".into(),
            chain_id: 20240101,
            pubkey: kp.public,
            node_address: WalletAddress::zero(),
            genesis_hash: genesis,
            best_block_height: 0,
            best_block_hash: Blake3Hash::compute(b"block0"),
            shard_ids: vec![0],
            timestamp: 1000,
            nonce: 42,
        };
        assert!(validate_handshake(&req, 20240101, &genesis).is_ok());
    }

    #[test]
    fn test_validate_chain_id_mismatch() {
        let kp = KeyPair::generate();
        let genesis = Blake3Hash::compute(b"genesis");
        let req = HandshakeRequest {
            version: "0.1.0".into(),
            chain_id: 999,
            pubkey: kp.public,
            node_address: WalletAddress::zero(),
            genesis_hash: genesis,
            best_block_height: 0,
            best_block_hash: Blake3Hash::compute(b"block0"),
            shard_ids: vec![0],
            timestamp: 1000,
            nonce: 42,
        };
        assert!(validate_handshake(&req, 20240101, &genesis).is_err());
    }

    #[test]
    fn test_accept_handshake() {
        let kp = KeyPair::generate();
        let resp = accept_handshake(
            kp.public,
            WalletAddress::zero(),
            20240101,
            100,
            Blake3Hash::compute(b"best"),
            vec![0, 1],
        );
        assert!(resp.accepted);
        assert!(resp.reason.is_none());
    }

    #[test]
    fn test_reject_handshake() {
        let resp = reject_handshake("Bad peer".into());
        assert!(!resp.accepted);
        assert_eq!(resp.reason.unwrap(), "Bad peer");
    }
}
