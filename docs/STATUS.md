# STATUS

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Last updated: 2026-07-11  
Current phase: Release Readiness  
Current milestone: M14 - Packaging and Release Completed

## 1. Current Summary

Proyek ini telah menyelesaikan seluruh tahapan pengembangan MVP POSQ (M1-M14). Seluruh fungsionalitas utama seperti kasir offline, manajemen inventaris, pembukuan shift, laporan penjualan, pencatatan audit log, modul F&B & Retail, abstraksi printer, keamanan lisensi, proteksi clock rollback, serta backup lokal AES-256-GCM telah terimplementasi dan diuji melalui static analysis & manual review. Konfigurasi Windows Installer (NSIS) juga telah terpasang di `tauri.conf.json`. Seluruh Svelte 5 a11y warnings dan TypeScript type errors telah dibersihkan secara penuh dan terverifikasi sukses via `npm run check`. Fungsionalitas hardware abstraction (ADR-0012) serta layout kasir adaptif dengan touch targets minimal 48x48px (numpad, cart quantity adjust, checkout) telah diintegrasikan secara penuh untuk mendukung kesiapan perangkat mobile.

Current source documents:

- `docs/PRD.md`
- `docs/WORKFLOW.md`
- `docs/IMPLEMENTATION_PLAN.md`
- `docs/ADR/*`
- `docs/ADR/0012-mobile-vs-desktop-hardware-integration-strategy.md`
- `docs/TEST_PLAN.md`
- `docs/DECISIONS.md`
- `docs/STATUS.md`
- `docs/AGENT_HANDOFF.md`
- `docs/AGENTS.md`
- `docs/CLAUDE.md`
- `docs/DEVIN.md`
- `docs/AGENT_EXECUTION_GUIDE.md`
- `docs/TASK_BACKLOG.md`
- `docs/PROMPTS_FOR_AGENTS.md`
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
- `docs/.github/copilot-instructions.md`
- `docs/.cursor/rules/pos-blueprint.mdc`
- `docs/INSTALLATION_RUNBOOK.md`
- `docs/SECURITY_MODEL.md`
- `docs/ENTITLEMENT_MATRIX.md`
- `docs/API_SPEC.md`
- `docs/DATA_MODEL.md`
- `docs/LOCAL_POSTGRESQL_STRATEGY.md`
- `docs/LICENSE_LIFECYCLE.md`
- `docs/BACKUP_KEY_RECOVERY.md`
- `docs/UI_FLOW.md`
- `docs/ERROR_HANDLING.md`
- `docs/BILLING_RENEWAL.md`
- `docs/INDONESIA_COMPLIANCE.md`
- `docs/OBSERVABILITY_SUPPORT.md`
- `docs/RELEASE_CHECKLIST.md`

## 2. Milestone Progress

| Milestone | Name | Status | Notes |
| :--- | :--- | :--- | :--- |
| M0 | Planning, Audit, and Architecture Lock | Completed | PRD, implementation plan, ADR, dan test plan siap |
| M1 | Technical PoC | Completed | Integrasi Tauri/Svelte/Rust/PostgreSQL terbukti sukses |
| M2 | Local Database Foundation | Completed | Skema, seed, dan backup-before-migration telah dibuat sesuai DATA_MODEL.md |
| M3 | Core Checkout Local MVP | Completed | Svelte UI (Product, Cart, Receipt), Rust backend, stock validation terimplementasi |
| M4 | Shift, RBAC, and Audit | Completed | Implementasi auth, close shift, refund, dan UI untuk role & audit |
| M5 | Inventory MVP | Completed | Svelte UI inventory, low stock alert, stock movement recording, dan opname/adjustment terimplementasi |
| M6 | Local Reporting | Completed | Report summary, product ranking, CSV export, dan RBAC report.view terintegrasi |
| M7 | Control Plane API Foundation | Completed | Axum API scaffold, Auth/Sub stubs, pos_server DB tersendiri |
| M8 | Backup and Metadata Sync Foundation | Completed | Local backup/restore (pg_dump), AES-256-GCM encryption, Svelte UI |
| M9 | License and Subscription | Completed | Offline-first license parsing, Grace/Restricted mode layout guard |
| M10 | App Update and Migration Safety | Completed | Safe migration (backup-before-migrate), version check, UI Update |
| M11 | Hardware Abstraction | Completed | Printer mock, receipt preview UI, generic hardware command |
| M12 | F&B and Retail Mode Basic | Completed | Kitchen Print UI/Logic, Retail Return Flow (Mocked) |
| M13 | QA Hardening & Security Review | Completed | Static security review, license bypass fix, clock rollback fix, privacy fix |
| M14 | Packaging and Release | Completed | Tauri Windows Installer (NSIS) terkonfigurasi, Release Checklist selesai |

## 3. Completed Artifacts

| Artifact | Status |
| :--- | :--- |
| PRD | Prepared |
| Implementation plan | Prepared |
| ADR-0001 Tech Stack | Prepared |
| ADR-0002 Local-first Architecture | Prepared |
| ADR-0003 Reliable Background Jobs and Metadata Sync | Prepared |
| ADR-0004 License and Subscription | Prepared |
| ADR-0005 Update and Migration Safety | Prepared |
| ADR-0006 Domain Boundaries | Prepared |
| ADR-0007 Security/RBAC/Audit | Prepared |
| ADR-0008 User-Owned Backup and Control Plane Server | Prepared |
| ADR-0009 Restricted Expired Mode and License Enforcement | Prepared |
| ADR-0010 Control Plane Server Blueprint | Prepared |
| ADR-0011 Device-Bound License and Zero-Trust Integration | Prepared |
| ADR-0012 Mobile vs. Desktop Hardware Integration Strategy | Prepared |
| Server blueprint | Prepared |
| Server implementation plan | Prepared |
| Server data model | Prepared |
| Server API workflows | Prepared |
| Server admin dashboard blueprint | Prepared |
| Server security and deployment blueprint | Prepared |
| Server matching matrix | Prepared |
| Server test plan | Prepared |
| Server-local integration security | Prepared |
| Server hardening review | Prepared |
| Installation runbook | Prepared |
| Security model | Prepared |
| Entitlement matrix | Prepared |
| Control plane API spec | Prepared |
| Data model | Prepared |
| Local PostgreSQL strategy | Prepared |
| License lifecycle | Prepared |
| Backup key recovery | Prepared |
| UI flow | Prepared |
| Error handling | Prepared |
| Billing and renewal | Prepared |
| Indonesia compliance notes | Prepared |
| Observability and support | Prepared |
| Release checklist | Prepared |
| Test plan | Prepared |
| Decisions log | Prepared |
| Cross-agent handoff | Prepared |
| Generic agent instructions | Prepared |
| Claude Code instructions | Prepared |
| GitHub Copilot instructions | Prepared |
| Cursor rules | Prepared |
| Devin instructions | Prepared |
| Gradual agent execution guide | Prepared |
| Executable task backlog | Prepared |
| Reusable agent prompts | Prepared |

## 4. Current Risks

| Risk | Severity | Status | Mitigation |
| :--- | :--- | :--- | :--- |
| Local PostgreSQL installation may be difficult for users | High | Open | Build installer, health check, setup guide |
| Background jobs may duplicate backup/device/license metadata if idempotency is weak | High | Open | Idempotency key, retry tests, metadata uniqueness constraints |
| Server scope may drift into storing all merchant operational data | Critical | Open | ADR-0008, DEC-019, server schema guardrails, test CP/BAK metadata-only cases |
| Cloud backup misconfiguration may create false sense of safety | High | Open | Provider connection test, scheduled backup status, restore drill |
| Backup encryption key loss may make restore impossible | High | Open | Recovery key UX, clear warning, Business escrow decision pending |
| Backup storage cost may grow unexpectedly | Medium | Open | Retention policy by plan, size dashboard, cleanup worker |
| User may confuse subscription server with transaction cloud storage | Medium | Open | Product wording: server is control plane, operational data stays local unless backup/opt-in |
| License may block merchant during offline period | High | Open | Signed token and grace period |
| Hard lock after expiry may create data hostage perception | High | Mitigated by policy | Restricted Expired Mode: block new operations, allow old data/export/backup/renewal |
| User may bypass subscription by keeping app offline | High | Open | Short-lived signed token, heartbeat, lease expiry, anti-clock-rollback checks |
| Local clock manipulation may extend license | High | Open | Store last_server_time, detect rollback, require online verification |
| License signing key exposure would break monetization | Critical | Open | Private key server-side only, desktop stores public key only |
| Migration may damage local data | Critical | Open | Backup before migration and migration log |
| Scope may expand too early | High | Open | Follow MVP gates and ADR-0006 |
| Hardware compatibility may vary | Medium | Open | Printer/barcode abstraction and adapter pattern |
| Different AI agents may interpret blueprint inconsistently | Medium | Mitigated by docs | AGENT_HANDOFF, AGENTS, agent-specific adapters, TASK_BACKLOG, and PROMPTS_FOR_AGENTS added |
| Server implementation may drift into cloud POS database | Critical | Mitigated by docs | SERVER_BLUEPRINT, SERVER_MATCHING_MATRIX, SERVER_TEST_PLAN, ADR-0010, and server schema guardrail added |
| License token may be copied to another PC | Critical | Mitigated by docs | ADR-0011 requires device-bound license, challenge-response activation, signed heartbeat, nonce replay defense |
| BYOS backup configuration may create SSRF risk | High | Mitigated by docs | SERVER_LOCAL_INTEGRATION_SECURITY requires endpoint validation, private IP blocking, timeout, redirect limits |
| Admin dashboard abuse may alter subscription/device/update state | High | Mitigated by docs | Admin MFA, step-up auth, reason, immutable audit, support access log required |

## 5. Current Blockers

| Blocker | Owner | Status |
| :--- | :--- | :--- |
| Decide local PostgreSQL deployment mode: per device or per outlet local server | Human/Architect Agent | Open |
| Decide initial OS target: Windows only or Windows + macOS | Human | Open |
| Decide grace period duration | Human/Product | Open |
| Decide whether payment in MVP is manual QRIS only or payment provider integration | Human/Product | Open |
| Decide initial F&B scope | Human/Product | Open |
| Decide default managed backup provider | Human/Architect Agent | Open |
| Decide backup retention per subscription level | Human/Product | Open |
| Decide whether backup encryption recovery is user-held only or includes enterprise escrow | Human/Product/Security | Open |
| Decide exact plan pricing and included device counts | Human/Product | Open |
| Decide admin dashboard MVP modules and support access boundaries | Human/Product/Architect Agent | Open |
| Decide license lease duration: 3, 7, or 14 days | Human/Product/Security | Open |
| Decide suspicious local time behavior | Human/Product/Security | Open |
| Decide whether MVP uses one DB runtime user or separate migration/runtime DB users | Human/Architect Agent | Open |
| Decide exact local app data path conventions per OS | Human/Architect Agent | Open |
| Decide whether cloud backup is allowed in Restricted Expired Mode | Human/Product/Security | Open |
| Decide close-shift behavior in Restricted Expired Mode | Human/Product | Open |
| Decide manual billing process owner and payment confirmation SLA | Human/Product/Operations | Open |
| Validate tax/service/receipt requirements with tax/legal advisor before production | Human/Legal/Tax | Open |
| Decide diagnostic bundle upload policy | Human/Product/Security | Open |

## 6. Next Steps

1. Jalankan pengujian pilot dengan merchant terpilih untuk memvalidasi performa database.
2. Siapkan penandatanganan sertifikat installer (Authenticode) untuk rilis komersial.
