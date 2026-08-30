---
title: Packaged reviewed server-module release upgrade and rollback
pipeline_id: 4f5a60a7-b2ab-4c18-a93e-76b2c047763d
status: Phase 5 — Complete PASS
ticket: TICKET-042
ticket_doc: docs/planning/tickets/closed/TICKET-042-packaged-reviewed-server-module-release-upgrade-and-rollback.md
aar: docs/planning/knowledge/aar/AAR-042-packaged-reviewed-server-module-release-upgrade-and-rollback.md
created: 2026-08-29
---

# Packaged reviewed server-module release upgrade and rollback — spec

## Intent

Close the remaining reviewed server-module lifecycle gap with one exact
packaged successor release and a real administrator-controlled upgrade and
one-step rollback path. Preserve the Ticket 040/041 runtime, authority,
availability, provenance, recovery, and client boundaries while proving that
reviewed compatibility is more than a single fixed fixture.

## Scope

- In:
  - bounded packaged reviewed release catalog and exact lookup;
  - compatible reviewed release upgrade, candidate namespace/state, readiness,
    atomic admission swap, immediate rollback, stale-work disposition, audit,
    idempotency, restart, and restore;
  - shared reviewed/custom conformance and operator documentation.
- Out:
  - marketplace-vetted module import or approval, automatic update, remote
    module administration, broader hook/capability/intent vocabulary, egress,
    admission hooks, client code, gameplay authority, and federation.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the reviewed module is configured on a new or existing server, the system shall register a bounded immutable packaged release catalog while preserving release 1.0.0 as the initial selection and never auto-upgrading an existing instance. | Startup, registration replay/conflict, and restart PostgreSQL tests. |
| REQ-002 | When a database-local administrator upgrades the reviewed module to the exact packaged compatible successor, the system shall require expected lifecycle/configuration/state revisions, an explicit bounded candidate state, contained readiness, and one atomic admission/state/release transition with an idempotent receipt and immutable audit. | Real CLI plus PostgreSQL upgrade, replay, migration, and crash-boundary tests. |
| REQ-003 | When an administrator rolls back the upgraded reviewed module, the system shall restore only the retained immediate predecessor and namespace snapshot once, publish a fresh exact admission, terminalize stale work visibly, and reject arbitrary or repeated downgrade graphs. | Rollback, stale-admission, state restoration, and concurrent command tests. |
| REQ-004 | When a target release, migration, readiness result, package artifact, WIT/schema, capability, or expected revision is missing, changed, incompatible, or stale, the system shall fail without mutating the live release, namespace, admission, or lifecycle. | Hostile package/contract corpus and PostgreSQL atomicity/race tests. |
| REQ-005 | When reviewed and operator-custom modules coexist, the system shall apply the same WIT, capability, sandbox, dispatcher, receipt, restore, and effect-reauthorization rules while retaining distinct provenance and player-warning behavior and adding no public administration or executable-delivery route. | Shared conformance matrix, discovery/QML regression, and route/source inventory. |
| REQ-006 | When a restart, package downgrade, or database restore cannot supply the exact active reviewed release, the system shall execute no substitute, preserve core availability with bounded gap evidence, and require exact package/readiness review before recovery. | Restart, absent-release, backup/restore, recovery, and fail-open availability drills. |
| REQ-007 | When the slice completes, the roadmap, architecture, operator guidance, and generated engineering map shall describe the reviewed release lifecycle and all focused, security, CodeGraph, OpenWiki, and local diff-gate evidence shall pass. | Documentation audit and canonical local workflow evidence. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Ticket 042 adds one exact compatible successor to the packaged first-party reviewed catalog; it does not create a marketplace-vetted module import path. | This closes the explicit reviewed upgrade/rollback roadmap gap without conflating packaged project review with separately governed marketplace onboarding. |
| 2 | Release 1.0.0 remains the initial selection for a new instance, and package startup may register candidates but never changes an existing selected release automatically. | Installing or restarting a server package must not silently change executable behavior or state. |
| 3 | The successor keeps the exact WIT major, hook, capability, resource, sandbox, and typed-effect authority while using a distinct immutable release/component identity and explicit compatible state schema transition. | The slice proves lifecycle compatibility without expanding module power. |
| 4 | Upgrade requires a complete bounded candidate namespace supplied by the administrator, contained readiness outside SQL, and a final locked comparison of every lifecycle/configuration/state/release input before atomic publication. | Compatibility and readiness are evidence, not authorization to race a later state. |
| 5 | Rollback consumes only the retained immediate predecessor and its pre-upgrade snapshot once; it cannot become an arbitrary downgrade graph. | Bounded recovery is auditable and avoids unsupported migration paths. |
| 6 | Packaged reviewed releases and database-custodied custom releases use one provenance-neutral dispatcher/host/effect path, but packaged review never triggers the custom-code player warning. | Runtime power remains independent from provenance and support claims. |
| 7 | If the installed package cannot reproduce the exact selected reviewed release, no substitute runs and core availability remains fail-open with bounded operational evidence. | A package downgrade or incomplete restore must not relabel or execute different bytes under an old admission. |
| 8 | Administration remains database-local and private; no HTTP, WebSocket, discovery inventory, QML action, or executable client payload is added. | Release lifecycle does not justify a remotely exposed executable-control surface. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-042-packaged-reviewed-server-module-release-upgrade-and-rollback.md`
- Architecture: `docs/architecture/adr-0004-process-isolated-wasm-server-modules.md`; `docs/architecture/server-modules.md`
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope and decisions recorded |
| 2 Design | Architecture, file manifest, regression plan | CodeGraph design receipt |
| 3 Implement | Code matching the design | focused compile/tests and self-review |
| 3.5 Inspect | Independent findings ledger and fixes | fresh CodeGraph inspection receipt |
| 4 Validate | Tests run and delivery gate green | matching gate receipt |
| 5 Complete | AC audit, OpenWiki, docs, submitted AAR, archive | matching completion receipt and no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt and explicit delivery authorization |
