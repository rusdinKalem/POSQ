# PROMPTS FOR AGENTS

Purpose: Prompt siap pakai agar blueprint ini dapat dijalankan oleh Antigravity dan coding agent lain dengan hasil konsisten.

## 1. Universal Startup Prompt

```text
Anda adalah coding agent untuk Aplikasi POS SaaS Indonesia berbasis Tauri, Svelte, Rust, dan PostgreSQL local-first.

Baca dokumen berikut sebelum coding:
- AGENT_HANDOFF.md
- AGENTS.md
- IMPLEMENTATION_PLAN.md
- DECISIONS.md
- STATUS.md
- semua ADR/*.md
- TASK_BACKLOG.md
- SERVER_BLUEPRINT.md
- SERVER_MATCHING_MATRIX.md
- SERVER_LOCAL_INTEGRATION_SECURITY.md

Aturan utama:
- Checkout harus tetap berjalan tanpa server.
- PostgreSQL lokal adalah database operasional utama.
- Server hanya control plane: credential, merchant, device, subscription, license, update metadata, backup metadata.
- Jangan simpan order/payment/inventory/customer purchase history di server secara default.
- Cloud backup harus opt-in dan terenkripsi client-side.
- Subscription expired memakai Restricted Expired Mode, bukan full hard lock.
- Jangan pernah memasukkan private signing key ke desktop app.
- Migration lokal wajib didahului backup.

Pilih satu task kecil dari TASK_BACKLOG.md, implementasikan, jalankan test, lalu update STATUS.md.
```

## 1A. Server Startup Prompt

```text
Anda adalah coding agent untuk membangun aplikasi server/control plane POS.

Baca:
- SERVER_BLUEPRINT.md
- SERVER_IMPLEMENTATION_PLAN.md
- SERVER_DATA_MODEL.md
- SERVER_API_WORKFLOWS.md
- SERVER_ADMIN_DASHBOARD.md
- SERVER_SECURITY_DEPLOYMENT.md
- SERVER_MATCHING_MATRIX.md
- SERVER_TEST_PLAN.md
- SERVER_LOCAL_INTEGRATION_SECURITY.md
- SERVER_HARDENING_REVIEW.md
- API_SPEC.md
- DATA_MODEL.md
- ADR/0010-control-plane-server-blueprint.md
- ADR/0011-device-bound-license-and-zero-trust-integration.md

Jangan membuat server menjadi database transaksi toko.

Tugas:
1. Konfirmasi scope server control plane.
2. Konfirmasi tabel yang boleh dibuat.
3. Konfirmasi tabel yang dilarang.
4. Pilih task server pertama dari S0/S1.
5. Jalankan hanya task kecil tersebut.

Validasi wajib:
- server schema guardrail
- tenant isolation
- license private key tidak masuk desktop
- device-bound license wajib
- heartbeat nonce replay wajib ditolak
- BYOS SSRF defense wajib
- license signing key dan update signing key harus terpisah
- admin audit untuk semua mutasi
```

## 2. Antigravity Prompt

```text
Bertindak sebagai engineering agent utama.

Baca seluruh folder docs/ dengan urutan:
AGENT_HANDOFF.md, AGENTS.md, IMPLEMENTATION_PLAN.md, DECISIONS.md, STATUS.md, ADR/*.md, TEST_PLAN.md, TASK_BACKLOG.md.

Mulai dari M1 Technical PoC:
1. Scaffold Tauri v2 + Svelte + TypeScript.
2. Tambahkan Rust health command.
3. Tambahkan koneksi PostgreSQL lokal.
4. Tambahkan migration pertama yang idempotent.
5. Simpan dan baca dummy order dari PostgreSQL lokal.

Jangan mulai core checkout sebelum PoC ini lulus.
Update STATUS.md setelah selesai.
```

## 3. Claude Code Prompt

```text
Read AGENT_HANDOFF.md, AGENTS.md, IMPLEMENTATION_PLAN.md, DECISIONS.md, STATUS.md, ADR/*.md, and TASK_BACKLOG.md.

Implement task <TASK_ID>.

Preserve:
- local-first checkout
- PostgreSQL local as operational DB
- server as control plane only
- encrypted opt-in backup
- Restricted Expired Mode
- signed license token validation

Run relevant tests and update STATUS.md. If you need to change architecture, stop and propose an ADR first.
```

## 4. OpenAI Codex Prompt

```text
You are working in a local-first POS repository.

Before editing files, inspect:
- AGENT_HANDOFF.md
- AGENTS.md
- IMPLEMENTATION_PLAN.md
- DECISIONS.md
- STATUS.md
- ADR/*.md
- TASK_BACKLOG.md

Pick task <TASK_ID>. Make a focused implementation. Prefer existing project patterns. Add or update tests. Update STATUS.md. Do not alter architecture without DECISIONS.md and ADR updates.
```

## 5. GitHub Copilot Coding Agent Prompt

```text
Create a PR for task <TASK_ID> from TASK_BACKLOG.md.

Follow .github/copilot-instructions.md and the blueprint docs.

PR requirements:
- Reference task ID.
- Keep scope narrow.
- Include tests.
- Update STATUS.md.
- Do not store operational merchant data on server.
- Do not make checkout require server.
```

## 6. Cursor Agent Prompt

```text
Use .cursor/rules/pos-blueprint.mdc as always-on rule.

Implement task <TASK_ID>.

Read the owning document before editing. Keep changes small. Run tests. Update STATUS.md. Ask for approval before changing ADR-level decisions.
```

## 7. Devin Prompt

```text
Use DEVIN.md as the execution guide.

Start with M1 batch unless already completed.

At each checkpoint, report:
- task IDs
- completed work
- changed files
- tests run
- blockers
- risks
- next step

Stop before changing architecture or server data ownership.
```

## 8. Security Review Prompt

```text
Review implementation against SECURITY_MODEL.md, LICENSE_LIFECYCLE.md, BACKUP_KEY_RECOVERY.md, ENTITLEMENT_MATRIX.md, and ADR-0008/ADR-0009.

Find:
- server storing operational data by mistake
- checkout dependency on server
- plaintext backup exposure
- weak license token validation
- private key leakage
- expired mode bypass
- RBAC enforced only in UI
- missing audit log
- migration without backup

Return findings by severity and required fixes.
```

## 9. Release Readiness Prompt

```text
Audit the app against RELEASE_CHECKLIST.md and TEST_PLAN.md.

Confirm:
- clean install
- offline checkout
- backup and restore drill
- encrypted cloud backup
- license active/grace/expired/renewal/revoke
- signed update
- migration rollback safety
- RBAC and audit
- server control-plane-only schema
- support diagnostic redaction

Produce a go/no-go report.
```
