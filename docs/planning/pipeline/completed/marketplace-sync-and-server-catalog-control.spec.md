---
title: Marketplace synchronization and server catalog control
status: Phase 5 — Complete PASS
pipeline_id: 48c86d57-dc5a-40bc-9779-d65b1e635b63
ticket: TICKET-032
ticket_doc: docs/planning/tickets/closed/TICKET-032-marketplace-sync-and-server-catalog-control.md
aar: docs/planning/knowledge/aar/AAR-032-marketplace-sync-and-server-catalog-control.md
created: 2026-08-26
---

# Marketplace synchronization and server catalog control — specification

## Outcome

An owner-operated OmarchyGS administrator can synchronize one cryptographically
pinned vetted marketplace, securely import exact inert cartridge releases, and
independently control the server-local active release for each game. Players
can inspect only the effective catalog metadata; package acquisition remains a
separate ticket.

## Locked decisions

- Marketplace review and server admission remain separate authorities. A
  signed marketplace entry never activates a release by itself.
- Ticket 032 supports one configured marketplace source. Multiple-source
  policy and conflict resolution are deferred rather than implied.
- The source is a canonical HTTPS origin plus a pinned Ed25519 public key and
  an explicit bounded DER TLS root. The transport does not inherit proxy,
  redirect, ambient local-network, or unbounded system-root behavior.
  Snapshot entries use bounded relative paths resolved only beneath that
  origin; redirects and cartridge-supplied destinations are forbidden.
- A marketplace snapshot and every lifecycle policy are monotonic. Newer
  authenticated denial is retained before enforcement; omission does not
  delete imported bytes or silently activate another release.
- Publisher integrity, marketplace review/lifecycle, and server admission are
  stored as distinct facts. Public responses use explicit provenance fields,
  never a generic `verified` flag.
- Exact release bytes enter only through the production verifier and existing
  descriptor-relative `SecureCartridgeStore`.
- Server admission is database-authoritative, idempotent, serialized per game,
  and immutable-audited. Rollback means explicitly selecting an older already
  imported and currently lifecycle-permitted digest.
- At most one release per game is locally active. Marketplace status can make
  that admission ineffective without erasing the administrator's selection.
- The first player API is metadata-only and authenticated. It contains no
  download URL, local path, secret, signing key, provider address, code, or
  render document.
- Client download/cache/mount, gameplay launch integration, marketplace
  publication, multiple sources, and operator-custom trust remain follow-ups.

## EARS acceptance criteria

| ID | EARS requirement | Required evidence |
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

## Failure behavior

- Invalid configuration fails before any network or database work.
- TLS, timeout, redirect, size, signature, schema, digest, compatibility, or
  lifecycle failure returns a stable operator error and publishes no new
  synchronized snapshot.
- A failed release prevents the containing snapshot from becoming current.
  Already imported immutable bytes may remain unreferenced and harmless.
- Snapshot and policy downgrades fail closed.
- Replayed operator commands return their original receipt; reuse of an
  operation ID with different intent fails as a conflict.
- Marketplace denial removes effective visibility immediately but never
  substitutes another release. Recovery requires a newer authenticated policy
  and an explicit permitted administrator activation when appropriate.

## Observable non-goals

- No client downloads or mounts cartridge bytes in this ticket.
- No cartridge becomes playable merely because it appears in the metadata
  catalog.
- No operator-custom content can claim marketplace review.
- No new provider or module execution authority is introduced.
- No remote server can send executable QML, JavaScript, native code, Web
  content, or arbitrary network destinations to the official client.
