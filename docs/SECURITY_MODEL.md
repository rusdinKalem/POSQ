# SECURITY MODEL

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menetapkan model keamanan untuk desktop app, local PostgreSQL, license/subscription, backup, update, control plane API, dan admin dashboard.

## 1. Security Objectives

Tujuan keamanan:

- Checkout lokal tetap tersedia saat internet tidak stabil.
- Data operasional merchant tetap berada di lokal secara default.
- Subscription dapat ditegakkan tanpa mengunci data historis.
- Backup cloud tidak dapat dibaca oleh storage provider atau server control plane.
- Server tidak menjadi penyimpan transaksi merchant secara default.
- Update aplikasi tidak dapat dipalsukan.
- Admin internal tidak dapat mengakses data operasional merchant tanpa consent.
- Audit log tersedia untuk tindakan sensitif.

## 2. Trust Boundaries

| Boundary | Trusted Side | Untrusted/Less Trusted Side | Controls |
|---|---|---|---|
| Svelte WebView -> Rust commands | Rust command handlers | UI/WebView input | Command allowlist, validation, RBAC check |
| App -> PostgreSQL local | Rust local service | User-modified config/input | Least privilege DB user, migrations, prepared queries |
| App -> Control plane API | Server API | Network | TLS, signed tokens, idempotency, server-side auth |
| App -> Object storage | Encrypted backup payload | Storage provider | Client-side encryption, checksum, no plaintext upload |
| Admin dashboard -> Control plane DB | Server-side admin services | Browser/admin input | MFA, RBAC, audit log |
| Update server -> Desktop app | Signed release | Network/update source | Signature verification, version policy |
| License server -> Desktop app | Server private signing key | Local device | Signed short-lived token, public key verification |

## 3. Threat Model

### 3.1 Local User Attempts License Bypass

Threats:

- Modify local license token.
- Change local system date backward.
- Block internet after subscription expires.
- Patch frontend UI to unlock button.
- Call Rust command directly.
- Copy license file to another device.

Controls:

- Server-signed license token.
- Token bound to `device_id`.
- Short token validity, recommended 3-7 days.
- Store `last_server_time`.
- Detect clock rollback.
- Rust command handlers enforce license status.
- UI-only lock is not sufficient.
- Token tampering fails signature validation.

### 3.2 Backup Data Exposure

Threats:

- Backup uploaded in plaintext.
- Backup encryption key stored insecurely.
- Support/admin downloads readable backup.
- Cloud bucket misconfigured.
- BYOS credential leaked.

Controls:

- Client-side encryption before upload.
- Backup manifest with checksum.
- Server stores metadata only.
- Backup encryption key stored in OS secure storage where possible.
- User-held recovery key for MVP.
- Admin dashboard cannot download plaintext backup.
- Provider connection test before enabling cloud backup.
- Credential redaction in logs.

### 3.3 Control Plane Compromise

Threats:

- Attacker steals admin credential.
- Attacker revokes devices.
- Attacker changes subscription status.
- Attacker issues fraudulent license token.
- Tenant data leakage.

Controls:

- MFA for admin users.
- Strong admin RBAC.
- Admin audit log.
- Least privilege service accounts.
- Token signing private key protected in vault/CI secret.
- Rate limit login.
- Tenant scoping middleware.
- Manual override requires reason code.

### 3.4 Desktop App Update Attack

Threats:

- Fake update binary.
- Compromised update endpoint.
- Downgrade attack to vulnerable version.
- Migration destroys local data.

Controls:

- Signed update package.
- Signed update metadata.
- App rejects invalid signature.
- Minimum supported version policy.
- Backup before migration.
- Migration log.
- No migration without pre-migration backup.

### 3.5 Tauri IPC Abuse

Threats:

- Malicious WebView content calls privileged commands.
- Command accepts arbitrary path or shell command.
- XSS in UI becomes local command execution.
- Local file read/write abuse.

Controls:

- Do not load remote UI for core POS.
- Strict Content Security Policy.
- Tauri capabilities/permissions scoped narrowly.
- No generic shell command API.
- No generic file system API exposed to UI.
- Every command validates input and permission.
- Rust command returns structured errors.

## 4. Security Requirements

## 4.1 Desktop App

- Use Tauri command allowlist.
- Disable dangerous generic command patterns.
- Validate all IPC input.
- Enforce RBAC and license checks in Rust, not only Svelte.
- Store secrets using OS secure storage where possible.
- Do not log tokens, passwords, DB passwords, backup keys.
- Detect tampered local license state.

Forbidden:

- `run_shell(command)` style command.
- UI-only permission checks.
- Hardcoded owner/admin IDs.
- Private signing key in desktop app.
- Plaintext backup upload.

## 4.2 Local PostgreSQL

Requirements:

- App-specific DB user.
- Prepared statements or ORM query builder.
- Migration version table.
- Backup before destructive migration.
- Audit log append-only from app perspective.
- Sensitive local config protected.

Tables with sensitive data:

- users
- audit_logs
- payments
- customers if implemented
- device_licenses
- backup_metadata

## 4.3 License and Subscription

Requirements:

- License token signed by server.
- Desktop contains public key only.
- Token bound to merchant, outlet, and device.
- Token has `valid_until`, `paid_until`, and `grace_until`.
- App supports `active`, `grace`, `restricted_expired`, `revoked`, `suspicious_time`.
- Short-lived token lease: recommended 7 days for MVP.
- Clock rollback detection.
- Renewal available from Restricted Expired Mode.

Blocked in Restricted Expired Mode:

- New checkout.
- Refund/void.
- Stock adjustment.
- Purchase/receiving.
- Transfer.
- New F&B order.
- Premium modules.

Allowed in Restricted Expired Mode:

- View old data.
- View reports.
- Export.
- Local backup.
- Restore with confirmation.
- Security update.
- Renewal.

## 4.4 Backup Encryption

Requirements:

- Encrypt backup before leaving device.
- Use authenticated encryption.
- Store checksum and manifest.
- Server stores metadata only.
- Restore verifies checksum before applying.
- Wrong key must not alter current database.
- Pre-restore backup is mandatory.

Recommended key policy:

- MVP: user-held recovery key.
- Business/Enterprise: evaluate optional escrow with explicit consent.

## 4.5 Control Plane API

Requirements:

- TLS required.
- Server-side tenant scoping.
- Idempotency keys for metadata writes.
- Rate limit login and sensitive endpoints.
- Admin actions audited.
- Device revoke does not delete local data.
- Subscription expiry changes license state only.

Server must not store by default:

- orders
- payments
- order_items
- stock_movements
- full customer purchase history
- plaintext backup content

## 4.6 Admin Dashboard

Requirements:

- MFA for admin.
- Role-based admin access.
- Audit log for every sensitive admin action.
- Manual subscription override requires reason.
- Device revoke requires reason.
- Support tools expose metadata, not operational data.
- No default view of order/payment/customer/inventory data.

Admin roles:

| Role | Access |
|---|---|
| Super Admin | Full control plane |
| Billing Admin | Subscription and billing state |
| Support Admin | Merchant/device/license metadata, no destructive actions by default |
| Release Admin | App version/update metadata |
| Read-only Auditor | Read-only audit and metadata |

## 5. Error Codes

Security-sensitive error codes:

| Code | Meaning |
|---|---|
| TOKEN_INVALID | License token signature or format invalid |
| LICENSE_EXPIRED | Subscription expired |
| LICENSE_RESTRICTED_EXPIRED | Action blocked in Restricted Expired Mode |
| DEVICE_REVOKED | Device revoked by control plane |
| CLOCK_ROLLBACK_DETECTED | Local time rollback detected |
| ENTITLEMENT_DENIED | Feature not included in current plan |
| RBAC_DENIED | User lacks permission |
| BACKUP_KEY_INVALID | Backup decryption key invalid |
| UPDATE_SIGNATURE_INVALID | Update signature invalid |
| TENANT_SCOPE_DENIED | Cross-tenant access blocked |

## 6. Security Test Checklist

- Token tampering fails.
- Private signing key absent from app artifact.
- Clock rollback detected.
- Expired mode blocks Rust commands, not only UI.
- Cashier cannot access owner dashboard.
- Cashier cannot refund/void without permission.
- Backup file is encrypted.
- Wrong backup key fails safely.
- Server schema has no default operational transaction tables.
- Admin action creates audit log.
- Device revoke blocks future license refresh but does not delete local data.
- Invalid update signature rejected.
- Migration without backup is blocked.

## 7. Security Review Gates

| Gate | Required Before |
|---|---|
| SR-01 Tauri command review | M3 checkout release |
| SR-02 Local DB privilege review | M2 database foundation complete |
| SR-03 License implementation review | M9 release |
| SR-04 Backup encryption review | M8 release |
| SR-05 Admin dashboard review | M7 release |
| SR-06 Update signing review | M10 release |

## 8. Residual Risks

Known residual risks:

- Fully compromised local machine can still be reverse engineered.
- User-held backup key loss can make encrypted backup unrecoverable.
- Offline grace/lease creates limited delayed enforcement.
- BYOS credential security depends partly on user configuration.
- Hardware printer drivers may introduce external risk.

These are acceptable for MVP if documented, tested, and mitigated with clear UX.
