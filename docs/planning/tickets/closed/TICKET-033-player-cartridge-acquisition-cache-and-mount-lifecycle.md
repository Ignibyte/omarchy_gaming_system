---
title: TICKET-033-player-cartridge-acquisition-cache-and-mount-lifecycle
status: closed
ticket_number: 033
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/player-cartridge-acquisition-cache-and-mount-lifecycle.spec.md
---

# TICKET-033 — Player cartridge acquisition, cache, and mount lifecycle

## Summary

Let an authenticated player acquire the selected server's exact signed Game
Cartridge through the server, verify publisher, marketplace, server-admission,
digest, conformance, and compatibility claims in a trusted native client
companion, and manage a content-addressed local cache with isolated
server-profile mounts from the flagship QML shell.

## Why

Ticket 032 gives every owner-operated server an independently admitted catalog
and securely staged immutable release bytes, but players can see only metadata.
This slice completes the distribution half of the cartridge model without
letting QML presentation data receive credentials, arbitrary network access,
filesystem authority, or executable frontend privilege.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When cartridge distribution is configured for the normal server, the process shall require one pre-provisioned secure store and the exact current marketplace public key, shall advertise the acquisition capability truthfully, and shall fail closed without exposing a partial distribution surface when configuration is absent or invalid. | Server configuration, discovery, and startup tests |
| REQ-002 | When a marketplace snapshot is published, the server shall durably retain the bounded exact signed snapshot and public verification key that produced the current reviewed inventory, without storing private signing material or weakening snapshot monotonicity. | Migration and PostgreSQL synchronization/replay tests |
| REQ-003 | When an authenticated player requests an exact cartridge acquisition, the server shall reauthorize that the requested game and digest are the currently effective local admission and shall return only a bounded, no-store bundle assembled from immutable server-staged bytes and the separate publisher, marketplace, and server-admission claims. | Axum authentication, lifecycle, exact-response, and size tests |
| REQ-004 | When the selected release changes, is omitted, becomes incompatible, or receives a denied lifecycle policy, acquisition of the formerly advertised digest shall fail closed and shall never substitute another release; an in-flight client shall recheck the exact current admission before mounting. | PostgreSQL transition/race and client TOCTOU tests |
| REQ-005 | When the flagship client starts, its launcher shall start one same-user Rust cartridge companion on a random loopback endpoint protected by an unguessable per-process credential, shall keep cache mutation outside QML, and shall stop the companion when the shell exits. | Companion API, launcher cleanup, hostile local-request, and packaged-client smoke tests |
| REQ-006 | When the QML shell asks to acquire a cartridge, the companion shall accept only the selected canonical server origin, stable server UUID, current device bearer, exact game key, digest, and admission revision; remote servers shall require HTTPS and neither the server nor cartridge may redirect or choose a different destination. | Request validation, TLS/redirect, server-identity, and authority-isolation tests |
| REQ-007 | When an acquisition bundle is received, the companion shall require its complete marketplace key to equal an independently provisioned client trust anchor—even when authority and key labels match—then shall verify the signed marketplace snapshot/review entry, publisher release signature, conformance reconstruction, lifecycle policy, exact digest and identity tuple, supported SDK, host compatibility, and selected server's exact admission before publishing a local mount. | Contract hostile corpus, substituted-key rejection, and end-to-end acquisition tests |
| REQ-008 | When verified bytes are installed, the companion shall atomically retain them in a private descriptor-relative content-addressed read-only cache with no executable permission, and identical digests may be reused without duplicating bytes. | Linux filesystem, symlink/race, mode, atomicity, and deduplication tests |
| REQ-009 | When a cartridge is mounted, the companion shall record a bounded server-UUID-scoped binding to the exact digest, client-trusted marketplace-key fingerprint, and admission revision, so restarts and independent server profiles never reuse admission or marketplace provenance authority even when they share immutable content bytes. | Multi-profile, trust-key substitution, and restart tests |
| REQ-010 | When a player refreshes the cartridge library, installs an available release, explicitly updates to a newly admitted release, or removes a local mount, the keyboard-first QML shell shall show bounded loading, ready, update, deprecated, unavailable, and error states and shall preserve the prior good mount on any failed acquisition. | QML fixture and live companion interaction tests |
| REQ-011 | When a player removes a mounted cartridge, the client shall remove only that profile binding and may reclaim exact immutable bytes only when no profile references them; it shall not delete account, persona, social, session, save, achievement, server catalog, or provider state. | Removal/reference-count and domain-absence tests |
| REQ-012 | When the native client package is built, it shall contain the exact reviewed QML payload and native companion/launcher with reproducible provenance, least-privilege modes, no embedded credentials, and a clean extracted-package acquisition smoke path. | Source contract, two-build comparison, payload inspection, and extracted-package smoke |
| REQ-013 | When Ticket 033 is delivered, the canonical diff gate shall pass the acquisition/database/QML/package path together with every existing cartridge, provider, recovery, and admission gate. | `bin/gate.sh --diff` receipt |

## Scope

- In: current signed marketplace snapshot retention; authenticated exact
  server distribution; a same-user Rust client companion; independent
  cryptographic and compatibility verification; private descriptor-relative
  content cache; server-profile mounts; explicit install/update/remove UI;
  native package and operator/player documentation; automated hostile,
  PostgreSQL, QML, launcher, and extracted-package evidence.
- Out: cartridge-supplied executable QML, JavaScript, native code, WebEngine,
  shell commands, credentials, or network clients; direct marketplace or
  provider access from the player client; automatic background updates;
  session-to-cartridge render-plan binding and live cartridge gameplay;
  operator-custom trust; multiple marketplaces; privileged multi-user cache
  service; cross-server identity/federation; deletion of authoritative game or
  account state.

## Links

- Intake: none
- Pipeline spec: [completed spec](../../pipeline/completed/player-cartridge-acquisition-cache-and-mount-lifecycle.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
