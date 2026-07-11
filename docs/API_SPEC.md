# API SPEC - CONTROL PLANE

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan kontrak API server control plane untuk account, credential, device, license, subscription, update, backup metadata, dan admin dashboard.

## 1. API Scope

Server API adalah control plane. API ini tidak menjadi tempat penyimpanan transaksi operasional merchant secara default.

Allowed server data:

- Merchant account.
- Credential/security state.
- Device registration.
- License token state.
- Subscription/billing state.
- App update metadata.
- Backup metadata.
- Admin audit log.

Rejected by default:

- Full order sync.
- Full payment history.
- Inventory movement sync.
- Customer purchase history sync.
- Plaintext backup upload.

Any endpoint that stores operational merchant transactions requires new ADR.

## 2. Base Conventions

Base path:

```text
/api/v1
```

Required headers:

```text
Content-Type: application/json
Authorization: Bearer <access_token>
X-Request-Id: <uuid>
Idempotency-Key: <uuid>  # required for mutating requests that can retry
```

Standard response:

```json
{
  "success": true,
  "data": {},
  "error": null,
  "server_time": "2026-07-05T10:00:00Z"
}
```

Standard error:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "LICENSE_RESTRICTED_EXPIRED",
    "message": "Action is not allowed in restricted expired mode.",
    "details": {}
  },
  "server_time": "2026-07-05T10:00:00Z"
}
```

## 3. Error Codes

| Code | HTTP | Meaning |
|---|---:|---|
| AUTH_REQUIRED | 401 | Missing token |
| AUTH_INVALID | 401 | Invalid token/session |
| RBAC_DENIED | 403 | User lacks permission |
| TENANT_SCOPE_DENIED | 403 | Cross-tenant access blocked |
| DEVICE_LIMIT_EXCEEDED | 403 | Plan device limit reached |
| DEVICE_REVOKED | 403 | Device revoked |
| SUBSCRIPTION_EXPIRED | 402 | Subscription expired |
| LICENSE_EXPIRED | 402 | License expired |
| LICENSE_RESTRICTED_EXPIRED | 402 | Restricted expired state |
| CLOCK_ROLLBACK_DETECTED | 409 | Suspicious device time |
| TOKEN_INVALID | 401 | Signed token invalid |
| ENTITLEMENT_DENIED | 403 | Plan does not include feature |
| IDEMPOTENCY_CONFLICT | 409 | Same key used with different payload |
| BACKUP_METADATA_DUPLICATE | 200/409 | Duplicate metadata depending on idempotency result |
| UPDATE_SIGNATURE_INVALID | 422 | Update metadata/package invalid |
| VALIDATION_ERROR | 422 | Invalid payload |
| RATE_LIMITED | 429 | Too many requests |
| SERVER_ERROR | 500 | Unexpected server error |

## 4. Authentication Endpoints

### POST /auth/login

Purpose: Login owner/admin to control plane.

Request:

```json
{
  "email": "owner@example.com",
  "password": "password",
  "device_hint": "KASIR-01"
}
```

Response:

```json
{
  "access_token": "jwt-or-opaque-token",
  "refresh_token": "refresh-token",
  "merchant_id": "uuid",
  "user_id": "uuid",
  "requires_mfa": false
}
```

Security:

- Rate limited.
- Password never logged.
- MFA required for admin dashboard roles.

### POST /auth/refresh

Purpose: Refresh access token.

Request:

```json
{
  "refresh_token": "refresh-token"
}
```

Response:

```json
{
  "access_token": "new-access-token",
  "expires_in": 3600
}
```

### POST /auth/logout

Purpose: End session.

Response:

```json
{
  "logged_out": true
}
```

## 5. Device and License Endpoints

### POST /devices/activate

Purpose: Register a local POS device and issue signed license token.

Request:

```json
{
  "merchant_id": "uuid",
  "outlet_id": "uuid",
  "device_id": "stable-local-device-id",
  "device_name": "KASIR-01",
  "app_version": "1.0.0",
  "os": "windows",
  "install_id_hash": "sha256",
  "device_fingerprint_hash": "sha256",
  "device_public_key": "pem-or-jwk",
  "activation_challenge_id": "uuid",
  "activation_challenge_signature": "base64-signature"
}
```

Response:

```json
{
  "device_id": "uuid",
  "license_token": "signed-license-token",
  "license_status": "active",
  "valid_until": "2026-07-12T10:00:00Z",
  "grace_until": "2026-07-19T10:00:00Z",
  "server_time": "2026-07-05T10:00:00Z",
  "next_heartbeat_after_seconds": 3600
}
```

Validation:

- Merchant must exist.
- Outlet must belong to merchant.
- Device count must not exceed plan.
- Subscription must allow activation.
- Activation challenge must be valid and unused.
- Device public key must verify challenge signature.
- License token must be bound to device id, install id hash, and device public key thumbprint.

### POST /devices/heartbeat

Purpose: Device status update and license state refresh.

Request:

```json
{
  "merchant_id": "uuid",
  "outlet_id": "uuid",
  "device_id": "uuid",
  "app_version": "1.0.0",
  "local_time": "2026-07-05T17:00:00+07:00",
  "last_server_time_seen": "2026-07-05T09:00:00Z",
  "license_token_version": 3,
  "nonce": "uuid-or-random-base64",
  "device_signature": "signature-over-canonical-request-body",
  "health": {
    "db": "ok",
    "backup": "ok",
    "disk_free_mb": 10240
  }
}
```

Response:

```json
{
  "license_status": "active",
  "license_token": "refreshed-signed-license-token",
  "runtime_mode": "active",
  "server_time": "2026-07-05T10:00:00Z",
  "next_heartbeat_after_seconds": 3600,
  "messages": []
}
```

Possible runtime modes:

```text
active
grace
restricted_expired
revoked
suspicious_time
```

Validation:

- Device signature must match activated device public key.
- Nonce must not have been used before within replay retention window.
- Merchant/device context must be derived from verified device binding, not trusted from body alone.

### POST /licenses/refresh

Purpose: Explicit refresh after renewal/payment.

Request:

```json
{
  "merchant_id": "uuid",
  "device_id": "uuid",
  "current_license_token_version": 3
}
```

Response:

```json
{
  "license_token": "signed-license-token",
  "license_status": "active",
  "runtime_mode": "active"
}
```

### GET /subscriptions/current

Purpose: Get subscription state for merchant.

Response:

```json
{
  "merchant_id": "uuid",
  "plan": "growth",
  "status": "active",
  "paid_until": "2026-08-05T00:00:00Z",
  "grace_until": "2026-08-12T00:00:00Z",
  "entitlements": {
    "device_limit": 3,
    "managed_cloud_backup": true,
    "byos_backup": false,
    "fnb_mode": "basic"
  }
}
```

## 6. Backup Metadata Endpoints

### POST /backup-metadata

Purpose: Store metadata for backup without storing plaintext backup content.

Headers:

```text
Idempotency-Key: <backup_id or uuid>
```

Request:

```json
{
  "backup_id": "uuid",
  "merchant_id": "uuid",
  "outlet_id": "uuid",
  "device_id": "uuid",
  "destination_type": "local|managed_cloud|byos_s3",
  "storage_ref": "logical/path/or/object-key",
  "size_bytes": 123456789,
  "checksum_sha256": "hex",
  "encrypted": true,
  "encryption_alg": "AES-256-GCM",
  "app_version": "1.0.0",
  "db_schema_version": "202607050001",
  "created_at": "2026-07-05T10:00:00Z",
  "restore_compatibility_version": "1"
}
```

Response:

```json
{
  "backup_id": "uuid",
  "status": "recorded",
  "server_time": "2026-07-05T10:00:00Z"
}
```

Rules:

- Reject plaintext backup metadata if `encrypted=false` for cloud destination.
- Do not accept full database payload.
- Duplicate idempotency key with same payload returns same result.
- Duplicate idempotency key with different payload returns `IDEMPOTENCY_CONFLICT`.

### GET /backup-metadata

Purpose: List backup metadata for merchant/device.

Query:

```text
merchant_id
device_id optional
limit
cursor
```

Response:

```json
{
  "items": [
    {
      "backup_id": "uuid",
      "device_id": "uuid",
      "destination_type": "managed_cloud",
      "size_bytes": 123456789,
      "checksum_sha256": "hex",
      "encrypted": true,
      "created_at": "2026-07-05T10:00:00Z",
      "status": "available"
    }
  ],
  "next_cursor": null
}
```

## 7. Update Endpoints

### GET /app-updates/check

Purpose: Check update availability.

Query:

```text
app_version=1.0.0
os=windows
channel=stable
device_id=uuid
```

Response:

```json
{
  "update_available": true,
  "version": "1.0.1",
  "critical": false,
  "minimum_supported_version": "1.0.0",
  "download_url": "signed-or-controlled-url",
  "signature": "update-signature",
  "sha256": "hex",
  "release_notes": "Bug fixes",
  "requires_backup_before_migration": true
}
```

Rules:

- App verifies signature before install.
- Critical/security update remains allowed in Restricted Expired Mode.
- Feature/non-critical update can be blocked in Restricted Expired Mode.

## 8. Admin Dashboard Endpoints

Admin endpoints require admin session, MFA where configured, RBAC, tenant scope, and audit log.

### GET /admin/merchants

Purpose: List merchants.

### GET /admin/merchants/{merchant_id}

Purpose: Merchant profile and control-plane status only.

Must not include:

- orders
- payments
- inventory movements
- customer purchase history

### POST /admin/devices/{device_id}/revoke

Request:

```json
{
  "reason": "Device lost",
  "requested_by": "admin-user-id"
}
```

Response:

```json
{
  "device_id": "uuid",
  "status": "revoked"
}
```

Rules:

- Must create admin audit log.
- Must not delete local data.
- Device learns revoke on next heartbeat/license refresh.

### POST /admin/subscriptions/{merchant_id}/override

Request:

```json
{
  "status": "active|grace|expired|suspended",
  "plan": "starter|growth|pro|business|enterprise",
  "paid_until": "2026-08-05T00:00:00Z",
  "reason": "Manual payment confirmed"
}
```

Rules:

- Requires Billing Admin or Super Admin.
- Must create admin audit log.
- App receives new state through heartbeat/license refresh.

### POST /admin/app-versions

Purpose: Create update metadata.

Request:

```json
{
  "version": "1.0.1",
  "channel": "stable",
  "os": "windows",
  "critical": false,
  "minimum_supported_version": "1.0.0",
  "download_url": "url",
  "signature": "signature",
  "sha256": "hex"
}
```

## 9. Idempotency Rules

Endpoints requiring idempotency:

- `POST /devices/activate`
- `POST /devices/heartbeat`
- `POST /licenses/refresh`
- `POST /backup-metadata`
- Admin mutation endpoints.

Rules:

- Same key + same payload returns same result.
- Same key + different payload returns `IDEMPOTENCY_CONFLICT`.
- Store idempotency key with request hash and response hash.
- Expiry for idempotency records: recommended 7-30 days depending endpoint.

## 10. Acceptance Criteria

- No default endpoint stores full merchant operational data.
- Device activation issues signed license token.
- Heartbeat refreshes token and runtime mode.
- Expired subscription returns `restricted_expired`.
- Backup metadata accepts encrypted metadata only for cloud destinations.
- Duplicate backup metadata does not create duplicate records.
- Admin device revoke creates audit log.
- Subscription override creates audit log.
- Update check returns signature and hash.
- Tenant scope is enforced on every endpoint.
