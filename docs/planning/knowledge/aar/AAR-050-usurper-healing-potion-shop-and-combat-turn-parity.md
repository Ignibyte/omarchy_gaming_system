---
aar: AAR-050-usurper-healing-potion-shop-and-combat-turn-parity
ticket: TICKET-050
pipeline: usurper-healing-potion-shop-and-combat-turn-parity
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-050-usurper-healing-potion-shop-and-combat-turn-parity

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` | Knowledge register and Ticket 049 notes | Yes — kept the slice inside player-private provider state. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Knowledge register | Yes — exposed distinct full-health and zero-potion combat branches. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Ticket 049 AAR | Yes — fixed the reviewed release to `QuaffOpt = 1` before composing healing and attack. |
| `PR-omarchy-gaming-system-verify-generated-provider-profiles-before-live-use-001` | Ticket 049 AAR | Yes — the v3 gameplay profile passed strict canonical input and the complete live corpus. |
| Ticket 049 completed evidence | nearest completed notes and OpenWiki | Yes — supplied the tested rules/provider/cartridge baseline. |

## What happened

The separate Usurper workspace advanced to an unadmitted rules-v3 slice. It
now exposes Merlin's Magic Shop, derives healing-potion price from level,
enforces the configured purchase ceiling, and preserves the original's
independent 150-potion creation balance. In combat, quick healing now consumes
no random draw and continues through the existing normal attack in the same
transition under configured quaff option 1. Full-health and wounded/no-potion
oddities remain distinct and tested.

The signed cartridge grew to seventeen inert screens and 71 declared actions;
the public Provider SDK/starter, separate PostgreSQL state, exact replay,
restart, and trusted platform-owned QML boundaries did not change. The full
external check, fifteen-case provider corpus twice across restart, OpenWiki
lifecycle, and 24-stage platform gate passed without production registration,
admission, shared-realm state, platform gameplay code, or publication.

The initial full-day integration rerun found that its loop still sent a
separate `Attack` immediately after `QuickHeal`. Because v3 intentionally makes
quick healing a composite heal-and-attack command, that stale driver sometimes
sent an invalid command after combat had already ended. Updating both day
drivers to select exactly one command per combat iteration fixed the failure.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-usurper-composite-quaff-double-attack-001` | The full-day driver unconditionally sent a separate attack after v3 quick healing had already attacked and could have ended combat. | First post-implementation workspace test run. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | When one accepted game command absorbs a previously separate follow-up transition, update every driver to choose one command per loop iteration and assert phase before issuing another command. | Composite command semantics invalidate orchestration assumptions even when the reducer itself is correct. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep the healing-potion economy and combat turn entirely inside the separate deterministic player-private provider/cartridge; shared realm and production admission remain deferred. | Existing Ticket 047 decision; consistent with ADR-0002 and ADR-0003. |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. The recalled branch/mode rules prevented normalization of the
source's 150-versus-75 potion oddity and caught the wounded/no-potion attack
branch. The topology rule kept the change out of platform gameplay code. The
new composite-command driver rule came directly from a real red integration
test, and the corrected drivers then passed all focused, live-provider,
trusted-presentation, and platform delivery evidence.
