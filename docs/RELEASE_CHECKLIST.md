# RELEASE CHECKLIST

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Menentukan checklist release untuk PoC, internal alpha, pilot merchant, beta, dan production.

## 1. Release Principles

- Do not release if checkout can lose data.
- Do not release if migration can run without backup.
- Do not release if license private signing key is in desktop artifact.
- Do not release if expired mode blocks export/backup/renewal.
- Do not release if backup restore is untested.
- Do not release if installer can delete existing local DB.

## 2. Release Stages

| Stage | Goal | Audience |
| :--- | :--- | :--- |
| PoC | Prove Tauri + Svelte + Rust + PostgreSQL local | Internal |
| Internal Alpha | Core checkout and DB foundation | Internal testers |
| Pilot | Real merchant limited use | 1-3 merchants |
| Beta | Broader merchant testing | Selected merchants |
| Production | Paid public release | Customers |

## 3. PoC Checklist

Required:

- [x] Tauri app opens.
- [x] Svelte UI loads.
- [x] Rust command callable.
- [x] PostgreSQL local connection works.
- [x] Dummy migration runs.
- [x] Dummy order insert/read works.
- [x] Basic health check works.

Exit criteria:

- [x] M1 Technical PoC accepted.

## 4. Internal Alpha Checklist

Required:

- [x] Local schema implemented from DATA_MODEL.
- [x] Product CRUD.
- [x] Shift open/close.
- [x] Cash checkout.
- [x] Order/payment/stock/audit atomic write.
- [x] Local report summary.
- [x] RBAC basic.
- [x] App runs offline for checkout.

Must fail release if:

- [x] Checkout depends on server.
- [x] Stock movement not recorded.
- [x] Refund/void lacks audit.
- [x] Migration cannot be repeated safely.

## 5. Pilot Merchant Checklist

Required:

- [x] Installer configured in tauri.conf.json (NSIS).
- [x] Local PostgreSQL setup tested.
- [x] Backup before migration tested.
- [x] Local backup tested.
- [x] Restore tested.
- [x] License active/grace/restricted expired tested.
- [x] Export tested.
- [x] Receipt preview tested.
- [x] Printer mock or target printer tested.
- [x] Error handling tested.
- [x] Support diagnostic bundle tested.

Pilot constraints:

- [x] Limited merchant count.
- [x] Manual billing acceptable.
- [x] Manual QRIS record acceptable.
- [x] Payment gateway real integration not required.
- [x] Daily backup review required.

## 6. Beta Checklist

Required:

- [ ] Managed cloud backup tested.
- [ ] BYOS backup tested if Pro enabled.
- [ ] Admin dashboard tested.
- [ ] Subscription renewal flow tested.
- [ ] Update signed release tested.
- [ ] Invalid update signature rejected.
- [ ] Performance tested with 5,000 SKU.
- [ ] Report tested with 10,000 transactions.
- [x] Security review completed.

Beta constraints:

- [ ] Known limitations documented.
- [ ] Support process ready.
- [ ] Recovery procedure ready.
- [ ] Rollback plan ready.

## 7. Production Checklist

Required:

- [ ] App installer signed.
- [ ] App update package signed.
- [ ] Control plane TLS configured.
- [ ] Admin MFA enabled.
- [ ] Admin audit log enabled.
- [x] Backup encryption verified.
- [x] Recovery key UX verified.
- [ ] Terms/privacy/support docs ready.
- [ ] Legal/tax review completed for Indonesia-facing claims.
- [ ] Payment provider review completed if integrated.
- [ ] Monitoring/support dashboard ready.

Must fail production release if:

- [ ] Cloud backup uploads plaintext.
- [ ] Server stores operational merchant data by default.
- [ ] Subscription expiry hides/export-locks old data.
- [ ] Private signing key appears in desktop artifact.
- [ ] Critical data-loss bug remains.
- [ ] Migration failure can corrupt DB without backup.

## 8. Required Test Gates

| Gate | Required Before | Status |
| :--- | :--- | :--- |
| TG-01 Technical PoC | Internal Alpha | PASS |
| TG-02 Local DB stable | Checkout MVP | PASS |
| TG-03 Checkout offline works | Pilot | PASS |
| TG-04 Backup metadata idempotent | Cloud backup | PASS |
| TG-05 License behavior proven | Subscription release | PASS |
| TG-06 Migration backup works | Installer release | PASS |
| TG-07 Security audit passes | Pilot | PASS |
| TG-08 Performance acceptable | Beta | PENDING |
| TG-09 P0 docs implemented | MVP foundation | PASS |
| TG-10 P1 docs implemented | Feature implementation | PASS |

## 9. Artifact Checklist

Release artifacts:

- [ ] Installer.
- [ ] App version metadata.
- [ ] Signed update package.
- [x] Local migration files.
- [x] Server migration files.
- [x] `.env.example`.
- [x] User setup guide.
- [x] Admin setup guide.
- [x] Recovery guide.
- [x] Known limitations.
- [x] Test report.

## 10. Rollback Plan

Rollback must define:

- [x] How to stop rollout.
- [x] How to keep current app version running.
- [x] How to restore local DB backup after failed migration.
- [x] How to revoke bad update metadata.
- [x] How to communicate to pilot/beta users.

## 11. Support Readiness

Support must have:

- [x] Error code list.
- [x] Diagnostic bundle guide.
- [x] Backup restore guide.
- [x] License refresh guide.
- [x] Device revoke/reactivation guide.
- [x] PostgreSQL health check guide.
- [x] Printer troubleshooting guide.

## 12. Final Go/No-Go

Go only if:

- [x] P0 tests pass.
- [x] No critical data-loss risk.
- [x] No secret leakage risk.
- [x] Backup and restore proven.
- [x] Restricted Expired Mode proven.
- [x] Installer does not destroy data.
- [x] Support process ready.

No-go if:

- [ ] Any P0 checkout, migration, backup, restore, or license test fails.
- [ ] Any release signing check fails.
- [ ] Any server data boundary guardrail fails.
- [ ] Any private key exposure is detected.
