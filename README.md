# NEXARA (NXR) — Post-Quantum Layer 1 Blockchain

A production-grade, open-source Layer 1 blockchain with post-quantum cryptography, a custom smart contract language (NEXLANG), and a 100-shard architecture.

## Features

- **Post-Quantum Cryptography**: ML-DSA-65 signatures, ML-KEM-1024 key encapsulation, BLAKE3 hashing
- **100-Shard Architecture**: 2,500 TPS per shard (250,000 TPS aggregate)
- **200ms Block Time**: Ultra-fast finality
- **HybridSync Consensus**: DPoQS + AIBFT + ZK-Finality
- **NEXLANG**: Type-safe smart contract language with built-in security analysis
- **NXVM**: Purpose-built virtual machine with per-opcode gas metering
- **Cross-Chain Bridge**: Ethereum, BNB Chain, Solana, Cosmos (IBC), Bitcoin (HTLC)
- **Deflationary Tokenomics**: 50% fee burn, Protocol-Owned Liquidity
- **MEV Protection**: Encrypted transaction pool with sandwich attack detection

## Quick Start

### Prerequisites

- Rust 1.75+ (install from [rustup.rs](https://rustup.rs))
- macOS, Linux, or Windows with WSL2

### Build

```bash
git clone https://github.com/nexara/nexara.git
cd nexara
cargo build --release
```

### Run Tests

```bash
cargo test --workspace
```

### Run Node

```bash
# Full node
./target/release/nexara

# Validator mode
./target/release/nexara --validator

# With custom config
./target/release/nexara --config config.json --rpc-port 9944

# Show info
./target/release/nexara info
```

## Project Structure

```
nexara/
├── crates/
│   ├── nexara-crypto/       # PQC primitives (ML-DSA-65, ML-KEM-1024, BLAKE3)
│   ├── nexara-core/         # Transaction, Block, State, Genesis
│   ├── nexara-consensus/    # HybridSync (DPoQS + AIBFT + ZK-Finality)
│   ├── nexara-network/      # libp2p P2P networking
│   ├── nexara-shard/        # 100-shard architecture + beacon chain
│   ├── nexara-mempool/      # Fee-priority tx pool + MEV protection
│   ├── nexlang/             # Smart contract language compiler
│   ├── nxvm/                # Virtual machine
│   ├── nexara-bridge/       # Cross-chain bridge (NCCP)
│   ├── nexara-tokenomics/   # Supply, vesting, staking, burn, POL
│   └── nexara-node/         # Full node binary
├── contracts/               # Example NEXLANG contracts
├── tests/
│   ├── integration/         # End-to-end tests
│   └── benchmarks/          # Performance benchmarks
├── docs/                    # Whitepaper, specs, architecture
└── scripts/                 # Build, test, benchmark scripts
```

## Crate Overview

| Crate | Description |
|-------|-------------|
| `nexara-crypto` | ML-DSA-65 signing, ML-KEM-1024 KEM, BLAKE3, wallets, MPC |
| `nexara-core` | Transaction types, blocks, Merkle trees, chain state |
| `nexara-consensus` | Validators, committee selection, block finalization |
| `nexara-network` | P2P gossip, PQ handshake, peer discovery |
| `nexara-shard` | Shard management, beacon chain, cross-shard messaging |
| `nexara-mempool` | Transaction ordering, dedup, eviction, MEV protection |
| `nexlang` | Lexer, parser, type checker, security analyzer, codegen |
| `nxvm` | Stack-based VM with 45 opcodes, gas metering, storage |
| `nexara-bridge` | NCCP bridge for ETH, BNB, SOL, Cosmos, BTC |
| `nexara-tokenomics` | 500M NXR supply, vesting, staking, 50% fee burn |
| `nexara-node` | CLI, JSON-RPC, configuration, main event loop |

## Tokenomics

- **Total Supply**: 500,000,000 NXR
- **Validator Rewards**: 35% (175M NXR)
- **Ecosystem Fund**: 20% (100M NXR)
- **Team**: 15% (75M NXR, 4-year vest, 1-year cliff)
- **Community**: 10% (50M NXR)
- **Treasury**: 10% (50M NXR)
- **Liquidity**: 5% (25M NXR)
- **Airdrop**: 5% (25M NXR)

## Example NEXLANG Contract

```nexlang
@pqc_secure
contract Counter {
    state count: U256;
    state owner: Address;

    event Incremented(new_value: U256);

    @constructor
    fn init() {
        self.owner = msg.sender;
        self.count = 0;
    }

    @public
    fn increment() {
        self.count = self.count + 1;
        emit Incremented(self.count);
    }

    @view
    fn get_count() -> U256 {
        return self.count;
    }
}
```

## Documentation

- [Whitepaper](docs/whitepaper.md)
- [NEXLANG Specification](docs/nexlang-spec.md)
- [Architecture Guide](docs/architecture.md)

## Scripts

```bash
# Build and test
./scripts/build.sh

# Run all tests + clippy
./scripts/test.sh

# Run benchmarks
./scripts/bench.sh
```

## Chain Parameters

| Parameter | Value |
|-----------|-------|
| Chain ID | 20240101 |
| Block Time | 200ms |
| Shards | 100 |
| TPS (per shard) | 2,500 |
| TPS (aggregate) | 250,000 |
| Min Stake | 1,000 NXR |
| Quorum | 67% |
| Fee Burn | 50% |
| Unstake Cooldown | 7 days |

## License

MIT License — see [LICENSE](LICENSE) for details.

---

**NEXARA** — Built for the Post-Quantum Era.


## Architecture Overview

### Project Type
- **Primary stack:** Rust workspace / blockchain components
- **Primary language:** Rust
- **Primary entrypoint/build root:** Cargo workspace via Cargo.toml

### High-Level Architecture
- This repository is organized in modular directories grouped by concern (application code, configuration, scripts, documentation, and assets).
- Runtime/build artifacts such as virtual environments, node modules, and compiled outputs are intentionally excluded from architecture mapping.
- The project follows a layered flow: entry point -> domain/application modules -> integrations/data/config.

### Component Breakdown
- **Application layer:** Core executables, services, UI, or command handlers.
- **Domain/business layer:** Feature logic and processing modules.
- **Integration layer:** External APIs, databases, files, or platform-specific connectors.
- **Support layer:** Config, scripts, docs, tests, and static assets.

### Data/Execution Flow
1. Start from the configured entrypoint or package scripts.
2. Route execution into feature-specific modules.
3. Process domain logic and interact with integrations/storage.
4. Return results to UI/API/CLI outputs.

### Directory Map (Top-Level + Key Subfolders)
```
Cargo.toml
crates
crates/nxvm
crates/nexara-core
crates/nexara-crypto
crates/.DS_Store
crates/nexara-shard
crates/nexara-mempool
crates/nexara-bridge
crates/nexlang
crates/nexara-network
crates/nexara-consensus
crates/nexara-node
crates/nexara-tokenomics
.DS_Store
target
contracts
contracts/governance.nxl
contracts/nft.nxl
contracts/token.nxl
contracts/shard_bridge.nxl
contracts/dex.nxl
tests
tests/nexara-integration-tests
Cargo.lock
docs
docs/nexlang-spec.md
docs/architecture.md
docs/whitepaper.md
README.md
.gitignore
scripts
scripts/bench.sh
scripts/build.sh
scripts/test.sh
```

### Notes
- Architecture section auto-generated on 2026-03-22 and can be refined further with exact runtime/deployment details.
