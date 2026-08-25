---
title: Keyboard-first QML account and persona onboarding — notes
pipeline_id: e538f6de-de94-432e-80b1-d41da6ccc417
---

# Keyboard-first QML account and persona onboarding — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: `main` began clean at delivered Ticket 019 commit `c1636b2`; no
  active bulletin or pipeline blocks work. The ticket index incorrectly kept
  Ticket 019 in the open queue after its archive, so that planning-only
  bookkeeping was corrected before opening Ticket 022.
- Recall: `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0,
  OpenWiki 0.3.3, and Codex-only provenance ready.
- Recall: every server-side private-alpha identity outcome already exists:
  registration, revocable device sessions, opt-in TOTP/recovery login,
  persona creation/inventory, owner authorization, stable error envelopes, and
  no-store responses. The QML `Main.qml` still calls only `/health` and its
  reconnect control is pointer-only.
- Recall: `PR-omarchy-bbs-verify-the-vertical-slice-001` requires the real
  database, HTTP endpoint, and QML consumer to run together. Owner-scoped
  account/persona rules and independent MFA challenge semantics remain binding
  at the client boundary.
- Recall: the product charter's private-alpha definition begins with two clean
  Omarchy installations reaching one server and creating personas. The roadmap
  orders keyboard/accessibility client work before packaging and sysop work;
  README explicitly leaves QML gameplay/challenge screens for later slices.
- Decision: Ticket 022 owns endpoint selection through authenticated persona
  selection only. It includes existing MFA login because omitting it would
  strand opted-in accounts, but excludes enrollment/settings and all social or
  game navigation.
- Decision: persistent sign-in is not implemented by writing bearer tokens to
  QML settings. A later keyring-aware slice may add it; this slice logs in per
  process and clears every secret on terminal transitions.

## Phase 2 — Design

- CodeGraph evidence: the design exploration traced `Main.qml`'s current
  health-only entrypoint and the server router through `create_session`,
  `complete_mfa_session`, `create_persona`, and `list_personas`, including
  explicit transport DTOs and their domain callers. The worktree-bound receipt
  records pipeline `e538f6de-de94-432e-80b1-d41da6ccc417` at gated-state hash
  `c12cbc87867cec0b56959c22dbfd8f7f42538a647f58225a3cf475be98cdbc19`.
  CodeGraph does not model QML, Bash, Python, or generated docs completely, so
  those surfaces and their existing tests were inspected directly.
- Architecture: keep `Main.qml` as a thin window/router over one
  `OnboardingController`. The controller owns the finite player state
  (`connection`, `access`, `mfa`, `personas`, `home`), public status/error text,
  persona inventory and selection, while `ApiClient` owns a single bounded XHR,
  endpoint admission, generation-based stale-response rejection, body limits,
  and the only in-memory bearer field. Screen components emit intent and never
  construct URLs, authorization headers, or retain secrets.
- Data flow: startup parses an optional `--server-url`, admits only HTTPS or
  loopback HTTP, and calls exact `/health`. A ready health identity enables
  registration/login. Registration returns only the canonical username to the
  access view. Login either installs a strictly validated session token in the
  API client or stores one in-memory MFA challenge; successful MFA takes the
  same session path. Authenticated persona inventory uses the header-only
  bearer, exact seven-field profiles, and an explicit selected persona. Logout,
  endpoint changes, invalid sessions, and protocol-fatal responses abort the
  request and clear token, challenge, inventory, and selection.
- Transport contract: keep all server routes and schemas unchanged. XHR allows
  only one active operation, applies a ten-second timeout and a 256 KiB response
  ceiling, ignores callbacks whose generation is no longer current, sends
  `Content-Type: application/json` only for JSON bodies, and sends
  `Authorization: Bearer ...` only for authenticated calls. Success documents
  require exact key sets, UUID/token/timestamp shapes, and bounded strings.
  Error UI maps only allowlisted stable error codes and never renders arbitrary
  server messages or markup.
- Security/privacy: no `Settings`, LocalStorage, file, log, URL, or display
  surface receives a password, bearer, recovery code, TOTP, or challenge. Form
  controls clear password/factor text synchronously after copying it into the
  call. Device/challenge tokens remain process-memory-only until terminal use;
  ordinary logout intentionally does not promise remote revocation because
  session management is out of scope. Non-loopback plaintext endpoints,
  userinfo, path, query, fragment, and invalid ports fail before health.
- Concurrency/recovery: the UI disables duplicate submission, while the API
  layer still cancels and invalidates any previous generation so programmatic
  overlap cannot cross-contaminate responses. A local timer aborts hung XHRs.
  Recoverable action errors remain on their owning screen; health failures use
  explicit offline/protocol/configuration states. An authenticated 401 with
  `invalid_session` clears all authority and returns to sign-in.
- Accessibility/layout: standard Qt Quick Controls provide native keyboard
  activation. Styled controls add explicit accessible names, `activeFocusOnTab`,
  and a visible high-contrast focus ring. Each screen declares its initial
  focus plus Enter/Escape semantics; scrollable bounded layouts support both
  640×420 and 920×600 without hiding the only recovery action.
- Test architecture: Qt Quick Test is available at the Qt installation's
  `bin/qmltestrunner` even though it is not on `PATH`. The focused script resolves
  it through `qmake6 -query QT_INSTALL_BINS`, starts a quiet loopback Python
  fixture, and runs real key events against `Main.qml` plus transport/schema,
  timeout, overlap, 401, malformed, and oversized-response cases. The fixture
  lives under `client/` so it participates in the gated-state hash and never
  logs request bodies or headers. The existing migrated live smoke invokes a
  second QML Test case for registration/login/persona creation and replaces the
  current curl-only MFA completion with a QML-controller completion before
  replay proof.

### File manifest

| Path | Purpose |
|---|---|
| `client/qml/ApiClient.qml` | Admit/normalize the endpoint; serialize one bounded XHR; own generation, timeout, size cap, header-only bearer, abort, and secret clearing. |
| `client/qml/OnboardingController.qml` | Own the onboarding state machine, exact response validators, safe error mapping, session/MFA transitions, persona inventory/selection, and terminal cleanup. |
| `client/qml/components/OgsButton.qml`, `OgsTextField.qml`, `OgsTextArea.qml` | Provide consistent visible focus, accessible names, keyboard activation, secret masking, and retro styling. |
| `client/qml/screens/ConnectionScreen.qml` | Endpoint editing, health state, connect/retry, and configuration/offline/protocol recovery. |
| `client/qml/screens/AccessScreen.qml` | Registration and primary-login forms with masked/cleared password and stable error presentation. |
| `client/qml/screens/MfaScreen.qml` | TOTP/recovery entry, expiry display, retry, and cancel semantics without exposing the challenge. |
| `client/qml/screens/PersonaScreen.qml` | Owned inventory selection and bounded persona creation with keyboard-only list/form transitions. |
| `client/qml/screens/HomeScreen.qml` | Selected public persona proof, endpoint identity, next-slice placeholder, change-server, and local logout. |
| `client/qml/Main.qml` | Replace the health probe with the thin responsive window, controller wiring, screen routing, and initial-focus transfer. |
| `client/qml/tests/fixture_server.py` | Serve deterministic exact, malformed, delayed, oversized, stale, MFA, persona, and unauthorized responses without credential/header logging. |
| `client/qml/tests/fixture/tst_onboarding.qml` | Drive production Main/controller/API behavior with Qt key events, exact state/secret/accessibility assertions, and two supported window sizes. |
| `client/qml/tests/live/tst_live_onboarding.qml` | Exercise real migrated registration/login/MFA/persona flow through the production controller. |
| `scripts/test-qml-onboarding.sh` | Resolve Qt Quick Test, manage an ephemeral fixture/port, run the focused corpus, and preserve failure output. |
| `scripts/dev.sh` | Add required Qt-test tooling, invoke the fixture corpus, pass ephemeral test-only live inputs, and require real QML onboarding/MFA evidence in smoke mode. |
| `docs/api.md`, `README.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md` | Document endpoint/credential/client boundaries, the implemented client slice, and remaining social/gameplay/package work. |
| `CONSTITUTION.md`, `openwiki/*` | Reconcile the canonical gate description and generated engineering evidence during completion; do not hand-edit generated pages before OpenWiki. |
| Ticket/spec/notes/AAR/knowledge index | Preserve requirements, evidence, findings, decisions, and workflow state. |

- Database/migration consequences: none. The client consumes only committed v1
  endpoints, and test state uses the existing isolated/migrated database path.
- Compatibility: default launch remains `http://127.0.0.1:8080`; `--smoke-test`
  remains supported; `scripts/dev.sh` still opens one visible client and stops
  the child server when it closes. New explicit endpoint configuration is
  additive. Old bearer prefixes remain accepted only when returned by the
  server, but newly issued smoke tokens must remain `ogs1_`.
- Alternatives rejected: plaintext QML `Settings` token persistence would turn
  a convenience into credential storage; a Rust client daemon would add an
  unnecessary authority/process boundary; WebSockets would not improve durable
  onboarding; one monolithic `Main.qml` would make state and secret lifetime
  untestable; fixture-only validation would miss server drift; extending this
  slice through inbox/challenges/gameplay would obscure the authentication
  boundary and delay the first usable client checkpoint.

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Endpoint unit matrix plus QML fixture cases for exact health, invalid scheme/userinfo/path/port, offline, timeout, malformed and wrong-identity responses; real default-loopback smoke. |
| REQ-002 | Key-driven registration validation/conflict/success flow; fixture request-shape assertion and exact response validation; live QML registration with canonical username carryover. |
| REQ-003 | Normal-login fixture/live paths; fixture asserts Authorization is absent from public calls and exact on persona calls; QML asserts password clearing and bearer removal after logout/401; source scan rejects persistent storage. |
| REQ-004 | Fixture invalid-factor/retry/cancel/expiry/success matrix plus real PostgreSQL-enrolled recovery-code completion through the production controller and existing replay rejection. |
| REQ-005 | Empty, one, multiple, malformed, foreign-field and creation-conflict fixture shapes; key-driven selection/create; live owned-persona create/list/select with exact public fields. |
| REQ-006 | Delayed superseded response, local timeout, oversized body, invalid JSON/schema/status and authenticated 401 tests assert only the current operation can transition state. |
| REQ-007 | Qt Quick Test sends Tab/Backtab/Enter/Escape at 640×420 and 920×600, verifies focus visibility, accessible names, activation, scroll recovery, masked fields, and plain-text status. |
| REQ-008 | `scripts/test-qml-onboarding.sh`, enhanced `scripts/dev.sh --smoke-test`, and the final `bin/gate.sh --diff` all run after the last gated edit. |

## Phase 3 — Implement

- Built: a strict HTTPS/loopback endpoint boundary; a single-generation QML
  XHR client with timeout, response ceiling, redirect comparison, header-only
  in-memory bearer, deferred abort, and stale callback rejection; an explicit
  connection/access/MFA/persona/home controller with exact schema validators
  and allowlisted error text; five responsive screens; three shared accessible
  controls; keyboard focus/Enter/Escape behavior; masked and synchronously
  cleared password/factor fields; persona inventory/create/select and local
  logout; deterministic Python HTTP fixtures; Qt Quick UI/transport tests; a
  live migrated QML registration/persona scenario; and a live MFA completion
  that replaces the prior curl-only happy path while retaining factor-replay
  proof.
- Built: `scripts/dev.sh --smoke-test` now runs the 15-case hostile/keyboard
  fixture corpus, two real QML controller scenarios, the visible `Main.qml`
  health entrypoint, and all existing API/game/social/MFA assertions. Ephemeral
  live credentials and recovery factors move through a NUL-delimited pipe into
  a mode-0600 ignored config, never command-line arguments. Cleanup removes
  exact config files on success or failure.
- Documentation: Constitution gate 16 now describes the actual onboarding
  evidence and the previously implemented Door Legends authority proof is
  restored as gate 18. README, API, system overview, and roadmap document the
  client boundary and remaining screens without claiming token persistence or
  remote revocation.
- Focused evidence: `scripts/test-qml-onboarding.sh` passed twice consecutively
  after the final explicit Enter handler and passed again after live-smoke
  wiring; each run reported 15/15 Qt tests. `scripts/dev.sh --smoke-test`
  passed the fixture corpus, real QML registration/login/persona creation,
  real enrolled MFA/recovery completion and selection, Main health startup,
  and the existing migrated server smoke. Bash/Python syntax, pipeline
  structure, secret scan, and `git diff --check` passed.
- Deviation: `qmltestrunner` rejects arbitrary application arguments. Fixture
  and live inputs therefore use separate mode-0600 `.dev/qml-onboarding` JSON
  files read only under the test-only `QML_XHR_ALLOW_FILE_READ=1` process
  environment. Production `qml6 Main.qml -- --server-url=...` remains an
  additive CLI path, and the visible connection screen is authoritative.
- Deviation/fix: the first timeout corpus exposed a Qt 6.11 XHR use-after-free
  crash when `abort()` and handler removal occurred synchronously around an
  in-flight callback and the test owner was destroyed. Cancellation now first
  invalidates the operation generation, retains the XHR briefly, replaces its
  handler, and schedules abort outside the current callback. The complete
  timeout/oversize/supersession corpus then passed repeatedly without a crash.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| I-001 | Secret-file lifecycle | The ignored live-test credential file was mode 0600, but its deterministic parent directory retained the caller's umask and the writer followed a pre-existing final-component symlink. | Low | Fixed: both runners force the directory to 0700 and the Python writer rejects a symlink parent and opens the file with `O_NOFOLLOW` where supported. The focused 15-case corpus passed after the change. |
| I-002 | API-contract fidelity | Persona response/form bounds were safe but looser than the authoritative server contract, and required success strings had no non-empty lower bound. | Low | Fixed: client validation and controls now match username, handle, display-name, bio, and status limits; required response strings reject empty values; newly created sessions must expire in the future. |
| I-003 | EARS test reconciliation | The fixture implemented registration/persona conflicts and a terminal MFA challenge error, but the Qt corpus did not invoke them even though the regression table claimed that coverage. | Low | Fixed: added safe registration conflict/malformed-success, terminal MFA cleanup, persona-conflict/session-preservation, and exact response-bound tests. |

- Direct inspection covered every changed QML, Bash, Python, and documentation
  surface plus the supporting Axum/session/MFA/persona paths. The final
  CodeGraph exploration reconciled the server response DTOs, owner-scoped
  persona queries, and one-hop callers with the QML validators. Its inspection
  receipt matches pipeline `e538f6de-de94-432e-80b1-d41da6ccc417` at gated
  state `b2dc1db92391a8f02e501c863320f093da4ca8cbf076811a9632823d91d31fc0`.
- Codex Security diff scan `1941a96b-0977-49ff-8270-079233cbd976` completed
  against final source snapshot
  `codex-security-snapshot/v1:sha256:da4bdd28af8dfa3362ee7212c5506bf3e6360e1c8eeb1d2829189d221f0d3518`
  with zero candidates and zero reportable findings. The compact scanner
  inventory omitted untracked QML/Python, so those files were included in the
  manual scoped coverage and the independent threat-boundary review.
- Rejected as a vulnerability: redirect comparison occurs after XHR
  completion, but the explicitly selected origin already receives the same
  submitted password or bearer before it can direct a redirect; this creates
  no new attacker authority, while the final-URL check still prevents a
  foreign response from being accepted as protocol success. Secure memory
  zeroization and same-user process isolation are not claimed.
- Phase 3.5 exit: all three confirmed low-severity findings are fixed, the
  expanded focused corpus reports 19/19 passing tests, no security finding
  remains, and the fresh CodeGraph receipt matches the post-fix gated tree.

## Phase 4 — Validate

- Tests run: the post-fix `scripts/test-qml-onboarding.sh` run passed 19/19
  fixture/UI/transport cases in 5.6 seconds with the deterministic offscreen
  harness. The final `bin/gate.sh --diff` reran rustfmt, clippy, workspace
  tests, rustdoc, Compose validation, shell syntax, pipeline structure, secret
  scan, hook self-tests, whitespace, cartridge contract/renderer/SDK/spike,
  44/44 PostgreSQL integration tests, the 19/19 QML fixture corpus, two 3/3
  real migrated QML scenarios, remote-provider security conformance, and the
  clean-clone Door Legends authority pilot.
- Gate run: the first full gate honestly ended `GATE RED [diff]` with only
  stage 16 failed after the inherited Wayland platform hang was interrupted;
  stages 17 and 18 still passed. After the deterministic environment fix and
  refreshed CodeGraph receipt, the complete rerun printed `GATE GREEN [diff]`.
  The gate receipt and inspection receipt both bind state
  `8eaf77e5210562aaaf604062d8925d6eb49155ef1f18e56ccbc036d116e33fa6`.
- Skips or pre-existing failures: the ordinary workspace test stage reported
  only its expected PostgreSQL/provider-backed ignored cases; the later
  canonical integration stages exercised those cases successfully. The final
  QML Main smoke emitted non-fatal software-renderer EGL warnings and exited
  successfully. No failure or unplanned skip remains.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — endpoint fixtures admit only HTTPS or exact loopback HTTP;
    malformed origins fail before a request; normal, malformed, wrong-identity,
    delayed, and oversized health responders prove distinct bounded states; the
    real loopback API reaches access without credentials before readiness.
  - REQ-002 PASS — key-driven registration masks and clears the password,
    requires the exact request, maps validation/conflict codes through the
    allowlist, rejects malformed success, carries only canonical username, and
    completes against the migrated API.
  - REQ-003 PASS — the API client owns the only in-memory bearer and emits it
    only on authenticated requests; password input clears synchronously;
    logout, endpoint change, malformed authenticated success, and
    `invalid_session` clear all authority and persona state in fixtures and the
    live path.
  - REQ-004 PASS — retryable MFA errors retain only the in-memory challenge;
    cancel, local expiry, and terminal challenge errors clear it; a real
    enrolled single-use recovery code completes QML login and existing smoke
    proof rejects replay.
  - REQ-005 PASS — exact seven-field persona inventories permit keyboard
    selection, empty inventory permits bounded creation, conflict preserves
    the valid session, malformed/private-field shapes fail closed, and both
    live create/select and MFA owned-selection reach home with one persona.
  - REQ-006 PASS — one generation owns each XHR; timeout, response ceiling,
    malformed JSON/schema, terminal authorization, and superseded slow-request
    cases prove stale callbacks and wrong-operation responses cannot transition
    current state.
  - REQ-007 PASS — production screens at 640×420 and 920×600 prove accessible
    names, visible focus, Tab/Backtab traversal, Enter activation, Escape
    recovery, masked inputs, explicit plain-text output, and no pointer-only
    action.
  - REQ-008 PASS — the canonical gate executes the 19-case hostile/keyboard
    corpus, both real migrated QML controller scenarios, and the standalone
    production-root smoke before accepting stage 16.
- Docs: README, API, system overview, roadmap, Constitution, and the OpenWiki
  quickstart/runtime/development pages describe the implemented access shell,
  process-memory authority, focused gate, and remaining social/game client
  boundary. OpenWiki update run `cb5b4463-9c5f-4b05-bb6c-4a0a69a3fd6e`
  returned `status: complete`; it reported pre-existing unresolved evidence
  debt on the three broad pages, but the ticket's changed claims were inspected
  and resolved before prose authoring. The completion receipt matches pipeline
  `e538f6de-de94-432e-80b1-d41da6ccc417` and gated state
  `5c6eb8c4ae7495e5d319c09488d273397079f56da408f0ba82b456f8b9bc6c74`.
- AAR: AAR-022 is submitted at effectiveness 5 with six captured failures, six
  standing prevention rules, and the QML onboarding authority decision. Every
  new ID is registered in `docs/planning/knowledge/INDEX.md`.
- Final validation: the post-OpenWiki `bin/gate.sh --diff` passed all 18 stages
  and printed `GATE GREEN [diff]`: 44/44 migrated PostgreSQL tests, 19/19 QML
  fixture/UI/transport cases, both 3/3 live QML scenarios, the production-root
  smoke, provider conformance, and the clean-clone Door Legends authority
  pilot. Delivery, completion, and current gated hashes all equal
  `5c6eb8c4ae7495e5d319c09488d273397079f56da408f0ba82b456f8b9bc6c74`.
- Archive: Ticket 022 is closed and this spec/notes pair is archived. No
  requirement was deferred or silently dropped; social, inbox, game catalog,
  cartridge launch, gameplay, token persistence, and settings remain explicit
  later slices.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first post-inspection fixture run failed to compile `PersonaScreen.qml`: `TextArea` has no native `maximumLength` property. | Qt Quick `TextArea` and `TextField` have different input-limit APIs. | Added an explicit bounded `maximumLength` property and truncation handler to the shared `OgsTextArea`. | Exercise the production `Main.qml` after every QML control-contract edit; do not assume controls share `TextInput` properties. |
| 2 | The first canonical diff gate reached stage 16 and then its QML fixture process waited indefinitely for `windowShown`; the gate was interrupted after the stuck process was confirmed and ultimately reported one failed stage. | The focused script honored an already-set desktop `QT_QPA_PLATFORM=wayland;xcb` instead of enforcing its intended deterministic offscreen harness. | The focused fixture runner now sets `QT_QPA_PLATFORM=offscreen` and `QT_QUICK_BACKEND=software` unconditionally; the live runner already did so. | Headless UI gate scripts must own their platform/backend environment rather than inherit an interactive desktop session. |
