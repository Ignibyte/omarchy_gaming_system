---
title: TICKET-042-packaged-reviewed-server-module-release-upgrade-and-rollback
status: closed
ticket_number: 042
type: feature
created: 2026-08-29
closed: 2026-08-29
intake:
pipeline_spec: docs/planning/pipeline/completed/packaged-reviewed-server-module-release-upgrade-and-rollback.spec.md
---

# TICKET-042-packaged-reviewed-server-module-release-upgrade-and-rollback

## Summary

Add a second exact packaged reviewed release for the production observation
module and database-local, revision-checked upgrade and one-step rollback
operations that preserve immutable release, state, admission, delivery, and
audit evidence without auto-upgrading an existing server.

## Why

Tickets 040 and 041 prove the reviewed production base and operator-custom
lifecycle, but the reviewed first-party path still has only one fixed release.
The next roadmap gap is proving a real compatible reviewed release transition
and recovery path before any separately gated marketplace module onboarding.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the reviewed module is configured on a new or existing server, the system shall register a bounded immutable packaged release catalog while preserving release 1.0.0 as the initial selection and never auto-upgrading an existing instance. | Startup, registration replay/conflict, and restart PostgreSQL tests. |
| REQ-002 | When a database-local administrator upgrades the reviewed module to the exact packaged compatible successor, the system shall require expected lifecycle/configuration/state revisions, an explicit bounded candidate state, contained readiness, and one atomic admission/state/release transition with an idempotent receipt and immutable audit. | Real CLI plus PostgreSQL upgrade, replay, migration, and crash-boundary tests. |
| REQ-003 | When an administrator rolls back the upgraded reviewed module, the system shall restore only the retained immediate predecessor and namespace snapshot once, publish a fresh exact admission, terminalize stale work visibly, and reject arbitrary or repeated downgrade graphs. | Rollback, stale-admission, state restoration, and concurrent command tests. |
| REQ-004 | When a target release, migration, readiness result, package artifact, WIT/schema, capability, or expected revision is missing, changed, incompatible, or stale, the system shall fail without mutating the live release, namespace, admission, or lifecycle. | Hostile package/contract corpus and PostgreSQL atomicity/race tests. |
| REQ-005 | When reviewed and operator-custom modules coexist, the system shall apply the same WIT, capability, sandbox, dispatcher, receipt, restore, and effect-reauthorization rules while retaining distinct provenance and player-warning behavior and adding no public administration or executable-delivery route. | Shared conformance matrix, discovery/QML regression, and route/source inventory. |
| REQ-006 | When a restart, package downgrade, or database restore cannot supply the exact active reviewed release, the system shall execute no substitute, preserve core availability with bounded gap evidence, and require exact package/readiness review before recovery. | Restart, absent-release, backup/restore, recovery, and fail-open availability drills. |
| REQ-007 | When the slice completes, the roadmap, architecture, operator guidance, and generated engineering map shall describe the reviewed release lifecycle and all focused, security, CodeGraph, OpenWiki, and local diff-gate evidence shall pass. | Documentation audit and canonical local workflow evidence. |

## Scope

- In:
  - a bounded two-release packaged reviewed catalog for the existing Sentinel
    module identity;
  - database-local expected-revision upgrade and one-step rollback with
    explicit candidate state, readiness, audit, replay, and recovery;
  - exact packaged-release lookup in the shared host/dispatcher path;
  - restart, restore, package-mismatch, stale-work, custom-coexistence, and
    operator documentation evidence.
- Out:
  - marketplace-vetted module import/onboarding, remote administration,
    additional hooks/capabilities/intents, egress, admission hooks, native or
    client executable delivery, game-provider authority, and federation.

## Links

- Intake:
- Pipeline spec: [packaged-reviewed-server-module-release-upgrade-and-rollback.spec.md](../../pipeline/completed/packaged-reviewed-server-module-release-upgrade-and-rollback.spec.md)
- Architecture: [ADR-0004](../../../architecture/adr-0004-process-isolated-wasm-server-modules.md), [Server modules](../../../architecture/server-modules.md)
