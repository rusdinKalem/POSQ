# SERVER IMPLEMENTATION PLAN

Project: Aplikasi POS SaaS Indonesia - Server Control Plane  
Purpose: Rencana implementasi server yang matching dengan desktop local-first POS.

## 1. Milestone Overview

| Server Milestone | Name | Maps To Local Milestone | Goal |
|---|---|---|---|
| S0 | Server Architecture Lock | M0 | Kunci scope control plane |
| S1 | Server Scaffold | M7 | API, dashboard, worker skeleton |
| S2 | Server Database Foundation | M7 | Migration control plane DB |
| S3 | Auth and Tenant Scope | M7 | Login, session, tenant isolation |
| S4 | Merchant and Device | M7/M9 | Merchant account, activation, revoke |
| S5 | Subscription and Entitlement | M7/M9 | Plan, status, entitlement matrix |
| S6 | License Issuer | M9 | Signed token, heartbeat, refresh |
| S7 | Update Service | M10 | Version metadata, signed update contract |
| S8 | Backup Metadata Service | M8 | Metadata-only backup API |
| S9 | Admin Dashboard | M7/M9 | Internal SaaS dashboard |
| S10 | Worker Jobs | M8/M9/M10 | Reminder, cleanup, rollout |
| S11 | Security and Observability | M13 | Audit, logs, rate limit, diagnostics |
| S12 | Deployment and Release | M14 | Production deployment readiness |

## 2. S0 - Server Architecture Lock

Tasks:

- Read `SERVER_BLUEPRINT.md`.
- Read `SERVER_MATCHING_MATRIX.md`.
- Read `API_SPEC.md`.
- Read `DATA_MODEL.md`.
- Read `SECURITY_MODEL.md`.
- Read `LICENSE_LIFECYCLE.md`.
- Read ADR-0008, ADR-0009, and ADR-0010.

Acceptance criteria:

- Server scope confirmed as control plane only.
- Forbidden operational tables documented.
- Any proposed cloud operational sync rejected unless new ADR exists.

## 3. S1 - Server Scaffold

Tasks:

- Create `services/control-plane-api`.
- Create `services/control-plane-worker`.
- Create `services/admin-dashboard`.
- Add shared config.
- Add health endpoint.
- Add environment validation.
- Add Docker compose for server PostgreSQL.

Acceptance criteria:

- API boots.
- Worker boots.
- Admin dashboard boots.
- Health endpoint returns version and server_time.
- No business logic yet.

## 4. S2 - Server Database Foundation

Tasks:

- Create server migration runner.
- Create control plane tables.
- Add idempotency table.
- Add admin audit table.
- Add server schema guardrail test.

Required tables:

- merchants
- merchant_users
- admin_users
- devices
- subscriptions
- subscription_events
- plans
- entitlements
- device_licenses
- license_signing_keys
- app_versions
- backup_metadata
- idempotency_keys
- admin_audit_logs
- job_queue

Acceptance criteria:

- Migrations are idempotent.
- Server schema has no default operational POS tables.
- Guardrail test fails if `orders`, `payments`, or `stock_movements` appear.

## 5. S3 - Auth and Tenant Scope

Tasks:

- Implement owner login.
- Implement admin login.
- Implement refresh token.
- Add password hashing.
- Add rate limiting.
- Add tenant scoping middleware.
- Add RBAC middleware.

Acceptance criteria:

- Unauthorized request rejected.
- Cross-tenant access blocked.
- Admin access is separated from merchant owner access.
- Admin mutation creates audit log.

## 6. S4 - Merchant and Device

Tasks:

- Create merchant.
- Create merchant owner credential.
- Activate device.
- Enforce device limit.
- Track heartbeat.
- Revoke device.

Acceptance criteria:

- Desktop can activate device.
- Device count respects plan.
- Revoked device cannot refresh license.
- Device revoke does not delete local merchant data.

## 7. S5 - Subscription and Entitlement

Tasks:

- Create plan catalog.
- Create entitlement snapshots.
- Create subscription state machine.
- Implement manual billing extension.
- Emit subscription events.

Acceptance criteria:

- Admin can extend paid_until.
- Grace/expired state computed correctly.
- Entitlements match `ENTITLEMENT_MATRIX.md`.
- Duplicate billing event does not double-extend subscription.

## 8. S6 - License Issuer

Tasks:

- Generate or load server signing key.
- Store signing key metadata.
- Issue signed license token.
- Refresh license token.
- Add token versioning.
- Add heartbeat runtime mode.
- Add anti-clock-rollback support using server_time and last_server_time_seen.

Acceptance criteria:

- Desktop receives signed token.
- Desktop can verify with public key.
- Token tampering fails.
- Private key is absent from desktop artifacts.
- Active, grace, restricted_expired, revoked, suspicious_time modes are supported.

## 9. S7 - Update Service

Tasks:

- Add app version metadata.
- Add update channel stable/beta.
- Add min supported version.
- Add SHA256 and signature.
- Add critical update flag.

Acceptance criteria:

- Desktop can check update metadata.
- Invalid or missing signature is rejected by desktop.
- Admin can publish metadata with audit log.

## 10. S8 - Backup Metadata Service

Tasks:

- Accept backup metadata.
- Validate encrypted flag.
- Validate checksum and size.
- Store logical storage ref only.
- Enforce idempotency.
- Add metadata dashboard query.

Acceptance criteria:

- Server never receives raw backup payload in metadata endpoint.
- Repeated metadata upload with same idempotency key returns one logical result.
- Backup dashboard shows status, checksum, size, db schema version, and failure reason only.

## 11. S9-S12 - Dashboard, Worker, Security, Deployment

Tasks:

- Build admin dashboard pages from `SERVER_ADMIN_DASHBOARD.md`.
- Implement worker jobs for renewal, retention, idempotency cleanup, and update rollout.
- Add observability, redaction, admin MFA, and audit.
- Prepare staging/production deployment with server DB backup/restore drill.

Acceptance criteria:

- Dashboard operates SaaS business without operational POS data.
- Worker jobs are idempotent.
- Tenant isolation and audit tests pass.
- Deployment runbook and rollback plan exist.

