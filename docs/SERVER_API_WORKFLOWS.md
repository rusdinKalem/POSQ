# SERVER API WORKFLOWS

Project: Aplikasi POS SaaS Indonesia - Server Control Plane  
Purpose: Workflow API server yang matching dengan desktop POS local-first.

## 1. Endpoint Groups

Base path:

```text
/api/v1
```

Groups:

- `/auth/*`
- `/merchants/*`
- `/devices/*`
- `/licenses/*`
- `/subscriptions/*`
- `/updates/*`
- `/backups/*`
- `/support/*`
- `/admin/*`

## 2. Device Activation

```mermaid
sequenceDiagram
  participant D as Desktop POS
  participant API as Control Plane API
  participant DB as Server PostgreSQL
  participant L as License Signer
  D->>D: generate install_id + device key pair
  D->>API: POST /devices/activation-challenge
  API-->>D: challenge + expires_at
  D->>API: POST /devices/activate + signed challenge
  API->>DB: validate merchant/subscription/device limit
  API->>DB: verify device public key and upsert device
  API->>L: sign license token
  L-->>API: signed token
  API->>DB: store device_license token_hash/version
  API-->>D: license_token + server_time + runtime_mode
```

Rules:

- Activation requires active or grace subscription unless admin override exists.
- Device count must obey plan.
- Response must include server_time for local anti-clock-rollback.
- Activation must bind license to `install_id_hash` and `device_public_key_thumbprint`.
- Device private key must not leave the desktop.
- Repeated activation request must be idempotent.

## 3. Heartbeat and License Refresh

```mermaid
sequenceDiagram
  participant D as Desktop POS
  participant API as Control Plane API
  participant DB as Server PostgreSQL
  participant L as License Signer
  D->>D: build heartbeat body + nonce
  D->>API: POST /devices/heartbeat + device signature
  API->>DB: validate device status
  API->>API: verify device signature and nonce
  API->>DB: compute subscription state
  API->>API: detect suspicious time signals
  API->>L: sign refreshed token if needed
  API->>DB: update heartbeat/license version
  API-->>D: runtime_mode + optional refreshed token
```

Runtime modes:

- active
- grace
- restricted_expired
- revoked
- suspicious_time

Heartbeat rules:

- Server rejects replayed nonce.
- Server rejects request signed by unknown device key.
- Server derives merchant/device context from verified binding.
- Server may return same token if no state change is required.

## 4. Manual Renewal MVP

```mermaid
sequenceDiagram
  participant Owner as Merchant Owner
  participant Admin as Admin Dashboard
  participant API as Control Plane API
  participant DB as Server PostgreSQL
  participant D as Desktop POS
  Owner->>Owner: pays via manual channel/payment link
  Admin->>API: extend subscription paid_until
  API->>DB: update subscription
  API->>DB: write subscription_event
  API->>DB: write admin_audit_log
  D->>API: POST /licenses/refresh
  API-->>D: signed active license token
```

Rules:

- Duplicate renewal event must not double-extend subscription.
- Every manual mutation requires admin reason.
- High-risk billing mutation requires admin MFA step-up.
- Desktop exits Restricted Expired Mode only after receiving valid signed token.

## 5. Backup Metadata Upload

```mermaid
sequenceDiagram
  participant D as Desktop POS
  participant S as Object Storage
  participant API as Control Plane API
  participant DB as Server PostgreSQL
  D->>D: create backup manifest/checksum
  D->>D: encrypt backup client-side
  D->>S: upload encrypted backup payload
  D->>API: POST /backups/metadata
  API->>DB: validate idempotency key
  API->>DB: upsert backup_metadata
  API-->>D: metadata accepted
```

Rules:

- API metadata endpoint must not accept database dump payload.
- Cloud backup must be encrypted before upload.
- Server stores checksum, size, status, storage logical ref, and compatibility info only.
- Managed cloud upload must use short-lived pre-signed URL with content length and checksum constraints.
- BYOS validation must block SSRF targets.

## 6. Update Check

```mermaid
sequenceDiagram
  participant D as Desktop POS
  participant API as Control Plane API
  participant DB as Server PostgreSQL
  D->>API: GET /updates/check?os=windows&channel=stable&version=1.0.0
  API->>DB: find compatible app version
  API-->>D: version, sha256, signature, download_url, critical flag
  D->>D: verify signature and sha256 before install
```

## 7. Idempotency Rules

Mutating endpoints requiring `Idempotency-Key`:

- `POST /devices/activate`
- `POST /licenses/refresh`
- `POST /subscriptions/manual-renewal`
- `POST /backups/metadata`
- `POST /updates/publish`
- `POST /admin/devices/{id}/revoke`

Behavior:

- Same key + same payload returns same result.
- Same key + different payload returns `IDEMPOTENCY_CONFLICT`.
- Processing failures are safe to retry.
- Idempotency record must store request hash and response.
- Same key processing must use transaction lock or equivalent race protection.

## 8. Authorization Rules

Server must protect against object-level authorization failures:

- Do not trust `merchant_id` from request body.
- Resolve tenant from session/device binding.
- Every object id lookup must include tenant scope.
- Admin cross-tenant access requires role, reason, and audit log.
- Response DTOs must be allowlisted to prevent object property exposure.

## 9. API Acceptance Criteria

- All server responses include `server_time`.
- All mutating retryable endpoints support idempotency.
- Tenant scope is enforced on every merchant-owned resource.
- Runtime mode returned by server matches `LICENSE_LIFECYCLE.md`.
- API never requires checkout data before allowing local checkout.
- API contract is compatible with `API_SPEC.md`.
- Device-bound license prevents token copy to another PC.
- BYOS validation cannot reach private/internal network resources.
