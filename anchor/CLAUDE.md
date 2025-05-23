# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Anchor is a Rust implementation of an SSV (Secret Shared Validators) network node. It enables distributed validation of Ethereum validators by splitting validator keys across multiple operators using threshold cryptography. The project is built on top of Lighthouse and implements the SSV protocol for secure, distributed validator operations.

## Development Commands

### Building
```bash
# Build release binary
cargo build --release

# Install binary locally
make install

# Cross-compile for different architectures
make build-x86_64
make build-aarch64
```

### Testing
```bash
# Run all tests in release mode (recommended)
make test-release
# or
cargo test --release

# Run tests in debug mode
make test-debug

# Run tests with nextest (if installed)
make nextest-release

# Run a single test
cargo test --release test_name

# Run tests for a specific crate
cargo test -p database --release
```

### Code Quality
```bash
# Format code
make cargo-fmt

# Check formatting
make cargo-fmt-check

# Run clippy linter
make lint

# Auto-fix clippy warnings
make lint-fix

# Check for unused dependencies
make udeps

# Audit dependencies for security vulnerabilities
make audit
```

## Architecture

### Core Components

1. **Network Layer** (`network/`)
   - LibP2P-based networking with custom SSV protocol
   - Peer discovery and management
   - Message routing and gossipsub implementation

2. **Database Layer** (`database/`)
   - SQLite persistent storage with in-memory caching
   - Multi-index maps for efficient data access
   - Stores operators, validators, clusters, and shares

3. **Consensus Engine** (`qbft_manager/`, `common/qbft/`)
   - QBFT (Byzantine Fault Tolerant) consensus implementation
   - Manages consensus instances for each validator duty

4. **Ethereum Integration** (`eth/`)
   - Syncs with SSV smart contract events
   - Processes validator registrations, operator changes, and cluster updates

5. **Message Processing** (`processor/`, `message_validator/`, `message_receiver/`)
   - Validates incoming consensus and partial signature messages
   - Routes messages to appropriate handlers
   - Manages message queues and backpressure

6. **Validator Operations** (`validator_store/`, `duties_tracker/`)
   - Tracks validator duties (attestations, proposals, sync committees)
   - Manages validator key shares and signing operations

### Key Data Flows

1. **Event Sync**: Ethereum events → Event Processor → Database → State Update
2. **Consensus**: Duty Detection → QBFT Instance → Message Exchange → Signature Collection → Duty Submission
3. **Networking**: P2P Message → Validation → Processor → Handler

## Running the Node

### Basic Operation
```bash
# Run with default configuration (Holesky testnet)
cargo run --release -- node

# Run with specific network
cargo run --release -- node --network mainnet

# Run with custom data directory
cargo run --release -- node --data-dir /path/to/data

# Run with specific operator ID
cargo run --release -- node --operator-id 123
```

### Key Generation (for operators)
```bash
# Generate unencrypted RSA key
cargo run --release -- keygen

# Generate encrypted RSA key
cargo run --release -- keygen --password "secure-password"
```

### Key Splitting (for validators)
```bash
# Manual key splitting
cargo run --release -- keysplit manual --keystore-path /path/to/keystore.json --password keystore_password --owner 0x... --operators 1,2,3,4 --output-path output.json --nonce 0 --public-keys key1,key2,key3,key4

# Onchain key splitting (fetches data from blockchain)
cargo run --release -- keysplit onchain --keystore-path /path/to/keystore.json --password keystore_password --owner 0x... --operators 1,2,3,4 --output-path output.json --rpc https://eth-mainnet.provider.com --network mainnet
```

## Network Configuration

- **Mainnet**: SSV Contract at `0xDD9BC35aE942eF0cFa76930954a156B3fF30a4E1`
- **Holesky**: SSV Contract at `0x38A4794cCEd47d3baf7370CcC43B560D3a1beEFA`
- **Default ports**: P2P on 12001, HTTP API on 16000, Metrics on 15000

## Testing Guidelines

- Database tests use in-memory SQLite instances
- Network tests often use mock message senders
- QBFT tests include both unit and integration scenarios
- Use `--nocapture` to see test output: `cargo test -- --nocapture`

## Important Notes

- The project uses Lighthouse's types and utilities extensively
- BLS operations use both blst and blsful libraries for different purposes
- All validator operations require proper slashing protection
- The node must sync historical events before processing live duties