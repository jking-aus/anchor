# Anchor - SSV Network Node

Anchor is a Rust implementation of an SSV (Secret Shared Validators) network node. It enables distributed validation of Ethereum validators by splitting validator keys across multiple operators using threshold cryptography.

## Overview

The SSV network allows Ethereum validators to distribute their validation duties across multiple operators, improving security, uptime, and decentralization. Anchor implements the SSV protocol, enabling operators to participate in distributed validation.

### Key Features

- **Distributed Validation**: Split validator keys across 4, 7, 10, or 13 operators
- **Byzantine Fault Tolerance**: Uses QBFT consensus for agreement between operators
- **Ethereum Integration**: Syncs with SSV smart contracts on Ethereum mainnet and Holesky
- **High Performance**: Built in Rust with efficient networking and storage
- **Slashing Protection**: Prevents double signing and slashable attestations

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│ Ethereum Node   │────▶│ Event Sync       │────▶│ Database        │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                                                           │
┌─────────────────┐     ┌──────────────────┐              ▼
│ Beacon Node     │────▶│ Duties Tracker   │     ┌─────────────────┐
└─────────────────┘     └──────────────────┘     │ Processor       │
                                 │                └─────────────────┘
                                 ▼                         │
┌─────────────────┐     ┌──────────────────┐              ▼
│ P2P Network     │◀───▶│ QBFT Manager     │     ┌─────────────────┐
└─────────────────┘     └──────────────────┘     │ Message Valid.  │
                                 │                └─────────────────┘
                                 ▼
                        ┌──────────────────┐
                        │ Signature Coll.  │
                        └──────────────────┘
```

### Core Components

- **Network Layer**: LibP2P-based networking with gossipsub for message propagation
- **Consensus Engine**: QBFT implementation for Byzantine fault tolerance
- **Database**: SQLite with in-memory caching for state management
- **Ethereum Integration**: Monitors SSV contract events and syncs network state
- **Validator Store**: Manages validator keys and signing operations

## Quick Start

### Prerequisites

- Rust 1.85.0 or later
- Access to Ethereum execution and beacon nodes
- For operators: RSA key pair for operator identity

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/anchor.git
cd anchor

# Build the release binary
cargo build --release

# Or install directly
make install
```

### Running an Operator Node

1. **Generate operator keys** (if you haven't already):
```bash
anchor keygen --password "your-secure-password"
```

2. **Register your operator** on the SSV network (via webapp or contract)

3. **Run the node**:
```bash
anchor node \
  --operator-id YOUR_OPERATOR_ID \
  --beacon-node http://localhost:5052 \
  --execution-node http://localhost:8545 \
  --execution-node-ws ws://localhost:8546 \
  --rsa-key-path ./key.pem \
  --rsa-key-password "your-secure-password"
```

### Configuration

Key configuration options:

- `--network`: Network to connect to (`mainnet` or `holesky`, default: `holesky`)
- `--data-dir`: Data directory path (default: `~/.anchor/holesky`)
- `--p2p-port`: P2P listening port (default: 12001)
- `--http-api-port`: HTTP API port (default: 16000)
- `--metrics-port`: Metrics server port (default: 15000)

See `anchor node --help` for all options.

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
make test

# Format code
make cargo-fmt

# Run linter
make lint
```

### Testing

```bash
# Run all tests
cargo test --release

# Run specific test
cargo test test_name --release

# Run tests for specific crate
cargo test -p database --release
```

## Tools

### Key Generation (for operators)

Generate RSA keys for operator registration:

```bash
# Generate unencrypted key
anchor keygen

# Generate encrypted key
anchor keygen --password "secure-password"
```

### Key Splitting (for validators)

Split validator keys for distributed validation:

```bash
# Manual splitting
anchor keysplit manual \
  --keystore-path validator_keystore.json \
  --password keystore_password \
  --owner 0xYourAddress \
  --operators 1,2,3,4 \
  --output-path output.json

# Fetch operator data from chain
anchor keysplit onchain \
  --keystore-path validator_keystore.json \
  --password keystore_password \
  --owner 0xYourAddress \
  --operators 1,2,3,4 \
  --output-path output.json \
  --rpc https://eth-mainnet.g.alchemy.com/v2/YOUR_KEY
```

## Network Information

### Mainnet
- SSV Contract: `0xDD9BC35aE942eF0cFa76930954a156B3fF30a4E1`
- Network: Ethereum Mainnet
- ChainID: 1

### Holesky Testnet
- SSV Contract: `0x38A4794cCEd47d3baf7370CcC43B560D3a1beEFA`
- Network: Holesky
- ChainID: 17000

## Monitoring

Anchor exposes Prometheus metrics on the configured metrics port (default: 15000):

- `anchor_consensus_instances`: Active consensus instances
- `anchor_peer_count`: Connected peers
- `anchor_message_validation_errors`: Validation error counts
- `anchor_duties_performed`: Successfully performed duties

## Troubleshooting

### Common Issues

1. **"Failed to connect to beacon node"**
   - Ensure your beacon node is running and accessible
   - Check the beacon node URL is correct
   - Try `--use-long-timeouts` if beacon node is slow

2. **"No peers connected"**
   - Check firewall settings for P2P port (default: 12001)
   - Ensure you're on the correct network
   - Verify bootstrap nodes are accessible

3. **"Database migration failed"**
   - Stop the node cleanly
   - Backup and remove the database file
   - Restart the node to recreate database

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under [LICENSE](LICENSE).

## Resources

- [SSV Network Documentation](https://docs.ssv.network)
- [SSV Network Webapp](https://app.ssv.network)
- [Discord Community](https://discord.gg/ssvnetwork)