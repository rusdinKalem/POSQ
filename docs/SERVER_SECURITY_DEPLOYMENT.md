# SERVER SECURITY AND DEPLOYMENT

Project: Aplikasi POS SaaS Indonesia - Server Control Plane  
Purpose: Security, deployment, operations, dan release readiness untuk aplikasi server.

## 1. Security Principles

- Server is control plane only.
- Tenant isolation is mandatory.
- Admin access is high risk and must be audited.
- License signing key is a critical secret.
- Backup payload is encrypted client-side.
- Server logs must be redacted.
- Idempotency is required for retryable mutating operations.
- Device license must be bound to activated device identity.
- Object-level tenant authorization is mandatory for every merchant-owned resource.

## 2. Secrets

Secrets:

- Database password.
- JWT/session secret.
- License signing private key or KMS reference.
- Object storage credentials.
- Email provider credentials.
- Payment provider secrets if added later.

Rules:

- Never commit secrets.
- Never log secrets.
- Use secret manager or environment injection.
- Rotate secrets with runbook.
- Desktop app receives public key only.
- License signing key and update signing key must be separate.
- Emergency key compromise procedure must exist before production.

## 3. License Signing Key

Recommended:

- Ed25519 signing.
- Private key stored in KMS or encrypted secret manager.
- Public key distributed to desktop.
- Key id included in token header.
- Support key rotation.
- Track key state: active, rotating, retired, compromised.
- Keep retired public keys available until all valid tokens using them expire.
- Add emergency compromise flow that revokes affected token versions and forces online refresh.

Prohibited:

- Private signing key in desktop app.
- Private signing key in Git repository.
- Private signing key in plaintext database.
- Reusing license signing key as update signing key.

## 3A. Device-Bound License Security

Requirements:

- Desktop creates `install_id` and device key pair during activation.
- Server stores device public key thumbprint.
- Activation includes challenge-response.
- Heartbeat signs request body or nonce with device private key.
- Server rejects heartbeat nonce replay.
- License token includes `merchant_id`, `device_id`, `install_id_hash`, `device_public_key_thumbprint`, `aud`, `iss`, `jti`, `token_version`, `nbf`, and `exp`.
- Desktop verifies expected algorithm explicitly and does not trust the token header alone.

If secure storage is lost:

- Device must require reactivation.
- Local data remains accessible/exportable/backuppable according to runtime mode.

## 4. API Security

Requirements:

- HTTPS only.
- CORS allowlist.
- Security headers.
- Request size limit.
- Input validation.
- Structured error codes.
- No stack traces to client in production.
- Idempotency conflict handling.
- Audit log for admin mutation.
- Rate limit by IP, account, merchant, and device where appropriate.
- Tenant/object authorization for every object id.
- Mass assignment protection through explicit DTO allowlists.
- Request body hard limits for metadata endpoints.
- SSRF protection for BYOS endpoint validation.

High-risk endpoint rules:

- Device activation requires idempotency key.
- License refresh requires verified device binding.
- Admin subscription override requires MFA step-up and reason.
- Update publish requires release_manager role, MFA step-up, checksum, and signature.
- Backup metadata endpoint rejects large/raw DB payloads.

## 5. Database Security

Requirements:

- Separate production database credentials.
- Least-privilege DB user.
- Migration user separated if feasible before production.
- Server DB backup schedule.
- Restore drill.
- Guardrail test for forbidden operational tables.
- Row-level tenant constraints or mandatory repository-level tenant filters.
- Server schema guardrail in CI.
- Migration review for any table that resembles operational transaction storage.

## 6. Deployment Environments

Recommended:

```text
local
staging
production
```

Local:

- Docker compose PostgreSQL.
- Local signing key for development only.

Staging:

- Separate DB.
- Test signing key.
- Test object storage bucket.
- No production merchant data.

Production:

- Managed PostgreSQL.
- Secret manager/KMS.
- Monitoring and alerts.
- Server DB backups.
- Signed update publishing restricted.
- License signing key and update signing key separated.
- BYOS endpoint validation hardened against SSRF.

## 7. Deployment Checklist

- Environment variables validated.
- DB migrations applied.
- Server schema guardrail test passed.
- Admin MFA enabled.
- Step-up MFA enabled for high-risk admin actions.
- License signing key configured.
- Update signing key configured separately.
- Public key available to desktop build.
- HTTPS enabled.
- Logs redacted.
- Backup/restore server DB tested.
- Worker running.
- Monitoring active.
- Admin audit verified.
- Device-bound license tests passed.
- BYOS SSRF defense tests passed.
- API backward compatibility test passed.

## 8. Production Acceptance Criteria

- Server deploy succeeds cleanly.
- Desktop can activate and refresh license.
- Admin dashboard can manage subscription.
- Backup metadata upload works.
- Update metadata check works.
- Tenant isolation tests pass.
- Forbidden operational table guardrail passes.
- Security review has no Critical/High unresolved findings.
- Device-bound license cannot be copied to another PC.
- BYOS validation cannot reach private/link-local/metadata IPs.
- License key rotation runbook tested in staging.
