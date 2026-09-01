---
aar: AAR-051-usurper-level-one-spellcasting-and-mana-loop
ticket: TICKET-051
pipeline: usurper-level-one-spellcasting-and-mana-loop
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-051-usurper-level-one-spellcasting-and-mana-loop

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` | Knowledge register and Ticket 050 notes | Yes — kept spell/mana state in the player-private provider snapshot. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Knowledge register | Yes — expanded the trace from spell effect lines through resistance, turn response, and encounter reset. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Knowledge register | Yes — bounded the slice to player level-one spells and deferred adjacent magic systems. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Ticket 050 AAR | Yes — identified cast-plus-monster-response as one combat command before implementation. |
| Ticket 050 completed evidence | nearest completed notes and OpenWiki | Yes — supplied the tested v3 rules/provider/cartridge baseline. |

## What happened

The separate Usurper workspace advanced to unadmitted rules and cartridge v4.
Character state now preserves all twelve learned/active flags and exact caster
mana; combat exposes Cure Light, Magic Missile, or Fog of War with the original
ten-mana cost, resistance mapping, RNG order, turn replacement, living-monster
response, temporary absorption, encounter reset, and daily mana refill. One
fixed inert combat action reaches the same generic ordinal command through the
existing public Provider SDK/starter, while the platform remains the sole QML
and session-envelope authority.

New invariants caught an older Singuman translation bug: `saturating_sub(5)`
turned a zero noncaster mana gain into negative five even though Pascal guards
the decrement with `cp > 0`. Inspection then found that creation incorrectly
coupled the stored first-spell flag to caster eligibility. Both were corrected
to preserve source state and branch conditions independently of access gates.

The corrected snapshot passed 30 Rust tests, Clippy, rustdoc, upstream
provenance, all seventeen signed screens, trusted QML smoke, and the live
fifteen-case provider corpus twice across restart. A complete 46-file terminal
security scan reported zero findings after the hosted scanner could not resolve
this new repository's absent `HEAD`. The full 24-stage platform gate and
OpenWiki lifecycle also passed. Packaging, production admission, commit, push,
deployment, and publication remained deferred.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-usurper-guarded-saturating-sub-drift-001` | A saturating subtraction translated a guarded Pascal decrement and changed a zero noncaster mana gain into negative five. | New nonnegative mana invariant during implementation. |
| `BF-omarchy-gaming-system-usurper-access-gate-state-conflation-001` | Character creation stored spell 1 only for caster classes even though the original stores the flag for every class and gates use separately. | Phase 3.5 source-fidelity inspection. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-preserve-legacy-guards-before-safe-arithmetic-001` | Translate the source branch guard before applying checked or saturating arithmetic; safer arithmetic must not make an originally skipped adjustment execute. | Arithmetic safety and source fidelity are separate obligations. |
| `PR-omarchy-gaming-system-separate-legacy-state-from-access-gates-001` | Preserve legacy stored flags independently from menu, class, phase, and capability gates unless the source explicitly couples them. | Equivalent current access can hide incompatible durable state needed by later rules or migrations. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep level-one spell rules and private combat state inside the separate deterministic provider/cartridge; shared realm and production admission remain deferred. | Existing Ticket 047 decision; consistent with ADR-0002. |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. The recalled branch-scope and provider-topology rules kept the slice
bounded and exposed the exact creation/cast/response/reset sequence before code
changed. The new invariants found a pre-existing guarded-arithmetic error; the
fresh inspection found the stored-state/access conflation before completion.
The provider-profile and composite-command rules kept live restart/replay and
one-command-per-turn evidence intact. The terminal security fallback accounted
for every file despite the new repository having no commit baseline, and the
OpenWiki plus platform gates confirmed the game added no platform authority.
