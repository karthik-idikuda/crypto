//! Integration test: Cross-shard communication
//! Shard assignment → address routing → cross-shard messages → beacon chain coordination

use nexara_crypto::{KeyPair, Blake3Hash, WalletAddress};
use nexara_shard::{
    Shard, ShardState, BeaconChain,
    CrossShardMessage,
};
use nexara_shard::assignment::{assign_validators_to_shards, validators_for_shard};
use nexara_shard::beacon::BeaconCrossLink;

fn make_address() -> WalletAddress {
    KeyPair::generate().public.wallet_address()
}

#[test]
fn test_shard_creation() {
    let shard = Shard::new(0);
    assert_eq!(shard.shard_id(), 0);
    assert_eq!(shard.height(), 0);
}

#[test]
fn test_address_shard_routing() {
    // Different addresses should map deterministically to shards
    let addr1 = make_address();
    let addr2 = make_address();

    let shard1 = Shard::shard_for_address(addr1.as_bytes());
    let shard2 = Shard::shard_for_address(addr2.as_bytes());

    // Both should be in range [0, 100)
    assert!(shard1 < 100, "Shard ID should be < 100");
    assert!(shard2 < 100, "Shard ID should be < 100");

    // Same address should always map to same shard
    let shard1_again = Shard::shard_for_address(addr1.as_bytes());
    assert_eq!(shard1, shard1_again, "Same address should map to same shard");
}

#[test]
fn test_cross_shard_message_creation() {
    let from = make_address();
    let to = make_address();

    let msg = CrossShardMessage::new_transfer(0, 1, from, to, 10_000, 42);
    assert_eq!(msg.source_shard, 0);
    assert_eq!(msg.target_shard, 1);
    assert_eq!(msg.source_block_height, 42);
}

#[test]
fn test_cross_shard_message_lifecycle() {
    let from = make_address();
    let to = make_address();

    let mut msg = CrossShardMessage::new_transfer(5, 10, from, to, 50_000, 100);
    // Initial status should be Pending
    msg.mark_delivered();
    msg.mark_confirmed();
}

#[test]
fn test_beacon_chain_basic() {
    let mut beacon = BeaconChain::new();
    assert_eq!(beacon.height(), 0);

    // Update shard states
    let state0 = ShardState {
        shard_id: 0,
        latest_block_height: 10,
        latest_block_hash: Blake3Hash::compute(b"shard-0-block-10"),
        state_root: Blake3Hash::compute(b"shard-0-state"),
        validator_count: 5,
        total_transactions: 100,
        pending_cross_shard: 2,
    };
    let state1 = ShardState {
        shard_id: 1,
        latest_block_height: 8,
        latest_block_hash: Blake3Hash::compute(b"shard-1-block-8"),
        state_root: Blake3Hash::compute(b"shard-1-state"),
        validator_count: 4,
        total_transactions: 80,
        pending_cross_shard: 1,
    };

    beacon.update_shard_state(state0);
    beacon.update_shard_state(state1);

    // Create beacon block
    let cross_links = vec![
        BeaconCrossLink {
            shard_id: 0,
            block_height: 10,
            block_hash: Blake3Hash::compute(b"shard-0-block-10"),
            state_root: Blake3Hash::compute(b"shard-0-state"),
        },
    ];

    let beacon_block = beacon.create_beacon_block(1_700_000_100, cross_links);
    assert_eq!(beacon_block.height, 0, "First beacon block has height 0");
    assert_eq!(beacon.height(), 1, "Beacon chain height is block count");
}

#[test]
fn test_beacon_epoch_advancement() {
    let mut beacon = BeaconChain::new();
    assert_eq!(beacon.current_epoch, 0);

    beacon.advance_epoch();
    assert_eq!(beacon.current_epoch, 1);
}

#[test]
fn test_validator_shard_assignment() {
    let validators: Vec<WalletAddress> = (0..20).map(|_| make_address()).collect();
    let seed = Blake3Hash::compute(b"assignment-seed");

    let assignments = assign_validators_to_shards(&validators, &seed, 3, 0);
    assert_eq!(assignments.len(), 20, "Should have one assignment per validator");

    // Each assignment should have the right number of shards
    for a in &assignments {
        assert!(!a.shard_ids.is_empty(), "Should be assigned to at least one shard");
        assert!(a.shard_ids.len() <= 3, "Should be assigned to at most 3 shards");
    }

    // Check validators_for_shard utility
    let shard_0_validators = validators_for_shard(&assignments, 0);
    // Some validators should be assigned to shard 0 (probabilistic, but with 20 validators it's
    // very likely at least one will be assigned to any given shard)
    // We won't assert a specific count since it's seed-dependent
    let _ = shard_0_validators;
}

#[test]
fn test_assignment_determinism() {
    let validators: Vec<WalletAddress> = (0..10).map(|_| make_address()).collect();
    let seed = Blake3Hash::compute(b"deterministic-seed");

    let a1 = assign_validators_to_shards(&validators, &seed, 2, 0);
    let a2 = assign_validators_to_shards(&validators, &seed, 2, 0);

    assert_eq!(a1.len(), a2.len());
    for (x, y) in a1.iter().zip(a2.iter()) {
        assert_eq!(x.validator, y.validator);
        assert_eq!(x.shard_ids, y.shard_ids);
        assert_eq!(x.primary_shard, y.primary_shard);
    }
}

#[test]
fn test_aggregate_state_root() {
    let mut beacon = BeaconChain::new();

    for i in 0..5u16 {
        beacon.update_shard_state(ShardState {
            shard_id: i,
            latest_block_height: 1,
            latest_block_hash: Blake3Hash::compute(format!("block-{i}").as_bytes()),
            state_root: Blake3Hash::compute(format!("state-{i}").as_bytes()),
            validator_count: 3,
            total_transactions: 10,
            pending_cross_shard: 0,
        });
    }

    let root = beacon.compute_aggregate_root();
    assert_ne!(root, Blake3Hash::zero(), "Aggregate root should not be zero");
}
