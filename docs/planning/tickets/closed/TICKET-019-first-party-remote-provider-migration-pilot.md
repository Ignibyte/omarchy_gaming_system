---
title: TICKET-019-first-party-remote-provider-migration-pilot
status: closed
ticket_number: 019
type: migration
created: 2026-08-24
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/first-party-remote-provider-migration-pilot.spec.md
---

# TICKET-019-first-party-remote-provider-migration-pilot

## Summary

Pilot one operator-owned remote game provider, move its durable gameplay
authority without dual snapshots, and propose the exact Constitution §10
amendment required before the provider can serve production sessions.

## Why

Ticket 018 built and proved the dormant provider trust boundary. Door Legends
already proves the separate-repository cartridge and SDK release workflow, but
it has no deployed rules authority. This pilot connects those two completed
seams with one operator-owned provider before any external onboarding is
possible, while leaving existing compiled Signal Siege sessions unchanged.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | Before remote authority is enabled, the pipeline shall approve a Constitution §10 amendment distinguishing platform authority from registered scoped gameplay authority and preserving every identity, credential, broker, audit, and recovery invariant in ADR-0002. | Reviewed ADR/amendment and gate |
| REQ-002 | When a session is created or explicitly migrated to the pilot provider, exactly one durable system shall own gameplay state and revision; OmarchyGS shall store the platform envelope, pinned identities, and authenticated receipts without a writable shadow snapshot. | Migration and invariant tests |
| REQ-003 | When commands time out, conflict, replay, or race, the platform and provider shall converge through stable idempotency keys, expected revisions, authenticated receipts, and reconciliation without timestamp winner selection. | Fault-injected distributed tests |
| REQ-004 | When results or achievement claims arrive, OmarchyGS shall authenticate, deduplicate, validate participant/game/version/definition policy, and atomically record only platform-owned projections and sync invalidations. | Result/achievement transaction tests |
| REQ-005 | When the provider is unavailable, suspended, rolled back, restored, or permanently retired, players and operators shall receive explicit recoverable states and a tested disaster-recovery path without exposing private provider or platform data. | Outage/restore/runbook drill |
| REQ-006 | When the pilot appears through public catalog, session, result, achievement, or sync APIs, responses shall expose only exact public authority/release/availability and validated projection fields, never provider endpoints, pairwise subjects, credentials, grants, raw signed bodies, or account ownership. | Multi-account API/privacy tests |
| REQ-007 | When pilot conformance runs, it shall build Door Legends from a clean separate-repository clone, launch its TLS service with an independent durable database, exercise the real OmarchyGS broker and API, and prove restart/restore behavior before the canonical gate may pass. | Separate-process authority-pilot gate |

## Scope

- In: Door Legends v1 as one first-party remote-only provider; a separately
  built TLS provider with independent durable state; operator activation;
  platform envelope, authenticated receipt, view-cache, result, and achievement
  projections; command/reconciliation/callback APIs; outage/restore/retirement;
  observability, runbooks, Constitution amendment, and docs.
- Out: public third-party onboarding, marketplace, multi-provider federation,
  arbitrary provider UI, direct client-provider traffic, rewriting or remotely
  migrating existing Signal Siege sessions, main-QML gameplay UI, and Git
  delivery.

## Outcome

Implemented Door Legends v1 as the sole operator-enabled first-party remote
authority pilot. Migration 0015 enforces one durable gameplay owner per
session; the optional server broker exposes authority-tagged catalog and
participant-private start/command/reconcile/read routes; authenticated
callbacks atomically project allowlisted results, achievements, audit, and
sync; and suspension, restoration, retirement, restart, and independent
provider backup/restore are proven. Compiled Signal Siege remains unchanged,
and external providers, direct client networking, executable cartridge UI, and
main-QML gameplay remain out of scope.

## Links

- Depends on: `TICKET-018` and accepted cartridge/SDK contracts.
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md)
- Pipeline: [completed spec](../../pipeline/completed/first-party-remote-provider-migration-pilot.spec.md)
- Operations: [authority pilot runbook](../../../operators/provider-authority-pilot.md)
