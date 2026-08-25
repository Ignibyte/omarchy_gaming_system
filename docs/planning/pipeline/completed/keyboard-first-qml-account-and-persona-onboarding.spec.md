---
title: Keyboard-first QML account and persona onboarding
pipeline_id: e538f6de-de94-432e-80b1-d41da6ccc417
status: Phase 5 — Complete PASS
ticket: TICKET-022
ticket_doc: docs/planning/tickets/closed/TICKET-022-keyboard-first-qml-account-and-persona-onboarding.md
aar: docs/planning/knowledge/aar/AAR-022-keyboard-first-qml-account-and-persona-onboarding.md
created: 2026-08-25
---

# Keyboard-first QML account and persona onboarding — spec

## Intent

Turn the health-only QML connector into the first private-alpha player shell.
A clean client can select a safe server endpoint, register or authenticate,
complete an already-enabled MFA challenge, create or select one owned persona,
and arrive at a bounded authenticated home state using only the keyboard. The
slice deliberately keeps bearer material in memory and stops before social or
game navigation.

## Scope

- In: all eight Ticket 022 requirements; strict endpoint admission; bounded
  JSON requests and response validation; health, registration, password and
  MFA login, persona inventory/creation/selection, logout, invalid-session
  recovery, keyboard and accessibility semantics, hostile fixture proof, live
  PostgreSQL/API/QML proof, gate integration, documentation, OpenWiki, and AAR.
- Out: persistent credentials/keyring; session-management and MFA-enrollment
  settings; persona editing; social, inbox, sync/live, challenges, gameplay or
  cartridge presentation; packaging; server contract/schema changes.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the connector starts with its default local endpoint or an explicit server endpoint, it shall admit only loopback HTTP or HTTPS, probe health with a bounded request, and present distinct connecting, ready, offline, configuration-error, and protocol-error states without sending credentials before readiness. | Deterministic endpoint/health fixture matrix and live smoke |
| REQ-002 | When registration is submitted, the connector shall use the exact v1 contract, mask and clear the password, render allowlisted validation/conflict feedback, and carry only the canonical returned username into sign-in. | Fixture interaction matrix and real API registration smoke |
| REQ-003 | When primary login creates a session, the connector shall keep the bearer only in memory, attach it only as a Bearer Authorization header, clear credential input, and erase authenticated state on logout or invalid-session response. | QML state/header/privacy tests and live login/logout smoke |
| REQ-004 | When primary login requires MFA, the connector shall hold only that challenge in memory until success, expiry, cancel, or logout; accept TOTP or recovery input; preserve retryable challenge state; and enter the ordinary authenticated flow on success. | MFA success/error/expiry/cancel fixture matrix and PostgreSQL-backed smoke |
| REQ-005 | When authentication succeeds, the connector shall list only owned personas, allow keyboard selection, offer bounded persona creation, and enter its shell with exactly one selected public persona. | Persona shape/privacy fixture tests and live create/select smoke |
| REQ-006 | When network operations overlap, time out, return malformed or unexpected bodies, or lose authorization, the connector shall ignore stale callbacks, show a recoverable bounded error, and never cross-contaminate operation credentials or responses. | Hostile fixture ordering/timeout/schema/401 corpus |
| REQ-007 | When onboarding is used at the supported minimum size or larger, all fields and actions shall have accessible names, visible focus, keyboard-only traversal and activation, plain-text status/errors, and documented Enter/Escape behavior. | Offscreen focus/accessibility/input smoke at 640×420 and 920×600 |
| REQ-008 | When the canonical gate runs, it shall execute the deterministic QML onboarding corpus and a real migrated server onboarding path before accepting the client slice. | Focused script, live smoke, and `bin/gate.sh --diff` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | The existing QML connector calls the versioned REST API directly; no new client daemon, server endpoint, or WebSocket protocol is introduced. | REST is durable truth and the slice is an access shell, not a new authority layer. |
| 2 | Raw device and MFA challenge tokens live only in QML process memory and are cleared on every terminal transition; persistent sign-in waits for a reviewed OS keyring boundary. | QML settings or plaintext files are not acceptable credential storage. |
| 3 | Plain HTTP is accepted only for loopback development; every explicit non-loopback endpoint must use HTTPS and userinfo/fragments are rejected. | The client must not send passwords or bearer tokens over a remote plaintext link. |
| 4 | The client validates exact success response shapes and maps only stable error codes to player-facing messages; arbitrary server text is never treated as trusted UI markup. | This preserves protocol integrity and prevents accidental data/markup exposure. |
| 5 | This ticket ends at an authenticated shell with one selected persona. Social/inbox and challenge/gameplay navigation become separate tickets sharing the resulting client state boundary. | It produces a real vertical slice without making one unreviewable all-client rewrite. |
| 6 | Automated evidence includes deterministic hostile transport behavior and the real migrated server. Test-only automation is activated only by explicit smoke arguments and never carries production credentials. | Fixture control proves edge states while the live path proves contract integration. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-022-keyboard-first-qml-account-and-persona-onboarding.md`
- Architecture: `docs/architecture/system-overview.md`, `docs/product-charter.md`
- Dependencies: Tickets 004–006 and 008, plus the existing `client/qml/Main.qml` health probe.
- Intake: none.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | autonomous continuation and bounded requirements |
| 2 Design | Client state machine, transport/schema boundary, file manifest, regression plan, CodeGraph receipt | actionable no-secret-persistence design |
| 3 Implement | QML shell, tests, scripts, integration and docs | focused fixture and live checks |
| 3.5 Inspect | Correctness/security/concurrency/accessibility findings and fixes | resolved ledger and fresh CodeGraph receipt |
| 4 Validate | Focused client tests and canonical gate | matching gate receipt |
| 5 Complete | EARS audit, OpenWiki, AAR/knowledge, ticket/archive | matching completion and gate receipts |
| Delivery | Staged review and authorized commit/push | explicit delivery authorization |
