---
title: TICKET-025-end-to-end-qml-accessibility-and-visual-polish
status: closed
ticket_number: 025
type: feature
created: 2026-08-25
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/end-to-end-qml-accessibility-and-visual-polish.spec.md
---

# TICKET-025 — End-to-end QML accessibility and visual polish

## Summary

Turn the complete private-alpha QML connector into one coherent, readable,
keyboard-complete player experience. The slice establishes shared visual and
accessibility semantics, then proves every current screen and dynamic state at
the supported minimum and default window sizes without changing server
authority or gameplay behavior.

## Why

The first playable now works end to end, but its screens were delivered in
separate vertical slices and still repeat raw styling, status, layout, and
focus conventions. Accessibility and visual polish are the next ordered
private-alpha roadmap gap and must be completed before packaging or external
testing can represent the connector as a single product.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When any connection, access, MFA, persona, home, social, inbox, games, challenges, or gameplay screen is shown, the system shall expose a consistent plain-text screen heading, current-state message, and navigation context whose meaning does not depend on color alone. | Production-root QML interaction matrix plus direct accessibility-tree/property assertions for every routed state. |
| REQ-002 | When a keyboard user enters a screen or the screen changes mode after an asynchronous response, the system shall place focus on a deterministic enabled control, retain a clearly visible focus indicator, and keep forward and reverse traversal within reachable enabled actions. | Qt Quick Test key traversal and focus-restoration cases at both supported sizes. |
| REQ-003 | When Escape is pressed from any non-destructive player subflow, the system shall return to the documented parent or cancel the local editing mode without discarding authenticated account, selected-persona, social, or game authority. | Production-root Escape matrix with controller-authority assertions. |
| REQ-004 | While the application is rendered at 640×420 or 920×600, every screen shall keep its headings, state feedback, and enabled controls inside the visible viewport or a keyboard-reachable scroll surface without overlap or horizontal clipping. | Settled-layout geometry audit across all routed screens and representative dynamic collections. |
| REQ-005 | When a shared button, text field, text area, row action, or status surface is enabled, disabled, focused, pressed, loading, successful, warning, or failed, the system shall render the state through centralized semantic styling with an accessible name and a non-color cue. | Component contract tests, QML source policy check, and visual-state fixture matrix. |
| REQ-006 | When normal-size text or essential focus and control boundaries are rendered, the system shall meet the project contrast thresholds of 4.5:1 for normal text and 3:1 for large text or graphical focus/control indicators against their effective backgrounds. | Deterministic palette contrast test using the centralized semantic color contract. |
| REQ-007 | When untrusted persona, message, challenge, catalog, session, or game-derived text is displayed, the system shall preserve explicit plain-text rendering, bounded wrapping or elision, and the existing trusted presenter/cartridge provenance boundaries. | Hostile fixture corpus, source inspection, and existing exact-envelope tests. |
| REQ-008 | When loading, offline, empty, completed, validation-error, protocol-error, or retryable-uncertainty states occur, the system shall expose stable human-readable and accessibility-readable feedback while preventing duplicate or unavailable actions. | Existing hostile transport/social/game fixtures expanded with shared status and action-state assertions. |
| REQ-009 | When the complete keyboard-only private-alpha fixture path runs, the system shall navigate account access, MFA, persona selection, home, social/inbox, games/challenges, and gameplay through the production `Main.qml` without mouse input or a focus dead end. | New full-shell Qt Quick Test scenario plus the existing live migrated two-account smoke path. |

## Scope

- In: the production QML shell, all current screens, shared controls and new
  platform-owned visual/accessibility primitives, deterministic theme and
  contrast validation, focus/layout/state interaction coverage, QML fixture
  support, relevant documentation, OpenWiki, security review, and AAR.
- Out: Rust/API/database changes; authentication or game-rule changes;
  WebSocket client scheduling; installer/package work; OS keyring work;
  publisher-provided executable UI; new cartridge/provider acquisition;
  animation-heavy effects; OS-specific screen-reader certification; Git
  delivery.

## Links

- Intake: none
- Pipeline spec: `docs/planning/pipeline/completed/end-to-end-qml-accessibility-and-visual-polish.spec.md`
- Architecture: `docs/architecture/system-overview.md`, `docs/architecture/game-cartridges.md`
