# ADR-0003: Reliable Background Jobs, Metadata Sync, Retry, and Idempotency

Status: Accepted  
Date: 2026-07-05

## Context

The application must accept transactions offline and store operational data locally. The updated architecture does not sync full operational transactions to the vendor server by default. However, the app still needs reliable background communication for device heartbeat, license refresh, subscription status refresh, update checks, backup metadata upload, and optional encrypted cloud backup.

Retrying failed background jobs must not create duplicate backup records, duplicate device events, or inconsistent license/update state.

## Decision

Use a reliable background job engine with:

- `job_outbox` or equivalent for local jobs waiting to be sent.
- `job_inbox` or equivalent only when the control plane sends state back to the device.
- Idempotency keys for backup metadata, device heartbeat, license refresh, subscription refresh, and update checks.
- Retry with exponential backoff.
- Dead-letter status after repeated failures.
- Explicit state application policies for license, subscription, update, and backup metadata.

## Event Lifecycle

1. Local operation creates a background job when online communication is needed.
2. Local app writes the job to `job_outbox`.
3. Background worker sends pending job to server or backup provider.
4. Server checks idempotency key.
5. Server or provider applies the request once.
6. Server returns success, rejection, or retryable error.
7. Local app marks job as sent, failed, retrying, or dead_letter.

## Rationale

The outbox pattern ensures background jobs are not lost when the app restarts or the network fails. Idempotency protects against retry, network timeout, and repeated submission.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Direct API write during checkout | Breaks offline checkout |
| Cron-based table diff | Harder to reason about, less auditable |
| Last-write-wins for everything | Dangerous for stock, payment, and order data |
| Full operational sync to vendor server by default | Violates ADR-0008 and updated product direction |

## Conflict Policy

| Domain | Policy |
|---|---|
| Backup metadata | Idempotency by backup_id and device_id |
| Device heartbeat | Latest valid heartbeat wins |
| Subscription | Server signed status wins when online; cached token/grace applies offline |
| Update metadata | Server signed version metadata wins |
| License | Server signed entitlement wins when online |

## Implementation Notes

Required fields for `job_outbox`:

- job_id
- aggregate_type
- aggregate_id
- merchant_id
- device_id
- job_type
- payload
- idempotency_key
- status
- retry_count
- last_error
- created_at
- sent_at

Required statuses:

- pending
- sending
- sent
- failed
- rejected
- dead_letter

## Test Implications

- Send same backup metadata five times; server stores one logical record.
- Create ten offline transactions; checkout remains local and no operational rows are sent to server.
- Kill app during backup metadata upload and restart.
- Server returns rejection and UI shows actionable status.
- Failed background job must not block new checkout.
