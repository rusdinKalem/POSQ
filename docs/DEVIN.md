# DEVIN

Purpose: Instructions for Devin or another long-running autonomous software engineering agent.

## 1. Mission

Implement the POS blueprint incrementally, with strict preservation of local-first architecture and data ownership boundaries.

## 2. Start Procedure

Before creating code or PRs:

1. Read `AGENT_HANDOFF.md`.
2. Read `AGENTS.md`.
3. Read `IMPLEMENTATION_PLAN.md`.
4. Read `DECISIONS.md`.
5. Read `STATUS.md`.
6. Read all `ADR/*.md`.
7. Read `TASK_BACKLOG.md`.
8. Select one milestone and propose a small task batch.

## 3. Recommended Devin Task Batches

Batch 1:

- Scaffold Tauri + Svelte + TypeScript.
- Add Rust health command.
- Add PostgreSQL local health check.
- Add first migration.
- Add dummy order write/read.

Batch 2:

- Implement local schema foundation.
- Add migration runner.
- Add backup-before-migration guard.
- Add DB health UI.

Batch 3:

- Implement product, shift, cart, cash checkout, receipt preview.
- Prove offline checkout with server disabled.

Batch 4:

- Implement control plane API skeleton.
- Implement merchant, device, subscription, license token issue.
- Ensure server schema excludes operational transaction tables.

Batch 5:

- Implement backup, restore, encrypted cloud backup, and metadata upload.
- Add idempotency and retry tests.

## 4. Required Status Report

At every checkpoint, report:

```text
Checkpoint:
Task IDs:

Completed:
-

Changed files:
-

Tests:
-

Blocked:
-

Risk:
-

Next:
-
```

## 5. Escalation

Stop and ask for human decision before:

- Changing database architecture.
- Adding cloud operational sync.
- Adding payment gateway.
- Changing expired subscription behavior.
- Changing backup encryption/key recovery.
- Storing operational merchant data on the control plane.
- Implementing multi-terminal outlet server.

