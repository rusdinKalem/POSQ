# IMPLEMENTATION PLAN

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Target: Desktop POS local-first untuk UMKM, retail, dan F&B Indonesia  
Primary stack: Tauri v2, Svelte + TypeScript, Rust, PostgreSQL lokal, PostgreSQL server untuk control plane, object storage opsional untuk backup, signed license token, signed updater  
Source of truth: `docs/PRD.md` dan `docs/WORKFLOW.md`

## 1. Product Intent

Aplikasi ini bukan web POS biasa. Produk harus berjalan sebagai aplikasi desktop local-first yang tetap bisa melakukan transaksi saat internet mati. Database operasional merchant berada di PostgreSQL lokal, bukan di server pusat. Server online berperan sebagai control plane untuk akun, credential, device activation, subscription, license validation, update aplikasi, metadata backup, dan kebijakan entitlement.

Backup data operasional harus menjadi pilihan user. Secara default aplikasi mendukung backup lokal. Untuk backup cloud, aplikasi dapat menyediakan managed cloud backup atau Bring Your Own Storage (BYOS) ke object storage yang dipilih user. Isi backup harus terenkripsi sebelum keluar dari perangkat.

Keputusan produk utama:

- Checkout harian tidak boleh bergantung pada server.
- Data operasional utama disimpan di PostgreSQL lokal.
- Server tidak menyimpan seluruh database transaksi, inventory, customer, dan laporan merchant secara default.
- Sinkronisasi online ke server hanya untuk data control plane: akun, device, license, subscription, update metadata, dan metadata backup.
- Backup cloud bersifat opt-in, terenkripsi client-side, dan tidak dapat dibaca server tanpa kunci user.
- Jika kelak ada multi-device atau cloud operational sync, fitur tersebut harus menjadi modul enterprise opt-in dengan ADR baru.
- License memakai signed token, entitlement, heartbeat, grace period, dan Restricted Expired Mode.
- Saat subscription expired setelah grace period, aplikasi tidak boleh dipakai untuk operasional baru, tetapi data lama, export, backup, restore, update keamanan, dan renewal tetap dapat diakses.
- Update aplikasi harus signed release dan migration database lokal harus diawali backup.
- MVP harus stabil untuk transaksi sebelum fitur marketplace, payment gateway real, dan enterprise workflow dikerjakan.

## 2. Non-Negotiable Requirements

| ID | Requirement | Gate |
|---|---|---|
| NFR-01 | Aplikasi berjalan sebagai Tauri desktop app | App dapat dibuka di Windows target |
| NFR-02 | UI menggunakan Svelte + TypeScript | Build frontend lulus typecheck |
| NFR-03 | Local service menggunakan Rust/Tauri commands | UI dapat memanggil command Rust |
| NFR-04 | PostgreSQL lokal menjadi database utama | Checkout tersimpan lokal tanpa internet |
| NFR-05 | Checkout tidak menunggu server | Server mati, transaksi tetap berhasil |
| NFR-06 | Background job idempotent | Retry 5 kali tidak membuat metadata backup/device/license ganda |
| NFR-07 | License token ditandatangani server | App membaca entitlement dari token |
| NFR-08 | Grace period tersedia | App tetap dapat dipakai offline sesuai kebijakan |
| NFR-09 | Update signed release | App hanya menerima update valid |
| NFR-10 | Migration aman | Backup dibuat sebelum migration |
| NFR-11 | Audit log untuk aksi sensitif | Void, refund, stock adjustment tercatat |
| NFR-12 | RBAC aktif | Kasir tidak bisa mengakses fitur owner/admin |
| NFR-13 | Server hanya control plane secara default | Server tidak memiliki tabel order/payment/inventory lengkap merchant |
| NFR-14 | Backup lokal tersedia | User dapat membuat dan restore backup lokal |
| NFR-15 | Backup cloud opt-in dan terenkripsi | Backup cloud dapat diuji, terenkripsi, dan memiliki checksum |
| NFR-16 | Metadata backup terpisah dari isi backup | Server hanya menyimpan status, lokasi logical, checksum, dan waktu backup |
| NFR-17 | Dashboard admin server tersedia | Admin dapat mengelola merchant, device, license, subscription, update, dan metadata backup |
| NFR-18 | Restricted Expired Mode | Subscription expired memblokir transaksi baru tetapi tetap membuka data lama, export, backup, restore, update keamanan, dan renewal |
| NFR-19 | Anti clock rollback | App mendeteksi manipulasi waktu lokal yang mencoba memperpanjang license |

## 3. Recommended Antigravity Model Assignment

| Phase | Primary Model | Backup Model | Why |
|---|---|---|---|
| PRD/workflow audit | Claude Opus 4.6 | Gemini 3.1 Pro high | Reasoning dan risk discovery |
| ADR creation | Claude Opus 4.6 | Gemini 3.1 Pro high | Keputusan arsitektur besar |
| Scaffold repo | Claude Sonnet 4.6 | Gemini 3.1 Pro low | Coding dan struktur project |
| PostgreSQL local design | Claude Opus 4.6 | Gemini 3.1 Pro high | Data safety dan migration |
| Core POS coding | Claude Sonnet 4.6 | Gemini 3.1 Pro low | Implementasi harian |
| Backup/metadata queue | Claude Opus 4.6 | Gemini 3.1 Pro high | Idempotency, retry, restore safety |
| License/subscription | Claude Opus 4.6 | Gemini 3.1 Pro high | Business rule dan security |
| UI polish | Gemini 3.5 Flash | Gemini 3 Flash | Tugas ringan |
| Security review | Model berbeda dari implementer | Claude Opus 4.6 | Review silang |
| Final MVP review | Claude Opus 4.6 | Gemini 3.1 Pro high | Kelayakan rilis |

## 4. Repository Structure

```text
pos-local-online/
  docs/
    PRD.md
    WORKFLOW.md
    IMPLEMENTATION_PLAN.md
    TEST_PLAN.md
    STATUS.md
    DECISIONS.md
    AGENT_HANDOFF.md
    AGENTS.md
    CLAUDE.md
    DEVIN.md
    AGENT_EXECUTION_GUIDE.md
    TASK_BACKLOG.md
    PROMPTS_FOR_AGENTS.md
    SERVER_BLUEPRINT.md
    SERVER_IMPLEMENTATION_PLAN.md
    SERVER_DATA_MODEL.md
    SERVER_API_WORKFLOWS.md
    SERVER_ADMIN_DASHBOARD.md
    SERVER_SECURITY_DEPLOYMENT.md
    SERVER_MATCHING_MATRIX.md
    SERVER_TEST_PLAN.md
    INSTALLATION_RUNBOOK.md
    SECURITY_MODEL.md
    ENTITLEMENT_MATRIX.md
    API_SPEC.md
    DATA_MODEL.md
    LOCAL_POSTGRESQL_STRATEGY.md
    LICENSE_LIFECYCLE.md
    BACKUP_KEY_RECOVERY.md
    UI_FLOW.md
    ERROR_HANDLING.md
    BILLING_RENEWAL.md
    INDONESIA_COMPLIANCE.md
    OBSERVABILITY_SUPPORT.md
    RELEASE_CHECKLIST.md
    OPEN_QUESTIONS.md
    BUGS.md
    ADR/
    .github/
      copilot-instructions.md
    .cursor/
      rules/
        pos-blueprint.mdc
  apps/
    desktop/
      src/
      src-tauri/
  services/
    control-plane-api/
    control-plane-worker/
  packages/
    shared/
    database/
    backup/
    license/
    updater/
    billing/
  migrations/
    local/
    server/
  scripts/
  tests/
    integration/
    e2e/
```

## 5. Milestone Overview

| Milestone | Goal | Output | Must Pass Before Next |
|---|---|---|---|
| M0 | Read PRD/workflow and lock plan | PRD audit, ADR, implementation plan | No critical ambiguity |
| M1 | Technical PoC | Tauri app, Svelte UI, Rust command, PostgreSQL local connection | App opens and writes dummy transaction locally |
| M2 | Local database foundation | Local schema, migrations, seed, health check | Migration idempotent, no data loss |
| M3 | Core checkout local MVP | Product, cart, cash checkout, receipt, stock movement, audit | Checkout works with server off |
| M4 | Shift, RBAC, audit | Open/close shift, role permission, audit log | Cashier cannot bypass permissions |
| M5 | Inventory MVP | Stock in, adjustment, transfer model, opname, low stock | Every stock change creates movement |
| M6 | Local reporting | Sales summary, payment breakdown, product ranking | Reports match order/payment totals |
| M7 | Control plane API foundation | Auth, merchant, device, subscription, license, update metadata | API accepts health/auth/license request |
| M8 | Backup and metadata sync foundation | Local backup, encrypted cloud backup, metadata upload, retry | Backup/restore works without exposing raw merchant DB to server |
| M9 | License/subscription | Activation, signed token, entitlement, heartbeat, grace period | Expired/grace behavior proven |
| M10 | Update/migration safety | Version check, signed update design, backup before migration | Failed migration does not destroy data |
| M11 | Hardware abstraction | Printer service, barcode scanner handling, receipt template | Receipt preview/print abstraction works |
| M12 | F&B/Retail modes | F&B table/modifier/kitchen basic, retail return/barcode basic | Optional workflows isolated from core checkout |
| M13 | QA hardening | Tests, security audit, performance audit | MVP readiness report approved |
| M14 | Packaging | Windows installer, env docs, release checklist | Clean install works |

## 6. Milestone Details

### M0 - Planning, Audit, and Architecture Lock

Recommended model: Claude Opus 4.6

Tasks:

- Read `docs/PRD.md` and `docs/WORKFLOW.md`.
- Read `docs/AGENT_HANDOFF.md`.
- Read `docs/AGENTS.md`.
- Read `docs/TASK_BACKLOG.md`.
- Read `docs/INSTALLATION_RUNBOOK.md`.
- Read `docs/SECURITY_MODEL.md`.
- Read `docs/ENTITLEMENT_MATRIX.md`.
- Read `docs/API_SPEC.md`.
- Read `docs/DATA_MODEL.md`.
- Read `docs/LOCAL_POSTGRESQL_STRATEGY.md`.
- Read `docs/LICENSE_LIFECYCLE.md`.
- Read `docs/BACKUP_KEY_RECOVERY.md`.
- Read `docs/UI_FLOW.md`.
- Read `docs/ERROR_HANDLING.md`.
- Read `docs/BILLING_RENEWAL.md`.
- Read `docs/INDONESIA_COMPLIANCE.md`.
- Read `docs/OBSERVABILITY_SUPPORT.md`.
- Read `docs/RELEASE_CHECKLIST.md`.
- Create or update ADRs.
- Confirm MVP boundaries.
- Identify technical assumptions and open questions.
- Create `docs/STATUS.md`, `docs/DECISIONS.md`, `docs/OPEN_QUESTIONS.md`, and `docs/TEST_PLAN.md`.

Acceptance criteria:

- Every P0 requirement has a mapped milestone.
- Every high-risk architecture choice has an ADR.
- No coding begins before ADRs are reviewed.

Prompt:

```text
Baca docs/PRD.md dan docs/WORKFLOW.md. Jangan coding dulu.

Buat implementation plan final, risk register, ADR list, milestone map, dan test gate untuk setiap fase.

Pastikan aplikasi tetap local-first: checkout tidak boleh bergantung server.

Wajib baca dokumen P0 sebelum coding:
- INSTALLATION_RUNBOOK.md
- SECURITY_MODEL.md
- ENTITLEMENT_MATRIX.md
- API_SPEC.md
- DATA_MODEL.md

Wajib baca dokumen P1 sebelum implementasi fitur terkait:
- LOCAL_POSTGRESQL_STRATEGY.md
- LICENSE_LIFECYCLE.md
- BACKUP_KEY_RECOVERY.md
- UI_FLOW.md
- ERROR_HANDLING.md

Wajib baca dokumen P2 sebelum pilot/beta/production:
- BILLING_RENEWAL.md
- INDONESIA_COMPLIANCE.md
- OBSERVABILITY_SUPPORT.md
- RELEASE_CHECKLIST.md

Wajib baca lapisan instruksi lintas-agent:
- AGENT_HANDOFF.md
- AGENTS.md
- TASK_BACKLOG.md
- PROMPTS_FOR_AGENTS.md
```

### M0A - Cross-Agent Instruction Layer

Recommended model: Claude Opus 4.6 or equivalent reasoning model

Purpose:

Membuat blueprint dapat dijalankan oleh Antigravity, Claude Code, OpenAI Codex, GitHub Copilot Coding Agent, Cursor Agent, Devin, atau agent lain tanpa mengubah arah produk.

Required files:

- `AGENT_HANDOFF.md`: kontrak universal lintas-agent.
- `AGENTS.md`: instruksi generic untuk coding agent.
- `CLAUDE.md`: instruksi Claude Code.
- `.github/copilot-instructions.md`: instruksi GitHub Copilot Coding Agent.
- `.cursor/rules/pos-blueprint.mdc`: instruksi Cursor.
- `DEVIN.md`: instruksi Devin.
- `AGENT_EXECUTION_GUIDE.md`: panduan prompt gradual dari orientasi sampai release.
- `TASK_BACKLOG.md`: task breakdown executable.
- `PROMPTS_FOR_AGENTS.md`: prompt siap pakai untuk tiap agent.

Acceptance criteria:

- Semua agent diarahkan ke rule local-first yang sama.
- Tidak ada instruksi agent-specific yang boleh mengalahkan ADR dan DECISIONS.
- Semua agent wajib update `STATUS.md` setelah sesi.
- Perubahan arsitektur wajib update `DECISIONS.md` dan ADR.
- Stop conditions untuk perubahan berisiko tertulis eksplisit.

### M1 - Technical Proof of Concept

Recommended model: Claude Sonnet 4.6 or Gemini 3.1 Pro low

Tasks:

- Scaffold Tauri v2 + Svelte + TypeScript.
- Add Rust command callable from Svelte.
- Add local PostgreSQL connectivity check.
- Add dummy migration.
- Insert dummy transaction into PostgreSQL local.

Acceptance criteria:

- App opens on development machine.
- UI can call Rust command.
- Rust command can connect to PostgreSQL local.
- Dummy order can be inserted and read.

Tests:

- `cargo test`
- frontend typecheck
- manual app open
- local DB health check

### M2 - Local Database Foundation

Recommended model: Claude Opus 4.6 for design, Sonnet for implementation

Local tables:

- merchants
- outlets
- users
- roles
- permissions
- products
- categories
- inventory_items
- stock_movements
- orders
- order_items
- payments
- shifts
- audit_logs
- job_outbox
- job_inbox
- backup_jobs
- backup_metadata
- device_licenses
- app_versions

Acceptance criteria:

- Migration is repeatable.
- Seed data works.
- Health check gives actionable errors.
- DB schema supports local checkout, background jobs, license, and audit.
- Local schema follows DATA_MODEL.md.
- Server schema guardrail from DATA_MODEL.md is not violated.
- PostgreSQL deployment follows LOCAL_POSTGRESQL_STRATEGY.md.

### M3 - Core Checkout Local MVP

Recommended model: Claude Sonnet 4.6

Tasks:

- Product list and search.
- Cart.
- Quantity update.
- Discount simple.
- Tax/service charge simple.
- Cash payment.
- Order save.
- Payment save.
- Stock movement.
- Audit log.
- Receipt preview.

Acceptance criteria:

- Checkout works when server/API is unavailable.
- Totals match between cart, order, payment, and receipt.
- Stock decreases after checkout.
- Audit log records checkout.
- No sync is required for local sale completion.

### M4 - Shift, RBAC, and Audit

Tasks:

- Open shift.
- Starting cash.
- Checkout requires active shift.
- Close shift.
- Shift report.
- Role definitions: owner, manager, cashier, inventory, finance.
- Permission guards.
- Audit log for sensitive actions.

Acceptance criteria:

- Cashier cannot checkout without active shift.
- Cashier cannot access owner-only screens.
- Void/refund/stock adjustment require permission.
- Open/close shift is audited.

### M5 - Inventory MVP

Tasks:

- Stock in.
- Stock adjustment with reason.
- Transfer stock model.
- Stock opname.
- Low stock warning.
- Stock movement history.

Acceptance criteria:

- Every inventory change produces stock movement.
- Adjustment requires reason code.
- Product stock in checkout reflects inventory.
- Negative stock policy is explicit.

### M6 - Local Reporting

Tasks:

- Sales summary.
- Payment breakdown.
- Product ranking.
- Shift report.
- Low stock report.
- Export CSV/XLSX plan.

Acceptance criteria:

- Sales total equals order total.
- Payment report equals payment records.
- Reports filter by date, outlet, shift, cashier.

### M7 - Control Plane API Foundation

Recommended model: Gemini 3.1 Pro low or Sonnet

Required server blueprint:

- `docs/SERVER_BLUEPRINT.md`
- `docs/SERVER_IMPLEMENTATION_PLAN.md`
- `docs/SERVER_DATA_MODEL.md`
- `docs/SERVER_API_WORKFLOWS.md`
- `docs/SERVER_ADMIN_DASHBOARD.md`
- `docs/SERVER_SECURITY_DEPLOYMENT.md`
- `docs/SERVER_MATCHING_MATRIX.md`
- `docs/SERVER_TEST_PLAN.md`
- `docs/SERVER_LOCAL_INTEGRATION_SECURITY.md`
- `docs/SERVER_HARDENING_REVIEW.md`
- `docs/ADR/0010-control-plane-server-blueprint.md`
- `docs/ADR/0011-device-bound-license-and-zero-trust-integration.md`

Tasks:

- API service skeleton.
- PostgreSQL server schema for control plane only.
- Auth endpoint.
- Merchant account endpoint.
- Device activation endpoint.
- License issuance endpoint.
- Subscription status endpoint.
- Update metadata endpoint.
- Backup metadata endpoint.
- Admin dashboard API.
- Tenant scoping middleware for control plane resources.
- Device-bound license activation and signed heartbeat.
- BYOS SSRF protection.
- Admin MFA step-up for sensitive actions.
- Separate license signing and update signing keys.

Server-side tables:

- merchants
- merchant_users
- devices
- device_licenses
- subscriptions
- subscription_events
- entitlements
- app_versions
- update_channels
- backup_jobs
- backup_metadata
- audit_logs_admin

Tables that must not exist in the default server schema:

- orders
- order_items
- payments
- stock_movements
- inventory_items with live stock quantity
- full customer transaction history

Acceptance criteria:

- API validates merchant/outlet/device context.
- Server does not become required for local checkout.
- API can activate device and issue signed license token.
- API can store backup metadata without receiving plaintext backup contents.
- Admin dashboard can view merchant/device/subscription/update/backup status.
- API behavior follows API_SPEC.md.
- Control plane server does not expose default operational transaction endpoints.

### M8 - Backup and Metadata Sync Foundation

Recommended model: Claude Opus 4.6

Tasks:

- Create local backup format for PostgreSQL database and app config.
- Encrypt backup before upload.
- Add backup destination selector: local folder, managed cloud backup, or BYOS.
- Add provider adapters for S3-compatible storage.
- Add connection test for configured provider.
- Implement backup job status: pending, running, uploaded, failed, retained, deleted.
- Upload only metadata to server: backup_id, device_id, size, checksum, encrypted flag, created_at, destination type, restore compatibility version.
- Implement retry with backoff.
- Add backup/restore status UI.

Acceptance criteria:

- Local backup can be created without internet.
- Cloud backup can be created when configured.
- Server does not receive plaintext operational database.
- Retrying backup metadata upload does not create duplicate backup records.
- Failed backup does not block local checkout.
- Backup status is visible to user.
- Backup key and restore policy follow BACKUP_KEY_RECOVERY.md.

Test scenarios:

- API offline during checkout.
- Local backup while offline.
- Cloud provider unavailable during scheduled backup.
- Same backup metadata submitted 5 times.
- Restore backup to clean local database.
- Invalid encryption key restore attempt.

### M9 - License and Subscription

Recommended model: Claude Opus 4.6

Tasks:

- Device activation.
- Generate device_id.
- Server signed license token.
- Local token verification.
- Entitlement-based feature flag.
- Heartbeat.
- Grace period.
- Restricted Expired Mode.
- Renewal status refresh.
- Anti clock rollback detection.
- Tamper-evident local license state.
- Renewal screen available from expired mode.

Acceptance criteria:

- App can activate device online.
- App can operate offline during grace period.
- App restricts features outside entitlement.
- Expired subscription blocks new operational actions: checkout, new refund, void, stock adjustment, purchase, transfer, and premium modules.
- Expired subscription does not lock old data, report viewing, export, local backup, restore, or renewal.
- Renewal updates entitlement without reinstall.
- Local clock rollback does not extend subscription.
- Token tampering fails verification.
- Device revoke blocks future license refresh but does not delete local data.
- Runtime mode and plan behavior follow ENTITLEMENT_MATRIX.md.
- Security implementation follows SECURITY_MODEL.md.
- License lifecycle follows LICENSE_LIFECYCLE.md.
- Billing and renewal behavior follows BILLING_RENEWAL.md.

### M9B - License Lifecycle and Startup Gate

Startup flow:

1. App starts locally.
2. License manager loads cached signed license token.
3. App verifies server signature using bundled public key.
4. App checks token `merchant_id`, `outlet_id`, `device_id`, `valid_until`, `grace_until`, `plan`, and entitlement flags.
5. App compares local time against `last_server_time`, `last_seen_local_time`, and token validity.
6. If online, app sends heartbeat to control plane.
7. Server returns refreshed signed token if subscription is active.
8. Server returns grace/expired/revoked state if subscription is not active.
9. App chooses runtime mode: `active`, `grace`, `restricted_expired`, `revoked`, or `suspicious_time`.

Runtime modes:

| Mode | Allowed | Blocked |
|---|---|---|
| active | All features within plan entitlement | Features outside plan |
| grace | Operational features within plan, with warning banner | Features outside plan |
| restricted_expired | Login, read old data, view reports, export, local backup, restore, update security, renewal/payment screen | New checkout, new refund, void, stock adjustment, purchase, transfer, premium modules |
| revoked | Read/export/backup if policy allows, renewal/support screen | Operational features and license refresh |
| suspicious_time | Renewal/support screen, export/backup depending policy | Operational features until online verification |

License token fields:

- merchant_id
- outlet_id
- device_id
- plan
- entitlement flags
- issued_at
- valid_until
- paid_until
- grace_until
- license_status
- app_min_version
- token_version
- signature

Security rules:

- Private signing key must exist only on server/CI secret vault, never in the desktop app.
- Desktop app stores only server public key for verification.
- Token validity should be short, recommended 3-7 days.
- Heartbeat refreshes token when online.
- App must keep `last_server_time` and detect large local clock rollback.
- License state stored locally must be tamper-evident.

### M9A - Subscription Levels and Entitlements

Recommended subscription levels for Indonesian UMKM market:

| Level | Target User | Recommended Monthly Price | Included Entitlements |
|---|---|---|---|
| Starter | Mikro, 1 kasir, 1 outlet | Rp99.000-Rp149.000 | 1 device, local POS, product, checkout, shift, local report, local backup |
| Growth | UMKM aktif, toko kecil, cafe kecil | Rp199.000-Rp299.000 | 2-3 devices, inventory, RBAC basic, managed cloud backup, receipt printer, basic support |
| Pro | Retail/F&B serius | Rp399.000-Rp699.000 | 5-10 devices, multi-outlet basic, advanced inventory, F&B mode, retail return, scheduled cloud backup, BYOS backup, priority support |
| Business | Multi-outlet | Rp999.000-Rp2.500.000 | 10+ devices, role custom, approval flow, advanced report, API/webhook limited, centralized admin, SLA business hours |
| Enterprise | Chain/brand besar | Custom annual contract | Custom device/outlet, SSO optional, dedicated support, private deployment option, custom integration, stronger compliance |

Rules:

- Starter must be useful without cloud dependency.
- Growth should introduce managed cloud backup because this is high-value and easy to understand.
- Pro should unlock BYOS backup and advanced operational modules.
- Business/Enterprise may include optional cloud operational sync only after new ADR and explicit user consent.
- Expired subscription must not block historical data access, export, or local backup.
- Annual plan can use 10-20% discount, but monthly plan should remain available for UMKM cash-flow flexibility.
- Extra device add-on can be priced around Rp49.000-Rp99.000 per device/month depending on plan.
- Managed cloud backup storage over quota should be a transparent add-on, not silently blocked.

### M10 - App Update and Migration Safety

Recommended model: Claude Opus 4.6

Tasks:

- Version endpoint.
- Current version and minimum supported version.
- Signed release verification design.
- Update notification UI.
- Backup before local migration.
- Migration log.
- Failed migration handling.

Acceptance criteria:

- App checks update availability.
- Critical update can be required after grace period.
- Backup is created before migration.
- Migration failure gives clear recovery path.

## 7. Backup Architecture

Backup destinations:

| Destination | Recommended Use | Notes |
|---|---|---|
| Local folder | Default for all plans | Simple, works offline, user can copy to external drive |
| External drive/NAS folder | Small business with local admin | Good for stores with poor internet |
| Managed cloud backup | Growth plan and above | App vendor manages object storage; easiest for user |
| Cloudflare R2 | Managed or BYOS | S3-compatible, practical for object backup |
| Backblaze B2 | Managed or BYOS | Cost-effective object storage option |
| Amazon S3 | BYOS/business | Mature, broad region and lifecycle features |
| Google Cloud Storage | BYOS/business | Good for users already in Google Cloud ecosystem |
| MinIO/S3-compatible storage | Enterprise/private | Useful for private deployment or local server |

Backup rules:

- Backup is encrypted before upload.
- Encryption key must be stored in OS secure storage when available.
- Server stores metadata only, not decrypted backup content.
- Backup metadata must include checksum, app version, DB schema version, created_at, size, destination type, and restore compatibility.
- Restore must require explicit confirmation and create a pre-restore backup first.
- Scheduled backup must not interrupt checkout.
- Backup retention policy must be configurable per plan.

Recommended default:

- Starter: local backup manual.
- Growth: local backup plus scheduled managed cloud backup.
- Pro: managed cloud backup plus BYOS S3-compatible backup.
- Business/Enterprise: configurable retention, multiple destinations, restore drill support.

## 8. Server Admin Dashboard

The server dashboard is not a POS transaction dashboard. It is an operational control plane for the SaaS business.

Admin dashboard modules:

| Module | Purpose |
|---|---|
| Merchant management | Create, suspend, verify, and view merchant profile |
| User and credential management | Manage owner/admin credentials and password reset |
| Device management | Activate, revoke, rename, and inspect device heartbeat |
| License management | Issue signed token, refresh entitlement, view grace/expired status |
| Subscription management | Plan, billing status, renewal, invoice/payment reference, manual override |
| Update management | App version, release channel, minimum supported version, critical update policy |
| Backup metadata | Last backup time, destination type, size, checksum, failed jobs, restore request status |
| Support tools | Read-only diagnostics, device logs metadata, user consent based support session |
| Admin audit log | Track all admin actions |

Dashboard boundaries:

- Admin dashboard must not show merchant order/payment/customer/inventory data by default.
- Any support access to operational data requires explicit merchant export or consent workflow.
- Device revoke must stop future license refresh but must not delete local merchant data.
- Subscription expiry must trigger Restricted Expired Mode: block new operations but allow old data access, export, backup, restore, update security, and renewal.

## 9. Cloud/Server Data Boundary

Data stored on server by default:

- Merchant account profile.
- Login credentials and security state.
- Device registration and heartbeat.
- License token state and entitlement.
- Subscription plan and billing status.
- App update metadata.
- Backup metadata.
- Admin audit log.

Data not stored on server by default:

- Full order records.
- Payment detail history.
- Complete inventory movement history.
- Customer purchase history.
- Local report result data.
- Plaintext backup contents.

If future product direction requires cross-device cloud operational sync, create a new ADR and add opt-in plan terms before implementation.

### M11 - Hardware Abstraction

Tasks:

- Receipt template.
- Print service interface.
- Printer adapter mock.
- Barcode input handling.
- Cash drawer abstraction.
- Kitchen printer abstraction for later F&B.

Acceptance criteria:

- Receipt can be previewed.
- Print adapter can be swapped.
- Barcode input adds product to cart.

### M12 - F&B and Retail Mode Basic

F&B tasks:

- Table map basic.
- Modifier.
- Kitchen note.
- Hold/fire order state.
- Split bill basic.
- Kitchen print/display abstraction.

Retail tasks:

- Barcode checkout.
- Return flow basic.
- Serial number model if needed.
- Price book model.

Acceptance criteria:

- F&B and retail features do not break generic POS checkout.
- Optional modes can be enabled/disabled.
- Core order model supports both.

### M13 - QA Hardening

Recommended model: model different from implementer

Required tests:

- Checkout local-only.
- Checkout with API down.
- Backup/control-plane metadata retry idempotency.
- Stock movement.
- Shift open/close.
- Permission denial.
- License active.
- License grace period.
- License expired.
- Migration backup.
- Report total consistency.
- 5,000 SKU search.
- 10,000 transaction reporting.

Acceptance criteria:

- P0 tests pass.
- Known bugs are documented.
- No critical data-loss risk remains.

### M14 - Packaging and Release

Tasks:

- Windows installer.
- `.env.example`.
- Setup guide.
- Local PostgreSQL setup guide.
- Admin recovery guide.
- Release checklist.

Acceptance criteria:

- Clean install works.
- App can connect to local PostgreSQL.
- App can run local checkout.
- Documentation explains common setup failures.
- Installation behavior follows INSTALLATION_RUNBOOK.md.
- Error and recovery behavior follows ERROR_HANDLING.md.
- UI flow follows UI_FLOW.md.
- Release readiness follows RELEASE_CHECKLIST.md.

## 10. Final MVP Readiness Checklist

| Area | Required Status |
|---|---|
| Tauri app | Runs on Windows |
| Svelte UI | Checkout, product, shift, report screens usable |
| Rust local service | DB, backup, license, hardware commands available |
| PostgreSQL local | Migration and health check stable |
| Checkout | Works fully offline |
| Stock | Decreases and records movement |
| Shift | Required for cashier checkout |
| RBAC | Blocks unauthorized actions |
| Audit | Sensitive actions recorded |
| Background jobs | Backup metadata, heartbeat, license refresh, and update checks are idempotent |
| Backup | Local backup and encrypted cloud backup path verified |
| Server control plane | Account, device, license, subscription, update, and backup metadata working |
| License | Active, grace, expired flows work |
| Restricted Expired Mode | New operations blocked, old data/export/backup/renewal available |
| Update | Version check and migration backup ready |
| Reports | Daily sales and payment totals consistent |
| Installation runbook | Clean install, first-run, activation, health check documented and tested |
| Security model | IPC, license, backup, update, admin dashboard controls reviewed |
| Entitlement matrix | UI and Rust command guards match matrix |
| API spec | Control plane endpoints implemented without operational data drift |
| Data model | Local/server migrations match data ownership boundary |
| Local PostgreSQL strategy | MVP per-device DB strategy documented and followed |
| License lifecycle | Activation, heartbeat, grace, expired, revoke, suspicious time documented and tested |
| Backup key recovery | Recovery key and restore policy documented and tested |
| UI flow | First-run, checkout, backup, restore, expired screens documented |
| Error handling | Structured errors and recovery behavior documented |
| Billing and renewal | Manual MVP renewal and future payment provider path documented |
| Indonesia compliance | Rupiah, receipt, tax/service config, manual QRIS, export, audit checklist documented |
| Observability/support | Logs, diagnostic bundle, health screen, support workflow documented |
| Release checklist | PoC, alpha, pilot, beta, production gates documented |
| Tests | P0 test suite passes |

## 11. Stop Conditions

Stop and ask for human approval if:

- Agent wants to replace PostgreSQL local with SQLite for primary data.
- Agent wants checkout to call server before saving sale.
- Agent wants server to store all merchant operational database by default.
- Agent wants cloud backup without client-side encryption.
- Agent introduces real payment gateway before payment mock is stable.
- Agent changes architecture without ADR.
- Agent deletes migrations or historical data.
- Agent makes destructive command proposal.
- Backup metadata upload has no idempotency.
- License expired mode blocks access to old data.
- Agent implements hard lock that prevents export, backup, or renewal after subscription expiry.
- Agent stores license private signing key inside desktop app.
- Update migration has no backup.

## 12. Development Rule

No feature is considered done unless it has:

- Code implementation.
- Way to run.
- Test or manual verification steps.
- Known limitations.
- Updated `docs/STATUS.md`.
- Updated `docs/NEXT_STEPS.md`.
