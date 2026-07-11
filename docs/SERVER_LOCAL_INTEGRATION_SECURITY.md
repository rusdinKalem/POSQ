# SERVER-LOCAL INTEGRATION SECURITY

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Mengunci kontrak keamanan antara desktop local app dan server control plane.

## 1. Security Goal

Integrasi local-server harus mencapai tiga tujuan:

1. Desktop tetap bisa checkout saat server offline.
2. Server tetap bisa mengontrol subscription, device, update, dan backup metadata.
3. Celah umum seperti token copy, tenant bypass, replay, plaintext backup, SSRF, dan admin abuse dapat dicegah atau terdeteksi.

## 2. Trust Boundary

Local desktop tidak boleh dianggap sepenuhnya trusted. Server juga tidak boleh menjadi dependency checkout harian.

Rules:

- Server tidak mempercayai `merchant_id`, `device_id`, atau `outlet_id` dari request body tanpa validasi session/device binding.
- Desktop tidak mempercayai server response untuk update/license kecuali signature valid.
- Desktop tidak pernah menjalankan operational DB mutation hanya karena server menyuruh.
- Server tidak menerima operational DB payload kecuali user melakukan explicit export flow yang belum termasuk MVP.

## 3. Device-Bound License

License token harus diikat ke device agar tidak mudah disalin ke PC lain.

Activation flow:

1. Desktop membuat `install_id`.
2. Desktop membuat device key pair lokal.
3. Private key disimpan di OS secure storage jika tersedia.
4. Desktop mengirim `device_public_key`, `install_id_hash`, `device_fingerprint_hash`, app version, dan OS ke server.
5. Server membuat challenge.
6. Desktop menandatangani challenge dengan private key.
7. Server memverifikasi signature.
8. Server menerbitkan signed license token.

License token claims minimal:

```json
{
  "iss": "pos-control-plane",
  "aud": "pos-desktop",
  "sub": "device:<device_id>",
  "jti": "uuid",
  "merchant_id": "uuid",
  "device_id": "uuid",
  "install_id_hash": "sha256",
  "device_public_key_thumbprint": "sha256",
  "token_version": 1,
  "runtime_mode": "active",
  "entitlements": {},
  "iat": 1780000000,
  "nbf": 1780000000,
  "exp": 1780604800,
  "grace_until": "2026-07-19T10:00:00Z"
}
```

Desktop verification must check:

- Expected algorithm, not only token header.
- `kid` maps to pinned/known public key.
- `iss`.
- `aud`.
- `exp`.
- `nbf`.
- `merchant_id`.
- `device_id`.
- `install_id_hash`.
- `device_public_key_thumbprint`.
- `token_version`.
- Signature.

## 4. Heartbeat Security

Heartbeat request must include:

- Device id.
- App version.
- Local runtime mode.
- Last server time seen.
- Local time.
- License token version.
- Device nonce.
- Signature over request body using device private key.

Server must:

- Verify device is active.
- Verify request signature.
- Reject stale nonce/replay.
- Detect suspicious clock rollback.
- Return server_time.
- Return refreshed token only if license state changed or lease refresh is due.

## 5. Replay and Token Copy Defense

Required:

- Short-lived license token.
- Device-bound claims.
- Device private key challenge-response.
- Nonce on heartbeat and refresh.
- Token versioning.
- Revocation list by device id and token version.
- Server-side last_seen tracking.

If private key is lost or OS secure storage is reset:

- Device must require reactivation.
- Existing device may be revoked or marked replaced.
- Local data must remain exportable/backuppable.

## 6. Offline Rules

Desktop may operate offline only while:

- Signed local token is valid, or
- Grace policy allows it, or
- Restricted Expired Mode allows data-safety actions.

Desktop must not:

- Extend its own subscription.
- Mint license token.
- Change entitlements.
- Ignore token expiry by using local clock only.

## 7. Tenant and Object Authorization

Server must derive tenant scope from authenticated session or verified device binding.

Prohibited:

- Querying `merchant_id` from request body without comparing to auth context.
- Admin endpoint that accepts arbitrary merchant id without role and reason.
- Returning object fields not required by the caller.
- Mass assignment from request JSON into DB model.

Every server query touching tenant data must include tenant scope unless it is a super_admin operation with audit reason.

## 8. Backup Integration Security

Cloud backup flow:

1. Desktop creates local backup.
2. Desktop creates manifest and checksum.
3. Desktop encrypts backup client-side.
4. Desktop uploads encrypted payload to storage.
5. Desktop uploads metadata to server.

Server must not:

- Receive raw DB dump through metadata endpoint.
- Store recovery key.
- Store BYOS secret in plaintext.
- Generate restore without local key validation.

Managed storage upload should use short-lived pre-signed URLs with:

- Content length limit.
- Expected checksum.
- Expiry.
- Merchant/device scoped path.

## 9. BYOS and SSRF Defense

BYOS provider validation must not fetch arbitrary user-provided URLs.

Rules:

- Prefer S3-compatible SDK with explicit endpoint allowlist or strict validation.
- Block localhost, link-local, private IP ranges, metadata service IPs, and internal DNS names.
- Resolve DNS and re-check resolved IP.
- Use request timeout.
- Limit redirects or disable redirects.
- Log only redacted endpoint metadata.

## 10. Update Integration Security

Update trust chain:

- Server publishes metadata.
- Desktop downloads update.
- Desktop verifies checksum.
- Desktop verifies update signature.
- Desktop creates backup before migration.

Rules:

- License signing key and update signing key must be separate.
- Update publish requires release_manager role.
- Critical update cannot delete old local data.
- Rollback must preserve compatibility with currently deployed desktop versions.

## 11. Admin Abuse Defense

Admin dashboard is a high-risk surface.

Required:

- MFA for all admins in production.
- Step-up auth for subscription override, device revoke, update publish, signing key changes.
- Reason required for every mutation.
- Immutable audit log.
- Support access logging.
- Break-glass admin account stored offline with documented process.

## 12. Integration Acceptance Criteria

- License token copied to another PC fails device-bound verification.
- Replayed heartbeat nonce is rejected.
- Cross-tenant object access is rejected.
- BYOS endpoint cannot access internal metadata/private network.
- Backup metadata endpoint rejects raw DB payload.
- Desktop checkout works with server down.
- Desktop renewal requires valid signed token refresh.
- Admin mutation creates audit log.
- Update with invalid signature is rejected.

