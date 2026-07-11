# ADR-0005: App Update and Local Database Migration Safety

Status: Accepted  
Date: 2026-07-05

## Context

Desktop POS updates can introduce schema changes. Because operational data is stored in PostgreSQL local, failed migration can damage merchant operations. Updates must be secure, controlled, and recoverable.

## Decision

Use signed app releases and mandatory backup before local database migration.

Update workflow:

1. App checks version endpoint when online.
2. Server returns latest version, minimum supported version, release channel, notes, and signature metadata.
3. App verifies update signature.
4. App downloads update if user accepts or if critical update is required.
5. Before migration, app creates local backup.
6. App runs migration.
7. App records migration result.
8. If migration fails, app stops upgrade path and shows recovery instructions.

## Rationale

Signed releases reduce supply chain risk. Backup-before-migration protects merchant data. Minimum supported version lets the server block dangerously outdated versions after a safe grace period.

## Alternatives Considered

| Alternative | Reason Rejected |
|---|---|
| Silent auto-update without backup | Too risky for POS data |
| Manual update only | Slow security patch adoption |
| No signed update | Update spoofing risk |
| Migration without log | Hard to diagnose failures |

## Implementation Notes

Update metadata should include:

- version
- channel
- minimum_supported_version
- critical flag
- release notes
- download URL
- signature
- migration_required flag

Migration log should include:

- migration_id
- from_version
- to_version
- started_at
- completed_at
- status
- backup_path
- error_message

## Test Implications

- Update available.
- Critical update available.
- Invalid signature rejected.
- Backup created before migration.
- Migration success.
- Migration failure preserves backup and gives clear error.

