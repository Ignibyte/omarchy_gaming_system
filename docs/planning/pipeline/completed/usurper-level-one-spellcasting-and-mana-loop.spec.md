---
title: Usurper level-one spellcasting and mana loop
pipeline_id: e5b12d13-82f9-49e9-8d57-2e9a083778bb
status: Phase 5 — Complete PASS
ticket: TICKET-051
ticket_doc: docs/planning/tickets/closed/TICKET-051-usurper-level-one-spellcasting-and-mana-loop.md
aar: docs/planning/knowledge/aar/AAR-051-usurper-level-one-spellcasting-and-mana-loop.md
created: 2026-08-31
completed: 2026-08-31
---

# Usurper level-one spellcasting and mana loop — spec

## Intent

Make the original caster choices meaningfully playable by connecting their
first learned spell and mana to dungeon combat and the trusted visible game
surface, while preserving the separate deterministic provider boundary.

## Scope

- In:
  - learned/active spell state and level-one spell metadata;
  - three class-specific level-one effects and combat response order;
  - mana spend/refill, encounter reset, provider action, and inert view data;
  - source-linked deterministic tests and a live trusted QML preview.
- Out:
  - higher spells, monster magic, Magic Shop objects, poison, specials, teams,
    events, quests, shared realm, platform application code, and packaging;
  - production registration, admission, deployment, or publication.

## Acceptance criteria (EARS)

The authoritative requirements and verification matrix are the six EARS
requirements in [TICKET-051](../../tickets/closed/TICKET-051-usurper-level-one-spellcasting-and-mana-loop.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port all three level-one player spells together. | They share one creation/cost/turn seam and make every original caster class observably distinct. |
| 2 | Use the editor's explicit monster magic-resistance default of 10 as the development encounter fixture. | The release has no initialized `MONSTER.DAT`; this is source-backed and honest, while a fabricated live-world record or displaced editor RNG would not be. |
| 3 | Keep the cast command generic by spell ordinal while exposing only the currently learned first spell in the fixed cartridge action. | This preserves the original A–L spell vocabulary without pretending higher spells are implemented. |
| 4 | Keep learned and active spell state in the player-private provider snapshot. | This one-player combat state fits the existing public Provider SDK and needs no shared-realm seam. |
| 5 | Add only signed inert combat bindings and fixture data. | The platform-owned QML renderer remains the sole executable presentation surface. |

## Linked artifacts

- Ticket: [TICKET-051](../../tickets/closed/TICKET-051-usurper-level-one-spellcasting-and-mana-loop.md)
- Prior milestone: [Ticket 050 notes](../completed/usurper-healing-potion-shop-and-combat-turn-parity.notes.md)
- Port map: adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | canonical spell/turn trace, state-command-view manifest, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | external model/data/rules/provider/cartridge expansion | focused checks and visible preview |
| 3.5 Inspect | cross-repository finding ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | external full checks and platform diff gate | evidence matches every requirement |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | authorized commit/publication only | explicitly deferred |
