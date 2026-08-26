---
title: TICKET-032-marketplace-sync-and-server-catalog-control
status: closed
ticket_number: 032
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/marketplace-sync-and-server-catalog-control.spec.md
---

# TICKET-032 — Marketplace synchronization and server catalog control

## Summary

Give an owner-operated OmarchyGS server a production, administrator-controlled
path to synchronize one pinned vetted marketplace, import exact reviewed Game
Cartridge releases, and independently activate, deactivate, or roll back its
server-local catalog with durable provenance and lifecycle enforcement.

## Why

Ticket 031 lets a player recognize and select independent communities. The
next playable dependency is for each community owner to choose a cartridge
library without allowing marketplace publication to become automatic local
admission or weakening the existing signed inert-package boundary.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When marketplace synchronization is enabled, the server-admin path shall require one canonical HTTPS marketplace origin, one exact marketplace Ed25519 public key, one bounded pinned TLS root, and one pre-provisioned secure cartridge-store root, and shall reject invalid or incomplete configuration before network or database mutation. | Configuration unit tests and CLI fixtures |
| REQ-002 | When a marketplace snapshot is received, the system shall accept only a bounded, domain-separated, exact-schema signature from the configured authority with a monotonically increasing snapshot version, unique exact releases, bounded review metadata, and release locations relative to the configured origin. | Contract unit tests with hostile signature, schema, ordering, duplication, downgrade, URL, and size fixtures |
| REQ-003 | When synchronizing an exact release, the system shall use bounded TLS requests without redirects, verify the publisher release, reconstructed conformance report, marketplace lifecycle policy, compatibility, and all pinned digests through the production cartridge verifier before publishing any server inventory state. | Separately spawned TLS marketplace fixture and tamper/timeout/redirect/oversize tests |
| REQ-004 | When verified bytes are imported, the system shall use the existing descriptor-relative secure cartridge store, preserve immutable content-addressed releases, and commit the synchronized database snapshot atomically so a failed synchronization cannot partially publish reviewed inventory. | Secure-store integration and PostgreSQL transaction tests |
| REQ-005 | When an administrator inspects synchronized inventory, the CLI shall return bounded exact release identity, publisher integrity, marketplace authority/review/lifecycle metadata, compatibility, import state, and effective server-admission state without exposing private keys, credentials, filesystem paths, or untrusted rich text. | CLI JSON contract and secret/path-absence tests |
| REQ-006 | When an administrator activates, deactivates, or rolls back a game, the server shall apply an idempotent, concurrency-safe command to one exact imported digest, keep at most one locally active release per game, and append an immutable audit event containing the actor, reason, previous release, and resulting release. | PostgreSQL replay, collision, race, transition, and audit-immutability tests |
| REQ-007 | When local activation is requested, the server shall deny any missing, incompatible, unimported, mismatched, suspended, revoked, or retired release; a deprecated reviewed release may be activated only with its warning preserved. | Domain and database lifecycle matrix tests |
| REQ-008 | When a newer marketplace lifecycle policy is synchronized, effective catalog visibility shall immediately honor its status without deleting immutable imported bytes or silently substituting a release, and older snapshot or policy versions shall never reopen denied content. | Monotonic-policy, restart, omission, and rollback regression tests |
| REQ-009 | When an authenticated player lists the selected server's cartridge catalog, the API shall return only effectively active exact releases with bounded plain-text display/review provenance, compatibility, warning state, marketplace identity, server-admission revision, and content digest, and shall expose no acquisition URL, local path, key material, or executable content. | Axum API authentication, exact JSON, lifecycle-filter, and absence tests |
| REQ-010 | When the database is backed up and restored through the operator drill, synchronized inventory, exact active release, lifecycle state, and immutable audit receipts shall be preserved. | Extended operator backup/isolated-restore drill |
| REQ-011 | When Ticket 032 is delivered, the canonical diff gate shall pass the marketplace TLS/database/CLI/API path together with every existing cartridge, provider, client, recovery, and admission gate. | `bin/gate.sh --diff` receipt |

## Scope

- In: one operator-pinned vetted marketplace per server; a signed monotonic
  snapshot contract; guarded TLS synchronization; publisher and marketplace
  attestation verification; secure exact-release import; PostgreSQL reviewed
  inventory, local admission, lifecycle, and immutable audit state; operator
  inventory and activation/deactivation/rollback commands; a metadata-only
  authenticated player catalog; recovery and operator documentation.
- Out: marketplace publisher/onboarding service or UI; multiple marketplace
  aggregation; client acquisition, cache, mounting, or rendering; cartridge
  launch/session integration; operator-custom signing/import; remote-provider
  onboarding; server modules/hooks; federation; arbitrary URLs or redirects;
  raw QML, JavaScript, native code, or direct cartridge networking.

## Links

- Intake: none
- Pipeline spec: [completed spec](../../pipeline/completed/marketplace-sync-and-server-catalog-control.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
