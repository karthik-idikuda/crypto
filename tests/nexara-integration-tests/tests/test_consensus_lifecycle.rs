//! Integration test: Consensus lifecycle
//! Validator setup → committee election → block proposal → attestations → quorum → finalization

use nexara_crypto::{KeyPair, Blake3Hash, MlDsaSignature};
use nexara_core::{Block, BlockHeader};
use nexara_consensus::{
    Validator, ValidatorSet, HybridSyncEngine, Attestation,
    ConsensusResult, SlashOffense,
    SlashProposal, SlashEvidence,
};
use nexara_consensus::committee::elect_committee;
use nexara_consensus::slashing::{calculate_slash_amount, validate_slash_evidence, create_slash_event};

fn make_validator(stake: u128) -> Validator {
    let kp = KeyPair::generate();
    let addr = kp.public.wallet_address();
    Validator::new(addr, kp.public, stake)
}

fn test_block(height: u64, shard_id: u16) -> Block {
    let header = BlockHeader {
        version: 1,
        shard_id,
        height,
        timestamp: 1_700_000_000 + height,
        parent_hash: Blake3Hash::zero(),
        state_root: Blake3Hash::zero(),
        tx_root: Blake3Hash::zero(),
        validator_set_hash: Blake3Hash::zero(),
        proposer_address: nexara_crypto::WalletAddress::zero(),
        zk_finality_proof: vec![],
        attestation_bitfield: vec![],
    };
    Block::new(header, vec![])
}

#[test]
fn test_validator_set_creation() {
    let mut vs = ValidatorSet::new();
    assert!(vs.is_empty());

    let v1 = make_validator(100_000);
    let v2 = make_validator(200_000);
    let v1_addr = v1.address;

    vs.add_validator(v1);
    vs.add_validator(v2);

    assert_eq!(vs.len(), 2);
    assert!(vs.get_validator(&v1_addr).is_some());
    assert_eq!(vs.total_active_stake(), 300_000);
}

#[test]
fn test_validator_active_status() {
    let v = make_validator(50_000);
    assert!(v.is_active());
}

#[test]
fn test_committee_election() {
    let mut vs = ValidatorSet::new();
    for _ in 0..50 {
        vs.add_validator(make_validator(100_000));
    }

    // Try multiple seeds to account for probabilistic selection
    let mut found_non_empty = false;
    for i in 0..10u64 {
        let seed = Blake3Hash::compute(format!("epoch-seed-{i}").as_bytes());
        let committee = elect_committee(&vs, 1, 0, &seed, 25);
        if !committee.members.is_empty() {
            found_non_empty = true;
            assert!(committee.total_committee_stake > 0);
            break;
        }
    }
    assert!(found_non_empty, "At least one seed should produce a non-empty committee");
}

#[test]
fn test_hybridsync_propose_and_attest() {
    // Create validator set with 4 validators
    let mut vs = ValidatorSet::new();
    let mut keypairs: Vec<(KeyPair, nexara_crypto::WalletAddress)> = Vec::new();
    for _ in 0..4 {
        let kp = KeyPair::generate();
        let addr = kp.public.wallet_address();
        let v = Validator::new(addr, kp.public.clone(), 100_000);
        vs.add_validator(v);
        keypairs.push((kp, addr));
    }

    let mut engine = HybridSyncEngine::new(vs, 67);

    // Propose a block
    let block = test_block(1, 0);
    let block_hash = block.hash();
    engine.propose_block(block).expect("propose should succeed");

    // Submit attestations from 3 of 4 validators (75% > 67% threshold)
    for (_, addr) in keypairs.iter().take(3) {
        let attestation = Attestation {
            validator: *addr,
            block_hash,
            epoch: 0,
            round: 0,
            signature: MlDsaSignature(vec![0u8; 3309]),
            timestamp: 1_700_000_001,
        };
        engine.submit_attestation(attestation).expect("attest should succeed");
    }

    // Check quorum
    let qs = engine.check_quorum();
    assert!(qs.is_met(), "Quorum should be met with 75% attestation");

    // Finalize
    let result = engine.finalize_block();
    match result {
        ConsensusResult::Finalized(finalized_block) => {
            assert_eq!(finalized_block.header.height, 1);
        }
        other => panic!("Expected Finalized, got {:?}", other),
    }
}

#[test]
fn test_hybridsync_quorum_not_met() {
    let mut vs = ValidatorSet::new();
    let mut keypairs = Vec::new();
    for _ in 0..4 {
        let kp = KeyPair::generate();
        let addr = kp.public.wallet_address();
        vs.add_validator(Validator::new(addr, kp.public.clone(), 100_000));
        keypairs.push((kp, addr));
    }

    let mut engine = HybridSyncEngine::new(vs, 67);
    let block = test_block(1, 0);
    let block_hash = block.hash();
    engine.propose_block(block).expect("propose");

    // Only 1 of 4 attestations (25% < 67%)
    let attestation = Attestation {
        validator: keypairs[0].1,
        block_hash,
        epoch: 0,
        round: 0,
        signature: MlDsaSignature(vec![0u8; 3309]),
        timestamp: 1_700_000_001,
    };
    engine.submit_attestation(attestation).expect("attest");

    let qs = engine.check_quorum();
    assert!(!qs.is_met(), "Quorum should NOT be met with 25%");

    let result = engine.finalize_block();
    if let ConsensusResult::Finalized(_) = result {
        panic!("Should not finalize without quorum");
    }
}

#[test]
fn test_epoch_advancement() {
    let mut vs = ValidatorSet::new();
    vs.add_validator(make_validator(100_000));

    let mut engine = HybridSyncEngine::new(vs, 67);
    assert_eq!(engine.epoch, 0);

    engine.advance_epoch();
    assert_eq!(engine.epoch, 1);
    assert_eq!(engine.round, 0);
}

#[test]
fn test_slashing_calculation() {
    // Double-sign: 10% of stake
    let amount = calculate_slash_amount(SlashOffense::DoubleSign, 1_000_000);
    assert!(amount > 0, "Slash amount should be > 0");

    // Unavailability: smaller penalty
    let unavail_amount = calculate_slash_amount(SlashOffense::Unavailability, 1_000_000);
    assert!(unavail_amount > 0);
}

#[test]
fn test_slash_evidence_validation() {
    let kp = KeyPair::generate();
    let offender_addr = kp.public.wallet_address();
    let proposer_addr = KeyPair::generate().public.wallet_address();

    let proposal = SlashProposal {
        proposer: proposer_addr,
        offender: offender_addr,
        offense: SlashOffense::DoubleSign,
        evidence: SlashEvidence::DoubleSign {
            block_hash_a: Blake3Hash::compute(b"block-a"),
            block_hash_b: Blake3Hash::compute(b"block-b"),
            height: 100,
        },
        evidence_hash: Blake3Hash::compute(b"evidence"),
        block_height: 100,
    };

    let valid = validate_slash_evidence(&proposal);
    assert!(valid, "Double-sign evidence with different hashes should be valid");

    let slash_amount = calculate_slash_amount(proposal.offense, 500_000);
    let event = create_slash_event(&proposal, slash_amount);
    assert_eq!(event.offense, SlashOffense::DoubleSign);
    assert!(event.amount_slashed > 0);
}

#[test]
fn test_validator_set_hash_deterministic() {
    let mut vs = ValidatorSet::new();
    vs.add_validator(make_validator(100_000));
    vs.add_validator(make_validator(200_000));

    let h1 = vs.set_hash();
    let h2 = vs.set_hash();
    assert_eq!(h1, h2, "Same set should produce same hash");
}
