# Anchor Configuration Guide

This guide covers all configuration options available in Anchor, with explanations and examples for common scenarios.

## Table of Contents

1. [Core Configuration](#core-configuration)
2. [Network Configuration](#network-configuration)
3. [Node Connections](#node-connections)
4. [API Configuration](#api-configuration)
5. [Performance Tuning](#performance-tuning)
6. [Security Settings](#security-settings)
7. [MEV Configuration](#mev-configuration)
8. [Advanced Options](#advanced-options)

## Core Configuration

### Data Directory
```bash
--data-dir <PATH>
```
- **Default**: `~/.anchor/{network}`
- **Description**: Base directory for all node data
- **Example**: `--data-dir /opt/anchor/data`

### Network Selection
```bash
--network <NETWORK>
```
- **Options**: `mainnet`, `holesky` 
- **Default**: `holesky`
- **Description**: SSV network to connect to
- **Example**: `--network mainnet`

### Operator Configuration
```bash
--operator-id <ID>
```
- **Required**: Yes
- **Description**: Your operator ID from SSV network registration
- **Example**: `--operator-id 12345`

### RSA Key Configuration
```bash
--rsa-key-path <PATH>
--rsa-key-password <PASSWORD>
```
- **Description**: Path to RSA private key and optional password
- **Default path**: `{data-dir}/key.pem`
- **Example**: 
  ```bash
  --rsa-key-path ./operator_key.pem \
  --rsa-key-password "my-secure-password"
  ```

## Network Configuration

### P2P Settings
```bash
--p2p-port <PORT>
--p2p-tcp-port <PORT>
--p2p-quic-port <PORT>
```
- **Default ports**: 
  - General: 12001
  - TCP: 12001
  - QUIC: 12001
- **Description**: Ports for P2P communication
- **Example**: `--p2p-port 13001`

### Listen Address
```bash
--listen-address <ADDRESS>
```
- **Default**: `0.0.0.0`
- **Description**: IP address to bind to
- **Example**: `--listen-address 127.0.0.1`

### External Address
```bash
--p2p-external-ip <IP>
--p2p-external-tcp-port <PORT>
--p2p-external-quic-port <PORT>
```
- **Description**: External IP/ports for NAT traversal
- **Example**:
  ```bash
  --p2p-external-ip 203.0.113.45 \
  --p2p-external-tcp-port 12001
  ```

### Subnet Configuration
```bash
--subnets <SUBNET_LIST>
```
- **Default**: All subnets (0-127)
- **Description**: Comma-separated list of subnets to subscribe to
- **Example**: `--subnets 0,1,2,3`

## Node Connections

### Beacon Node
```bash
--beacon-node <URL>
```
- **Default**: `http://localhost:5052`
- **Description**: Beacon node HTTP endpoint(s)
- **Multiple**: Can specify multiple times
- **Example**:
  ```bash
  --beacon-node http://beacon1:5052 \
  --beacon-node http://beacon2:5052
  ```

### Execution Node
```bash
--execution-node <URL>
--execution-node-ws <URL>
```
- **Default**: 
  - HTTP: `http://localhost:8545`
  - WebSocket: `ws://localhost:8546`
- **Description**: Execution layer endpoints
- **Example**:
  ```bash
  --execution-node https://eth-mainnet.g.alchemy.com/v2/KEY \
  --execution-node-ws wss://eth-mainnet.g.alchemy.com/v2/KEY
  ```

### TLS Certificates
```bash
--beacon-node-tls-cert <PATH>
--execution-node-tls-cert <PATH>
```
- **Description**: Custom TLS certificates for node connections
- **Example**: `--beacon-node-tls-cert /path/to/cert.pem`

### Connection Options
```bash
--allow-unsynced-beacon-node
--use-long-timeouts
```
- **Description**: 
  - Allow connection to unsynced beacon node
  - Use longer timeouts for slow connections

## API Configuration

### HTTP API
```bash
--http-api
--http-api-port <PORT>
--http-api-address <ADDRESS>
--http-api-allow-origin <ORIGIN>
```
- **Default**: 
  - Enabled: false
  - Port: 16000
  - Address: 127.0.0.1
- **Example**:
  ```bash
  --http-api \
  --http-api-port 16000 \
  --http-api-allow-origin "https://dashboard.example.com"
  ```

### Metrics API
```bash
--metrics
--metrics-port <PORT>
--metrics-address <ADDRESS>
--metrics-allow-origin <ORIGIN>
```
- **Default**:
  - Enabled: false
  - Port: 15000
  - Address: 127.0.0.1
- **Example**:
  ```bash
  --metrics \
  --metrics-port 15000
  ```

### High Validator Count Metrics
```bash
--enable-high-validator-count-metrics
```
- **Description**: Enable detailed metrics for >64 validators
- **Warning**: Increases metrics cardinality significantly

## Performance Tuning

### Worker Configuration
```bash
--max-workers <COUNT>
```
- **Default**: Number of CPU cores
- **Description**: Maximum concurrent worker threads
- **Example**: `--max-workers 8`

### Queue Sizes
```bash
--max-work-queue-size <SIZE>
--max-consensus-queue-size <SIZE>
--max-permitless-queue-size <SIZE>
```
- **Defaults**:
  - Work queue: 20,000
  - Consensus queue: 5,000
  - Permitless queue: 10,000
- **Description**: Maximum items in processing queues

### Message Buffer
```bash
--message-buffer-size <SIZE>
```
- **Default**: 1000
- **Description**: Network message buffer size

## Security Settings

### Slashing Protection
```bash
--disable-slashing-protection
```
- **Default**: Enabled
- **Description**: Disable slashing protection (DANGEROUS!)
- **Warning**: Only use for testing

### Impostor Mode
```bash
--impostor <OPERATOR_ID>
```
- **Description**: Act as a different operator (testing only)
- **Warning**: Do not use in production

## MEV Configuration

### Builder API
```bash
--builder-proposals
--prefer-builder-proposals
--builder-boost-factor <PERCENTAGE>
```
- **Description**:
  - `builder-proposals`: Enable external block builders
  - `prefer-builder-proposals`: Always prefer builder blocks
  - `builder-boost-factor`: Percentage boost for builder bids
- **Example**:
  ```bash
  --builder-proposals \
  --builder-boost-factor 10
  ```

### Gas Limit
```bash
--gas-limit <LIMIT>
```
- **Default**: 30,000,000
- **Description**: Gas limit for proposed blocks
- **Example**: `--gas-limit 35000000`

## Advanced Options

### Latency Measurement
```bash
--disable-latency-measurement-service
```
- **Description**: Disable latency monitoring service
- **Use case**: Reduce resource usage

### Logging Configuration
```bash
--log-level <LEVEL>
--log-format <FORMAT>
--log-dir <PATH>
```
- **Options**:
  - Level: `error`, `warn`, `info`, `debug`, `trace`
  - Format: `json`, `pretty`, `compact`
- **Example**:
  ```bash
  --log-level debug \
  --log-format json \
  --log-dir /var/log/anchor
  ```

### Database Options
```bash
--db-threads <COUNT>
--db-cache-size <MB>
```
- **Description**: Database performance tuning
- **Example**: `--db-threads 4 --db-cache-size 512`

## Configuration File

Instead of command-line arguments, you can use a configuration file:

```yaml
# anchor-config.yaml
network: mainnet
operator_id: 12345
data_dir: /opt/anchor/data

beacon_nodes:
  - http://beacon1:5052
  - http://beacon2:5052

execution_node: https://eth-mainnet.g.alchemy.com/v2/KEY
execution_node_ws: wss://eth-mainnet.g.alchemy.com/v2/KEY

p2p:
  port: 12001
  external_ip: 203.0.113.45

api:
  http_enabled: true
  http_port: 16000
  metrics_enabled: true
  metrics_port: 15000

performance:
  max_workers: 8
  max_work_queue_size: 30000

mev:
  builder_proposals: true
  builder_boost_factor: 10
```

Load with: `anchor node --config anchor-config.yaml`

## Environment Variables

All options can also be set via environment variables:
```bash
export ANCHOR_NETWORK=mainnet
export ANCHOR_OPERATOR_ID=12345
export ANCHOR_BEACON_NODE=http://localhost:5052
export ANCHOR_P2P_PORT=12001
```

## Common Configurations

### High-Performance Setup
```bash
anchor node \
  --operator-id 12345 \
  --network mainnet \
  --max-workers 16 \
  --max-work-queue-size 50000 \
  --message-buffer-size 5000 \
  --db-cache-size 1024 \
  --use-long-timeouts
```

### Resource-Constrained Setup
```bash
anchor node \
  --operator-id 12345 \
  --network holesky \
  --max-workers 4 \
  --max-work-queue-size 10000 \
  --disable-latency-measurement-service \
  --subnets 0,1,2,3
```

### Development Setup
```bash
anchor node \
  --operator-id 12345 \
  --network holesky \
  --log-level debug \
  --http-api \
  --metrics \
  --allow-unsynced-beacon-node
```