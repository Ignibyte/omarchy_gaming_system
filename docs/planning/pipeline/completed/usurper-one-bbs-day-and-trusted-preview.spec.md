---
title: Usurper one BBS day and trusted preview
pipeline_id: 523ecc99-9c51-4cca-bb2f-597075f23baa
status: Phase 5 — Complete PASS
ticket: TICKET-048
ticket_doc: docs/planning/tickets/closed/TICKET-048-usurper-one-bbs-day-and-trusted-preview.md
aar: docs/planning/knowledge/aar/AAR-048-usurper-one-bbs-day-and-trusted-preview.md
created: 2026-08-31
completed: 2026-08-31
---

# Usurper one BBS day and trusted preview — spec

## Intent

Turn the authenticated Usurper v0.20e port map into the first executable,
source-linked vertical slice: a deterministic one-day rules/provider flow in
the separate game repository and a signed multi-screen cartridge rendered by
the real trusted OmarchyGS QML surface.

## Scope

- In:
  - buildable GPL Rust game workspace beside the platform repository;
  - fixed-development-alias creation, race/class selection, Main Street,
    status, dungeon encounter/combat/retreat/death/reward, healer, level
    master, sleep, atomic day advance, news, and re-entry;
  - explicit legacy scalar, clock, RNG, source-symbol, and fixture traces;
  - packaged public Provider SDK/starter consumption for a session-bounded
    development provider with independent PostgreSQL state;
  - a signed inert Core cartridge and trusted QML preview of every first-day
    screen and fixed client state;
  - cross-repository tests, inspection, documentation, and full platform gate.
- Out:
  - the remaining solo game, shared-world systems, or final parity claim;
  - generalized shared-realm provider storage or trusted alias input;
  - production provider registration/admission, marketplace publication,
    deployment packaging, release signing operations, or public SDK hosting;
  - porting unmarked infrastructure code, binaries, or uncleared historical art.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the separate Usurper repository is prepared for implementation, it shall build as a GPL-2.0-or-later Rust workspace, keep authenticated upstream bytes ignored and immutable, and trace every translated rule and seed value in this slice to the canonical v0.20e source. | Repository inventory, license/provenance scan, source-trace ledger, formatting, Clippy, tests, and upstream hash/readback checks. |
| REQ-002 | When a new deterministic Usurper session launches, the game shall create the explicitly labeled fixed-alias development character, accept one of the original race and class choices, apply source-linked initial scalar behavior, and enter Main Street with a bounded revisioned view. | Table-driven creation fixtures, invalid-command tests, deterministic twin-run comparison, and view-schema validation. |
| REQ-003 | When the character enters the dungeon, the game shall select an allowed level and resolve a source-linked normal encounter with attack and retreat actions, HP/death handling, XP/gold reward, fight consumption, and an indexed bound/result trace for every random draw. | Reducer unit tests covering victory, retreat, death, stale/invalid input, draw ordering, and scalar boundaries. |
| REQ-004 | When the character visits the healer or level master and then sleeps, the game shall apply eligible healing or advancement, end the visit, advance one realm day exactly once, reset the first-slice daily counters, expose bounded mail/news facts, and allow re-entry. | End-to-end one-day fixture, replay/revision tests, maintenance idempotency tests, and byte-identical state/view/event/RNG traces across clean runs. |
| REQ-005 | When Usurper is exercised as an OmarchyGS provider, it shall implement only the public `ProviderGame` rules seam while the starter owns transport, signed protocol, operation receipts, provider PostgreSQL persistence, callbacks, and lifecycle; serialized state and views shall stay within the starter and cartridge bounds and contain no platform identity or credential. | Packaged-SDK build, provider conformance/fault run, PostgreSQL restart/replay test, bounded serialization checks, and privacy scans. |
| REQ-006 | When a development player opens the Usurper cartridge, the trusted renderer shall show signed multi-screen entry, race, class, Main Street, status, dungeon, combat, healer, level-master, mail/news, and sleep presentation using only declared bounded nodes/actions and plain replacement text. | Cartridge pack/conformance, render-plan compilation for every screen/state, QML loading/offline/empty/error and keyboard smoke, plus a captured visible ready-state preview. |
| REQ-007 | When this slice completes, Usurper shall remain a separate process/repository and game-state authority with no platform rule copy, platform database migration, raw publisher QML/JavaScript, historical ANSI asset, external provider admission, marketplace publication, or compiled fallback. | Cross-repository diff and dependency scan, platform route/migration review, cartridge inventory, full local gate, and authority review. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Use the canonical v0.20e Git tree and only GPL-marked game logic as translation input; reimplement infrastructure and use plain replacement presentation. | Preserves the authenticated behavioral baseline without importing ambiguous bundled code or art. |
| 2 | Implement the complete Milestone 1 day, not a decorative mock screen, while keeping later game milestones outside this ticket. | The visible proof must be driven by real deterministic game state and move directly toward the mapped port. |
| 3 | Use the existing session-bounded public `ProviderGame` seam for this solo slice and defer the shared-realm seam to the already recorded pre-Milestone-3 decision. | It exercises the real backend SDK now without pretending the 32 KiB per-session starter is the final Usurper realm model. |
| 4 | Use a conspicuously labeled fixed alias for development because cartridge v1 has no trusted text input node. | It permits an honest creation flow without adding publisher code or silently claiming alias parity. |
| 5 | Keep Usurper rules, provider state, and cartridge source outside the platform repository; platform changes are limited to workflow evidence unless a demonstrated contract defect requires its own recorded deviation. | Preserves the single gameplay authority and independent game history. |
| 6 | Defer marketplace/channel packaging and production admission while still consuming and conforming to the exact public SDK artifacts. | Packaging is not required to prove the game implementation and trusted presentation path. |

## Linked artifacts

- Ticket: [TICKET-048](../../tickets/closed/TICKET-048-usurper-one-bbs-day-and-trusted-preview.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | Source-linked flows, exact file manifest, provider/cartridge contracts, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | External Rust engine/provider/cartridge and development preview harness | focused compile/tests and visible preview |
| 3.5 Inspect | Cross-repository findings ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | External checks, provider/cartridge/QML smoke, full platform gate | matching gate receipt |
| 5 Complete | AC audit, docs, submitted AAR, OpenWiki, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt and explicit authorization |
