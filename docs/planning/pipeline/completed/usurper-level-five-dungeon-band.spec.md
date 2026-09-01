---
title: Usurper Level-Five Dungeon Band
pipeline_id: 977608e8-61c6-4ce9-a596-4cc73b8701b7
status: Phase 5 — Complete PASS
ticket: TICKET-057
ticket_doc: docs/planning/tickets/closed/TICKET-057-usurper-level-five-dungeon-band.md
aar: docs/planning/knowledge/aar/AAR-057-usurper-level-five-dungeon-band.md
created: 2026-08-31
---

# Usurper Level-Five Dungeon Band — spec

## Intent

Ship the next source-complete normal dungeon band in the separate Usurper Rust
provider: exact level-five data and legacy selection, deterministic combat
composition, a signed inert control, and a visible trusted-QML preview.

## Scope

- In:
  - v0.20e monster records 40–49 with reviewed strength and equipment flags;
  - source-order `Random(50)` rejection draws, normal selection 41–49, and
    source-derived 45 HP;
  - strict rules/state v10, levels one through five, provider action/replay,
    signed cartridge, fixtures, provenance, tests, documentation, and preview.
- Out:
  - level six or higher and the composite dungeon event/team/shared-world paths;
  - platform gameplay logic, migrations, new provider protocol, executable
    cartridge content, packaging, admission, deployment, commit, push, or
    publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When this development release is built, the game shall identify the exact v0.20e level-five editor records and retain the established dungeon change-level, encounter-selection, and monster-HP source links. | Canonical-source readback, source-trace validation, compatibility documentation review, and fixed-data tests. |
| REQ-002 | When provider-owned state is accepted, dungeon level and any active monster shall be internally consistent with implemented levels one through five and exact rules schema v10, while malformed or out-of-band state shall fail without advancing RNG. | Hostile state/JSON tests, schema-version tests, and complete-state/RNG equality. |
| REQ-003 | When a player chooses dungeon level one, two, three, four, or five from an allowed non-combat location, the reducer shall retain no monster, move the character into the dungeon, and expose the selected level without consuming RNG; any other level shall be rejected unchanged. | Reducer transition, phase, boundary, immutability, and draw-trace tests. |
| REQ-004 | When Look starts a level-five encounter, the reducer shall spend one fight and repeat `Random(50)` until the result is greater than forty, select exact editor record 41–49, initialize HP to three times reviewed base strength, and preserve record 40 as normally unreachable. | Rejection-loop draw-trace fixtures, exact roster/order/flags tests, deterministic twins, and encounter-state assertions. |
| REQ-005 | When level-five combat is played, existing attack, retreat, potion, spell, class-special, reward, and Gnoll-player poison behavior shall compose unchanged except for source-defined level/monster inputs, including the level-five retreat-damage bound. | Cross-feature combat regressions, exact retreat trace, replay/restart tests, and full workspace checks. |
| REQ-006 | When the provider and cartridge expose this slice, a player shall be able to switch among levels one through five and visibly enter a signed level-five encounter through existing provider and trusted-QML protocols, without level six, dungeon events, shared realm, platform gameplay code, packaging, admission, deployment, or publication. | Fixed-action/view tests, live profile twice across restart, signed cartridge conformance, QML smoke, scope review, security inspection, and visible preview. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port the canonical v0.20e Level 5 records at indices 40–49 exactly. | Preserves the chosen publisher-linked baseline rather than inventing balance data. |
| 2 | Retain record 40 while normal selection accepts only 41–49. | The original rejection loop makes the boundary record unreachable but still part of the source table. |
| 3 | Preserve every rejected `Random(50)` draw in deterministic state. | Discarded source RNG work changes all subsequent replay-visible outcomes. |
| 4 | Advance state, rules, and cartridge identity to v10 without migrating v9 state. | Avoids silently interpreting older persisted JSON under broader rules. |
| 5 | Keep game rules/state in the external provider and presentation executable-free. | Maintains the established single-authority and trusted-renderer architecture. |
| 6 | Defer packaging and delivery. | Matches the user's direction to build and show the game first. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-057-usurper-level-five-dungeon-band.md`
- Architecture: `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md`, `docs/architecture/game-cartridges.md`
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope and EARS requirements settled |
| 2 Design | Architecture, file manifest, regression plan | actionable design and CodeGraph receipt |
| 3 Implement | Code matching the design | focused checks and self-review |
| 3.5 Inspect | Findings ledger and fixes | resolved findings and fresh CodeGraph receipt |
| 4 Validate | Tests run and delivery gate green | matching gate receipt and visible preview |
| 5 Complete | AC audit, docs, submitted AAR, archive | no silent drops and OpenWiki complete |
| Delivery | Fresh gate, staged review, authorized commit | separately authorized only |
