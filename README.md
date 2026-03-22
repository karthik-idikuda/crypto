# Nexara Blockchain

A next-generation Layer-1 blockchain platform written in Rust, featuring adaptive sharding, a custom virtual machine (NXVM), a domain-specific smart contract language (NexLang), cross-chain bridging, and a tokenomics engine. Nexara is designed for high throughput, low latency, and horizontal scalability.

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Technology Stack](#technology-stack)
- [Workspace Structure](#workspace-structure)
- [Crate Descriptions](#crate-descriptions)
- [Building](#building)
- [Testing](#testing)
- [License](#license)

---

## Overview

Nexara is a modular blockchain platform built as a Rust workspace with 12 crates, each handling a distinct concern:

- **Consensus** -- Custom consensus protocol for block finalization
- **Sharding** -- Adaptive shard management for horizontal scaling
- **NexLang** -- Domain-specific language for writing smart contracts
- **NXVM** -- Custom virtual machine for contract execution
- **Bridge** -- Cross-chain interoperability layer
- **Tokenomics** -- Economic model, staking, and reward distribution
- **Mempool** -- Transaction queuing and prioritization
- **Networking** -- Peer-to-peer communication via libp2p

---

## Architecture

```
+-----------------------------------------------------------+
|                      nexara-node                          |
|  CLI entry point | Service orchestration | RPC server     |
+-----------------------------------------------------------+
        |         |          |           |          |
        v         v          v           v          v
+--------+ +----------+ +---------+ +--------+ +--------+
| nexara | | nexara   | | nexara  | | nexara | | nexara |
| -core  | | -consen  | | -shard  | | -net   | | -mem   |
|        | |  sus     | |         | |  work  | |  pool  |
| Blocks | | PoS/BFT  | | Dynamic | | libp2p | | Tx     |
| State  | | Finality | | Shards  | | P2P    | | Queue  |
| Crypto | |          | | Routing | | Gossip | |        |
+--------+ +----------+ +---------+ +--------+ +--------+
        |                     |
        v                     v
+-------------------+ +-------------------+
|     nexlang       | |    nexara-bridge  |
|  Smart Contract   | |  Cross-Chain      |
|  Language (DSL)   | |  Interoperability |
|  Parser + Compiler| |  Lock & Mint      |
+-------------------+ +-------------------+
        |
        v
+-------------------+
|       nxvm        |
|  Virtual Machine  |
|  Bytecode Exec    |
|  Gas Metering     |
+-------------------+

+-------------------------------------------+
|          nexara-tokenomics               |
|  Staking | Rewards | Supply Schedule     |
+-------------------------------------------+
```

---

## Technology Stack

| Component           | Technology                                        |
|---------------------|---------------------------------------------------|
| Language            | Rust (2021 edition)                               |
| Async Runtime       | Tokio (full features)                             |
| Serialization       | serde + serde_json + bincode                      |
| Cryptography        | blake3, sha3                                      |
| Networking          | libp2p (TCP + Noise + Yamux + Gossipsub + mDNS)   |
| Storage             | RocksDB                                           |
| CLI                 | clap 4.4                                          |
| Concurrency         | dashmap, rayon                                    |
| Benchmarking        | criterion                                         |
| Error Handling      | thiserror, anyhow                                 |
| Logging             | tracing + tracing-subscriber                      |

---

## Workspace Structure

```
nexara/
|
|-- Cargo.toml                    # Workspace root configuration
|-- Cargo.lock                    # Dependency lock file
|
|-- crates/
|   |-- nexara-core/              # Core blockchain primitives
|   |-- nexara-crypto/            # Cryptographic operations
|   |-- nexara-consensus/         # Consensus protocol
|   |-- nexara-network/           # P2P networking layer
|   |-- nexara-shard/             # Adaptive sharding engine
|   |-- nexara-mempool/           # Transaction mempool
|   |-- nexlang/                  # Smart contract language
|   |-- nxvm/                     # Virtual machine
|   |-- nexara-bridge/            # Cross-chain bridge
|   |-- nexara-tokenomics/        # Economic model
|   +-- nexara-node/              # Node binary and orchestration
|
|-- contracts/                    # Example NexLang contracts
|-- docs/                         # Technical documentation
|-- scripts/                      # Build and deployment scripts
+-- tests/
    +-- nexara-integration-tests/ # End-to-end integration tests
```

---

## Crate Descriptions

| Crate               | Purpose                                               |
|----------------------|-------------------------------------------------------|
| nexara-core          | Block structures, state management, transaction types |
| nexara-crypto        | Hashing (blake3, sha3), key generation, signatures    |
| nexara-consensus     | Block proposal, voting, finalization                  |
| nexara-network       | libp2p peer discovery, gossipsub message propagation  |
| nexara-shard         | Dynamic shard creation, routing, cross-shard messaging|
| nexara-mempool       | Transaction queuing, fee-based prioritization         |
| nexlang              | DSL parser, compiler, AST representation              |
| nxvm                 | Bytecode interpreter, gas metering, contract state    |
| nexara-bridge        | Cross-chain lock-and-mint, relay verification         |
| nexara-tokenomics    | Token supply, staking, validator rewards              |
| nexara-node          | CLI entry point, service wiring, RPC endpoints        |

---

## Building

```bash
# Build all crates
cargo build --release

# Build a specific crate
cargo build -p nexara-node --release
```

---

## Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test -p nexara-integration-tests

# Run benchmarks
cargo bench
```

---

## License

This project is released for educational and research purposes.
