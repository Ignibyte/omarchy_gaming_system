---
title: TICKET-045-provider-starter-conformance-and-second-game
status: closed
ticket_number: 045
type: feature
created: 2026-08-30
closed: 2026-08-30
intake: docs/planning/intake/public-provider-sdk-starter-and-sidecar.md
pipeline_spec: docs/planning/pipeline/completed/provider-starter-conformance-and-second-game.spec.md
---

# TICKET-045-provider-starter-conformance-and-second-game

## Summary

Ship a game-agnostic provider starter, portable conformance and fault kit, and
a second clean-room SDK-dependent game named Relay Forge, all reproducibly
consumable outside the OmarchyGS source tree and exercised through the real
broker without admitting external providers.

## Why

Ticket 044 made the protocol independently packageable, but developers still
have to reconstruct durable operation handling, callback delivery, process
configuration, and fault tests from the game-specific Door Legends pilot. A
public starter and conformance kit must prove that the boundary is reusable
before the project can design a co-located deployment profile or consider
reviewed external providers.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the starter receives launch, command, or reconcile traffic, it shall authenticate and validate the exact SDK contract before invoking game rules, persist expected-revision and whole-operation idempotency state in its own PostgreSQL database, and produce stable retry receipts and authenticated callbacks without platform database access or compiled fallback. | Starter unit/PostgreSQL tests and real broker callback/outage/restart/reconcile exercise. |
| REQ-002 | When a developer supplies game rules, the starter shall keep deterministic game state transitions behind a narrow game trait and keep transport, grant/signature verification, compatibility, persistence, callback delivery, configuration, and process lifecycle game-agnostic. | Rule-only unit tests, two distinct game implementations, dependency review, and substitution tests. |
| REQ-003 | When the portable conformance kit targets a provider, it shall exercise valid compatibility/launch/command/reconcile/callback flow and a bounded published fault corpus covering replay, changed intent, stale revision, timeout/unknown outcome, outage/restart, signature/digest/context mismatch, malformed/oversized input, callback deduplication, and recovery. | Standalone CLI/library runs with bounded machine-readable receipts against the starter and clean-room provider. |
| REQ-004 | When Relay Forge is built from a clean source tree, it shall consume only packaged public Provider SDK/starter/conformance artifacts, own distinct rules, keys, process, and PostgreSQL state, pass the public conformance kit, and integrate through the real OmarchyGS broker with no repository path dependency or private platform hook. | Two clean clones, dependency/path scans, deterministic rule tests, real TLS/broker run, and database separation assertions. |
| REQ-005 | When the expanded public developer kit is exported twice, its sources, templates, migrations, schemas, fixtures, checksums, provenance, and builds shall be byte-identical and verify from fresh repositories without OmarchyGS source, platform credentials, or private keys. | Exact inventories, signed release verification, two-export comparison, packaged dependency trees, and clean-clone builds. |
| REQ-006 | When starter or conformance documentation describes identity and authority, it shall state that providers receive only pairwise game-scoped subjects and scoped grants and never gain account/persona identity, reusable device credentials, platform database access, arbitrary egress, client executable privilege, direct client connectivity, registration, activation, discovery, trust, or publication authority. | Documentation contract assertions, serialized traffic scan, public API diff, and operator-boundary review. |

## Scope

- In:
  - reusable provider starter library/process with separate PostgreSQL state;
  - narrow deterministic game-rules abstraction and callback outbox;
  - standalone public conformance library/CLI and bounded fault corpus;
  - Relay Forge clean-room example and real-broker integration proof;
  - deterministic public artifact export and local gate integration.
- Out:
  - co-located sidecar transport, service supervision, or deployment guide;
  - external/self-service provider registration, review, activation, or support;
  - public package-registry or hosted CI/CD publication;
  - direct client-provider networking or shared platform state/credentials.

## Links

- Intake: [Public Provider SDK](../../intake/public-provider-sdk-starter-and-sidecar.md)
- Pipeline spec: [provider-starter-conformance-and-second-game.spec.md](../../pipeline/completed/provider-starter-conformance-and-second-game.spec.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- AAR: [AAR-045](../../knowledge/aar/AAR-045-provider-starter-conformance-and-second-game.md)
