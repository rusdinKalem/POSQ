# INSTALLATION RUNBOOK

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menjelaskan cara instalasi, first-run setup, aktivasi device, PostgreSQL lokal, update, backup, restore, dan perilaku subscription expired.

## 1. Installation Principles

Aplikasi POS ini adalah aplikasi desktop local-first. Installer harus membuat aplikasi bisa berjalan di komputer merchant tanpa bergantung pada server untuk checkout harian.

Prinsip wajib:

- Target MVP adalah Windows first.
- PostgreSQL lokal adalah database operasional utama.
- Server hanya control plane: account, credential, device, license, subscription, update, backup metadata.
- Checkout tidak boleh menunggu server.
- App harus bisa dibuka saat offline jika license token masih valid atau masih dalam grace period.
- Saat subscription expired setelah grace period, app masuk Restricted Expired Mode.
- App tidak boleh menghapus data lokal karena subscription expired.

## 2. Target Installation Modes

| Mode | Scope | MVP Status | Notes |
|---|---|---|---|
| Single-device local PostgreSQL | 1 komputer kasir/owner | Required | Default MVP |
| Existing PostgreSQL local | User/admin sudah punya PostgreSQL | Required | App harus bisa connect ke instance yang ada |
| Outlet local server PostgreSQL | 1 PostgreSQL untuk beberapa terminal di outlet | Deferred | Butuh ADR multi-terminal |
| Cloud operational database | Server menyimpan transaksi merchant | Rejected by default | Melanggar local-first/control-plane boundary |

Recommendation:

- MVP gunakan single-device PostgreSQL lokal.
- Multi-terminal/outlet server ditunda sampai checkout lokal stabil.
- Jangan implement cloud operational sync sebelum ADR baru.

## 3. Installer Responsibilities

Installer harus:

1. Memasang aplikasi Tauri desktop.
2. Mengecek dependency minimum.
3. Mengecek apakah PostgreSQL lokal tersedia.
4. Menawarkan setup PostgreSQL lokal jika belum tersedia.
5. Menyimpan konfigurasi koneksi lokal secara aman.
6. Menjalankan first-run wizard saat app pertama dibuka.
7. Menyiapkan shortcut dan app data directory.
8. Tidak membutuhkan login online untuk sekadar membuka setup lokal.

Installer tidak boleh:

- Menghapus database lama tanpa persetujuan eksplisit.
- Menimpa konfigurasi database lama tanpa backup.
- Menyimpan server private signing key.
- Mengirim data transaksi lokal ke server.

## 4. Installation Flow

```text
Download installer
  -> Verify installer signature
  -> Run installer
  -> Install app files
  -> Check local PostgreSQL
  -> Configure DB connection
  -> Launch app
  -> First-run wizard
  -> Local migration
  -> Create local owner/admin
  -> Online activation if available
  -> Receive signed license token
  -> Open POS dashboard
```

## 5. First-Run Wizard

First-run wizard screens:

1. Welcome.
2. Local PostgreSQL check.
3. Database connection setup.
4. Migration confirmation.
5. Merchant/outlet setup.
6. Local owner account setup.
7. Device activation.
8. License token verification.
9. Backup destination recommendation.
10. Finish.

Required behavior:

- User must see actionable error if PostgreSQL is unavailable.
- Migration must be repeatable.
- App must create backup before destructive migration.
- Device activation requires online connection.
- If online activation fails, app may allow setup completion but not operational use unless a valid token exists.

## 6. Local PostgreSQL Setup

Required local database configuration:

```text
host: localhost
port: 5432
database: pos_local
user: app-specific user
ssl: disabled for local default
```

Security requirements:

- Do not use PostgreSQL superuser for normal app runtime.
- Use app-specific DB user with minimum required privileges.
- Store DB credentials using OS secure storage where available.
- Never log DB password.
- DB health check must not print secrets.

Recommended local roles:

| Role | Purpose |
|---|---|
| migration user | Runs migrations during setup/update |
| runtime user | Used by normal app runtime |
| backup user | Optional, used for backup operations |

For MVP, one app-specific user is acceptable if installer complexity must be reduced, but this must be documented as a known limitation.

## 7. Local App Data Paths

App should define platform-specific paths:

```text
config/
logs/
backups/
exports/
cache/
license/
updates/
```

Rules:

- Logs must not contain passwords, private keys, full backup content, or payment secrets.
- Backups must have manifest and checksum.
- License token must be stored securely and tamper-evident.
- Export files are user-readable and should require explicit user action.

## 8. Device Activation Flow

```text
Owner login online
  -> App generates device_id
  -> App generates device keypair if needed
  -> App sends activation request
  -> Server validates merchant/subscription/device limit
  -> Server registers device
  -> Server signs license token
  -> App stores token locally
  -> App verifies token using server public key
```

Activation payload:

```json
{
  "merchant_id": "uuid",
  "outlet_id": "uuid",
  "device_name": "KASIR-01",
  "device_id": "stable-device-id",
  "app_version": "1.0.0",
  "os": "windows",
  "public_key": "optional-device-public-key"
}
```

Activation result:

```json
{
  "device_id": "uuid",
  "license_token": "signed-token",
  "server_time": "2026-07-05T10:00:00Z",
  "next_heartbeat_after_seconds": 3600
}
```

## 9. Startup Gate

Every app startup:

1. Load local configuration.
2. Check PostgreSQL availability.
3. Load signed license token.
4. Verify license token signature.
5. Check device binding.
6. Check `valid_until`, `grace_until`, and `license_status`.
7. Detect local clock rollback.
8. If online, heartbeat to server.
9. Apply runtime mode.

Runtime modes:

| Mode | Behavior |
|---|---|
| active | App runs normally within entitlement |
| grace | App runs with warning and renewal CTA |
| restricted_expired | New operations blocked; old data/export/backup/renewal allowed |
| revoked | Operational use blocked; local data not deleted |
| suspicious_time | Operational use blocked until online verification |

## 10. Restricted Expired Mode Installation Impact

When subscription expires after grace period:

Allowed:

- Open app.
- Login locally.
- View old transactions.
- View reports.
- Export data.
- Create local backup.
- Restore backup with explicit confirmation.
- Open renewal/payment screen.
- Refresh license after renewal.
- Install security/critical updates.

Blocked:

- New checkout.
- Refund/void.
- Stock adjustment.
- Purchase/receiving.
- Stock transfer.
- New F&B table/kitchen order.
- Premium modules.

Installer/update must not remove this capability.

## 11. Update Flow

```text
App checks update metadata
  -> Server returns signed update metadata
  -> App verifies metadata
  -> User confirms or policy requires update
  -> App downloads signed update
  -> App verifies update signature
  -> App creates pre-update/pre-migration backup
  -> App installs update
  -> App runs migration
  -> App writes migration log
```

Rules:

- Invalid signature means update is rejected.
- Migration must never run without backup.
- Failed migration must preserve backup and show recovery steps.
- Security/critical updates remain allowed in Restricted Expired Mode.

## 12. Backup and Restore During Installation

Backup before risky operation:

- Before migration.
- Before restore.
- Before major version update.
- Before database repair.

Restore flow:

```text
Select backup
  -> Verify manifest
  -> Verify checksum
  -> Verify encryption key if encrypted
  -> Create pre-restore backup
  -> Stop write operations
  -> Restore database
  -> Run compatibility check
  -> Restart app
```

Restore must require explicit confirmation.

## 13. Health Checks

Health check categories:

| Check | Required |
|---|---|
| App version | yes |
| PostgreSQL connection | yes |
| Migration status | yes |
| License token validity | yes |
| Clock rollback state | yes |
| Backup destination | yes |
| Disk free space | yes |
| Printer config | optional |
| Server connectivity | optional for checkout |

Health check result must be actionable:

```text
OK
WARNING
BLOCKED
ACTION_REQUIRED
```

## 14. Repair and Recovery

Common recovery actions:

- Re-run DB health check.
- Re-run failed migration after backup.
- Change PostgreSQL connection.
- Restore from local backup.
- Re-activate device.
- Refresh license.
- Export data from Restricted Expired Mode.
- Contact support with redacted diagnostic bundle.

Diagnostic bundle must redact:

- Passwords.
- License token raw value.
- Backup encryption key.
- Customer personal data unless user explicitly exports it.
- Full payment details.

## 15. Acceptance Criteria

Installer is acceptable when:

- Clean Windows install works.
- App opens after install.
- PostgreSQL local health check works.
- Migration creates local schema.
- Dummy order can be stored locally.
- Device activation returns signed token.
- App works offline with valid token.
- App enters grace when policy requires.
- App enters Restricted Expired Mode after grace.
- Restricted Expired Mode blocks checkout but allows export/backup/renewal.
- Update rejects invalid signature.
- Migration creates backup first.
