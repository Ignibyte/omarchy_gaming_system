---
aar: AAR-074-usurper-level-twenty-dungeon-band
ticket: TICKET-074
pipeline: usurper-level-twenty-dungeon-band
status: submitted
opened: 2026-09-03
submitted: 2026-09-03
effectiveness: effective
---

# AAR-074-usurper-level-twenty-dungeon-band

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Ticket 073 / `AAR-073` | Supplies rules/state/cartridge v24, levels one through nineteen, bounded-trace evidence, and real activation of all twenty-two controls. | Yes; Level 20 reused the generic external reducer, provider, signed cartridge, and real-input path while widening only bounded data and fixtures. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Level 20 rows alone do not establish ordinary selection and event semantics. | Yes; source review kept events separate, retained record 190 as data, and limited normal encounters to records 191–199. |
| `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` | Boundary record 190 must remain stored yet normally unreachable. | Yes; all rejected `Random(200)` values remain in the deterministic trace. |
| `PR-omarchy-gaming-system-size-rejection-traces-against-valid-tail-risk-001` | The fixed-cap trace must be reconsidered as the acceptance tail narrows. | Yes; Level 20 has a quantified exhaustion probability, an exact 256-draw valid run, and a maximum serialized-state proof. |
| `PR-omarchy-gaming-system-bind-loader-row-to-loaded-item-geometry-001` and the trusted-QML real-input/cardinality rules | The user's inert/duplicate report makes the twenty-third control surface a direct regression boundary. | Yes; the Loader height is explicit, the regression replaces 22 controls with 23 non-overlapping delegates, and every current control produces exactly one pointer and Return action. |
| `PR-omarchy-gaming-system-exercise-live-shell-across-provider-screen-transitions-001` | A one-action local-play smoke cannot prove later controls survive plan replacement. | Yes; the provider-backed smoke now confirms seven actions across entry, creation, street, dungeon, and combat screens. |

## What happened

- Implemented external rules/state/cartridge v25 with exact Level 20 records
  190–199, draw-free switching across levels 1–20, preserved rejection draws,
  strength 24/defence 12/72 HP encounters, Level 20 retreat behavior, bounded
  `option_t`, one fixed provider action, signed presentation, fixtures,
  provenance, compatibility documentation, and full tests.
- Ratcheted the trusted-QML largest-plan regression from twenty-two to
  twenty-three current controls. It proves loaded delegate cardinality,
  positive non-overlapping rows, stale removal, dynamic enablement, and one
  surface pointer plus Return activation per current control. The live local
  play harness additionally refuses to act unless exactly one current node
  owns the requested action and now crosses seven provider-confirmed plan
  replacements.
- The user's duplicate/inert report reproduced the existing Loader-row failure
  mode in an old preview. The clean v25 process on workspace 8 shows one
  separated row per race choice and reached provider revision 1 after the
  visible Continue button, confirming both rendering and input against a real
  compositor output.
- Codex Security found one low-severity local-development secret exposure:
  process argv disclosed the loopback bearer and endpoint to another UID. The
  launcher now hands QML only a validated private startup-file path and gives
  curl a mode-0600 private config; the repeated cross-UID probe cannot recover
  the secret or read the file.
- External validation passed 136 Rust tests, strict lint/docs, authenticated
  source/provenance, six QML cases, seventeen signed screens, expanded local
  play, and the complete provider restart corpus. The isolated platform diff
  gate passed all twenty-four stages before completion documentation and again
  after OpenWiki finalization; the final gate and completion receipts match.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-local-play-capability-argv-exposure-001` | The development launcher copied its ephemeral loopback capability into curl and long-lived QML command arguments, allowing a different local UID to recover and use it through procfs. | Codex Security inspection and a cross-UID read/mutation reproduction. |
| `BF-omarchy-gaming-system-placeholder-output-preview-evidence-001` | Launching QML while Hyprlock withheld Wayland outputs caused Qt to create a placeholder screen, making that process invalid visual or pointer evidence. | The first workspace-8 v25 launch and Qt runtime diagnostics. |

No new duplicate-control ID was needed. The observed overlapping rows matched
`BF-omarchy-gaming-system-implicit-qml-loader-row-overlap-001`, and its existing
prevention rule directly required the explicit Loader height and settled
non-overlap regression.

No new database-fixture ID was needed. The first final gate invocation omitted
the compose override inside its namespace and hit the existing host-port
collision class; the exact disposable resources were removed, the override was
propagated, and the complete rerun passed.

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-require-real-compositor-output-for-preview-evidence-001` | Accept desktop preview visual or pointer evidence only from a process launched after the compositor exposes a real output; retire any process that reports placeholder output. | A placeholder screen can paint or route input differently from the user's actual workspace and produced misleading duplicate/inert observations. |

The existing
`PR-omarchy-gaming-system-protect-test-secret-file-handoffs-001` remains the
durable prevention for the argv finding: development authority belongs in a
private file, never process metadata.

## Architecture decisions

No new ADR was required. ADR-0002's authority boundary held: the separate
provider owns Usurper data, rules, state, RNG, and revisions, while OmarchyGS
changes are game-neutral trusted-renderer and development-test safeguards.
The 256-draw development trace still fits the provider envelope at Level 20,
with its remaining tail risk explicit.

## Effectiveness

Effective. Every recalled rule influenced source selection, rejection traces,
provider transitions, state sizing, or QML evidence. Treating the user's live
report as release-blocking exposed both an already-known Loader lifecycle risk
and a distinct invalid compositor-evidence path; the clean workspace-8 window,
twenty-three-control input regression, and seven-action provider lifecycle now
cover them directly. Full external, security, restart, OpenWiki, and isolated
platform-gate evidence completed without committing, publishing, deploying, or
admitting the development cartridge.
