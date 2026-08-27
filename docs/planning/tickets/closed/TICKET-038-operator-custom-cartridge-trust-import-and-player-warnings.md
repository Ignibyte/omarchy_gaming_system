---
title: TICKET-038-operator-custom-cartridge-trust-import-and-player-warnings
status: closed
ticket_number: 038
type: feature
created: 2026-08-27
closed: 2026-08-27
intake:
pipeline_spec: docs/planning/pipeline/completed/operator-custom-cartridge-trust-import-and-player-warnings.spec.md
---

# TICKET-038-operator-custom-cartridge-trust-import-and-player-warnings

## Summary

Add a server-local, explicitly enabled path for importing and lifecycle-managing
publisher-signed inert cartridges under an operator signing authority, while
requiring players to opt into that exact server authority and showing durable
unvetted-content provenance everywhere the cartridge is selected, installed,
mounted, or used for presentation.

## Why

Owner-operated servers should be able to host private or experimental games
without pretending they passed marketplace review. The platform already has
the safe cartridge, renderer, server admission, acquisition, and historical
session boundaries; this slice must compose them without letting a server
silently replace the player's marketplace trust, deliver executable client
code, or turn custom presentation into gameplay authority.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | While operator-custom cartridges are not explicitly configured, the server shall expose no custom authority, accept no custom import/lifecycle operation, and preserve the existing discovery, catalog, and marketplace behavior. | Config/CLI/API exact-shape tests and existing regression suites. |
| REQ-002 | When an administrator configures a custom authority, the admin process shall accept only an absolute owner-private regular Ed25519 catalog key whose derived public identity matches the separately configured public key and stable server identity, while the normal server shall never read the private key. | Key path/mode/symlink/identity hostile corpus and process-boundary inspection. |
| REQ-003 | When an administrator imports a custom release, the system shall snapshot bounded input bytes once, verify the publisher-signed release against the production SDK and host profile, sign a server-scoped custom attestation and lifecycle policy, and stage only those verified immutable bytes. | Deterministic import tests with input mutation, archive/conformance/release/key/schema/media/capability tamper cases. |
| REQ-004 | When a custom import succeeds or is replayed, PostgreSQL shall atomically retain exact operator and publisher provenance, immutable release identity, lifecycle state, import result, idempotency identity, and append-only audit without any marketplace-review claim or partial database publication. | Migrated PostgreSQL success/replay/collision/rollback/failure tests. |
| REQ-005 | When an administrator changes a custom release lifecycle, the system shall require a monotonically newer signed policy, serialize competing writers, persist denial before enforcement, apply the existing active/deprecated/suspended/revoked/retired semantics, and append an immutable audit event. | Lifecycle, concurrency, restart, denial, replay, and audit tests. |
| REQ-006 | When an administrator lists inventory or players request the catalog, the system shall return a bounded exact provenance union in which existing marketplace-vetted responses remain unchanged and custom responses contain only operator-custom identity, warning, lifecycle, and server-admission facts. | Exact CLI/API JSON, sorting/bounds, mixed-source, and sensitive-field absence tests. |
| REQ-007 | When an administrator activates, deactivates, upgrades, or rolls back a game selection across vetted or custom sources, the system shall resolve one unambiguous exact release, preserve expected-state/idempotency/concurrency rules, increment admission revision, and audit the source transition. | Mixed-source selection race/replay/rollback and digest-collision tests. |
| REQ-008 | While custom content is enabled, discovery shall advertise one bounded server-scoped public authority and fingerprint without private material; changing or removing the advertised key shall never silently update an existing player trust decision. | Discovery exact-contract, restart, key mismatch, and privacy tests. |
| REQ-009 | When an authenticated player acquires a selected custom release, the server shall return a bounded custom acquisition envelope binding stable server identity, operator attestation, publisher release, current signed lifecycle policy, exact admission, and immutable bytes without manufacturing marketplace evidence. | Acquisition construction/verification, route auth, body bound, tamper, wrong-server/key/admission, and sensitive-field tests. |
| REQ-010 | When a player explicitly trusts a server's custom authority through the local companion, the client shall persist that exact server-origin/server-ID/key binding in a private descriptor-anchored store; absent trust, key replacement, malformed state, or cross-server reuse shall fail closed. | Companion enrollment/removal/restart/race/symlink/mode/key-substitution tests. |
| REQ-011 | When the client installs or refreshes a custom cartridge, it shall re-fetch catalog state before and after transfer, verify operator and publisher signatures plus policy and admission against the pinned per-server key, and mount exact content-addressed bytes with operator-custom provenance. | Remote TLS fixture, companion service, cache/mount, stale catalog, tamper, and offline tests. |
| REQ-012 | When custom content is listed or selected in QML, the client shall show a persistent plain-text unvetted warning, server/operator identity, and key fingerprint; trust and removal shall require explicit keyboard-accessible player actions, and install/play shall remain disabled until trust is current. | QML exact-schema, keyboard/focus/accessibility, warning, trust, hostile-envelope, and no-process-spawn tests. |
| REQ-013 | When a game session pins a custom cartridge, the system shall retain its source and custom provenance for current and historical presentation while continuing to route gameplay only through the existing compiled or registered-provider authority. | PostgreSQL session pin/action/history tests, companion historical acquisition, and provider/compiled authority regressions. |
| REQ-014 | When the platform or client restarts or the server backup is restored, custom release, lifecycle, admission, audit, player trust, cache, mount, and historical evidence shall recover without rollback, marketplace conflation, or secret disclosure. | Server recovery drill plus client restart/reconciliation tests. |
| REQ-015 | When Ticket 038 completes, focused contract/server/client/QML/recovery/security suites and the canonical diff gate shall pass, OpenWiki shall be reconciled, and external server modules, arbitrary client code, and marketplace-review claims shall remain absent. | Security diff inspection, CodeGraph inspection, OpenWiki lifecycle, acceptance audit, and `bin/gate.sh --diff`. |

## Scope

- In:
  - server-local custom catalog authority, verified release import, lifecycle,
    immutable audit, and mixed-source server admission;
  - a distinct server-scoped custom acquisition contract and per-server client
    trust enrollment/removal;
  - explicit catalog/mount/session/QML custom provenance and warnings;
  - restart, backup/restore, concurrency, tamper, historical-session, and
    existing-authority regression evidence.
- Out:
  - arbitrary publisher QML, JavaScript, WebEngine content, native client code,
    shell/install authority, or a server-supplied marketplace root;
  - automatic trust on connection, transparent custom-key rotation, federation,
    shared global identity, or cross-server custom trust;
  - custom game backend registration, the public Provider SDK, external
    provider onboarding, server modules/hooks, or executable plugin loading;
  - official hosted infrastructure and the external two-clean-installation
    acceptance event.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/operator-custom-cartridge-trust-import-and-player-warnings.spec.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
