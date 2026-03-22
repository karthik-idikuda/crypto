//! Transaction ordering strategies.

use nexara_core::transaction::Transaction;

/// Transaction ordering methods.
pub enum TransactionOrdering {
    /// Order by fee (highest first).
    ByFee,
    /// Order by nonce (lowest first per sender).
    ByNonce,
    /// First-in-first-out.
    Fifo,
}

/// Sort transactions by fee descending.
pub fn sort_by_fee(txs: &mut [Transaction]) {
    txs.sort_by(|a, b| b.fee.cmp(&a.fee));
}

/// Sort transactions by nonce ascending.  
pub fn sort_by_nonce(txs: &mut [Transaction]) {
    txs.sort_by(|a, b| a.nonce.cmp(&b.nonce));
}

/// Group transactions by sender, sorted by nonce within each group.
pub fn group_by_sender(txs: &[Transaction]) -> Vec<Vec<&Transaction>> {
    use std::collections::HashMap;
    let mut groups: HashMap<_, Vec<&Transaction>> = HashMap::new();
    for tx in txs {
        groups.entry(tx.sender).or_default().push(tx);
    }
    let mut result: Vec<Vec<&Transaction>> = groups.into_values().collect();
    for group in &mut result {
        group.sort_by(|a, b| a.nonce.cmp(&b.nonce));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_core::transaction::Transaction;
    use nexara_crypto::WalletAddress;

    fn make_tx(fee: u128, nonce: u64) -> Transaction {
        Transaction::new_transfer(
            WalletAddress::zero(),
            WalletAddress::zero(),
            1000,
            fee,
            nonce,
            0,
        )
    }

    #[test]
    fn test_sort_by_fee() {
        let mut txs = vec![make_tx(10, 0), make_tx(50, 1), make_tx(30, 2)];
        sort_by_fee(&mut txs);
        assert!(txs[0].fee >= txs[1].fee);
        assert!(txs[1].fee >= txs[2].fee);
    }

    #[test]
    fn test_sort_by_nonce() {
        let mut txs = vec![make_tx(10, 3), make_tx(50, 1), make_tx(30, 2)];
        sort_by_nonce(&mut txs);
        assert!(txs[0].nonce <= txs[1].nonce);
        assert!(txs[1].nonce <= txs[2].nonce);
    }

    #[test]
    fn test_group_by_sender() {
        let txs = vec![make_tx(10, 0), make_tx(20, 1), make_tx(30, 2)];
        let groups = group_by_sender(&txs);
        // All from WalletAddress::zero() so should be 1 group
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }
}
