# Issue #502: Beneficiary Conflict Resolution - Quick Reference

## What Was Implemented

Automated conflict resolution when multiple beneficiaries claim the same vault. Beneficiaries can file conflict claims and administrators can resolve disputes.

## API Quick Start

### Beneficiary Files Conflict
```rust
client.file_beneficiary_conflict(&vault_id, &reason)?;
```

### Admin Resolves Conflict
```rust
client.resolve_beneficiary_conflict(&vault_id, &approved_beneficiary)?;
```

### Check Conflict Status
```rust
if let Some(conflict) = client.get_beneficiary_conflict(&vault_id) {
    println!("Claims: {}", conflict.claims.len());
}
```

## Key Points

| Aspect | Details |
|--------|---------|
| **Who Can File** | Beneficiary only (requires auth) |
| **Who Can Resolve** | Admin only (requires auth) |
| **Reason** | Must not be empty |
| **Multiple Claims** | Yes - all claims recorded |
| **Blocks Release** | No - informational only |
| **Event Emitted** | BENEFICIARY_CONFLICT_FILED_TOPIC, BENEFICIARY_CONFLICT_RESOLVED_TOPIC |

## Error Codes

| Error | Cause | Fix |
|-------|-------|-----|
| `InvalidAmount` | Reason is empty | Provide non-empty reason |
| `NotBeneficiary` | Caller is not beneficiary | Call as beneficiary |
| `InvalidBeneficiary` | No conflict exists | File conflict first |

## Test Coverage

✅ 10 comprehensive tests added:
- Beneficiary-only access
- Owner cannot file
- Empty reason validation
- Admin-only resolution
- Non-admin cannot resolve
- No conflict error handling
- Query functionality
- Multiple claims
- Timestamp storage
- Event emission

## Files Modified

1. **`contracts/ttl_vault/src/types.rs`**
   - Added event topics
   - Added BeneficiaryConflict struct
   - Added BeneficiaryConflictClaim struct
   - Added ConflictResolution enum
   - Added DataKey variant

2. **`contracts/ttl_vault/src/lib.rs`**
   - Added file_beneficiary_conflict() function
   - Added resolve_beneficiary_conflict() function
   - Added get_beneficiary_conflict() function
   - Updated imports

3. **`contracts/ttl_vault/src/test.rs`**
   - Added 10 new test cases

4. **`README.md`**
   - Updated features list
   - Added documentation link

5. **`docs/beneficiary-conflict-resolution.md`** (NEW)
   - Complete feature documentation

## Example Workflow

```rust
// 1. Beneficiary files conflict
let reason = String::from_str(&env, "Another party claims to be beneficiary");
client.file_beneficiary_conflict(&vault_id, &reason)?;

// 2. Admin reviews and resolves
client.resolve_beneficiary_conflict(&vault_id, &beneficiary)?;

// 3. Check resolution
if let Some(conflict) = client.get_beneficiary_conflict(&vault_id) {
    match conflict.resolution {
        ConflictResolution::Approved(addr) => println!("Approved: {}", addr),
        _ => println!("Not resolved"),
    }
}
```

## Integration with Other Features

| Feature | Interaction |
|---------|-------------|
| Release Process | Conflicts don't block release |
| Multi-Beneficiary | Applies to primary beneficiary |
| Disputes | Separate from conflicts |
| Conditional Acceptance | Independent feature |

## Performance

- **Storage**: 1 entry per vault (Vec of claims)
- **Lookup**: O(1) persistent storage read
- **Comparison**: O(n) where n = number of claims
- **Gas**: Minimal overhead

## Security

✅ Beneficiary-only filing (requires auth)  
✅ Admin-only resolution (requires auth)  
✅ Immutable claims after filing  
✅ On-chain audit trail  
✅ No external dependencies  

## Future Enhancements

- [ ] Voting-based resolution
- [ ] Time-based auto-resolution
- [ ] Beneficiary acceptance of resolution
- [ ] Conflict appeal mechanism
- [ ] Legal document integration
