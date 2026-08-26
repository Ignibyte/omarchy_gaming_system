---
aar: AAR-026-explicit-qml-application-exit-control
ticket: TICKET-026
pipeline: explicit-qml-application-exit-control
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: 5
---

# AAR-026-explicit-qml-application-exit-control

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001` | Knowledge-register search for QML shell rules | Yes — requires production-root evidence for the shell edit. |
| `PR-omarchy-gaming-system-assert-minimum-layout-after-settle-001` | Ticket 025 completion notes | Yes — retains the 640×420 layout contract. |
| `AD-omarchy-gaming-system-host-owned-semantic-qml-theme-001` | Ticket 025 architecture handoff | Yes — keeps the control inside trusted shell/theme primitives. |

## What happened

Ticket 026 added one persistent, platform-owned EXIT button to the production
QML shell. The control remains visible above every routed screen, uses the
existing keyboard and focus treatment, and requests only a normal
`ApplicationWindow.close()`. Closing does not log out, revoke the durable
device session, clear the selected persona, or dispatch any server, game, or
cartridge action. Under the development launcher, the existing cleanup trap
stops the child Rust server after the QML process returns.

The production-root fixture now checks the control on every route at compact
and default sizes and exercises both keyboard and pointer activation. The
authenticated keyboard case proves that the controller still retains its
session and selected persona after the window becomes invisible. Inspection
also narrowed the documented emergency process command to the current
checkout.

Two validation defects were corrected. Qt exposed the inherited shell button
as `Accessible.NoRole`, so the production control now declares its button role
explicitly. The first full gate also exposed a latent test race in registration
mode: input could begin before the deferred focus handoff completed. The
fixture now waits for that documented focus target before typing. After both
fixes, all 40 QML cases and every stage of the 18-stage diff gate passed.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-qml-inherited-accessible-role-gap-001` | A shell action implemented with the shared button control was exposed as `Accessible.NoRole` through the production root. | First focused production-root accessibility run. |
| `BF-omarchy-gaming-system-qml-mode-focus-test-race-001` | Registration input began before the screen's deferred mode-change focus handoff settled, allowing focus to move after typing started. | First full diff-gate QML run. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-assert-explicit-accessible-role-for-shell-actions-001` | Declare and production-root test the explicit accessible role for every persistent shell action, even when its shared control usually supplies one. | Inherited Qt Control semantics may not survive the exact composed QML object path seen by assistive technology. |
| `PR-omarchy-gaming-system-wait-for-deferred-qml-focus-before-input-001` | When a QML mode change schedules a deferred focus handoff, wait for the documented target to own active focus before injecting test input. | Component state can be updated before the queued focus transition has completed. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| None | The normal application-window close follows the existing host-owned shell and session-lifetime boundaries; no new durable architecture decision was required. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. All three EARS requirements have direct production-root keyboard,
pointer, accessibility, route, authority-retention, and compact-layout
evidence. Inspection narrowed the operator fallback and focused validation
caught the accessible-role gap; the canonical gate then caught the independent
deferred-focus fixture race. OpenWiki and the durable architecture records now
agree with the behavior. The work remains deliberately uncommitted and
unpushed until the user separately authorizes Git delivery.
