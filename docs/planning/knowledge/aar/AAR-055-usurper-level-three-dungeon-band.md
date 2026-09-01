---
aar: AAR-055-usurper-level-three-dungeon-band
ticket: TICKET-055
pipeline: usurper-level-three-dungeon-band
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-055-usurper-level-three-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | The encounter result set alone hides rejected RNG draws and unreachable boundary records. | Yes — retain exact row 20 while proving normal level-three selection accepts only 21–29. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Level bands must compose with the already declared solo non-classic combat mode. | Yes — prevents accidental import of events, teams, special areas, and shared realm. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Rejected monster candidates are discarded but still advance deterministic state. | Yes — preserve every rejected `Random(30)` result in the replay trace. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Earlier encounter draws can change later retreat/death phases. | Yes — reconcile the complete level-three provider profile rather than assuming Ticket 054's later phases. |
| Ticket 054 | Supplies the current rules-v7 levels-one/two combat and visible cartridge baseline. | Yes — level three can extend a proven band boundary without changing platform authority. |

## What happened

The separate Usurper workspace advanced to unadmitted rules and cartridge v8.
It now stores the ten exact level-three editor records, permits only dungeon
levels one through three, switches levels without consuming RNG, preserves the
original `Random(30)` rejection loop and normally unreachable record 20, and
initializes accepted records 21–29 to their source-backed level-three combat
state. Existing attacks, retreat, potions, caster spells, class specials,
rewards, and Gnoll poison compose through the same deterministic reducer and
provider boundary.

The fixed `enter_dungeon_level_3` action and existing generic level command are
equivalent, provider state/replay remains private and durable, and the signed
seventeen-screen cartridge visibly presents all three level controls through
trusted QML. The final external snapshot passed 49 Rust tests, Clippy, rustdoc,
pinned-source and 37-entry provenance checks, signed-cartridge/QML smoke, and
the fifteen-case TLS/replay/fault/callback corpus twice across restart in both
default and private-credential-file modes. A complete 46-file security scan
reported zero findings after credential-harness remediation, the full 24-stage
platform gate passed, and the signed rules-v8 Level 3 preview remained open.
Packaging, admission, commit, push, deployment, and publication remain
deferred.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| No new ID | Added level-three rejection draws shifted the later deterministic retreat from Ticket 054's death to success. | First focused provider run; fixed the stale driver to assert the source-faithful Dungeon then `main_street` path instead of bending the reducer. |
| No new ID | The test harness exposed PostgreSQL credentials through `psql`/`jq` arguments, an inherited uppercase URL environment variable, an ambient lowercase shell-variable collision, and the `sslpassword` query key. | Three successive security snapshots; fixed with a descriptor-validated private credential file, passfile-scoped subprocesses, a non-secret allowlist, explicit environment cleanup, and fail-closed unsupported keys. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Preserve stored boundary records and prove actual selection reachability separately. | The exact data contains record 20 even though the normal level-three loop cannot select it. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Preserve rejected/discarded RNG work in deterministic state and tests. | Rejected 0–20 candidates change every later draw and the live command outcome. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Reconcile full live command drivers after earlier RNG or phase work changes. | The level-three rejection loop changed a later retreat from death to a successful return. |
| `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001` | Validate private credential input through one no-follow descriptor with ownership, mode, link, bound, and stability checks. | It removed pathname and ambient-shell ambiguity while keeping the secret out of argv and child environments. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep level-three data, rejection RNG, combat, state, and presentation inside the separate deterministic provider/cartridge; OmarchyGS continues to transport only opaque bounded action/state/view data. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new knowledge ID was introduced.

## Effectiveness

Effective. The recalled legacy-port rules preserved both the unreachable source
record and every discarded draw, while the composite-driver rule caught the
downstream retreat change. Exact data and trace tests, provider replay/restart,
the clean final security snapshot, the full platform gate, and the signed
visible Level 3 screen jointly prove that the three-level slice is
source-linked, deterministic, bounded, and visible without expanding platform
authority.
