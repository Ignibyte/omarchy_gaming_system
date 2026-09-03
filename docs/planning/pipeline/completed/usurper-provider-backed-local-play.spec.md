---
title: Usurper Provider-Backed Local Play
pipeline_id: 91f08583-7519-448d-9c69-7e8790d469bf
status: Phase 5 — Complete PASS
ticket: TICKET-061
ticket_doc: docs/planning/tickets/closed/TICKET-061-usurper-provider-backed-local-play.md
aar: docs/planning/knowledge/aar/AAR-061-usurper-provider-backed-local-play.md
created: 2026-09-02
---

# Usurper Provider-Backed Local Play — spec

## Intent

Ship the smallest honest visible-play loop for the separate Usurper provider:
real provider-owned state and reducer transitions, exact signed presentation,
the platform-owned renderer, and trusted QML, without crossing the development,
admission, persistence, or publication boundaries.

## Scope

- In:
  - explicit signed-screen selection in the development preview compiler;
  - a non-packaged development-only trusted QML local-play shell;
  - a loopback, capability-protected, in-memory Usurper provider driver;
  - provider-command and signed-navigation behavior with stale-request and
    render-before-commit protection;
  - automated and visible proof, including workspace 8 for the live view;
  - clear non-interactive labeling for the existing fixture preview.
- Out:
  - new game rules, state schema, content, database rows, migrations, public
    protocol, platform gameplay authority, registration, admission, deployment,
    delivery, or publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a developer launches local Usurper play, the system shall start one in-memory `UsurperGame` session, compile its view from the exact signed cartridge through the platform renderer, and display it through the trusted QML surface with an explicit development-only label. | Local-play script smoke and visible workspace-8 verification. |
| REQ-002 | When the player requests a provider action declared by the currently rendered plan, the shell shall apply it through `ProviderGame`, compile the resulting view, and publish the new state only after both operations succeed. | Rust reducer/commit tests and local HTTP integration. |
| REQ-003 | When the player requests a current signed `navigate.*` action, the shell shall render its authenticated target without mutating provider state; when an action is stale, undeclared, malformed, or carries an unsupported payload, the shell shall reject it without state mutation. | Rust navigation, revision, declaration, payload, and rollback tests. |
| REQ-004 | While the local-play service is running, it shall bind only to loopback, require an unguessable session capability for state/action requests, bound request and response data, and serve only renderer-emitted assets addressed by validated tokens. | Focused HTTP integration and security inspection. |
| REQ-005 | When the signed fixture preview is opened, it shall visibly identify itself as non-interactive and shall not present enabled controls outside its automated input smoke. | QML smoke plus visible fixture review. |
| REQ-006 | When the slice is validated, existing provider, signed-cartridge, renderer, QML, and platform delivery gates shall remain green without database, protocol, admission, registration, deployment, or publication changes. | Focused suites, full external suite, platform `bin/gate.sh --diff`, and scope review. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | The local shell invokes the existing `ProviderGame` adapter and never reimplements Usurper rules in QML or the platform. | The provider remains the sole game authority. |
| 2 | Every visible plan is compiled from the exact signed archive by the platform renderer; no fixture or hand-authored plan is accepted as live state. | Visible play must preserve the established cartridge trust boundary. |
| 3 | Provider actions and `navigate.*` actions remain distinct, and only successful provider actions advance provider revision. | This matches the production client contract and prevents navigation from becoming game state. |
| 4 | Candidate provider state is committed only after its view successfully renders. | A renderer failure must not leave the visible shell behind authoritative local state. |
| 5 | The service is an ephemeral loopback development harness with capabilities, bounded bodies, and validated current-plan actions; it owns no database or platform credential. | The slice proves local play without implying production admission or persistence. |
| 6 | The fixture preview remains a rendering tool and becomes visibly disabled outside smoke mode. | Rendering proof and gameplay proof stay explicit rather than visually ambiguous. |

## Linked artifacts

- Ticket: [TICKET-061](../../tickets/closed/TICKET-061-usurper-provider-backed-local-play.md)
- Architecture: [game-cartridges.md](../../../architecture/game-cartridges.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | concrete shippable contract |
| 2 Design | Architecture, file manifest, regression plan | worktree-bound CodeGraph receipt |
| 3 Implement | Platform preview/QML and external provider harness | compile and self-review |
| 3.5 Inspect | Findings ledger and security review | lead disposition |
| 4 Validate | Focused/full tests and delivery gate | matching gate receipt |
| 5 Complete | AC audit, docs, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | explicit user authorization only |
