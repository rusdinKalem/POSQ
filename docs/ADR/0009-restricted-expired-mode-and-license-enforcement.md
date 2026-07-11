# ADR-0009: Restricted Expired Mode and License Enforcement

Status: Accepted  
Date: 2026-07-05

## Context

The POS is installed locally and must keep core business data in the merchant's local PostgreSQL database. The product is sold as a subscription, so the application must enforce paid access. A simple hard lock after subscription expiry would technically enforce payment, but it creates a serious product and trust problem because the merchant may lose access to historical records, reports, export, backup, and renewal.

The desired behavior is to prevent unpaid operational use while preserving the merchant's access to their own data.

## Decision

Use Restricted Expired Mode after the subscription grace period ends.

When subscription is active:

- App runs normally according to plan entitlement.

When subscription is expired but still inside grace period:

- App continues operating according to entitlement.
- UI shows clear warning and renewal call-to-action.
- Heartbeat should attempt renewal/status refresh when online.

When subscription is expired after grace period:

- App enters `restricted_expired` mode.
- New operational actions are blocked.
- Historical data access remains available.
- Export, local backup, restore, update security, and renewal remain available.

## Allowed in Restricted Expired Mode

- Login to local app.
- View old orders.
- View historical payments.
- View inventory records.
- View reports.
- Export reports and operational data.
- Create local backup.
- Restore backup with explicit confirmation.
- Open renewal/payment screen.
- Refresh license after renewal.
- Install security/critical updates.
- Contact support.

## Blocked in Restricted Expired Mode

- New checkout.
- New refund.
- Void transaction.
- Discount override.
- Stock adjustment.
- Stock opname submission.
- Purchase/receiving.
- Stock transfer.
- New F&B table order.
- New kitchen order.
- Premium modules.
- API/webhook operations that create new operational data.

## License Enforcement Model

Use signed, short-lived license tokens.

Token should include:

- merchant_id
- outlet_id
- device_id
- plan
- entitlement flags
- issued_at
- valid_until
- paid_until
- grace_until
- license_status
- app_min_version
- token_version
- signature

Rules:

- Server signs token with private key.
- Desktop app verifies token with bundled public key.
- Private signing key must never be bundled in desktop app.
- Token validity should be short, recommended 3-7 days.
- App refreshes token on heartbeat when online.
- If token expires and no valid refresh exists, app enters restricted/suspicious mode.
- Local clock rollback must be detected using last known server time and last seen local time.
- Local license state must be tamper-evident.

## Rationale

Restricted Expired Mode balances business enforcement and merchant trust.

It prevents unpaid use of the POS as an operating tool, but it does not hold merchant data hostage. This reduces support disputes, protects brand trust, and keeps the product defensible for real businesses.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Full hard lock | Blocks data access, export, backup, and renewal; creates data hostage perception |
| No expiry enforcement | Subscription business model cannot be enforced |
| Online check for every checkout | Breaks local-first reliability |
| Long-lived offline token | User can remain offline for too long after cancellation |

## Implementation Notes

- License manager must run before navigation to operational screens.
- Rust/local service must enforce expired restrictions, not only frontend UI.
- UI should hide or disable blocked actions in restricted mode.
- Backend command handlers must reject blocked actions even if UI is bypassed.
- Every denied action should return a clear error code such as `LICENSE_RESTRICTED_EXPIRED`.
- Renewal flow must be reachable from restricted mode.
- Export and backup commands must remain available in restricted mode.
- Security updates must remain available in restricted mode.
- App should not delete, mutate, or hide local operational data due to subscription expiry.

## Test Implications

- Expired after grace blocks checkout before order/payment write.
- Expired after grace blocks stock adjustment before stock movement write.
- Expired after grace allows old reports.
- Expired after grace allows export.
- Expired after grace allows local backup.
- Expired after grace allows renewal and license refresh.
- Token tampering fails verification.
- Clock rollback is detected.
- Offline beyond token lease eventually prevents operational use.
- Desktop artifact does not contain server private signing key.
