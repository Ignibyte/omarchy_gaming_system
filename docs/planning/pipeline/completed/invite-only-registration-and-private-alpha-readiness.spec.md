---
title: Invite-only registration and private-alpha readiness
pipeline_id: 9453a1ce-c7c6-405b-bfa5-25972f28a0be
status: Phase 5 — Complete PASS
ticket: TICKET-030
ticket_doc: docs/planning/tickets/closed/TICKET-030-invite-only-registration-and-private-alpha-readiness.md
aar: docs/planning/knowledge/aar/AAR-030-invite-only-registration-and-private-alpha-readiness.md
created: 2026-08-26
---

# Invite-only registration and private-alpha readiness — spec

## Intent

Give an owner-operated OmarchyGS community a narrow admission boundary for its
first external testers: operators issue expiring one-account invitation codes
locally, players use them through the existing REST/QML onboarding path, and an
executable drill plus runbook proves the workflow without creating a remotely
reachable administrator authority system.

## Scope

- In: all nine Ticket 030 requirements; invitation schema, database-local
  issuance/inventory/revocation, immutable operator audit, invitation-required
  atomic registration and exact retry, QML invitation UX, isolated end-to-end
  admission drill, private-alpha operator/tester runbook, docs, tests, and gate.
- Out: remote admin identity/API, public registration, message delivery,
  bulk/multi-use codes, referrals/waitlists, distributed edge controls,
  production hosting automation, telemetry/support systems, legal approval,
  and the actual external human test event.

## Acceptance criteria (EARS)

The binding requirements are REQ-001 through REQ-009 in
`TICKET-030-invite-only-registration-and-private-alpha-readiness.md`.

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Registration becomes invitation-required at the existing `POST /v1/accounts` route; the request adds an invitation code and no parallel open-registration route remains. | A private-alpha admission boundary is ineffective if an older public path bypasses it. |
| 2 | Operators issue, list, and revoke invitations through the existing PostgreSQL-local CLI; no network administrator role, token, or listener is added. | Ticket 029 established the reviewed private-alpha authority boundary and invitation management fits it. |
| 3 | Each code is a high-entropy bearer usable for exactly one account, expires, and is stored only as a digest; first issue output is the only recoverable raw-code delivery. | Database, backup, inventory, and audit compromise must not disclose usable invitations. |
| 4 | Account creation and invite consumption are one PostgreSQL transaction; a canonical-username conflict rolls both back, while an exact credential-proven retry can recover the original public receipt. | The admission capability must neither be lost on a failed registration nor create two accounts under concurrency or uncertain delivery. |
| 5 | Invitation failures collapse to one stable player-visible response, and client rendering uses only allowlisted local text. | Lifecycle and operator metadata are private and must not become an enumeration surface. |
| 6 | Completion makes the software ready to run an external alpha but does not claim the external two-installation event occurred. | Operational evidence must be recorded only after real people and machines execute it. |

## Linked artifacts

- Ticket: [TICKET-030](../../tickets/closed/TICKET-030-invite-only-registration-and-private-alpha-readiness.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Roadmap: [private alpha](../../ROADMAP.md)
- Prior operator boundary: [Ticket 029 notes](../completed/operator-reporting-suspension-audit-and-recovery-drill.notes.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS complete |
| 2 Design | schema/authority/data flow, API/CLI/QML contracts, file manifest, regression plan | CodeGraph receipt and actionable design |
| 3 Implement | migration, server, CLI, QML, drills, docs, and tests | focused invitation evidence |
| 3.5 Inspect | correctness, auth/privacy, transactions/concurrency, secret handling, UX | resolved ledger and fresh CodeGraph receipt |
| 4 Validate | focused tests and canonical delivery gate | matching gate receipt |
| 5 Complete | EARS audit, OpenWiki, AAR, ticket/archive | no silent drops |
| Delivery | staged review and authorized commit/push | remote commit/tree readback |
