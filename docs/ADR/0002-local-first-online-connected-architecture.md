# ADR-0002: Local-First, Online-Connected Architecture

Status: Accepted  
Date: 2026-07-05

## Context

The POS will be used in environments where connectivity is not always reliable. Cashier operations must continue during internet outages. At the same time, the business model requires online license validation, subscription renewal, app update distribution, backup status, and SaaS account management.

The updated product direction is that the vendor server is not the default place to store the full operational database of every merchant. Operational data remains local by default. Cloud backup is optional and must be encrypted before leaving the device.

## Decision

Adopt a local-first, online-connected architecture with a control plane server.

Local app responsibilities:

- Checkout.
- Shift open/close.
- Product and price access.
- Inventory movement.
- Receipt preview/print.
- Audit log.
- Local reporting.
- Backup job queue.
- License token verification from cached signed token.

Server responsibilities:

- Merchant account.
- Merchant credentials and security state.
- Device registration and heartbeat.
- Subscription billing state.
- License issuance and renewal.
- Backup metadata.
- Update metadata.
- Admin dashboard for account, credential, device, license, subscription, update, and backup metadata.

Default server non-responsibilities:

- Storing full order records.
- Storing payment history.
- Storing inventory movement history.
- Storing customer purchase history.
- Storing plaintext backup contents.
- Acting as the primary database for checkout or local reports.

## Rationale

This design protects the core business operation: selling at the cashier. If the internet is unavailable, the merchant can still transact. Online services enhance the product but do not become a single point of failure for daily checkout.

Keeping the server as a control plane also reduces privacy exposure, server cost, operational complexity, and support risk. It makes the product clearer: the merchant owns the operational database locally, while the vendor manages subscription, license, update, device, and backup metadata.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Cloud-first checkout | Network failure would stop sales |
| Local-only application | Cannot support subscription, update, license refresh, or managed backup metadata |
| Hybrid but server-authoritative checkout | Still too risky for unstable internet |
| Server stores every merchant transaction by default | Too much privacy, cost, and ownership risk for the current product direction |

## Consequences

Positive:

- Cashier reliability improves.
- Merchant trust improves.
- Server downtime does not stop local sales.
- Operational merchant data ownership remains clear.
- Server cost and data breach blast radius are reduced.

Negative:

- Backup, restore, and key management become critical.
- Multi-device operational sync is not automatic in the default architecture.
- Local data security and backup become critical.

## Implementation Notes

- Local order IDs must be globally unique or client-prefixed.
- UI must always show online/offline/license/backup status.
- Server can reject invalid license, device, or backup metadata requests but must not retroactively break local checkout flow.
- Backup payload must be encrypted client-side before upload to object storage.
- Server should store backup metadata: backup_id, device_id, size, checksum, encrypted flag, created_at, destination type, schema version, restore compatibility version.
- Any future cloud operational sync must be an opt-in module with a separate ADR.

## Test Implications

- Checkout with network disabled.
- Checkout with API down.
- Local backup with network disabled.
- Cloud backup retry when provider is unavailable.
- Backup metadata idempotency.
- Server schema check: no default operational order/payment/inventory tables.
- Local report accuracy before sync.
