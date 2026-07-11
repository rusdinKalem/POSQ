# ADR-0004: License, Subscription, Entitlement, and Grace Period

Status: Accepted  
Date: 2026-07-05

## Context

The product is sold as a subscription, but the POS must continue working during temporary internet failure. The application must validate subscription status online without making checkout depend on server availability. Merchant data must remain accessible even if subscription expires.

The accepted expiry policy is Restricted Expired Mode, not full hard lock. The app must block new unpaid operational use while preserving access to old data, export, backup, restore, security update, and renewal.

## Decision

Use signed device license tokens with entitlement, grace period, and Restricted Expired Mode.

Activation flow:

1. Owner logs in online.
2. Owner selects merchant and outlet.
3. App registers device and receives `device_id`.
4. Server returns signed license token.
5. Local app stores token securely.
6. Local app verifies token signature and entitlement offline.

Token contains:

- merchant_id
- outlet_id
- device_id
- plan
- entitlement flags
- issued_at
- expires_at
- paid_until
- grace_until
- license_status
- app_min_version
- token_version
- signature

## Rationale

Signed tokens allow offline validation while keeping server authority over subscription state. Grace period prevents merchant operations from failing due to temporary connectivity or payment delay. Restricted Expired Mode protects the subscription business model without turning merchant data into hostage data.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Online check for every checkout | Checkout fails when internet fails |
| No license validation | Subscription business model cannot be enforced |
| Hard lock when expired | Merchant loses access to business records, export, backup, and renewal; support burden increases |
| No lock when expired | Subscription cannot be enforced |
| Long-lived offline token | User can continue unpaid operation for too long |

## Expired Mode Policy

After grace period, app enters Restricted Expired Mode.

- Existing data remains viewable.
- Export remains available.
- Local backup remains available.
- Restore remains available with explicit confirmation.
- Security/critical update remains available.
- Renewal screen remains available.
- New checkout is blocked.
- New refund, void, stock adjustment, purchase, transfer, and premium modules are blocked.
- Renewal online restores entitlement.
- No reinstall is required.

Runtime modes:

| Mode | Policy |
|---|---|
| active | All plan-entitled features available |
| grace | Operational features still available with warning |
| restricted_expired | New operational actions blocked; old data/export/backup/renewal allowed |
| revoked | Future license refresh blocked; local data not deleted |
| suspicious_time | Operational actions blocked until online verification |

## Implementation Notes

- License verification runs locally on startup.
- Heartbeat runs when online.
- Entitlement should be mapped to feature flags.
- Device revoke must be handled on next heartbeat.
- Server is source of truth for renewal and plan changes.
- License token should be short-lived, recommended 3-7 days.
- Desktop app must contain only server public key, never private signing key.
- Local license state must store `last_server_time` and detect clock rollback.
- Rust/local command handlers must enforce Restricted Expired Mode, not only frontend UI.
- Denied commands should return explicit error code such as `LICENSE_RESTRICTED_EXPIRED`.

## Test Implications

- Active subscription online.
- Active subscription offline.
- Expired but inside grace period.
- Expired after grace period.
- Expired blocks new checkout.
- Expired blocks stock adjustment.
- Expired allows old data, export, local backup, restore, and renewal.
- Renewal updates local entitlement.
- Token tampering fails verification.
- Local clock rollback is detected.
- Private signing key is absent from desktop artifacts.
