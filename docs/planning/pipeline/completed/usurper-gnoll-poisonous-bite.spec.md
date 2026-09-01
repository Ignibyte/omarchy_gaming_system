---
title: Usurper Gnoll Poisonous Bite
pipeline_id: 29c361f7-4873-417d-8b2a-6fae36585000
status: Phase 5 — Complete PASS
ticket: TICKET-053
ticket_doc: docs/planning/tickets/closed/TICKET-053-usurper-gnoll-poisonous-bite.md
aar: docs/planning/knowledge/aar/AAR-053-usurper-gnoll-poisonous-bite.md
created: 2026-08-31
completed: 2026-08-31
---

# Usurper Gnoll Poisonous Bite — spec

## Intent

Make the original Gnoll choice meaningfully distinct in solo dungeon combat by
porting its passive poisonous bite and per-turn monster poison through the
deterministic provider and trusted visible cartridge.

## Scope

- In:
  - source-linked Gnoll bite and transient monster poison state;
  - exact attack, spell/special, poison-tick, response, and lethal order;
  - provider replay, view narration/status, deterministic tests, and a visible
    trusted QML preview.
- Out:
  - Alchemist weapon poison, Sage poison, PvP/NPC poison, teams, disease,
    multiple monsters, events, quests, shared realm, and platform application
    code;
  - packaging, production registration, admission, deployment, or publication.

## Acceptance criteria (EARS)

The authoritative requirements and verification matrix are the six EARS
requirements in [TICKET-053](../../tickets/closed/TICKET-053-usurper-gnoll-poisonous-bite.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Model the Gnoll bite as a passive part of every completed offensive combat turn, not as another cartridge command. | The original applies the bite roll in the shared attack phase after ordinary power calculation; its stray menu-key allowance performs no Gnoll action. |
| 2 | Store `poisoned` only on the current `MonsterState`, initialized false for every encounter. | The source resets the monster flag at creation and retains it only for the fight. |
| 3 | Preserve the exact bound-4 bite and bound-5 tick position across Attack, configured Quick Heal, level-one spells, Backstab, and Soul Strike. | Gnoll race and class are independent, and all of those accepted choices enter the same source attack phase. |
| 4 | Preserve poison-lethal completion without immediate XP/gold reward. | The source calls its reward helper before the later poison tick and exits the fight when no monster remains; the oddity is observable. |
| 5 | Advance the unadmitted game/rules/cartridge identity to v6 and use only existing provider and presentation protocols. | Durable encounter state and command outcomes change, but the opaque provider and inert view contracts already fit. |

## Linked artifacts

- Ticket: [TICKET-053](../../tickets/closed/TICKET-053-usurper-gnoll-poisonous-bite.md)
- Prior milestone: [Ticket 052 notes](../completed/usurper-assassin-backstab-and-paladin-soul-strike.notes.md)
- Port map: adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | canonical poison trace, state/turn/view manifest, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | external model/rules/provider/cartridge expansion | focused checks and visible preview |
| 3.5 Inspect | cross-repository finding ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | external full checks and platform diff gate | evidence matches every requirement |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | authorized commit/publication only | explicitly deferred |
