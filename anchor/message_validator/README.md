# Message Validator

The Message Validator is responsible for validating all incoming SSV network messages before they are processed. It implements comprehensive validation rules to ensure message integrity, prevent attacks, and maintain consensus safety.

## Overview

The validator performs both stateless checks (signature verification, format validation) and stateful checks (consensus rules, message ordering) on incoming messages. It maintains per-validator consensus state to enforce protocol rules.

## Architecture

```
┌─────────────────────────┐
│   Incoming Message      │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│   Basic Validation      │
│  - Format checks        │
│  - Signature verify     │
│  - Domain validation    │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│  Consensus Validation   │
│  - State checks         │
│  - Round validity       │
│  - Justification        │
└───────────┬─────────────┘
            │
            ▼
┌─────────────────────────┐
│   Message Accepted      │
│   or Rejected with      │
│   Specific Error        │
└─────────────────────────┘
```

## Validation Types

### 1. Consensus Message Validation

Validates QBFT consensus messages including:
- Proposals
- Prepares
- Commits
- Round changes

Key checks:
- Message height matches current slot
- Round number is valid
- Proper justification for proposals
- Signature verification
- No equivocation (double voting)

### 2. Partial Signature Validation

Validates partial signatures for duties:
- Attestations
- Block proposals
- Sync committee messages
- Voluntary exits

Key checks:
- Signature validity
- Signing root matches duty
- Validator is active
- No double signing

## Validation Rules

### Basic Validation (Stateless)

1. **Format Validation**
   - Message structure is valid
   - Required fields are present
   - Data types are correct

2. **Signature Verification**
   - Signature matches signer's public key
   - Signature covers correct message data
   - Domain separation is correct

3. **Network Validation**
   - Message is for correct network (mainnet/holesky)
   - Subnet assignment is valid

### Consensus Validation (Stateful)

1. **Height Validation**
   - Message height matches current slot
   - Not too far in future (1 slot tolerance)
   - Not too old (beyond retention)

2. **Round Validation**
   - Round number is non-negative
   - Round changes are justified
   - No round regression

3. **State Machine Validation**
   - Message is valid for current consensus state
   - State transitions are legal
   - Justifications are valid

4. **Equivocation Detection**
   - No double proposals in same round
   - No double prepares/commits
   - Tracks all signed messages

## Error Types

The validator returns specific error types for different validation failures:

### Consensus Errors
- `WrongHeight`: Message height doesn't match slot
- `InvalidRound`: Round number invalid
- `InvalidJustification`: Proposal justification failed
- `Equivocation`: Double signing detected
- `UnknownValidator`: Validator not in committee

### Signature Errors
- `InvalidSignature`: Signature verification failed
- `WrongSigningRoot`: Signing root mismatch
- `DuplicateSignature`: Already received this signature

### State Errors
- `ConsensusAlreadyRunning`: Consensus already active
- `InvalidStateTransition`: Illegal state change
- `MessageTooOld`: Beyond retention period

## State Management

### Consensus State Tracking

The validator maintains state for each active consensus instance:

```rust
pub struct ConsensusState {
    /// Current consensus height (slot)
    pub height: Height,
    
    /// Current round number
    pub round: Round,
    
    /// Current state (AwaitingProposal, Prepare, Commit)
    pub state: ConsensusStage,
    
    /// Messages received in this instance
    pub received_messages: MessageLog,
    
    /// Prepared round and value
    pub prepared: Option<(Round, Value)>,
}
```

### Message Counting

Tracks message counts to detect spam and enforce limits:
- Per-validator message counts
- Per-round message limits
- Rate limiting for signature messages

## Performance Optimizations

### Caching
- Signature verification results cached
- Validator public keys cached
- Committee memberships cached

### Early Rejection
- Quick format checks before expensive operations
- Height checks before signature verification
- Rate limit checks at entry point

### Batch Processing
- Signatures verified in parallel
- State updates batched
- Lock contention minimized

## Usage Example

```rust
// Validate a consensus message
let validation_result = validator.validate_consensus_message(
    &message,
    &validator_pk,
    committee_size,
).await;

match validation_result {
    Ok(()) => {
        // Message is valid, process it
        processor.handle_message(message).await;
    }
    Err(ValidationError::WrongHeight) => {
        // Message is for wrong slot, ignore
    }
    Err(ValidationError::Equivocation) => {
        // Slashable offense detected!
        slashing_detector.report(message);
    }
    Err(e) => {
        // Other validation failure
        metrics.validation_error(e);
    }
}
```

## Configuration

### Retention Settings
- `message_retention_slots`: How long to keep message history (default: 3)
- `state_retention_slots`: How long to keep consensus state (default: 5)

### Rate Limits
- `max_messages_per_round`: Maximum messages per validator per round
- `signature_rate_limit`: Maximum signatures per slot per validator

## Metrics

Key metrics exposed:
- `message_validation_success{message_type}`: Successful validations
- `message_validation_failure{message_type, error}`: Failed validations
- `validation_duration{message_type}`: Time spent validating
- `equivocation_detections`: Slashable offenses detected

## Security Considerations

### Attack Prevention
- Rate limiting prevents message spam
- Equivocation detection prevents double signing
- State validation prevents consensus manipulation

### Slashing Protection
- All signing operations checked against history
- Double voting immediately detected
- Slashable events logged and reported