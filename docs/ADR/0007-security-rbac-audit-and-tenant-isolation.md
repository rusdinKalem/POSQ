# ADR-0007: Security, RBAC, Audit Log, and Tenant Isolation

Status: Accepted  
Date: 2026-07-05

## Context

POS systems handle sensitive operational data such as sales, cash, refunds, stock adjustments, employee activity, and payment records. The application must protect actions locally and in the cloud. Since the product may grow into multi-outlet and multi-tenant usage, tenant isolation must be built into the foundation.

## Decision

Implement security controls from MVP:

- Role-based access control.
- Permission checks in UI and Rust/local service.
- Audit log for sensitive actions.
- Merchant/outlet/device scoping for local and server data.
- Signed license token verification.
- Signed update verification.
- Local DB credential isolation.

## Required Roles

- Owner.
- Manager.
- Cashier.
- Inventory admin.
- Finance.
- Kitchen/waiter for F&B mode.
- Super admin for cloud/admin operations.

## Sensitive Actions

Must create audit log:

- Login/logout.
- Open shift.
- Close shift.
- Checkout.
- Refund.
- Void.
- Discount override.
- Stock adjustment.
- Stock opname.
- Transfer stock.
- Role change.
- License activation.
- App update/migration.

## Rationale

Audit and RBAC are not enterprise luxuries for POS. They prevent cash leakage, stock manipulation, and unauthorized refunds. Adding them late would require rewriting business logic.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| UI-only permission | Can be bypassed from local commands |
| Add audit later | Hard to reconstruct historical events |
| Single admin role | Not suitable for cashier/inventory/finance separation |

## Implementation Notes

- UI should hide unauthorized features.
- Rust/local commands must enforce permissions again.
- Server API must enforce tenant scope.
- Audit logs should be append-only from app perspective.
- Old data must remain exportable after license expiration.

## Test Implications

- Cashier cannot access owner dashboard.
- Cashier cannot refund without permission.
- Inventory adjustment requires reason.
- Every sensitive action creates audit entry.
- Cross-tenant server access is rejected.

