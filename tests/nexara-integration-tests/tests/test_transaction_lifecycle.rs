//! Integration test: Full transaction lifecycle
//! Create keys → build transaction → sign → add to mempool → build block → verify

use nexara_crypto::{KeyPair, Signer, Blake3Hash, WalletAddress};
use nexara_core::{Transaction, Block, BlockHeader, ChainState, AccountState};
use nexara_mempool::Mempool;

fn make_keypair_and_address() -> (KeyPair, WalletAddress) {
    let kp = KeyPair::generate();
    let addr = kp.public.wallet_address();
    (kp, addr)
}

fn genesis_header(shard_id: u16) -> BlockHeader {
    BlockHeader {
        version: 1,
        shard_id,
        height: 0,
        timestamp: 1_700_000_000,
        parent_hash: Blake3Hash::zero(),
        state_root: Blake3Hash::zero(),
        tx_root: Blake3Hash::zero(),
        validator_set_hash: Blake3Hash::zero(),
        proposer_address: WalletAddress::zero(),
        zk_finality_proof: vec![],
        attestation_bitfield: vec![],
    }
}

#[test]
fn test_create_and_sign_transaction() {
    let (sender_kp, sender_addr) = make_keypair_and_address();
    let (_recip_kp, recip_addr) = make_keypair_and_address();

    let tx = Transaction::new_transfer(sender_addr, recip_addr, 1_000, 10, 0, 0);
    assert_eq!(tx.sender, sender_addr);
    assert_eq!(tx.recipient, recip_addr);
    assert_eq!(tx.amount, 1_000);
    assert_eq!(tx.fee, 10);
    assert_eq!(tx.nonce, 0);
    assert_eq!(tx.shard_id, 0);

    let tx_hash = tx.hash();

    let signer = Signer::new(sender_kp.clone());
    let sig = signer.sign_transaction(&tx_hash);
    let result = Signer::verify(&sender_kp.public, tx_hash.as_bytes(), &sig);
    assert!(result.valid, "Signature verification must succeed");
    assert_eq!(result.signer_address, sender_addr);
}

#[test]
fn test_transaction_serialization_roundtrip() {
    let (_, sender_addr) = make_keypair_and_address();
    let (_, recip_addr) = make_keypair_and_address();

    let tx = Transaction::new_transfer(sender_addr, recip_addr, 50_000, 100, 1, 5);
    let bytes = tx.serialize();
    let tx2 = Transaction::deserialize(&bytes).expect("deserialization should succeed");

    assert_eq!(tx.sender, tx2.sender);
    assert_eq!(tx.recipient, tx2.recipient);
    assert_eq!(tx.amount, tx2.amount);
    assert_eq!(tx.fee, tx2.fee);
    assert_eq!(tx.nonce, tx2.nonce);
    assert_eq!(tx.shard_id, tx2.shard_id);
}

#[test]
fn test_mempool_add_and_retrieve() {
    let (_, sender_addr) = make_keypair_and_address();
    let (_, recip_addr) = make_keypair_and_address();

    let mut mempool = Mempool::new(1000, 1);

    let tx1 = Transaction::new_transfer(sender_addr, recip_addr, 100, 50, 0, 0);
    let tx2 = Transaction::new_transfer(sender_addr, recip_addr, 200, 100, 1, 0);
    let tx3 = Transaction::new_transfer(sender_addr, recip_addr, 300, 25, 2, 0);

    let h1 = mempool.add_transaction(tx1).expect("add tx1");
    let h2 = mempool.add_transaction(tx2).expect("add tx2");
    let h3 = mempool.add_transaction(tx3).expect("add tx3");

    assert_eq!(mempool.len(), 3);
    assert!(mempool.contains(&h1));
    assert!(mempool.contains(&h2));
    assert!(mempool.contains(&h3));

    // Top transactions ordered by fee descending
    let top = mempool.get_top_transactions(2);
    assert_eq!(top.len(), 2);
    assert!(top[0].fee >= top[1].fee, "Should be ordered by fee descending");
}

#[test]
fn test_mempool_remove_finalized() {
    let (_, sender_addr) = make_keypair_and_address();
    let (_, recip_addr) = make_keypair_and_address();

    let mut mempool = Mempool::new(1000, 1);

    let tx = Transaction::new_transfer(sender_addr, recip_addr, 100, 50, 0, 0);
    let hash = mempool.add_transaction(tx).expect("add tx");

    assert_eq!(mempool.len(), 1);
    mempool.remove_finalized(&[hash]);
    assert_eq!(mempool.len(), 0);
}

#[test]
fn test_block_construction_and_verification() {
    let (_, sender_addr) = make_keypair_and_address();
    let (_, recip_addr) = make_keypair_and_address();
    let (_, proposer_addr) = make_keypair_and_address();

    let tx = Transaction::new_transfer(sender_addr, recip_addr, 1_000, 10, 0, 0);
    let txs = vec![tx];

    let header = BlockHeader {
        version: 1,
        shard_id: 0,
        height: 1,
        timestamp: 1_700_000_001,
        parent_hash: genesis_header(0).hash(),
        state_root: Blake3Hash::zero(),
        tx_root: Blake3Hash::zero(),
        validator_set_hash: Blake3Hash::zero(),
        proposer_address: proposer_addr,
        zk_finality_proof: vec![],
        attestation_bitfield: vec![],
    };

    let block = Block::new(header, txs);
    assert_eq!(block.tx_count(), 1);
    assert_eq!(block.header.height, 1);
    assert!(!block.is_genesis());

    let block_hash = block.hash();
    assert_ne!(block_hash, Blake3Hash::zero());
}

#[test]
fn test_block_serialization_roundtrip() {
    let (_, sender_addr) = make_keypair_and_address();
    let (_, recip_addr) = make_keypair_and_address();

    let tx = Transaction::new_transfer(sender_addr, recip_addr, 500, 5, 0, 0);
    let header = BlockHeader {
        version: 1,
        shard_id: 0,
        height: 1,
        timestamp: 1_700_000_001,
        parent_hash: Blake3Hash::zero(),
        state_root: Blake3Hash::zero(),
        tx_root: Blake3Hash::zero(),
        validator_set_hash: Blake3Hash::zero(),
        proposer_address: WalletAddress::zero(),
        zk_finality_proof: vec![],
        attestation_bitfield: vec![],
    };

    let block = Block::new(header, vec![tx]);
    let bytes = block.serialize();
    let block2 = Block::deserialize(&bytes).expect("deserialize");
    assert_eq!(block.tx_count(), block2.tx_count());
    assert_eq!(block.header.height, block2.header.height);
}

#[test]
fn test_chain_state_transfers() {
    let (_, alice) = make_keypair_and_address();
    let (_, bob) = make_keypair_and_address();

    let mut state = ChainState::new();
    state.set_account(alice, AccountState::new(10_000));
    state.set_account(bob, AccountState::new(0));

    assert_eq!(state.balance_of(&alice), 10_000);
    assert_eq!(state.balance_of(&bob), 0);

    state.transfer(&alice, &bob, 3_000).expect("transfer should succeed");

    assert_eq!(state.balance_of(&alice), 7_000);
    assert_eq!(state.balance_of(&bob), 3_000);
}

#[test]
fn test_chain_state_insufficient_balance() {
    let (_, alice) = make_keypair_and_address();
    let (_, bob) = make_keypair_and_address();

    let mut state = ChainState::new();
    state.set_account(alice, AccountState::new(100));

    let result = state.transfer(&alice, &bob, 200);
    assert!(result.is_err(), "Should fail when balance insufficient");
}

#[test]
fn test_full_lifecycle_keys_to_block() {
    // End-to-end: generate keys, create tx, add to mempool, build block
    let (sender_kp, sender_addr) = make_keypair_and_address();
    let (_, recip_addr) = make_keypair_and_address();
    let (_, proposer_addr) = make_keypair_and_address();

    // Sign a transaction
    let tx = Transaction::new_transfer(sender_addr, recip_addr, 5_000, 50, 0, 0);
    let signer = Signer::new(sender_kp.clone());
    let sig = signer.sign_transaction(&tx.hash());

    // Verify signature
    let vr = Signer::verify(&sender_kp.public, tx.hash().as_bytes(), &sig);
    assert!(vr.valid);

    // Add to mempool
    let mut mempool = Mempool::new(1000, 1);
    let _tx_hash = mempool.add_transaction(tx).expect("add to mempool");
    assert_eq!(mempool.len(), 1);

    // Retrieve and build block
    let top_txs: Vec<Transaction> = mempool.get_top_transactions(10).into_iter().cloned().collect();

    let header = BlockHeader {
        version: 1,
        shard_id: 0,
        height: 1,
        timestamp: 1_700_000_001,
        parent_hash: Blake3Hash::zero(),
        state_root: Blake3Hash::zero(),
        tx_root: Blake3Hash::zero(),
        validator_set_hash: Blake3Hash::zero(),
        proposer_address: proposer_addr,
        zk_finality_proof: vec![],
        attestation_bitfield: vec![],
    };

    let block = Block::new(header, top_txs);
    assert_eq!(block.tx_count(), 1);
    assert!(!block.hash().as_bytes().iter().all(|&b| b == 0));
}
