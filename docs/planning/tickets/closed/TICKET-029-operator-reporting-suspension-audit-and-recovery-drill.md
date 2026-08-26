---
title: TICKET-029-operator-reporting-suspension-audit-and-recovery-drill
status: closed
ticket_number: 029
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/operator-reporting-suspension-audit-and-recovery-drill.spec.md
---

# TICKET-029-operator-reporting-suspension-audit-and-recovery-drill

## Summary

Complete the private-alpha operator-safety loop: let an authenticated persona
submit a bounded report about another persona, let a database-local sysop
review and disposition reports, suspend or reactivate an account with immediate
session containment, retain immutable operator audit evidence, and prove the
platform database can be backed up and restored into an isolated database.

## Why

Private-alpha players can connect, message, challenge, and play, but a server
owner has no general moderation surface or platform recovery drill. The schema
already denies inactive accounts, yet direct SQL status edits do not provide a
safe action contract, session revocation, reason, or durable audit trail.
Inviting external testers before those controls exist would leave both players
and owner-operated servers without a basic response path.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an authenticated owned persona reports another existing persona, the system shall store one bounded category/detail report under an idempotency UUID and return only the reporter-visible receipt fields. | Multi-account API tests for ownership, replay, collision, self-report denial, bounds, and response allowlist. |
| REQ-002 | When a player uses the Social screen to report an exact persona handle, the client shall resolve the public persona, submit the bounded report through the existing Bearer gateway, and expose accessible success/error state without retaining the report text after success. | QML fixture request audit, keyboard/accessibility flow, hostile response cases, and production-root smoke. |
| REQ-003 | When a sysop lists reports through the local administration command, the system shall return a bounded newest-first inventory with reporter/subject public personas, category, detail, status, and timestamps without exposing credentials or password/session material. | CLI/database integration tests and exact JSON schema assertions. |
| REQ-004 | When a sysop suspends an active account with a bounded actor and reason, the system shall lock the account, set it suspended, revoke every live device session in the same transaction, and append immutable audit evidence. | PostgreSQL transaction, concurrent/replay, session-authentication, and append-only audit tests. |
| REQ-005 | When a sysop reactivates a suspended account, the system shall restore login eligibility without restoring previously revoked sessions and shall append immutable audit evidence; disabled accounts shall not be reactivated by this reversible command. | State-transition matrix and new-login/old-token tests. |
| REQ-006 | When a sysop resolves or dismisses an open report, the system shall apply one valid terminal transition and append audit evidence that binds the report, actor, reason, prior state, and resulting state. | Idempotency/collision, invalid-transition, concurrency, and audit linkage tests. |
| REQ-007 | When the administration surface is deployed for private alpha, it shall be a database-local CLI using the configured PostgreSQL authority and shall not add a network admin endpoint, reusable admin token, or player-visible account-owner mapping. | Router inventory, CLI input bounds, secret scan, and architecture review. |
| REQ-008 | When the platform backup/restore drill runs, it shall create a custom-format backup, restore into a newly created isolated database, and prove reports, audit, suspension, revoked sessions, identities, social/inbox, and game history retain their expected counts and security state. | Automated `pg_dump`/`pg_restore` drill plus restored-state queries and authentication denial. |
| REQ-009 | When a sysop follows the operator guide, it shall document report review, suspension/reactivation, report disposition, immutable audit, backup/restore, external MFA-key custody, rollback, and the limits of these private-alpha controls. | Documentation and canonical DIFF/FULL gate review. |

## Scope

- In:
  - persona-targeted report persistence and authenticated API;
  - keyboard-first exact-handle reporting from the Social screen;
  - local sysop CLI for bounded report inventory, disposition, and reversible
    account suspension;
  - same-transaction session revocation and immutable operator audit;
  - platform PostgreSQL backup/isolated-restore drill and operator docs;
  - API/QML/PostgreSQL tests and canonical gate integration.
- Out:
  - network-accessible administration API, administrator accounts/roles, or a
    reusable admin credential;
  - automated moderation, content deletion, message/game takedown, bans,
    appeals, evidence attachments, or report notifications;
  - changing the terminal `disabled` account state through the reversible
    suspension command;
  - provider-database backup, federation, cross-server moderation, legal terms,
    or production backup scheduling/storage/encryption infrastructure.

## Links

- Intake: next unchecked private-alpha roadmap outcome
- Pipeline spec: [completed spec](../../pipeline/completed/operator-reporting-suspension-audit-and-recovery-drill.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Operator boundary: [owner-operated servers](../../../operators/owner-operated-servers.md)
