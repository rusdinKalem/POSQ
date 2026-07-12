# DECISIONS

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Record product and architecture decisions so Antigravity agents do not re-debate or accidentally change core direction.

## 1. Decision Log

| ID | Decision | Status | Reason |
|---|---|---|---|
| DEC-001 | Build a desktop local-first POS, not web-only POS | Accepted | Checkout must keep working during internet outage |
| DEC-002 | Use Tauri v2 as desktop shell | Accepted | Lightweight desktop app with local system access |
| DEC-003 | Use Svelte + TypeScript for UI | Accepted | Fast, simple, modular UI for cashier and owner screens |
| DEC-004 | Use Rust/Tauri commands for local service | Accepted | Reliable access to DB, background jobs, license, update, hardware |
| DEC-005 | Use PostgreSQL local as primary operational DB | Accepted | Strong relational model for order, stock, audit, reporting |
| DEC-006 | Use PostgreSQL server only for control plane data by default | Accepted | Server manages account, credential, device, license, subscription, update, and backup metadata without owning merchant operational DB |
| DEC-007 | Checkout must save locally without waiting for server/control plane | Accepted | Server must not block cashier transaction |
| DEC-008 | Background jobs use outbox/inbox-style durability where needed | Accepted | Prevents lost license, update, heartbeat, backup, and metadata jobs |
| DEC-009 | Control plane and backup metadata operations must be idempotent | Accepted | Retry must not create duplicate device, license, or backup metadata records |
| DEC-010 | License uses signed device token | Accepted | Allows offline entitlement validation |
| DEC-011 | Subscription must support grace period | Accepted | Merchant should not stop operating due to temporary internet issue |
| DEC-012 | Expired subscription must not block old data access | Accepted | Merchant must retain access/export to historical data |
| DEC-013 | Update must use signed release | Accepted | Prevents malicious or invalid update |
| DEC-014 | Local DB backup required before migration | Accepted | Protects transaction data |
| DEC-015 | RBAC and audit log are MVP requirements | Accepted | Prevents unauthorized refund, void, discount, stock changes |
| DEC-016 | Marketplace connector is not MVP P0 | Accepted | Sync and checkout must be stable first |
| DEC-017 | Real payment gateway integration is not required before local payment records are stable | Accepted | Avoids adding external complexity too early |
| DEC-018 | F&B and retail workflows are modular extensions | Accepted | Core POS must remain stable across business types |
| DEC-019 | Server must not store full merchant operational database by default | Accepted | Preserves local-first ownership and reduces privacy, cost, and operational risk |
| DEC-020 | Cloud backup is optional and user-controlled | Accepted | Merchants can choose local only, managed cloud backup, or BYOS depending on plan |
| DEC-021 | Cloud backup must be encrypted before upload | Accepted | Storage provider and control plane should not read operational data by default |
| DEC-022 | Server stores backup metadata only | Accepted | Needed for support, restore listing, status, and retention without storing plaintext business data |
| DEC-023 | Admin dashboard is a SaaS control plane dashboard | Accepted | Dashboard manages merchant, credential, device, license, subscription, update, and backup metadata |
| DEC-024 | Starter plan must work without cloud dependency | Accepted | Entry users still need reliable local POS and local backup |
| DEC-025 | BYOS backup starts from higher tier | Accepted | BYOS requires support burden and credential/security handling better suited for Pro/Business |
| DEC-026 | Use Restricted Expired Mode after subscription grace period | Accepted | Blocks new paid operations while preserving merchant access to historical data, export, backup, restore, update security, and renewal |
| DEC-027 | Do not use full hard lock on expired subscription | Accepted | Full lock creates data hostage risk, support burden, and trust problem |
| DEC-028 | License token must be server-signed and short-lived | Accepted | Offline operation is possible while preventing indefinite use without renewal |
| DEC-029 | Desktop app must never contain license signing private key | Accepted | Prevents attackers from minting valid licenses locally |
| DEC-030 | Local clock rollback must be detected | Accepted | Prevents extending subscription by changing device time |
| DEC-031 | Antigravity must read P0 docs before coding | Accepted | Installation, security, entitlement, API, and data model details prevent ambiguous implementation |
| DEC-032 | API_SPEC.md is authoritative for control plane endpoints | Accepted | Prevents server drifting into operational transaction storage |
| DEC-033 | DATA_MODEL.md is authoritative for MVP schema boundaries | Accepted | Keeps local operational DB and server control plane DB separated |
| DEC-034 | ENTITLEMENT_MATRIX.md is authoritative for feature gating | Accepted | Keeps UI and Rust command enforcement consistent |
| DEC-035 | MVP uses local PostgreSQL per device | Accepted | Simplifies installation, offline reliability, migration, and backup for first release |
| DEC-036 | Outlet local server is deferred until multi-terminal ADR | Accepted | Prevents accidental LAN/multi-cashier complexity in MVP |
| DEC-037 | MVP backup encryption uses user-held recovery key | Accepted | Vendor cannot read backup; user must preserve recovery key |
| DEC-038 | UI_FLOW.md is authoritative for main product screens | Accepted | Prevents unclear expired, backup, restore, and first-run UX |
| DEC-039 | ERROR_HANDLING.md is authoritative for structured errors | Accepted | Keeps data-safe recovery behavior consistent |
| DEC-040 | MVP billing uses manual renewal confirmation | Accepted | Keeps MVP payment complexity low while preserving license/subscription enforcement |
| DEC-041 | Tax/service rules must be configurable, not hardcoded | Accepted | Indonesian tax and service handling needs merchant/legal validation |
| DEC-042 | Manual QRIS record is MVP payment behavior | Accepted | Avoids premature payment gateway complexity before local payment records are stable |
| DEC-043 | Observability must use redacted local diagnostics | Accepted | Supports troubleshooting without leaking merchant data or secrets |
| DEC-044 | Release cannot proceed without backup/restore/license/update gates | Accepted | Prevents data loss and monetization/security failures |
| DEC-045 | Blueprint must be agent-portable | Accepted | Antigravity is preferred, but Claude Code, Codex, Copilot, Cursor, Devin, or other agents should follow the same implementation contract |
| DEC-046 | AGENT_HANDOFF.md and AGENTS.md are the universal cross-agent contract | Accepted | Prevents agent-specific prompts from drifting away from local-first architecture and data ownership rules |
| DEC-047 | TASK_BACKLOG.md is the executable task source for agents | Accepted | Keeps autonomous implementation incremental, testable, and traceable |
| DEC-048 | Agent-specific instruction files are adapters, not architecture authorities | Accepted | CLAUDE.md, DEVIN.md, Copilot instructions, and Cursor rules must not override ADR or DECISIONS |
| DEC-049 | Server blueprint is required and separate from desktop blueprint | Accepted | Server must be implemented as SaaS control plane, not guessed from scattered API notes |
| DEC-050 | SERVER_MATCHING_MATRIX.md is authoritative for local-server responsibility boundaries | Accepted | Prevents server from becoming operational transaction source of truth by accident |
| DEC-051 | SERVER_TEST_PLAN.md is required before server pilot | Accepted | Server guardrails, tenant isolation, license, backup metadata, and admin audit need explicit tests |
| DEC-052 | License must be device-bound | Accepted | Signed token alone can be copied; activation and heartbeat must prove device possession |
| DEC-053 | Heartbeat and license refresh require nonce/replay protection | Accepted | Prevents replaying old valid requests |
| DEC-054 | License signing key and update signing key must be separate | Accepted | Compromise of one signing domain must not compromise the other |
| DEC-055 | BYOS validation must include SSRF defenses | Accepted | User-controlled endpoints can target internal/private resources if not validated |
| DEC-056 | Admin high-risk actions require MFA step-up | Accepted | Subscription override, device revoke, update publish, and key operations are sensitive SaaS controls |
| DEC-057 | Mobile vs. Desktop Hardware Integration Strategy | Accepted | Decided to use a unified abstraction layer in Svelte/Rust supporting USB/LAN/Serial on Desktop and BLE/Camera on Mobile |
| DEC-058 | Responsive Layout & Touch-First UI Strategy | Accepted | Decided to use adaptive screen tab navigation on mobile portrait layouts (<768px) and enforce touch target sizes (minimum 48x48px) for crucial POS buttons (numpad, cart quantity adjust, checkout) to support mobile tablet/HP expansion |

## 2. Decisions Requiring Human Confirmation

| ID | Pending Decision | Options | Recommendation |
|---|---|---|---|
| PDEC-001 | Local PostgreSQL deployment mode | Per device / Per outlet local server | Start per device for PoC, evaluate outlet local server before multi-terminal |
| PDEC-002 | Initial OS target | Windows only / Windows + macOS + Linux | Start Windows first |
| PDEC-003 | Grace period duration | 3 days / 7 days / 14 days | 7 days for MVP |
| PDEC-004 | Payment MVP | Manual QRIS record / Payment gateway integration | Manual QRIS record first |
| PDEC-005 | Printer support | Mock only / Specific printer model / Generic ESC/POS | Start mock + generic ESC/POS abstraction |
| PDEC-006 | F&B MVP scope | Modifier only / Table + modifier / KDS | Start table + modifier + kitchen print abstraction |
| PDEC-007 | Retail return scope | Return record only / Return + stock adjustment / Full exchange flow | Start return + stock adjustment |
| PDEC-008 | Default managed backup provider | Cloudflare R2 / Backblaze B2 / Amazon S3 / Google Cloud Storage | Start with S3-compatible abstraction; choose Cloudflare R2 or Backblaze B2 for managed backup pilot |
| PDEC-009 | Cloud backup entitlement | Growth and above / Pro and above / Add-on | Include managed backup in Growth; BYOS in Pro |
| PDEC-010 | Backup retention | 7/30/90/365 days | Starter local only; Growth 30 days; Pro 90 days; Business configurable |
| PDEC-011 | Backup encryption key recovery | User-held recovery key / vendor escrow / no recovery | Start with user-held recovery key; evaluate escrow only for Business/Enterprise |
| PDEC-012 | Admin dashboard MVP scope | License only / License + subscription / Full control plane | Build license + subscription + device + update + backup metadata in MVP |
| PDEC-013 | Future cloud operational sync | Never / Enterprise opt-in / all plans | Enterprise opt-in only, after new ADR |
| PDEC-014 | License lease duration | 3 days / 7 days / 14 days | 7 days for MVP; shorten to 3 days for high-risk enterprise policies |
| PDEC-015 | Suspicious time behavior | Full lock / restricted expired / require online verification | Require online verification; allow export/backup if data safety policy permits |
| PDEC-016 | Whether MVP uses one DB runtime user or separate migration/runtime DB users | One app user / Separate migration and runtime users | Start one app-specific non-superuser for PoC; separate before production if feasible |
| PDEC-017 | Exact local app data path conventions per OS | Windows only / Cross-platform paths | Define Windows first, keep cross-platform abstraction |
| PDEC-018 | Whether cloud backup remains allowed in Restricted Expired Mode | Local only / Existing scheduled cloud allowed / All cloud backup allowed | Local backup must remain; cloud backup can be limited for MVP |
| PDEC-019 | Closing active shift in Restricted Expired Mode | Allow close only / Block all shift actions | Allow close shift to preserve cash/report consistency |
| PDEC-020 | Billing MVP process | Manual admin confirmation / Payment link / Full payment gateway | Manual admin confirmation first |
| PDEC-021 | Tax/service default | Disabled by default / Enabled with configurable rate | Disabled by default, merchant config required |
| PDEC-022 | QRIS MVP | Manual record / Payment gateway integration | Manual QRIS record first |
| PDEC-023 | Diagnostic upload | User manually sends bundle / Auto upload with consent | Manual bundle first; add consented upload later |

## 3. Rules for Future Decisions

Any new decision that changes architecture must be recorded here and, if significant, must create a new ADR.

Requires ADR:

- Changing database.
- Changing sync strategy.
- Changing license model.
- Changing update model.
- Changing primary desktop stack.
- Adding payment gateway.
- Adding marketplace sync.
- Adding multi-terminal local server.
- Changing data ownership between local and cloud.
- Storing operational transaction/inventory/customer data on server.
- Adding cloud operational sync.
- Changing backup encryption or key ownership model.
- Changing plan entitlements that affect data access or backup.
- Changing expired subscription behavior.
- Changing license token signing, lease duration, or anti-clock-rollback policy.
- Changing P0 implementation contracts in INSTALLATION_RUNBOOK, SECURITY_MODEL, ENTITLEMENT_MATRIX, API_SPEC, or DATA_MODEL.
- Changing P1 operational contracts in LOCAL_POSTGRESQL_STRATEGY, LICENSE_LIFECYCLE, BACKUP_KEY_RECOVERY, UI_FLOW, or ERROR_HANDLING.
- Changing P2 operational/release contracts in BILLING_RENEWAL, INDONESIA_COMPLIANCE, OBSERVABILITY_SUPPORT, or RELEASE_CHECKLIST.
- Changing cross-agent implementation contract in AGENT_HANDOFF, AGENTS, TASK_BACKLOG, or agent-specific instruction files.
- Changing server control plane boundaries in SERVER_BLUEPRINT, SERVER_MATCHING_MATRIX, SERVER_DATA_MODEL, or ADR-0010.
- Changing device-bound license, signed heartbeat, key separation, or BYOS SSRF rules in ADR-0011 or SERVER_LOCAL_INTEGRATION_SECURITY.

Does not require ADR:

- UI wording changes.
- Small component refactor.
- README updates.
- Test data changes.
- Non-breaking helper functions.

## 4. Decision Prompt for Antigravity

Use this prompt whenever an agent proposes a major change:

```text
Before implementing this change, update docs/DECISIONS.md.

Explain:
1. What decision is being proposed?
2. Which PRD requirement does it support?
3. What alternatives were considered?
4. What are the consequences?
5. Does this require a new ADR?

Do not implement until the decision is approved.
```

## 4A. P0 Documents Required Before Coding

Antigravity must read these before M1/M2 implementation:

- `docs/INSTALLATION_RUNBOOK.md`
- `docs/SECURITY_MODEL.md`
- `docs/ENTITLEMENT_MATRIX.md`
- `docs/API_SPEC.md`
- `docs/DATA_MODEL.md`

If generated repository uses a different filename casing, keep content equivalent and update this list.

## 4B. P1 Documents Required Before Feature Implementation

Antigravity must read these before implementing related features:

- `docs/LOCAL_POSTGRESQL_STRATEGY.md`
- `docs/LICENSE_LIFECYCLE.md`
- `docs/BACKUP_KEY_RECOVERY.md`
- `docs/UI_FLOW.md`
- `docs/ERROR_HANDLING.md`

These documents refine MVP implementation details after the P0 contracts.

## 4C. P2 Documents Required Before Pilot/Beta/Production

Antigravity must read these before pilot/beta/production hardening:

- `docs/BILLING_RENEWAL.md`
- `docs/INDONESIA_COMPLIANCE.md`
- `docs/OBSERVABILITY_SUPPORT.md`
- `docs/RELEASE_CHECKLIST.md`

These documents define operational readiness, billing, compliance checklist, observability, and release gates.

## 5. Explicitly Rejected Directions

| Rejected Direction | Reason |
|---|---|
| Web-only POS | Violates offline/local-first requirement |
| Checkout must call server before saving | Violates local checkout reliability |
| SQLite as primary operational DB | PRD specifies PostgreSQL local as primary DB |
| No license grace period | Too risky for merchant operations |
| Hard lock all data after subscription expired | Bad merchant experience and data access risk |
| Migration without backup | Data loss risk |
| Sync without idempotency | Duplicate transaction risk |
| Marketplace-first MVP | Too much integration complexity before POS core is stable |
| Server as primary database for all merchant transactions by default | Violates local-first ownership, increases privacy/cost risk, and contradicts updated product direction |
| Plaintext cloud backup | Storage compromise would expose merchant operations |
| Admin dashboard showing merchant order/payment/customer data by default | Turns control plane into operational data platform without user consent |
| Subscription expiry deleting or hiding historical data | Merchant must retain access, export, and backup rights |
| Full hard lock after subscription expiry | Blocks export/backup/renewal, creates data hostage risk, and worsens support disputes |
| Long-lived perpetual local license token | User can remain offline indefinitely after cancellation |
| License signing private key bundled in desktop app | Attackers can extract key and forge licenses |
