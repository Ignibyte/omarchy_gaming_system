---
aar: AAR-064-usurper-level-eleven-dungeon-band
ticket: TICKET-064
pipeline: usurper-level-eleven-dungeon-band
status: submitted
opened: 2026-09-02
submitted: 2026-09-02
effectiveness: effective
---

# AAR-064-usurper-level-eleven-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `BUL-002-pre-rebuild-delivery-handoff` | The source corpus, provider kit, preview state, database state, and receipts are intentionally local/ignored. | Yes — exact local identities were retained and no registration, admission, delivery, or publication occurred. |
| Ticket 063 / `AAR-063` | Supplies rules-v15 levels one through ten and the reconciled provider-backed Level 10 trace. | Yes — Level 11 extended the same generic band and preserved the unique-control and activation guards. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Level 11 rows alone do not prove enclosing selection/event semantics. | Yes — editor rows, ordinary selection, event separation, HP, and retreat branches were authenticated before translation. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Boundary record 100 must remain stored yet normally unreachable. | Yes — exact tests retain rejected `Random(110)` draws and make only records 101–109 normally reachable. |
| `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Level 11 may change later HP/phase outcomes in the live corpus. | Yes — it directly corrected the stale second-combat command after the Level 11 trace ended in death. |

## What happened

The separate Usurper workspace advanced to unadmitted rules, state, and
cartridge v16. It stores exact v0.20e Level 11 editor records 100–109, supports
draw-free switching across levels one through eleven, preserves every
source-order `Random(110)` rejection draw, and initializes normally reachable
Level 11 monsters at strength 20, defence 10, and 60 HP. Existing combat,
healing, spell, class-special, reward, retreat, and Gnoll poison behavior stays
inside the same deterministic provider-owned reducer.

The signed cartridge adds exactly one Level 11 button over the existing bounded
view. Provider-backed trusted QML reached Level 11 combat with every expected
revision advancing once; all seventeen signed screens passed duplicate-label
rejection and keyboard auto-repeat suppression. The stale workspace-8 preview
was replaced with the current v16 application and left open there. The desktop
was locked during final visual inspection, so no input was sent into its
password prompt.

The final external suite passed all 88 Rust tests, strict lint/docs,
source/provenance and privacy checks, signed cartridge conformance, and local
play. The fifteen-case live provider corpus passed twice across restart on an
isolated loopback PostgreSQL instance. A complete security diff inspection
reported zero findings. The platform fast gate passed; the database-bearing
diff gate's code and deterministic package stages passed, but its database
drills could not claim the host's already occupied fixed port 5432. The
unrelated system PostgreSQL was deliberately left untouched. OpenWiki updated
the two relevant pages and completed with their pre-existing Claims debt
warning.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| No new ID | The default provider and platform database harnesses could not bind their fixed localhost port because an unrelated system PostgreSQL already owned 5432. | Initial provider corpus and final platform diff gate. |
| No new ID | The old live profile issued a second combat command after the exact Level 11 retreat had already killed the Cleric, producing a bounded 422. | First isolated v16 provider-corpus run. |
| No new ID | The eleventh fixed action pushed `decode_command` beyond the enforced 100-line Clippy limit. | Full external Clippy run. |
| No new ID | The workspace-8 desktop was locked, so blind input would have targeted a password prompt. | Visible final application inspection. |
| No new ID | OpenWiki completed but retained pre-existing unresolved Claims evidence debt on both broad pages. | Phase 5 OpenWiki finalization. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| Existing `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001` | Composite drivers must follow authenticated post-command phase before choosing the next command. | It exactly covers the Level 11 death and stale second-combat mismatch. |
| Existing `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001` | Keep one phase-valid provider command per visible choice and reject duplicate labels. | It prevented the reported doubled controls from recurring when adding Level 11. |
| Existing `PR-omarchy-gaming-system-reject-activation-autorepeat-across-plan-replacement-001` | Consume but ignore activation auto-repeat across asynchronous plan replacement. | It preserves one provider revision per held key or click-like activation. |
| No new ID | Respect unrelated local listeners and use an explicit isolated database endpoint rather than stopping host services to make a test pass. | Test isolation must not mutate system infrastructure outside the ticket. |
| No new ID | Treat a desktop lock screen as an input-safety boundary. | Window placement can be verified without risking credentials or unintended actions. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| No new ID | Reuse the established provider-owned reducer, signed inert presentation, existing `option_k`, render-before-commit local play, and game-neutral trusted controls. | No ADR needed; the accepted provider/cartridge boundary is unchanged. |

## Effectiveness

Effective. Authenticated source/control-flow recall preserved the exact Level
11 boundary record, roster, selection work, HP, and retreat semantics. The
reconciled live driver, unique-control and one-revision regressions, restarted
provider corpus, zero-finding security inspection, green fast platform gate,
and completed OpenWiki update jointly validate this slice without Level 12,
dungeon events, shared realm, platform gameplay authority, registration,
admission, deployment, commit, push, or publication. The full platform gate's
fixed-port database failures are explicitly environmental and do not overlap
the unchanged platform database surface.
