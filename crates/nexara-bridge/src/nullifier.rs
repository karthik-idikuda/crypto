//! Nullifier set for preventing double-spending in bridge transfers.

use std::collections::HashSet;

/// A set of spent nullifiers to prevent replay/double-spend.
#[derive(Debug, Clone, Default)]
pub struct NullifierSet {
    spent: HashSet<[u8; 32]>,
}

impl NullifierSet {
    /// Create a new empty nullifier set.
    pub fn new() -> Self {
        NullifierSet { spent: HashSet::new() }
    }

    /// Check if a nullifier has been spent.
    pub fn contains(&self, nullifier: &[u8; 32]) -> bool {
        self.spent.contains(nullifier)
    }

    /// Mark a nullifier as spent. Returns false if already spent.
    pub fn insert(&mut self, nullifier: [u8; 32]) -> bool {
        self.spent.insert(nullifier)
    }

    /// Number of spent nullifiers.
    pub fn len(&self) -> usize {
        self.spent.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.spent.is_empty()
    }

    /// Generate a nullifier from transfer data.
    pub fn generate(source: &str, dest: &str, amount: u128, nonce: u64) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"NULLIFIER_V1");
        hasher.update(source.as_bytes());
        hasher.update(dest.as_bytes());
        hasher.update(&amount.to_le_bytes());
        hasher.update(&nonce.to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullifier_insert_contains() {
        let mut set = NullifierSet::new();
        let n = [42u8; 32];
        assert!(!set.contains(&n));
        assert!(set.insert(n));
        assert!(set.contains(&n));
        assert!(!set.insert(n)); // duplicate
    }

    #[test]
    fn test_nullifier_generate() {
        let n1 = NullifierSet::generate("src", "dst", 100, 1);
        let n2 = NullifierSet::generate("src", "dst", 100, 2);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_nullifier_deterministic() {
        let n1 = NullifierSet::generate("a", "b", 50, 0);
        let n2 = NullifierSet::generate("a", "b", 50, 0);
        assert_eq!(n1, n2);
    }

    #[test]
    fn test_empty() {
        let set = NullifierSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }
}
