---
title: TICKET-030-invite-only-registration-and-private-alpha-readiness
status: closed
ticket_number: 030
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/invite-only-registration-and-private-alpha-readiness.spec.md
---

# TICKET-030-invite-only-registration-and-private-alpha-readiness

## Summary

Replace open account creation with operator-issued, expiring, single-account
registration invitations and carry that contract through the database-local
operator command, versioned API, keyboard-first QML client, private-alpha
admission drill, and operator/tester runbook.

## Why

Every implemented player and operator surface is ready for a small external
alpha, but `POST /v1/accounts` currently admits anyone who can reach the
server. Owner-operated communities need a narrow way to decide who may create
server-local identity without introducing remote administrator accounts,
email delivery, or a general control plane. The invitation secret must also
remain outside PostgreSQL, logs, URLs, and later operator inventories.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a trusted operator issues a registration invitation with a bounded label, lifetime, actor, reason, and operation UUID, the database-local command shall create at most one invitation, return one cryptographically random code only on first delivery, retain only its digest, and append immutable audit evidence. | Operator domain and real-CLI PostgreSQL tests for entropy/shape, exact issuance, operation replay, bounds, digest-only persistence, and audit linkage. |
| REQ-002 | When a trusted operator inventories registration invitations, the command shall return a bounded newest-first metadata view with derived issued, used, expired, or revoked state and shall never return a raw code, code digest, credential, or session material. | Exact JSON inventory tests across all states and source/secret review. |
| REQ-003 | When a trusted operator revokes an unused unexpired invitation, the command shall apply one idempotent transition and append immutable audit; used, expired, absent, and already-revoked invitations shall not become usable. | Transition, replay/collision, concurrency, and append-only audit tests. |
| REQ-004 | When account registration presents a valid unused invitation, the system shall atomically create one canonical Argon2id account and consume that invitation; simultaneous attempts shall create no more than one account. | PostgreSQL API/domain tests for transaction linkage, username conflict rollback, and concurrent consumption. |
| REQ-005 | When registration is retried after the valid invitation already created its account, the system shall return the original public account receipt only when canonical username and password exactly prove the same registration; changed intent shall receive the uniform invalid-invitation response. | API tests for exact replay, wrong username/password, reused code, and response/status allowlists. |
| REQ-006 | When registration receives an absent, malformed, expired, revoked, or already-used invitation, the system shall return one non-enumerating stable error, create no account, and disclose no invitation lifecycle or operator metadata. | Error-precedence, expiry/revocation/use, body-bound, response, logging, and database side-effect tests. |
| REQ-007 | When a player creates an account from the QML access screen, the client shall accept a masked invitation code only in registration mode, clear it and the password on submission or exit, send it only in the JSON body to an admitted server origin, and present accessible allowlisted success/error state. | Keyboard/accessibility fixture, request audit, secret-lifetime assertions, hostile response corpus, and live QML registration. |
| REQ-008 | When the private-alpha admission drill runs, it shall use an isolated migrated database and real operator/server boundaries to prove issue, invited registration, exact retry, one-use denial, revocation, metadata-only inventory, and ordinary sign-in after registration. | Executable end-to-end shell drill integrated into the canonical DIFF/FULL gate. |
| REQ-009 | When an owner prepares an external alpha, the runbook shall define deployment/TLS and secret preconditions, invitation issue/delivery/revocation, clean-client onboarding, MFA recommendation, player-flow checklist, feedback/reporting, backup/restore, incident stop conditions, and explicit unsupported responsibilities. | Operator/tester documentation review and canonical gate. |

## Scope

- In:
  - forward-only registration-invitation schema and audit linkage;
  - database-local issue/list/revoke commands with one-time secret output;
  - invitation-required transactional account registration and exact retry;
  - masked keyboard-first QML invitation entry and stable error mapping;
  - isolated real-boundary private-alpha admission drill and runbook;
  - API, PostgreSQL, CLI, QML, documentation, and gate evidence.
- Out:
  - remote administration API, administrator roles, reusable admin tokens, or
    a hosted control plane;
  - email/SMS delivery, public self-registration, waitlists, referrals,
    multi-use/bulk invitations, or account recovery;
  - distributed edge rate limiting, production TLS automation, telemetry,
    crash collection, support ticketing, or legal/privacy-policy approval;
  - claiming that two external people or clean physical Omarchy machines have
    completed the alpha before that operational event actually occurs.

## Links

- Intake: next unfinished private-alpha roadmap outcome
- Pipeline spec: [completed spec](../../pipeline/completed/invite-only-registration-and-private-alpha-readiness.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Operator boundary: [owner-operated servers](../../../operators/owner-operated-servers.md)
