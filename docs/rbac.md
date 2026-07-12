Anda bertindak sebagai Principal Software Architect, Application Security Engineer, dan Senior Full-Stack Developer yang berpengalaman membangun aplikasi POS Retail, Jasa, dan F&B production-grade.

Saya ingin Anda menganalisis dan mengembangkan modul:

1. Role-Based Access Control.
2. Supervisor Authorization menggunakan PIN dan RFID/NFC.
3. Sensitive Action Policy Engine.
4. Cashier Shift Management.
5. Cash Drawer Control.
6. Starting Cash.
7. Cash In.
8. Cash Out atau Petty Cash.
9. Safe Drop atau Cash Skim.
10. Closing Cash Count dan Cash Reconciliation.
11. Immutable Audit Trail.
12. Fraud Detection dan Security Alert.
13. Security Report untuk pemilik, manager, dan auditor.

Stack aplikasi:

* Frontend: [ISI SESUAI CODEBASE]
* Backend: [ISI SESUAI CODEBASE]
* Database: [ISI SESUAI CODEBASE]
* Authentication: [ISI SESUAI CODEBASE]
* State management: [ISI SESUAI CODEBASE]
* Deployment: [ISI SESUAI CODEBASE]
* Mode offline: [YA/TIDAK]
* Multi-outlet: [YA/TIDAK]
* Multi-device: [YA/TIDAK]

Sebelum mengubah codebase, analisis terlebih dahulu:

1. Struktur repository.
2. Model user, role, permission, outlet, device, transaction, payment, shift, dan cash drawer yang sudah tersedia.
3. Authentication dan session management yang sudah digunakan.
4. Mekanisme audit yang sudah ada.
5. Mekanisme offline dan synchronization bila tersedia.
6. Risiko keamanan dan technical debt yang relevan.
7. Daftar file, migration, service, controller, component, test, dan konfigurasi yang perlu ditambah atau diubah.

Jangan mengganti arsitektur utama tanpa alasan yang kuat. Ikuti naming convention, coding style, folder structure, dependency injection, error handling, validation, dan testing convention repository.

# A. Prinsip Keamanan Utama

Terapkan prinsip berikut:

* Deny by default.
* Least privilege.
* Separation of duties.
* Four-eyes principle untuk tindakan berisiko tinggi.
* Tidak ada shared supervisor PIN.
* Tidak ada hardcoded PIN.
* Tidak ada raw PIN atau RFID identifier dalam log.
* Tidak ada penghapusan fisik untuk transaksi finansial.
* Tidak ada perubahan langsung terhadap transaksi yang telah posted atau paid.
* Semua koreksi menggunakan void, refund, reversal, atau adjustment.
* Semua tindakan sensitif harus memiliki reason code.
* Semua tindakan sensitif harus menghasilkan immutable audit trail.
* Semua authorization grant harus scoped, short-lived, dan single-use.
* Semua operasi finansial harus atomic dan idempotent.
* Sistem harus fail-secure. Jika otorisasi tidak dapat diverifikasi, tindakan sensitif harus ditolak.

Jangan mengklaim bahwa sistem sepenuhnya aman. Implementasikan security-in-depth dan dokumentasikan residual risk.

# B. Role dan Permission

Implementasikan RBAC yang mendukung multi-outlet.

Role awal:

* OWNER
* ADMIN
* MANAGER
* SUPERVISOR
* CASHIER
* WAITER
* SERVICE_STAFF
* INVENTORY_STAFF
* AUDITOR

Satu user dapat memiliki role berbeda pada outlet yang berbeda.

Contoh permission:

## Transaksi umum

* transaction.create
* transaction.edit_open
* transaction.void_item
* transaction.void
* transaction.post_payment_void
* transaction.reopen
* transaction.change_payment_method
* transaction.reprint_receipt
* transaction.override_price
* transaction.override_tax
* transaction.manual_discount
* transaction.special_discount
* transaction.complimentary
* transaction.view_cost
* transaction.view_margin

## Refund dan return

* refund.create
* refund.partial
* refund.full
* refund.cash
* refund.without_receipt
* refund.override_limit
* return.create
* exchange.create

## Cash drawer dan shift

* shift.open
* shift.close
* shift.force_close
* shift.reopen
* shift.view_expected_cash
* cash.starting_cash
* cash.cash_in
* cash.cash_out
* cash.safe_drop
* cash.drawer_open_no_sale
* cash.adjustment
* cash.approve_variance
* cash.view_other_cashier_shift

## Retail

* inventory.adjust
* inventory.negative_adjustment
* inventory.write_off
* inventory.change_serial
* inventory.change_batch
* retail.return_without_receipt

## Jasa

* service.cancel_completed
* service.change_commission
* service.refund_deposit
* service.change_package_usage
* service.mark_complimentary

## F&B

* fnb.void_unsent_item
* fnb.void_sent_item
* fnb.void_served_item
* fnb.reopen_bill
* fnb.split_partially_paid_bill
* fnb.join_partially_paid_bill
* fnb.remove_service_charge
* fnb.comp_item

Gunakan permission granular. Jangan hanya menggunakan pengecekan role seperti `if user.role === manager`.

# C. Sensitive Action Policy Engine

Buat policy engine yang dapat dikonfigurasi per outlet.

Policy harus dapat mempertimbangkan:

* Jenis tindakan.
* Mode bisnis: Retail, Jasa, atau F&B.
* Nominal tindakan.
* Persentase diskon.
* Status transaksi.
* Metode pembayaran.
* Umur transaksi.
* Apakah transaksi memiliki receipt.
* Apakah transaksi sudah dibayar.
* Apakah item sudah dikirim ke dapur.
* Apakah item sudah disajikan.
* Apakah transaksi berasal dari outlet yang sama.
* Role kasir.
* Role supervisor.
* Batas otorisasi supervisor.
* Jam operasional.
* Status shift.
* Status konektivitas.
* Risk score.
* Riwayat approval sebelumnya.

Policy menghasilkan salah satu keputusan:

* ALLOW
* REQUIRE_SUPERVISOR
* REQUIRE_MANAGER
* REQUIRE_DUAL_APPROVAL
* DENY

Contoh policy configurable:

* Diskon sampai 10 persen dapat dilakukan kasir tertentu.
* Diskon di atas 10 persen memerlukan supervisor.
* Diskon di atas 25 persen memerlukan manager.
* Refund tunai selalu memerlukan supervisor.
* Refund tunai di atas batas tertentu memerlukan manager.
* Refund tanpa receipt memerlukan manager.
* Post-payment void selalu memerlukan supervisor.
* Post-payment void bernilai tinggi memerlukan dual approval.
* Void item F&B yang sudah served memerlukan supervisor.
* Perubahan closing shift memerlukan manager.
* Cash variance di atas threshold memerlukan supervisor atau manager.

Semua threshold harus berasal dari configuration atau database, bukan hardcoded di frontend.

# D. Supervisor Authorization

Supervisor authorization mendukung:

* PIN.
* RFID/NFC.
* Kombinasi RFID dan PIN.
* Manager approval.
* Dual approval untuk tindakan sangat berisiko.

Setiap supervisor harus menggunakan akun pribadi. Jangan menggunakan shared PIN.

## PIN Security

* Gunakan PIN minimal enam digit atau panjang configurable.
* Simpan menggunakan Argon2id atau algoritma password hashing yang kuat.
* Tambahkan server-side pepper dari secret manager.
* Jangan menyimpan PIN plaintext.
* Jangan mengirim PIN ke analytics, logging, crash reporting, atau monitoring.
* Gunakan TLS untuk komunikasi.
* Mask input PIN.
* Bersihkan nilai PIN dari memory state setelah request selesai.
* Terapkan rate limit per user, device, outlet, dan IP.
* Terapkan temporary lock setelah beberapa kegagalan.
* Catat failed authorization attempt tanpa mencatat PIN.
* Kirim security alert bila kegagalan berulang.
* Supervisor yang dinonaktifkan atau locked tidak boleh memberikan approval.

## RFID/NFC Security

* Jangan menganggap UID RFID biasa sebagai faktor keamanan yang kuat.
* Simpan token atau hash identifier, bukan raw UID bila memungkinkan.
* Untuk high-risk action, gunakan smart card cryptographic atau kombinasi RFID dan PIN.
* RFID yang hilang harus dapat dicabut.
* Catat waktu penggunaan terakhir dan device.
* Tolak credential yang expired, revoked, atau tidak terdaftar pada outlet.

## Authorization Flow

Implementasikan alur:

1. Kasir memulai tindakan sensitif.
2. Backend mengevaluasi policy.
3. Jika supervisor diperlukan, frontend menampilkan authorization dialog.
4. Dialog menampilkan:

   * Jenis tindakan.
   * Transaksi.
   * Item terkait.
   * Nilai tindakan.
   * Kasir.
   * Outlet.
   * Reason code.
   * Risiko atau warning.
5. Supervisor memasukkan PIN atau menempelkan kartu.
6. Backend memvalidasi:

   * Identitas supervisor.
   * Status akun.
   * Role dan permission.
   * Outlet assignment.
   * Authorization limit.
   * Shift atau kehadiran bila diwajibkan.
   * Device.
   * Rate limit.
   * Conflict of interest.
7. Backend menerbitkan authorization grant.
8. Grant digunakan untuk satu command spesifik.
9. Tindakan dan grant dikonsumsi dalam database transaction yang sama.
10. Grant tidak dapat digunakan kembali.

Authorization grant harus terikat pada:

* authorization_id
* action_type
* resource_type
* resource_id
* cashier_id
* supervisor_id
* outlet_id
* device_id
* shift_id
* approved_amount
* approved_percentage
* reason_code
* issued_at
* expires_at
* used_at
* status
* policy_snapshot
* request_hash

Status:

* PENDING
* APPROVED
* REJECTED
* EXPIRED
* USED
* REVOKED

Default expiration harus singkat, misalnya 60 sampai 120 detik dan configurable.

Jangan membuat endpoint bypass generik yang hanya mengembalikan boolean. Gunakan authorization token yang cryptographically signed atau opaque random token dan tersimpan secara aman di server.

Authorization tidak boleh membuka seluruh menu supervisor. Authorization hanya berlaku untuk tindakan dan resource yang disetujui.

Secara default, supervisor tidak boleh menyetujui tindakannya sendiri untuk tindakan high-risk. Sediakan konfigurasi `allow_self_approval` tetapi default harus false.

# E. Reason Code

Buat master reason code per jenis tindakan.

Contoh:

## Void

* WRONG_ITEM
* WRONG_QUANTITY
* DUPLICATE_ENTRY
* CUSTOMER_CANCELLED
* PRODUCT_UNAVAILABLE
* SERVICE_NOT_PROVIDED
* KITCHEN_ERROR
* CASHIER_ERROR
* OTHER

## Refund

* PRODUCT_DEFECT
* WRONG_PRODUCT
* SERVICE_COMPLAINT
* DUPLICATE_PAYMENT
* ORDER_NOT_DELIVERED
* CUSTOMER_REQUEST
* PRICE_DISPUTE
* OTHER

## Cash out

* ICE_PURCHASE
* PARKING
* DELIVERY
* GAS
* CLEANING_SUPPLIES
* EMERGENCY_PURCHASE
* TRANSPORT
* OTHER

Reason `OTHER` harus mewajibkan catatan manual.

Reason code harus disimpan sebagai snapshot agar perubahan master reason tidak mengubah histori lama.

# F. Transaction Integrity

Setelah transaksi berstatus PAID, POSTED, CLOSED, atau SETTLED:

* Item tidak boleh diedit atau dihapus secara langsung.
* Quantity tidak boleh diubah secara langsung.
* Harga tidak boleh diubah secara langsung.
* Metode pembayaran tidak boleh diganti secara langsung.
* Transaksi tidak boleh dihapus.
* Koreksi harus melalui workflow void, refund, return, reversal, atau adjustment.

Gunakan status yang jelas:

* DRAFT
* OPEN
* PAYMENT_PENDING
* PAID
* PARTIALLY_REFUNDED
* REFUNDED
* VOIDED
* CLOSED
* REVERSED

Void sebelum pembayaran dan refund setelah pembayaran harus menjadi workflow berbeda.

## Void sebelum pembayaran

* Membatalkan item atau transaksi yang belum dibayar.
* Tetap menyimpan histori.
* Item F&B yang sudah dikirim ke dapur memerlukan permission tambahan.
* Tidak boleh menghapus KOT lama.

## Post-payment void

* Harus diperlakukan sebagai koreksi finansial.
* Memerlukan supervisor.
* Harus membuat reversal atau adjustment document.
* Harus memperbarui ledger, stok, pajak, dan payment allocation secara konsisten.
* Jangan mengubah transaksi asli tanpa histori.

## Refund

* Mendukung full dan partial refund.
* Menghubungkan refund dengan transaksi asli.
* Tidak boleh melebihi refundable balance.
* Item atau quantity tidak boleh direfund melebihi quantity yang dibeli.
* Refund harus menggunakan metode pembayaran asli bila diwajibkan.
* Refund tunai dari transaksi non-tunai memerlukan policy khusus.
* Refund provider harus memiliki status:

  * PENDING
  * PROCESSING
  * SUCCEEDED
  * FAILED
  * CANCELLED
* Jangan menganggap refund berhasil sebelum payment provider mengonfirmasi.
* Gunakan idempotency key untuk mencegah double refund.

# G. Cashier Shift dan Register Session

Pisahkan konsep:

* Store atau Outlet.
* POS Device.
* Register.
* Cash Drawer.
* Cashier Shift.
* Cash Drawer Session.
* Business Day.

Satu shift harus memiliki:

* id
* outlet_id
* register_id
* cash_drawer_id
* cashier_id
* business_date
* opened_at
* closed_at
* opening_status
* closing_status
* opening_cash_total
* expected_cash_total
* actual_cash_total
* variance_total
* status
* version
* created_at
* updated_at

Status shift:

* PENDING_OPEN
* OPEN
* CLOSING
* PENDING_APPROVAL
* CLOSED
* SUSPENDED
* FORCE_CLOSED

Secara default:

* Kasir tidak boleh memiliki dua shift aktif pada register yang sama.
* Cash drawer tidak boleh ditugaskan kepada dua kasir secara bersamaan.
* Transaksi tidak boleh dibuat tanpa shift aktif, kecuali konfigurasi tertentu.
* Transaksi harus terhubung ke shift, register, device, cashier, outlet, dan business date.
* Closing shift harus menggunakan server time sebagai sumber utama.
* Perubahan jam lokal device tidak boleh mengubah business date tanpa otorisasi.

Sediakan shared drawer mode hanya sebagai konfigurasi eksplisit. Beri warning bahwa shared drawer mengurangi akuntabilitas per kasir.

# H. Starting Cash

Alur:

1. Kasir memilih register.
2. Kasir menghitung modal awal berdasarkan denomination.
3. Sistem menghitung total.
4. Kasir mengonfirmasi.
5. Jika diperlukan policy, supervisor memberikan approval.
6. Sistem membuka shift dan cash drawer session.

Gunakan blind opening count jika dikonfigurasi:

* Kasir tidak melihat expected starting cash.
* Sistem hanya memberi tahu apakah cocok atau terdapat selisih setelah submit.

Simpan detail denomination:

* denomination
* quantity
* total

Starting cash yang sudah posted tidak boleh diedit. Koreksi menggunakan cash adjustment atau reversal dengan supervisor authorization.

# I. Cash Movement Ledger

Buat cash ledger append-only.

Jenis movement:

* STARTING_CASH
* CASH_SALE
* CASH_REFUND
* CASH_IN
* CASH_OUT
* PETTY_CASH
* SAFE_DROP
* SAFE_RECEIPT
* TENDER_EXCHANGE
* CASH_ADJUSTMENT
* CLOSING_VARIANCE
* REVERSAL

Setiap movement memiliki:

* id
* movement_number
* outlet_id
* register_id
* cash_drawer_id
* shift_id
* type
* direction
* amount
* currency
* category_id
* reason_code
* notes
* transaction_id
* payment_id
* parent_movement_id
* requested_by
* approved_by
* posted_by
* status
* attachment_ids
* created_at
* approved_at
* posted_at
* reversed_at
* version

Direction:

* IN
* OUT

Status:

* DRAFT
* PENDING_APPROVAL
* APPROVED
* REJECTED
* POSTED
* REVERSED

Movement yang sudah POSTED tidak boleh diedit atau dihapus. Koreksi harus menggunakan reversal yang mengacu ke movement asli.

Pastikan reversal tidak dapat dilakukan dua kali.

# J. Cash In

Cash in harus mendukung:

* Penambahan change fund.
* Transfer dari safe.
* Pengembalian petty cash.
* Setoran internal.
* Adjustment berotorisasi.

Validasi:

* Amount lebih besar dari nol.
* Currency sesuai cash drawer.
* Shift masih aktif.
* Kategori valid.
* Reason tersedia.
* Supervisor diperlukan bila melewati batas.
* Tidak boleh menggunakan authorization grant transaksi lain.

# K. Cash Out dan Petty Cash

Cash out harus mendukung:

* Kategori pengeluaran.
* Nominal.
* Penerima.
* Tujuan.
* Nomor bukti.
* Catatan.
* Foto receipt atau attachment.
* Supervisor approval.
* Limit per kategori.
* Limit per shift.
* Limit per kasir.
* Limit harian outlet.

Untuk amount tertentu, attachment dan supervisor approval wajib.

Cash out yang disetujui harus mengurangi expected cash.

Tolak cash out jika:

* Shift tidak aktif.
* Kas tidak mencukupi menurut policy.
* Supervisor tidak memiliki batas approval yang cukup.
* Authorization expired.
* Kategori dinonaktifkan.
* Limit harian terlampaui.
* Attachment wajib tetapi tidak tersedia.

# L. Safe Drop

Sediakan safe drop ketika kas di drawer melebihi threshold.

Safe drop memiliki dua sisi:

1. Drawer menyerahkan uang.
2. Safe atau supervisor mengonfirmasi penerimaan.

Status:

* INITIATED
* HANDED_OVER
* RECEIVED
* DISPUTED
* CANCELLED

Simpan:

* Nominal.
* Denomination.
* Bag atau envelope number.
* Kasir.
* Penerima.
* Supervisor.
* Waktu penyerahan.
* Waktu penerimaan.
* Catatan.
* Attachment opsional.

Expected cash drawer berkurang setelah status sesuai kebijakan, misalnya setelah `RECEIVED`.

# M. Cash Drawer Opening

Setiap pembukaan cash drawer harus tercatat.

Jenis:

* SALE
* REFUND
* CASH_IN
* CASH_OUT
* STARTING_COUNT
* CLOSING_COUNT
* NO_SALE
* MAINTENANCE

`NO_SALE` harus membutuhkan:

* Permission.
* Reason.
* Supervisor sesuai policy.

Log harus menyimpan:

* user_id
* shift_id
* register_id
* device_id
* reason
* opened_at
* related_transaction_id

Jangan hanya mencatat pembukaan drawer yang berhasil. Catat juga attempt yang ditolak.

# N. Closing Cash Count

Gunakan blind closing count secara default:

* Kasir menghitung uang fisik berdasarkan denomination.
* Kasir tidak melihat expected cash sebelum submit.
* Sistem menghitung actual cash.
* Setelah submit, sistem menghitung variance.

Rumus:

Expected Cash =
Starting Cash

* Cash Sales
* Cash In

- Cash Refunds
- Cash Out
- Safe Drops
  ± Posted Adjustments

Variance = Actual Cash - Expected Cash

Simpan snapshot komponen perhitungan pada saat closing agar laporan lama tidak berubah.

Alur:

1. Kasir memilih `Close Shift`.
2. Sistem menghentikan transaksi baru pada shift atau masuk ke state CLOSING.
3. Sistem memastikan tidak ada payment pending.
4. Kasir memasukkan denomination count.
5. Kasir submit blind count.
6. Sistem menghitung expected, actual, dan variance.
7. Jika variance dalam tolerance:

   * Shift dapat ditutup sesuai policy.
8. Jika variance melebihi tolerance:

   * Wajib recount.
   * Wajib reason.
   * Wajib supervisor atau manager approval.
9. Sistem membuat closing report.
10. Shift berstatus CLOSED.
11. Data closing tidak dapat diedit secara langsung.

Support minimal dua kali recount dan simpan semua attempt. Jangan menimpa count sebelumnya.

Status closing:

* NOT_STARTED
* COUNTING
* RECOUNT_REQUIRED
* PENDING_APPROVAL
* APPROVED
* CLOSED

# O. Cash Variance Policy

Buat policy configurable:

* Absolute tolerance.
* Percentage tolerance.
* Threshold supervisor.
* Threshold manager.
* Maksimum recount.
* Apakah shift dapat ditutup dengan variance.
* Apakah kasir dapat melihat expected cash.
* Apakah foto uang atau deposit wajib.
* Apakah alasan wajib.

Contoh keputusan:

* Variance Rp0 sampai Rp5.000: dapat ditutup.
* Variance di atas Rp5.000: supervisor approval.
* Variance di atas Rp100.000: manager approval dan security alert.

Jangan hardcode angka tersebut. Simpan sebagai konfigurasi outlet.

# P. Data Model

Evaluasi schema yang ada dan tambahkan atau sesuaikan model berikut:

* users
* roles
* permissions
* role_permissions
* user_outlet_roles
* user_security_credentials
* supervisor_limits
* authorization_policies
* authorization_requests
* authorization_grants
* authorization_attempts
* reason_codes
* pos_devices
* registers
* cash_drawers
* cashier_shifts
* cash_drawer_sessions
* cash_movements
* cash_count_sessions
* cash_count_details
* cash_variance_approvals
* transaction_voids
* refunds
* refund_items
* transaction_adjustments
* receipt_reprints
* drawer_open_events
* audit_logs
* security_events
* fraud_rules
* fraud_alerts
* attachments
* outbox_events
* idempotency_records

Entitas penting harus memiliki:

* id
* outlet_id jika relevan
* status
* version
* created_at
* updated_at
* created_by
* device_id jika relevan

Nilai uang harus menggunakan integer minor currency unit atau tipe decimal yang aman. Jangan menggunakan floating point.

# Q. Audit Trail

Buat immutable append-only audit log.

Audit harus mencatat:

* event_id
* event_type
* action_type
* module
* resource_type
* resource_id
* parent_resource_id
* outlet_id
* business_date
* shift_id
* cashier_id
* supervisor_id
* manager_id
* device_id
* correlation_id
* idempotency_key
* before_snapshot
* after_snapshot
* changed_fields
* amount
* currency
* reason_code
* notes
* policy_snapshot
* authorization_id
* result
* failure_code
* server_timestamp

Audit wajib untuk:

* Login dan logout.
* Failed login.
* Failed supervisor PIN.
* RFID rejected.
* Permission denied.
* Void.
* Refund.
* Return.
* Diskon.
* Price override.
* Tax override.
* Reopen transaction.
* Cash in.
* Cash out.
* Safe drop.
* No-sale drawer opening.
* Starting cash.
* Closing count.
* Recount.
* Variance approval.
* Shift force close.
* Role atau permission change.
* Supervisor PIN reset.
* RFID assignment atau revocation.
* Configuration change.
* Export laporan sensitif.

Jangan mencatat:

* PIN.
* Password.
* Raw authentication token.
* Full payment card data.
* Secret key.
* Raw RFID credential bila tidak diperlukan.

Tambahkan opsi tamper-evident audit menggunakan hash chain:

* previous_hash
* entry_hash

Gunakan server timestamp. Jangan mempercayai waktu device sebagai waktu audit utama.

# R. Fraud Detection

Buat rule-based fraud detection yang configurable.

Rule minimal:

* HIGH_VOID_RATE
* POST_PAYMENT_VOID
* REPEATED_CASH_REFUND
* REFUND_WITHOUT_RECEIPT
* REFUND_TO_DIFFERENT_TENDER
* DISCOUNT_NEAR_APPROVAL_LIMIT
* EXCESSIVE_MANUAL_DISCOUNT
* EXCESSIVE_RECEIPT_REPRINT
* EXCESSIVE_NO_SALE_OPEN
* REPEATED_CASH_SHORT
* REPEATED_CASH_OVER
* CASH_OUT_ROUND_AMOUNT_PATTERN
* CASH_OUT_WITHOUT_ATTACHMENT
* VOID_NEAR_SHIFT_CLOSE
* REFUND_SHORTLY_AFTER_SALE
* SAME_SUPERVISOR_HIGH_APPROVAL_RATE
* REPEATED_FAILED_SUPERVISOR_PIN
* RFID_USED_ON_MULTIPLE_DEVICES
* TRANSACTION_EDIT_AFTER_PAYMENT
* NEGATIVE_STOCK_ADJUSTMENT
* OUT_OF_HOURS_SENSITIVE_ACTION

Severity:

* INFO
* LOW
* MEDIUM
* HIGH
* CRITICAL

Fraud rule menghasilkan alert, bukan tuduhan otomatis.

Fraud alert menyimpan:

* rule_id
* severity
* user_id
* supervisor_id
* outlet_id
* shift_id
* transaction_id
* supporting_event_ids
* calculated_metrics
* status
* assigned_to
* resolution
* created_at
* resolved_at

Status:

* OPEN
* REVIEWING
* FALSE_POSITIVE
* CONFIRMED
* RESOLVED

# S. Reporting

Buat laporan minimal:

1. Supervisor Authorization Report.
2. Void Report.
3. Post-Payment Void Report.
4. Refund Report.
5. Discount and Price Override Report.
6. Cash In/Out Report.
7. Petty Cash Report.
8. Safe Drop Report.
9. Cash Drawer Opening Report.
10. Shift Summary.
11. Cash Variance Report.
12. Cashier Performance and Exception Report.
13. Supervisor Approval Frequency Report.
14. Failed Authorization Report.
15. Fraud Alert Report.
16. Role and Permission Change Report.
17. Audit Trail Export.

Filter:

* Date range.
* Business date.
* Outlet.
* Register.
* Shift.
* Cashier.
* Supervisor.
* Action type.
* Reason.
* Payment method.
* Amount range.
* Severity.
* Status.

Export laporan sensitif harus memiliki permission dan audit log.

# T. API Design

Gunakan API yang mengikuti convention codebase.

Contoh endpoint:

## Authorization

* POST /api/v1/authorizations/evaluate
* POST /api/v1/authorizations/request
* POST /api/v1/authorizations/verify-pin
* POST /api/v1/authorizations/verify-rfid
* POST /api/v1/authorizations/{id}/reject
* POST /api/v1/authorizations/{id}/revoke

## Void dan refund

* POST /api/v1/transactions/{id}/void/preview
* POST /api/v1/transactions/{id}/void
* POST /api/v1/transactions/{id}/refund/preview
* POST /api/v1/transactions/{id}/refund

## Shift

* POST /api/v1/shifts/open/preview
* POST /api/v1/shifts/open
* GET /api/v1/shifts/current
* POST /api/v1/shifts/{id}/start-closing
* POST /api/v1/shifts/{id}/cash-count
* POST /api/v1/shifts/{id}/recount
* POST /api/v1/shifts/{id}/approve-variance
* POST /api/v1/shifts/{id}/close
* POST /api/v1/shifts/{id}/force-close

## Cash movement

* POST /api/v1/shifts/{id}/cash-in/preview
* POST /api/v1/shifts/{id}/cash-in
* POST /api/v1/shifts/{id}/cash-out/preview
* POST /api/v1/shifts/{id}/cash-out
* POST /api/v1/shifts/{id}/safe-drop/preview
* POST /api/v1/shifts/{id}/safe-drop
* POST /api/v1/cash-movements/{id}/approve
* POST /api/v1/cash-movements/{id}/reject
* POST /api/v1/cash-movements/{id}/reverse

## Drawer

* POST /api/v1/cash-drawers/{id}/open
* GET /api/v1/cash-drawers/{id}/events

Semua command finansial harus menerima:

* expectedVersion
* idempotencyKey
* authorizationToken bila diperlukan
* reasonCode
* notes bila diperlukan
* deviceId
* shiftId

Gunakan preview sebelum commit untuk tindakan sensitif dan nominal besar.

# U. Concurrency dan Idempotency

Gunakan:

* Database transaction.
* Optimistic locking dengan field version.
* Row-level locking ketika commit.
* Idempotency key.
* Unique constraint untuk mencegah duplicate operation.
* Consistent lock ordering.
* Transactional outbox.

Lock minimal:

* Transaction.
* Payment.
* Refund balance.
* Shift.
* Cash drawer session.
* Cash movement.
* Authorization grant.

Authorization grant harus dikonsumsi dalam transaction yang sama dengan tindakan sensitif.

Contoh:

1. Lock authorization grant.
2. Pastikan status APPROVED.
3. Pastikan belum expired.
4. Pastikan belum digunakan.
5. Pastikan action dan resource cocok.
6. Jalankan tindakan.
7. Tandai grant USED.
8. Simpan audit.
9. Simpan outbox event.
10. Commit.

Jika salah satu langkah gagal, rollback seluruhnya.

# V. Offline Mode

Jika aplikasi mendukung offline:

* Jangan menyimpan raw supervisor PIN pada device.
* Gunakan encrypted local storage.
* Credential cache harus memiliki expiration.
* Cache permission harus ditandatangani server.
* Authorization offline harus dibatasi berdasarkan risk policy.
* Refund, post-payment void, cash refund, role changes, dan tindakan high-risk dapat dikonfigurasi `online_only`.
* Sinkronisasi harus menggunakan idempotency key.
* Konflik tidak boleh diselesaikan dengan last-write-wins untuk transaksi finansial.
* Offline action harus diberi flag dan direview setelah sync.
* Local audit event harus ditandatangani atau diberi integrity protection.
* Jika credential tidak dapat diverifikasi secara aman, tindakan harus ditolak.

# W. UI dan UX

## Supervisor authorization popup

Tampilkan:

* Judul tindakan.
* Transaksi atau item.
* Nilai.
* Kasir.
* Reason.
* Warning.
* Input PIN.
* Tombol scan RFID/NFC.
* Cancel.
* Supervisor identity setelah berhasil.

PIN harus masked.

Jangan menampilkan pesan yang memudahkan brute-force. Gunakan pesan umum seperti:

`Otorisasi tidak valid atau tidak memiliki izin.`

Setelah approval:

* Popup tertutup.
* Tindakan dilanjutkan menggunakan grant.
* Grant tidak boleh tersimpan permanen di local storage.
* Grant dibersihkan setelah dipakai atau expired.

## Shift opening

Tampilkan denomination input, total aktual, register, drawer, cashier, dan business date.

## Cash out

Tampilkan kategori, amount, recipient, receipt number, note, attachment, dan approval status.

## Closing count

Gunakan blind count. Jangan menampilkan expected cash sebelum submit jika konfigurasi blind count aktif.

Setelah submit, tampilkan:

* Expected cash.
* Actual cash.
* Variance.
* Status.
* Recount requirement.
* Approval requirement.

# X. Error Code

Gunakan machine-readable error code:

* PERMISSION_DENIED
* SUPERVISOR_REQUIRED
* MANAGER_REQUIRED
* DUAL_APPROVAL_REQUIRED
* INVALID_SUPERVISOR_CREDENTIAL
* AUTHORIZATION_EXPIRED
* AUTHORIZATION_ALREADY_USED
* AUTHORIZATION_SCOPE_MISMATCH
* AUTHORIZATION_LIMIT_EXCEEDED
* SELF_APPROVAL_NOT_ALLOWED
* ACCOUNT_LOCKED
* CREDENTIAL_REVOKED
* RATE_LIMIT_EXCEEDED
* SHIFT_NOT_OPEN
* SHIFT_ALREADY_OPEN
* SHIFT_CLOSING
* SHIFT_ALREADY_CLOSED
* CASH_DRAWER_IN_USE
* PAYMENT_PENDING
* TRANSACTION_ALREADY_PAID
* TRANSACTION_ALREADY_VOIDED
* REFUND_AMOUNT_EXCEEDED
* REFUND_QUANTITY_EXCEEDED
* CASH_MOVEMENT_ALREADY_POSTED
* CASH_MOVEMENT_ALREADY_REVERSED
* INSUFFICIENT_CASH
* RECOUNT_REQUIRED
* VARIANCE_APPROVAL_REQUIRED
* VERSION_CONFLICT
* DUPLICATE_OPERATION
* ONLINE_CONNECTION_REQUIRED
* INVALID_REASON_CODE
* ATTACHMENT_REQUIRED

Frontend harus menerjemahkan error code ke pesan yang dapat dipahami pengguna.

# Y. Domain Event

Publish domain event:

* SupervisorAuthorizationRequested
* SupervisorAuthorizationApproved
* SupervisorAuthorizationRejected
* SupervisorAuthorizationFailed
* SupervisorAuthorizationLocked
* TransactionVoided
* PostPaymentVoidCreated
* RefundRequested
* RefundCompleted
* RefundFailed
* ShiftOpened
* ShiftClosingStarted
* CashCountSubmitted
* CashVarianceDetected
* CashVarianceApproved
* ShiftClosed
* CashInPosted
* CashOutRequested
* CashOutApproved
* CashOutPosted
* CashMovementReversed
* SafeDropInitiated
* SafeDropReceived
* CashDrawerOpened
* FraudAlertCreated
* PermissionChanged
* CredentialRevoked

Gunakan transactional outbox agar event tidak hilang setelah commit.

# Z. Acceptance Criteria

Buat unit test, integration test, security test, dan end-to-end test untuk minimal skenario berikut:

1. Kasir tanpa permission mencoba void dan ditolak.
2. Kasir meminta supervisor authorization.
3. PIN supervisor valid menghasilkan grant.
4. PIN salah tidak menghasilkan grant.
5. PIN tidak pernah muncul dalam log.
6. Account terkunci setelah kegagalan berulang.
7. RFID revoked ditolak.
8. Supervisor dari outlet berbeda ditolak.
9. Supervisor tanpa permission ditolak.
10. Supervisor dengan limit lebih kecil dari amount ditolak.
11. Self-approval high-risk ditolak.
12. Grant hanya berlaku pada transaksi yang disetujui.
13. Grant tidak berlaku pada action lain.
14. Grant expired ditolak.
15. Grant hanya dapat digunakan satu kali.
16. Dua request memakai grant yang sama; hanya satu berhasil.
17. Post-payment item tidak dapat dihapus langsung.
18. Post-payment void membuat reversal dan audit.
19. Refund tidak dapat melebihi refundable balance.
20. Partial refund mengurangi refundable balance.
21. Double refund dicegah dengan idempotency.
22. Refund provider gagal tidak ditandai sukses.
23. Shift tidak dapat dibuka dua kali pada drawer yang sama.
24. Starting cash tersimpan berdasarkan denomination.
25. Starting cash posted tidak dapat diedit.
26. Cash in menambah expected cash.
27. Cash out mengurangi expected cash.
28. Cash out melebihi limit meminta supervisor.
29. Cash out wajib attachment ditolak tanpa attachment.
30. Cash movement posted tidak dapat diedit.
31. Reversal cash movement membuat entry baru.
32. Entry yang sama tidak dapat direversal dua kali.
33. Safe drop mengurangi expected drawer cash.
34. No-sale drawer opening tercatat.
35. Closing count menggunakan blind count.
36. Variance dihitung dengan benar.
37. Variance melebihi tolerance meminta recount.
38. Variance besar meminta manager approval.
39. Shift tidak dapat ditutup ketika payment pending.
40. Shift yang closed tidak dapat menerima transaksi.
41. Audit log memiliki before dan after snapshot.
42. Audit log tidak mengandung credential sensitif.
43. Fraud rule membuat alert untuk excessive void.
44. Fraud rule membuat alert untuk repeated cash short.
45. Multi-terminal update menghasilkan version conflict.
46. Duplicate request tidak membuat operation ganda.
47. Error di tengah transaction menghasilkan rollback penuh.
48. Offline high-risk action ditolak jika policy online-only.
49. Permission change langsung berlaku pada request berikutnya.
50. Disabled supervisor tidak dapat memberi approval.

Gunakan Given, When, Then dan fixture yang jelas.

# Output Implementasi

Sebelum coding, berikan:

1. Ringkasan pemahaman terhadap codebase.
2. Threat model.
3. Daftar fraud scenario.
4. Matriks role dan permission.
5. Matriks sensitive action dan approval level.
6. Desain authorization flow.
7. Desain shift lifecycle.
8. Desain cash ledger.
9. Database migration plan.
10. API contract.
11. Concurrency dan idempotency strategy.
12. Offline security strategy.
13. Daftar file yang akan diubah.
14. Risiko dan compatibility impact.

Setelah analisis, implementasikan secara bertahap:

1. Database migration.
2. Domain model.
3. RBAC dan permission guard.
4. Policy engine.
5. Credential security.
6. Authorization request dan grant.
7. Transaction void/refund integration.
8. Shift dan register session.
9. Cash movement ledger.
10. Starting cash.
11. Cash in dan cash out.
12. Safe drop.
13. Closing count dan reconciliation.
14. Audit trail.
15. Fraud detection.
16. Domain event dan outbox.
17. API.
18. UI.
19. Reporting.
20. Unit test.
21. Integration test.
22. End-to-end test.
23. Security documentation.
24. Operational documentation.

Jangan menghasilkan placeholder, endpoint kosong, mock authorization, TODO tanpa implementasi, atau menyimpan credential dalam plaintext.

Sebelum menyatakan selesai, verifikasi invariant berikut:

* Tidak ada transaksi finansial yang dihapus secara fisik.
* Tidak ada item paid yang dapat dihapus langsung.
* Tidak ada refund melebihi transaksi asli.
* Tidak ada supervisor authorization yang dapat digunakan ulang.
* Tidak ada shared supervisor PIN.
* Tidak ada PIN atau token rahasia di log.
* Tidak ada cash movement posted yang dapat diedit.
* Tidak ada shift ditutup tanpa reconciliation.
* Tidak ada dua kasir menggunakan drawer eksklusif yang sama.
* Tidak ada duplicate refund, void, atau cash movement akibat retry.
* Tidak ada perubahan parsial jika transaction gagal.
* Semua tindakan sensitif memiliki reason, actor, approver, device, dan timestamp.
* Semua perubahan role dan permission memiliki audit trail.
* Semua operasi high-risk mengikuti policy outlet.
* Semua laporan dapat direkonsiliasi dengan ledger dan audit.
