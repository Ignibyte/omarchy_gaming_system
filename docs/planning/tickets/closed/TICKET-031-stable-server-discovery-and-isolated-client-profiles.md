---
title: TICKET-031-stable-server-discovery-and-isolated-client-profiles
status: closed
ticket_number: 031
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/stable-server-discovery-and-isolated-client-profiles.spec.md
---

# TICKET-031-stable-server-discovery-and-isolated-client-profiles

## Summary

Give every owner-operated OmarchyGS community a durable public identity and
versioned capability document, then let the keyboard-first QML client save,
select, and remove multiple non-secret server profiles without mixing authority
or silently accepting a changed server identity.

## Why

The client can already connect to an arbitrary compatible origin, but it treats
the endpoint as a single editable string and recognizes a server only by the
generic health document. Players need to move among independent communities as
deliberately as they choose game systems: each origin must resolve to a stable
community identity, saved client state must remain server-scoped, and a reused
origin must not silently become a different trusted community.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When migration `0018` initializes a database, the system shall create exactly one random server identity that remains stable across process restarts and database backup/restore. | PostgreSQL migration constraints plus restart/restore integration evidence. |
| REQ-002 | When a client requests public server discovery, the server shall return one exact bounded document containing the stable server UUID, operator-configured public name, protocol version, and deterministic implemented capability set without account, credential, database, provider-secret, or operator-private data. | Router/config/unit and PostgreSQL API tests for exact success, degraded database, bounds, ordering, and field absence. |
| REQ-003 | When server-name configuration is absent or invalid, startup shall use the documented safe default or fail before listening, and changing the public name shall not change the durable server UUID. | Configuration tests and real restart smoke. |
| REQ-004 | When a player saves a discovered server, the QML client shall persist only its canonical origin, stable UUID, bounded public name, and supported protocol metadata in a bounded deduplicated profile inventory. | QML persistence/reload tests plus serialized-setting secret and schema inspection. |
| REQ-005 | When a player connects through a saved profile, the client shall require the discovery UUID to match the pinned UUID and shall fail closed without replacing the profile or exposing account access when the origin now identifies a different server. | Hostile fixture identity-change tests and exact state assertions. |
| REQ-006 | When a player switches, removes, or directly replaces a server selection, the client shall clear bearer, MFA, invitation, persona, social, inbox, challenge, and game authority before issuing requests under the new origin. | Production-root QML tests with request and controller-state observation. |
| REQ-007 | When saved-profile state is malformed, oversized, duplicated, unsupported, or contains fields outside the public profile schema, the client shall discard or reject it safely and shall never auto-connect or interpret persisted credential-like fields. | Hostile settings fixtures and bounded parser tests. |
| REQ-008 | When a server advertises protocol or capabilities, the client shall accept supported protocol v1 with the required onboarding capability subset, tolerate bounded unknown capabilities, and present a fixed incompatible-server state when required capability negotiation fails. | Discovery compatibility matrix in transport fixtures and real API smoke. |
| REQ-009 | When the connection screen presents saved servers, every save/select/remove/direct-connect action shall remain keyboard-operable, explicitly accessible, plain text, and contained at 640×420 with clear empty, selected, mismatch, offline, and incompatible states. | QML keyboard/accessibility/visual-policy fixtures. |
| REQ-010 | When the complete development and package workflows run, they shall prove two distinct server profiles remain isolated across selection and client restart while the existing registration, authentication, persona, social, inbox, challenge, and gameplay flow remains green. | Extended live QML smoke, package-source/artifact checks, and canonical diff gate. |

## Scope

- In:
  - forward-only singleton server identity persistence;
  - bounded operator-configured public server name;
  - public protocol/capability discovery endpoint;
  - non-secret bounded QML saved-profile persistence and management;
  - identity pinning, protocol/capability negotiation, authority clearing;
  - two-server fixture/live evidence, docs, OpenWiki, and gate integration.
- Out:
  - federation, cross-server accounts/personas/social graphs, shared sessions,
    global recovery, or account migration;
  - certificate/public-key pinning, custom certificate authorities, DNS
    ownership proofs, or replacing HTTPS validation;
  - persistent bearer tokens, passwords, invitation codes, MFA challenges,
    recovery codes, or automatic login;
  - marketplace/catalog synchronization, cartridge acquisition, remote server
    administration, server identity rotation, merge, or fork tooling.

## Links

- Intake: first engineering outcome after private-alpha software readiness
- Pipeline spec: [completed spec](../../pipeline/completed/stable-server-discovery-and-isolated-client-profiles.spec.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Operator boundary: [owner-operated servers](../../../operators/owner-operated-servers.md)
