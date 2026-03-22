//! Chain state management for the NEXARA blockchain.
//!
//! Tracks account balances, nonces, contract storage, and computes state roots.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use nexara_crypto::{Blake3Hash, WalletAddress};

/// State of a single account on the NEXARA blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    /// Account balance in base units (10^18 per NXR).
    pub balance: u128,
    /// Next expected transaction nonce.
    pub nonce: u64,
    /// If this account is a contract, its code hash.
    pub code_hash: Option<Blake3Hash>,
    /// Contract storage root (Merkle root of storage trie).
    pub storage_root: Blake3Hash,
}

impl AccountState {
    /// Create a new account with a given balance.
    pub fn new(balance: u128) -> Self {
        AccountState {
            balance,
            nonce: 0,
            code_hash: None,
            storage_root: Blake3Hash::zero(),
        }
    }

    /// Create a new contract account.
    pub fn new_contract(balance: u128, code_hash: Blake3Hash) -> Self {
        AccountState {
            balance,
            nonce: 0,
            code_hash: Some(code_hash),
            storage_root: Blake3Hash::zero(),
        }
    }

    /// Check if this account is a smart contract.
    pub fn is_contract(&self) -> bool {
        self.code_hash.is_some()
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// The complete chain state (all accounts).
pub struct ChainState {
    accounts: HashMap<WalletAddress, AccountState>,
}

/// Statistics about the chain state.
#[derive(Debug, Clone)]
pub struct StateStats {
    pub total_accounts: usize,
    pub total_balance: u128,
    pub contract_count: usize,
}

impl ChainState {
    /// Create a new empty chain state.
    pub fn new() -> Self {
        ChainState {
            accounts: HashMap::new(),
        }
    }

    /// Get the state of an account.
    pub fn get_account(&self, address: &WalletAddress) -> Option<&AccountState> {
        self.accounts.get(address)
    }

    /// Get or create account (returns mutable reference).
    pub fn get_or_create_account(&mut self, address: WalletAddress) -> &mut AccountState {
        self.accounts.entry(address).or_default()
    }

    /// Set the state of an account.
    pub fn set_account(&mut self, address: WalletAddress, state: AccountState) {
        self.accounts.insert(address, state);
    }

    /// Get the balance of an account (0 if not found).
    pub fn balance_of(&self, address: &WalletAddress) -> u128 {
        self.accounts.get(address).map(|a| a.balance).unwrap_or(0)
    }

    /// Get the nonce of an account (0 if not found).
    pub fn nonce_of(&self, address: &WalletAddress) -> u64 {
        self.accounts.get(address).map(|a| a.nonce).unwrap_or(0)
    }

    /// Transfer funds between accounts.
    ///
    /// SECURITY: Checks for insufficient balance and overflow.
    pub fn transfer(
        &mut self,
        from: &WalletAddress,
        to: &WalletAddress,
        amount: u128,
    ) -> Result<(), crate::error::CoreError> {
        let sender_balance = self.balance_of(from);
        if sender_balance < amount {
            return Err(crate::error::CoreError::InsufficientBalance {
                have: sender_balance,
                need: amount,
            });
        }

        // Debit sender
        let sender = self.get_or_create_account(*from);
        sender.balance = sender.balance.checked_sub(amount)
            .ok_or(crate::error::CoreError::InsufficientBalance {
                have: sender_balance,
                need: amount,
            })?;

        // Credit recipient
        let recipient = self.get_or_create_account(*to);
        recipient.balance = recipient.balance.checked_add(amount)
            .ok_or_else(|| crate::error::CoreError::InvalidTransaction(
                "balance overflow".into(),
            ))?;

        Ok(())
    }

    /// Compute the state root (Merkle root of all account states).
    pub fn state_root(&self) -> Blake3Hash {
        if self.accounts.is_empty() {
            return Blake3Hash::zero();
        }

        // Sort by address for deterministic ordering
        let mut sorted: Vec<(&WalletAddress, &AccountState)> = self.accounts.iter().collect();
        sorted.sort_by_key(|(addr, _)| addr.0);

        let hashes: Vec<Blake3Hash> = sorted
            .iter()
            .map(|(addr, state)| {
                let mut data = Vec::new();
                data.extend_from_slice(&addr.0);
                data.extend_from_slice(&state.balance.to_le_bytes());
                data.extend_from_slice(&state.nonce.to_le_bytes());
                Blake3Hash::compute(&data)
            })
            .collect();

        crate::block::calculate_merkle_root(&hashes)
    }

    /// Get statistics about the current state.
    pub fn stats(&self) -> StateStats {
        let total_balance: u128 = self.accounts.values().map(|a| a.balance).sum();
        let contract_count = self.accounts.values().filter(|a| a.is_contract()).count();
        StateStats {
            total_accounts: self.accounts.len(),
            total_balance,
            contract_count,
        }
    }

    /// Number of accounts in the state.
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }
}

impl Default for ChainState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexara_crypto::keys::KeyPair;

    #[test]
    fn test_new_account() {
        let acc = AccountState::new(1000);
        assert_eq!(acc.balance, 1000);
        assert_eq!(acc.nonce, 0);
        assert!(!acc.is_contract());
    }

    #[test]
    fn test_contract_account() {
        let code_hash = Blake3Hash::compute(b"contract code");
        let acc = AccountState::new_contract(0, code_hash);
        assert!(acc.is_contract());
    }

    #[test]
    fn test_chain_state_transfer() {
        let mut state = ChainState::new();
        let alice = KeyPair::generate().public.wallet_address();
        let bob = KeyPair::generate().public.wallet_address();

        state.set_account(alice, AccountState::new(1000));
        state.set_account(bob, AccountState::new(0));

        state.transfer(&alice, &bob, 400).unwrap();
        assert_eq!(state.balance_of(&alice), 600);
        assert_eq!(state.balance_of(&bob), 400);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut state = ChainState::new();
        let alice = KeyPair::generate().public.wallet_address();
        let bob = KeyPair::generate().public.wallet_address();

        state.set_account(alice, AccountState::new(100));
        assert!(state.transfer(&alice, &bob, 200).is_err());
    }

    #[test]
    fn test_state_root_deterministic() {
        let mut state = ChainState::new();
        let addr = KeyPair::generate().public.wallet_address();
        state.set_account(addr, AccountState::new(1000));
        let r1 = state.state_root();
        let r2 = state.state_root();
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_state_stats() {
        let mut state = ChainState::new();
        let addr1 = KeyPair::generate().public.wallet_address();
        let addr2 = KeyPair::generate().public.wallet_address();
        state.set_account(addr1, AccountState::new(500));
        state.set_account(addr2, AccountState::new_contract(300, Blake3Hash::zero()));

        let stats = state.stats();
        assert_eq!(stats.total_accounts, 2);
        assert_eq!(stats.total_balance, 800);
        assert_eq!(stats.contract_count, 1);
    }
}
