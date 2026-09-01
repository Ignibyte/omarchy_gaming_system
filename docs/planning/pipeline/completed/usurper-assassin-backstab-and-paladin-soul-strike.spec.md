---
title: Usurper Assassin Backstab and Paladin Soul Strike
pipeline_id: 207dd49f-1088-4058-bde5-f25cf2145a87
status: Phase 5 — Complete PASS
ticket: TICKET-052
ticket_doc: docs/planning/tickets/closed/TICKET-052-usurper-assassin-backstab-and-paladin-soul-strike.md
aar: docs/planning/knowledge/aar/AAR-052-usurper-assassin-backstab-and-paladin-soul-strike.md
created: 2026-08-31
completed: 2026-08-31
---

# Usurper Assassin Backstab and Paladin Soul Strike — spec

## Intent

Make the original Assassin and Paladin choices meaningfully distinct in
dungeon combat by porting their shared class-special menu seam through the
deterministic rules provider and trusted visible cartridge.

## Scope

- In:
  - source-linked mental-health/addiction defaults and class-special commands;
  - Backstab and Soul Strike attack/response composition;
  - strict provider actions, inert combat bindings, deterministic tests, and
    a visible trusted QML preview.
- Out:
  - Gnoll poison, PvP/NPC special use, teams, mercy, fight-to-death, higher
    spells, monster magic, dungeon events, quests, shared realm, and platform
    application code;
  - packaging, production registration, admission, deployment, or publication.

## Acceptance criteria (EARS)

The authoritative requirements and verification matrix are the six EARS
requirements in [TICKET-052](../../tickets/closed/TICKET-052-usurper-assassin-backstab-and-paladin-soul-strike.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Port Assassin Backstab and Paladin Soul Strike together. | The original dungeon menu assigns both to the same class-gated `1` option and composes each with the same normal-attack/monster-response phase. |
| 2 | Preserve mental health and addiction as bounded durable player scalars, initialized to the exact new-character values. | Soul Strike conditionally reads both even though the current playable paths do not yet degrade them. |
| 3 | Keep generic class-specific commands and expose one shared fixed class-special action that the provider adapter maps from authenticated current class state to Backstab or a one-HP Soul Strike. | This mirrors the original shared menu key, preserves variable HP investment in the provider API, and avoids rendering an ineligible second special when Cartridge v1 has no trusted numeric-input or conditional-node facility. |
| 4 | Preserve the existing translated normal-attack compatibility branch and place special draws/effects at their exact source-relative boundaries. | This adds the missing class behavior without claiming broader monster/PvP parity or rewriting an already source-linked scalar slice. |
| 5 | Keep every new scalar and rule inside the player-private provider snapshot and use only signed inert cartridge data. | No platform authority, SDK, route, migration, or executable publisher-QML change is needed. |

## Linked artifacts

- Ticket: [TICKET-052](../../tickets/closed/TICKET-052-usurper-assassin-backstab-and-paladin-soul-strike.md)
- Prior milestone: [Ticket 051 notes](../completed/usurper-level-one-spellcasting-and-mana-loop.notes.md)
- Port map: adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS criteria recorded |
| 2 Design | canonical special/attack trace, state-command-view manifest, regression plan | CodeGraph design receipt and actionable manifest |
| 3 Implement | external model/rules/provider/cartridge expansion | focused checks and visible preview |
| 3.5 Inspect | cross-repository finding ledger and fixes | fresh CodeGraph receipt and all findings disposed |
| 4 Validate | external full checks and platform diff gate | evidence matches every requirement |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | authorized commit/publication only | explicitly deferred |
