---
aar: AAR-022-keyboard-first-qml-account-and-persona-onboarding
ticket: TICKET-022
pipeline: keyboard-first-qml-account-and-persona-onboarding
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-022-keyboard-first-qml-account-and-persona-onboarding

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | Knowledge register and current health smoke | Yes — the QML identity flow needs real migrated API evidence, not fixture-only UI checks. |
| `PR-omarchy-bbs-owner-scope-account-resources-001` | Knowledge register, persona API, and system overview | Yes — the client may select only personas returned through its authenticated owned inventory. |
| `PR-omarchy-gaming-system-preserve-independent-mfa-challenges-001` | Knowledge register and MFA API | Yes — canceling or retrying one client challenge must not manufacture global invalidation semantics. |
| Product charter and roadmap | Product preflight | Yes — keyboard-first client access is the first unfinished private-alpha outcome and precedes packaging and sysop operations. |
| Ticket 019 completed pipeline | Nearest completion and clean-branch readback | Yes — remote-provider work is closed; QML gameplay remains explicitly excluded and therefore available as later client work. |

## What happened

Ticket 022 replaced the health-only QML connector with the first usable
keyboard-first player access shell. A player can select a safely admitted
server origin, register, sign in with a password or an already-enabled MFA
factor, load or create an owned persona, select it, reach an authenticated
home, and locally log out. The client uses the existing REST contracts, keeps
Bearer and MFA authority in process memory only, validates exact bounded
response shapes, permits remote endpoints only over HTTPS, and clears
authority on every terminal or protocol-fatal path.

The implementation separates transport, onboarding state, shared accessible
controls, and five screens. The single-generation XHR boundary rejects stale
callbacks, timeouts, oversized responses, and unexpected final URLs. A
deterministic Python fixture plus Qt Quick Test exercises 19 hostile transport,
keyboard, focus, accessibility, schema, conflict, expiry, and cleanup cases.
The migrated development smoke separately drives the production controller
through registration/password/persona and MFA-recovery/persona flows.

Inspection fixed three low-severity findings: the test-secret directory and
writer now preserve file authority, response and form bounds match the server,
and claimed conflict/terminal paths have executable cases. It also exposed a
Qt XHR abort lifetime crash, a `TextArea` API mismatch, and inherited desktop
Qt state in the headless gate. Each was fixed and regression tested. The final
Codex Security diff scan reported zero findings, OpenWiki completed, and the
18-stage delivery gate passed 44 PostgreSQL tests, 19 QML fixture cases, both
live QML scenarios, provider conformance, and the clean-clone authority pilot.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-qml-xhr-abort-lifetime-crash-001` | Synchronously aborting an in-flight Qt XHR while its callback owner was being destroyed could trigger a Qt 6.11 use-after-free crash. | Timeout and supersession fixture corpus |
| `BF-omarchy-gaming-system-qml-test-secret-path-authority-gap-001` | The ignored live-test credential file was mode 0600, but its parent permissions were umask-dependent and the writer could follow a pre-existing final-component symlink. | Phase 3.5 secret-lifecycle inspection |
| `BF-omarchy-gaming-system-qml-client-contract-bound-drift-001` | Initial QML profile bounds and required success-string validation were looser than the authoritative server contract. | Phase 3.5 API-contract inspection |
| `BF-omarchy-gaming-system-qml-regression-claim-coverage-gap-001` | The fixture implemented conflict and terminal MFA outcomes that the stated regression matrix claimed but the Qt corpus did not invoke. | Phase 3.5 EARS reconciliation |
| `BF-omarchy-gaming-system-qml-textarea-limit-api-assumption-001` | `PersonaScreen.qml` initially used `TextField`'s `maximumLength` API on a Qt Quick `TextArea`, preventing the production root from compiling. | First post-inspection focused run |
| `BF-omarchy-gaming-system-qml-headless-platform-inheritance-001` | The focused runner inherited an interactive Wayland platform value and waited indefinitely for `windowShown` during the first full gate. | Canonical diff gate stage 16 |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-retire-qml-xhr-after-generation-invalidation-001` | Invalidate the current QML request generation before retiring an XHR, detach its callback, retain it briefly, and abort outside the active callback. | Stale work must become inert without destroying Qt network state during its callback lifetime. |
| `PR-omarchy-gaming-system-protect-test-secret-file-handoffs-001` | For test-only credential handoffs, use a mode-0700 directory and mode-0600 non-symlink file, keep secrets out of argv and logs, and remove the exact file on every exit. | Ignored development state still carries real authority during live tests. |
| `PR-omarchy-gaming-system-mirror-authoritative-client-response-bounds-001` | Client success validators and form limits must mirror the authoritative server contract exactly and reject empty required values or expired authority. | A merely safe but looser client contract hides drift and accepts states the server never promises. |
| `PR-omarchy-gaming-system-reconcile-regression-claims-with-executed-cases-001` | Reconcile every claimed hostile fixture outcome with an invoked test case before accepting the inspection gate. | Fixture capability is not evidence until the client actually drives and asserts the path. |
| `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001` | Instantiate the production QML root after shared-control contract edits instead of relying only on isolated component assumptions. | Qt Quick controls do not share every input API even when their surface looks similar. |
| `PR-omarchy-gaming-system-own-headless-qt-test-environment-001` | Headless QML gate entrypoints must set their platform and rendering backend unconditionally. | Inheriting an interactive desktop backend makes otherwise deterministic UI evidence session-dependent. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-qml-onboarding-authority-boundary-001` | The flagship QML client calls the existing REST API directly, keeps Bearer and MFA authority only in process memory, and ends this slice after explicit owned-persona selection; persistence, social navigation, and game launch require later reviewed boundaries. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All eight requirements have focused and integrated evidence. The design
reused committed server contracts without adding a daemon, endpoint, migration,
or competing authority layer; process-memory-only credentials kept persistent
sign-in honestly out of scope. Direct inspection found three low-severity
contract and test-boundary gaps, and dynamic validation found three Qt-specific
lifetime/environment/control issues before delivery. All were fixed, the
security scan closed with zero findings, OpenWiki completed, and delivery and
completion receipts match gated state
`5c6eb8c4ae7495e5d319c09488d273397079f56da408f0ba82b456f8b9bc6c74`.
