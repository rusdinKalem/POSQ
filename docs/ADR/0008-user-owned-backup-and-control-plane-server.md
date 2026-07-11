# ADR-0008: User-Owned Backup and Control Plane Server

Status: Accepted  
Date: 2026-07-05

## Context

The POS is a local-first desktop application. Merchant operational data such as orders, payments, inventory movements, customers, reports, and audit logs must remain usable without internet access.

The SaaS business still needs online systems for account management, credential recovery, subscription renewal, license validation, update distribution, device management, and support visibility. The product also needs backup beyond the local device, but not by turning the vendor server into the default central database for every merchant.

## Decision

Use a server-side control plane, not a default operational data plane.

The server stores:

- Merchant account profile.
- Owner/admin credentials and security state.
- Device registrations and heartbeat metadata.
- License token state and entitlement.
- Subscription plan, billing status, and renewal state.
- App version and update metadata.
- Backup job metadata.
- Admin/support audit logs.

The server does not store by default:

- Full order records.
- Payment history.
- Inventory movement history.
- Customer purchase history.
- Local report results.
- Plaintext backup contents.

Backup is user-owned and destination-based:

- Local backup is available to all users.
- Managed cloud backup can be offered by the vendor for Growth plan and above.
- Bring Your Own Storage (BYOS) can be offered for Pro plan and above.
- Backup payloads must be encrypted before leaving the device.
- The server may store backup metadata, but not readable backup contents.

## Recommended Backup Destinations

| Destination | Role | Recommendation |
|---|---|---|
| Local folder | Default | Required for all plans |
| External drive/NAS folder | Offline-friendly backup | Recommended for stores with weak internet |
| Cloudflare R2 | Managed/BYOS object storage | Good S3-compatible candidate |
| Backblaze B2 | Managed/BYOS object storage | Good cost-focused candidate |
| Amazon S3 | BYOS/business | Mature enterprise option |
| Google Cloud Storage | BYOS/business | Good for Google Cloud users |
| MinIO/S3-compatible | Enterprise/private | Useful for private or self-hosted deployment |

## Subscription Entitlement Recommendation

| Plan | Recommended Monthly Price | Backup | Server Dashboard Scope | Recommended Target |
|---|---|---|---|---|
| Starter | Rp99.000-Rp149.000 | Manual local backup | Account, device, license, subscription | Mikro and first-time POS users |
| Growth | Rp199.000-Rp299.000 | Local + managed cloud backup | Starter scope + backup metadata | Small store/cafe that needs safety without setup complexity |
| Pro | Rp399.000-Rp699.000 | Managed cloud + BYOS | Growth scope + advanced entitlement controls | Serious retail/F&B with more devices |
| Business | Rp999.000-Rp2.500.000 | Configurable retention + multi-destination | Pro scope + centralized device/outlet control | Multi-outlet operators |
| Enterprise | Custom annual contract | Custom backup/storage policy | Custom SLA, private deployment option | Chains and brands |

## Admin Dashboard Scope

The dashboard is for SaaS operations, not merchant transaction reporting.

MVP dashboard modules:

- Merchant list and profile.
- Owner/admin credential reset.
- Device activation, rename, revoke, and heartbeat status.
- Subscription plan, expiry, renewal, and payment reference.
- License issuance, refresh, grace, and expired state.
- App update channel, current version, minimum version, and critical update policy.
- Backup metadata: latest backup time, destination type, size, checksum, status, retention state, and failure reason.
- Admin audit log.

Dashboard boundaries:

- Do not show merchant order, payment, inventory, customer, or report data by default.
- Do not allow support staff to download plaintext backups.
- Any operational data support workflow must require explicit merchant consent or merchant-provided export.
- Device revoke must prevent future license refresh but must not delete local data.
- Subscription expiry must not remove access to historical data, export, or local backup.

## Rationale

This decision protects local-first reliability while still allowing SaaS monetization and operational control. It also avoids turning the server into a large sensitive data warehouse too early.

Benefits:

- Lower server cost.
- Lower privacy and breach exposure.
- Clearer merchant data ownership.
- Simpler MVP server schema.
- Better offline reliability.
- Easier backup provider flexibility.

Tradeoffs:

- Cross-device operational sync is not automatic.
- Restore UX and encryption key recovery require careful design.
- Support cannot inspect merchant transactions unless the merchant explicitly exports or grants access.
- Advanced cloud analytics would require a future opt-in data flow.

## Implementation Notes

- Create a `control-plane-api` service, not a generic `cloud-sync-api`.
- Keep server migrations separated from local POS migrations.
- Add tests that fail if default server schema includes operational tables such as orders, payments, order_items, stock_movements, or customer transaction history.
- Backup files should include manifest metadata: app version, schema version, device_id, created_at, size, checksum, encryption algorithm, and restore compatibility version.
- Store cloud provider credentials in OS secure storage where possible.
- Use idempotency keys for backup metadata upload.
- Scheduled backup must run in background and must not block checkout.
- Restore must create a pre-restore backup before modifying current local data.

## Consequences

Accepted consequences:

- The MVP focuses on local POS, license/subscription, update, and backup reliability.
- Multi-device real-time operational sync is deferred unless explicitly approved in a new ADR.
- Pricing must clearly explain backup and device entitlements.
- The admin dashboard is valuable for SaaS operations even without merchant transaction data.

## Future ADR Required If

- The product adds cloud operational sync.
- The server starts storing merchant transactions.
- The product adds cloud analytics from operational data.
- The vendor introduces backup key escrow.
- A private deployment or enterprise data plane is added.
