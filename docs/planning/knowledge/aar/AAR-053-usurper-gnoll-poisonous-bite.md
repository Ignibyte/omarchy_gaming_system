---
aar: AAR-053-usurper-gnoll-poisonous-bite
ticket: TICKET-053
pipeline: usurper-gnoll-poisonous-bite
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-053-usurper-gnoll-poisonous-bite

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Knowledge register and Ticket 052 notes | Yes — expanded the passive from one bite roll into attack, tick, response, and completion order. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Knowledge register | Yes — bounded the slice to one solo non-classic dungeon monster. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Knowledge register | Yes — classified the entire action/bite/tick/response as one provider transition. |
| `PR-omarchy-gaming-system-separate-legacy-state-from-access-gates-001` | Ticket 052 recall | Yes — separated monster poison storage from Gnoll eligibility. |
| Ticket 052 completed evidence | nearest completed notes and OpenWiki | Yes — supplied the tested v5 factored combat/provider/cartridge baseline. |

## What happened

The separate Usurper workspace advanced to unadmitted rules and cartridge v6.
Every new encounter now owns an explicit false poison flag. A Gnoll's passive
bite is attempted at the source-relative point after ordinary attack power is
calculated, and an active poison tick resolves on the same and later offensive
turns before a living monster's response. Attack, configured Quick Heal,
Backstab, Soul Strike, and accepted level-one spells share that phase without
adding a fabricated racial command. Direct lethal attacks keep the prior
reward path; a later poison-lethal tick preserves the source's unusual lack of
an immediate XP/gold award.

The snapshot passed 39 Rust tests, Clippy, rustdoc, all source-provenance
checks, seventeen signed screens, trusted QML smoke, and the fifteen-case
provider TLS/replay/fault/callback corpus twice across restart. A complete
terminal security scan reported zero findings, fresh platform inspection
confirmed that the new state remains opaque provider JSON, and the full
24-stage platform gate passed. OpenWiki completed and the signed rules-v6
combat preview visibly showed the Gnoll bite, poison narration, and persistent
`Poisoned` status. Packaging, admission, commit, push, deployment, and
publication remained deferred.

## Failures captured

No new durable failure ID was needed. Source-order review found that a living
accepted spell still calculates and discards the ordinary attack before the
Gnoll bite attempt. The first composition would otherwise have put the bite
too early and shifted all later deterministic draws. The shared cast path was
corrected before final validation and is covered by exact draw-trace tests.

## Prevention rules captured

| ID | Rule |
|---|---|
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Preserve source calculations whose values are discarded when they consume RNG or position later draws; observable deterministic order is behavior even when the intermediate value is unused. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep Gnoll poison state, combat timing, and reward semantics inside the separate deterministic provider/cartridge; platform registration, shared realm, and packaging remain deferred. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new architecture decision was introduced.

## Effectiveness

Effective. Tracing the bite through initialization, the shared attack phase,
the later duration tick, monster response, and encounter completion prevented
a headline-roll-only port. The new discarded-RNG-work rule captures the less
obvious cast-order lesson. Deterministic unit fixtures, provider replay and
restart conformance, the zero-finding security review, full platform gate, and
visible signed preview jointly prove that the feature is source-linked,
replayable, player-private, and visible without expanding platform authority.
