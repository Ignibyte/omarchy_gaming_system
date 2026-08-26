---
aar: AAR-025-end-to-end-qml-accessibility-and-visual-polish
ticket: TICKET-025
pipeline: end-to-end-qml-accessibility-and-visual-polish
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-025-end-to-end-qml-accessibility-and-visual-polish

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Private-alpha roadmap and product charter | First-playable game flow is complete; accessibility/visual polish is the next ordered unchecked outcome. | Yes — it fixes the slice boundary and prevents installer/operations scope creep. |
| `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001` | Ticket 022 shared-control compilation failure | Yes — every shared primitive change must compile through production `Main.qml`, not only an isolated component. |
| `PR-omarchy-gaming-system-own-headless-qt-test-environment-001` | Ticket 022 headless platform inheritance failure | Yes — deterministic QML validation must force offscreen/software rendering. |
| `PR-omarchy-gaming-system-mirror-authoritative-client-response-bounds-001` | Ticket 022 hostile client-envelope inspection | Yes — visual refactoring must retain exact bounded response validation. |
| `PR-omarchy-gaming-system-preserve-bodyless-qml-requests-001` | Ticket 023 XHR keep-alive failure | Yes — presentation work must not perturb the existing request boundary. |
| `PR-omarchy-gaming-system-assert-minimum-layout-after-settle-001` | Ticket 024 home-grid overflow | Yes — every screen needs settled geometry evidence at 640×420 and 920×600. |
| `PR-omarchy-gaming-system-bind-presentation-cardinality-before-indexing-001` | Ticket 024 hostile game envelope inspection | Yes — shared rows and presenter polish must remain downstream of exact identity/cardinality validation. |
| Current QML source inventory | Direct `rg` over `Main.qml`, shared components, game surface, and screens | Yes — focus/accessibility primitives exist, but raw style/state conventions are duplicated and full-screen semantics are incomplete. |

## What happened

Ticket 025 turned the separately delivered QML slices into one coherent player
surface without changing any API, controller, authentication, social, or game
authority. `OgsTheme` now owns the semantic palette, typography, spacing,
focus, and control geometry. Shared heading, status, section, card, button,
field, area, and row primitives carry that contract through `Main.qml`, all ten
routed screens, Signal Siege, and the trusted cartridge renderer. Every visual
`Text` object explicitly chooses plain text, and visible state always includes
a non-color label.

Inspection exposed two timing-specific focus defects: gameplay actions did not
exist when the Loader performed its initial handoff, and a populated persona
delegate materialized after the same callback. The screens now restore focus
after the authoritative presentation or delegate exists, without dispatching
an action or changing authority. Inspection also expanded the theme boundary to
the complete trusted cartridge surface/node library and strengthened the style
checker from token rejection to a balanced QML-object policy that forbids
implicit `AutoText`.

The final focused corpus passed 38 Qt cases and covers contrast, every routed
heading/status/navigation landmark, deterministic initial focus, reversible
Tab traversal, Escape authority, compact/default layout, and hostile controller
envelopes. The 33-file source policy and trusted-renderer suite passed. A final
Codex Security diff scan found zero reportable findings, OpenWiki completed,
and the first full 18-stage diff gate was green before Phase 5 reconciliation.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-asynchronous-qml-focus-handoff-001` | Loader-level initial focus ran before an authoritative gameplay action or delayed persona delegate existed, so keyboard focus could remain on a non-action container. | Production-root accessibility fixture on Gameplay and Persona routes. |
| `BF-omarchy-gaming-system-partial-trusted-visual-policy-scope-001` | The first centralized-theme pass covered the shell, screens, and Signal Siege but left the trusted cartridge preview and fixed node library on a duplicate raw palette. | Phase 3.5 direct visual-boundary inventory. |
| `BF-omarchy-gaming-system-qml-plain-text-policy-default-gap-001` | Rejecting explicit rich-text tokens still allowed a newly added `Text` object to inherit Qt's automatic text-format default. | Phase 3.5 provenance review of `scripts/check-qml-style.py`. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-restore-focus-after-qml-materialization-001` | When a routed QML focus target depends on asynchronous data or delegate creation, restore focus only after the enabled target materializes and prove that handoff through the production root. | Loader completion proves component creation, not readiness of data-dependent child actions or virtualized delegates. |
| `PR-omarchy-gaming-system-scope-style-policy-to-the-trusted-visual-boundary-001` | A centralized UI contract and its source policy must inventory every platform-owned visual surface, including trusted cartridge renderer nodes, rather than only the main application routes. | A second trusted palette silently defeats consistency, contrast, and future enforcement claims even when game data remains inert. |
| `PR-omarchy-gaming-system-require-plain-text-on-every-qml-text-object-001` | Parse every in-scope QML `Text` object and require explicit `Text.PlainText`; rejecting named rich formats alone is insufficient. | Qt's default automatic detection can interpret markup even when no unsafe format token appears in source. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-host-owned-semantic-qml-theme-001` | One repository-owned semantic theme and component vocabulary governs the main shell, all player routes, compiled-game presentation, and trusted cartridge rendering; controller and cartridge data may select content/state but never styling or executable presentation. | `docs/architecture/system-overview.md`; `docs/architecture/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. All nine EARS requirements have direct production-root, source-
policy, hostile-fixture, renderer, live-smoke, or canonical-gate evidence.
Inspection materially corrected asynchronous focus, expanded the trusted visual
boundary, and closed implicit automatic-text drift before completion. The
durable product, architecture, roadmap, OpenWiki, AAR, and knowledge records now
agree with the implementation. The final post-archive diff gate passed all 18
stages and its receipt matches the completed gated state. Git delivery is
intentionally separate and still requires explicit authorization.
