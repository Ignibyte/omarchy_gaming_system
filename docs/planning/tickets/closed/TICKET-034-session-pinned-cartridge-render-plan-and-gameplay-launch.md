---
title: TICKET-034-session-pinned-cartridge-render-plan-and-gameplay-launch
status: closed
ticket_number: 034
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/session-pinned-cartridge-render-plan-and-gameplay-launch.spec.md
---

# TICKET-034-session-pinned-cartridge-render-plan-and-gameplay-launch

## Summary

Bind one exact server-admitted Game Cartridge release to each eligible new
authoritative game session, compile its mounted presentation through the native
client companion, render it through trusted QML, and dispatch only its declared
actions back through OmarchyGS. Prove the complete path with the separately
built Door Legends cartridge and provider.

## Why

Ticket 033 made signed cartridges independently verifiable, cacheable, and
mountable, but a mount is deliberately not playable. This slice closes the
next trust boundary without granting cartridge code, credentials, filesystem,
or network authority and turns the existing first-party Door Legends release
into the first real playable portable cartridge.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an eligible game session is created, the server shall atomically pin either no presentation or one immutable marketplace release and admission revision whose game key and rules version match the session. | Migration checks plus PostgreSQL solo, challenge, provider, mismatch, and concurrency tests. |
| REQ-002 | When the session uses registered-provider authority, the server shall pin a cartridge only when the provider release's immutable cartridge digest equals the effective server-catalog release; otherwise it shall preserve the session with no presentation binding. | Provider launch database/API tests for exact match, absence, and mismatch. |
| REQ-003 | When catalog selection or lifecycle changes after session creation, the system shall never silently repin that session to another cartridge and shall apply the pinned release's explicit active-session lifecycle decision. | PostgreSQL upgrade, rollback, omission, deprecated, suspended, revoked, and retired-session tests. |
| REQ-004 | When a participant reads a game session, the API shall return a bounded exact presentation binding and current presentation state without exposing operator-only metadata, keys, paths, credentials, provider endpoints, or another participant's private authority. | Exact-schema API and privacy tests. |
| REQ-005 | When the companion prepares a bound session, it shall require a matching server-profile mount, client-trusted marketplace-key fingerprint, publisher key, signed policy, game/rules/cartridge identity, digest, and server admission before resolving immutable cached content. | Client-runtime hostile binding, profile isolation, key substitution, lifecycle, restart, and cache-race tests. |
| REQ-006 | When a matching mount and authoritative bounded view are supplied, the companion shall compile the production `omarchygs.render-plan/v1` contract with trusted preferences and return only a strict session-bound plan envelope plus host-created asset authority. | Rust renderer/companion integration tests with valid and schema-invalid Door Legends views and Core/Rich-2D limits. |
| REQ-007 | When a plan contains assets, the companion shall expose only authenticated digest-named bytes through a random per-plan loopback capability with exact Host, token, media type, count, byte, memory, lifetime, and eviction bounds. | Loopback asset success plus wrong-host/token/path/digest/media, eviction, concurrency, and shutdown tests. |
| REQ-008 | When QML opens a bound session, the client shall prefer the accepted trusted cartridge plan, otherwise preserve the eligible platform presenter, and expose explicit loading, missing-mount, incompatible, offline, stale, revoked, and protocol-error states without treating a mount as authority. | QML production-root fixtures at supported sizes with keyboard, focus, accessibility, plain-text, fallback, and hostile-envelope assertions. |
| REQ-009 | When a trusted cartridge node emits an action, the client shall send only the session ID, expected revision, exact pinned cartridge digest, declared action ID, schema-shaped payload, and a fresh or retained idempotency identity to the selected OmarchyGS server. | QML request-shape and uncertain-retry fixtures. |
| REQ-010 | When the server receives a cartridge action, it shall participant-authorize the session, re-resolve the exact pinned verified cartridge under active-session policy, validate that the entry screen declares the exact action/payload shape, and only then translate it to the session's sole compiled or registered-provider authority. | Rust unit and PostgreSQL API tests for foreign session, digest/action/payload tampering, stale revision, lifecycle denial, and both authority paths. |
| REQ-011 | When an action is replayed, conflicts, times out, or commits, the system shall preserve the existing command idempotency/revision semantics, never report an unconfirmed result, and refresh/recompile only authoritative REST state. | Existing command replay suite plus cartridge-action replay, collision, timeout, conflict, and QML refetch tests. |
| REQ-012 | When the Door Legends provider, exact signed cartridge, matching server admission, client mount, and QML shell are composed, a player shall render the provider's authenticated view, invoke `enter`, reach the provider-owned terminal result, and recover it after restart without loading publisher executable code. | Clean-clone provider/cartridge, real PostgreSQL/broker/companion/QML vertical integration drill. |
| REQ-013 | When no eligible binding, distribution runtime, helper, trusted key, or matching mount exists, existing Signal Siege and generic session/catalog APIs shall remain compatible and the client shall fail closed or use the current platform-owned presenter as appropriate. | Existing server/QML suites plus capability-subset and legacy-row fixtures. |
| REQ-014 | Before delivery, the native package and complete repository shall include the exact companion/renderer/QML payload and pass focused checks, reproducible package evidence, and the canonical worktree-bound diff gate. | Package source/build/smoke tests and `bin/gate.sh --diff`. |

## Scope

- In:
  - forward-only session-presentation pinning and active-session lifecycle;
  - exact participant API projection and a cartridge-action endpoint;
  - private mounted-content resolution, production render-plan compilation,
    bounded ephemeral asset delivery, and session-bound receipts in the native
    companion;
  - trusted QML gameplay integration with platform fallback;
  - the real first-party Door Legends cartridge/provider vertical proof;
  - compatibility, privacy, lifecycle, package, and recovery evidence.
- Out:
  - publisher QML/JavaScript/native/WebEngine execution or direct client-to-provider networking;
  - automatic acquisition of an unmounted historical session release;
  - multiple-screen routing beyond the signed v1 entry screen;
  - public marketplace trust-key enrollment, operator-custom cartridges,
    public Provider SDK, external providers, or server modules/hooks;
  - removal of the existing raw platform command API or Signal Siege presenter.

## Links

- Intake: none; continuation of the approved owner-operated cartridge roadmap.
- Pipeline spec: [session-pinned-cartridge-render-plan-and-gameplay-launch.spec.md](../../pipeline/completed/session-pinned-cartridge-render-plan-and-gameplay-launch.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [system overview](../../../architecture/system-overview.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
