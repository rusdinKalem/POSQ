# SERVER TEST PLAN

Project: Aplikasi POS SaaS Indonesia - Server Control Plane  
Purpose: Test plan khusus server agar matching dengan local-first POS.

## 1. Test Gates

| Gate | Must Pass Before |
|---|---|
| STG-01 Server scaffold | Implementing business endpoints |
| STG-02 Server DB guardrail | Device/license/subscription work |
| STG-03 Auth/tenant isolation | Admin dashboard release |
| STG-04 License issuer | Desktop license integration |
| STG-05 Backup metadata idempotency | Cloud backup beta |
| STG-06 Admin audit | Manual billing pilot |
| STG-07 Security review | Production deploy |
| STG-08 Deployment readiness | Pilot merchant |
| STG-09 Device-bound license | Pilot merchant |

## 2. Core Tests

| ID | Scenario | Expected Result | Priority |
|---|---|---|---|
| SRV-T001 | API health | Returns status, version, server_time | P0 |
| SRV-T002 | Worker health | Worker starts and can claim job | P0 |
| SRV-T003 | Admin dashboard loads | Login screen loads | P0 |
| SRV-T004 | Environment validation | Missing critical env blocks boot | P0 |
| SDB-T001 | Fresh migration | Control plane tables created | P0 |
| SDB-T002 | Repeat migration | No duplicate/broken schema | P0 |
| SDB-T003 | Forbidden tables | `orders`, `payments`, `stock_movements` absent | P0 |
| SAUTH-T001 | Owner login | Valid credential returns session | P0 |
| SAUTH-T002 | Cross-tenant access | Access blocked | P0 |
| SLIC-T001 | Device activation | Device created, signed token returned | P0 |
| SLIC-T002 | Device limit | Exceeding plan limit blocked | P0 |
| SLIC-T003 | Expired subscription | restricted_expired token returned | P0 |
| SLIC-T004 | Token signature | Desktop-compatible verification passes | P0 |
| SLIC-T005 | Device-bound token copy | Token copied to another install/device fails verification | P0 |
| SLIC-T006 | Activation challenge replay | Reused challenge rejected | P0 |
| SLIC-T007 | Heartbeat nonce replay | Replayed nonce rejected | P0 |
| SLIC-T008 | Wrong signing algorithm | Token with unexpected alg rejected | P0 |
| SLIC-T009 | Key rotation | Old valid tokens work during overlap; compromised key can be revoked | P0 |
| SSUB-T001 | Manual renewal | paid_until extended, event/audit created | P0 |
| SSUB-T002 | Duplicate renewal event | No double extension | P0 |
| SBAK-T001 | Metadata upload | Metadata stored | P0 |
| SBAK-T002 | Duplicate metadata retry | One logical record | P0 |
| SBAK-T003 | Plaintext payload attempt | Rejected/request too large | P0 |
| SBAK-T004 | BYOS SSRF private IP | Endpoint validation rejects localhost/private/link-local/metadata IP | P0 |
| SBAK-T005 | Managed upload constraints | Pre-signed upload enforces expiry, size, and checksum | P1 |
| SUPD-T001 | Publish update metadata | Version stored with checksum/signature | P0 |
| SUPD-T002 | Update/license key separation | License key cannot sign update metadata and vice versa | P0 |
| SUPD-T003 | Update publish step-up | Release publish requires release_manager and step-up MFA | P0 |
| SADM-T001 | Dashboard forbidden transaction data | No default transaction page exists | P0 |
| SADM-T002 | Admin mutation | Audit log created | P0 |
| SADM-T003 | Admin high-risk step-up | Subscription override/device revoke/update publish requires step-up MFA | P0 |
| SAUTHZ-T001 | BOLA merchant object | User cannot access another merchant's device/subscription/backup metadata by id | P0 |
| SAUTHZ-T002 | Mass assignment | Request cannot set restricted fields not in allowlist | P0 |
| SDEP-T001 | Staging deploy | API, worker, dashboard healthy | P0 |
| SDEP-T002 | Log redaction | Secrets absent from logs | P0 |
| SDEP-T003 | API backward compatibility | Current desktop release can still activate/heartbeat/refresh/check update | P0 |

## 3. Acceptance Criteria

- Server can support desktop activation, heartbeat, renewal, update, and backup metadata.
- Server does not store operational transaction data by default.
- License private key is never exposed.
- Tenant isolation passes.
- Admin audit passes.
- Backup metadata idempotency passes.
- Device-bound license tests pass.
- BYOS SSRF tests pass.
- Admin step-up tests pass.
- Deployment readiness passes before pilot.
