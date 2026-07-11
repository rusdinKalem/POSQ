# TEST PLAN

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Scope: MVP local-first POS dengan Tauri, Svelte, Rust, PostgreSQL lokal, PostgreSQL server sebagai control plane, backup lokal/cloud terenkripsi, license, subscription, update, inventory, shift, reporting, hardware abstraction, F&B basic, dan retail basic.

## 1. Testing Principles

POS harus diuji seperti sistem operasional lapangan, bukan hanya aplikasi CRUD. Fokus utama testing:

- Kasir tetap bisa transaksi saat internet mati.
- Data transaksi tersimpan di PostgreSQL lokal.
- Server tidak menyimpan database operasional merchant secara default.
- Backup lokal dan cloud dapat dibuat, diverifikasi, dan direstore.
- Upload metadata backup tidak membuat record ganda.
- Stock movement benar.
- License aktif, grace, dan expired mode berjalan.
- Restricted Expired Mode memblokir transaksi baru tetapi tetap membuka data lama, export, backup, restore, update keamanan, dan renewal.
- Update dan migration tidak merusak data.
- RBAC dan audit log mencegah penyalahgunaan.
- UI cukup sederhana untuk kasir/owner awam.

## 2. Test Levels

| Level | Target | Tool/Method |
|---|---|---|
| Unit test | Domain logic, calculation, permission, backup/job event builder | Rust test, TypeScript test |
| Integration test | Local DB, migration, control plane API, backup worker | Test database, API test |
| E2E test | Checkout flow, shift, inventory, license | Playwright/Tauri test/manual script |
| Manual UAT | Kasir, owner, inventory, finance workflow | Checklist |
| Failure test | Offline, API down, DB unavailable, migration failure | Controlled simulation |
| Performance test | 5.000 SKU search, 10.000 transaction report | Benchmark script |
| Security test | RBAC, tenant scope, token tampering | Audit checklist |

## 3. MVP Test Gates

| Gate | Must Pass Before |
|---|---|
| TG-01 Technical PoC | Proceeding to core database |
| TG-02 Local DB stable | Proceeding to checkout |
| TG-03 Checkout offline works | Proceeding to backup/control plane integration |
| TG-04 Backup and metadata upload idempotent | Proceeding to cloud backup release |
| TG-05 License behavior proven | Proceeding to subscription release |
| TG-06 Migration backup works | Proceeding to installer/release |
| TG-07 Security audit passes | Pilot merchant |
| TG-08 Performance acceptable | Wider beta |
| TG-09 P0 docs implemented | Antigravity can proceed beyond MVP foundation |
| TG-10 P1 operational docs implemented | Antigravity can implement feature flows without guessing |
| TG-11 P2 operational readiness implemented | Pilot/beta/production readiness |
| TG-12 Cross-agent instructions consistent | Any selected coding agent can start without contradicting ADR/DECISIONS |
| TG-13 Server blueprint implemented | Control plane can support desktop activation, license, billing, update, and backup metadata |
| TG-14 Server hardening implemented | Device-bound license, tenant authorization, BYOS SSRF defense, key separation, and admin step-up pass |

## 4. Core Test Cases

### 4.0 Cross-Agent Documentation Consistency

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| AGENT-T001 | Required handoff exists | Inspect docs | `AGENT_HANDOFF.md`, `AGENTS.md`, `AGENT_EXECUTION_GUIDE.md`, `TASK_BACKLOG.md`, and `PROMPTS_FOR_AGENTS.md` exist | P0 |
| AGENT-T002 | Agent adapters exist | Inspect docs | Claude, Copilot, Cursor, and Devin instruction files exist | P0 |
| AGENT-T003 | No adapter overrides architecture | Compare adapter docs with ADR/DECISIONS | Adapter docs preserve local-first, control-plane-only server, encrypted backup, Restricted Expired Mode | P0 |
| AGENT-T004 | Task IDs are traceable | Inspect `TASK_BACKLOG.md` and Implementation Plan | Task IDs map to milestones M0-M14 | P0 |
| AGENT-T005 | Status update requirement | Inspect agent docs | All agent docs require `STATUS.md` update after session | P0 |

### 4.1 App Startup and Environment

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| APP-T001 | App opens successfully | Start desktop app | App loads dashboard/login | P0 |
| APP-T002 | Rust command callable | Click health check | UI receives Rust command response | P0 |
| APP-T003 | PostgreSQL local available | Run DB health check | Status shows connected | P0 |
| APP-T004 | PostgreSQL local unavailable | Stop DB then open app | App shows actionable setup error | P0 |
| APP-T005 | App version visible | Open about/settings | Current version shown | P1 |

### 4.2 Local Database and Migration

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| DB-T001 | Fresh migration | Run migration on empty DB | All tables created | P0 |
| DB-T002 | Repeat migration | Run migration twice | No duplicate/broken schema | P0 |
| DB-T003 | Seed demo data | Run seed | Merchant, outlet, products created | P0 |
| DB-T004 | Migration with existing data | Create order then migrate | Existing order remains | P0 |
| DB-T005 | Failed migration | Force migration error | Error shown and data intact | P0 |
| DB-T006 | Local schema matches DATA_MODEL | Inspect migration tables/columns | Required local tables and constraints exist | P0 |
| DB-T007 | Server schema guardrail | Inspect server migrations | No default orders/payments/stock_movements operational tables | P0 |
| DB-T008 | PostgreSQL strategy compliance | Inspect installer/config/migrations | MVP uses per-device PostgreSQL and does not implement unapproved outlet server mode | P1 |

### 4.3 Product and Catalog

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| CAT-T001 | Create product | Add SKU, name, price, category | Product saved locally | P0 |
| CAT-T002 | Search product | Search by name/SKU/barcode | Correct product shown | P0 |
| CAT-T003 | Disable product | Mark inactive | Product not shown in checkout | P1 |
| CAT-T004 | 5.000 SKU search | Seed 5.000 products, search | P95 under target | P0 |

### 4.4 Shift

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| SHF-T001 | Checkout without shift | Try checkout before open shift | Checkout blocked | P0 |
| SHF-T002 | Open shift | Input starting cash | Active shift created | P0 |
| SHF-T003 | Close shift | Close after transactions | Shift summary generated | P0 |
| SHF-T004 | Audit shift | Open/close shift | Audit log created | P0 |

### 4.5 Checkout Local-Only

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| CHK-T001 | Cash checkout online | Add item, pay cash | Order, payment, receipt saved | P0 |
| CHK-T002 | Cash checkout offline | Turn off API/network, checkout | Transaction still succeeds locally | P0 |
| CHK-T003 | Cart total accuracy | Add item, qty, discount, tax | Total consistent in cart/order/payment | P0 |
| CHK-T004 | Stock decrement | Checkout product with stock | Stock decreases and movement recorded | P0 |
| CHK-T005 | Receipt preview | Complete checkout | Receipt preview generated | P0 |
| CHK-T006 | Void permission | Cashier tries void without permission | Action denied | P0 |
| CHK-T007 | Refund permission | Cashier tries refund without permission | Action denied | P0 |

### 4.6 Inventory

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| INV-T001 | Stock in | Add stock to product | Stock movement created | P0 |
| INV-T002 | Adjustment with reason | Adjust stock with reason | Adjustment saved and audited | P0 |
| INV-T003 | Adjustment without reason | Try adjustment without reason | Action blocked | P0 |
| INV-T004 | Stock opname | Run stock opname | Difference recorded as movement | P1 |
| INV-T005 | Low stock | Reduce stock below minimum | Low stock warning appears | P1 |

### 4.7 RBAC and Audit

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| SEC-T001 | Cashier access owner dashboard | Login as cashier | Access denied | P0 |
| SEC-T002 | Inventory admin checkout | Login as inventory admin | Checkout denied unless permission exists | P0 |
| SEC-T003 | Manager refund approval | Manager approves refund | Audit log records approval | P0 |
| SEC-T004 | Role change | Owner changes permission | Audit log created | P0 |

### 4.8 Backup, Restore, and Metadata Upload

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| BAK-T001 | Manual local backup | Create backup to local folder | Backup file created with manifest and checksum | P0 |
| BAK-T002 | Backup before migration | Trigger migration | Pre-migration backup is created first | P0 |
| BAK-T003 | Restore local backup | Restore backup to clean local DB | Orders, products, users, inventory restored correctly | P0 |
| BAK-T004 | Encrypted backup | Enable encryption and create backup | Backup cannot be read as plaintext | P0 |
| BAK-T005 | Wrong restore key | Restore encrypted backup with wrong key | Restore fails safely without altering current DB | P0 |
| BAK-T006 | Managed cloud backup | Configure managed cloud and run backup | Encrypted backup uploaded and metadata recorded | P1 |
| BAK-T007 | BYOS connection test | Configure S3-compatible credentials | App validates bucket/access before enabling schedule | P1 |
| BAK-T008 | Cloud provider unavailable | Force upload failure | Backup job fails with retry status; checkout unaffected | P0 |
| BAK-T009 | Metadata idempotency | Submit same backup metadata 5 times | Server stores one logical backup record | P0 |
| BAK-T010 | Server metadata only | Inspect control plane DB after backup | No order/payment/inventory rows exist on server | P0 |
| BAK-T011 | Retention policy | Run cleanup policy | Old backups follow plan retention rules | P1 |
| BAK-T012 | Pre-restore safety | Restore over existing DB | App creates pre-restore backup and requires confirmation | P0 |

### 4.8A Control Plane API and Admin Dashboard

For full server test coverage, use `SERVER_TEST_PLAN.md` as the source of truth. The table below is the desktop-facing subset.

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| CP-T001 | Merchant account create | Create merchant in dashboard/API | Merchant exists without operational DB tables | P0 |
| CP-T002 | Owner credential reset | Trigger password reset | Credential flow succeeds and audit log is created | P0 |
| CP-T003 | Device activation | Activate new desktop device | Device registered and signed license issued | P0 |
| CP-T004 | Device revoke | Revoke active device | Future license refresh blocked; local data not deleted | P0 |
| CP-T005 | Subscription plan change | Change Starter to Growth | Entitlement token reflects new backup/device features | P0 |
| CP-T006 | Expired subscription | Mark subscription expired | App enters configured expired/grace behavior | P0 |
| CP-T007 | Update channel | Assign device to stable/beta channel | Version check returns correct update metadata | P1 |
| CP-T008 | Backup metadata dashboard | Open merchant backup tab | Admin sees last backup status, size, checksum, and failure reason only | P1 |
| CP-T009 | Admin audit log | Admin changes subscription/license | Action is recorded with actor, time, target, reason | P0 |
| CP-T010 | Tenant isolation | Admin/support accesses different merchant | Unauthorized access is blocked or requires elevated role | P0 |
| CP-T011 | API spec compliance | Run API contract tests from API_SPEC | Endpoint payloads, errors, and idempotency match spec | P0 |
| CP-T012 | Manual billing renewal | Admin extends paid_until, desktop refreshes license | Subscription event/audit created and app returns active | P2 |

### 4.9 License and Subscription

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| LIC-T001 | Device activation | Login owner, activate device | Signed token stored locally | P0 |
| LIC-T002 | Active offline | Disconnect internet with active token | App works according to entitlement | P0 |
| LIC-T003 | Grace period | Set expired but grace active | App still allows configured operations | P0 |
| LIC-T004 | Expired after grace | Set grace_until past | App enters Restricted Expired Mode | P0 |
| LIC-T005 | Renewal | Renew online | Entitlement updates without reinstall | P0 |
| LIC-T006 | Token tampering | Modify token payload | Verification fails | P0 |
| LIC-T007 | Expired blocks checkout | Enter Restricted Expired Mode, try checkout | Checkout is blocked before order/payment write | P0 |
| LIC-T008 | Expired blocks stock adjustment | Enter Restricted Expired Mode, try stock adjustment | Action denied and no stock movement is created | P0 |
| LIC-T009 | Expired allows old data read | Enter Restricted Expired Mode, open historical order/report | Historical data is visible | P0 |
| LIC-T010 | Expired allows export | Enter Restricted Expired Mode, export report/orders | Export succeeds | P0 |
| LIC-T011 | Expired allows local backup | Enter Restricted Expired Mode, run local backup | Backup succeeds with checksum | P0 |
| LIC-T012 | Expired allows renewal | Enter Restricted Expired Mode, open renewal screen and refresh after payment | New signed token restores entitlement | P0 |
| LIC-T013 | Device revoked | Server revokes device, heartbeat runs | Future license refresh blocked; local data not deleted | P0 |
| LIC-T014 | Clock rollback | Set local time backward after token issue | App detects suspicious time and requires online verification | P0 |
| LIC-T015 | Offline beyond token lease | Keep app offline beyond valid_until/grace_until | App enters restricted/suspicious mode according to policy | P0 |
| LIC-T016 | Private key not bundled | Inspect build artifacts/config | Server signing private key is absent from desktop app | P0 |
| LIC-T017 | Entitlement matrix compliance | Run allowed/blocked actions across runtime modes | UI and Rust commands match ENTITLEMENT_MATRIX | P0 |
| LIC-T018 | License lifecycle compliance | Simulate activation, heartbeat, offline, grace, expired, renewal, revoke | Runtime states follow LICENSE_LIFECYCLE | P1 |

### 4.9A Installation and Security

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| INS-T001 | Clean install | Install app on clean Windows target | App launches first-run wizard | P0 |
| INS-T002 | PostgreSQL unavailable | Launch first-run without PostgreSQL | Actionable setup error shown | P0 |
| INS-T003 | First migration | Complete first-run migration | Local schema created | P0 |
| INS-T004 | Activation online | Activate device during first-run | Signed token stored and verified | P0 |
| INS-T005 | Installer does not delete data | Reinstall over existing local DB | Existing data remains | P0 |
| SEC-T005 | Tauri command guard | Call blocked Rust command directly in expired mode | Command returns license error and no DB write occurs | P0 |
| SEC-T006 | Diagnostic redaction | Generate diagnostic bundle | Secrets/tokens/backup keys are redacted | P0 |
| SEC-T007 | Backup plaintext check | Inspect cloud backup payload | Payload is encrypted, not plaintext DB dump | P0 |
| BAK-T013 | Recovery key required | Enable encrypted backup and attempt restore | Restore requires recovery key and wrong key fails safely | P1 |
| UI-T001 | First-run UI flow | Complete first-run screens | Screens follow UI_FLOW and end in valid runtime mode | P1 |
| UI-T002 | Restricted expired screen | Enter restricted_expired | Renewal/export/backup visible; checkout hidden/blocked | P1 |
| ERR-T001 | Structured command error | Trigger license, DB, backup, printer errors | Error has code, severity, user action, retry flag | P1 |
| ERR-T002 | Secret redaction | Trigger diagnostic bundle after failures | Logs redact passwords, tokens, backup keys, BYOS secrets | P1 |
| OBS-T001 | Health screen | Open health dashboard | DB, license, backup, disk, server, update, printer status shown | P2 |
| OBS-T002 | Diagnostic bundle | Generate support bundle | Bundle contains redacted logs and no operational DB dump | P2 |
| CMP-T001 | Rupiah formatting | Create orders/reports | Amounts display/store consistently as rupiah | P2 |
| CMP-T002 | Receipt fields | Complete checkout and preview receipt | Receipt includes merchant/outlet/date/order/cashier/items/totals/payment | P2 |
| CMP-T003 | Manual QRIS record | Record QRIS manual payment | Payment method/report separates QRIS manual from cash | P2 |
| REL-T001 | Release checklist gate | Prepare release candidate | RELEASE_CHECKLIST required gates are passed or documented | P2 |

### 4.10 App Update and Migration Safety

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| UPD-T001 | Update available | Version endpoint returns newer version | App shows update notice | P1 |
| UPD-T002 | Critical update | Version endpoint marks critical | App requires update after policy window | P1 |
| UPD-T003 | Invalid signature | Provide invalid signature | Update rejected | P0 |
| UPD-T004 | Backup before migration | Trigger migration | Backup created before migration | P0 |
| UPD-T005 | Migration failure | Force failure | Backup remains, data not destroyed | P0 |

### 4.11 Reporting

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| RPT-T001 | Daily sales | Complete several orders | Daily report equals order totals | P0 |
| RPT-T002 | Payment breakdown | Mix payment methods | Breakdown equals payment rows | P0 |
| RPT-T003 | Product ranking | Sell multiple products | Ranking sorted correctly | P1 |
| RPT-T004 | Shift report | Close shift | Shift report matches transactions | P0 |

### 4.12 Hardware Abstraction

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| HW-T001 | Receipt preview | Complete checkout | Receipt renders correctly | P0 |
| HW-T002 | Printer adapter mock | Send receipt to mock printer | Print command recorded | P0 |
| HW-T003 | Barcode input | Scan/type barcode | Product added to cart | P0 |

### 4.13 F&B Basic

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| FNB-T001 | Table order | Create order for table | Order linked to table | P1 |
| FNB-T002 | Modifier | Add modifier to item | Modifier appears in order/receipt | P1 |
| FNB-T003 | Kitchen note | Add kitchen note | Note visible in kitchen output | P1 |
| FNB-T004 | Split bill basic | Split table bill | Split totals remain accurate | P1 |

### 4.14 Retail Basic

| ID | Scenario | Steps | Expected Result | Priority |
|---|---|---|---|---|
| RTL-T001 | Barcode checkout | Scan SKU barcode | Product added to cart | P0 |
| RTL-T002 | Return basic | Return paid item | Return recorded and stock adjusted | P1 |
| RTL-T003 | Serial number optional | Product requires serial | Serial captured before sale | P2 |

## 5. Performance Benchmarks

| Benchmark | Target |
|---|---|
| Product search with 1.000 SKU | Feels instant |
| Product search with 5.000 SKU | P95 < 1 second locally |
| Checkout save local | P95 < 2 seconds |
| Daily report with 10.000 transactions | Acceptable under local benchmark target |
| Backup metadata upload 1.000 records | Completes without duplicates |
| Encrypted backup upload 1 GB | Completes or resumes according to policy |
| App startup | No blocking network dependency |

## 5A. Subscription Plan Test Matrix

| Feature | Starter | Growth | Pro | Business | Enterprise |
|---|---|---|---|---|---|
| Local checkout | Must work | Must work | Must work | Must work | Must work |
| Local backup | Manual | Manual/scheduled | Manual/scheduled | Configurable | Configurable |
| Managed cloud backup | Denied or add-on | Allowed | Allowed | Allowed | Custom |
| BYOS backup | Denied | Denied | Allowed | Allowed | Custom |
| Device count | 1 | 2-3 | 5-10 | 10+ | Custom |
| F&B/Retail advanced modules | Denied/basic | Basic | Allowed | Allowed | Custom |
| API/webhook | Denied | Denied | Limited | Allowed | Custom |

Each entitlement must be validated both from UI visibility and backend command enforcement.

## 6. Test Execution Report Template

```text
Date:
Build version:
Tester:
Environment:

Summary:
- Passed:
- Failed:
- Blocked:
- Not run:

Failed tests:
| ID | Scenario | Expected | Actual | Severity | Owner |

Release decision:
[ ] Continue
[ ] Fix first
[ ] Block release
```
