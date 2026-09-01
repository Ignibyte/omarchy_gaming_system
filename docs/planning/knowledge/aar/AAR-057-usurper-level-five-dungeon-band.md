---
aar: AAR-057-usurper-level-five-dungeon-band
ticket: TICKET-057
pipeline: usurper-level-five-dungeon-band
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-057-usurper-level-five-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | The encounter result set alone hides rejected RNG draws and unreachable boundary records. | Yes — exact record 40 remains stored while normal Level 5 selection accepts only 41–49. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Level bands must compose with the already declared solo non-classic combat mode. | Yes — prevented accidental import of events, teams, special areas, and shared realm. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Rejected monster candidates are discarded but still advance deterministic state. | Yes — every rejected `Random(50)` result remains in the replay trace. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Earlier encounter draws can change later retreat/death phases. | Yes — the complete Level 5 provider profile was reconciled and retained its source-faithful death/re-entry path. |
| `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001` | Provider conformance accepts optional private database credentials. | Yes — the hardened descriptor-owned credential boundary remained intact. |
| Ticket 056 | Supplies the rules-v9 levels-one-through-four combat and visible cartridge baseline. | Yes — Level 5 extended the generic band path without changing platform authority. |

## What happened

The separate Usurper workspace advanced to unadmitted rules and cartridge v10.
It now stores the ten exact Level 5 editor records, permits only dungeon levels
one through five, switches levels without consuming RNG, preserves the original
`Random(50)` rejection loop and normally unreachable record 40, and initializes
accepted records 41–49 at strength 15, defence 7, and 45 HP. Existing attack,
retreat, potion, caster spell, class-special, reward, and Gnoll poison behavior
continues through the same deterministic reducer and provider boundary.

The fixed `enter_dungeon_level_5` action and generic level command are
equivalent, provider state and replay remain private and durable, and the signed
seventeen-screen cartridge visibly presents all five level controls through
trusted QML. The final external snapshot passed 57 Rust tests, Clippy, rustdoc,
pinned-source and 39-entry provenance checks, signed-cartridge/QML smoke, and
the fifteen-case TLS/replay/fault/callback corpus twice across restart. A full
46-file security scan reported zero findings, the complete 24-stage platform
gate passed, and the signed rules-v10 Level 5 preview remains open. Packaging,
admission, commit, push, deployment, and publication remain deferred.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| No new ID | No implementation or validation failure occurred; the additional rejection draws retained the previously reconciled death/re-entry profile. | Full rules/provider suites and visible preview. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Preserve stored boundary records and prove actual selection reachability separately. | Exact source data contains record 40 even though the normal Level 5 loop cannot select it. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Preserve rejected or discarded RNG work in deterministic state and tests. | Rejected 0–40 candidates change every later draw and live command outcome. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Reconcile full live command drivers after earlier RNG or phase work changes. | Level-band extension must not assume later retreat/re-entry phases from an earlier tape. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep Level 5 data, rejection RNG, combat, state, and presentation inside the separate deterministic provider/cartridge; OmarchyGS continues to transport only opaque bounded action/state/view data. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new knowledge ID was introduced.

## Effectiveness

Effective. The recalled legacy-port rules preserved the normally unreachable
source record and every discarded draw, while the composite-driver rule kept
the full live profile explicit. Exact data and trace tests, provider
replay/restart, the clean security snapshot, the full platform gate, and the
signed visible Level 5 screen jointly prove that the five-level slice is
source-linked, deterministic, bounded, and visible without expanding platform
authority.
