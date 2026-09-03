---
aar: AAR-059-usurper-level-seven-dungeon-band
ticket: TICKET-059
pipeline: usurper-level-seven-dungeon-band
status: submitted
opened: 2026-09-02
submitted: 2026-09-02
effectiveness: effective
---

# AAR-059-usurper-level-seven-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `BUL-002-pre-rebuild-delivery-handoff` | The machine was rebuilt after Ticket 058 and ignored evidence had to be reconstructed. | Yes — both clean repositories, upstream corpus, provider kit, pipeline tools, PostgreSQL, and untracked-state boundaries were verified before planning. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | The encounter result set alone hides rejected RNG draws and unreachable boundary records. | Yes — Level 7 retains record 60 while normal selection accepts only 61–69. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Level bands must compose with the declared solo non-classic combat mode. | Yes — no events, teams, special areas, or shared realm entered the slice. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Rejected monster candidates still advance deterministic state. | Yes — every rejected `Random(70)` result remains observable in deterministic traces. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Earlier encounter draws can change later retreat/death phases. | Yes — the complete provider profile was replayed and its v12 Level 7 sequence passed twice across restart. |
| `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001` | Provider conformance accepts optional private database credentials. | Yes — descriptor-owned credentials remained intact through the live provider proof. |
| Ticket 058 | Supplies the rules-v11 levels-one-through-six combat and visible cartridge baseline. | Yes — Level 7 extended the generic band path without changing platform authority. |

## What happened

The separate Usurper workspace advanced to unadmitted rules, state, and
cartridge v12. It stores exact Level 7 editor records 60–69, permits draw-free
switching across levels one through seven, preserves every `Random(70)`
rejection until records 61–69 are selected, and initializes the selected
monster at strength 17, defence 8, and 51 HP. Existing attack, retreat, potion,
spell, class-special, reward, and Gnoll poison paths continue through the same
deterministic provider-owned reducer.

The final external suite passed 66 Rust tests plus Clippy, rustdoc, immutable
upstream/provenance and privacy checks, and all seventeen signed screens. The
live provider passed the fixed fifteen-case TLS, authentication, replay, fault,
callback, and reconciliation corpus twice across restart. A complete final
security snapshot reported zero findings. The full 24-stage platform gate was
green, and the signed Level 7 combat preview visibly showed an Orc at 51 HP
through production trusted QML on workspace 8. OpenWiki and the hand-maintained
architecture were reconciled before the ticket and pipeline were archived.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-cargo-target-directory-assumption-001` | The live provider script built v12 into the ambient Cargo target directory but launched a stale v11 binary from `<repo>/target`; the analogous existing Door Legends gate drill also failed when the ambient override was present. | First post-change live provider run and the first platform validation gate. |
| No new ID | Rebuilt-machine system PostgreSQL occupied host port 5432 while four existing platform drills assume that port, and hook self-test cleanup assumes `TMPDIR` resolves below `/tmp`. | Initial full platform gate. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-resolve-cargo-artifacts-from-metadata-001` | A script that executes a Cargo-built binary must resolve the exact manifest's `target_directory` through structured `cargo metadata`, validate the path, and invoke the quoted resolved artifact instead of assuming `<repo>/target`. | Ambient `CARGO_TARGET_DIR` otherwise lets a build pass while the script launches a missing or stale binary. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Preserve stored boundary records and prove normal reachability separately. | Exact source data contains record 60 even though the normal Level 7 loop cannot select it. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Preserve rejected or discarded RNG work in deterministic state and tests. | Rejected 0–60 candidates shift every later draw and live command outcome. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep Level 7 data, rejection RNG, combat, state, and presentation inside the separate deterministic provider/cartridge; OmarchyGS continues transporting only opaque bounded action/state/view data. | Existing Ticket 047 decision; consistent with ADR-0002. |

No new architecture decision was introduced.

## Effectiveness

Effective. Recalled legacy-branch and discarded-RNG rules preserved the normally
unreachable boundary row and every rejected draw. Full-profile replay proved
that Level 7 composes with prior combat features. The stale-binary failure
produced a reusable target-directory rule and a final harness that follows
Cargo's actual artifact location. Exact data/trace tests, provider replay,
zero-finding security review, the complete platform gate, and the visible
workspace-8 preview jointly prove a source-linked deterministic seven-level
slice without expanding platform gameplay authority.
