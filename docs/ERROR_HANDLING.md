# ERROR HANDLING

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan standar error handling untuk app lokal, PostgreSQL, license, backup, restore, update, printer, dan control plane.

## 1. Error Handling Principles

- Jangan hilangkan data.
- Jangan buat transaksi ganda.
- Jangan blokir checkout karena server/control plane offline jika license lokal masih valid.
- Error harus actionable.
- Error harus punya kode.
- Secret tidak boleh muncul di log.
- Operasi mutasi harus atomic.

## 2. Error Response Shape

Rust command error:

```json
{
  "code": "LICENSE_RESTRICTED_EXPIRED",
  "message": "Action is blocked because subscription is expired.",
  "severity": "blocked",
  "user_action": "Renew subscription to enable checkout.",
  "safe_to_retry": false,
  "details": {}
}
```

Severity:

```text
info
warning
blocked
critical
```

Retry flag:

```text
safe_to_retry=true|false
```

## 3. Standard Error Codes

| Code | Severity | Meaning |
|---|---|---|
| DB_UNAVAILABLE | blocked | PostgreSQL cannot be reached |
| DB_MIGRATION_FAILED | critical | Migration failed |
| DB_SCHEMA_INCOMPATIBLE | blocked | Schema version incompatible |
| DISK_FULL | blocked | Not enough disk space |
| SHIFT_REQUIRED | blocked | Checkout requires active shift |
| PAYMENT_TOTAL_MISMATCH | blocked | Payment and order total mismatch |
| STOCK_POLICY_BLOCKED | blocked | Stock policy blocks operation |
| RBAC_DENIED | blocked | User lacks permission |
| ENTITLEMENT_DENIED | blocked | Plan does not include feature |
| LICENSE_RESTRICTED_EXPIRED | blocked | Restricted expired mode blocks action |
| DEVICE_REVOKED | blocked | Device revoked |
| CLOCK_ROLLBACK_DETECTED | blocked | Time manipulation suspected |
| TOKEN_INVALID | critical | License token invalid |
| BACKUP_FAILED | warning/critical | Backup failed |
| BACKUP_KEY_INVALID | blocked | Wrong backup key |
| BACKUP_CHECKSUM_MISMATCH | critical | Backup integrity failed |
| RESTORE_FAILED | critical | Restore failed |
| UPDATE_SIGNATURE_INVALID | critical | Update signature invalid |
| PRINTER_UNAVAILABLE | warning | Printer unavailable |
| API_UNAVAILABLE | warning | Control plane unavailable |
| RATE_LIMITED | warning | Too many API attempts |

## 4. PostgreSQL Errors

### DB unavailable

When:

- PostgreSQL service stopped.
- Wrong credential.
- Port unavailable.
- DB missing.

Behavior:

- Show setup/repair screen.
- Do not attempt checkout.
- Do not delete config.
- Offer health check retry.
- Offer connection settings.

### Migration failed

Behavior:

- Stop migration.
- Preserve pre-migration backup.
- Show migration log path.
- Block app operational use until resolved.
- Do not partially continue if schema uncertain.

### Disk full

Behavior:

- Block backup/update/checkout if write safety is uncertain.
- Show required free space.
- Offer export/delete old backup guidance.

## 5. Checkout Errors

Checkout must run in one DB transaction.

If any part fails:

- Rollback order.
- Rollback payment.
- Rollback stock movement.
- Rollback inventory qty update.
- Write error log.
- Do not print receipt as paid.

Common checkout blockers:

- No active shift.
- License restricted.
- Payment total mismatch.
- Product inactive.
- Stock policy violation.
- DB unavailable.

## 6. License Errors

### Token invalid

Behavior:

- Reject token.
- Do not allow operational actions.
- Ask user to reconnect and refresh license.
- Keep old data accessible if policy allows.

### Restricted expired

Behavior:

- Block new operational actions.
- Allow old data/export/backup/renewal.
- Return `LICENSE_RESTRICTED_EXPIRED`.

### Clock rollback detected

Behavior:

- Enter suspicious_time.
- Require online verification.
- Do not silently fix clock.
- Allow support/renewal.

## 7. Backup Errors

### Local backup failed

Possible causes:

- Disk full.
- Permission denied.
- DB dump failed.
- Checksum failed.

Behavior:

- Mark backup job failed.
- Show reason.
- Do not mark backup as successful.
- Do not upload metadata as successful.

### Cloud backup failed

Possible causes:

- Provider unavailable.
- Wrong BYOS credential.
- Network failure.
- Upload interrupted.

Behavior:

- Retry with backoff.
- Keep local backup if created.
- Show last successful backup.
- Do not block checkout.

### Wrong restore key

Behavior:

- Stop restore before modifying current DB.
- Return `BACKUP_KEY_INVALID`.
- Allow retry.

### Checksum mismatch

Behavior:

- Stop restore.
- Return `BACKUP_CHECKSUM_MISMATCH`.
- Mark backup as suspicious/corrupt.

## 8. Restore Errors

Restore must be treated as high-risk.

Required preconditions:

- Backup manifest valid.
- Checksum valid.
- Encryption key valid.
- Schema compatibility checked.
- Pre-restore backup created.

If restore fails:

- Do not leave DB half-restored.
- Attempt rollback or restore pre-restore backup.
- Show recovery steps.
- Log restore failure.

## 9. Update Errors

### Invalid signature

Behavior:

- Reject update.
- Keep current version.
- Show security warning.
- Log event.

### Migration after update fails

Behavior:

- Preserve pre-migration backup.
- Show recovery path.
- Do not run app with unknown schema state.

## 10. Printer and Hardware Errors

Printer unavailable:

- Checkout should still save order/payment locally.
- Receipt preview remains available.
- Print can be retried.
- Printer failure must not rollback paid order after payment committed.

Barcode scanner:

- If scan fails, allow manual search/input.

Cash drawer:

- Failure should be warning, not data loss.

## 11. Control Plane Errors

Control plane unavailable:

- Do not block local checkout if token valid.
- Queue heartbeat/license refresh job.
- Show offline indicator.

Subscription endpoint returns expired:

- Apply runtime mode from signed token/response.
- Enter grace or restricted_expired according policy.

Idempotency conflict:

- Do not retry blindly.
- Mark job rejected.
- Show support/debug status.

## 12. Logging and Diagnostics

Logs should include:

- Error code.
- Timestamp.
- App version.
- DB schema version.
- Device id hash.
- Operation name.
- Safe metadata.

Logs must redact:

- Passwords.
- DB password.
- License token raw value.
- Backup recovery key.
- BYOS secret key.
- Payment sensitive data.

## 13. User-Facing Error Copy

Bad:

```text
Error.
```

Good:

```text
Database lokal tidak dapat dihubungi.
Kode: DB_UNAVAILABLE
Pastikan PostgreSQL berjalan, lalu klik Coba Lagi.
```

```text
Checkout tidak dapat dilanjutkan karena langganan sudah berakhir.
Kode: LICENSE_RESTRICTED_EXPIRED
Perpanjang langganan untuk mengaktifkan kembali checkout. Data lama, export, dan backup tetap tersedia.
```

## 14. Acceptance Criteria

- Every Rust command returns structured errors.
- Every blocked operation has a stable error code.
- Checkout rollback works on partial failure.
- Backup failure does not block checkout.
- Restore failure preserves current/pre-restore data.
- Printer failure does not erase paid order.
- Control plane offline does not block checkout with valid token.
- Logs redact secrets.
- User-facing errors include next action.
