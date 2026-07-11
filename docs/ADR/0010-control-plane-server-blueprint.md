# ADR-0010: Control Plane Server Blueprint

Date: 2026-07-05  
Status: Accepted

## Context

The POS product is local-first. The desktop app stores operational merchant data locally and must keep checkout working without internet. However, the SaaS business still needs a server for account management, device activation, subscription, license, update metadata, backup metadata, and admin support.

Existing documents define pieces of this server in `API_SPEC.md`, `DATA_MODEL.md`, `BILLING_RENEWAL.md`, and `OBSERVABILITY_SUPPORT.md`. A dedicated server blueprint is needed so implementation agents do not turn the server into a cloud POS database.

## Decision

Create a dedicated server control plane blueprint.

The server will include:

- Control Plane API.
- Server PostgreSQL control database.
- Admin dashboard.
- Background worker.
- License signer.
- Update metadata publisher.
- Backup metadata service.
- Billing/subscription management.
- Admin audit and support metadata.

The server will not store merchant operational transaction data by default.

Forbidden default server data:

- Orders.
- Payments.
- Inventory movements.
- Live stock.
- Customer purchase history.
- Plaintext backups.

Any future operational cloud sync must be enterprise opt-in and require a new ADR.

## Consequences

Positive:

- The SaaS business can manage subscription and device activation.
- Desktop can keep operating offline.
- Merchant operational data ownership remains local-first.
- Server scope is explicit and testable.
- Implementation agents get clear server milestones.

Tradeoffs:

- Server admin dashboard cannot show real-time sales unless a future opt-in sync module exists.
- Support may need diagnostic metadata or user-exported reports instead of direct transaction access.
- Multi-device operational sync remains outside MVP.

## Implementation References

- `SERVER_BLUEPRINT.md`
- `SERVER_IMPLEMENTATION_PLAN.md`
- `SERVER_DATA_MODEL.md`
- `SERVER_API_WORKFLOWS.md`
- `SERVER_ADMIN_DASHBOARD.md`
- `SERVER_SECURITY_DEPLOYMENT.md`
- `SERVER_MATCHING_MATRIX.md`
- `SERVER_TEST_PLAN.md`
- `API_SPEC.md`
- `DATA_MODEL.md`
- `ADR-0008`
- `ADR-0009`

