# OBSERVABILITY AND SUPPORT

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan logging, diagnostics, support workflow, audit, dan observability untuk aplikasi local-first.

## 1. Principles

- Local app must be diagnosable without sending operational data by default.
- Logs must help support but must not leak secrets.
- Merchant consent is required before sharing diagnostic bundle.
- Server control plane stores metadata, not transaction data.
- Critical local failures must be visible to user with actionable guidance.

## 2. Observability Scope

Local app observes:

- App startup.
- Local DB health.
- Migration status.
- Checkout failures.
- Backup job status.
- Restore attempts.
- License state.
- Heartbeat result.
- Update check/install.
- Printer/hardware status.
- Structured command errors.

Server observes:

- Device activation.
- Device heartbeat metadata.
- License refresh.
- Subscription events.
- Backup metadata status.
- Admin dashboard actions.
- Update metadata access.

Server does not observe by default:

- Full order lines.
- Payment details.
- Customer purchase history.
- Inventory movement rows.

## 3. Local Logs

Recommended local log files:

```text
logs/app.log
logs/db.log
logs/migration.log
logs/backup.log
logs/license.log
logs/update.log
logs/hardware.log
logs/security.log
```

Log levels:

```text
DEBUG
INFO
WARN
ERROR
CRITICAL
```

Production default:

- INFO and above.
- DEBUG only when support mode enabled.

## 4. Log Redaction

Must redact:

- Passwords.
- DB password.
- License token raw value.
- Refresh token.
- Backup recovery key.
- BYOS access/secret keys.
- Payment sensitive data.
- Customer personal data unless explicitly exported by user.

Safe identifiers:

- Hashed device id.
- Merchant id.
- App version.
- DB schema version.
- Error code.
- Operation name.
- Timestamp.

## 5. Diagnostic Bundle

Diagnostic bundle is generated only by explicit user action.

Bundle may include:

- App version.
- OS version.
- DB schema version.
- Health check result.
- Recent redacted logs.
- Last backup metadata.
- License mode, not raw token.
- Migration status.
- Printer config summary.

Bundle must not include:

- Raw database dump.
- Backup file.
- Passwords.
- Recovery key.
- Raw license token.
- BYOS secret.
- Customer list.
- Order/payment details unless user explicitly exports separate data.

## 6. Health Dashboard

Local health screen should show:

| Component | Status |
|---|---|
| Local DB | OK/WARNING/BLOCKED |
| Schema migration | OK/ACTION_REQUIRED |
| License | active/grace/restricted/revoked/suspicious |
| Last backup | success/failed/never |
| Disk space | OK/WARNING/BLOCKED |
| Control plane | online/offline |
| Update | current/update available |
| Printer | connected/not configured/error |

## 7. Support Workflow

```text
User reports issue
  -> Support asks for error code
  -> User opens health screen
  -> User generates diagnostic bundle
  -> User reviews/consents
  -> User sends bundle to support
  -> Support analyzes redacted metadata
  -> If operational data needed, user exports specific report manually
```

Support must not ask for:

- PostgreSQL password.
- Backup recovery key.
- License token raw value.
- Remote access without consent.

## 8. Admin Support Tools

Control plane support screen may show:

- Merchant profile.
- Subscription status.
- Device list.
- Last heartbeat.
- App version.
- Runtime mode.
- Last backup metadata.
- License token version.
- Admin audit log.

Must not show by default:

- Orders.
- Payments.
- Inventory movements.
- Customer purchase history.
- Plaintext backup.

## 9. Audit Logs

Local audit logs:

- Checkout.
- Refund.
- Void.
- Discount override.
- Stock adjustment.
- Stock opname.
- Role change.
- Backup restore.
- License activation.
- Migration.

Server admin audit logs:

- Device activation/revoke.
- Subscription override.
- Plan change.
- License refresh/manual issue.
- App version publish.
- Admin credential change.
- Support metadata access.

## 10. Alerts and Warnings

Local user-facing alerts:

- Last backup failed.
- No backup configured.
- Disk space low.
- License near expiry.
- Grace period active.
- Restricted Expired Mode active.
- Update available.
- PostgreSQL health issue.

Do not spam alerts during checkout. Use compact banners and health center.

## 11. Metrics

Local-only metrics:

- Checkout save duration.
- Product search duration.
- Backup duration.
- Restore duration.
- Migration duration.
- Error counts by code.

Server metadata metrics:

- Device heartbeat count.
- License refresh count.
- Backup metadata success/failure.
- Update check count.
- Admin actions.

No operational transaction analytics on server by default.

## 12. Acceptance Criteria

- App has structured local logs.
- Diagnostic bundle redacts secrets.
- Health screen shows local DB, license, backup, update, disk, printer status.
- Support dashboard shows metadata only.
- Admin actions create audit logs.
- User can export specific data separately when needed.
- Restricted Expired Mode still allows diagnostics, export, and backup.
