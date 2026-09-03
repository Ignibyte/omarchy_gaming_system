---
aar: AAR-065-usurper-local-play-control-regression
ticket: TICKET-065
pipeline: usurper-local-play-control-regression
status: submitted
opened: 2026-09-02
submitted: 2026-09-02
effectiveness: effective
---

# AAR-065-usurper-local-play-control-regression

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Direct user report | The loaded development client showed doubled and inert buttons. | Yes — it identified the real visible/input acceptance boundary missed by direct-call smoke. |
| `BF-omarchy-gaming-system-qml-action-enablement-snapshot-001` | Controls can materialize while the surface is disabled during loading. | Yes — the real click test now begins disabled and proves the existing dynamic binding becomes live. |
| `BF-omarchy-gaming-system-cartridge-command-navigation-twins-001` | Provider and navigation controls previously duplicated visible choices. | Yes — live-plan and all-screen uniqueness checks distinguish provider command duplication from QML delegate duplication. |
| `BF-omarchy-gaming-system-trusted-action-autorepeat-plan-crossing-001` | Input can cross an asynchronous render-plan replacement. | Yes — the replacement test asserts old delegates disappear and only the new current action fires. |

## What happened

The provider-backed entry response already contained one signed button, but the
existing QML smoke called `trigger()` directly. That proved signal plumbing
while bypassing the pointer geometry, enabled state, hit-testing, focus, and
event-delivery path a player actually uses. A focused Qt Quick test now places
the trusted surface in a visible window, drives real mouse and Return events,
and replaces its plan while asserting exact delegate and action cardinality.

The runtime change itself is deliberately authority-neutral: each loaded node
receives an automation `objectName` derived from its already validated signed
ID. No provider, renderer, serialized state, or action contract changed.

## Failures captured

- `BF-omarchy-gaming-system-qml-direct-trigger-hit-testing-blind-spot-001`:
  a QML smoke that invokes a control method directly can pass while the
  user-visible pointer path is inert because geometry, hit-testing, focus, or
  dynamic enablement is broken.
- The first visible-window harness correctly failed to click while its test
  parent had zero usable geometry. Giving the test an explicit 920×640 window
  made the assertion cover the intended boundary instead of weakening it.
- The host PostgreSQL port conflicted with the full gate's test database. The
  final gate used a private network namespace and Unix-socket relay, preserving
  the host service unchanged while still running the canonical gate command.
- An inherited external `CARGO_TARGET_DIR` made two provider drills execute a
  stale repo-local binary. Unsetting it aligned the scripts' build and run
  paths for the final green gate; the earlier durable Cargo-path rule remains
  applicable.

## Prevention rules captured

- `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`: for an
  interactive QML regression, place the production control in a visible test
  window and synthesize real pointer/key input across enablement and plan
  replacement, asserting exact control and action cardinality.
- Retain direct method calls only as narrow unit evidence; they cannot stand in
  for player input acceptance.
- Preserve both signed-plan uniqueness and delegate uniqueness checks because
  they diagnose different ownership layers.

## Architecture decisions

No authority boundary changed. Fixture preview remains inert; provider-backed
local play remains the only development surface that can confirm mutations.
The trusted surface still admits one delegate per validated plan node and each
control follows the current surface action authority. Automation identity uses
only the bounded, validated signed node ID and grants no action capability.

## Effectiveness

Effective. The five-case Qt Quick suite passes through real mouse and Return
delivery, disabled-to-enabled transition, plan replacement, stale-delegate
absence, and exact one-action behavior. It runs in both the platform renderer
gate and external Usurper cartridge suite. The complete external suite, fast
gate, and all 24 canonical diff-gate stages pass. One mapped local-play window
with a unique signed entry action remains open on workspace 8; because the
desktop is locked, no separate manual unlocked click-through is claimed.
