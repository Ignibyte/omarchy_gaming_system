---
title: Explicit QML application exit control — notes
pipeline_id: bc70b184-3b03-44f0-9435-0b263117cede
---

# Explicit QML application exit control — completion notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: no active bulletin or pipeline blocked work; `main` was clean and
  matched `origin/main` at `d7758d9` before the slice. Ticket 026 was next.
- Recall: CodeGraph 1.5.0 and OpenWiki 0.3.3 were ready with verified local
  provenance, and the existing PostgreSQL service was healthy.
- Recall: Ticket 025 established the host-owned semantic theme, production-root
  accessibility matrix, 640×420 minimum, deterministic routed-screen focus,
  and the requirement to compile/test `Main.qml` after shared shell changes.
- Recall: `Main.qml` currently provides shell status, branding, one routed
  loader, and keyboard guidance but no application-owned close action.
  `ApplicationWindow.close()` can use the normal Qt window lifecycle without
  touching any controller authority.
- Decision: take a shell-only slice. The EXIT control remains visible on every
  route, uses `OgsButton`, and requests a normal close without logout or API
  calls. Installer work remains the next roadmap feature after this gap.

## Phase 2 — Design

- Architecture and lifecycle flow:
  - `Main.qml` remains the sole application-window and routing owner. A
    platform-owned `OgsButton` lives in the persistent brand bar outside the
    routed `Loader`, so every connection, access, MFA, persona, home, social,
    inbox, games, challenges, and gameplay state exposes the same action.
  - Activation calls only `ApplicationWindow.close()`. Qt emits its ordinary
    window close lifecycle; under `scripts/dev.sh`, `qml6` then returns and the
    existing shell trap terminates the child Rust server. The button does not
    call `logout`, clear a persona, revoke a session, send HTTP, alter a game,
    or dispatch a cartridge action.
  - Routed screens retain initial-focus ownership. The button participates in
    strong Tab focus through `OgsButton`, but it never calls
    `forceActiveFocus()` during load or route changes. Existing Escape behavior
    remains screen-owned.
  - The brand bar expands just enough to contain the existing 44-pixel control
    contract. The routed loader remains bounded between the brand bar and
    footer, and the production-root fixture proves the exit control remains in
    bounds at 640×420 and 920×600.
- Database and migration consequences: none. No SQL, server, API, transport,
  credential, account, persona, social, game, provider, or cartridge contract
  changes.
- API compatibility: unchanged. No controller function or public JSON schema
  changes; closing deliberately does not revoke the durable device session.
- Exact file manifest:
  - `client/qml/Main.qml` — replace the standalone brand item with a bounded
    brand bar containing the persistent EXIT button and normal close action.
  - `client/qml/tests/fixture/tst_accessibility.qml` — assert shell visibility,
    role/name, keyboard focus, compact/default bounds, and separate keyboard
    and pointer close behavior through the production root.
  - `docs/architecture/system-overview.md` and `README.md` — document normal
    client exit versus session revocation and development-server cleanup.
  - Ticket/spec/notes/AAR plus generated OpenWiki evidence — durable workflow
    record only.
- Regression plan:

| Requirement | Evidence |
|---|---|
| REQ-001 | Production-root fixture finds one `shellExitButton` before and after representative route changes and asserts its geometry at 640×420 and 920×600. |
| REQ-002 | Independent production-root cases activate the button with Return and a pointer, observe the window become non-visible, and directly review that the handler calls only `root.close()`. Existing controller-flow cases prove ordinary operations remain intact. |
| REQ-003 | The fixture checks `Accessible.Button`, the fixed descriptive name, enabled state, strong keyboard focus, and existing `OgsButton` focus contract. |

- Risks and rollback:
  - Accidental logout/revocation is prevented by keeping all controllers out of
    the handler and checking the exact source path during inspection.
  - A close test could terminate the entire Qt test process if it used
    `Qt.quit()`; production and tests therefore use window-local `close()` on a
    temporary `ApplicationWindow` while the Qt Test host remains alive.
  - Shell height changes can reduce compact routed content; the existing full
    screen matrix plus explicit button bounds exercises the settled 640×420
    layout.
  - Rollback is one QML shell/test/docs revert with no durable-data action.
- Alternatives rejected:
  - `Qt.quit()` is process-global and harder to exercise safely in the
    production-root test runner.
  - A per-screen exit duplicates global lifecycle policy and can drift.
  - A confirmation dialog is not justified while all player mutations are
    already durable and there is no unsaved local document.
  - Logout-before-close would conflate local process lifetime with durable
    device-session revocation.
- CodeGraph evidence: `mcp__codegraph__codegraph_explore` ran against the
  stable Ticket 026 design. The index again confirmed that it does not model
  the QML application or Qt Test graph and returned unrelated ambiguous
  server/provider symbols plus the Python fixture entrypoint. Direct review
  therefore covered the complete production `Main.qml`, `OgsButton`,
  `OgsTheme`, `tst_accessibility.qml`, and `scripts/dev.sh` close/cleanup path.
  No indexed Rust, API, database, game, or provider dependency is reached by
  the QML-only window close. The successful call is the worktree-bound design
  receipt; its unrelated matches are not treated as application coupling.

## Phase 3 — Implement

- Built:
  - Replaced the standalone brand item with a bounded persistent brand bar and
    added one shell-owned `OgsButton` labeled EXIT. It is visible outside the
    routed loader, has a fixed descriptive accessible name/description and
    explicit button role, and calls only `root.close()`.
  - Extended the production-root accessibility fixture so every routed-screen
    assertion also verifies the exit control's visibility, enabled state,
    accessibility contract, and horizontal bounds. Added independent Return
    and pointer activation cases; the authenticated Return case verifies the
    in-memory session and selected persona remain intact after window close.
  - Documented the normal close/session distinction and the exact development
    `pkill` fallback in README and the system overview.
  - The first focused run passed 38 cases and failed two route-matrix cases at
    the new role assertion: Qt reported `Accessible.NoRole` for the inherited
    button role. Added an explicit `Accessible.Button` role on the shell
    control. The fresh run passed all 40 cases with zero failures or skips.
- Deviations: production code remains inside the approved manifest and
  authority boundary. Validation expanded the test-only manifest as recorded
  below.
- Phase 4 candidate run exposed an existing `tst_onboarding.qml` timing race:
  after changing to registration mode, its deferred documented focus handoff
  could run after `enterText()` started and move focus away from the password
  field after one character. The shell-height change altered timing but not
  the underlying behavior. The fixture now waits for username focus to prove
  the handoff has settled before keyboard input. This one-file test deviation
  is required to make the existing focus contract deterministic; production
  `AccessScreen` behavior is unchanged.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Accessibility/correctness | The first implementation relied on Qt Controls to infer the exit control's accessible role, but the production-root fixture observed `Accessible.NoRole`. | medium | Resolved: set `Accessible.Button` explicitly and reran all 40 QML cases green. Capture `BF-omarchy-gaming-system-qml-inherited-accessible-role-gap-001` and `PR-omarchy-gaming-system-assert-explicit-accessible-role-for-shell-actions-001`. |
| 2 | Lifecycle/authority | Closing could have been implemented as logout/revocation or a controller mutation, changing durable account semantics. | high hypothesis | Closed with no finding: the one-line handler is `root.close()`; authenticated production-root evidence retains the session and selected persona after close, and no controller is referenced by the handler. |
| 3 | Layout/focus | The persistent 44-pixel action could overlap the brand or reduce routed content below the existing compact contract. | medium hypothesis | Closed with no finding: the bounded brand bar contains the action, every routed state remains green at 640×420, and screen-owned initial focus/reversible traversal still pass. |
| 4 | Operator safety | The first README fallback used a wildcard checkout path and could match another running checkout's QML process. | medium | Resolved: require invocation from the repository root and bind the process pattern to `$(pwd)/client/qml/Main.qml`. |
| 5 | CodeGraph/blast radius | The post-implementation query does not parse QML and returned unrelated symbols matching session/provider terms. | info | Accepted limitation: direct inspection covered the exact changed shell/test and existing `scripts/dev.sh` cleanup; no Rust/API/database/game/provider file changed or is called by the close handler. The successful explore remains the inspect receipt. |
| 6 | Test determinism | The first full gate's live QML stage entered registration input before `AccessScreen`'s deferred mode-change focus handoff settled; password focus was stolen after one character. | medium | Resolved in the fixture by waiting for the documented username focus target before typing. Record `BF-omarchy-gaming-system-qml-mode-focus-test-race-001` and `PR-omarchy-gaming-system-wait-for-deferred-qml-focus-before-input-001`; focused and full reruns are required. |

- Security/privacy review: no credential, token, account, persona, provider, or
  cartridge data is displayed or transmitted by the control. It is fixed
  platform text with no untrusted input and no request path. The test account
  session remains durable by design; closing only erases the client's
  process-memory bearer when the process exits.
- Fresh CodeGraph evidence: the final explore ran after the accessible-role
  fix over the changed shell and test terms. QML remains unsupported, so its
  unrelated server/provider matches were rejected; direct review supplies the
  authoritative source-level blast-radius conclusion.
- Post-focus-fix CodeGraph evidence: a second fresh explore ran against the
  final three changed QML/test files after the validation fix. It again
  returned unrelated game/provider terms because QML is unsupported. Direct
  inspection confirmed the only new test behavior is waiting for the existing
  `AccessScreen.onAccessModeChanged` `Qt.callLater(root.focusInitial)` handoff;
  no production controller or server surface changed. This successful call is
  the current inspect receipt.

## Phase 4 — Validate

- Tests run:
  - `./scripts/test-qml-onboarding.sh` after the role fix — PASS: visual policy
    passed across 33 production QML files; 40 Qt Quick cases passed with zero
    failures or skips.
  - The first `bin/gate.sh --diff` candidate passed stages 1–15 and 17–18 but
    correctly returned `GATE RED [diff]` because stage 16 exposed the existing
    deferred registration-mode focus race in `tst_onboarding.qml`.
  - `./scripts/test-qml-onboarding.sh` after the deterministic focus wait —
    PASS: the same 40 cases passed with zero failures or skips.
  - Final `bin/gate.sh --diff` — PASS: all 18 stages, including 45 migrated
    PostgreSQL cases, 40 deterministic QML cases, real API/QML scenarios,
    provider conformance, and the Door Legends authority pilot.
- Gate run: `GATE GREEN [diff]`; the exact gated worktree and delivery receipt
  both equal `ce03549e6eb5e80a7e0c59de8164f8619498d79cea358316abfa79ba4f69b87c`.
- Skips or pre-existing failures: none in the canonical final gate. Qt emitted
  non-fatal host EGL `dri2` diagnostics after successful offscreen smoke.

## Phase 5 — Complete

- Acceptance-criteria audit:

| Requirement | Evidence | Result |
|---|---|---|
| REQ-001 | The production-root fixture finds one visible, enabled `shellExitButton` on every one of the ten routed states and asserts its settled horizontal bounds at 640×420 and representative default-size coverage at 920×600. | PASS |
| REQ-002 | Independent Return and pointer cases close the application window; the authenticated case retains the exact session and selected persona, while source inspection confirms the handler calls only `root.close()`. | PASS |
| REQ-003 | The production root observes `Accessible.Button`, the stable descriptive name and description, enabled Tab focus through `OgsButton`, and the existing visible focus treatment. | PASS |

- Docs: updated README with the visible action and checkout-bound emergency
  command, and updated the system overview with the normal close/session
  distinction. OpenWiki update run
  `7b6df1fb-0ad7-4cab-86e0-c48abf33c0d7` reconciled quickstart, runtime, and
  validation pages and returned `status: complete`. Its warning concerned
  pre-existing unresolved quickstart evidence debt; no Ticket 026 claim or
  requirement remains unresolved.
- AAR: submitted `AAR-026` with two captured failures and two prevention rules;
  all four IDs are registered in `docs/planning/knowledge/INDEX.md`. No new
  durable architecture decision was required.
- Archive: Ticket 026 is closed and this only active spec/notes pair is moved
  to `docs/planning/pipeline/completed/`. Git delivery remains separately
  controlled and is not authorized by this completion step.
- Final receipt: the post-OpenWiki, post-archive `bin/gate.sh --diff` run passed
  all 18 stages with `GATE GREEN [diff]`. Both the delivery receipt and current
  gated state equal
  `6e32ad0b1b41c6060adac2a10066f9ebb1028e4186e66b292f94f9372b663554`;
  the Ticket 026 completion receipt matches the same gated worktree.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The shell EXIT action appeared as `Accessible.NoRole`. | The composed production root did not preserve the role inferred from the shared Qt Control. | Declare `Accessible.Button` explicitly and assert it through `Main.qml`. | `PR-omarchy-gaming-system-assert-explicit-accessible-role-for-shell-actions-001` |
| 2 | Registration fixture input could lose focus after its first character. | Input started before the screen's queued mode-change focus handoff settled. | Wait for username active focus before keyboard injection. | `PR-omarchy-gaming-system-wait-for-deferred-qml-focus-before-input-001` |
