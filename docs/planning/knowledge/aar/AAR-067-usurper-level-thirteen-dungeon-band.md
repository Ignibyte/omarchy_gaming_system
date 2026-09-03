---
aar: AAR-067-usurper-level-thirteen-dungeon-band
ticket: TICKET-067
pipeline: usurper-level-thirteen-dungeon-band
status: submitted
opened: 2026-09-02
submitted: 2026-09-02
effectiveness: effective
---

# AAR-067-usurper-level-thirteen-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Ticket 066 / `AAR-066` | Supplies rules/state/cartridge v17, levels one through twelve, and the repaired real-input boundary. | Yes; Level 13 reused the generic band and input path without a reducer or renderer fork. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Level 13 rows alone do not establish ordinary selection and event semantics. | Yes; source review kept events excluded and distinguished stored record 120 from normally reachable records 121–129. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Boundary record 120 must remain stored yet normally unreachable. | Yes; the deterministic trace retains the rejected `Random(130)` result before accepting the next draw. |
| `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001` | A thirteenth level must not recreate same-label command/navigation twins. | Yes; all seventeen signed screens and the live plans have unique button IDs, labels, and actions. |
| `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001` | A new visible control must stay covered by actual Qt input rather than direct calls. | Yes; offscreen Qt tests and workspace-8 Tab/Return input both advanced exactly one provider revision. |

## What happened

- Implemented external Usurper rules/state/cartridge v18 with exact Level 13
  records 120–129, draw-free levels 1–13 switching, forced rejection-loop
  evidence, source-derived combat/retreat behavior, `option_m`, one fixed
  provider action, signed presentation, fixtures, provenance, docs, and tests.
- The user's duplicate/inert-control report was not reproducible against the
  current plan or delegate tree. The investigation still exposed a useful
  evidence gap: the existing regression asserted model/repeater count but did
  not recursively count the instantiated trusted nodes across a realistic
  10-button to 11-button screen replacement. That direct assertion is now in
  the platform renderer gate.
- Full external tests, provider restart conformance, the 24-stage platform
  diff gate, OpenWiki, and live workspace-8 play passed. The v18 preview remains
  open at a signed Level 13 Big Bad Wolf encounter.

## Failures captured

- The first full external validation failed strict Clippy because the expanded
  sequential level-switch test exceeded the 100-line function ceiling. It was
  refactored into a table-driven loop and rerun successfully without a lint
  suppression.
- Early final-gate isolation attempts used a nonstandard temporary directory,
  omitted the isolated Compose routing, or inherited
  `CARGO_TARGET_DIR=/mnt/fast/target`. The last setting redirected the
  deliberately clean-clone Door Legends binary away from the path asserted by
  the authority drill. No product source changed; a focused rerun and the
  complete 24-stage gate passed after normalizing the harness environment.
- No duplicate or inert current control failure was confirmed. Current signed
  plans, actual QML delegate counts, screenshots, and real input all showed one
  control and one revision per accepted action, so no speculative production
  renderer change was made.

## Prevention rules captured

| ID | Rule |
|---|---|
| `PR-omarchy-gaming-system-count-instantiated-delegates-across-plan-replacement-001` | For data-driven QML controls, recursively assert that actual instantiated trusted delegates equal the current plan cardinality before and after a realistic large-screen replacement; model or repeater counts alone do not prove stale or duplicate visual delegates are absent. |

## Architecture decisions

No new ADR was required. ADR-0002's authority boundary held: the external
provider owns rules/state/revisions, while OmarchyGS authenticates and renders
the inert plan. The only platform implementation change is regression evidence;
the durable architecture prose was reconciled through Level 13.

## Effectiveness

Effective. All recalled rules directly shaped source review, deterministic RNG
evidence, the single-action presentation, and the real-input validation. The
new delegate-cardinality rule closes the specific evidence gap highlighted by
the user's live report without widening production behavior or authority.
