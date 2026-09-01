---
aar: AAR-052-usurper-assassin-backstab-and-paladin-soul-strike
ticket: TICKET-052
pipeline: usurper-assassin-backstab-and-paladin-soul-strike
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-052-usurper-assassin-backstab-and-paladin-soul-strike

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Knowledge register and Ticket 051 notes | Yes — expanded each special beyond its headline damage formula into preflight, normal attack, and response order. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Knowledge register | Yes — fixed the port to solo non-classic dungeon combat and excluded PvP/NPC variants. |
| `PR-omarchy-gaming-system-preserve-legacy-guards-before-safe-arithmetic-001` | Ticket 051 AAR | Yes — made Soul Strike's HP spend and conditional health/addiction checks explicit. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Knowledge register | Yes — classified each special plus normal strike and monster response as one command. |
| Ticket 051 completed evidence | nearest completed notes and OpenWiki | Yes — supplied the tested v4 combat/provider/cartridge baseline. |

## What happened

The separate Usurper workspace advanced to unadmitted rules and cartridge v5.
The durable player snapshot now preserves the original mental-health and
addiction defaults, while dungeon combat gives an armed Assassin Backstab and
an eligible Paladin variable-HP Soul Strike with the original preflight, draw,
ordinary-attack, and same-turn monster-response order. A single inert combat
action is routed from provider-owned current class state to the corresponding
typed reducer command; generic commands retain the full Soul Strike parameter.

The snapshot passed 34 Rust tests, Clippy, rustdoc, all 31 provenance links,
seventeen signed screens, trusted QML smoke, and the fifteen-case provider
security/replay corpus twice across restart. The live combat preview opened
through the production trusted-cartridge preview boundary. A complete 46-file
terminal security scan reported zero findings, fresh CodeGraph inspection
confirmed no platform contract change, the full 24-stage platform gate passed,
and the OpenWiki lifecycle completed. Packaging, production admission, commit,
push, deployment, and publication remained deferred.

## Failures captured

No new durable failure ID was needed. The first compile caught that the new
player-owned scalars had initially been placed in the reusable combat `Stats`
row instead of `Character`; moving them to the ownership named by the design
manifest corrected every initializer. Clippy then caught an oversized command
validator, and extracting the combat-special preflight restored the project
line budget without suppressing the lint. Both corrections occurred before
behavioral proof and are covered by existing manifest, ownership, and quality
gates.

## Prevention rules captured

No new prevention rule was needed. The recalled
`PR-omarchy-gaming-system-separate-legacy-state-from-access-gates-001` rule
kept mental/addiction storage independent from Soul Strike eligibility, while
`PR-omarchy-gaming-system-preserve-legacy-guards-before-safe-arithmetic-001`
kept the HP spend and conditional checks source ordered. The exact Phase 2 file
manifest plus compile and Clippy gates caught the two implementation-shape
mistakes before they could become behavior.

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep Backstab, Soul Strike, and their player-private source state inside the separate deterministic provider/cartridge; platform registration, shared realm, and packaging remain deferred. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new architecture decision was introduced.

## Effectiveness

Effective. Tracing both specials through their shared menu key, ordinary attack,
and response phase prevented headline-formula-only ports. The existing
state/access and guarded-arithmetic rules preserved source data even where the
current default skips failure checks. Provider generic/fixed equivalence,
restart/replay conformance, complete terminal security coverage, and the live
trusted preview jointly proved that the new behavior remains game-owned,
deterministic, replayable, and visible without expanding platform authority.
