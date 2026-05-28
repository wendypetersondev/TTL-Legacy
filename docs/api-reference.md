# TTL-Legacy Backend API Reference

Base URL: `http://localhost:3000`

---

## Reminder Preferences

### POST `/api/vaults/{vault_id}/reminder-preferences`

Create or update reminder preferences for a vault.

**Path Parameters**

| Name       | Type   | Description          |
|------------|--------|----------------------|
| `vault_id` | uint64 | On-chain vault ID    |

**Request Body** (`application/json`)

| Field                  | Type            | Required | Description                                              |
|------------------------|-----------------|----------|----------------------------------------------------------|
| `channels`             | array of string | Yes      | One or more of: `"email"`, `"sms"`, `"push"`            |
| `hours_before_expiry`  | integer (> 0)   | Yes      | Hours before TTL expiry to send the first reminder       |
| `frequency`            | string          | Yes      | How often to repeat: `"once"`, `"daily"`, `"hourly"`    |

**Example Request**

```json
{
  "channels": ["email", "sms"],
  "hours_before_expiry": 48,
  "frequency": "daily"
}
```

**Responses**

| Status | Description                              |
|--------|------------------------------------------|
| 200    | Preferences saved; returns saved object  |
| 422    | Validation error (empty channels, zero hours) |
| 500    | Internal server error                    |

**Example Response (200)**

```json
{
  "vault_id": 42,
  "channels": ["email", "sms"],
  "hours_before_expiry": 48,
  "frequency": "daily"
}
```

---

### GET `/api/vaults/{vault_id}/reminder-preferences`

Retrieve current reminder preferences for a vault.

**Path Parameters**

| Name       | Type   | Description       |
|------------|--------|-------------------|
| `vault_id` | uint64 | On-chain vault ID |

**Responses**

| Status | Description                                  |
|--------|----------------------------------------------|
| 200    | Returns the stored preferences               |
| 404    | No preferences found for this vault          |
| 500    | Internal server error                        |

**Example Response (200)**

```json
{
  "vault_id": 42,
  "channels": ["email", "sms"],
  "hours_before_expiry": 48,
  "frequency": "daily"
}
```

---

## Scheduler Behaviour

The background scheduler polls every 60 seconds. For each vault with stored preferences it:

1. Fetches the vault's TTL remaining (hours) from the Stellar RPC.
2. Compares against `hours_before_expiry`.
3. Fires reminders on the configured channels according to `frequency`:
   - `once` — fires exactly once when TTL enters the window.
   - `daily` — fires every 24 hours while inside the window.
   - `hourly` — fires every hour while inside the window.

Preferences are stored off-chain in the backend SQLite database and are never written to the Soroban contract.
