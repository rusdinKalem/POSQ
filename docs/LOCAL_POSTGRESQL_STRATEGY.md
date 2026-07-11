# LOCAL POSTGRESQL STRATEGY

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan strategi deployment PostgreSQL lokal untuk MVP, beta, dan fase multi-terminal.

## 1. Decision Summary

MVP menggunakan PostgreSQL lokal per device.

Artinya:

- Setiap komputer POS memiliki database PostgreSQL lokal sendiri.
- Checkout, shift, inventory, report, audit, license cache, dan backup berjalan dari database lokal tersebut.
- Server tetap control plane, bukan database transaksi.
- Multi-terminal dengan satu database outlet ditunda sampai fondasi single-device stabil.

## 2. Why Per-Device for MVP

Alasan:

- Instalasi lebih sederhana.
- Risiko concurrency multi-terminal lebih rendah.
- Checkout offline lebih mudah dijamin.
- Migration dan backup lebih mudah diuji.
- Cocok untuk target awal UMKM dan kasir tunggal.
- Mengurangi scope sebelum core POS stabil.

Tradeoff:

- Tidak cocok untuk beberapa kasir aktif di outlet yang sama jika harus berbagi stok real-time.
- Data antar device tidak otomatis sama.
- Multi-outlet/multi-terminal perlu desain tambahan.

## 3. Deployment Phases

| Phase | Mode | Status | Notes |
|---|---|---|---|
| MVP | PostgreSQL per device | Required | Single cashier/device |
| Beta | Optional outlet local server | Planned | Butuh ADR multi-terminal |
| Business | Outlet local server + multiple terminals | Future | Butuh LAN discovery, locking, backup central |
| Enterprise | Outlet local server + optional cloud operational sync | Future | Butuh ADR cloud operational sync |

## 4. MVP Architecture

```text
Tauri Desktop App
  -> Rust local service
  -> PostgreSQL local on same machine
  -> Control plane API for license/subscription/update/backup metadata
  -> Object storage optional for encrypted backup
```

Rules:

- Checkout writes only to local DB.
- Reports read only from local DB.
- Backup reads local DB and app config.
- License state is cached locally.
- Server does not store orders/payments/inventory.

## 5. Local PostgreSQL Installation Options

Antigravity may implement one of these for MVP:

| Option | Recommendation | Notes |
|---|---|---|
| Installer bundles PostgreSQL setup | Preferred for production | Better UX, more installer work |
| App detects existing PostgreSQL | Required fallback | Useful for technical users |
| App starts embedded/portable PostgreSQL | Optional research | Must still be PostgreSQL, not SQLite |
| Manual setup only | Acceptable for PoC | Not acceptable for production UX |

MVP recommendation:

1. PoC: manual PostgreSQL or existing local PostgreSQL.
2. MVP installer: guided PostgreSQL setup/check.
3. Production: installer-managed PostgreSQL with repair flow.

## 6. Database Users

Ideal production model:

| DB User | Purpose |
|---|---|
| migration user | Runs migrations |
| runtime user | Normal app operations |
| backup user | Backup/export if separated |

MVP acceptable simplification:

- One app-specific DB user.
- Must not use PostgreSQL superuser for app runtime.
- Must not log DB credentials.

Open decision:

- Whether MVP uses one app DB user or separate migration/runtime users.

## 7. Connection Configuration

Default local config:

```text
host=localhost
port=5432
database=pos_local
user=pos_app
sslmode=disable
```

Storage:

- Store connection settings in app config.
- Store password in OS secure storage if available.
- Redact password in logs and diagnostics.

## 8. Health Check

Local PostgreSQL health check must verify:

- Server reachable.
- Database exists.
- App user can connect.
- Migration table exists.
- Current schema version is compatible.
- Required tables exist.
- Disk space is sufficient.
- Last backup status is visible.

Health states:

```text
OK
WARNING
ACTION_REQUIRED
BLOCKED
```

## 9. Migration Strategy

Rules:

- All schema changes use versioned migrations.
- Migration must be idempotent where possible.
- App creates backup before destructive migration.
- Migration log is required.
- Failed migration must not destroy current data.
- App must show recovery instructions.

Migration table:

```text
schema_migrations(
  version text primary key,
  name text not null,
  applied_at timestamptz not null,
  checksum text not null
)
```

## 10. Backup Strategy for Local PostgreSQL

Backup required before:

- Migration.
- Restore.
- Major update.
- Repair operation.

Backup must include:

- Database dump.
- App config manifest.
- Schema version.
- App version.
- Device id.
- Checksum.
- Encryption metadata if encrypted.

## 11. Multi-Terminal Future

Outlet local server mode must not be bolted on casually.

Future ADR must decide:

- How terminals discover outlet DB server.
- How shift ownership works.
- How stock locking works.
- How receipt/order numbering works.
- How backup works for shared outlet DB.
- How offline terminal works if LAN server unavailable.
- Whether local server needs service installer.

Until then:

- Do not implement cross-device operational sync.
- Do not use server control plane as transaction database.
- Do not pretend per-device DB supports live multi-cashier stock consistency.

## 12. Acceptance Criteria

- MVP can run with local PostgreSQL per device.
- App can detect unavailable PostgreSQL and show actionable error.
- App can run migration on fresh DB.
- App can store dummy order locally.
- App can backup local DB.
- Reinstall does not delete existing DB.
- App runtime does not use PostgreSQL superuser.
- Server schema does not store operational POS data by default.
