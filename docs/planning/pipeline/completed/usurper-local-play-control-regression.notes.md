---
title: Usurper Local-play Control Regression — notes
pipeline_id: 963f0e95-1a0d-45e7-8519-1b6f2270188e
---

# Usurper Local-play Control Regression — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User report: visible buttons appeared duplicated and did not respond when the
  development application was loaded for review; workspace 8 is the requested
  test location.
- Initial live evidence:
  - Hyprland reported one visible `org.qt-project.qml` local-play window on
    workspace 8 and accepting compositor input;
  - the authenticated loopback session remained at revision 0 on `entry`;
  - its current signed plan contained three nodes and exactly one button,
    `entry_continue`, with no duplicate node ID, label, or action;
  - the desktop became locked before visual capture, so no input was injected
    into the password surface and no unlocked visual claim is made.
- Existing `scripts/test-local-play.sh` passed, but inspection found its QML
  smoke invokes `surface.smokeExercise()`, which directly calls `trigger()` and
  does not synthesize pointer input. It therefore cannot disprove the reported
  hit-testing failure.
- Recalled `BF-omarchy-gaming-system-qml-action-enablement-snapshot-001`,
  `BF-omarchy-gaming-system-cartridge-command-navigation-twins-001`, and
  `BF-omarchy-gaming-system-trusted-action-autorepeat-plan-crossing-001`; all
  affect this exact control lifecycle and must remain covered.
- Decision: pause Level 12 and repair/prove current interaction first. No Level
  12 pipeline artifacts or implementation had been created.

## Phase 2 — Design

- Architecture and data flow:
  - the external provider produces a player-private view and exact next screen;
    the signed presentation plus platform renderer compile that into one
    bounded `RenderedNode::Button` per declared choice;
  - `TrustedCartridgeSurface` independently validates the render plan, uses one
    `Repeater`/`Loader` delegate per node, and forwards node signals only while
    its action authority is enabled;
  - `CartridgeLocalPlay` disables the surface during HTTP work, enables it
    after the signed response is accepted, and binds the current revision,
    screen, action, and empty payload into one provider request;
  - provider success returns the next signed plan and advances the confirmed
    revision. Fixture preview intentionally stops before this boundary.
- Current evidence distinguishes layers: the live provider response has one
  unique button, so any current duplicate is QML materialization/presentation,
  not a duplicated provider command. Revision 0 establishes that no reported
  activation reached the provider during the observed session.
- CodeGraph traced `RenderedNode` and `compile_render_plan` to renderer callers
  and tests and found the Rust compiler blast radius confined to renderer and
  client-runtime consumers. It did not resolve the QML symbols, so the QML
  producer/consumer files were inspected directly as required for unsupported
  file types. Design receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `963f0e95-1a0d-45e7-8519-1b6f2270188e`, state hash
  `f493fc81f8a86518506f553ae5b4d24af09e04c0e6aab30646904dba139dd7a7`.
- API and compatibility: no serialized, HTTP, provider, cartridge, renderer,
  or game-state contract changes. A validated node ID will become the QML
  object's automation identity only; action payloads remain exact empty
  objects and fixture preview stays inert.
- Database and migration consequences: none.
- Exact file manifest:
  - `client/qml/cartridge/TrustedCartridgeSurface.qml` — give each loaded node
    a validated stable automation identity without adding a second delegate;
  - `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml` — synthesize
    real mouse and keyboard input across disabled/enabled and plan-replacement
    lifecycles and assert delegate/action cardinality;
  - `scripts/test-game-cartridge-renderer.sh` — execute the focused QML event
    suite inside the renderer gate;
  - `/srv/stacks/omarchygs_usurper/scripts/test-cartridge.sh` — include the same
    event-path proof in the external game's complete test entrypoint;
  - planning, architecture, OpenWiki, and AAR files — durable evidence and
    lifecycle reconciliation only.
- Regression table:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | Exact node/delegate count before and after two plan replacements; old automation IDs disappear. |
  | REQ-002 | Plan loads disabled, bound node later enables, and a synthesized center mouse click emits one action. |
  | REQ-003 | Mouse and Return each emit exactly one current action; provider-backed smoke confirms one revision. |
  | REQ-004 | Real Return event emits once; existing synthetic auto-repeat guard remains and is checked after replacement. |
  | REQ-005 | Existing title and action-authority assertions plus fixture and local smoke. |
  | REQ-006 | Platform/external gates, unlocked screenshot/readback when available, and fresh workspace-8 launch. |
- Risk and rollback review:
  - test automation must not create a bypass around `actionsEnabled`; assigning
    only `objectName` leaves action authority and hit-testing unchanged;
  - an old delegate surviving replacement could activate a stale action, so
    replacement checks require exact cardinality and absence of old IDs;
  - real pointer synthesis can be backend-sensitive, so it runs under the same
    deterministic offscreen software backend already used by renderer smoke;
  - no credentials, account/persona data, network expansion, concurrency
    primitive, reconnect behavior, database, or migration is introduced;
  - rollback is deletion of the added test identity/suite wiring; provider and
    cartridge compatibility are unchanged.
- Alternatives rejected:
  - another direct `trigger()` assertion would repeat the blind spot;
  - enabling the fixture viewer would misrepresent unconfirmed actions as
    gameplay;
  - changing provider actions or game rules cannot repair a QML-only failure
    when the signed live payload is already unique.
- Phase 2 exit: the event-path test and minimal QML identity change are
  actionable inside the approved manifest.

## Phase 3 — Implement

- Built:
  - assigned each loaded trusted node the stable validated automation identity
    `trusted-node-<signed id>` without changing its delegate, action, or
    enablement authority;
  - added a Qt Quick Test window that accepts bounded plans, asserts exact
    delegate counts, loads a button while disabled, enables it, sends a real
    center mouse click, sends a real Return event, replaces a two-button plan
    with one button, proves old automation identities disappear, and confirms
    exactly one current action with an empty payload;
  - wired the event suite into the platform renderer gate and the external
    Usurper cartridge gate using the Qt 6 test runner discovered from `qmake6`,
    avoiding the host's unrelated Qt 5 `/usr/bin/qmltestrunner`.
- During harness bring-up, the first mouse test had a zero-sized/invisible test
  parent and correctly emitted no action. Moving the surface into an explicit
  visible 920×640 `Window` made the test exercise real hit-testing. A
  one-millisecond event-loop turn after plan replacement waits for the new
  delegate connection and layout that a human click necessarily follows.
- Focused proof:
  - Qt 6 QML event suite: 5 passed, 0 failed;
  - `scripts/test-game-cartridge-renderer.sh`: 11 Rust renderer tests, the new
    QML event suite, all ready/non-ready rendering states, budget rejection,
    and renderer metrics passed;
  - external `scripts/test-cartridge.sh`: the QML event suite and all seventeen
    signed Usurper screens passed with unique visible labels;
  - external `scripts/test-local-play.sh`: provider-backed HTTP and trusted-QML
    smoke passed.
- No provider, game rule, cartridge, serialized state, HTTP schema, database,
  migration, packaging, admission, deployment, or publication behavior was
  changed.
- Phase 3 exit: the missing real-input regression coverage is implemented and
  focused checks pass.

## Phase 3.5 — Inspect

- Finding ledger:

  | # | Lens | Finding | Severity | Disposition |
  |---|---|---|---|---|
  | 1 | Control correctness | The live signed provider plan has one unique entry button and current QML creates one delegate per plan node; no current provider or delegate duplication was found. | None | Accepted; exact plan-replacement/cardinality test added. |
  | 2 | Pointer/keyboard behavior | Existing smoke bypassed hit-testing by calling `trigger()` directly. The new visible-window test proves real mouse and Return paths plus dynamic enablement. | Medium evidence gap | Fixed by Qt Quick Test coverage in both platform and external gates. |
  | 3 | Stale action/replacement | A replacement test must allow the QML event loop to attach the new delegate connection before simulating an impossible immediate click. | Low test correctness | Added one bounded event-loop turn, then proved old IDs absent and one new action. |
  | 4 | Input identity/security | `objectName` receives only the already signed, globally unique cartridge node ID. The verifier restricts IDs to 1–96 ASCII identifier bytes; QML object identity grants no action authority and the independent plan parser still bounds strings. | None | No security or authority expansion. |
  | 5 | Toolchain compatibility | `/usr/bin/qmltestrunner` is Qt 5 while the client is Qt 6. | Medium test portability | Resolve the matching Qt 6 runner from `qmake6 -query QT_INSTALL_BINS` in both gates. |
- Direct scope review found no drift from the Phase 2 manifest. The only
  runtime addition is an inert automation object name; all action behavior is
  the pre-existing dynamic enablement and repeat-safe path.
- QML visual policy, shell static analysis (excluding the script's pre-existing
  intentionally indirect source variable warning), whitespace checks, the
  complete renderer suite, external signed-screen suite, and local-provider
  smoke pass.
- Repeating the Qt Quick event suite with the runner's repeat mode remained
  green. No reportable authentication, authorization, secrets, privacy,
  network, abuse, persistence, concurrency, or state-integrity finding exists
  in this test-only authority-neutral delta.
- Fresh CodeGraph inspection retraced signed node lowering and renderer
  consumers; QML remained outside its symbol resolution and was reconciled by
  direct inspection. Inspect receipt:
  `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`, pipeline
  `963f0e95-1a0d-45e7-8519-1b6f2270188e`, state hash
  `b98d320e71aa7e8fbc683860dafd2a871f2220eb0ebb9005c8d03495199cc05f`.
- Phase 3.5 exit: the missing real-input proof is fixed, no confirmed finding
  remains, and the slice is ready for validation.

## Phase 4 — Validate

- Complete external game validation passed:
  `OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system ./scripts/test.sh`
  ran formatting, strict Clippy, 88 Rust tests, rustdoc, authenticated upstream
  hashes/provenance, the real Qt pointer/keyboard suite, all seventeen signed
  screens with unique labels, and provider-backed local play.
- `TMPDIR=/tmp bin/gate.sh --fast` passed every code, contract, renderer,
  package-source, architecture, secret, whitespace, hook, and module check and
  printed `GATE GREEN [fast]`.
- The first `TMPDIR=/tmp bin/gate.sh --diff` attempt exposed an environmental
  conflict: the unrelated system PostgreSQL already owned fixed
  `127.0.0.1:5432`. The system service was not stopped, modified, or supplied
  with test roles.
- The canonical diff gate was then run unchanged inside a temporary private
  user/network namespace. A user-owned Docker-socket relay and PostgreSQL
  Unix-socket relay gave the gate its expected loopback test endpoint without
  exposing a host port or touching the system database. The inherited
  `CARGO_TARGET_DIR` was unset so provider scripts built and executed the same
  repository artifact path. All 24 stages passed, including:
  - the five-case real Qt pointer/Return control suite;
  - two byte-identical native package builds;
  - 8 database-library, 66 server, 5 administrator, and 7 operator-CLI
    PostgreSQL tests;
  - 55 QML fixture assertions plus repeated live onboarding;
  - provider authority, compatibility-race, sidecar-operation, recovery,
    private-alpha, and server-module drills.
  It printed `GATE GREEN [diff]`; the gate receipt and inspected state both bind
  `b98d320e71aa7e8fbc683860dafd2a871f2220eb0ebb9005c8d03495199cc05f`.
  The isolated database and relays were removed afterward, while the host
  PostgreSQL remained active and unchanged.
- Replaced the two stale test-owned Usurper launchers with one fresh
  provider-backed development process. Hyprland readback shows exactly one
  mapped, visible, input-accepting `org.qt-project.qml` window titled
  `OmarchyGS Usurper Local Play — Development` on workspace 8. Its authenticated
  entry plan is revision 0 with three nodes, exactly one button, zero duplicate
  node IDs, and zero duplicate button labels.
- The desktop remains locked, so no input was injected into the password
  surface and no manual unlocked-click claim is made. The current window is
  ready in workspace 8 for the user's direct check after unlock. Behavioral
  acceptance instead comes from the visible-window Qt Quick test's real mouse
  and key events, exact delegate count, and one-action assertions, together
  with the live provider's unique current plan and revision checks.
- Phase 4 exit: every focused and external suite passes, a matching canonical
  diff-gate receipt exists, and the current repaired local-play window remains
  mapped on workspace 8 without expanding ticket scope.

## Phase 5 — Complete

- OpenWiki update run `23be4a7f-d068-4367-aa5a-7de2a2958bbf` completed after
  reconciling the engineering quickstart and game-cartridge architecture with
  the real-input lifecycle suite. Its finalizer reported only pre-existing
  unresolved evidence-debt warnings for older large-page claims; the run
  itself completed and removed its temporary plan.
- Acceptance audit:

  | Requirement | Result | Concrete evidence |
  |---|---|---|
  | REQ-001 | Satisfied | The Qt Quick suite asserts the initial and replacement delegate cardinality, unique automation identities, disappearance of both old identities, and exactly one remaining action. The live entry plan independently contains one unique button. |
  | REQ-002 | Satisfied | A button is materialized with `actionsEnabled: false`, receives no disabled click, follows the surface to enabled, and emits exactly one action from a synthesized center mouse click without reloading. |
  | REQ-003 | Satisfied | Real mouse and Return events each emit one exact current empty-payload action; provider-backed local play and the external suite confirm one expected provider revision per accepted action. |
  | REQ-004 | Satisfied | Real Return input, existing auto-repeat suppression, and the replacement test prove a held/stale activation cannot target the new control. |
  | REQ-005 | Satisfied | Fixture title/action-authority checks keep signed preview controls inert, while the development local-play title and provider-backed smoke retain interactive authority. |
  | REQ-006 | Satisfied | Hyprland reports one mapped, visible local-play window on workspace 8; its current provider plan is unique; real-input visible-window automation passes; external/full diff gates are green; scope review found no Level 12, admission, deployment, commit, push, or publication. The desktop lock prevented a separate manual click-through, so none is claimed. |
- Durable learning records:
  - `BF-omarchy-gaming-system-qml-direct-trigger-hit-testing-blind-spot-001`;
  - `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`.
- Hand-maintained game-cartridge architecture and the knowledge register were
  reconciled with the same evidence.
- The post-documentation `bin/gate.sh --diff` passed all 24 stages and printed
  `GATE GREEN [diff]`. The gate and OpenWiki completion receipts both bind
  pipeline `963f0e95-1a0d-45e7-8519-1b6f2270188e` to gated state
  `825b00a007df59a9849c4400f2f3a4fe3833ec60c344b7a427d7a4934e486879`.
  The temporary isolated database/relays were removed and the unrelated host
  PostgreSQL remained active and unchanged.
- AAR 065 is submitted and effective, every new durable ID appears in the
  knowledge register, every acceptance criterion is satisfied, and Ticket 065
  is ready for closed/completed archival.
