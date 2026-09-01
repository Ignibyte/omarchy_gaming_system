---
aar: AAR-058-usurper-level-six-dungeon-band
ticket: TICKET-058
pipeline: usurper-level-six-dungeon-band
status: submitted
opened: 2026-08-31
submitted: 2026-09-01
effectiveness: effective
---

# AAR-058-usurper-level-six-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | The encounter result set alone hides rejected RNG draws and unreachable boundary records. | Yes — exact row 50 remains stored while normal Level 6 selection accepts only 51–59. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Level bands must compose with the already declared solo non-classic combat mode. | Yes — no events, teams, special areas, or shared realm entered the slice. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Rejected monster candidates are discarded but still advance deterministic state. | Yes — every rejected `Random(60)` result remains observable in deterministic traces. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Earlier encounter draws can change later retreat/death phases. | Yes — replaying the full profile found and corrected its shifted death/re-entry sequence. |
| `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001` | Provider conformance accepts optional private database credentials. | Yes — the descriptor-owned credential boundary passed the full external and platform gates. |
| Ticket 057 | Supplies the current rules-v10 levels-one-through-five combat and visible cartridge baseline. | Yes — Level 6 extended the generic band path without changing platform authority. |

## What happened

The separate Usurper workspace advanced to unadmitted rules, state, and
cartridge v11. It stores the exact Level 6 editor rows 50–59, allows draw-free
switching across levels one through six, preserves every `Random(60)` rejection
until records 51–59 are selected, and initializes the selected monster at
strength 16, defence 8, and 48 HP. Existing attack, retreat, potion, spell,
class-special, reward, and Gnoll poison behavior composes through the same
deterministic reducer and provider boundary.

The external static gate passed 61 Rust tests plus Clippy, rustdoc, immutable
upstream/provenance checks, privacy scanning, all seventeen signed screens, and
trusted-QML smoke. The live provider passed the fixed fifteen-case TLS,
authentication, replay, fault, callback, and reconciliation corpus twice across
restart. A complete security scan reported zero findings, the 24-stage platform
gate passed twice around the handoff notes, and the signed Level 6 combat
preview visibly projected Lister at 48 HP through production QML. OpenWiki and
the hand-maintained architecture were reconciled before the ticket and pipeline
were archived. The user separately authorized a private pre-rebuild checkpoint
delivery; this did not authorize production admission or public release.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| No new ID | The first Level 6 live profile inherited a death assertion that no longer matched after the added rejection draws. | Full provider command profile. |
| No new ID | Adding the sixth dungeon label crossed Clippy's 100-line function limit. | Full external warning-denying gate. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Preserve stored boundary records and prove normal reachability separately. | Exact source data contains record 50 even though the normal Level 6 loop cannot select it. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Preserve rejected or discarded RNG work in deterministic state and tests. | Rejected 0–50 candidates shift every later draw and live command outcome. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Replay the complete live driver after earlier RNG work changes. | The Level 6 rejection sequence changed the later retreat/death/re-entry path. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep Level 6 data, rejection RNG, combat, state, and presentation inside the separate deterministic provider/cartridge; OmarchyGS continues transporting only opaque bounded action/state/view data. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new knowledge ID was introduced.

## Effectiveness

Effective. The recalled source-branch and discarded-RNG rules preserved the
normally unreachable boundary record and every rejection draw. The
composite-driver rule exposed the shifted live death path before completion.
Exact data/trace tests, provider replay across restart, the zero-finding
security scan, the full platform gate, and the signed visible Level 6 screen
jointly prove a source-linked, deterministic, bounded six-level slice without
expanding platform gameplay authority.
