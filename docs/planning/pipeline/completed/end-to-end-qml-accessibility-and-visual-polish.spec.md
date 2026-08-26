---
title: End-to-end QML accessibility and visual polish
pipeline_id: 3c97b5a7-d0e8-4d3e-a28a-e028358fe255
status: Phase 5 — Complete PASS
ticket: TICKET-025
ticket_doc: docs/planning/tickets/closed/TICKET-025-end-to-end-qml-accessibility-and-visual-polish.md
aar: docs/planning/knowledge/aar/AAR-025-end-to-end-qml-accessibility-and-visual-polish.md
created: 2026-08-25
---

# End-to-end QML accessibility and visual polish — completed spec

## Intent

Complete the next private-alpha roadmap slice by turning the already functional
QML vertical slices into one coherent keyboard-first interface. The product
shall use consistent semantic styling, screen landmarks, status feedback,
focus behavior, and responsive layouts across the complete first-playable flow,
with deterministic evidence at the supported minimum and default sizes.

## Scope

- In: all nine Ticket 025 requirements; production QML shell/screens/shared
  controls; platform-owned theme and accessibility primitives; hostile,
  keyboard, focus, layout, and contrast evidence; documentation, OpenWiki,
  security inspection, and AAR.
- Out: server, API, migration, authentication, social, and game semantic
  changes; live-sync scheduling; installer/package work; keyring integration;
  external executable frontend code; cartridge acquisition; OS-specific
  assistive-technology certification; Git delivery.

## Acceptance criteria (EARS)

The authoritative acceptance criteria are REQ-001 through REQ-009 in
[`TICKET-025`](../../tickets/closed/TICKET-025-end-to-end-qml-accessibility-and-visual-polish.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Preserve the current Rust/API/database and controller authority contracts; this is a QML presentation, navigation, and evidence slice. | Accessibility polish must not silently change authentication, persona, social, challenge, or game semantics. |
| 2 | Keep the supported window contract at 640×420 minimum and 920×600 default, and make content responsive/scrollable inside it rather than raising the minimum. | The existing Omarchy connector contract and prior regression evidence depend on the smaller footprint. |
| 3 | Centralize semantic colors, typography, spacing, focus, and state presentation in trusted repository-owned QML primitives. | Raw repeated values already drift across vertical slices and cannot provide one testable contrast/state contract. |
| 4 | Every state remains understandable through text, label, iconography or geometry in addition to color; untrusted text remains `Text.PlainText`. | Color-only state and implicit rich text are incompatible with the accessibility and trust boundaries. |
| 5 | Use deterministic property, keyboard, geometry, and contrast tests instead of platform-dependent golden screenshots as the delivery proof. | Software-rendered pixel output can vary by Qt/font/platform; semantic layout and palette invariants are stable and reviewable. |
| 6 | Expose a coherent QML accessibility tree and keyboard contract, but do not claim OS-specific screen-reader certification in this slice. | Certification requires a real target matrix and assistive-technology lab beyond the current private-alpha gate. |
| 7 | Add no decorative motion that can gate comprehension or action. | The product does not need animation to feel polished, and reduced-motion policy should be designed before motion is introduced. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-025-end-to-end-qml-accessibility-and-visual-polish.md`
- Architecture: `docs/architecture/system-overview.md`, `docs/architecture/game-cartridges.md`
- Dependencies: Tickets 022–024 and the existing QML fixture/live harness.
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, EARS scope, active spec/notes, open AAR | bounded QML-only private-alpha polish slice |
| 2 Design | Semantic UI contract, screen/focus map, exact file manifest, regression table, CodeGraph receipt | actionable design with no authority drift |
| 3 Implement | Shared primitives, full-screen adoption, and deterministic fixture coverage | focused QML/source-policy checks |
| 3.5 Inspect | Correctness, accessibility, visual, security/provenance, reuse, and blast-radius ledger | resolved findings and fresh CodeGraph receipt |
| 4 Validate | Focused QML suite and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki, docs, AAR/knowledge, ticket/archive | no silent drops and matching completion receipt |
| Delivery | Fresh gate, staged review, separately authorized commit/push | explicit delivery authorization |
