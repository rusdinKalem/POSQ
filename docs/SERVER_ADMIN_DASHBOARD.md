# SERVER ADMIN DASHBOARD

Project: Aplikasi POS SaaS Indonesia - Server Control Plane  
Purpose: Blueprint dashboard server untuk mengelola merchant, device, license, subscription, update, dan backup metadata.

## 1. Dashboard Role

Dashboard server adalah SaaS admin dashboard, bukan dashboard transaksi toko.

Allowed:

- Merchant profile.
- Subscription status.
- Device status.
- License status.
- App version status.
- Backup metadata.
- Admin audit.
- Support diagnostic metadata.

Not allowed by default:

- Order list.
- Payment transaction list.
- Inventory movement list.
- Customer purchase history.
- Plaintext backup content.

## 2. Admin Roles

| Role | Access |
|---|---|
| super_admin | All control plane functions |
| ops | Merchant, device, update metadata, support metadata |
| billing | Subscription, plan, payment confirmation |
| support | Read support metadata, device/backup/license status |
| release_manager | Publish app version/update metadata |

Production rule:

- Admin MFA required.
- Every mutation requires reason.
- Every mutation writes `admin_audit_logs`.

## 3. Pages

Required pages:

- Login and MFA.
- Merchant list.
- Merchant detail.
- Devices.
- Subscription.
- License.
- Backup metadata.
- Updates.
- Support.
- Audit log.

## 4. Page Requirements

### Merchant List

Show merchant name, status, plan, subscription status, paid_until, device count, last heartbeat, and last backup status.

### Devices

Show device name, OS, app version, status, last heartbeat, and license runtime mode.

Actions:

- Revoke device.
- Rename device metadata.

Forbidden:

- Delete local desktop data.

### Subscription

Actions:

- Change plan.
- Extend paid_until.
- Set grace_until.
- Mark suspended/cancelled.
- Add manual payment confirmation.

Rules:

- Manual changes require reason.
- Subscription event and audit log created.
- Desktop unlocks only after license refresh.

### License

Show token version, license status, runtime mode, valid_until, grace_until, and signing key id.

Forbidden:

- Show raw license token.
- Show private signing key.

### Backup Metadata

Show backup id, destination type, status, created_at, size, checksum, encryption algorithm, app version, DB schema version, and failure code.

Forbidden:

- Download plaintext DB.
- Show recovery key.
- Show BYOS secret.

### Updates

Actions:

- Publish update metadata.
- Assign stable/beta rollout.
- Retire version.

Rules:

- Release manager role required.
- Signature and checksum required.

### Support

Show last heartbeat, runtime mode, app version, local DB health, backup status, error codes, and redacted diagnostic metadata.

Every support view creates `support_access_logs`.

## 5. Dashboard Acceptance Criteria

- Admin can manage merchant lifecycle.
- Admin can activate/revoke devices.
- Billing admin can extend subscription.
- Release manager can publish update metadata.
- Support can view metadata without seeing transaction data.
- All admin mutations create audit logs.
- Dashboard never shows operational POS data by default.

