---
title: Historical session cartridge acquisition and multi-screen navigation
pipeline_id: e6c0e63b-200a-481d-8670-8531db96661f
status: Phase 5 — Complete PASS
ticket: TICKET-035
ticket_doc: docs/planning/tickets/closed/TICKET-035-historical-session-cartridge-acquisition-and-multi-screen-navigation.md
aar: docs/planning/knowledge/aar/AAR-035-historical-session-cartridge-acquisition-and-multi-screen-navigation.md
created: 2026-08-26
---

# Historical session cartridge acquisition and multi-screen navigation — spec

## Intent

Complete the next portable-cartridge seam by letting a participant explicitly
acquire the exact signed release already pinned to an old session even when the
server's current catalog has changed, then navigate multiple reviewed signed
screens through trusted OmarchyGS controls. Historical proof must remain
cryptographic and release-exact; navigation must remain inert presentation,
not a new execution or networking authority.

## Scope

- In:
  - bounded retained marketplace evidence for exact historical session
    acquisition, participant authorization, lifecycle linearization, and
    client-controlled trust verification;
  - explicit companion/QML install and recovery for a missing session mount;
  - a bounded signed host-navigation contract, arbitrary signed-screen render
    preparation, trusted QML history/focus, and screen-bound server actions;
  - Door Legends clean-client proof after catalog advancement, backward
    compatibility, native packaging, tests, operations, and documentation.
- Out:
  - automatic background acquisition or substitution with any current catalog
    release;
  - publisher executable QML, JavaScript, native code, WebEngine, URLs,
    arbitrary payload navigation, direct provider access, or client rules;
  - marketplace trust-key enrollment/rotation, operator-custom cartridges,
    public provider onboarding/SDK, modules/hooks, cross-server gameplay, or
    session release migration.

## Acceptance criteria (EARS)

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

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Historical acquisition resolves only the immutable session presentation pin and never consults the current catalog as a release selector. | A saved match must retain its reviewed visual identity across upgrade, rollback, and omission. |
| 2 | The server retains authentic marketplace snapshot evidence; it never synthesizes or rewrites marketplace review claims for an old release. | Client verification must remain independently cryptographic rather than trusting a server-authored historical assertion. |
| 3 | Missing historical content requires an explicit participant install action. | Downloading is a player-visible trust and resource transition, while opening session truth must remain possible without it. |
| 4 | One profile may retain multiple exact mounts for the same game, keyed by digest and admission revision; install and removal never clobber another session's release. | Historical and current sessions must coexist rather than taking turns replacing one game-level pointer. |
| 5 | Navigation is a signed bounded host operation between reviewed screen IDs and has no URL, payload, provider call, or gameplay effect. | Multiple screens must not become an executable frontend or covert network bridge. |
| 6 | Gameplay actions bind the exact rendered screen and are revalidated against that screen's signed emitter before durable server admission. | Entry-only validation cannot safely or truthfully authorize actions emitted by another screen. |
| 7 | The same authoritative session view is validated independently against each requested screen's signed schema. | Providers and compiled rules remain the sole state authority; a cartridge may present but not manufacture screen state. |
| 8 | QML history is bounded and scoped to one session plus release; authoritative session/release changes cannot carry stale destinations across contexts. | Local navigation state must not escape its exact signed presentation identity. |
| 9 | Door Legends proves the complete historical and multi-screen path while retaining its separate provider authority. | The existing real cartridge/provider seam is stronger evidence than a fixture-only game. |

## Linked artifacts

- Ticket: [TICKET-035](../../tickets/closed/TICKET-035-historical-session-cartridge-acquisition-and-multi-screen-navigation.md)
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
