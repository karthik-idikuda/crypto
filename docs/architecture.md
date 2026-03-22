# NEXARA Architecture Guide

## System Architecture

```
                    ┌──────────────────────────┐
                    │      nexara-node         │
                    │  (Binary Entry Point)     │
                    │  CLI / Config / RPC       │
                    └─────────┬────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
   ┌────▼────┐        ┌──────▼──────┐        ┌────▼────┐
   │ nexara- │        │  nexara-    │        │ nexara- │
   │ network │        │  consensus  │        │  shard  │
   │ (P2P)   │        │ (HybridSync)│        │ (100x)  │
   └────┬────┘        └──────┬──────┘        └────┬────┘
        │                     │                     │
        │              ┌──────▼──────┐              │
        │              │  nexara-    │              │
        └──────────────▶  mempool   ◀──────────────┘
                       └──────┬──────┘
                              │
                    ┌─────────▼─────────┐
                    │    nexara-core    │
                    │ (Tx, Block, State)│
                    └─────────┬─────────┘
                              │
                    ┌─────────▼─────────┐
                    │   nexara-crypto   │
                    │ (PQC, BLAKE3)     │
                    └───────────────────┘

  ┌──────────┐    ┌──────────┐    ┌──────────────┐    ┌──────────┐
  │ nexlang  │───▶│   nxvm   │    │nexara-bridge │    │ nexara-  │
  │(Compiler)│    │  (VM)    │    │   (NCCP)     │    │tokenomics│
  └──────────┘    └──────────┘    └──────────────┘    └──────────┘
```

## Crate Dependency Graph

```
nexara-crypto ──► (standalone, no internal deps)
     │
     ├──► nexara-core
     │         │
     │         ├──► nexara-consensus
     │         ├──► nexara-shard
     │         ├──► nexara-mempool
     │         ├──► nexara-tokenomics
     │         └──► nexara-bridge
     │
     ├──► nexlang
     ├──► nxvm
     │
     └──► nexara-node (depends on all crates)
```

## Crate Descriptions

### nexara-crypto
Post-quantum cryptographic primitives. Provides key generation (ML-DSA-65), signing/verification, KEM (ML-KEM-1024), BLAKE3 hashing, wallet management, and MPC key splitting.

### nexara-core
Core blockchain types: Transaction (7 types), Block (with Merkle trees), ChainState, and Genesis configuration. Used by all other crates.

### nexara-consensus
HybridSync consensus engine combining DPoQS, AIBFT, and ZK-Finality. Manages validators, committee selection, block proposal, attestation, and slashing.

### nexara-network
Peer-to-peer networking via libp2p. Handles gossip protocol, PQ handshake, peer discovery (XOR-distance), and message propagation.

### nexara-shard
100-shard architecture with beacon chain coordination. Manages shard assignment (BLAKE3-deterministic), cross-shard messaging, and validator distribution.

### nexara-mempool
Transaction pool with fee-priority ordering (BTreeMap), duplicate detection, sender grouping, eviction, and MEV protection (encrypted transactions, sandwich attack detection).

### nexlang
NEXARA's smart contract language. Full pipeline: Lexer → Parser (recursive descent) → Type Checker → Security Analyzer (6 checks) → Code Generator (NXVM bytecode).

### nxvm
NEXARA Virtual Machine. Executes compiled NEXLANG bytecode with 32-byte stack values, per-opcode gas metering, storage, memory, events, and BLAKE3 hashing.

### nexara-bridge
Cross-chain bridge protocol (NCCP). Supports Ethereum, BNB Chain, Solana, Cosmos (IBC), and Bitcoin (HTLC). Nullifier-based double-spend prevention.

### nexara-tokenomics
Token economics engine. Manages 500M NXR supply, 7 allocations, vesting schedules, staking pool (proof-of-stake rewards), deflationary burn (50% of fees), and Protocol-Owned Liquidity.

### nexara-node
Full node binary. CLI (clap), JSON configuration, RPC server, and main event loop with graceful shutdown.

## Data Flow

### Transaction Processing

```
1. User signs tx with PQC key (nexara-crypto)
2. Tx submitted via RPC (nexara-node)
3. Tx validated and added to mempool (nexara-mempool)
4. Validator proposes block with top txs (nexara-consensus)
5. Shard processes txs in parallel (nexara-shard)
6. Cross-shard messages routed via beacon (nexara-shard)
7. Block attested by committee (nexara-consensus)
8. Block finalized at 67% quorum (nexara-consensus)
9. State updated and fees processed (nexara-core, nexara-tokenomics)
```

### Smart Contract Deployment

```
1. Developer writes contract in NEXLANG (.nxl file)
2. Lexer tokenizes source (nexlang)
3. Parser builds AST (nexlang)
4. Type checker validates types (nexlang)
5. Security analyzer audits for vulnerabilities (nexlang)
6. Code generator emits NXVM bytecode (nexlang)
7. Bytecode deployed as transaction (nexara-core)
8. NXVM executes contract calls (nxvm)
```

## Configuration

Default node configuration:
- Chain ID: 20240101
- RPC Port: 9944
- P2P Port: 30333
- Shards: 100
- Block Time: 200ms
- Max Peers: 50
- Minimum Stake: 1,000 NXR
