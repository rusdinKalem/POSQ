# LICENSE LIFECYCLE

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menjelaskan lifecycle lisensi dari aktivasi, heartbeat, grace, expired, renewal, revoke, sampai suspicious time.

## 1. Core Rule

Aplikasi diinstall lokal dan menyimpan data operasional lokal, tetapi hak operasionalnya dikontrol oleh signed license token dari control plane server.

License enforcement tidak boleh:

- Menghapus data lokal.
- Mengunci akses historis.
- Menghalangi export.
- Menghalangi backup lokal.
- Menghalangi renewal.

License enforcement boleh:

- Memblokir transaksi baru.
- Memblokir mutation operasional.
- Memblokir fitur premium.
- Memblokir device yang revoked.

## 2. License States

| State | Meaning |
|---|---|
| active | Subscription aktif dan token valid |
| grace | Subscription butuh perhatian, tetapi masih dalam toleransi |
| restricted_expired | Grace selesai; operasi baru diblokir, data lama tetap dapat diakses |
| revoked | Device dicabut oleh control plane |
| suspicious_time | App mendeteksi manipulasi waktu lokal atau state license tidak konsisten |

## 3. Token Fields

Signed token must include:

```text
merchant_id
outlet_id
device_id
plan
entitlements
issued_at
valid_until
paid_until
grace_until
license_status
app_min_version
token_version
signature
```

Recommended token lease:

- MVP: 7 days.
- Higher risk policy: 3 days.
- Avoid 14 days unless support/offline conditions require it.

## 4. First Activation

```text
First-run wizard
  -> owner login online
  -> app generates stable device_id
  -> app sends activation request
  -> server validates merchant/subscription/device limit
  -> server registers device
  -> server issues signed token
  -> app verifies token signature
  -> app stores token and last_server_time
  -> app enters active mode
```

Failure behavior:

| Failure | Behavior |
|---|---|
| Server offline | Setup can continue but operational use requires valid token |
| Device limit reached | Show device limit error and renewal/upgrade CTA |
| Subscription expired | Enter restricted/renewal screen |
| Token invalid | Reject activation |

## 5. Startup Lifecycle

Every startup:

```text
load local license state
  -> verify token signature
  -> verify token device binding
  -> check local time
  -> compare last_server_time and last_seen_local_time
  -> detect rollback
  -> check valid_until/grace_until
  -> if online, heartbeat
  -> determine runtime mode
```

Startup must not depend on server if local token is valid.

## 6. Heartbeat Lifecycle

Heartbeat runs when online:

```text
send device status
  -> server checks subscription
  -> server checks device revoked status
  -> server checks app version policy
  -> server signs refreshed token
  -> app stores refreshed token
  -> app updates last_server_time
```

Heartbeat payload should include:

- merchant_id
- outlet_id
- device_id
- app_version
- local_time
- last_server_time_seen
- token_version
- health summary

Heartbeat response should include:

- runtime_mode
- license_status
- refreshed license token if applicable
- server_time
- next heartbeat interval
- warning messages

## 7. Offline Behavior

| Condition | Behavior |
|---|---|
| Token valid | App runs according to entitlement |
| Subscription likely active but server unreachable | App runs until token/grace limits |
| Token valid_until passed but grace valid | App enters grace |
| grace_until passed | App enters restricted_expired |
| local time suspicious | App enters suspicious_time or requires online verification |

Important:

- User cannot use app indefinitely by staying offline.
- Short token lease and grace limit enforce eventual verification.

## 8. Grace Period

Grace period exists to avoid blocking merchant because of:

- Temporary internet issue.
- Payment processing delay.
- Server outage.
- Weekend/admin delay.

Grace behavior:

- Operational features remain available.
- UI shows clear warning.
- Renewal CTA visible.
- Heartbeat attempts status refresh.

Recommended MVP grace:

- 7 days.

## 9. Restricted Expired Mode

Triggered when:

- Subscription expired.
- Grace period ended.
- No renewed valid token exists.

Allowed:

- Login.
- View old data.
- View reports.
- Export.
- Local backup.
- Restore with confirmation.
- Security/critical update.
- Renewal/payment.
- Support diagnostics.

Blocked:

- Checkout.
- Refund/void.
- Stock adjustment.
- Stock opname submit.
- Purchase/receiving.
- Transfer.
- New F&B order.
- Premium modules.

All blocked commands must return:

```text
LICENSE_RESTRICTED_EXPIRED
```

## 10. Renewal Lifecycle

```text
User opens renewal screen
  -> payment/renewal handled by control plane/admin/payment provider
  -> subscription status becomes active
  -> desktop app calls /licenses/refresh
  -> server returns signed active token
  -> app verifies and stores token
  -> app exits restricted_expired
```

No reinstall required.

## 11. Device Revocation

Revocation use cases:

- Lost device.
- Employee left.
- Device replaced.
- Device limit management.
- Suspicious license abuse.

Behavior:

- Server marks device revoked.
- Next heartbeat returns revoked.
- App enters revoked mode.
- Local data is not deleted.
- Future license refresh blocked.
- Export/backup may remain allowed according to policy.

Device revoke must be audited in admin dashboard.

## 12. Suspicious Time

Triggers:

- Local time moves backward beyond tolerance.
- `last_seen_local_time` inconsistent.
- Token issued_at appears in future.
- Server time and local time drift too far.

Recommended behavior:

- Block operational mutations.
- Require online verification.
- Allow renewal/support.
- Allow export/local backup if policy permits.

Do not silently correct local system time.

## 13. Tamper-Evident Local State

Local license state should include:

- signed token
- token_version
- last_server_time
- last_seen_local_time
- runtime_mode
- tamper_seal

Tamper seal:

- HMAC or local integrity hash.
- Stored separately if possible.
- Does not replace server signature.
- Used to detect local state modification.

## 14. Acceptance Criteria

- App activates device and stores signed token.
- App starts offline with valid token.
- App refreshes token on heartbeat.
- App enters grace after active token window if policy says grace.
- App enters restricted_expired after grace.
- Restricted mode blocks operational mutation commands.
- Restricted mode allows old data/export/backup/renewal.
- Renewal refreshes token without reinstall.
- Device revoke blocks future license refresh without deleting data.
- Clock rollback requires online verification.
- Token tampering fails.
