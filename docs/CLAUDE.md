# CLAUDE

Purpose: Instructions for Claude Code or Claude-powered coding agents.

## 1. Model Usage

Use a high-reasoning model such as Claude Opus for:

- Architecture review.
- License/subscription design.
- Backup encryption and restore safety.
- Local PostgreSQL migration strategy.
- Security model, RBAC, audit, and threat modeling.
- ADR changes.

Use a coding-oriented model such as Claude Sonnet for:

- Tauri/Svelte/Rust scaffolding.
- Feature implementation.
- Tests.
- Refactoring within approved architecture.
- UI flows already specified in `UI_FLOW.md`.

## 2. Mandatory Context

Before coding, read:

- `AGENT_HANDOFF.md`
- `AGENTS.md`
- `IMPLEMENTATION_PLAN.md`
- `DECISIONS.md`
- `STATUS.md`
- `ADR/*.md`
- `TASK_BACKLOG.md`

Claude must treat ADR and DECISIONS as higher priority than informal suggestions in a prompt.

## 3. Claude Prompt Template

```text
You are implementing the POS local-first blueprint.

Read AGENT_HANDOFF.md, AGENTS.md, IMPLEMENTATION_PLAN.md, DECISIONS.md, STATUS.md, ADR/*.md, and TASK_BACKLOG.md.

Select task: <TASK_ID>.

Do not change architecture. Do not store merchant operational data on the server. Do not make checkout depend on the server. Do not hard-lock historical data after subscription expiry.

Implement only this task, run relevant tests, then update STATUS.md and DECISIONS.md if needed.
```

## 4. Review Prompt Template

```text
Review the current implementation against the POS blueprint.

Focus on:
- local-first checkout
- PostgreSQL local correctness
- control-plane-only server boundary
- backup encryption and restore safety
- license token validation
- Restricted Expired Mode
- RBAC and audit
- migration safety

Return findings by severity with file/line references and required fixes.
```

