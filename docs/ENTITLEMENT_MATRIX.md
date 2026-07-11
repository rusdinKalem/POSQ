# ENTITLEMENT MATRIX

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan hak akses fitur berdasarkan plan subscription dan runtime license mode agar UI dan Rust command handler konsisten.

## 1. Principles

- Entitlement harus ditegakkan di UI dan Rust/local service.
- UI boleh menyembunyikan/disable fitur, tetapi Rust command tetap wajib melakukan guard.
- License mode mengontrol apakah fitur boleh dijalankan.
- Plan mengontrol fitur apa saja yang tersedia.
- RBAC mengontrol siapa yang boleh menjalankan fitur.
- Semua guard harus menghasilkan error code yang jelas.

Guard order:

```text
authentication
  -> local user active
  -> RBAC permission
  -> license mode
  -> plan entitlement
  -> domain validation
  -> database transaction
```

## 2. Runtime License Modes

| Mode | Meaning |
|---|---|
| active | Subscription aktif dan token valid |
| grace | Subscription bermasalah/expired sementara, tetapi masih dalam grace period |
| restricted_expired | Grace period selesai; operasi baru diblokir, data lama tetap dapat diakses |
| revoked | Device dicabut dari control plane |
| suspicious_time | App mendeteksi manipulasi waktu lokal atau state license tidak konsisten |

## 3. Feature Access by Runtime Mode

Legend:

- `ALLOW`: fitur berjalan.
- `WARN`: fitur berjalan dengan warning renewal.
- `READ`: hanya baca.
- `BLOCK`: ditolak.
- `SUPPORT`: hanya untuk renewal/support/recovery.
- `POLICY`: tergantung keputusan produk/security.

| Feature | active | grace | restricted_expired | revoked | suspicious_time |
|---|---|---|---|---|---|
| Open app | ALLOW | ALLOW | ALLOW | ALLOW | ALLOW |
| Local login | ALLOW | ALLOW | ALLOW | ALLOW | ALLOW |
| View dashboard summary | ALLOW | WARN | READ | READ | SUPPORT |
| Product view | ALLOW | WARN | READ | READ | READ |
| Product create/edit | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Checkout new order | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Hold order | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Refund | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Void | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Discount override | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Open shift | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Close active shift | ALLOW | WARN | POLICY | POLICY | POLICY |
| View old shifts | ALLOW | WARN | READ | READ | READ |
| Stock view | ALLOW | WARN | READ | READ | READ |
| Stock in | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Stock adjustment | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Stock opname submit | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Stock transfer | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| View reports | ALLOW | WARN | READ | READ | READ |
| Export reports | ALLOW | WARN | ALLOW | ALLOW | POLICY |
| Local backup | ALLOW | WARN | ALLOW | ALLOW | POLICY |
| Cloud backup | ALLOW | WARN | POLICY | POLICY | BLOCK |
| Restore backup | ALLOW | WARN | ALLOW | POLICY | POLICY |
| Update security/critical | ALLOW | ALLOW | ALLOW | ALLOW | ALLOW |
| Update feature/non-critical | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Renewal/payment screen | ALLOW | ALLOW | ALLOW | ALLOW | ALLOW |
| License refresh | ALLOW | ALLOW | ALLOW | BLOCK if revoked | ALLOW only online |
| Device reactivation | ALLOW | ALLOW | ALLOW | SUPPORT | SUPPORT |
| Admin settings local | ALLOW | WARN | READ | READ | READ |
| F&B table order | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Kitchen display new item | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Retail return | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| API/webhook create operation | ALLOW | WARN | BLOCK | BLOCK | BLOCK |
| Support diagnostic bundle | ALLOW | ALLOW | ALLOW | ALLOW | ALLOW |

Recommended policy decisions:

- Closing an already active shift in `restricted_expired` should be allowed if it prevents cash/report inconsistency.
- Export and local backup should remain allowed in `suspicious_time` unless tampering is severe.
- Cloud backup in `restricted_expired` may be allowed only if already entitled and no new paid operation is created; MVP can limit to local backup to reduce complexity.

## 4. Plan Entitlement Matrix

| Feature | Starter | Growth | Pro | Business | Enterprise |
|---|---|---|---|---|---|
| Device count | 1 | 2-3 | 5-10 | 10+ | Custom |
| Outlet count | 1 | 1 | 2-5 | 5+ | Custom |
| Local checkout | yes | yes | yes | yes | yes |
| Product/catalog | yes | yes | yes | yes | yes |
| Shift | yes | yes | yes | yes | yes |
| Basic local report | yes | yes | yes | yes | yes |
| Inventory basic | limited | yes | yes | yes | yes |
| RBAC basic | owner/cashier | basic roles | custom-ish | custom | custom |
| Audit log | yes | yes | yes | yes | yes |
| Local backup | manual | manual/scheduled | scheduled | configurable | custom |
| Managed cloud backup | no/add-on | yes | yes | yes | custom |
| BYOS backup | no | no | yes | yes | custom |
| F&B table/modifier | no | basic/add-on | yes | yes | custom |
| Kitchen display/printer | no | basic/add-on | yes | yes | custom |
| Retail return | basic | yes | yes | yes | custom |
| Serial number | no | no | yes | yes | custom |
| Multi-outlet basic | no | no | yes | yes | custom |
| Approval flow | no | no | limited | yes | custom |
| API/webhook | no | no | limited | yes | custom |
| Priority support | no | no | yes | yes | custom SLA |
| Private deployment | no | no | no | optional | yes |

## 5. Role Permission Matrix

| Feature | Owner | Manager | Cashier | Inventory | Finance | Kitchen/Waiter | Super Admin |
|---|---|---|---|---|---|---|---|
| Checkout | yes | yes | yes | no by default | no | no | no |
| Open/close shift | yes | yes | yes | no | no | no | no |
| Refund | yes | yes | no by default | no | no | no | no |
| Void | yes | yes | no by default | no | no | no | no |
| Discount override | yes | yes | limited | no | no | no | no |
| Product edit | yes | yes | no | yes | no | no | no |
| Stock adjustment | yes | yes | no | yes | no | no | no |
| Stock opname | yes | yes | no | yes | no | no | no |
| View reports | yes | yes | no by default | limited | yes | no | no |
| Export reports | yes | yes | no | no | yes | no | no |
| Backup/restore | yes | no by default | no | no | no | no | no |
| Local user management | yes | no by default | no | no | no | no | no |
| License/renewal | yes | no by default | no | no | no | no | no |
| Kitchen order view | yes | yes | no | no | no | yes | no |
| Control plane admin | no | no | no | no | no | no | yes |

## 6. Command Guard Examples

All Rust commands that mutate operational data must call entitlement guard.

Example pseudo-flow:

```text
create_order(input)
  -> require_authenticated_user()
  -> require_rbac("checkout:create")
  -> require_license_mode(["active", "grace"])
  -> require_plan_entitlement("local_checkout")
  -> validate_shift_open()
  -> validate_cart()
  -> write order/payment/stock movement/audit in one DB transaction
```

```text
export_report(input)
  -> require_authenticated_user()
  -> require_rbac("report:export")
  -> require_license_mode(["active", "grace", "restricted_expired", "revoked"])
  -> validate_date_range()
  -> generate export
```

```text
local_backup(input)
  -> require_authenticated_user()
  -> require_rbac("backup:create")
  -> require_license_mode(["active", "grace", "restricted_expired", "revoked"])
  -> verify_disk_space()
  -> create backup manifest
  -> write checksum
```

## 7. Error Codes

| Error Code | Trigger |
|---|---|
| AUTH_REQUIRED | User is not logged in |
| RBAC_DENIED | User role lacks permission |
| ENTITLEMENT_DENIED | Plan does not include feature |
| LICENSE_EXPIRED | License expired and action requires active/grace |
| LICENSE_RESTRICTED_EXPIRED | Action blocked in Restricted Expired Mode |
| DEVICE_REVOKED | Device is revoked |
| CLOCK_ROLLBACK_DETECTED | Suspicious local time |
| SHIFT_REQUIRED | Checkout requires active shift |
| BACKUP_REQUIRED | Operation requires backup first |

## 8. Acceptance Criteria

- UI and Rust command behavior match this matrix.
- Restricted Expired Mode blocks all operational mutation commands.
- Restricted Expired Mode allows historical read, export, local backup, restore, renewal.
- RBAC denial and license denial produce different error codes.
- Plan entitlement denial and role denial produce different error codes.
- Tests cover at least one allowed and one blocked action for each runtime mode.
