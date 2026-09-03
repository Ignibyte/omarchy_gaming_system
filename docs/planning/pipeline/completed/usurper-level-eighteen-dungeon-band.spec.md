---
title: Usurper Level Eighteen Dungeon Band
pipeline_id: f688f06e-041d-43c6-956f-5f56de3c88e3
status: Phase 5 — Complete PASS
ticket: TICKET-072
ticket_doc: docs/planning/tickets/closed/TICKET-072-usurper-level-eighteen-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-072-usurper-level-eighteen-dungeon-band.md
created: 2026-09-03
---

# Usurper Level Eighteen Dungeon Band — spec

## Intent

Advance the separate source-linked Usurper port by one complete normal dungeon
band while preserving provider ownership, signed inert presentation, unique
controls, real-input proof, and all existing combat behavior.

## Scope

- In:
  - exact v0.20e Level 18 monster records 170–179;
  - normal Level 18 rejection-loop selection, three-times-strength HP, and
    bounded trace risk/size evidence;
  - draw-free level switching across levels one through eighteen;
  - rules/state/cartridge v23, bounded eighteenth-choice projection,
    deterministic reducers, provider projections, signed presentation,
    fixtures, provenance, documentation, and tests;
  - game-neutral trusted-QML delegate row sizing and regression evidence needed
    to prevent transient overlap during asynchronous plan replacement;
  - live provider restart conformance and provider-backed workspace-8 play.
- Out:
  - Level 19+, dungeon events, quests, finale, shared realm, or new combat
    systems;
  - platform rules/state, database, Provider SDK/protocol, renderer protocol,
    or renderer compiler changes;
  - registration, admission, packaging, deployment, commit, push, or
    publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e Level 18 editor records 170–179 and retain the established dungeon-selection, monster-HP, and retreat source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through eighteen and exact rules schema v23, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one through eighteen from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a Level 18 encounter, the reducer shall spend one fight and repeat `Random(180)` until the result is greater than one hundred seventy, select exact editor record 171–179, initialize HP to three times reviewed base strength, preserve record 170 as normally unreachable, and retain a bounded trace whose quantified tail risk and maximum state size satisfy the development-provider contract. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic long-run progression, serialized-state ceiling proof, and encounter-state assertions. |
| REQ-005 | When Level 18 combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the Level 18 retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through eighteen and visibly enter a signed Level 18 encounter through provider-backed trusted QML, with each choice occupying one explicit delegate row and activating exactly once, without Level 19, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed action/view tests, explicit non-overlapping row geometry, unique-control and real-input checks, live profile twice across restart, signed cartridge conformance, local-play click smoke, scope/security inspection, and visible workspace-8 play. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port only the normal Level 18 band and preserve event routing as out of scope. | `DUNGEONC.PAS` keeps ordinary monster loading in the event-false branch; events remain an independent unfinished subsystem. |
| 2 | Retain editor record 170 in canonical data but keep normal selection restricted to 171–179 through the original rejection loop. | Record inventory and reachable runtime behavior are separate compatibility facts. |
| 3 | Advance rules, state schema, and cartridge identities together to v23. | Older serialized state and signed action surfaces must not be relabeled as the deeper rules release. |
| 4 | Reuse the generic provider-owned dungeon/combat reducer and add only Level 18 data, bounds, actions, projections, and tests. | OmarchyGS remains transport/rendering authority, never a second game-rules owner. |
| 5 | Add one bounded eighteenth-choice string to the external game view/schema and bind it to exactly one Level 18 action. | The existing A–Q fields are occupied on the dungeon screen; widening the data-only game view is smaller than overloading another semantic field or changing platform rendering. |
| 6 | Re-audit the bounded RNG trace against Level 18's valid-tail probability and provider state ceiling before retaining or changing its capacity. | The source loop is unbounded, while the development provider state is deliberately bounded; capacity must be evidence-driven rather than copied forward. |
| 7 | Give every asynchronous trusted-node loader an explicit height bound to its loaded item. | The live duplicate-control report exposed a transient geometry gap that settled object-count tests and offscreen frames could miss; the fix remains game-neutral and does not change action authority. |

## Linked artifacts

- Ticket: [TICKET-072](../../tickets/closed/TICKET-072-usurper-level-eighteen-dungeon-band.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: continuing goal after completing the Level 17 normal dungeon band.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | concrete shippable contract |
| 2 Design | Source/data/control-flow design and exact manifest | worktree-bound CodeGraph receipt |
| 3 Implement | External rules/data/provider/cartridge Level 18 slice | compile and self-review |
| 3.5 Inspect | Findings ledger and security review | lead disposition |
| 4 Validate | Focused/full tests, live corpus, and delivery gate | matching gate receipt |
| 5 Complete | AC audit, docs, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | explicit user authorization only |
