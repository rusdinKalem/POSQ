# SERVER MATCHING MATRIX

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Memastikan blueprint server matching dengan blueprint local.

## 1. Local vs Server Responsibility

| Capability | Local Desktop | Server Control Plane |
|---|---|---|
| Product catalog | Source of truth local | No default copy |
| Checkout | Source of truth local | Not involved |
| Payment record | Source of truth local | Not stored by default |
| Stock movement | Source of truth local | Not stored by default |
| Shift | Source of truth local | Not stored by default |
| Local reports | Computed local | Not computed from transaction data |
| Merchant account | Local seed/cache | Source of truth |
| Owner credential cloud | Login consumer | Source of truth |
| Local cashier PIN/user | Source of truth local | Not stored by default |
| Device activation | Stores signed token | Source of truth |
| Device private key | Source of truth local secure storage | Never stored |
| Device public key | Sends during activation | Stores thumbprint/public key |
| Subscription | Reads signed token | Source of truth |
| License validation | Verifies token | Issues token |
| Update | Downloads/verifies | Publishes metadata |
| Local backup | Creates file | No payload |
| Cloud backup | Encrypts/uploads | Metadata only |
| Admin support | Shows health local | Metadata dashboard |

## 2. Endpoint to Local Feature Mapping

| Local Feature | Server Endpoint | Required? | Offline Behavior |
|---|---|---:|---|
| First activation | `POST /devices/activate` | Yes once | Cannot activate new device offline |
| Daily checkout | None | No | Works offline |
| Heartbeat | `POST /devices/heartbeat` signed by device key | Periodic | Uses local signed token until lease/grace ends |
| Renewal refresh | `POST /licenses/refresh` | When paid | Renewal requires online |
| Update check | `GET /updates/check` | Optional periodic | App continues current version |
| Backup metadata | `POST /backups/metadata` | Optional | Local backup still works |
| Support diagnostic | `POST /support/diagnostics` | User consent | User can export/send manually |

## 3. Runtime Mode Matching

| Server Subscription State | Server License Status | Desktop Runtime Mode | Desktop Allowed |
|---|---|---|---|
| active | active | active | normal operations |
| grace | grace | grace | operations with warning |
| expired | restricted_expired | restricted_expired | old data/export/local backup/renewal only |
| suspended | restricted_expired or revoked | restricted_expired/revoked | depends policy |
| cancelled | restricted_expired | restricted_expired | old data/export/local backup/renewal only |
| device revoked | revoked | revoked | no new paid operations; data safety actions remain |
| suspicious time | suspicious_time | suspicious_time | require online verification; preserve data safety |

## 4. Must Not Drift

The implementation is wrong if:

- Server checkout endpoint is required for normal sale.
- Server has default `orders`, `payments`, or `stock_movements` tables.
- Admin dashboard shows merchant transaction data by default.
- Cloud backup uploads plaintext database.
- Expired subscription hides old local data.
- Server can unlock desktop without signed license token.
- Signed license token works after being copied to another device.
- Desktop trusts unsigned update metadata.
- Server trusts merchant_id from request body without auth/device scope.
- BYOS validation can reach private/internal network resources.
