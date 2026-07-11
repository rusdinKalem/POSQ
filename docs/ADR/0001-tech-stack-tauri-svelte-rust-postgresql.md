# ADR-0001: Tech Stack - Tauri, Svelte, Rust, and PostgreSQL

Status: Accepted  
Date: 2026-07-05

## Context

The POS must run locally in Indonesian retail and F&B environments where internet connectivity can be unstable. The application must provide a fast desktop cashier experience, access local hardware such as printers and barcode scanners, store operational data locally, and still connect online for license validation, subscription renewal, update delivery, backup metadata, and optional encrypted cloud backup.

## Decision

Use the following primary stack:

- Tauri v2 for desktop shell.
- Svelte + TypeScript for frontend UI.
- Rust inside Tauri for local commands, local services, backup worker, license verification, update integration, and hardware bridge.
- PostgreSQL local as primary operational database.
- PostgreSQL server as control plane database.
- Object storage for optional encrypted cloud backup.
- Idempotent background job pattern for backup metadata, device heartbeat, license refresh, and update checks.

## Rationale

Tauri provides a lightweight desktop app with system integration while avoiding the heavier runtime cost of typical browser-wrapped desktop stacks. Svelte provides a fast and simple UI layer suitable for cashier workflows. Rust is appropriate for local services that require reliability and controlled access to filesystem, database, printers, backup, and license logic. PostgreSQL is strong for relational transaction data, stock movement, audit logs, reporting, and multi-terminal outlet growth.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Electron | Heavier runtime and larger installer footprint |
| Web-only app | Cannot guarantee checkout during internet failure |
| SQLite primary local DB | Simpler, but weaker fit for relational multi-terminal/outlet growth and PostgreSQL parity |
| Full cloud-first POS | Server outage or network failure blocks checkout |
| Native Windows-only app | Less portable and slower cross-platform iteration |

## Consequences

Positive:

- Local checkout remains fast and resilient.
- Desktop app can integrate with local hardware.
- Local operational database and server control plane can both use PostgreSQL without sharing the same data ownership model.
- Rust can protect sensitive local operations.

Negative:

- Local PostgreSQL installation and maintenance adds onboarding complexity.
- Migration and backup strategy are mandatory.
- Tauri/Rust/PostgreSQL integration requires stronger engineering discipline.

## Implementation Notes

- Desktop app should target Windows first.
- PostgreSQL local may run per device or as an outlet local server; this must be decided before multi-terminal support.
- Server PostgreSQL must not become the default store for merchant transactions, payments, inventory history, or customer purchase history.
- SQLite may be used only for lightweight cache, not primary POS data.
- All schema changes must be versioned in migrations.

## Test Implications

- Test local DB connectivity.
- Test migration repeatability.
- Test app startup when PostgreSQL is unavailable.
- Test checkout with server offline.
- Test installer behavior with local PostgreSQL requirements.
