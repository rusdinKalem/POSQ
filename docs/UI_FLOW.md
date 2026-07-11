# UI FLOW

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan alur layar utama agar implementasi UI Svelte konsisten dengan arsitektur local-first, license, backup, dan control plane.

## 1. UI Principles

- Kasir harus bisa checkout cepat.
- Network status harus jelas tetapi tidak mengganggu.
- License status harus jelas.
- Restricted Expired Mode harus tegas tetapi tidak menyandera data.
- Error harus actionable.
- UI tidak boleh menjadi satu-satunya guard; Rust command tetap enforce.

Global indicators:

- Online/offline.
- Local DB health.
- License mode.
- Last backup status.
- Current shift.

## 2. First-Run Flow

```text
Welcome
  -> PostgreSQL Check
  -> Database Setup
  -> Migration
  -> Merchant/Outlet Setup
  -> Local Owner Setup
  -> Device Activation
  -> Backup Recommendation
  -> Finish
```

### 2.1 Welcome

Purpose:

- Explain local-first POS.
- Explain data stored locally.
- Continue to setup.

Actions:

- Start setup.
- Open setup guide.

### 2.2 PostgreSQL Check

States:

| State | UI |
|---|---|
| PostgreSQL detected | Continue |
| PostgreSQL unavailable | Show setup instructions |
| Credential invalid | Ask for credentials |
| DB exists | Confirm reuse |

### 2.3 Migration

UI requirements:

- Show schema version.
- Show backup requirement if existing DB.
- Show progress.
- Show recovery instruction if failed.

### 2.4 Device Activation

States:

| State | UI |
|---|---|
| Online and subscription active | Activate device |
| Device limit reached | Show upgrade/device management CTA |
| Subscription expired | Show renewal CTA |
| Server unavailable | Allow setup finish but block operational use without valid token |

## 3. Login Flow

```text
Open app
  -> Startup gate
  -> License mode calculated
  -> Local login
  -> Route by mode
```

Route by mode:

| Mode | Destination |
|---|---|
| active | Dashboard/POS |
| grace | Dashboard/POS with warning banner |
| restricted_expired | Expired mode screen |
| revoked | Device revoked screen |
| suspicious_time | Online verification screen |

## 4. Main Dashboard

Dashboard sections:

- Today sales summary.
- Active shift.
- Quick checkout button.
- Low stock alert.
- Last backup status.
- License status.
- Update alert if available.

Dashboard must not require server for local report summary.

## 5. Checkout Flow

```text
Open shift
  -> Product search/scan
  -> Add to cart
  -> Adjust qty/discount
  -> Select payment method
  -> Confirm payment
  -> Save order/payment/stock/audit locally
  -> Receipt preview/print
```

Requirements:

- Checkout must work offline.
- Save operation must be atomic.
- If license mode changes to restricted before payment save, block checkout.
- If stock negative policy triggers, show warning or block according policy.
- Receipt can be previewed even if printer unavailable.

## 6. Shift Flow

```text
Open shift
  -> Starting cash
  -> Active selling
  -> Close shift
  -> Count cash
  -> Shift report
```

Notes:

- Checkout requires active shift.
- Restricted Expired Mode blocks new shift.
- Closing an already-open shift in Restricted Expired Mode should be allowed if policy chooses cash consistency.

## 7. Inventory Flow

Flows:

- Stock in.
- Stock adjustment.
- Stock opname.
- Transfer model.
- Low stock view.

Requirements:

- Every change creates stock_movement.
- Adjustment requires reason.
- Restricted Expired Mode allows view only.

## 8. Backup Flow

```text
Backup settings
  -> choose local/cloud/BYOS
  -> configure destination
  -> configure encryption/recovery key
  -> test backup
  -> enable schedule
```

Backup status UI:

- Last success time.
- Last failure reason.
- Destination type.
- Encryption status.
- Backup size.
- Restore compatibility.

Restricted Expired Mode:

- Local backup remains available.
- Cloud backup policy can be limited; local backup must remain.

## 9. Restore Flow

```text
Select backup
  -> verify manifest
  -> enter recovery key if encrypted
  -> verify checksum
  -> create pre-restore backup
  -> confirm restore
  -> restore
  -> restart/check DB
```

UI must clearly warn:

- Restore modifies current local database.
- Pre-restore backup will be created.
- Wrong key will not alter current data.

## 10. Restricted Expired Mode UI

Purpose:

- Explain subscription expired.
- Block new operations.
- Preserve access to data and recovery tools.

Screen sections:

- Status: subscription expired.
- What is blocked.
- What is still available.
- Renewal button.
- Export data button.
- Local backup button.
- Support button.
- License refresh button.

Allowed navigation:

- Historical orders.
- Reports.
- Export.
- Backup.
- Restore.
- Renewal.
- Settings read-only.

Blocked navigation:

- Checkout.
- Inventory mutation.
- Refund/void.
- New F&B order.
- Premium modules.

Recommended copy:

```text
Langganan sudah berakhir. Untuk menjaga akses data Anda, aplikasi masih dapat digunakan untuk melihat data lama, export, backup, restore, dan perpanjangan. Fitur operasional baru seperti checkout dan perubahan stok akan aktif kembali setelah langganan diperpanjang.
```

## 11. Device Revoked UI

Show:

- Device revoked status.
- Reason if available.
- Contact owner/support.
- Export/backup access if policy allows.
- Reactivation request if allowed.

Do not delete local data.

## 12. Suspicious Time UI

Show:

- App detected time inconsistency.
- Ask user to connect internet for verification.
- Allow support/renewal.
- Allow export/backup depending policy.

Do not accuse user directly.

Recommended copy:

```text
Waktu perangkat perlu diverifikasi sebelum fitur operasional dapat digunakan. Hubungkan internet untuk memverifikasi lisensi.
```

## 13. Admin Dashboard UI

Server admin dashboard modules:

- Merchant list.
- Merchant profile.
- Device list.
- License status.
- Subscription status.
- Backup metadata.
- Update channel.
- Admin audit log.

Dashboard must not show by default:

- Orders.
- Payments.
- Inventory movement.
- Customer purchase history.
- Plaintext backup content.

## 14. Error UI

Every error should show:

- What happened.
- Why it matters.
- What user can do next.
- Error code.
- Support diagnostic option if needed.

Example:

```text
Checkout tidak dapat dilanjutkan karena langganan sudah berakhir.
Kode: LICENSE_RESTRICTED_EXPIRED
Perpanjang langganan untuk mengaktifkan kembali checkout.
```

## 15. Acceptance Criteria

- First-run can complete on clean install.
- Checkout screen works offline.
- Dashboard does not depend on server.
- Restricted Expired Mode blocks operational screens but allows export/backup/renewal.
- Backup and restore flows require confirmation and show clear state.
- Device revoked and suspicious time screens do not delete or hide local data.
- Every blocked action has clear error and CTA.
