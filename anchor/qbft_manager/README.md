# QBFT Manager

The QBFT Manager orchestrates Byzantine Fault Tolerant consensus instances for validator duties. It manages the lifecycle of QBFT instances, handles timeouts, and coordinates message flow between instances and the network.

## Overview

QBFT (Quorum-based Byzantine Fault Tolerance) is the consensus protocol used by SSV to achieve agreement among operators on validator duties. The QBFT Manager creates and manages instances of the QBFT protocol for each duty that needs consensus.

## Architecture

```
┌─────────────────────────────────────┐
│         Duty Detection              │
│  (Attestation, Proposal, etc.)      │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│         QBFT Manager                │
│  - Creates consensus instance       │
│  - Assigns instance ID              │
│  - Manages instance lifecycle       │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│      QBFT Instance                  │
│  - Runs consensus protocol          │
│  - Handles rounds and timeouts      │
│  - Produces decided value           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│    Signature Collection             │
│  - Aggregates partial signatures    │
│  - Produces final signature         │
└─────────────────────────────────────┘
```

## QBFT Protocol Flow

### 1. Instance Creation
When a new duty is detected:
1. Manager creates new QBFT instance
2. Instance initialized with duty data
3. First round (round 0) begins
4. Leader for round 0 selected

### 2. Consensus Rounds
Each round follows this sequence:

```
Round N:
┌──────────┐     ┌─────────┐     ┌────────┐     ┌──────────┐
│ Proposal │ ──▶ │ Prepare │ ──▶ │ Commit │ ──▶ │ Decision │
└──────────┘     └─────────┘     └────────┘     └──────────┘
     │                                                  │
     │                                                  │
     └──────── Round Change (on timeout) ──────────────┘
```

### 3. Message Types

**Proposal**: Leader proposes a value
- Contains proposed data and justification
- Only one per round from designated leader

**Prepare**: Validators prepare to accept proposal
- Indicates willingness to commit to proposed value
- Requires quorum (2f+1) to proceed

**Commit**: Validators commit to the value
- Final agreement on the value
- Requires quorum to decide

**Round Change**: Move to next round on timeout
- Contains justification for changing rounds
- New leader selected for next round

## Instance Lifecycle

### States
1. **Active**: Instance running consensus
2. **Decided**: Consensus reached, value decided
3. **Expired**: Timeout exceeded, instance failed

### Timeouts
- **Round timeout**: Time limit for each round
- **Instance timeout**: Overall time limit for consensus
- Timeouts increase with round number (exponential backoff)

## Key Components

### Instance Manager
```rust
pub struct QBFTManager {
    /// Active consensus instances
    instances: HashMap<ConsensusId, Instance>,
    
    /// Timeout scheduler
    timeout_manager: TimeoutManager,
    
    /// Message router
    message_handler: MessageHandler,
}
```

### Instance State
```rust
pub struct Instance {
    /// Unique identifier
    id: ConsensusId,
    
    /// Current round
    round: Round,
    
    /// QBFT protocol state
    qbft: QBFTStateMachine,
    
    /// Timeout for current round
    round_timeout: Duration,
}
```

## Leader Selection

Leaders are selected deterministically:
```
leader = (round + height) % committee_size
```

This ensures:
- Predictable leader rotation
- Different leaders across rounds
- Fair distribution over time

## Timeout Management

### Round Timeouts
- Initial timeout: 4 seconds
- Timeout doubles each round
- Maximum timeout: 60 seconds

### Instance Timeout
- Total time limit: 5 minutes
- Prevents instances from running forever
- Cleanup on expiration

## Message Handling

### Incoming Messages
1. Validate message (via Message Validator)
2. Route to correct instance
3. Process in QBFT state machine
4. Broadcast any resulting messages

### Outgoing Messages
1. Sign message with operator key
2. Broadcast to committee members
3. Track for potential retransmission

## Configuration

### Timing Configuration
```rust
pub struct TimeoutConfig {
    /// Base timeout for round 0
    pub base_timeout: Duration,
    
    /// Timeout multiplier per round
    pub timeout_multiplier: f64,
    
    /// Maximum round timeout
    pub max_round_timeout: Duration,
    
    /// Maximum instance lifetime
    pub max_instance_duration: Duration,
}
```

### Performance Tuning
- `max_concurrent_instances`: Limit parallel instances
- `message_buffer_size`: Buffer for incoming messages
- `cleanup_interval`: How often to clean expired instances

## Metrics

Key metrics:
- `qbft_instances_active`: Currently running instances
- `qbft_instances_decided`: Successfully decided instances
- `qbft_instances_failed`: Failed instances (timeout/error)
- `qbft_round_duration{round}`: Time per round
- `qbft_rounds_per_instance`: Rounds needed to decide

## Error Handling

### Common Failures
1. **Leader Failure**: Handled by round change mechanism
2. **Network Partition**: May cause timeouts, retries on heal
3. **Byzantine Nodes**: Protocol tolerates up to f failures

### Recovery Mechanisms
- Automatic round changes on timeout
- Message retransmission
- Instance cleanup and retry

## Usage Example

```rust
// Start consensus for attestation duty
let instance_id = qbft_manager.start_consensus(
    ConsensusData::Attestation {
        slot: current_slot,
        committee_index: 0,
        attestation_data: data,
    },
    committee_members,
).await?;

// Handle incoming consensus message
qbft_manager.handle_message(
    consensus_message,
    sender_id,
).await?;

// Check if consensus decided
if let Some(decided_value) = qbft_manager.get_decided(instance_id) {
    // Process decided value
    signature_collector.aggregate(decided_value).await?;
}
```

## Security Considerations

### Byzantine Fault Tolerance
- Tolerates up to f = (n-1)/3 faulty nodes
- Requires honest quorum of 2f+1 nodes
- Safety maintained even with network issues

### Message Authentication
- All messages signed by sender
- Signatures verified before processing
- Replay protection via instance IDs