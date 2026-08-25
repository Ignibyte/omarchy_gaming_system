---
title: TICKET-019-first-party-remote-provider-migration-pilot
status: open
ticket_number: 019
type: migration
created: 2026-08-24
closed:
intake:
pipeline_spec:
---

# TICKET-019-first-party-remote-provider-migration-pilot

## Summary

Pilot one operator-owned remote game provider, move its durable gameplay
authority without dual snapshots, and propose the exact Constitution §10
amendment required before the provider can serve production sessions.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | Before remote authority is enabled, the pipeline shall approve a Constitution §10 amendment distinguishing platform authority from registered scoped gameplay authority and preserving every identity, credential, broker, audit, and recovery invariant in ADR-0002. | Reviewed ADR/amendment and gate |
| REQ-002 | When a session is created or explicitly migrated to the pilot provider, exactly one durable system shall own gameplay state and revision; OmarchyGS shall store the platform envelope, pinned identities, and authenticated receipts without a writable shadow snapshot. | Migration and invariant tests |
| REQ-003 | When commands time out, conflict, replay, or race, the platform and provider shall converge through stable idempotency keys, expected revisions, authenticated receipts, and reconciliation without timestamp winner selection. | Fault-injected distributed tests |
| REQ-004 | When results or achievement claims arrive, OmarchyGS shall authenticate, deduplicate, validate participant/game/version/definition policy, and atomically record only platform-owned projections and sync invalidations. | Result/achievement transaction tests |
| REQ-005 | When the provider is unavailable, suspended, rolled back, restored, or permanently retired, players and operators shall receive explicit recoverable states and a tested disaster-recovery path without exposing private provider or platform data. | Outage/restore/runbook drill |

## Scope

- In: one first-party provider, authority migration, platform envelope and
  receipt persistence, reconciliation, results/achievements, outage/restore,
  observability, runbooks, constitution amendment, and docs.
- Out: public third-party onboarding, marketplace, multi-provider federation,
  arbitrary provider UI, direct client-provider traffic, and Git delivery.

## Links

- Depends on: `TICKET-018` and accepted cartridge/SDK contracts.
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md)
