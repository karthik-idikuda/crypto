//! Block structure for the NEXARA blockchain.
//!
//! Blocks contain a header with cryptographic commitments (state root, tx root,
//! validator set hash, ZK finality proof) and a body with transactions and
//! cross-shard receipts.

use serde::{Serialize, Deserialize};
use nexara_crypto::{Blake3Hash, WalletAddress};
use crate::transaction::Transaction;
use crate::error::CoreError;

/// Maximum block size in bytes (2 MB).
pub const MAX_BLOCK_SIZE: usize = 2 * 1024 * 1024;

/// Block header containing all cryptographic commitments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Protocol version.
    pub version: u8,
    /// Which shard this block belongs to (0-99).
    pub shard_id: u16,
    /// Block height (0 = genesis).
    pub height: u64,
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
    /// Hash of the parent block.
    pub parent_hash: Blake3Hash,
    /// Merkle root of the world state after executing this block.
    pub state_root: Blake3Hash,
    /// Merkle root of all transactions in this block.
    pub tx_root: Blake3Hash,
    /// Hash of the current validator set.
    pub validator_set_hash: Blake3Hash,
    /// Address of the block proposer.
    pub proposer_address: WalletAddress,
    /// PLONK ZK finality proof (~300 bytes).
    pub zk_finality_proof: Vec<u8>,
    /// Bitfield indicating which validators attested to this block.
    pub attestation_bitfield: Vec<u8>,
}

/// A complete block (header + body).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub cross_shard_receipts: Vec<CrossShardReceipt>,
}

/// A receipt proving a cross-shard transaction was executed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossShardReceipt {
    /// Source shard that initiated the transaction.
    pub source_shard: u16,
    /// Destination shard.
    pub dest_shard: u16,
    /// Hash of the original transaction.
    pub tx_hash: Blake3Hash,
    /// Merkle proof of inclusion in the source shard's state.
    pub proof: Vec<u8>,
}

impl BlockHeader {
    /// Serialize the header to bytes.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Compute the BLAKE3 hash of the header.
    pub fn hash(&self) -> Blake3Hash {
        Blake3Hash::compute(&self.serialize())
    }
}

impl Block {
    /// Create a new block from a header and transactions.
    pub fn new(header: BlockHeader, txs: Vec<Transaction>) -> Self {
        Block {
            header,
            transactions: txs,
            cross_shard_receipts: Vec::new(),
        }
    }

    /// Compute the BLAKE3 hash of this block (hash of the header).
    pub fn hash(&self) -> Blake3Hash {
        self.header.hash()
    }

    /// Serialize the entire block.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Deserialize a block from bytes.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, CoreError> {
        bincode::deserialize(bytes)
            .map_err(|e| CoreError::Serialization(e.to_string()))
    }

    /// Number of transactions in this block.
    pub fn tx_count(&self) -> usize {
        self.transactions.len()
    }

    /// Total size of the block in bytes.
    pub fn size_bytes(&self) -> usize {
        self.serialize().len()
    }

    /// Verify that the tx_root in the header matches the actual transactions.
    pub fn verify_tx_root(&self) -> bool {
        let tx_hashes: Vec<Blake3Hash> = self.transactions.iter().map(|tx| tx.hash()).collect();
        let computed_root = calculate_merkle_root(&tx_hashes);
        self.header.tx_root == computed_root
    }

    /// Check if this is the genesis block (height 0).
    pub fn is_genesis(&self) -> bool {
        self.header.height == 0
    }

    /// Validate block structure (not consensus).
    pub fn validate_structure(&self) -> Result<(), CoreError> {
        let size = self.size_bytes();
        if size > MAX_BLOCK_SIZE {
            return Err(CoreError::BlockTooLarge {
                size,
                max: MAX_BLOCK_SIZE,
            });
        }
        if self.header.shard_id >= crate::transaction::NUM_SHARDS {
            return Err(CoreError::ShardOutOfRange(self.header.shard_id));
        }
        Ok(())
    }
}

/// Calculate a binary Merkle tree root from a list of BLAKE3 hashes.
///
/// If the number of leaves is odd, the last leaf is duplicated.
/// An empty list produces a zero hash.
pub fn calculate_merkle_root(hashes: &[Blake3Hash]) -> Blake3Hash {
    if hashes.is_empty() {
        return Blake3Hash::zero();
    }
    if hashes.len() == 1 {
        return hashes[0];
    }

    let mut current_level: Vec<Blake3Hash> = hashes.to_vec();

    while current_level.len() > 1 {
        // Pad with duplicate of last element if odd
        if !current_level.len().is_multiple_of(2) {
            let last = *current_level.last().unwrap();
            current_level.push(last);
        }

        let mut next_level = Vec::with_capacity(current_level.len() / 2);
        for pair in current_level.chunks(2) {
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(pair[0].as_bytes());
            combined.extend_from_slice(pair[1].as_bytes());
            next_level.push(Blake3Hash::compute(&combined));
        }
        current_level = next_level;
    }

    current_level[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_crypto::keys::KeyPair;
    use crate::transaction::{Transaction, ONE_NXR};

    fn make_test_block(num_txs: usize) -> Block {
        let proposer = KeyPair::generate().public.wallet_address();
        let txs: Vec<Transaction> = (0..num_txs)
            .map(|i| {
                let sender = KeyPair::generate().public.wallet_address();
                let recipient = KeyPair::generate().public.wallet_address();
                Transaction::new_transfer(sender, recipient, ONE_NXR, ONE_NXR / 1000, i as u64, 0)
            })
            .collect();

        let tx_hashes: Vec<Blake3Hash> = txs.iter().map(|tx| tx.hash()).collect();
        let tx_root = calculate_merkle_root(&tx_hashes);

        let header = BlockHeader {
            version: 1,
            shard_id: 0,
            height: 1,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            parent_hash: Blake3Hash::zero(),
            state_root: Blake3Hash::zero(),
            tx_root,
            validator_set_hash: Blake3Hash::zero(),
            proposer_address: proposer,
            zk_finality_proof: vec![0u8; 300],
            attestation_bitfield: vec![0xFF; 16],
        };

        Block::new(header, txs)
    }

    #[test]
    fn test_block_creation() {
        let block = make_test_block(10);
        assert_eq!(block.tx_count(), 10);
        assert!(!block.is_genesis());
    }

    #[test]
    fn test_genesis_block() {
        let header = BlockHeader {
            version: 1,
            shard_id: 0,
            height: 0,
            timestamp: 0,
            parent_hash: Blake3Hash::zero(),
            state_root: Blake3Hash::zero(),
            tx_root: Blake3Hash::zero(),
            validator_set_hash: Blake3Hash::zero(),
            proposer_address: WalletAddress::zero(),
            zk_finality_proof: Vec::new(),
            attestation_bitfield: Vec::new(),
        };
        let block = Block::new(header, Vec::new());
        assert!(block.is_genesis());
    }

    #[test]
    fn test_verify_tx_root() {
        let block = make_test_block(5);
        assert!(block.verify_tx_root());
    }

    #[test]
    fn test_serialize_deserialize() {
        let block = make_test_block(3);
        let bytes = block.serialize();
        let recovered = Block::deserialize(&bytes).unwrap();
        assert_eq!(recovered.tx_count(), 3);
        assert_eq!(recovered.header.height, block.header.height);
    }

    #[test]
    fn test_merkle_root_empty() {
        assert_eq!(calculate_merkle_root(&[]), Blake3Hash::zero());
    }

    #[test]
    fn test_merkle_root_single() {
        let h = Blake3Hash::compute(b"test");
        assert_eq!(calculate_merkle_root(&[h]), h);
    }

    #[test]
    fn test_merkle_root_deterministic() {
        let hashes: Vec<Blake3Hash> = (0..4)
            .map(|i| Blake3Hash::compute(&[i as u8]))
            .collect();
        let r1 = calculate_merkle_root(&hashes);
        let r2 = calculate_merkle_root(&hashes);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_merkle_root_odd_count() {
        let hashes: Vec<Blake3Hash> = (0..3)
            .map(|i| Blake3Hash::compute(&[i as u8]))
            .collect();
        let root = calculate_merkle_root(&hashes);
        assert_ne!(root, Blake3Hash::zero());
    }
}
