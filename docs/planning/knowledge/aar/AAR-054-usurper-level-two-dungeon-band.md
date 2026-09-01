---
aar: AAR-054-usurper-level-two-dungeon-band
ticket: TICKET-054
pipeline: usurper-level-two-dungeon-band
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-054-usurper-level-two-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | The encounter result set alone hides rejected RNG draws and unreachable boundary records. | Yes — retained exact row 10 while proving normal level-two selection accepts only 11–19. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Level bands must compose with the already declared solo non-classic combat mode. | Yes — prevented accidental import of events, teams, special areas, and shared realm. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Rejected monster candidates are discarded but still advance deterministic state. | Yes — kept every rejected `Random(20)` result in the replay trace. |
| Ticket 053 | Supplies the current rules-v6 combat and visible cartridge baseline. | Yes — let level two compose with existing attack, spells, specials, potions, rewards, poison, and retreat. |

## What happened

The separate Usurper workspace advanced to unadmitted rules and cartridge v7.
It now stores the ten exact level-two editor records, permits only dungeon
levels one and two, changes levels without consuming RNG, preserves the
original rejection loop and unreachable boundary record, initializes accepted
level-two monsters from source-backed values, and composes the existing combat
subsystems with a level-aware retreat bound. Fixed provider actions and the
signed dungeon screen expose both levels through the unchanged public provider
and trusted-renderer protocols.

The final snapshot passed 45 Rust tests, Clippy, rustdoc, all pinned-source and
36-entry provenance checks, seventeen signed screens, trusted QML smoke, and
the fifteen-case TLS/replay/fault/callback corpus twice across restart. Two
complete parentless-repository security snapshots reported zero findings; the
second sealed the exact final lint-refactored tree. Fresh platform inspection
confirmed the new state remains opaque provider JSON, and the full 24-stage
platform gate passed. A signed rules-v7 dungeon preview visibly shows the
level-two descent and both level controls. Packaging, admission, commit, push,
deployment, and publication remain deferred.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| No new ID | Added rejected encounter draws shifted the later deterministic retreat, killed the Cleric, and made the inherited live driver issue `main_street` from `Dead`. | First v7 live provider conformance run returned 422; fixed by asserting the exact death/re-entry profile in a permanent provider test. |
| No new ID | `reduce` crossed the 100-line lint ceiling after the new level branch. | First complete `scripts/test.sh`; fixed by extracting the already validated accepted mutation into `enter_dungeon`. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Preserve stored boundary records and prove actual selection reachability separately. | The exact data contains record 10 even though the normal level-two loop cannot select it. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Preserve rejected/discarded RNG work in deterministic state and tests. | Rejected 0–10 candidates change every later draw and the live command outcome. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Reconcile full live command drivers after earlier RNG or phase work changes. | The level-two rejection loop changed a later retreat into a deterministic death requiring `reenter`. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep dungeon depth, monster catalogs, encounter RNG, combat, and presentation inside the separate deterministic provider/cartridge; platform rule copies and shared realm remain deferred. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new knowledge ID was introduced.

## Effectiveness

Effective. The three recalled rules prevented a visually plausible but
source-inaccurate nine-row sampler, preserved every rejection draw, and caught
the downstream live-driver phase change. Exact roster and trace tests,
provider replay/restart, a zero-finding final security snapshot, the full
platform gate, and the signed visible dungeon screen jointly prove that the
level-two slice is source-linked, deterministic, bounded, and visible without
expanding platform authority.
