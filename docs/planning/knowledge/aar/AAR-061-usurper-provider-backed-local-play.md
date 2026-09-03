---
aar: AAR-061-usurper-provider-backed-local-play
ticket: TICKET-061
pipeline: usurper-provider-backed-local-play
status: submitted
opened: 2026-09-02
submitted: 2026-09-02
effectiveness: effective
---

# AAR-061-usurper-provider-backed-local-play

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AAR-060-usurper-level-eight-dungeon-band` | User feedback showed that an enabled fixture-render button was mistaken for a working game control. | Yes — the fixture is now visibly inert, while the separate live shell proves confirmed provider mutation. |
| `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001` | A local convenience bridge could accidentally accept actions that are not present on the current signed screen. | Yes — action, empty payload, screen, revision, and current-plan membership are checked together. |
| `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001` | Each provider transition replaces the QML plan. | Yes — every replacement is admitted again by the trusted QML surface. |
| `PR-omarchy-gaming-system-resolve-cargo-artifacts-from-metadata-001` | The launcher crosses two Cargo workspaces with configurable target directories. | Yes — both workspaces' executables are resolved from structured Cargo metadata. |

## What happened

The slice added a non-packaged, provider-backed Usurper local-play client. Its
ephemeral loopback service owns one real `UsurperGame` session, accepts only
actions emitted by the current signed render plan, distinguishes signed
navigation from provider mutation, and commits a candidate state only after its
new view renders successfully. Every response is compiled from the signed
cartridge by the platform renderer and admitted by the production trusted QML
surface. The old fixture preview is now explicitly non-interactive.

The first real-click smoke exposed a pre-existing QML lifecycle defect: controls
captured the surface's disabled loading state and never became enabled. Dynamic
bindings fixed the bug. The first full platform gate then caught a separate
packaging mistake: the development harness had been put in the production QML
inventory. Moving it to the excluded test root restored the exact forty-file
native package while retaining the local-play smoke.

The external suite passed all 74 Rust tests, strict Clippy, rustdoc, provenance,
privacy, seventeen-screen, and live HTTP/QML checks. The fifteen-case live
provider corpus passed twice across restart, the security diff scan reported
zero findings, and the corrected full platform gate passed all stages. OpenWiki
and the hand-maintained cartridge architecture were reconciled before archival.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-qml-action-enablement-snapshot-001` | Dynamically created trusted controls copied `actionsEnabled=false` during plan loading and did not follow the surface when loading finished. | First provider-backed QML click smoke timed out without dispatching an action. |
| `BF-omarchy-gaming-system-development-qml-production-inventory-leak-001` | A development-only QML harness was initially placed in the exact production cartridge source inventory. | First full platform gate failed native package inventory stages 15 and 16. |
| No new ID | The human-facing launcher still defaulted to the platform's retired sibling name even though automated checks supplied the current platform root explicitly. | Final visible workspace-8 launch failed before build metadata resolution. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bind-control-enablement-across-loading-transitions-001` | Bind dynamically created interactive controls to their owning surface's current action authority, and test the disabled-to-enabled transition with a real click. | Construction-time property copies can strand a correctly loaded interface in a permanently disabled state. |
| `PR-omarchy-gaming-system-place-development-qml-outside-production-inventory-001` | Put development-only QML harnesses under an excluded test/tool root and retain an exact production package-inventory gate. | A visually useful harness must not silently become signed production application content. |
| No new ID | Exercise the no-override human launcher in the final visible check, in addition to an override-driven automation path. | An explicit test root can mask a stale default while all application behavior remains correct. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-provider-backed-local-play-001` | Visible Usurper development testing uses an ephemeral loopback capability service over the real provider with render-before-commit semantics; the fixture viewer stays inert, and this grants no production admission or persistence. | `docs/architecture/game-cartridges.md` |

## Effectiveness

Effective. The recalled action-binding, render-handoff, and Cargo-artifact rules
kept the convenience shell on established authority boundaries. A real trusted-
QML click now confirms revision 1 instead of merely logging a request, hostile
HTTP cases fail without mutation, and the fixture can no longer imply that it
is a playable client. Focused tests, the restarted provider corpus, a sealed
zero-finding security scan, the exact package inventory, and the green full
platform gate jointly validate the result without registration, admission,
persistence, deployment, commit, push, or publication.
