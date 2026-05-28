# Issue #503: Beneficiary Conditional Acceptance - Quick Reference

## What Was Implemented

Beneficiaries can now accept their vault role **conditionally** with a minimum balance threshold. The vault will only release funds if the balance meets or exceeds the threshold at release time.

## API Quick Start

### Beneficiary Accepts with Threshold
```rust
// Beneficiary accepts only if vault has >= 500,000 stroops
client.accept_with_threshold(&vault_id, &500_000i128)?;
```

### Check Acceptance Status
```rust
if let Some(acceptance) = client.get_beneficiary_conditional_acceptance(&vault_id) {
    println!("Threshold: {}", acceptance.min_balance_threshold);
    println!("Accepted at: {}", acceptance.accepted_at);
}
```

### Release Behavior
```rust
// If balance >= threshold: Release succeeds ✅
// If balance < threshold: Release fails with InsufficientBalance ❌
// If no threshold set: Release proceeds normally ✅
client.trigger_release(&vault_id)?;
```

## Key Points

| Aspect | Details |
|--------|---------|
| **Who Can Set** | Beneficiary only (requires auth) |
| **Threshold Value** | Must be > 0 |
| **When Checked** | At release time (trigger_release) |
| **Comparison** | `balance >= threshold` (inclusive) |
| **Backward Compatible** | Yes - existing vaults unaffected |
| **Immutable** | Yes - cannot change after acceptance |
| **Event Emitted** | `BENEFICIARY_CONDITION_ACCEPTED_TOPIC` |

## Error Codes

| Error | Cause | Fix |
|-------|-------|-----|
| `InvalidAmount` | Threshold <= 0 | Use positive threshold |
| `NotBeneficiary` | Caller is not beneficiary | Call as beneficiary |
| `InsufficientBalance` | Balance < threshold at release | Wait for more deposits |

## Test Coverage

✅ 10 comprehensive tests added:
- Beneficiary-only access
- Threshold validation
- Successful release (threshold met)
- Failed release (threshold not met)
- Normal release (no threshold)
- Exact threshold match
- Event emission
- Timestamp storage
- Query functionality

## Files Modified

1. **`contracts/ttl_vault/src/types.rs`**
   - Added `BENEFICIARY_CONDITION_ACCEPTED_TOPIC`
   - Added `BeneficiaryConditionalAcceptance` struct
   - Added `BeneficiaryConditionalAcceptance(u64)` DataKey variant
   - Updated `ConditionalAcceptanceEntry` with optional threshold field

2. **`contracts/ttl_vault/src/lib.rs`**
   - Added `accept_with_threshold()` function
   - Added `get_beneficiary_conditional_acceptance()` function
   - Added `check_conditional_acceptance_threshold()` helper
   - Updated `trigger_release()` to validate threshold

3. **`contracts/ttl_vault/src/test.rs`**
   - Added 10 new test cases

4. **`docs/beneficiary-conditional-acceptance.md`** (NEW)
   - Complete feature documentation

5. **`README.md`**
   - Updated features list
   - Added documentation link

## Example Workflow

```rust
// 1. Owner creates vault
let vault_id = client.create_vault(&owner, &beneficiary, &100u64, &None)?;

// 2. Owner deposits funds
client.deposit(&vault_id, &owner, &1_000_000i128)?;

// 3. Beneficiary accepts with threshold
client.accept_with_threshold(&vault_id, &500_000i128)?;

// 4. Owner stops checking in, vault expires
// (time passes, TTL expires)

// 5. Release is triggered
client.trigger_release(&vault_id)?;
// ✅ Success! Balance (1M) >= Threshold (500K)

// 6. Beneficiary receives funds
assert_eq!(token_client.balance(&beneficiary), 1_000_000i128);
```

## Integration with Other Features

| Feature | Interaction |
|---------|-------------|
| Vesting Schedules | Threshold checked first |
| Multi-Beneficiary | Applies to primary beneficiary |
| Spending Limits | Applied after threshold |
| Conditional Acceptance | Independent feature |
| Proof of Life | Both checked at release |
| Beneficiary Voting | Both checked at release |

## Performance

- **Storage**: 1 entry per vault (i128 + u64)
- **Lookup**: O(1) persistent storage read
- **Comparison**: O(1) integer comparison
- **Gas**: Minimal overhead

## Security

✅ Beneficiary-only (requires auth)  
✅ Threshold immutable after acceptance  
✅ Enforced at release time (cannot bypass)  
✅ On-chain validation  
✅ No external dependencies  

## Future Enhancements

- [ ] Allow threshold updates before release
- [ ] Multiple threshold conditions (min AND max)
- [ ] Time-based threshold adjustments
- [ ] Oracle-based threshold (e.g., token price)
- [ ] Conditional acceptance with custom logic
