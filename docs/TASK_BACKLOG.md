# TASK BACKLOG

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Break the blueprint into executable tasks for Antigravity and other coding agents.

## 1. Task Rules

- Work in order unless a human explicitly reprioritizes.
- Each task must preserve local-first checkout.
- Each task must update `STATUS.md`.
- Architecture changes require `DECISIONS.md` and ADR update.
- Tests are required for data, license, backup, migration, and permission behavior.

## 2. M0 - Planning Lock

| ID | Task | Acceptance Criteria |
|---|---|---|
| M0-001 | Read all blueprint docs | Agent confirms required docs read |
| M0-002 | Validate open blockers | Blockers copied into issue tracker or implementation notes |
| M0-003 | Confirm MVP scope | No marketplace/payment gateway/multi-terminal work starts before core POS |
| M0-004 | Confirm local-first rules | Agent reports exact guardrails before coding |

## 3. M1 - Technical PoC

| ID | Task | Acceptance Criteria |
|---|---|---|
| M1-001 | Scaffold Tauri v2 + Svelte + TypeScript | App opens in dev mode |
| M1-002 | Add Rust health command | UI calls Rust command and receives response |
| M1-003 | Add PostgreSQL local config | App can read DB config from local environment/config file |
| M1-004 | Add PostgreSQL health check | UI shows connected/unavailable states |
| M1-005 | Add first local migration | Migration runs idempotently |
| M1-006 | Add dummy order write/read | Dummy order is saved and read from PostgreSQL local |
| M1-007 | Document setup | `STATUS.md` updated with dev run steps |

## 4. M2 - Local Database Foundation

| ID | Task | Acceptance Criteria |
|---|---|---|
| M2-001 | Implement migration runner | Tracks migration version and failure status |
| M2-002 | Implement local tenant seed | Merchant, outlet, owner role, cashier role seeded |
| M2-003 | Implement product/category tables | Schema matches `DATA_MODEL.md` |
| M2-004 | Implement order/payment tables | Local-only operational tables exist locally |
| M2-005 | Implement stock movement tables | Every stock mutation can be traced |
| M2-006 | Implement audit log table | Sensitive action audit can be inserted |
| M2-007 | Implement backup-before-migration | Migration blocked if pre-migration backup fails |

## 5. M3 - Core Checkout MVP

| ID | Task | Acceptance Criteria |
|---|---|---|
| M3-001 | Product search UI | Search by name/SKU/barcode |
| M3-002 | Cart calculation | Qty, discount, tax/service config calculated consistently |
| M3-003 | Open shift requirement | Checkout blocked without active shift |
| M3-004 | Cash checkout | Order/payment/receipt saved locally |
| M3-005 | Offline checkout proof | Server/API off, checkout still succeeds |
| M3-006 | Stock decrement | Checkout writes stock movement |
| M3-007 | Receipt preview | Receipt preview generated from local order |

## 6. M4 - Shift, RBAC, Audit

| ID | Task | Acceptance Criteria |
|---|---|---|
| M4-001 | Role and permission model | Owner/manager/cashier/inventory/finance roles implemented |
| M4-002 | Backend permission guard | Sensitive command denied in Rust/backend layer |
| M4-003 | Open/close shift | Shift totals and cash summary saved |
| M4-004 | Void/refund approval | Unauthorized user blocked; approval audited |
| M4-005 | Audit viewer | Owner can inspect sensitive activity |

## 7. M5 - Inventory MVP

| ID | Task | Acceptance Criteria |
|---|---|---|
| M5-001 | Stock in | Movement created and stock balance updated |
| M5-002 | Stock adjustment | Reason required and audit written |
| M5-003 | Stock opname | Difference converted to adjustment movement |
| M5-004 | Low stock alert | Product below threshold is visible |
| M5-005 | Transfer model | Transfer tables exist, even if single-device MVP limits use |

## 8. M6 - Local Reporting

| ID | Task | Acceptance Criteria |
|---|---|---|
| M6-001 | Sales summary | Totals match local orders/payments |
| M6-002 | Payment breakdown | Cash/manual QRIS/etc summarized |
| M6-003 | Product ranking | Ranking based on local order lines |
| M6-004 | Shift report | Shift report matches close shift values |
| M6-005 | Export report | CSV/PDF export works locally |

## 9. M7 - Control Plane API

| ID | Task | Acceptance Criteria |
|---|---|---|
| M7-001 | Server scaffold | API boots with health endpoint |
| M7-002 | Auth and credential model | Owner/admin login supported |
| M7-003 | Merchant/device model | Device activation supported |
| M7-004 | Subscription model | Plan/status/paid_until stored server-side |
| M7-005 | License issue endpoint | Server returns signed device token |
| M7-006 | Update metadata endpoint | App can check assigned version/channel |
| M7-007 | Schema guardrail test | Server has no operational order/payment/stock tables |

## 10. M8 - Backup and Metadata Sync

| ID | Task | Acceptance Criteria |
|---|---|---|
| M8-001 | Local backup | Backup file, manifest, checksum created |
| M8-002 | Local restore | Restore succeeds after validation |
| M8-003 | Backup encryption | Backup unreadable as plaintext |
| M8-004 | Managed cloud backup adapter | Encrypted backup can upload to selected provider |
| M8-005 | BYOS S3-compatible adapter | User-provided bucket can be validated |
| M8-006 | Backup metadata upload | Server stores metadata only |
| M8-007 | Idempotent retry | Repeated metadata upload creates one logical record |
| M8-008 | Restore safety | Pre-restore backup and confirmation required |

## 11. M9 - License and Subscription

| ID | Task | Acceptance Criteria |
|---|---|---|
| M9-001 | Device activation UI | Owner activates device |
| M9-002 | Local token verification | App verifies signed token with public key |
| M9-003 | Heartbeat refresh | App refreshes token when online |
| M9-004 | Grace period | App remains usable within policy |
| M9-005 | Restricted Expired Mode | New operational writes blocked, old data/export/backup/renewal allowed |
| M9-006 | Anti-clock-rollback | Suspicious time requires online verification |
| M9-007 | Device revoke | Revoked device cannot refresh license |

## 12. M10 - Update and Migration Safety

| ID | Task | Acceptance Criteria |
|---|---|---|
| M10-001 | Version check | App reads update metadata |
| M10-002 | Signed update validation | Unsigned/invalid update rejected |
| M10-003 | Migration backup gate | Migration cannot run without backup |
| M10-004 | Failed migration recovery | Existing data remains intact |
| M10-005 | Release channel | Stable/beta channel supported |

## 13. M11-M14 - Release Hardening

| ID | Task | Acceptance Criteria |
|---|---|---|
| M11-001 | Printer abstraction | Receipt print adapter works with mock and ESC/POS target |
| M11-002 | Barcode scanner flow | Barcode input maps to product search |
| M12-001 | F&B basic mode | Table, modifier, kitchen print abstraction implemented |
| M12-002 | Retail return flow | Return creates payment/stock/audit records |
| M13-001 | Security review | Critical findings fixed |
| M13-002 | Performance test | SKU search and report targets pass |
| M14-001 | Windows installer | Clean install verified |
| M14-002 | Release checklist | `RELEASE_CHECKLIST.md` passed |

## 15. Server Track - Control Plane

These tasks are detailed in `SERVER_IMPLEMENTATION_PLAN.md` and must stay aligned with M7-M10.

| ID | Task | Acceptance Criteria |
|---|---|---|
| S0-001 | Lock server architecture | Server confirmed as control plane only |
| S1-001 | Scaffold control-plane API | API health endpoint works |
| S1-002 | Scaffold worker | Worker health/job claim works |
| S1-003 | Scaffold admin dashboard | Admin login page loads |
| S2-001 | Server DB migrations | Control plane tables created |
| S2-002 | Server schema guardrail | Forbidden operational tables absent |
| S3-001 | Auth and tenant scope | Cross-tenant access blocked |
| S4-001 | Merchant/device activation | Device activation works |
| S5-001 | Subscription/entitlement | Plan and entitlement snapshot works |
| S6-001 | License issuer | Signed token returned and desktop-compatible |
| S6-002 | Device-bound license | Token copied to another device fails |
| S6-003 | Signed heartbeat and nonce | Replayed heartbeat rejected |
| S7-001 | Update metadata | Desktop can check signed update metadata |
| S7-002 | Separate update signing | Update key is separate from license key |
| S8-001 | Backup metadata | Metadata stored idempotently, no payload |
| S8-002 | BYOS SSRF protection | Private/internal endpoints rejected |
| S9-001 | Admin dashboard | Merchant/device/subscription/update/backup views work |
| S9-002 | Admin step-up MFA | High-risk actions require step-up |
| S10-001 | Worker jobs | Renewal/retention/update jobs idempotent |
| S11-001 | Server security review | No Critical/High unresolved findings |
| S12-001 | Server deployment readiness | Staging deploy and DB restore drill pass |
