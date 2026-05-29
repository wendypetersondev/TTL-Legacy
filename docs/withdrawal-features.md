# Withdrawal Features

This document describes the withdrawal-related features implemented in TTL-Legacy, including audit trails, batching, notifications, and dispute resolution.

## Issue #569: Withdrawal Audit Trail

### Overview
The withdrawal audit trail tracks all withdrawal attempts (both successful and failed) with comprehensive details for security and compliance purposes.

### Features
- **Complete Tracking**: Records every withdrawal attempt with timestamp, amount, caller, and status
- **Failure Logging**: Captures failed withdrawal attempts with error reasons
- **Persistent Storage**: Audit entries are stored on-chain for permanent record
- **Event Emission**: Emits events for both successful and failed withdrawals

### Data Structure
```rust
pub struct WithdrawalAuditEntry {
    pub vault_id: u64,
    pub caller: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub success: bool,
    pub error_reason: String,
}
```

### API

#### Get Withdrawal Audit Log
```rust
pub fn get_withdrawal_audit_log(env: Env, vault_id: u64) -> Vec<WithdrawalAuditEntry>
```

Retrieves the complete withdrawal audit trail for a vault.

**Parameters:**
- `env`: Soroban environment
- `vault_id`: The vault ID to query

**Returns:** Vector of withdrawal audit entries

**Example:**
```rust
let audit_log = client.get_withdrawal_audit_log(&vault_id);
for entry in audit_log.iter() {
    println!("Withdrawal: {} stroops by {} at {}", 
        entry.amount, entry.caller, entry.timestamp);
}
```

### Events

#### WITHDRAWAL_AUDIT_TOPIC
Emitted for every withdrawal attempt (successful or failed).

**Event Data:**
- `vault_id`: The vault ID
- `caller`: The address attempting withdrawal
- `amount`: The withdrawal amount in stroops
- `success`: Whether the withdrawal succeeded
- `timestamp`: The ledger timestamp

#### WITHDRAWAL_FAILED_TOPIC
Emitted only for failed withdrawal attempts.

**Event Data:**
- `vault_id`: The vault ID
- `caller`: The address attempting withdrawal
- `amount`: The withdrawal amount in stroops
- `error_reason`: The reason for failure

## Issue #570: Withdrawal Batching

### Overview
Withdrawal batching allows multiple withdrawals from different vaults to be processed in a single transaction, reducing gas costs and improving efficiency.

### Features
- **Multi-Vault Withdrawals**: Withdraw from multiple vaults in one transaction
- **Atomic Validation**: All withdrawals are validated before any state changes
- **Audit Trail Integration**: Each withdrawal in the batch is recorded in the audit trail
- **Notification Support**: Each withdrawal generates a notification event

### API

#### Batch Withdraw
```rust
pub fn batch_withdraw(
    env: Env,
    vault_ids: Vec<u64>,
    amounts: Vec<i128>,
    caller: Address,
) -> Result<(), ContractError>
```

Withdraws from multiple vaults owned by the same caller in a single transaction.

**Parameters:**
- `env`: Soroban environment
- `vault_ids`: Vector of vault IDs to withdraw from
- `amounts`: Vector of amounts (in stroops) to withdraw from each vault
- `caller`: The address of the caller (must be the owner of all vaults)

**Returns:** `Ok(())` on success, `Err` on failure

**Errors:**
- `ContractError::Paused`: If the contract is paused
- `ContractError::InvalidAmount`: If vault_ids.len() != amounts.len() or any amount is not positive
- `ContractError::VaultNotFound`: If any vault does not exist
- `ContractError::NotOwner`: If caller is not the owner of any vault
- `ContractError::AlreadyReleased`: If any vault is not in Locked status
- `ContractError::InsufficientBalance`: If any vault balance is less than the requested amount

**Example:**
```rust
let vault_ids = vec![&env, vault_id1, vault_id2, vault_id3];
let amounts = vec![&env, 10_000i128, 20_000i128, 15_000i128];
client.batch_withdraw(&vault_ids, &amounts, &owner)?;
```

### Benefits
- **Gas Efficiency**: Single transaction overhead instead of multiple
- **Atomic Execution**: All-or-nothing semantics
- **Audit Trail**: Each withdrawal is individually tracked
- **Notifications**: Real-time alerts for each withdrawal

## Issue #571: Withdrawal Notifications

### Overview
Withdrawal notifications provide real-time alerts to vault owners whenever a withdrawal is attempted or completed.

### Features
- **Real-Time Alerts**: Immediate notification on withdrawal attempts
- **Comprehensive Data**: Includes caller, amount, and timestamp
- **Event-Based**: Leverages Soroban's event system for off-chain listeners
- **Batch Support**: Notifications for each withdrawal in a batch

### Events

#### WITHDRAWAL_NOTIF_TOPIC
Emitted for every successful withdrawal (both single and batch).

**Event Data:**
- `vault_id`: The vault ID
- `caller`: The address performing the withdrawal
- `amount`: The withdrawal amount in stroops
- `timestamp`: The ledger timestamp

### Off-Chain Integration

Backend services can listen to withdrawal notification events:

```javascript
// Example: Listen for withdrawal notifications
sorobanClient.events()
    .forContract(contractAddress)
    .onEvent('wd_notif', (event) => {
        const { vault_id, caller, amount, timestamp } = event;
        // Send email/SMS notification to vault owner
        notificationService.sendAlert({
            vaultId: vault_id,
            message: `Withdrawal of ${amount} stroops by ${caller}`,
            timestamp: timestamp
        });
    });
```

## Issue #572: Withdrawal Dispute

### Overview
The withdrawal dispute mechanism allows vault owners to challenge unauthorized withdrawals within a 24-hour grace period.

### Features
- **Grace Period**: 24-hour window to dispute withdrawals
- **Dispute Tracking**: Maintains complete dispute history
- **Resolution Mechanism**: Owner can approve or reject disputes
- **Event Logging**: All disputes and resolutions are logged

### Data Structure
```rust
pub struct WithdrawalDispute {
    pub vault_id: u64,
    pub withdrawal_timestamp: u64,
    pub dispute_filed_at: u64,
    pub dispute_expires_at: u64,
    pub status: DisputeStatus,
    pub reason: String,
    pub resolved_at: Option<u64>,
}

pub enum DisputeStatus {
    None,
    Filed,
    Resolved,
}
```

### API

#### File Withdrawal Dispute
```rust
pub fn file_withdrawal_dispute(
    env: Env,
    vault_id: u64,
    caller: Address,
    reason: String,
) -> Result<(), ContractError>
```

Files a dispute for a withdrawal within the grace period (24 hours).

**Parameters:**
- `env`: Soroban environment
- `vault_id`: The vault ID
- `caller`: The vault owner (requires auth)
- `reason`: The reason for the dispute

**Returns:** `Ok(())` on success, `Err` on failure

**Errors:**
- `ContractError::NotOwner`: If caller is not the vault owner

**Example:**
```rust
let reason = String::from_str(&env, "Unauthorized withdrawal detected");
client.file_withdrawal_dispute(&vault_id, &owner, &reason)?;
```

#### Resolve Withdrawal Dispute
```rust
pub fn resolve_withdrawal_dispute(
    env: Env,
    vault_id: u64,
    caller: Address,
    dispute_index: u32,
    approved: bool,
) -> Result<(), ContractError>
```

Resolves a withdrawal dispute.

**Parameters:**
- `env`: Soroban environment
- `vault_id`: The vault ID
- `caller`: The vault owner (requires auth)
- `dispute_index`: The index of the dispute to resolve
- `approved`: Whether to approve the dispute

**Returns:** `Ok(())` on success, `Err` on failure

**Errors:**
- `ContractError::NotOwner`: If caller is not the vault owner
- `ContractError::DisputeFiled`: If dispute index is invalid

**Example:**
```rust
// Approve the dispute (mark as resolved)
client.resolve_withdrawal_dispute(&vault_id, &owner, &0u32, &true)?;
```

#### Get Withdrawal Disputes
```rust
pub fn get_withdrawal_disputes(env: Env, vault_id: u64) -> Vec<WithdrawalDispute>
```

Retrieves all withdrawal disputes for a vault.

**Parameters:**
- `env`: Soroban environment
- `vault_id`: The vault ID to query

**Returns:** Vector of withdrawal disputes

**Example:**
```rust
let disputes = client.get_withdrawal_disputes(&vault_id);
for dispute in disputes.iter() {
    println!("Dispute: {} (Status: {:?})", 
        dispute.reason, dispute.status);
}
```

### Events

#### WITHDRAWAL_DISPUTE_FILED_TOPIC
Emitted when a withdrawal dispute is filed.

**Event Data:**
- `vault_id`: The vault ID
- `caller`: The vault owner
- `timestamp`: When the dispute was filed
- `reason`: The dispute reason

#### WITHDRAWAL_DISPUTE_RESOLVED_TOPIC
Emitted when a withdrawal dispute is resolved.

**Event Data:**
- `vault_id`: The vault ID
- `caller`: The vault owner
- `dispute_index`: The index of the resolved dispute
- `approved`: Whether the dispute was approved

### Grace Period
- **Duration**: 24 hours (86,400 seconds)
- **Start**: When the withdrawal is executed
- **End**: 24 hours after the withdrawal
- **Action**: Disputes must be filed within this window

## Integration Guide

### Backend Integration

```rust
// Record withdrawal with audit trail
let audit_log = client.get_withdrawal_audit_log(&vault_id);

// Check for disputes
let disputes = client.get_withdrawal_disputes(&vault_id);
for dispute in disputes.iter() {
    if dispute.status == DisputeStatus::Filed {
        // Handle pending dispute
        alert_compliance_team(&dispute);
    }
}
```

### Frontend Integration

```javascript
// Listen for withdrawal notifications
const withdrawalNotifications = [];
sorobanClient.events()
    .forContract(contractAddress)
    .onEvent('wd_notif', (event) => {
        withdrawalNotifications.push({
            vaultId: event.vault_id,
            amount: event.amount,
            timestamp: event.timestamp
        });
        updateUI(withdrawalNotifications);
    });

// Display audit trail
async function displayAuditTrail(vaultId) {
    const auditLog = await client.getWithdrawalAuditLog(vaultId);
    return auditLog.map(entry => ({
        amount: entry.amount,
        caller: entry.caller,
        timestamp: entry.timestamp,
        status: entry.success ? 'Success' : 'Failed',
        reason: entry.error_reason
    }));
}
```

## Security Considerations

1. **Audit Trail Immutability**: Audit entries cannot be modified or deleted
2. **Event Logging**: All events are permanently recorded on-chain
3. **Grace Period**: 24-hour dispute window provides time for investigation
4. **Owner-Only Disputes**: Only vault owners can file disputes
5. **Batch Atomicity**: Batch withdrawals are all-or-nothing

## Best Practices

1. **Regular Audits**: Periodically review withdrawal audit trails
2. **Dispute Monitoring**: Monitor pending disputes for security issues
3. **Batch Optimization**: Use batch withdrawals for multiple vaults to save gas
4. **Notification Handling**: Set up backend listeners for withdrawal notifications
5. **Grace Period Awareness**: File disputes within the 24-hour window

## Testing

Comprehensive tests are included in `contracts/ttl_vault/src/test.rs`:

- `test_withdrawal_audit_trail_records_successful_withdrawal`
- `test_withdrawal_audit_trail_records_failed_withdrawal`
- `test_withdrawal_audit_trail_multiple_attempts`
- `test_batch_withdraw_with_audit_trail`
- `test_batch_withdraw_efficiency`
- `test_withdrawal_notification_event_emitted`
- `test_batch_withdrawal_notifications`
- `test_file_withdrawal_dispute`
- `test_resolve_withdrawal_dispute`
- `test_dispute_grace_period`
- `test_multiple_disputes`
- `test_dispute_only_by_owner`
