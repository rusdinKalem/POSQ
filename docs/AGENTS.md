# AGENTS

This file is the universal instruction layer for coding agents working on this POS project.

## 1. Project Contract

Build the POS as a local-first desktop application:

- Desktop shell: Tauri v2.
- Frontend: Svelte + TypeScript.
- Local backend: Rust/Tauri commands.
- Operational database: PostgreSQL local.
- Server database: PostgreSQL control plane only.
- Server scope: credential, merchant account, device activation, subscription, license, update metadata, backup metadata, admin audit.

Do not turn this into a web-only POS or server-primary POS.

## 2. Non-Negotiable Rules

- Checkout must work when the server is unavailable.
- Operational merchant data stays local by default.
- Cloud backup is optional and encrypted before upload.
- Server must not store plaintext operational backup content.
- Subscription expiry enters Restricted Expired Mode.
- Expired mode blocks new operational writes but allows historical data, export, local backup, restore, renewal, and security update.
- License validation must verify server-signed token locally.
- Private signing key must stay server-side.
- Migration must create a backup before modifying local DB.
- RBAC and audit must enforce sensitive actions in backend, not only UI.

## 3. Required Reading

Read `AGENT_HANDOFF.md` first. Then read:

- `IMPLEMENTATION_PLAN.md`
- `DECISIONS.md`
- `STATUS.md`
- `ADR/*.md`
- `TASK_BACKLOG.md`

For feature work, also read the domain document that owns the feature.

For server/control-plane work, also read:

- `SERVER_BLUEPRINT.md`
- `SERVER_IMPLEMENTATION_PLAN.md`
- `SERVER_DATA_MODEL.md`
- `SERVER_API_WORKFLOWS.md`
- `SERVER_ADMIN_DASHBOARD.md`
- `SERVER_SECURITY_DEPLOYMENT.md`
- `SERVER_MATCHING_MATRIX.md`
- `SERVER_TEST_PLAN.md`
- `SERVER_LOCAL_INTEGRATION_SECURITY.md`
- `SERVER_HARDENING_REVIEW.md`
- `ADR/0010-control-plane-server-blueprint.md`
- `ADR/0011-device-bound-license-and-zero-trust-integration.md`

## 4. Implementation Discipline

- Work in small tasks.
- Keep data model changes explicit.
- Add tests for business rules and failure modes.
- Update `STATUS.md` after each session.
- Update `DECISIONS.md` when choices are made.
- Create a new ADR for architecture changes.
- Do not silently weaken security, backup, or license behavior.

## 5. Preferred Task Flow

1. Pick one task ID from `TASK_BACKLOG.md`.
2. Restate acceptance criteria.
3. Implement the smallest complete vertical slice.
4. Run relevant tests.
5. Update docs.
6. Report result using the template in `AGENT_HANDOFF.md`.
