---
aar: AAR-056-usurper-level-four-dungeon-band
ticket: TICKET-056
pipeline: usurper-level-four-dungeon-band
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-056-usurper-level-four-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | The encounter result set alone hides rejected RNG draws and unreachable boundary records. | Yes — exact record 30 remains stored while normal Level 4 selection accepts only 31–39. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Level bands must compose with the already declared solo non-classic combat mode. | Yes — prevented accidental import of events, teams, special areas, and shared realm. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Rejected monster candidates are discarded but still advance deterministic state. | Yes — every rejected `Random(40)` result remains in the replay trace. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Earlier encounter draws can change later retreat/death phases. | Yes — the complete Level 4 provider profile was reconciled after its later retreat became death. |
| `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001` | Provider conformance accepts optional private database credentials. | Yes — the hardened descriptor-owned credential boundary remained intact. |
| Ticket 055 | Supplies the rules-v8 levels-one-through-three combat and visible cartridge baseline. | Yes — Level 4 extended the generic band path without changing platform authority. |

## What happened

The separate Usurper workspace advanced to unadmitted rules and cartridge v9.
It now stores the ten exact Level 4 editor records, permits only dungeon levels
one through four, switches levels without consuming RNG, preserves the original
`Random(40)` rejection loop and normally unreachable record 30, and initializes
accepted records 31–39 at strength 14, defence 7, and 42 HP. Existing attack,
retreat, potion, caster spell, class-special, reward, and Gnoll poison behavior
continues through the same deterministic reducer and provider boundary.

The fixed `enter_dungeon_level_4` action and generic level command are
equivalent, provider state and replay remain private and durable, and the signed
seventeen-screen cartridge visibly presents all four level controls through
trusted QML. The final external snapshot passed 53 Rust tests, Clippy, rustdoc,
pinned-source and 38-entry provenance checks, signed-cartridge/QML smoke, and
the fifteen-case TLS/replay/fault/callback corpus twice across restart. A full
46-file security scan reported zero findings, the complete 24-stage platform
gate passed, and the signed rules-v9 Level 4 preview remained open. Packaging,
admission, commit, push, deployment, and publication remain deferred.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| No new ID | Added Level 4 rejection draws shifted the later deterministic retreat from Ticket 055's success to death. | First provider profile run; retained the source-faithful death and added `reenter` instead of bending the reducer. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Preserve stored boundary records and prove actual selection reachability separately. | Exact source data contains record 30 even though the normal Level 4 loop cannot select it. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Preserve rejected or discarded RNG work in deterministic state and tests. | Rejected 0–30 candidates change every later draw and the live command outcome. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Reconcile full live command drivers after earlier RNG or phase work changes. | The Level 4 rejection loop changed a later retreat from success to death. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep Level 4 data, rejection RNG, combat, state, and presentation inside the separate deterministic provider/cartridge; OmarchyGS continues to transport only opaque bounded action/state/view data. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new knowledge ID was introduced.

## Effectiveness

Effective. The recalled legacy-port rules preserved the normally unreachable
source record and every discarded draw, while the composite-driver rule caught
the downstream retreat change. Exact data and trace tests, provider
replay/restart, the clean security snapshot, the full platform gate, and the
signed visible Level 4 screen jointly prove that the four-level slice is
source-linked, deterministic, bounded, and visible without expanding platform
authority.
