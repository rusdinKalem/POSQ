# BACKUP KEY RECOVERY

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan kebijakan kunci enkripsi backup, recovery key, restore, dan batas tanggung jawab vendor.

## 1. Core Decision

MVP menggunakan user-held recovery key.

Artinya:

- Backup cloud dienkripsi sebelum keluar dari perangkat.
- Vendor/control plane tidak dapat membaca isi backup.
- User bertanggung jawab menyimpan recovery key.
- Jika recovery key hilang, backup terenkripsi tidak dapat direstore.

Enterprise escrow dapat dipertimbangkan nanti dengan consent eksplisit dan ADR baru.

## 2. Backup Encryption Principles

Backup cloud rules:

- Encrypt before upload.
- Use authenticated encryption.
- Verify checksum before restore.
- Do not upload plaintext DB dump.
- Server stores metadata only.
- Backup key must never appear in logs.
- Wrong key must fail safely before altering current DB.

## 3. Key Types

| Key | Purpose | Stored Where |
|---|---|---|
| Data encryption key | Encrypt actual backup payload | Generated per backup or key version |
| Recovery key | Allows restoring backup | Shown/saved to owner, not server by default |
| Local wrapping key | Protect local backup credentials/metadata | OS secure storage where available |
| BYOS credential | Access user's object storage | OS secure storage, never logs |

## 4. MVP Key Policy

Recommended MVP policy:

1. On enabling encrypted backup, app generates recovery key.
2. App shows recovery key once with strong warning.
3. User must confirm they saved it.
4. App stores only protected local reference if needed.
5. Control plane stores no recovery key.
6. Restore requires recovery key.

Warning text:

```text
Simpan recovery key ini dengan aman. Tanpa recovery key, backup terenkripsi tidak dapat dipulihkan. Tim support tidak dapat membaca atau membuka backup Anda.
```

## 5. Key Setup Flow

```text
Owner opens backup settings
  -> chooses local/cloud backup
  -> app explains encryption policy
  -> app generates recovery key
  -> user confirms saved key
  -> app creates test encrypted backup
  -> app verifies decrypt/manifest locally
  -> backup schedule can be enabled
```

## 6. Restore Flow

```text
User selects backup
  -> app reads manifest
  -> app asks recovery key if encrypted
  -> app validates key by decrypting manifest/sample
  -> app verifies checksum
  -> app creates pre-restore backup
  -> app restores DB
  -> app runs compatibility check
  -> app records audit log
```

Failure behavior:

| Failure | Required Behavior |
|---|---|
| Wrong key | Stop restore; current DB unchanged |
| Checksum mismatch | Stop restore; current DB unchanged |
| Incompatible schema | Stop restore or require migration path |
| Disk full | Stop restore; current DB unchanged |
| Pre-restore backup failed | Stop restore |

## 7. Managed Cloud Backup

Managed cloud backup means vendor manages storage location, not encryption access.

Allowed server metadata:

- backup_id
- merchant_id
- device_id
- destination type
- object key/logical storage ref
- size
- checksum
- encrypted flag
- encryption algorithm
- created_at
- app version
- DB schema version
- restore compatibility version

Not allowed:

- plaintext backup content
- recovery key
- database rows
- customer/order/payment data extracted from backup

## 8. BYOS Backup

BYOS supported for Pro and above.

Supported target:

- S3-compatible storage.

Provider examples:

- Cloudflare R2.
- Backblaze B2.
- Amazon S3.
- Google Cloud Storage through compatible adapter if implemented.
- MinIO/S3-compatible private storage.

BYOS setup must include:

- Endpoint.
- Bucket.
- Region if needed.
- Access key.
- Secret key.
- Path prefix.
- Connection test.
- Write test.
- Read/delete test for test object.

Secrets:

- Store in OS secure storage where possible.
- Redact in logs.
- Never send BYOS secret to control plane unless explicitly using managed credential proxy, which is not MVP.

## 9. Key Rotation

MVP:

- Manual key rotation can be deferred.
- New backups can use new key version.
- Old backups require old recovery key.

Future:

- Add key version metadata.
- Add re-encrypt backup job.
- Add escrow option for Business/Enterprise.

## 10. Enterprise Escrow Future

Enterprise escrow is not MVP.

If added later, must have:

- New ADR.
- Explicit customer consent.
- Separate escrow key management.
- Strong admin approval workflow.
- Audit log.
- Break-glass procedure.
- Legal/compliance review.

## 11. UX Requirements

Backup screen must show:

- Last backup time.
- Destination.
- Encryption status.
- Recovery key status: saved/unknown, not the key itself.
- Latest backup result.
- Restore test recommendation.

Restore screen must show:

- Backup date.
- Device source.
- App version.
- Schema version.
- Size.
- Checksum status.
- Warning that current DB will be backed up first.

## 12. Acceptance Criteria

- Cloud backup payload is encrypted.
- Backup metadata does not include recovery key.
- Restore with wrong key fails safely.
- Restore creates pre-restore backup.
- Diagnostic logs redact keys and BYOS credentials.
- User must confirm recovery key saved before enabling scheduled encrypted backup.
- Managed cloud backup does not give vendor plaintext access.
- BYOS credential connection test works before schedule enabled.
