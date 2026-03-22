//! Genesis block configuration for the NEXARA blockchain.
//!
//! Defines the initial state: supply allocation, validator set, and chain parameters.
//! Total supply: 500,000,000 NXR = 500_000_000 × 10^18 base units.

use serde::{Serialize, Deserialize};
use nexara_crypto::{Blake3Hash, WalletAddress};
use crate::block::{Block, BlockHeader, calculate_merkle_root};
use crate::transaction::{Transaction, TransactionType, ONE_NXR};
use crate::state::{ChainState, AccountState};
use nexara_crypto::keys::{MlDsaSignature, SIGNATURE_SIZE};

/// Total supply of NXR in base units (500M × 10^18).
pub const TOTAL_SUPPLY: u128 = 500_000_000 * ONE_NXR;

/// Blocks per year at 200ms block time.
/// 1 year = 365.25 days × 24h × 3600s / 0.2s ≈ 157,788,000
pub const BLOCKS_PER_YEAR: u64 = 157_788_000;

/// NEXARA mainnet chain ID.
pub const MAINNET_CHAIN_ID: u64 = 20240101;

/// Genesis configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chain ID (20240101 for mainnet).
    pub chain_id: u64,
    /// Network name.
    pub network_name: String,
    /// Initial validator set.
    pub initial_validators: Vec<GenesisValidator>,
    /// Token allocations.
    pub initial_allocations: Vec<GenesisAllocation>,
    /// Base fee per byte in base units.
    pub base_fee: u128,
    /// Maximum block size in bytes.
    pub max_block_size: usize,
    /// Number of shards (100).
    pub shard_count: u16,
    /// Block time in milliseconds (200).
    pub block_time_ms: u64,
    /// Target TPS per shard (2500).
    pub target_tps_per_shard: u32,
}

/// A genesis validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisValidator {
    pub address: WalletAddress,
    pub stake: u128,
    pub pubkey: Vec<u8>,
}

/// A genesis token allocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAllocation {
    pub address: WalletAddress,
    pub amount: u128,
    pub allocation_type: AllocationType,
    pub vesting_schedule: Option<VestingSchedule>,
}

/// Types of token allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationType {
    PublicSale,
    Ecosystem,
    Team,
    StakingRewards,
    DaoTreasury,
    Partners,
    Liquidity,
}

/// Vesting schedule for locked tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    /// Blocks before any tokens unlock.
    pub cliff_blocks: u64,
    /// Total vesting duration in blocks.
    pub total_blocks: u64,
    /// Start block (0 = genesis).
    pub start_block: u64,
}

/// Create the NEXARA mainnet genesis configuration.
///
/// Allocation breakdown:
///   100M NXR → Public Sale (no vesting)
///   125M NXR → Ecosystem (4 year linear from genesis)
///    50M NXR → Team (1 year cliff, then 4 year linear)
///   100M NXR → Staking Rewards (emitted by staking module)
///    75M NXR → DAO Treasury (DAO controlled)
///    25M NXR → Partners (2 year linear)
///    25M NXR → Liquidity (immediate)
pub fn create_genesis_config() -> GenesisConfig {
    // Deterministic genesis validator addresses from seeds
    let validators: Vec<GenesisValidator> = (0..4)
        .map(|i| {
            let mut seed = [0u8; 32];
            seed[0] = i;
            seed[31] = 0xAE_u8.wrapping_add(i); // Avoid collision
            let kp = nexara_crypto::keys::KeyPair::from_seed(&seed);
            GenesisValidator {
                address: kp.public.wallet_address(),
                stake: 1_000_000 * ONE_NXR, // 1M NXR each
                pubkey: kp.public.0.clone(),
            }
        })
        .collect();

    let four_years = BLOCKS_PER_YEAR * 4;
    let two_years = BLOCKS_PER_YEAR * 2;
    let one_year = BLOCKS_PER_YEAR;

    // Create deterministic addresses for each allocation pool
    let alloc_address = |tag: &[u8]| -> WalletAddress {
        let hash = Blake3Hash::compute(tag);
        WalletAddress(*hash.as_bytes())
    };

    let allocations = vec![
        GenesisAllocation {
            address: alloc_address(b"nexara-public-sale"),
            amount: 100_000_000 * ONE_NXR,
            allocation_type: AllocationType::PublicSale,
            vesting_schedule: None, // Immediate
        },
        GenesisAllocation {
            address: alloc_address(b"nexara-ecosystem"),
            amount: 125_000_000 * ONE_NXR,
            allocation_type: AllocationType::Ecosystem,
            vesting_schedule: Some(VestingSchedule {
                cliff_blocks: 0,
                total_blocks: four_years,
                start_block: 0,
            }),
        },
        GenesisAllocation {
            address: alloc_address(b"nexara-team"),
            amount: 50_000_000 * ONE_NXR,
            allocation_type: AllocationType::Team,
            vesting_schedule: Some(VestingSchedule {
                cliff_blocks: one_year,
                total_blocks: four_years,
                start_block: 0,
            }),
        },
        GenesisAllocation {
            address: alloc_address(b"nexara-staking-rewards"),
            amount: 100_000_000 * ONE_NXR,
            allocation_type: AllocationType::StakingRewards,
            vesting_schedule: None, // Controlled by staking module
        },
        GenesisAllocation {
            address: alloc_address(b"nexara-dao-treasury"),
            amount: 75_000_000 * ONE_NXR,
            allocation_type: AllocationType::DaoTreasury,
            vesting_schedule: None, // DAO-controlled
        },
        GenesisAllocation {
            address: alloc_address(b"nexara-partners"),
            amount: 25_000_000 * ONE_NXR,
            allocation_type: AllocationType::Partners,
            vesting_schedule: Some(VestingSchedule {
                cliff_blocks: 0,
                total_blocks: two_years,
                start_block: 0,
            }),
        },
        GenesisAllocation {
            address: alloc_address(b"nexara-liquidity"),
            amount: 25_000_000 * ONE_NXR,
            allocation_type: AllocationType::Liquidity,
            vesting_schedule: None, // Immediate
        },
    ];

    // Verify total allocation equals total supply
    let total: u128 = allocations.iter().map(|a| a.amount).sum();
    assert_eq!(total, TOTAL_SUPPLY, "Genesis allocations must sum to total supply");

    GenesisConfig {
        chain_id: MAINNET_CHAIN_ID,
        network_name: "nexara-mainnet".to_string(),
        initial_validators: validators,
        initial_allocations: allocations,
        base_fee: 1_000_000_000, // 1 gwei
        max_block_size: 2 * 1024 * 1024, // 2 MB
        shard_count: 100,
        block_time_ms: 200,
        target_tps_per_shard: 2500,
    }
}

/// Build the genesis block from a genesis configuration.
///
/// Creates block at height 0 with allocation transactions.
pub fn build_genesis_block(config: &GenesisConfig) -> Block {
    // Create allocation transactions
    let txs: Vec<Transaction> = config
        .initial_allocations
        .iter()
        .map(|alloc| Transaction {
            version: 1,
            tx_type: TransactionType::Transfer,
            sender: WalletAddress::zero(), // Minted from nowhere
            recipient: alloc.address,
            amount: alloc.amount,
            fee: 0,
            nonce: 0,
            shard_id: 0,
            data: Vec::new(),
            signature: MlDsaSignature(vec![0u8; SIGNATURE_SIZE]),
            pubkey_first_use: None,
            timestamp: 0,
        })
        .collect();

    let tx_hashes: Vec<Blake3Hash> = txs.iter().map(|tx| tx.hash()).collect();
    let tx_root = calculate_merkle_root(&tx_hashes);

    // Compute initial state root
    let mut state = ChainState::new();
    for alloc in &config.initial_allocations {
        state.set_account(alloc.address, AccountState::new(alloc.amount));
    }
    let state_root = state.state_root();

    let header = BlockHeader {
        version: 1,
        shard_id: 0,
        height: 0,
        timestamp: 0,
        parent_hash: Blake3Hash::zero(),
        state_root,
        tx_root,
        validator_set_hash: Blake3Hash::zero(),
        proposer_address: WalletAddress::zero(),
        zk_finality_proof: Vec::new(),
        attestation_bitfield: Vec::new(),
    };

    Block::new(header, txs)
}

/// Initialize chain state from genesis config.
pub fn init_chain_state(config: &GenesisConfig) -> ChainState {
    let mut state = ChainState::new();
    for alloc in &config.initial_allocations {
        state.set_account(alloc.address, AccountState::new(alloc.amount));
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_genesis_config() {
        let config = create_genesis_config();
        assert_eq!(config.chain_id, MAINNET_CHAIN_ID);
        assert_eq!(config.shard_count, 100);
        assert_eq!(config.block_time_ms, 200);
        assert_eq!(config.initial_validators.len(), 4);
        assert_eq!(config.initial_allocations.len(), 7);

        let total: u128 = config.initial_allocations.iter().map(|a| a.amount).sum();
        assert_eq!(total, TOTAL_SUPPLY);
    }

    #[test]
    fn test_build_genesis_block() {
        let config = create_genesis_config();
        let block = build_genesis_block(&config);
        assert!(block.is_genesis());
        assert_eq!(block.tx_count(), 7); // 7 allocation transactions
        assert!(block.verify_tx_root());
    }

    #[test]
    fn test_init_chain_state() {
        let config = create_genesis_config();
        let state = init_chain_state(&config);
        assert_eq!(state.account_count(), 7);
        let stats = state.stats();
        assert_eq!(stats.total_balance, TOTAL_SUPPLY);
    }

    #[test]
    fn test_allocation_types() {
        let config = create_genesis_config();
        let types: Vec<AllocationType> = config
            .initial_allocations
            .iter()
            .map(|a| a.allocation_type)
            .collect();
        assert!(types.contains(&AllocationType::PublicSale));
        assert!(types.contains(&AllocationType::Team));
        assert!(types.contains(&AllocationType::Ecosystem));
        assert!(types.contains(&AllocationType::StakingRewards));
        assert!(types.contains(&AllocationType::DaoTreasury));
        assert!(types.contains(&AllocationType::Partners));
        assert!(types.contains(&AllocationType::Liquidity));
    }

    #[test]
    fn test_vesting_schedules() {
        let config = create_genesis_config();
        // Public sale has no vesting
        let public_sale = config
            .initial_allocations
            .iter()
            .find(|a| a.allocation_type == AllocationType::PublicSale)
            .unwrap();
        assert!(public_sale.vesting_schedule.is_none());

        // Team has cliff + vesting
        let team = config
            .initial_allocations
            .iter()
            .find(|a| a.allocation_type == AllocationType::Team)
            .unwrap();
        let vs = team.vesting_schedule.as_ref().unwrap();
        assert_eq!(vs.cliff_blocks, BLOCKS_PER_YEAR);
        assert_eq!(vs.total_blocks, BLOCKS_PER_YEAR * 4);
    }
}
