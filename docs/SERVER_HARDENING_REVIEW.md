# SERVER HARDENING REVIEW

Project: Aplikasi POS SaaS Indonesia - Server Control Plane  
Purpose: Hasil review ulang server blueprint dan rekomendasi hardening.

## 1. Review Summary

Blueprint server sudah benar secara arah: server adalah control plane, bukan database transaksi. Perbaikan yang diperlukan adalah memperkuat boundary antara local dan server supaya subscription/license tidak mudah dibypass, tenant isolation tidak bocor, dan admin dashboard tidak menjadi jalur kebocoran data.

## 2. Findings and Fixes

| ID | Severity | Finding | Fix |
|---|---|---|---|
| SR-F001 | Critical | License token bisa disalin ke device lain jika hanya signed token tanpa device proof | Tambahkan device-bound license dengan install_id, device public key, challenge-response, dan token claims yang diverifikasi desktop |
| SR-F002 | Critical | Server endpoint rawan BOLA jika merchant_id dari body dipercaya | Tenant scope harus derived dari auth context/device binding; setiap object access dicek merchant_id |
| SR-F003 | High | BYOS object storage dapat menjadi jalur SSRF jika endpoint bebas | Tambahkan endpoint allowlist/validation, block private IP/link-local/metadata IP, disable redirect, timeout |
| SR-F004 | High | Key rotation belum cukup operasional | Tambahkan lifecycle active/rotating/retired/compromised dan overlapping validation window |
| SR-F005 | High | Admin dashboard berisiko abuse tanpa step-up | Wajib MFA, step-up untuk aksi sensitif, reason, audit, support access log |
| SR-F006 | High | Update signing dan license signing dapat bercampur | Pisahkan license signing key dan update signing key |
| SR-F007 | Medium | Idempotency perlu tahan race condition | Gunakan DB uniqueness + transaction lock + request hash |
| SR-F008 | Medium | Backup metadata endpoint perlu request-size hard limit | Reject large payload and enforce metadata schema only |
| SR-F009 | Medium | Diagnostic/support metadata dapat melebar | Tambahkan allowlist field dan redaction tests |
| SR-F010 | Medium | Desktop-server compatibility dapat pecah saat API berubah | Tambahkan API versioning dan backward compatibility test |

## 3. Hardening Decisions Applied

- Add `SERVER_LOCAL_INTEGRATION_SECURITY.md`.
- Add ADR-0011 for device-bound license and zero-trust integration.
- Update server data model with device key fields, nonce replay table, signing key lifecycle, and API clients.
- Update server API workflow with challenge-response activation and signed heartbeat.
- Update server test plan with device-bound, BOLA, BYOS SSRF, key rotation, admin step-up, and update signing separation tests.

## 4. Remaining Human Decisions

| Decision | Recommendation |
|---|---|
| Device key storage target | Use OS secure storage where available; fallback encrypted app storage with warning |
| Token lease duration | 7 days MVP; shorten for high-risk business plans |
| Admin MFA method | TOTP MVP; WebAuthn/passkey later |
| KMS provider | Use managed cloud KMS in production; encrypted secret only for staging/dev |
| BYOS scope | Pro/Business only due to support and SSRF risk |
| Break-glass account | One offline-controlled super admin process |

## 5. Final Recommendation

Implementation may proceed only if the server track includes:

- Device-bound license.
- Object-level tenant authorization.
- Admin MFA and step-up.
- Separate license/update signing keys.
- BYOS SSRF defense.
- Server schema guardrail.
- Backup metadata payload guardrail.
- Integration tests proving local checkout survives server outage.

