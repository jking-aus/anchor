# Processor

The Processor is the central work scheduling and execution engine in Anchor. It manages a priority-based queue system that processes various types of work items, from consensus messages to validator duties.

## Overview

The Processor implements a multi-queue architecture with different priority levels to ensure critical consensus operations are handled promptly while managing system resources efficiently.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  Work Submission                 │
│  (Consensus, Duties, Signatures, Network Msgs)  │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│                Priority Queues                   │
├─────────────────────────────────────────────────┤
│  1. Permitless Queue (Highest Priority)         │
│     - Consensus messages                        │
│     - Critical network operations               │
├─────────────────────────────────────────────────┤
│  2. Urgent Consensus Queue                      │
│     - Time-sensitive consensus operations       │
│     - QBFT round changes                        │
├─────────────────────────────────────────────────┤
│  3. Standard Work Queue                         │
│     - Validator duties                          │
│     - Signature operations                      │
│     - Non-critical tasks                        │
└────────────────────┬────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────┐
│              Worker Pool                         │
│  (Configurable number of worker threads)        │
└─────────────────────────────────────────────────┘
```

## Work Types

### 1. Consensus Work
- **Purpose**: Process QBFT consensus messages
- **Priority**: Highest (Permitless)
- **Examples**: Proposals, prepares, commits, round changes

### 2. Validator Duties
- **Purpose**: Execute validator responsibilities
- **Priority**: Standard
- **Examples**: Attestations, block proposals, sync committee duties

### 3. Signature Collection
- **Purpose**: Aggregate partial signatures
- **Priority**: Standard
- **Examples**: Attestation signatures, sync committee signatures

### 4. Network Messages
- **Purpose**: Handle P2P network events
- **Priority**: Varies by message type
- **Examples**: Peer messages, gossip propagation

## Configuration

The Processor can be configured through the following parameters:

```rust
pub struct Config {
    /// Maximum number of concurrent workers
    pub max_workers: usize,
    
    /// Maximum items in the permitless queue
    pub max_permitless_queue_size: usize,
    
    /// Maximum items in the urgent consensus queue
    pub max_urgent_consensus_queue_size: usize,
    
    /// Maximum items in the standard work queue
    pub max_work_queue_size: usize,
}
```

### Default Configuration
- `max_workers`: Number of CPU cores
- `max_permitless_queue_size`: 10,000
- `max_urgent_consensus_queue_size`: 5,000
- `max_work_queue_size`: 20,000

## Queue Priority System

The processor uses a three-tier priority system:

1. **Permitless Queue** (Highest Priority)
   - Always processed first
   - No rate limiting
   - Used for critical consensus messages
   - Prevents consensus deadlock

2. **Urgent Consensus Queue**
   - Processed after permitless queue is empty
   - For time-sensitive consensus operations
   - Includes QBFT timeouts and round changes

3. **Standard Work Queue**
   - Lowest priority
   - For regular validator duties
   - Subject to rate limiting to prevent overload

## Work Flow

1. **Submission**: Work items are submitted through type-specific channels
2. **Queuing**: Items are placed in appropriate priority queues
3. **Selection**: Workers pull from highest priority non-empty queue
4. **Execution**: Work is processed by dedicated worker threads
5. **Completion**: Results are sent back through response channels

## Performance Considerations

### Backpressure Handling
- Each queue has a maximum size to prevent memory exhaustion
- When queues are full, submissions block until space is available
- Critical consensus messages use the permitless queue to avoid blocking

### Worker Scaling
- Worker count should match CPU cores for CPU-bound work
- For I/O bound work, consider increasing worker count
- Monitor queue depths to identify bottlenecks

### Queue Sizing
- Permitless queue should be large enough for burst consensus activity
- Standard queue size depends on validator count and duty frequency
- Monitor dropped messages and adjust sizes accordingly

## Metrics

The processor exposes the following metrics:

- `processor_queue_depth{queue_type}`: Current items in each queue
- `processor_work_processed{work_type}`: Total work items processed
- `processor_work_duration{work_type}`: Processing time histogram
- `processor_queue_full_events{queue_type}`: Queue full occurrences

## Usage Example

```rust
// Submit consensus work (highest priority)
processor.send_consensus_work(
    ConsensusWork {
        validator_id: validator_id,
        message: consensus_message,
        response_channel: tx,
    }
).await?;

// Submit validator duty (standard priority)
processor.send_duty_work(
    DutyWork {
        duty_type: DutyType::Attestation,
        slot: current_slot,
        validator_index: index,
    }
).await?;
```

## Troubleshooting

### High Queue Depths
- Increase worker count if CPU usage is low
- Check for slow consensus operations
- Review network message processing

### Dropped Messages
- Increase queue sizes
- Check for message storms or attacks
- Review rate limiting configuration

### Poor Consensus Performance
- Ensure permitless queue is adequately sized
- Check worker thread count
- Monitor consensus message processing times