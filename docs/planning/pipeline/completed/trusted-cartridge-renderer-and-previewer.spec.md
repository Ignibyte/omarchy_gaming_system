---
title: Trusted Cartridge renderer and previewer
pipeline_id: 6494fb94-54e8-4fcf-b50d-9220c46f4564
status: Phase 5 — Complete PASS
ticket: TICKET-016
ticket_doc: docs/planning/tickets/closed/TICKET-016-trusted-cartridge-renderer-and-previewer.md
aar: docs/planning/knowledge/aar/AAR-016-trusted-cartridge-renderer-and-previewer.md
created: 2026-08-25
---

# Trusted Cartridge renderer and previewer — spec

## Intent

Ship the first production-owned visual runtime for verified cartridges: a Rust
render-plan compiler, trusted QML component vocabulary, and database/network/
credential-isolated previewer that turn authenticated presentation plus a
schema-valid view model into a keyboard-first Core or bounded Rich-2D screen.
Measure the exact software-rendering reference host and publish an honest
capability/genre envelope.

## Scope

- In: all six Ticket 016 EARS requirements; authenticated payload access;
  per-screen view-schema binding and validation; Terminal/Grid/Status plus
  trusted Image/Sprite/Meter/ParticleField/AudioCue/Button nodes; exact
  capability/fallback handling; fixed platform state/origin chrome; keyboard,
  pointer, accessibility, scalable-text, high-contrast, reduced-motion, and
  muted-audio behavior; a Rust-to-QML render-plan preview flow; Core and Rich-2D
  stress fixtures; benchmark/profile documentation; focused and canonical gate
  integration.
- Out: publisher QML or JavaScript, custom shaders, Canvas programs, WebEngine,
  video, 3D, remote or direct network access, provider authority, server routes,
  PostgreSQL migrations, launcher/catalog browsing, production game art,
  privileged/multi-user cartridge installation, public SDK publication, and
  Git delivery.

## Acceptance criteria (EARS)

The authoritative requirements are REQ-001 through REQ-006 in
[`TICKET-016`](../../tickets/open/TICKET-016-trusted-cartridge-renderer-and-previewer.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Rust verifies the cartridge, validates the pinned view schema, resolves bindings/assets/fallbacks, and emits a bounded render plan; QML never parses `.ogsc` or chooses paths/capabilities. | Keeps package, schema, and policy decisions outside the QML/JavaScript surface. |
| 2 | QML maps only allowlisted render-plan tags to repository-owned components and treats every string as plain text. | Publisher data never becomes QML, markup, a component URL, or executable expression. |
| 3 | Core adds Button, Image, and Meter to Terminal/Grid/Status; the first Rich-2D foundation adds Sprite, ParticleField, and AudioCue as versioned host capabilities. | Provides a useful retro-to-polished-2D step without claiming a general engine. |
| 4 | Non-ready states use fixed platform-owned labels and always-visible origin/state chrome; cartridge nodes cannot create credential input. | Keeps failure and authentication surfaces unspoofable. |
| 5 | The first published performance profile is the exact local software-rendering reference host, with measured budgets lowered when evidence requires it. | Avoids presenting Qt's theoretical ceiling or guessed hardware limits as a contract. |
| 6 | The previewer is an isolated developer path using the production verifier and plan compiler; it never connects to the server or provider. | Gives separate game repositories parity without widening runtime authority. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-016-trusted-cartridge-renderer-and-previewer.md`
- Architecture: `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md`, `docs/architecture/game-cartridges.md`
- Predecessor: `docs/planning/pipeline/completed/game-cartridge-contract-verifier-and-conformance-cli.spec.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Active spec/notes/AAR and exact first visual slice | scope and exclusions fixed |
| 2 Design | Render-plan/schema/component/profile contracts, file manifest, and regression map | CodeGraph receipt and actionable design |
| 3 Implement | Production Rust compiler, trusted QML, previewer, stress fixtures, focused gate | focused Rust/QML loop green |
| 3.5 Inspect | Correctness, input, QML, resource/GPU, action, and accessibility ledger | findings resolved plus fresh CodeGraph receipt |
| 4 Validate | Full matrix and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, benchmark docs, OpenWiki, AAR, ticket archive | matching OpenWiki and delivery receipts |
| Delivery | Authorized commit/push only | explicit user authorization |
