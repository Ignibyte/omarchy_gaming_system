---
title: Operator reporting, suspension, audit, and recovery drill
pipeline_id: 3515d516-b7b1-475d-bcbc-e44c383d7215
status: Phase 5 — Complete PASS
ticket: TICKET-029
ticket_doc: docs/planning/tickets/closed/TICKET-029-operator-reporting-suspension-audit-and-recovery-drill.md
aar: docs/planning/knowledge/aar/AAR-029-operator-reporting-suspension-audit-and-recovery-drill.md
created: 2026-08-26
---

# Operator reporting, suspension, audit, and recovery drill — spec

## Intent

Give a private-alpha community owner one small, auditable safety loop from a
player's report through local sysop action and disaster-recovery proof, without
creating a remotely exposed administrator authority system.

## Scope

- In: persona reports, Social-screen submission, database-local sysop CLI,
  reversible account suspension/reactivation, immediate session revocation,
  report disposition, immutable audit, platform backup/restore drill, docs,
  tests, and gate.
- Out: remote admin API/auth, roles, permanent bans, content deletion,
  automated moderation, appeals, attachments, provider backup, scheduling,
  federation, and production backup storage/key infrastructure.

## Acceptance criteria (EARS)

The binding requirements are REQ-001 through REQ-009 in
`TICKET-029-operator-reporting-suspension-audit-and-recovery-drill.md`.

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Keep sysop authority in a database-local command, not an HTTP endpoint. | Private alpha needs a safe owner workflow before it needs remote administrator identity, authentication, authorization, and exposure. |
| 2 | Make reports persona-targeted and intentionally small. | Personas are the player-visible identity; content takedown and evidence attachment require separate retention/privacy policy. |
| 3 | Suspension is reversible but revokes every current device session; reactivation never resurrects those tokens. | A moderation action must contain present authority, while later restoration requires fresh proof of credentials and MFA. |
| 4 | Preserve `disabled` as a distinct terminal/manual state outside this command. | A reversible moderation command must not silently weaken a stronger account disposition. |
| 5 | Treat reports and audit as platform data covered by the same PostgreSQL recovery boundary as identity, social, inbox, and games. | A restored community must retain why authority was constrained, not only gameplay state. |

## Linked artifacts

- Ticket: [TICKET-029](../../tickets/closed/TICKET-029-operator-reporting-suspension-audit-and-recovery-drill.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Roadmap: [private alpha](../../ROADMAP.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS complete |
| 2 Design | authority/data flow, migration, API/CLI/QML contracts, recovery plan | CodeGraph receipt and actionable design |
| 3 Implement | server/QML/CLI/migration/docs/tests/gate | focused safety and recovery tests |
| 3.5 Inspect | correctness, authorization, privacy, concurrency, recovery, UX, security | final CodeGraph receipt and finding dispositions |
| 4 Validate | focused tests and canonical delivery gate | matching gate receipt |
| 5 Complete | EARS audit, OpenWiki, AAR, ticket/archive | no silent drops |
| Delivery | staged review, authorized commit/push | remote commit/tree readback |
