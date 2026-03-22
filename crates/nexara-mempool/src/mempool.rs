//! Core mempool implementation.

use nexara_core::transaction::Transaction;
use nexara_crypto::Blake3Hash;
use crate::error::MempoolError;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Transaction mempool with priority ordering.
pub struct Mempool {
    /// Transactions indexed by hash.
    transactions: HashMap<Blake3Hash, Transaction>,
    /// Priority-ordered transaction hashes (fee descending).
    by_fee: BTreeMap<std::cmp::Reverse<u128>, Vec<Blake3Hash>>,
    /// Transactions by sender for nonce tracking.
    by_sender: HashMap<Blake3Hash, Vec<Blake3Hash>>,
    /// Known transaction hashes for dedup.
    known: HashSet<Blake3Hash>,
    /// Maximum pool size.
    max_size: usize,
    /// Minimum fee.
    min_fee: u128,
}

impl Mempool {
    /// Create a new mempool.
    pub fn new(max_size: usize, min_fee: u128) -> Self {
        Mempool {
            transactions: HashMap::new(),
            by_fee: BTreeMap::new(),
            by_sender: HashMap::new(),
            known: HashSet::new(),
            max_size,
            min_fee,
        }
    }

    /// Add a transaction to the mempool.
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<Blake3Hash, MempoolError> {
        let hash = tx.hash();

        if self.known.contains(&hash) {
            return Err(MempoolError::Duplicate(hex::encode(hash.as_bytes())));
        }

        if tx.fee < self.min_fee {
            return Err(MempoolError::FeeTooLow { minimum: self.min_fee });
        }

        if self.transactions.len() >= self.max_size {
            // Evict lowest-fee transaction
            if !self.evict_lowest_fee() {
                return Err(MempoolError::PoolFull("Cannot evict".into()));
            }
        }

        let fee = tx.fee;
        let sender_hash = Blake3Hash::compute(&tx.sender.0);

        self.known.insert(hash);
        self.by_sender.entry(sender_hash).or_default().push(hash);
        self.by_fee.entry(std::cmp::Reverse(fee)).or_default().push(hash);
        self.transactions.insert(hash, tx);

        Ok(hash)
    }

    /// Remove a transaction by hash.
    pub fn remove_transaction(&mut self, hash: &Blake3Hash) -> Option<Transaction> {
        if let Some(tx) = self.transactions.remove(hash) {
            self.known.remove(hash);

            let sender_hash = Blake3Hash::compute(&tx.sender.0);
            if let Some(sender_txs) = self.by_sender.get_mut(&sender_hash) {
                sender_txs.retain(|h| h != hash);
            }

            let fee_key = std::cmp::Reverse(tx.fee);
            if let Some(fee_txs) = self.by_fee.get_mut(&fee_key) {
                fee_txs.retain(|h| h != hash);
                if fee_txs.is_empty() {
                    self.by_fee.remove(&fee_key);
                }
            }

            Some(tx)
        } else {
            None
        }
    }

    /// Get up to `n` highest-fee transactions for block inclusion.
    pub fn get_top_transactions(&self, n: usize) -> Vec<&Transaction> {
        let mut result = Vec::new();
        for hashes in self.by_fee.values() {
            for hash in hashes {
                if result.len() >= n {
                    return result;
                }
                if let Some(tx) = self.transactions.get(hash) {
                    result.push(tx);
                }
            }
        }
        result
    }

    /// Evict the lowest-fee transaction.
    fn evict_lowest_fee(&mut self) -> bool {
        if let Some((_, hashes)) = self.by_fee.iter().next_back() {
            if let Some(hash) = hashes.first().copied() {
                return self.remove_transaction(&hash).is_some();
            }
        }
        false
    }

    /// Number of transactions in the pool.
    pub fn len(&self) -> usize {
        self.transactions.len()
    }

    /// Check if pool is empty.
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Check if a transaction is in the pool.
    pub fn contains(&self, hash: &Blake3Hash) -> bool {
        self.known.contains(hash)
    }

    /// Clear all transactions.
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.by_fee.clear();
        self.by_sender.clear();
        self.known.clear();
    }

    /// Remove transactions included in a finalized block.
    pub fn remove_finalized(&mut self, tx_hashes: &[Blake3Hash]) {
        for hash in tx_hashes {
            self.remove_transaction(hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_core::transaction::Transaction;
    use nexara_crypto::WalletAddress;

    fn make_tx(fee: u128) -> Transaction {
        Transaction::new_transfer(
            WalletAddress::zero(),
            WalletAddress::zero(),
            1000,
            fee,
            0,
            0,
        )
    }

    #[test]
    fn test_add_transaction() {
        let mut pool = Mempool::new(100, 0);
        let tx = make_tx(100);
        assert!(pool.add_transaction(tx).is_ok());
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_duplicate_rejection() {
        let mut pool = Mempool::new(100, 0);
        let tx = make_tx(100);
        let _hash = pool.add_transaction(tx.clone()).unwrap();
        assert!(pool.add_transaction(tx).is_err());
    }

    #[test]
    fn test_fee_too_low() {
        let mut pool = Mempool::new(100, 50);
        let tx = make_tx(10);
        assert!(pool.add_transaction(tx).is_err());
    }

    #[test]
    fn test_get_top_transactions() {
        let mut pool = Mempool::new(100, 0);
        pool.add_transaction(make_tx(10)).unwrap();
        pool.add_transaction(make_tx(100)).unwrap();
        pool.add_transaction(make_tx(50)).unwrap();
        let top = pool.get_top_transactions(2);
        assert_eq!(top.len(), 2);
        // Should be highest-fee first
        assert!(top[0].fee >= top[1].fee);
    }

    #[test]
    fn test_pool_full_eviction() {
        let mut pool = Mempool::new(2, 0);
        pool.add_transaction(make_tx(10)).unwrap();
        pool.add_transaction(make_tx(20)).unwrap();
        // Pool is full, adding a high-fee tx should evict the lowest
        pool.add_transaction(make_tx(30)).unwrap();
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_remove_finalized() {
        let mut pool = Mempool::new(100, 0);
        let tx = make_tx(100);
        let hash = pool.add_transaction(tx).unwrap();
        pool.remove_finalized(&[hash]);
        assert!(pool.is_empty());
    }
}
