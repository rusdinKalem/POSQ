# ADR-0006: Domain Boundaries and Modular MVP

Status: Accepted  
Date: 2026-07-05

## Context

The PRD includes many future capabilities: retail, F&B, marketplace, API/webhook, loyalty, payment, accounting, and enterprise workflows. Building everything at once would create scope risk and reduce stability. The MVP must focus on core operational reliability.

## Decision

Use modular domain boundaries and implement MVP in layers.

Core P0 domains:

- Account and onboarding.
- Catalog.
- Checkout.
- Shift.
- Payment record.
- Inventory.
- RBAC.
- Audit log.
- Local reporting.
- Sync.
- License.
- App update/migration safety.

P1 optional domains:

- Customer.
- Loyalty basic.
- F&B mode basic.
- Cloud backup.
- Payment integration.

P2 later domains:

- Marketplace connector.
- Public API/webhook.
- Advanced analytics.
- Enterprise multi-chain support.

## Rationale

Core POS reliability must come before breadth. Checkout, local DB, stock movement, backup/control-plane jobs, and license are the foundation. Optional vertical workflows can be layered after the core order model is stable.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Build all PRD features in one phase | Too much risk and slower learning |
| F&B-only app | Reduces general POS market flexibility |
| Retail-only app | Misses strong cafe/F&B segment |
| Marketplace-first app | Depends on integrations before local POS core is stable |

## Implementation Notes

- F&B and retail modes must not fork the core order model.
- Domain modules should expose clear services and typed interfaces.
- Payment gateway real integration must wait until local payment records are stable.
- Marketplace sync must wait until backup/control-plane background jobs are proven and a separate integration ADR is approved.

## Test Implications

- Generic checkout must pass before F&B/retail mode.
- F&B modifier/table flow must not break retail checkout.
- Retail return flow must not break order/payment audit.
- Optional modules must be feature-flagged.
