# SERVER DATA MODEL

Project: Aplikasi POS SaaS Indonesia - Server Control Plane  
Purpose: Skema database server yang matching dengan local-first POS.

## 1. Ownership Rule

Server PostgreSQL menyimpan data SaaS control plane. Server bukan source of truth untuk transaksi toko.

Allowed:

- Merchant account.
- Credential.
- Device.
- Subscription.
- License.
- Entitlement.
- Update metadata.
- Backup metadata.
- Admin audit.
- Support metadata.
- Worker job state.

Forbidden by default:

- orders
- order_items
- payments
- stock_movements
- inventory_items live stock
- customer purchase history
- plaintext backup payload

## 2. Core Tables

Required server tables:

- `merchants`
- `merchant_users`
- `admin_users`
- `plans`
- `entitlements`
- `subscriptions`
- `subscription_events`
- `devices`
- `device_activation_challenges`
- `device_nonces`
- `license_signing_keys`
- `update_signing_keys`
- `device_licenses`
- `app_versions`
- `backup_metadata`
- `idempotency_keys`
- `admin_audit_logs`
- `support_access_logs`
- `job_queue`

These extend the server section in `DATA_MODEL.md`; if there is conflict, this file and ADR-0010 clarify server-specific behavior.

## 3. Critical Table Rules

### license_signing_keys

- Store `key_id`, algorithm, public key, encrypted private key reference, status.
- Private key material must not be stored plaintext in database.
- Desktop receives public key only.
- Key state must support `active`, `rotating`, `retired`, and `compromised`.
- License signing keys must be separate from update signing keys.

### update_signing_keys

- Store update signing public key and encrypted private key reference.
- Release manager may publish update metadata, but private key must remain in KMS/secret manager.
- Key rotation must keep old public keys available while old signed releases remain valid.

### devices

- Store `device_fingerprint_hash`, `install_id_hash`, and `device_public_key_thumbprint`.
- Store latest app version, OS, last heartbeat, and status.
- Do not store raw fingerprint if a stable hash is enough.

### device_activation_challenges

- Store challenge id, merchant id, challenge hash, expires_at, consumed_at.
- Challenge expires quickly and can be used once.

### device_nonces

- Store recent heartbeat/refresh nonce hashes by device.
- Reject nonce replay within retention window.

### device_licenses

- Store token metadata and optional token hash.
- Do not store raw license token unless explicitly justified.
- Include token version, runtime mode, valid_until, grace_until, and signing key id.
- Include device binding fields in token payload and server metadata.
- Revoked token versions must not be refreshed.

### backup_metadata

- Store backup id, merchant id, device id, destination type, logical storage ref, size, checksum, encryption algorithm, app version, DB schema version, status, and failure code.
- Must not store plaintext DB dump.
- Cloud backup metadata must indicate encrypted=true.

### idempotency_keys

- Unique by `(key, scope)`.
- Same key + same request returns same result.
- Same key + different request returns conflict.
- Implementation must use uniqueness plus transaction locking to avoid concurrent duplicate processing.

## 4. Guardrail Migration Test

Server migration test must fail if these table names exist in default schema:

```text
orders
order_items
payments
stock_movements
inventory_items
receipt_items
customer_purchase_history
```

## 5. Required Indexes

```text
merchant_users(email)
devices(merchant_id, device_fingerprint_hash)
devices(merchant_id, install_id_hash)
devices(device_public_key_thumbprint)
devices(merchant_id, status)
device_activation_challenges(merchant_id, expires_at)
device_nonces(device_id, created_at)
subscriptions(merchant_id, status)
device_licenses(device_id, token_version)
backup_metadata(merchant_id, device_id, created_at)
backup_metadata(backup_id)
app_versions(os, channel, version)
admin_audit_logs(merchant_id, created_at)
idempotency_keys(key, scope)
job_queue(status, run_after)
```

## 6. Acceptance Criteria

- All control plane tables are created by server migrations.
- Operational POS tables are absent by default.
- All tenant-owned rows include `merchant_id` where applicable.
- Mutating endpoints use idempotency where retries are expected.
- Raw license token is not stored; token hash is allowed.
- Private signing key is not plaintext in database.
- Backup metadata never contains plaintext backup payload.
- Device-bound license fields exist and are indexed.
- Update signing keys are separate from license signing keys.
- Nonce replay table exists for heartbeat/refresh protection.
