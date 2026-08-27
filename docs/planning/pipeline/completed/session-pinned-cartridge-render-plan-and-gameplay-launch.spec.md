---
title: Session-pinned cartridge render plan and gameplay launch
pipeline_id: 68a0691d-8e6d-48d0-83a1-8c43c6b68b29
status: Phase 5 — Complete PASS
ticket: TICKET-034
ticket_doc: docs/planning/tickets/closed/TICKET-034-session-pinned-cartridge-render-plan-and-gameplay-launch.md
aar: docs/planning/knowledge/aar/AAR-034-session-pinned-cartridge-render-plan-and-gameplay-launch.md
created: 2026-08-26
---

# Session-pinned cartridge render plan and gameplay launch — spec

## Intent

Turn a Ticket 033 profile mount into a truthful playable frontend by pinning
one exact admitted cartridge to an authoritative game session, compiling only
its signed declarative entry screen in the native companion, rendering the
bounded plan through platform-owned QML, and routing declared actions through
the selected OmarchyGS server. The release proof is the existing independent
Door Legends cartridge/provider pair.

## Scope

- In:
  - additive immutable presentation binding for eligible newly created
    compiled or registered-provider sessions;
  - participant-visible binding/lifecycle projection and exact cartridge
    action dispatch through the server's existing authority paths;
  - client-companion mounted-release resolution, trusted render compilation,
    bounded ephemeral asset delivery, and QML session integration;
  - platform-presenter fallback, lifecycle/error UX, real Door Legends
    end-to-end evidence, packaging, recovery, and documentation.
- Out:
  - any publisher executable frontend, cartridge networking, provider endpoint
    exposure, or direct client-provider credentials;
  - automatic historical-release acquisition and multi-screen navigation;
  - public marketplace-key enrollment, operator-custom trust, generalized
    provider SDK/onboarding, or module/hook execution;
  - removal of legacy session rows, raw platform commands, or Signal Siege's
    trusted compiled presenter.

## Acceptance criteria (EARS)

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

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | A cartridge remains signed inert data; only the existing Rust compiler and repository-owned QML vocabulary may render it. | Playability must not create an executable frontend escape hatch. |
| 2 | The server pins a presentation only at new session creation and never silently repins an existing session. | Session history and active gameplay must retain exact release meaning across catalog updates. |
| 3 | Rendering remains in the same-user native companion and requires the client-controlled marketplace key plus an exact profile mount. | QML must not gain cache authority, publisher parsing, or trust-root selection. |
| 4 | Cartridge actions return through a new participant-authorized OmarchyGS endpoint and then the session's single existing rules authority. | The cartridge never obtains a credential, URL, provider grant, or direct socket. |
| 5 | Door Legends is the executable vertical proof; Signal Siege keeps its platform presenter unless an independently admitted exact cartridge is later supplied. | The existing clean-room release and provider view/action contract prove portability without inventing provenance. |
| 6 | V1 launch uses the signed entry screen only and does not auto-download an absent historical mount. | Multi-screen routing and historical acquisition require separate durable navigation/distribution contracts. |

## Linked artifacts

- Ticket: [TICKET-034](../../tickets/closed/TICKET-034-session-pinned-cartridge-render-plan-and-gameplay-launch.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [system overview](../../../architecture/system-overview.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | autonomous approved-continuation scope review |
| 2 Design | Architecture, file manifest, regression plan | actionable design plus CodeGraph receipt |
| 3 Implement | Code matching the design | focused compilation/tests and self-review |
| 3.5 Inspect | Findings ledger and fixes | verified dispositions plus fresh CodeGraph receipt |
| 4 Validate | Tests run and delivery gate green | matching worktree gate receipt |
| 5 Complete | AC audit, docs, submitted AAR, archive | no silent drops plus OpenWiki receipt |
| Delivery | Fresh gate, staged review, authorized commit/push | matching receipt and remote readback |
