---
title: TICKET-035-historical-session-cartridge-acquisition-and-multi-screen-navigation
status: closed
ticket_number: 035
type: feature
created: 2026-08-26
closed: 2026-08-27
intake:
pipeline_spec: docs/planning/pipeline/completed/historical-session-cartridge-acquisition-and-multi-screen-navigation.spec.md
---

# TICKET-035-historical-session-cartridge-acquisition-and-multi-screen-navigation

## Summary

Allow a participant to install the exact cartridge pinned to a historical game
session after the server catalog selects or omits another release, then navigate
the cartridge's signed multi-screen presentation through trusted host controls.

## Why

Ticket 034 made one mounted entry screen playable but truthfully fails when an
old session's exact mount is absent and cannot leave the entry screen. A portable
cartridge system needs recoverable historical presentation and more than one
reviewed screen without substituting current catalog content or admitting
publisher executable code.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an authenticated marketplace snapshot is committed, the server shall retain bounded immutable acquisition evidence for every exact reviewed release without allowing a later snapshot to rewrite that release's historical identity. | Migration constraints plus PostgreSQL first-seen, replay, later-snapshot, omission, and hostile-evidence tests. |
| REQ-002 | When a participant requests acquisition for a session with a pinned cartridge, the server shall authorize that persona/session, resolve only the pinned release under current active-session lifecycle policy, and never select the current catalog release as a substitute. | PostgreSQL/API tests for exact historical success, foreign persona, null pin, digest mismatch, catalog upgrade/rollback/omission, and lifecycle denial. |
| REQ-003 | When the server returns a historical acquisition, it shall build and self-verify the existing bounded inert acquisition envelope from retained signed marketplace evidence, the configured marketplace key, and the exact secure-store bytes while exposing no path, credential, provider endpoint, or operator-only state. | Contract and API exact-schema/privacy tests plus marketplace-key, snapshot, release, policy, and store-tampering cases. |
| REQ-004 | When catalog or lifecycle state changes concurrently with historical acquisition, the result shall linearize to one exact allowed release or fail closed, and suspension/revocation shall never be bypassed by retained older evidence. | PostgreSQL writer-first/acquisition-first concurrency tests and signed-policy transition cases. |
| REQ-005 | When the companion installs a session-pinned release, it shall authenticate discovery and participant-authorized session state before and after one same-origin bounded request, require the client-controlled marketplace key and exact pin, and atomically create only the matching server-profile mount. | Client-runtime integration and hostile origin, redirect, proxy, session, key, pin, response, race, and restart tests. |
| REQ-006 | When more than one exact release or admission of a game is installed for one server, the cache shall retain them as separate bounded mounts, resolve each session's exact tuple, and remove only the explicitly named tuple without deleting shared authenticated content. | Cache/profile migration-compatibility, coexistence, exact-resolution, exact-removal, capacity, concurrency, and restart tests. |
| REQ-007 | When the selected session has no matching local mount, the QML client shall preserve authoritative session state and present an explicit keyboard-operable exact-release install action with loading, offline, denied, incompatible, and retry states rather than silently downloading or substituting content. | Production-root QML fixtures and live helper/server smoke. |
| REQ-008 | When a cartridge declares screen navigation, the signed contract shall permit only bounded, unambiguous host-navigation actions from reviewed button emitters to existing screen IDs, disjoint from gameplay actions and incapable of carrying a URL or arbitrary payload. | Cartridge verifier/SDK schema/conformance tests with duplicate, missing-target, grid, payload, namespace, cycle, and limit fixtures. |
| REQ-009 | When the companion prepares a requested signed screen, it shall resolve the same exact mounted release, validate that screen's own schema against authoritative session view data, and return only a strict screen-bound render envelope with its reviewed local destinations. | Renderer/runtime tests for entry and secondary screens plus unknown-screen, invalid-view, cross-release, lifecycle, and envelope tampering. |
| REQ-010 | When a trusted navigation control is activated, QML shall navigate only to the companion-authorized destination, maintain a bounded release/session-scoped history, restore deterministic keyboard focus, and provide host Back/Entry controls without issuing a gameplay request. | QML keyboard, focus, accessibility, bounded-history, reset, malicious-plan, and no-network assertions. |
| REQ-011 | When a cartridge gameplay action is submitted from any signed screen, the client shall bind its current screen ID and the server shall reauthorize the participant, exact pin, current lifecycle, requested screen, emitter, action, and payload before durable admission and existing-authority dispatch; navigation actions shall be rejected at this endpoint. | Rust and PostgreSQL tests for valid secondary-screen action, wrong/unknown screen, cross-screen action, navigation-action injection, replay, revision conflict, and both authority paths. |
| REQ-012 | When authoritative session state changes or an uncertain gameplay mutation resolves, the client shall refetch REST truth and recompile the current screen only while it remains valid for the same session/release, otherwise reset to the signed entry screen with an explicit state. | QML/runtime refresh, completion, release-change, invalid-schema, retry, and reconnect fixtures. |
| REQ-013 | When Door Legends is exercised after its catalog selection has advanced, a clean client shall explicitly acquire the older pinned cartridge, navigate its signed secondary screen and back, invoke its gameplay action, and recover the exact result after restart without publisher executable code. | Clean-clone PostgreSQL/provider/companion/QML vertical drill. |
| REQ-014 | When a cartridge has only an entry screen, a session has no presentation pin, distribution is unavailable, or a client uses existing catalog install/remove flows, current API, cache, Signal Siege presenter, and entry-screen cartridge behavior shall remain compatible. | Existing server/runtime/QML suites plus legacy and capability-subset fixtures. |
| REQ-015 | Before delivery, the versioned SDK, native package, generated fixtures, authored docs, and repository shall describe and ship the same acquisition/navigation contract and pass focused checks plus the canonical worktree-bound diff gate. | SDK/package reproducibility checks, documentation review, and `bin/gate.sh --diff`. |

## Scope

- In:
  - retained exact marketplace evidence sufficient to distribute a session's
    historical pinned release;
  - participant-authorized session acquisition and explicit client install;
  - signed declarative screen navigation, companion screen compilation,
    trusted QML history/focus, and screen-bound gameplay actions;
  - a multi-screen Door Legends historical-release vertical, compatibility,
    packaging, operations, and documentation.
- Out:
  - automatic background downloads, current-release substitution, marketplace
    key enrollment/rotation, or public marketplace browsing;
  - publisher QML/JavaScript/native code, URLs, arbitrary navigation commands,
    direct cartridge/provider networking, or client-side rules authority;
  - operator-custom signing, public Provider SDK/onboarding, server modules,
    cross-server sessions, or migration of a session to a different release.

## Links

- Intake: none
- Pipeline spec: [completed spec](../../pipeline/completed/historical-session-cartridge-acquisition-and-multi-screen-navigation.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [system overview](../../../architecture/system-overview.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
