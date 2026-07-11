# AGENT HANDOFF

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Universal handoff untuk Antigravity, Claude Code, OpenAI Codex, GitHub Copilot Coding Agent, Cursor Agent, Devin, atau coding agent lain.

## 1. Operating Rule

Agent boleh membantu implementasi, review, test, dan dokumentasi, tetapi tidak boleh mengubah arah produk tanpa memperbarui keputusan dan ADR.

Core direction:

- Produk adalah desktop POS local-first.
- Stack utama: Tauri v2, Svelte + TypeScript, Rust, PostgreSQL lokal.
- PostgreSQL server adalah control plane, bukan tempat menyimpan database operasional merchant secara default.
- Checkout tidak boleh bergantung pada server.
- Backup cloud bersifat opt-in dan harus terenkripsi client-side.
- Subscription expired menggunakan Restricted Expired Mode, bukan full hard lock.
- License memakai signed short-lived device token.
- Private signing key tidak boleh masuk desktop app.
- Update aplikasi harus signed.
- Migration database lokal harus diawali backup.

## 2. Mandatory Read Order

Sebelum coding, agent wajib membaca dokumen dalam urutan ini:

1. `IMPLEMENTATION_PLAN.md`
2. `AGENTS.md`
3. `DECISIONS.md`
4. `STATUS.md`
5. Semua file `ADR/*.md`
6. `INSTALLATION_RUNBOOK.md`
7. `SECURITY_MODEL.md`
8. `ENTITLEMENT_MATRIX.md`
9. `API_SPEC.md`
10. `DATA_MODEL.md`
11. `LOCAL_POSTGRESQL_STRATEGY.md`
12. `LICENSE_LIFECYCLE.md`
13. `BACKUP_KEY_RECOVERY.md`
14. `UI_FLOW.md`
15. `ERROR_HANDLING.md`
16. `BILLING_RENEWAL.md`
17. `INDONESIA_COMPLIANCE.md`
18. `OBSERVABILITY_SUPPORT.md`
19. `RELEASE_CHECKLIST.md`
20. `SERVER_BLUEPRINT.md`
21. `SERVER_IMPLEMENTATION_PLAN.md`
22. `SERVER_DATA_MODEL.md`
23. `SERVER_API_WORKFLOWS.md`
24. `SERVER_ADMIN_DASHBOARD.md`
25. `SERVER_SECURITY_DEPLOYMENT.md`
26. `SERVER_MATCHING_MATRIX.md`
27. `SERVER_TEST_PLAN.md`
28. `SERVER_LOCAL_INTEGRATION_SECURITY.md`
29. `SERVER_HARDENING_REVIEW.md`
30. `TASK_BACKLOG.md`

Jika repository sudah memiliki `docs/` folder, letakkan seluruh file ini di `docs/` dan sesuaikan path. Jika file berada di root blueprint, perlakukan root blueprint sebagai `docs/`.

## 3. Agent Workflow

Setiap agent harus menjalankan pola kerja berikut:

1. Baca dokumen wajib sesuai urutan.
2. Pilih satu task kecil dari `TASK_BACKLOG.md`.
3. Jelaskan rencana implementasi singkat.
4. Implementasikan perubahan dengan scope kecil.
5. Jalankan test yang relevan.
6. Update `STATUS.md`.
7. Tambahkan keputusan baru ke `DECISIONS.md` bila ada perubahan arah.
8. Buat ADR baru jika perubahan menyentuh arsitektur.
9. Laporkan changed files, test result, risiko, dan next step.

## 4. Stop Conditions

Agent harus berhenti dan meminta keputusan manusia jika:

- Perubahan membuat server menyimpan order, payment, stock movement, customer purchase history, atau database operasional merchant.
- Perubahan menghapus akses user ke data lama saat subscription expired.
- Perubahan menghilangkan backup sebelum migration.
- Perubahan memasukkan private signing key ke desktop app.
- Perubahan membuat checkout menunggu server.
- Perubahan mengganti PostgreSQL lokal dengan database lain.
- Perubahan menambahkan cloud operational sync tanpa ADR baru.
- Perubahan mengubah model backup encryption/key recovery.
- Perubahan membutuhkan keputusan harga, SLA billing, pajak, atau legal.

## 5. Output Contract

Setiap sesi agent harus menghasilkan ringkasan dengan format:

```text
Agent/model:
Task ID:

Completed:
-

Changed files:
-

Tests run:
-

Result:
-

Risks:
-

Next step:
-
```

## 6. Quality Gates

Tidak boleh lanjut milestone jika gate berikut belum lulus:

- M1: Tauri app terbuka, Svelte UI berjalan, Rust command callable, PostgreSQL lokal tersambung.
- M2: Migration idempotent, local schema sesuai `DATA_MODEL.md`, backup sebelum migration tersedia.
- M3: Checkout offline berhasil tanpa server.
- M7: Control plane hanya menyimpan account, device, license, subscription, update, backup metadata.
- M8: Backup lokal dan cloud terenkripsi bisa diverifikasi dan direstore.
- M9: License active, grace, expired, renewal, revoke, anti-clock-rollback lulus test.
- M14: Installer, signed update, release checklist, dan backup/restore drill lulus.
