# SERVER BLUEPRINT - CONTROL PLANE

Project: Aplikasi POS SaaS Indonesia - Tauri Local Online  
Purpose: Blueprint aplikasi server yang matching dengan aplikasi POS local-first.

## 1. Kesimpulan Arsitektur

Aplikasi server wajib dibuat, tetapi bukan sebagai database transaksi toko. Server adalah SaaS control plane.

Server bertanggung jawab atas:

- Account dan credential merchant owner.
- Admin internal dan support.
- Device activation dan revoke.
- Subscription dan billing status.
- Signed license token.
- Entitlement per plan.
- App update metadata dan signed release.
- Backup metadata.
- Admin dashboard.
- Audit admin.
- Observability metadata.
- Worker untuk renewal reminder, cleanup, dan scheduled jobs.

Server tidak bertanggung jawab atas penyimpanan default:

- Order operasional toko.
- Payment transaksi toko.
- Inventory movement toko.
- Stock live toko.
- Customer purchase history.
- Plaintext backup.

## 2. Server Components

```text
services/
  control-plane-api/
  control-plane-worker/
  admin-dashboard/
migrations/
  server/
```

Recommended implementation:

- API runtime: TypeScript/Node.js or Rust. TypeScript is acceptable for faster SaaS delivery.
- Admin dashboard: SvelteKit or Svelte.
- Server database: PostgreSQL.
- Cache/rate limit: Redis optional for production.
- Object storage: Cloudflare R2, Backblaze B2, Amazon S3, or S3-compatible storage for encrypted backup payload if managed cloud backup is enabled.
- Queue/worker: PostgreSQL-backed job table for MVP; durable queue later if volume requires.

## 3. Server Boundary

Allowed server tables:

- merchants
- merchant_users
- admin_users
- devices
- subscriptions
- subscription_events
- plans
- entitlements
- device_licenses
- license_signing_keys
- app_versions
- backup_metadata
- backup_retention_policies
- idempotency_keys
- admin_audit_logs
- support_access_logs
- webhook_events
- job_queue

Forbidden default server tables:

- orders
- order_items
- payments
- stock_movements
- inventory_items
- local_customers with purchase history
- receipt_items
- local_shift_transactions

If future enterprise cloud operational sync is needed, create a new ADR and a separate opt-in module.

## 4. Matching Contract With Local App

Local app expects server to provide:

- `/api/v1/auth/login`
- `/api/v1/devices/activate`
- `/api/v1/devices/heartbeat`
- `/api/v1/licenses/refresh`
- `/api/v1/updates/check`
- `/api/v1/backups/metadata`
- `/api/v1/support/diagnostics`

Server expects local app to:

- Validate signed license token locally.
- Continue checkout without server.
- Never send raw operational DB by default.
- Encrypt backup before cloud upload.
- Send only redacted health/diagnostic metadata unless user explicitly exports data.

## 5. MVP Server Scope

MVP server must include:

- Auth.
- Merchant account.
- Device activation.
- Subscription manual billing.
- License token issue and refresh.
- Plan entitlements.
- App update metadata.
- Backup metadata.
- Admin dashboard basic.
- Admin audit.
- Idempotency.
- Tenant scoping.
- Device-bound license.
- BYOS SSRF defense.
- Admin MFA and step-up for high-risk actions.
- Separate license signing and update signing keys.

MVP server does not include:

- Payment gateway webhook.
- Marketplace integration.
- Cloud operational sync.
- Full CRM cloud database.
- Real-time sales dashboard from merchant transactions.

## 6. Acceptance Criteria

- Desktop can activate device and receive signed license token.
- Desktop can refresh license after renewal without reinstall.
- Desktop can check update metadata.
- Desktop can upload backup metadata idempotently.
- Admin can manage merchant, subscription, device, update metadata, and backup metadata.
- Server schema guardrail blocks operational transaction tables.
- Admin dashboard never displays order/payment/inventory data by default.
- Private license signing key exists only on server.
- All admin mutations produce audit logs.
- License token copied to another PC fails.
- Cross-tenant object access is rejected.
- BYOS validation cannot access internal/private network resources.
