# Network

The Network component implements Anchor's peer-to-peer communication layer using libp2p. It handles peer discovery, connection management, and message routing for the SSV network protocol.

## Overview

The network stack provides:
- Peer discovery via Discv5
- Connection management with peer scoring
- Topic-based message routing (gossipsub)
- Custom SSV protocol handshake
- Subnet-based message segregation

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 Application Layer                │
│        (Consensus, Signatures, etc.)             │
└────────────────────┬────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────┐
│               Network Service                    │
├─────────────────────────────────────────────────┤
│  Gossipsub     │  Peer Manager  │  Discovery   │
│  (Messaging)   │  (Connections) │  (Discv5)     │
├─────────────────────────────────────────────────┤
│              Transport Layer                     │
│            (TCP / QUIC / Noise)                 │
└─────────────────────────────────────────────────┘
```

## Core Components

### 1. Transport Layer

Supports multiple transport protocols:
- **TCP**: Reliable, established protocol
- **QUIC**: Modern, multiplexed transport
- **Noise**: Encryption protocol for secure channels

Configuration:
```rust
TransportConfig {
    tcp_port: 12001,
    quic_port: 12001,
    enable_quic: true,
}
```

### 2. Discovery (Discv5)

Peer discovery using Ethereum's Discovery v5 protocol:
- Bootstrap from known nodes
- DHT-based peer finding
- ENR (Ethereum Node Record) support

Key features:
- Automatic peer discovery
- Network topology maintenance
- Subnet interest broadcasting

### 3. Gossipsub

Message propagation using gossipsub protocol:
- Topic-based publish/subscribe
- Mesh network topology
- Message deduplication

Topic structure:
```
/ssv/network/{network_name}/topic/{topic_type}/{subnet_id}
```

Example topics:
- `/ssv/network/mainnet/topic/consensus/0`
- `/ssv/network/holesky/topic/partial_sig/42`

### 4. Peer Manager

Manages peer connections and scoring:
- Connection limits (min/max peers)
- Peer scoring based on behavior
- Automatic pruning of low-score peers
- Subnet-aware peer selection

## SSV Protocol Handshake

Custom handshake protocol for SSV-specific information exchange:

### Handshake Flow
```
Initiator                     Responder
    │                             │
    ├──── NodeInfoRequest ────────▶
    │     (version, subnets)      │
    │                             │
    ◀──── NodeInfoResponse ───────┤
    │     (version, subnets)      │
    │                             │
    ├──── Stream Established ─────▶
```

### NodeInfo Structure
```rust
pub struct NodeInfo {
    /// Protocol version
    pub version: Version,
    
    /// Network ID (mainnet/holesky)
    pub network_id: NetworkId,
    
    /// Subscribed subnets (0-127)
    pub subnets: Vec<SubnetId>,
    
    /// Node metadata
    pub metadata: Metadata,
}
```

## Subnet System

The network divides validators into 128 subnets for scalability:

### Subnet Assignment
- Validators assigned to subnets based on public key
- Operators subscribe to subnets of their validators
- Reduces network traffic by ~128x

### Subnet Topics
Each message type has per-subnet topics:
- Consensus messages: `/topic/consensus/{subnet}`
- Partial signatures: `/topic/partial_sig/{subnet}`
- Sync messages: `/topic/sync/{subnet}`

## Message Types

### 1. Consensus Messages
- QBFT protocol messages
- Broadcast to validator's subnet
- High priority routing

### 2. Partial Signatures
- Validator duty signatures
- Subnet-specific routing
- Aggregated by collectors

### 3. Sync Messages
- State synchronization
- Cluster updates
- Network-wide or subnet-specific

## Configuration

### Network Configuration
```rust
pub struct NetworkConfig {
    /// P2P listen address
    pub listen_address: IpAddr,
    
    /// P2P ports
    pub tcp_port: u16,
    pub quic_port: u16,
    
    /// External address (NAT)
    pub external_address: Option<SocketAddr>,
    
    /// Peer limits
    pub target_peers: usize,
    pub max_peers: usize,
    
    /// Subnets to subscribe
    pub subnets: Vec<SubnetId>,
}
```

### Example Configuration
```bash
# Basic configuration
--p2p-port 12001
--listen-address 0.0.0.0

# NAT traversal
--p2p-external-ip 203.0.113.45
--p2p-external-tcp-port 12001

# Subnet subscription
--subnets 0,1,2,3,10,11,12,13
```

## Peer Scoring

Peers are scored based on behavior:

### Positive Scoring
- Valid messages delivered
- Consistent uptime
- Low latency responses
- Protocol compliance

### Negative Scoring
- Invalid messages
- Protocol violations
- Excessive bandwidth usage
- Connection instability

### Score Thresholds
- **Good**: > 0.8 - Preferred peers
- **Acceptable**: 0.5 - 0.8 - Normal peers
- **Poor**: < 0.5 - May be disconnected
- **Banned**: < 0.0 - Immediately disconnected

## Security

### Transport Security
- All connections use Noise protocol
- Perfect forward secrecy
- Authenticated encryption

### Message Validation
- Signature verification
- Topic validation
- Rate limiting per peer

### DoS Protection
- Connection limits
- Message rate limiting
- Bandwidth quotas
- Peer banning for misbehavior

## Metrics

Key network metrics:
- `network_peer_count{state}`: Connected/disconnected peers
- `network_messages_sent{type}`: Outgoing messages
- `network_messages_received{type}`: Incoming messages
- `network_peer_score{peer_id}`: Individual peer scores
- `network_bandwidth{direction}`: Bandwidth usage

## Troubleshooting

### No Peers Connected
1. Check firewall rules for P2P port
2. Verify bootstrap nodes are reachable
3. Ensure correct network (mainnet/holesky)
4. Check external IP configuration

### High Message Rate
1. Review subscribed subnets
2. Check for message amplification
3. Monitor peer scores
4. Enable rate limiting

### Connection Issues
1. Test with telnet to P2P port
2. Check NAT configuration
3. Review peer manager logs
4. Verify protocol version compatibility

## Development

### Testing Network Code
```rust
// Create test network
let network = Network::new_test(config).await?;

// Simulate peer connection
let peer_id = PeerId::random();
network.inject_connection(peer_id).await;

// Send test message
network.publish_message(
    Topic::consensus(subnet_id),
    test_message,
).await?;
```

### Custom Protocols
Extend the network with custom protocols:
```rust
impl NetworkBehaviour for CustomProtocol {
    // Implementation
}

// Add to network
network.add_protocol(custom_protocol);
```