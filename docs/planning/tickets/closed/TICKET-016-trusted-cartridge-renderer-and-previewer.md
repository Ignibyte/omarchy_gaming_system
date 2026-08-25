---
title: TICKET-016-trusted-cartridge-renderer-and-previewer
status: closed
ticket_number: 016
type: feature
created: 2026-08-24
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/trusted-cartridge-renderer-and-previewer.spec.md
---

# TICKET-016-trusted-cartridge-renderer-and-previewer

## Summary

Build the trusted keyboard-first Cartridge Core/Rich-2D renderer and local
previewer that interpret verified presentation data without loading publisher
QML, JavaScript, native code, URLs, or filesystem capabilities.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a verified cartridge screen renders, the client shall instantiate only versioned platform-owned node components and bind only schema-validated view fields and declared actions. | QML/component integration tests |
| REQ-002 | When a player uses keyboard, pointer, assistive semantics, scalable text, high contrast, reduced motion, or muted audio, every Core control and state surface shall remain operable with deterministic focus and declared fallbacks. | QML interaction/accessibility matrix |
| REQ-003 | When a cartridge or view exceeds node, asset, animation, particle, audio, memory, or frame budgets, the renderer shall reject or degrade optional presentation without losing the trusted shell or reporting an unconfirmed command. | Stress fixtures and minimum-hardware profiling |
| REQ-004 | When loading, offline, stale, empty, protocol-error, unsupported-capability, or revoked states occur, the renderer shall display trusted origin/state chrome and prohibit cartridge-authored credential prompts. | State and phishing-containment tests |
| REQ-005 | When a game repository runs the previewer, it shall use the production parser/component vocabulary and export a conformance report without platform credentials or database access. | Previewer parity/isolation tests |
| REQ-006 | When the Core and Rich-2D profiles are published, OmarchyGS shall document the measured game genres and effects each profile can sustain, the minimum tested CPU/GPU/software-rendering hardware, ratified hard and soft budgets, and deterministic fallback behavior. | Reproducible benchmark matrix and profile documentation |

## Scope

- In: Core and calibrated Rich-2D components, actions, focus/accessibility,
  failure states, previewer, profile benchmarks, resource containment, a
  practical genre/effect capability matrix, and docs.
- Initial Rich-2D vocabulary: trusted image, sprite, meter, particle-field,
  audio-cue, and button nodes in addition to Terminal/Grid/Status, all behind
  exact versioned capabilities and typed fallbacks.
- Out: raw QML/JS, custom shaders, WebEngine, 3D, remote providers, a complete
  launcher/catalog UI, production game art, and Git delivery.

## Links

- Depends on: `TICKET-015`
- Pipeline: [completed spec](../../pipeline/completed/trusted-cartridge-renderer-and-previewer.spec.md)
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md)
