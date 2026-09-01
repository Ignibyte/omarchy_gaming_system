---
title: Usurper healing-potion shop and combat-turn parity
pipeline_id: a47cb42d-35b4-4137-914b-aae921ac99cc
status: Phase 5 — Complete PASS
ticket: TICKET-050
ticket_doc: docs/planning/tickets/closed/TICKET-050-usurper-healing-potion-shop-and-combat-turn-parity.md
aar: docs/planning/knowledge/aar/AAR-050-usurper-healing-potion-shop-and-combat-turn-parity.md
created: 2026-08-31
completed: 2026-08-31
---

# Usurper healing-potion shop and combat-turn parity — spec

## Intent

Make healing potions an earned/spendable solo resource and correct the active
combat turn semantics while preserving the separate deterministic provider,
signed inert cartridge, and platform-owned trusted QML boundaries.

## Scope

- In:
  - source-linked healing-potion purchase and limits;
  - Magic Shop location/phase, state transition, view, and inert screen;
  - dungeon healing and configured combat heal-then-attack behavior;
  - provider/cartridge/QML proof and full regression evidence.
- Out:
  - spells, Magic Shop items, poison/Alchemist behavior, class/race specials,
    quests, finale, immortality, shared state, or platform application changes;
  - production admission, deployment, or publication.

## Acceptance criteria (EARS)

The authoritative requirements and verification matrix are the six EARS
requirements in [TICKET-050](../../tickets/closed/TICKET-050-usurper-healing-potion-shop-and-combat-turn-parity.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port only the healing-potion branch of the Magic Shop in this slice. | It closes a complete resource loop without composing unrelated spell, item, and poison systems. |
| 2 | Declare the development combat mode as v0.20e `QuaffOpt = 1`. | That is the configured source default and requires healing followed by immediate normal attack. |
| 3 | Preserve the source's 150-potion launch balance while enforcing its configured 75-potion shop cap on purchases. | These values coexist in v0.20e; silently normalizing one into the other would reduce fidelity. |
| 4 | Keep all rules and private state in the separate Rust provider. | This remains one-player session-bounded behavior that fits the reviewed public Provider SDK. |
| 5 | Add only signed inert presentation data to the cartridge. | The platform-owned QML renderer remains the sole executable presentation surface. |

## Linked artifacts

- Ticket: [TICKET-050](../../tickets/closed/TICKET-050-usurper-healing-potion-shop-and-combat-turn-parity.md)
- Prior milestone: [Ticket 049 notes](../completed/usurper-solo-equipment-economy.notes.md)
- Port map: adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | canonical rule trace, state/command/view manifest, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | external model/rules/provider/cartridge expansion | focused checks and visible preview |
| 3.5 Inspect | cross-repository finding ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | external full checks and platform fast gate | evidence matches every requirement |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | full diff gate and authorized publication | matching receipt and explicit authorization |
