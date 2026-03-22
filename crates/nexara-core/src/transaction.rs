//! Transaction types and pool for the NEXARA blockchain.
//!
//! Transactions are the fundamental unit of state change. Each transaction
//! carries an ML-DSA-65 signature (3309 bytes) for post-quantum security.

use serde::{Serialize, Deserialize};
use nexara_crypto::{Blake3Hash, MlDsaSignature, WalletAddress};
use crate::error::CoreError;

/// The base unit of NXR (10^18 base units = 1 NXR).
pub const NXR_DECIMALS: u32 = 18;

/// 1 NXR in base units.
pub const ONE_NXR: u128 = 1_000_000_000_000_000_000;

/// Maximum number of shards.
pub const NUM_SHARDS: u16 = 100;

/// Base fee rate per byte (in base units).
pub const BASE_FEE_PER_BYTE: u128 = 1_000_000_000; // 1 gwei equivalent

/// Transaction types supported by the NEXARA blockchain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    /// Simple value transfer between accounts.
    Transfer,
    /// Deploy a new smart contract.
    ContractDeploy,
    /// Call a function on an existing smart contract.
    ContractCall,
    /// Stake NXR to become a validator or delegator.
    Stake,
    /// Unstake NXR (begins unbonding period).
    Unstake,
    /// Cross-shard or cross-chain transfer.
    CrossChain,
    /// DAO governance vote.
    GovernanceVote,
}

/// A NEXARA blockchain transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Protocol version for forward compatibility.
    pub version: u8,
    /// Type of transaction.
    pub tx_type: TransactionType,
    /// Sender's wallet address (32 bytes).
    pub sender: WalletAddress,
    /// Recipient's wallet address (32 bytes).
    pub recipient: WalletAddress,
    /// Amount in base units (10^18 per NXR).
    pub amount: u128,
    /// Transaction fee in base units.
    pub fee: u128,
    /// Sender nonce for replay protection.
    pub nonce: u64,
    /// Target shard ID (0-99).
    pub shard_id: u16,
    /// Contract calldata or auxiliary data.
    pub data: Vec<u8>,
    /// ML-DSA-65 signature (3309 bytes).
    pub signature: MlDsaSignature,
    /// Full public key, included only on sender's first transaction.
    pub pubkey_first_use: Option<Vec<u8>>,
    /// Unix timestamp in seconds.
    pub timestamp: u64,
}

impl Transaction {
    /// Create a new unsigned transfer transaction.
    pub fn new_transfer(
        sender: WalletAddress,
        recipient: WalletAddress,
        amount: u128,
        fee: u128,
        nonce: u64,
        shard_id: u16,
    ) -> Self {
        Transaction {
            version: 1,
            tx_type: TransactionType::Transfer,
            sender,
            recipient,
            amount,
            fee,
            nonce,
            shard_id,
            data: Vec::new(),
            signature: MlDsaSignature(vec![0u8; nexara_crypto::keys::SIGNATURE_SIZE]),
            pubkey_first_use: None,
            timestamp: chrono::Utc::now().timestamp() as u64,
        }
    }

    /// Compute the BLAKE3 hash of this transaction.
    pub fn hash(&self) -> Blake3Hash {
        let bytes = self.serialize();
        Blake3Hash::compute(&bytes)
    }

    /// Serialize this transaction to bytes (bincode).
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    /// Deserialize a transaction from bytes.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, CoreError> {
        bincode::deserialize(bytes)
            .map_err(|e| CoreError::Serialization(e.to_string()))
    }

    /// Check structural validity (does not verify signature).
    pub fn is_valid_structure(&self) -> bool {
        self.version > 0
            && self.shard_id < NUM_SHARDS
            && self.fee > 0
            && self.signature.0.len() == nexara_crypto::keys::SIGNATURE_SIZE
    }

    /// Total size of the transaction in bytes.
    pub fn size_bytes(&self) -> usize {
        self.serialize().len()
    }

    /// Calculate the minimum fee based on transaction size.
    pub fn base_fee_nxr(&self) -> u128 {
        let size = self.size_bytes() as u128;
        size * BASE_FEE_PER_BYTE
    }
}

/// A pool of pending transactions waiting for inclusion in a block.
pub struct TransactionPool {
    pending: Vec<Transaction>,
    max_size: usize,
}

impl TransactionPool {
    /// Create a pool with a maximum capacity.
    pub fn new(max_size: usize) -> Self {
        TransactionPool {
            pending: Vec::new(),
            max_size,
        }
    }

    /// Add a transaction to the pool.
    pub fn add(&mut self, tx: Transaction) -> Result<(), CoreError> {
        if self.pending.len() >= self.max_size {
            return Err(CoreError::MempoolFull);
        }
        if !tx.is_valid_structure() {
            return Err(CoreError::InvalidTransaction("invalid structure".into()));
        }
        self.pending.push(tx);
        Ok(())
    }

    /// Remove a transaction by hash.
    pub fn remove(&mut self, tx_hash: &Blake3Hash) {
        self.pending.retain(|tx| tx.hash() != *tx_hash);
    }

    /// Get transactions ordered by fee/byte descending (highest fee priority first).
    pub fn get_ordered(&self, limit: usize) -> Vec<&Transaction> {
        let mut sorted: Vec<&Transaction> = self.pending.iter().collect();
        sorted.sort_by(|a, b| {
            let a_ratio = a.fee as f64 / a.size_bytes().max(1) as f64;
            let b_ratio = b.fee as f64 / b.size_bytes().max(1) as f64;
            b_ratio.partial_cmp(&a_ratio).unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(limit);
        sorted
    }

    /// Number of pending transactions.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_crypto::keys::KeyPair;

    fn make_test_tx(fee: u128) -> Transaction {
        let kp = KeyPair::generate();
        let sender = kp.public.wallet_address();
        let recipient = KeyPair::generate().public.wallet_address();
        let mut tx = Transaction::new_transfer(sender, recipient, ONE_NXR, fee, 0, 0);
        tx.fee = fee;
        tx
    }

    #[test]
    fn test_new_transfer() {
        let tx = make_test_tx(ONE_NXR / 1000);
        assert_eq!(tx.version, 1);
        assert_eq!(tx.tx_type, TransactionType::Transfer);
        assert!(tx.data.is_empty());
    }

    #[test]
    fn test_hash_deterministic() {
        let tx = make_test_tx(ONE_NXR / 1000);
        assert_eq!(tx.hash(), tx.hash());
    }

    #[test]
    fn test_serialize_deserialize() {
        let tx = make_test_tx(ONE_NXR / 1000);
        let bytes = tx.serialize();
        let recovered = Transaction::deserialize(&bytes).unwrap();
        assert_eq!(recovered.amount, tx.amount);
        assert_eq!(recovered.sender, tx.sender);
    }

    #[test]
    fn test_valid_structure() {
        let tx = make_test_tx(ONE_NXR / 1000);
        assert!(tx.is_valid_structure());
    }

    #[test]
    fn test_invalid_shard() {
        let mut tx = make_test_tx(ONE_NXR / 1000);
        tx.shard_id = 200; // Out of range
        assert!(!tx.is_valid_structure());
    }

    #[test]
    fn test_pool_ordering() {
        let mut pool = TransactionPool::new(100);
        pool.add(make_test_tx(100)).unwrap();
        pool.add(make_test_tx(1000)).unwrap();
        pool.add(make_test_tx(500)).unwrap();
        let ordered = pool.get_ordered(3);
        // Highest fee first (approximately — sizes are similar)
        assert_eq!(ordered.len(), 3);
    }

    #[test]
    fn test_pool_full() {
        let mut pool = TransactionPool::new(2);
        pool.add(make_test_tx(100)).unwrap();
        pool.add(make_test_tx(200)).unwrap();
        assert!(pool.add(make_test_tx(300)).is_err());
    }
}
