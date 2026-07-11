# GitHub Copilot Coding Agent Instructions

## Project Direction

This repository implements a local-first desktop POS for Indonesia using Tauri v2, Svelte, TypeScript, Rust, local PostgreSQL, and a PostgreSQL-backed control plane server.

Copilot must follow `AGENT_HANDOFF.md`, `AGENTS.md`, `IMPLEMENTATION_PLAN.md`, `DECISIONS.md`, `STATUS.md`, and `ADR/*.md`.

## Hard Constraints

- Do not convert the app into a web-only POS.
- Do not make checkout depend on server availability.
- Do not store full merchant operational data on the server by default.
- Do not store plaintext backup content in the server.
- Do not bundle the license signing private key in the desktop app.
- Do not remove backup-before-migration.
- Do not implement full hard lock for expired subscriptions.

## Expected Pull Request Behavior

Each PR should:

- Reference one or more task IDs from `TASK_BACKLOG.md`.
- Include tests or explain why test coverage is deferred.
- Update `STATUS.md`.
- Update `DECISIONS.md` if a product or architecture choice was made.
- Add or update ADR if architecture changes.
- Preserve existing docs unless intentionally changing them.

## Preferred First PRs

1. Scaffold Tauri + Svelte + TypeScript app.
2. Add Rust health command callable from UI.
3. Add local PostgreSQL connection health check.
4. Add first idempotent local migration.
5. Add dummy order write/read proof.

