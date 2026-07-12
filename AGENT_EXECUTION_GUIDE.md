# AGENT EXECUTION GUIDE

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Panduan gradual dari prompt pertama sampai aplikasi POS selesai dibangun dengan Antigravity atau AI coding agent lain.

## 1. Prinsip Utama

> [!IMPORTANT]
> **ATURAN MUTLAK: JANGAN PERNAH PAKAI BROWSER / SUBAGENT BROWSER!**
> Developer/owner melarang keras AI agent mengaktifkan browser subagent atau melakukan manipulasi browser. Semua verifikasi dan validasi harus dilakukan secara offline/terminal (seperti `cargo check`, `npm run check`, unit testing), atau diserahkan sepenuhnya ke user untuk testing visual secara manual.

Jangan meminta agent langsung membuat seluruh aplikasi dalam satu prompt. Aplikasi POS ini menyentuh uang, stok, license, backup, database lokal, subscription, dan update. Implementasi harus bertahap, diuji, lalu dikunci sebelum lanjut.

Urutan kerja yang benar:

1. Buat project repository.
2. Masukkan seluruh folder `docs/` dari blueprint.
3. Jalankan prompt orientasi.
4. Jalankan M1 technical proof of concept.
5. Jalankan M2 database foundation.
6. Jalankan M3 core checkout.
7. Lanjutkan milestone satu per satu sampai M14.
8. Review security, backup, license, dan release readiness.
9. Build installer.
10. Pilot dengan data terbatas.

## 2. Struktur Project Awal

Buat repository dengan struktur minimum:

```text
pos-local-online/
  docs/
    AGENT_HANDOFF.md
    AGENTS.md
    IMPLEMENTATION_PLAN.md
    DECISIONS.md
    STATUS.md
    TEST_PLAN.md
    TASK_BACKLOG.md
    PROMPTS_FOR_AGENTS.md
    ADR/
    ...
```

Setelah itu agent akan membuat folder aplikasi:

```text
pos-local-online/
  apps/
    desktop/
  services/
    control-plane-api/
  packages/
  migrations/
  scripts/
  tests/
```

## 3. Aturan Prompting

Gunakan satu prompt untuk satu fase kecil. Jangan gabungkan banyak milestone.

Format prompt yang direkomendasikan:

```text
Konteks:
<jelaskan posisi project saat ini>

Tugas:
<satu task atau satu batch kecil>

Batasan:
<aturan yang tidak boleh dilanggar>

Output:
<hasil yang diminta>

Validasi:
<test atau bukti yang harus dijalankan>
```

## 4. Prompt 0 - Orientasi Project

Gunakan ini pertama kali setelah folder `docs/` dimasukkan ke repository.

```text
Anda adalah engineering agent untuk membangun aplikasi POS local-first.

Baca seluruh dokumen berikut sebelum coding:
- docs/AGENT_HANDOFF.md
- docs/AGENTS.md
- docs/IMPLEMENTATION_PLAN.md
- docs/DECISIONS.md
- docs/STATUS.md
- docs/TEST_PLAN.md
- semua docs/ADR/*.md
- docs/TASK_BACKLOG.md
- semua dokumen P0/P1/P2 di docs/
- semua dokumen di docs/

Jangan coding dulu.

Tugas Anda:
1. Buat ringkasan pemahaman arsitektur.
2. Sebutkan non-negotiable requirements.
3. Sebutkan risiko terbesar.
4. Sebutkan milestone pertama yang harus dikerjakan.
5. Sebutkan task ID dari TASK_BACKLOG.md yang akan dikerjakan lebih dulu.

Pastikan Anda menegaskan:
- checkout tidak boleh bergantung pada server
- PostgreSQL lokal adalah database operasional utama
- server hanya control plane
- backup cloud opt-in dan terenkripsi client-side
- subscription expired memakai Restricted Expired Mode
- private signing key tidak boleh masuk desktop app
- migration wajib backup dulu
```

Expected output:

- Agent memahami dokumen.
- Tidak ada kode dibuat.
- Agent memilih mulai dari M1.

## 5. Prompt 1 - Buat Rencana Eksekusi M1

```text
Berdasarkan docs/TASK_BACKLOG.md, buat rencana eksekusi M1 Technical PoC.

Jangan coding dulu.

Rencana harus mencakup:
1. Task M1-001 sampai M1-007.
2. File/folder yang akan dibuat.
3. Dependency yang dibutuhkan.
4. Cara menjalankan dev server/app.
5. Cara koneksi PostgreSQL lokal.
6. Test yang akan dijalankan.
7. Risiko teknis dan mitigasi.

Output harus berupa rencana singkat yang bisa langsung dieksekusi.
```

Expected output:

- Ada rencana teknis M1.
- Tidak ada scope M2/M3 masuk sebelum waktunya.

## 6. Prompt 2 - Eksekusi M1 Technical PoC

```text
Eksekusi M1 Technical PoC sesuai docs/TASK_BACKLOG.md:
- M1-001 Scaffold Tauri v2 + Svelte + TypeScript
- M1-002 Add Rust health command
- M1-003 Add PostgreSQL local config
- M1-004 Add PostgreSQL health check
- M1-005 Add first local migration
- M1-006 Add dummy order write/read
- M1-007 Document setup

Batasan:
- Jangan implement checkout asli dulu.
- Jangan buat control plane server dulu.
- Jangan ubah keputusan ADR.

Validasi:
- App desktop bisa dibuka.
- UI bisa memanggil Rust command.
- Rust bisa mengecek PostgreSQL lokal.
- Migration pertama idempotent.
- Dummy order bisa disimpan dan dibaca dari PostgreSQL lokal.
- Update docs/STATUS.md setelah selesai.

Laporkan changed files dan test result.
```

Gate sebelum lanjut:

- M1 lulus seluruh acceptance criteria.
- Jika gagal PostgreSQL/local migration, jangan lanjut ke M2 sebelum diperbaiki.

## 7. Prompt 3 - Review M1

```text
Review hasil M1.

Cek:
1. Apakah Tauri + Svelte + TypeScript berjalan.
2. Apakah Rust command callable dari UI.
3. Apakah PostgreSQL lokal tersambung.
4. Apakah migration idempotent.
5. Apakah dummy order tersimpan lokal.
6. Apakah docs/STATUS.md sudah diperbarui.

Jangan menambah fitur baru.

Berikan:
- temuan bug
- risiko
- rekomendasi fix
- apakah boleh lanjut M2 atau belum
```

Jika ada bug:

```text
Perbaiki semua blocker dari review M1.
Jangan mulai M2.
Setelah perbaikan, jalankan ulang test M1 dan update docs/STATUS.md.
```

## 8. Prompt 4 - Eksekusi M2 Local Database Foundation

```text
Lanjutkan ke M2 Local Database Foundation.

Baca ulang:
- docs/DATA_MODEL.md
- docs/LOCAL_POSTGRESQL_STRATEGY.md
- docs/SECURITY_MODEL.md
- docs/TEST_PLAN.md
- docs/TASK_BACKLOG.md

Implement task:
- M2-001 Migration runner
- M2-002 Local tenant seed
- M2-003 Product/category tables
- M2-004 Order/payment tables
- M2-005 Stock movement tables
- M2-006 Audit log table
- M2-007 Backup-before-migration guard

Batasan:
- Operational tables hanya di PostgreSQL lokal.
- Jangan buat server operational order/payment/stock tables.
- Migration harus idempotent.
- Backup-before-migration wajib.

Validasi:
- Jalankan test DB-T001 sampai DB-T008 dari TEST_PLAN.md jika sudah memungkinkan.
- Update docs/STATUS.md.
```

Gate:

- Schema lokal sesuai `DATA_MODEL.md`.
- Migration tidak merusak data.
- Backup-before-migration terbukti.

## 9. Prompt 5 - Eksekusi M3 Core Checkout

```text
Lanjutkan ke M3 Core Checkout Local MVP.

Baca ulang:
- docs/UI_FLOW.md
- docs/DATA_MODEL.md
- docs/ENTITLEMENT_MATRIX.md
- docs/ERROR_HANDLING.md
- docs/TEST_PLAN.md

Implement task:
- M3-001 Product search UI
- M3-002 Cart calculation
- M3-003 Open shift requirement
- M3-004 Cash checkout
- M3-005 Offline checkout proof
- M3-006 Stock decrement
- M3-007 Receipt preview

Batasan:
- Checkout harus menyimpan ke PostgreSQL lokal.
- Checkout tidak boleh memanggil server sebagai syarat transaksi.
- Stock decrement harus menghasilkan stock movement.
- Receipt dibuat dari data order lokal.

Validasi:
- CHK-T001 sampai CHK-T007.
- SHF-T001 minimal untuk blok checkout tanpa shift.
- Server/API dimatikan, checkout tetap berhasil.
- Update docs/STATUS.md.
```

Gate:

- Offline checkout sukses.
- Total cart/order/payment konsisten.
- Stock movement tercatat.

## 10. Prompt 6 - M4 Shift, RBAC, Audit

```text
Lanjutkan ke M4 Shift, RBAC, and Audit.

Baca:
- docs/SECURITY_MODEL.md
- docs/ENTITLEMENT_MATRIX.md
- docs/DATA_MODEL.md
- docs/TEST_PLAN.md

Implement:
- M4-001 Role and permission model
- M4-002 Backend permission guard
- M4-003 Open/close shift
- M4-004 Void/refund approval
- M4-005 Audit viewer

Batasan:
- Permission wajib dicek di backend/Rust command, bukan hanya UI.
- Void, refund, discount override, stock adjustment harus masuk audit log.
- Cashier tidak boleh mengakses owner dashboard atau aksi sensitif.

Validasi:
- SEC-T001 sampai SEC-T004.
- SHF-T001 sampai SHF-T004.
- CHK-T006 dan CHK-T007.
- Update docs/STATUS.md.
```

## 11. Prompt 7 - M5 Inventory MVP

```text
Lanjutkan ke M5 Inventory MVP.

Baca:
- docs/DATA_MODEL.md
- docs/UI_FLOW.md
- docs/ERROR_HANDLING.md
- docs/TEST_PLAN.md

Implement:
- M5-001 Stock in
- M5-002 Stock adjustment
- M5-003 Stock opname
- M5-004 Low stock alert
- M5-005 Transfer model

Batasan:
- Semua perubahan stok harus menghasilkan stock movement.
- Adjustment harus punya reason.
- Aksi sensitif harus diaudit.

Validasi:
- INV-T001 sampai INV-T005.
- Update docs/STATUS.md.
```

## 12. Prompt 8 - M6 Local Reporting

```text
Lanjutkan ke M6 Local Reporting.

Implement:
- M6-001 Sales summary
- M6-002 Payment breakdown
- M6-003 Product ranking
- M6-004 Shift report
- M6-005 Export report

Batasan:
- Report membaca PostgreSQL lokal.
- Report tidak boleh membutuhkan server.
- Total report harus cocok dengan order/payment/shift.

Validasi:
- Reports match local transaction data.
- Export CSV/PDF berjalan.
- Update docs/STATUS.md.
```

## 13. Prompt 9 - M7 Control Plane API

```text
Lanjutkan ke M7 Control Plane API.

Baca:
- docs/API_SPEC.md
- docs/DATA_MODEL.md
- docs/SECURITY_MODEL.md
- docs/DECISIONS.md
- ADR-0008

Implement:
- M7-001 Server scaffold
- M7-002 Auth and credential model
- M7-003 Merchant/device model
- M7-004 Subscription model
- M7-005 License issue endpoint
- M7-006 Update metadata endpoint
- M7-007 Schema guardrail test

Batasan keras:
- Server hanya control plane.
- Jangan buat tabel operational orders/payments/stock_movements/customer_purchase_history di server.
- Server boleh simpan merchant, credential, device, subscription, license, update metadata, backup metadata, admin audit.

Validasi:
- CP-T001 sampai CP-T011.
- Server schema guardrail test wajib lulus.
- Update docs/STATUS.md.
```

## 14. Prompt 10 - M8 Backup and Metadata Sync

```text
Lanjutkan ke M8 Backup and Metadata Sync.

Baca:
- docs/BACKUP_KEY_RECOVERY.md
- docs/API_SPEC.md
- docs/SECURITY_MODEL.md
- docs/ERROR_HANDLING.md
- docs/TEST_PLAN.md

Implement:
- M8-001 Local backup
- M8-002 Local restore
- M8-003 Backup encryption
- M8-004 Managed cloud backup adapter
- M8-005 BYOS S3-compatible adapter
- M8-006 Backup metadata upload
- M8-007 Idempotent retry
- M8-008 Restore safety

Batasan:
- Backup cloud harus encrypted sebelum upload.
- Server hanya menyimpan metadata backup.
- Restore harus validasi checksum dan manifest.
- Restore ke DB aktif harus membuat pre-restore backup.

Validasi:
- BAK-T001 sampai BAK-T012.
- Metadata idempotency: retry 5 kali hanya membuat satu logical backup record.
- Update docs/STATUS.md.
```

## 15. Prompt 11 - M9 License and Subscription

```text
Lanjutkan ke M9 License and Subscription.

Baca:
- docs/LICENSE_LIFECYCLE.md
- docs/ENTITLEMENT_MATRIX.md
- docs/BILLING_RENEWAL.md
- docs/SECURITY_MODEL.md
- ADR-0004
- ADR-0009

Implement:
- M9-001 Device activation UI
- M9-002 Local token verification
- M9-003 Heartbeat refresh
- M9-004 Grace period
- M9-005 Restricted Expired Mode
- M9-006 Anti-clock-rollback
- M9-007 Device revoke

Batasan keras:
- Private signing key hanya di server.
- Desktop hanya menyimpan public key untuk verifikasi.
- Expired mode tidak boleh menyembunyikan data lama.
- Expired mode harus blok new checkout, stock adjustment, new order, dan operasi paid lainnya.
- Expired mode tetap membolehkan data lama, export, local backup, restore, renewal, dan security update.

Validasi:
- LIC-T001 sampai LIC-T018.
- Token tampering gagal.
- Clock rollback terdeteksi.
- Update docs/STATUS.md.
```

## 16. Prompt 12 - M10 Update and Migration Safety

```text
Lanjutkan ke M10 Update and Migration Safety.

Baca:
- docs/RELEASE_CHECKLIST.md
- docs/LOCAL_POSTGRESQL_STRATEGY.md
- docs/SECURITY_MODEL.md
- ADR-0005

Implement:
- M10-001 Version check
- M10-002 Signed update validation
- M10-003 Migration backup gate
- M10-004 Failed migration recovery
- M10-005 Release channel

Batasan:
- Update unsigned/invalid harus ditolak.
- Migration harus backup dulu.
- Failed migration tidak boleh merusak data lama.

Validasi:
- Update invalid ditolak.
- Migration gagal tetap menjaga data.
- Update docs/STATUS.md.
```

## 17. Prompt 13 - M11 Hardware Abstraction

```text
Lanjutkan ke M11 Hardware Abstraction.

Implement:
- M11-001 Printer abstraction
- M11-002 Barcode scanner flow

Batasan:
- Hardware harus melalui adapter, bukan hardcoded ke satu printer.
- Receipt preview tetap bisa jalan tanpa printer.
- Barcode scanner dianggap input cepat ke search/checkout.

Validasi:
- Mock printer lulus.
- ESC/POS adapter dasar tersedia jika memungkinkan.
- Barcode input menemukan SKU.
- Update docs/STATUS.md.
```

## 18. Prompt 14 - M12 F&B and Retail Basic

```text
Lanjutkan ke M12 F&B and Retail Basic.

Implement:
- M12-001 F&B basic mode
- M12-002 Retail return flow

Batasan:
- F&B dan retail adalah modul, jangan merusak core checkout.
- Return harus menghasilkan payment/stock/audit records yang benar.
- Table/modifier/kitchen print basic cukup untuk MVP.

Validasi:
- F&B table/modifier/kitchen print basic berjalan.
- Retail return menyesuaikan stock dan audit.
- Update docs/STATUS.md.
```

## 19. Prompt 15 - M13 QA Hardening

```text
Lanjutkan ke M13 QA Hardening.

Lakukan review menyeluruh terhadap:
- TEST_PLAN.md
- SECURITY_MODEL.md
- ENTITLEMENT_MATRIX.md
- BACKUP_KEY_RECOVERY.md
- LICENSE_LIFECYCLE.md
- RELEASE_CHECKLIST.md

Tugas:
1. Jalankan semua test yang relevan.
2. Buat daftar bug berdasarkan severity.
3. Perbaiki bug Critical dan High.
4. Jalankan ulang test.
5. Update docs/STATUS.md.

Fokus:
- offline checkout
- backup/restore
- license active/grace/expired/renewal/revoke
- RBAC dan audit
- server control-plane-only
- migration safety
- update signed release
```

## 20. Prompt 16 - Security Review Independen

Gunakan model berbeda dari implementer jika memungkinkan.

```text
Lakukan security review independen.

Baca:
- docs/SECURITY_MODEL.md
- docs/LICENSE_LIFECYCLE.md
- docs/BACKUP_KEY_RECOVERY.md
- docs/ENTITLEMENT_MATRIX.md
- ADR-0008
- ADR-0009

Cari:
- server menyimpan operational data tanpa sengaja
- checkout bergantung server
- backup plaintext
- private signing key leakage
- license token bisa dimanipulasi
- expired mode bisa dibypass
- RBAC hanya dicek UI
- audit log hilang
- migration tanpa backup
- diagnostic log membocorkan data sensitif

Output:
- Findings by severity
- File/line reference
- Required fix
- Go/no-go security status
```

## 21. Prompt 17 - M14 Packaging and Release

```text
Lanjutkan ke M14 Packaging and Release.

Baca:
- docs/INSTALLATION_RUNBOOK.md
- docs/RELEASE_CHECKLIST.md
- docs/OBSERVABILITY_SUPPORT.md
- docs/INDONESIA_COMPLIANCE.md

Implement:
- M14-001 Windows installer
- M14-002 Release checklist

Validasi:
- Clean install di Windows target.
- PostgreSQL lokal terpasang/terhubung.
- App bisa dibuka.
- Device activation berjalan.
- Offline checkout berjalan.
- Backup dan restore drill lulus.
- License expired behavior sesuai policy.
- Update docs/STATUS.md.
```

## 22. Prompt 18 - Pilot Readiness

```text
Audit pilot readiness.

Pastikan:
1. Tidak ada Critical/High bug terbuka.
2. Offline checkout lulus.
3. Backup lokal dan restore lulus.
4. Backup cloud encrypted lulus jika enabled.
5. License lifecycle lulus.
6. Restricted Expired Mode lulus.
7. Update signed lulus.
8. Migration backup lulus.
9. Admin dashboard control plane lulus.
10. Diagnostic bundle redaction lulus.

Output:
- Go/no-go pilot
- daftar risiko tersisa
- batasan pilot
- checklist operasional untuk merchant pertama
```

## 23. Prompt 19 - Final Release Readiness

```text
Lakukan final release readiness audit.

Baca:
- docs/RELEASE_CHECKLIST.md
- docs/TEST_PLAN.md
- docs/STATUS.md
- docs/DECISIONS.md
- semua ADR

Berikan laporan final:
1. Status setiap milestone M1-M14.
2. Test yang sudah lulus.
3. Test yang belum lulus.
4. Risiko release.
5. Keputusan pending.
6. Apakah aplikasi layak release.
7. Jika belum layak, task blocker yang wajib diselesaikan.

Jangan menambah fitur baru.
```

## 24. Prompt 20 - Finish

Gunakan prompt ini hanya setelah pilot berhasil dan release blocker kosong.

```text
Buat laporan akhir penyelesaian aplikasi POS.

Isi laporan:
1. Ringkasan produk.
2. Fitur yang sudah selesai.
3. Arsitektur final.
4. Cara install.
5. Cara menjalankan aplikasi.
6. Cara backup dan restore.
7. Cara renewal subscription.
8. Cara update aplikasi.
9. Known limitations.
10. Rekomendasi roadmap berikutnya.

Update docs/STATUS.md menjadi release-ready jika semua gate lulus.
```

## 25. Urutan Ringkas

Jalankan berurutan:

1. Prompt 0 - Orientasi.
2. Prompt 1 - Rencana M1.
3. Prompt 2 - Eksekusi M1.
4. Prompt 3 - Review M1.
5. Prompt 4 - M2.
6. Prompt 5 - M3.
7. Prompt 6 - M4.
8. Prompt 7 - M5.
9. Prompt 8 - M6.
10. Prompt 9 - M7.
11. Prompt 10 - M8.
12. Prompt 11 - M9.
13. Prompt 12 - M10.
14. Prompt 13 - M11.
15. Prompt 14 - M12.
16. Prompt 15 - M13.
17. Prompt 16 - Security Review.
18. Prompt 17 - M14.
19. Prompt 18 - Pilot Readiness.
20. Prompt 19 - Final Release Readiness.
21. Prompt 20 - Finish.

## 26. Kapan Menggunakan Model Berat

Gunakan model reasoning paling kuat untuk:

- Review PRD dan workflow.
- ADR.
- Local PostgreSQL strategy.
- License/subscription.
- Backup encryption dan recovery.
- Security review.
- Release readiness.

Gunakan model coding cepat untuk:

- Scaffold.
- UI implementation.
- CRUD.
- Test implementation.
- Refactor kecil.
- Styling.

## 27. Kapan Harus Berhenti

Hentikan agent dan minta keputusan manusia jika:

- Agent ingin menyimpan order/payment/inventory/customer history ke server.
- Agent ingin mengganti PostgreSQL lokal.
- Agent ingin membuat checkout bergantung server.
- Agent ingin hard-lock seluruh aplikasi saat expired.
- Agent ingin menghapus backup-before-migration.
- Agent ingin menyimpan backup cloud tanpa encryption.
- Agent ingin memasukkan private signing key ke desktop.
- Agent ingin menambah payment gateway sebelum core POS stabil.
- Agent ingin menambah marketplace sync sebelum backup/license/checkout stabil.
