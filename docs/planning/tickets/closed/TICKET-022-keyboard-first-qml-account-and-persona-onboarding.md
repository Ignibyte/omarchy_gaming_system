---
title: TICKET-022-keyboard-first-qml-account-and-persona-onboarding
status: closed
ticket_number: 022
type: feature
created: 2026-08-25
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/keyboard-first-qml-account-and-persona-onboarding.spec.md
---

# TICKET-022-keyboard-first-qml-account-and-persona-onboarding

## Summary

Replace the health-only connector with a keyboard-first QML access shell that
can register or sign in, complete an existing MFA login challenge, and create
or select an owned persona before entering the authenticated game system.

## Why

The server already implements the private-alpha identity foundation, but the
shipped client exposes only `/health`. This is the smallest client slice that
turns a clean Omarchy installation into an actual player endpoint without
mixing social, challenge, and gameplay navigation into the authentication
boundary.

## Outcome

All eight requirements passed. The flagship QML client now provides safe
endpoint selection, account registration, password or MFA sign-in, owned
persona creation or selection, an authenticated home, and explicit local
logout with process-memory-only authority. The 19-case hostile keyboard and
transport corpus, both real migrated QML onboarding scenarios, Codex Security
review, OpenWiki lifecycle, and final 18-stage delivery gate all passed; the
gate included 44 PostgreSQL integration tests plus provider and clean-clone
authority proofs.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the connector starts with its default local endpoint or an explicit server endpoint, it shall admit only loopback HTTP or HTTPS, probe health with a bounded request, and present distinct connecting, ready, offline, configuration-error, and protocol-error states without sending credentials before readiness. | QML fixture matrix and live smoke |
| REQ-002 | When a player chooses registration, the connector shall submit the exact v1 account contract, keep the password masked, render stable server validation/conflict errors without leaking response internals, clear secret input after use, and return the new canonical username to sign-in. | QML interaction and real API smoke |
| REQ-003 | When valid primary credentials create a device session, the connector shall retain the bearer token only in process memory, use it only in the Authorization header for authenticated API calls, clear credential fields, and erase authenticated state on logout or invalid-session response. | QML state/negative fixture tests and live smoke |
| REQ-004 | When primary login returns an MFA challenge, the connector shall preserve only that in-memory challenge until success, expiry, cancel, or logout; accept a TOTP or recovery code; handle retryable factor errors; and enter the same authenticated flow after successful completion. | MFA fixture matrix and PostgreSQL-backed QML smoke |
| REQ-005 | When authentication succeeds, the connector shall load only owned personas, permit keyboard selection when one or more exist, offer persona creation when needed, and enter the shell with exactly one selected public persona without exposing account ownership. | Persona fixture/privacy tests and live smoke |
| REQ-006 | When requests overlap, time out, return malformed/unexpected bodies, or lose authorization, the connector shall ignore stale completions, expose a recoverable bounded error state, and never reuse a credential or response from the wrong operation. | Adversarial fixture and recovery tests |
| REQ-007 | When any onboarding screen is shown at 640×420 or larger, every action and field shall be reachable by keyboard alone with visible focus, plain-text error/status output, accessible names, predictable Enter/Escape behavior, and no pointer-only requirement. | Offscreen QML keyboard/accessibility smoke |
| REQ-008 | When the canonical delivery gate runs, it shall exercise the onboarding shell against both a hostile deterministic HTTP fixture and the real migrated OmarchyGS API before the existing health smoke may pass. | Focused client script and `bin/gate.sh --diff` |

## Scope

- In: validated server URL selection; health and request-state handling;
  registration; password login; existing TOTP/recovery-code login challenge;
  in-memory device token; owned-persona inventory, creation, and selection;
  logout/invalid-session reset; keyboard/accessibility behavior; deterministic
  hostile fixture coverage; real PostgreSQL/API/QML smoke; client, developer,
  API, architecture, and operator-facing documentation.
- Out: token persistence or a keyring; device-session inventory/revocation UI;
  TOTP enrollment/disablement UI; account or persona editing; connections,
  inbox, sync/WebSocket, challenges, game catalog/gameplay, cartridge dispatch;
  installer/package integration; new server endpoints or migrations; browser or
  mobile clients; Git delivery.

## Links

- Intake: none; selected from the ordered private-alpha roadmap after Ticket 019.
- Pipeline spec: `docs/planning/pipeline/completed/keyboard-first-qml-account-and-persona-onboarding.spec.md`
- Architecture: `docs/architecture/system-overview.md`
