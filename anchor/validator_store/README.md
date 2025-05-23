# Validator Store

The Validator Store manages validator keys, handles signing operations, and provides slashing protection for SSV validators. It acts as the secure interface between validator duties and the distributed key shares.

## Overview

The Validator Store is responsible for:
- Managing validator key shares
- Coordinating distributed signing operations
- Enforcing slashing protection rules
- Tracking validator metadata and status
- Integrating with beacon nodes for duties

## Architecture

```
┌─────────────────────────────────────┐
│         Beacon Node                 │
│    (Duties & Chain State)           │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│        Validator Store              │
├─────────────────────────────────────┤
│  Key Management │ Slashing Protection│
│  Signing Ops    │ Metadata Service   │
└──────────────┬──────────────────────┘
               │
┌──────────────┴──────────────────────┐
│     Distributed Signing             │
│   (QBFT Consensus + Sig Agg)       │
└─────────────────────────────────────┘
```

## Core Components

### 1. Key Management

Manages validator key shares:
```rust
pub struct ValidatorKeyShare {
    /// Validator public key
    pub validator_pubkey: PublicKey,
    
    /// Share index (position in committee)
    pub share_index: u64,
    
    /// Secret key share (encrypted)
    pub secret_share: SecretKeyShare,
    
    /// Committee public keys
    pub committee_pubkeys: Vec<PublicKey>,
}
```

Key features:
- Secure storage of key shares
- Share validation and verification
- Committee membership tracking

### 2. Signing Operations

Coordinates distributed signing:

#### Signing Flow
1. **Duty Detection**: Receive duty from beacon node
2. **Consensus Initiation**: Start QBFT for signing decision
3. **Partial Signing**: Create partial signature with key share
4. **Signature Aggregation**: Combine partial signatures
5. **Duty Submission**: Submit to beacon node

#### Supported Operations
- Block proposals
- Attestations
- Sync committee messages
- Voluntary exits
- Aggregation duties

### 3. Slashing Protection

Prevents slashable offenses:

#### Protection Rules
1. **No Double Voting**: 
   - One attestation per slot per validator
   - One block per slot per validator

2. **No Surround Voting**:
   - Attestation source/target don't surround previous
   - Implements MinSpan algorithm

3. **Voluntary Exit Protection**:
   - Only one exit per validator
   - Permanent record of exits

#### Implementation
```rust
pub struct SlashingProtection {
    /// Attestation history
    attestations: HashMap<ValidatorPubkey, AttestationHistory>,
    
    /// Block proposal history
    proposals: HashMap<ValidatorPubkey, ProposalHistory>,
    
    /// Exit tracking
    exits: HashSet<ValidatorPubkey>,
}
```

### 4. Metadata Service

Tracks validator information:
- Validator indices
- Activation/exit epochs
- Current status (active/exited/slashed)
- Performance metrics

## Distributed Signing Protocol

### 1. Signing Request
```rust
pub struct SigningRequest {
    /// Type of duty
    pub duty_type: DutyType,
    
    /// Data to sign
    pub signing_data: SigningData,
    
    /// Slot/epoch context
    pub slot: Slot,
    
    /// Validator public key
    pub validator_pubkey: PublicKey,
}
```

### 2. Partial Signature
```rust
pub struct PartialSignature {
    /// Signature share
    pub signature: SignatureShare,
    
    /// Signer index in committee
    pub signer_index: u64,
    
    /// Signing root for verification
    pub signing_root: Hash256,
}
```

### 3. Signature Aggregation
- Collect 2f+1 partial signatures
- Verify each signature share
- Combine using BLS aggregation
- Produce final signature

## Configuration

### Store Configuration
```rust
pub struct ValidatorStoreConfig {
    /// Slashing protection database path
    pub slashing_db_path: PathBuf,
    
    /// Enable slashing protection
    pub enable_slashing_protection: bool,
    
    /// Signature aggregation timeout
    pub aggregation_timeout: Duration,
    
    /// Metadata update interval
    pub metadata_update_interval: Duration,
}
```

### Performance Tuning
- `signature_cache_size`: Cache for recent signatures
- `parallel_signing_ops`: Concurrent signing operations
- `metadata_batch_size`: Batch size for metadata updates

## API Interface

### Signing API
```rust
// Request signature for attestation
let signature = validator_store.sign_attestation(
    attestation_data,
    validator_pubkey,
    slot,
).await?;

// Request block signature
let signature = validator_store.sign_block(
    block,
    validator_pubkey,
    slot,
).await?;
```

### Query API
```rust
// Get validator status
let status = validator_store.get_validator_status(
    validator_pubkey
).await?;

// Check slashing protection
let is_safe = validator_store.check_slashing_protection(
    signing_data,
    validator_pubkey,
).await?;
```

## Slashing Protection Database

### Schema
```sql
-- Attestation records
CREATE TABLE attestations (
    validator_pubkey BLOB PRIMARY KEY,
    source_epoch INTEGER NOT NULL,
    target_epoch INTEGER NOT NULL,
    signing_root BLOB NOT NULL
);

-- Block proposal records
CREATE TABLE proposals (
    validator_pubkey BLOB,
    slot INTEGER,
    signing_root BLOB NOT NULL,
    PRIMARY KEY (validator_pubkey, slot)
);
```

### Import/Export
Support for EIP-3076 interchange format:
```bash
# Export slashing protection data
anchor slashing-protection export --output interchange.json

# Import from another client
anchor slashing-protection import --input interchange.json
```

## Security Considerations

### Key Security
- Key shares never exposed in plaintext
- Encrypted storage at rest
- Memory protection for sensitive data

### Signing Security
- Double-sign prevention
- Signature verification before aggregation
- Consensus required for all signing

### Access Control
- API authentication required
- Rate limiting on signing requests
- Audit logging of all operations

## Metrics

Key metrics exposed:
- `validator_count{status}`: Validators by status
- `signing_operations{type}`: Signatures created
- `slashing_protection_checks`: Protection queries
- `signature_aggregation_time`: Time to aggregate
- `missed_duties{type}`: Failed duty count

## Error Handling

### Common Errors

1. **SlashingProtectionError**
   - Attempting to sign slashable data
   - Check attestation/block history

2. **InsufficientSignatures**
   - Not enough partial signatures received
   - Check peer connectivity and consensus

3. **InvalidKeyShare**
   - Corrupted or invalid key share
   - Verify share integrity

4. **BeaconNodeError**
   - Failed to communicate with beacon node
   - Check beacon node connectivity

## Integration Example

```rust
// Initialize validator store
let store = ValidatorStore::new(config).await?;

// Add validator
store.add_validator(
    validator_pubkey,
    key_share,
    committee_info,
).await?;

// Start duty processing
store.start_duty_service(
    beacon_node,
    duty_callback,
).await?;

// Handle signing request
async fn duty_callback(duty: ValidatorDuty) {
    match duty {
        ValidatorDuty::Attestation(data) => {
            let signature = store.sign_attestation(
                data,
                duty.validator_pubkey,
                duty.slot,
            ).await?;
            
            beacon_node.submit_attestation(
                attestation_with_signature
            ).await?;
        }
        // Handle other duty types...
    }
}
```

## Troubleshooting

### Missing Attestations
1. Check validator is active
2. Verify consensus is working
3. Review signature aggregation logs
4. Check beacon node sync status

### Slashing Protection Failures
1. Review protection database
2. Check for clock skew
3. Verify database isn't corrupted
4. Consider reimporting validators

### Performance Issues
1. Monitor signature aggregation times
2. Check database performance
3. Review concurrent operations
4. Consider increasing timeouts