# QBFT Protocol Implementation

This module implements the core QBFT (Quorum-based Byzantine Fault Tolerance) consensus protocol. QBFT is a leader-based BFT consensus algorithm that achieves agreement among distributed nodes in the presence of Byzantine failures.

## Protocol Overview

QBFT operates in rounds, where each round has a designated leader who proposes a value. Nodes exchange messages to agree on the proposed value, with the protocol guaranteeing safety (no two correct nodes decide different values) and liveness (eventual decision under synchrony).

## State Machine

The QBFT protocol progresses through the following states:

```
┌─────────────────┐
│ Awaiting        │ ◀─── Start of round
│ Proposal        │ ◀─── Waiting for leader's proposal
└────────┬────────┘
         │ Received valid proposal
         ▼
┌─────────────────┐
│                 │ ◀─── Broadcast PREPARE message
│    Prepare      │ ◀─── Wait for 2f+1 PREPAREs
│                 │
└────────┬────────┘
         │ Received 2f+1 matching PREPAREs
         ▼
┌─────────────────┐
│                 │ ◀─── Broadcast COMMIT message
│    Commit       │ ◀─── Wait for 2f+1 COMMITs
│                 │
└────────┬────────┘
         │ Received 2f+1 matching COMMITs
         ▼
┌─────────────────┐
│    Decided      │ ◀─── Consensus achieved!
└─────────────────┘

On timeout at any state → Round Change → New Round
```

## Message Types

### 1. Proposal Message
```rust
Proposal {
    height: u64,              // Consensus instance height
    round: u64,               // Current round number
    value: Vec<u8>,           // Proposed value
    justification: Option<Justification>,  // Proof for proposed value
}
```

**Justification Types:**
- `PrepareJustification`: 2f+1 PREPAREs from previous round
- `RoundChangeJustification`: 2f+1 ROUND_CHANGEs for this round

### 2. Prepare Message
```rust
Prepare {
    height: u64,              // Consensus instance height
    round: u64,               // Current round number
    value_hash: Hash256,      // Hash of prepared value
}
```

### 3. Commit Message
```rust
Commit {
    height: u64,              // Consensus instance height
    round: u64,               // Current round number
    value_hash: Hash256,      // Hash of committed value
}
```

### 4. Round Change Message
```rust
RoundChange {
    height: u64,              // Consensus instance height
    round: u64,               // Target round number
    prepared_round: Option<u64>,     // Last prepared round
    prepared_value: Option<Hash256>, // Last prepared value
}
```

## Core Algorithm

### Starting a Round

1. **Leader Selection**: `leader = (height + round) % committee_size`
2. **Leader Actions**:
   - Round 0: Propose input value
   - Round > 0: Check for prepared value in RC justification
   - Send PROPOSAL with appropriate justification

3. **Non-Leader Actions**:
   - Wait for valid proposal from leader
   - Timeout if no proposal received

### Processing Proposal

Upon receiving a valid proposal:
1. Verify sender is the correct leader
2. Validate justification (if round > 0)
3. Check value matches any prepared value
4. Broadcast PREPARE message
5. Transition to Prepare state

### Prepare Phase

1. Collect PREPARE messages
2. Upon 2f+1 matching PREPAREs:
   - Update prepared round and value
   - Broadcast COMMIT message
   - Transition to Commit state

### Commit Phase

1. Collect COMMIT messages
2. Upon 2f+1 matching COMMITs:
   - Decide on the value
   - Consensus complete!

### Round Change

On timeout or invalid state:
1. Broadcast ROUND_CHANGE for next round
2. Include prepared round/value if any
3. Upon 2f+1 ROUND_CHANGEs:
   - Start new round as leader (if selected)
   - Or wait for new proposal

## Safety Properties

### Agreement
- No two correct nodes decide different values
- Enforced by quorum intersection (2f+1 > n/2)

### Validity
- Decided value was proposed by some node
- Enforced by justification requirements

### Integrity
- Correct node decides at most once
- Enforced by state machine design

## Liveness Properties

Under eventual synchrony and with at most f failures:
- All correct nodes eventually decide
- Progress guaranteed through round changes

## Configuration

### Committee Configuration
```rust
pub struct QBFTConfig {
    /// Total committee size
    pub committee_size: usize,
    
    /// Byzantine fault threshold (f)
    pub fault_threshold: usize,
    
    /// Quorum size (2f + 1)
    pub quorum_size: usize,
}
```

### Validation Rules
- `committee_size >= 3f + 1`
- `quorum_size = 2f + 1`
- `fault_threshold = (committee_size - 1) / 3`

## Implementation Details

### State Storage
```rust
pub struct QBFTState {
    /// Current round
    round: Round,
    
    /// Current protocol state
    stage: Stage,
    
    /// Prepared round and value
    prepared: Option<(Round, Value)>,
    
    /// Message containers for each round
    messages: HashMap<Round, RoundMessages>,
}
```

### Message Validation

All messages must pass validation:
1. Correct height
2. Valid round number
3. Proper signature
4. Sender in committee
5. Message appropriate for current state

### Optimizations

1. **Message Caching**: Store messages for future rounds
2. **Early Decide**: Decide immediately upon 2f+1 COMMITs
3. **Justification Reuse**: Reuse valid justifications across rounds

## Example Usage

```rust
// Initialize QBFT instance
let config = QBFTConfig::new(committee_size);
let mut qbft = QBFT::new(config, height, input_value);

// Process incoming message
match qbft.process_message(message, sender) {
    Ok(Some(decided_value)) => {
        // Consensus reached!
        println!("Decided: {:?}", decided_value);
    }
    Ok(None) => {
        // Continue processing
    }
    Err(e) => {
        // Invalid message
        eprintln!("Error: {}", e);
    }
}

// Check for timeout
if qbft.should_timeout(current_time) {
    let round_change = qbft.create_round_change();
    broadcast(round_change);
}
```

## Testing

The implementation includes comprehensive tests:
- Unit tests for each state transition
- Property-based tests for safety
- Simulation tests for liveness
- Byzantine fault injection tests

## Security Considerations

### Message Authentication
- All messages must be signed
- Signatures verified before processing
- Replay protection via height/round

### DoS Protection
- Rate limiting per sender
- Message size limits
- Bounded message storage

### Byzantine Behavior
- Protocol tolerates up to f Byzantine nodes
- Common attacks handled:
  - Equivocation (double voting)
  - Invalid justifications
  - Proposal withholding