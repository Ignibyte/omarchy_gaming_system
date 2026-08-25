---
title: TICKET-018-production-remote-provider-security-foundation
status: open
ticket_number: 018
type: infrastructure
created: 2026-08-24
closed:
intake:
pipeline_spec:
---

# TICKET-018-production-remote-provider-security-foundation

## Summary

Design and implement the production-grade provider registry, broker identity,
grant, authenticated-message, egress, replay, quota, audit, and revocation
foundation without yet migrating gameplay authority.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an operator registers a provider release, the platform shall pin provider/game/version identities, approved TLS endpoint and keys, scopes, quotas, lifecycle state, and auditable rotation/revocation history. | Registry and authorization tests |
| REQ-002 | When the broker contacts a provider, it shall use guarded egress and short-lived sender-authenticated, audience/session/scope/pairwise-persona grants while disclosing no account ID, reusable device token, credential, or database capability. | Protocol and privacy tests |
| REQ-003 | When provider requests, responses, callbacks, redirects, DNS results, bodies, deadlines, concurrency, or signatures violate policy, the broker shall fail closed with bounded non-disclosing errors and operator-visible evidence. | SSRF/replay/failure corpus |
| REQ-004 | When a key, provider, release, or capability is suspended or revoked, new grants and launches shall stop according to explicit active-session policy without trusting WebSocket delivery. | Rotation/revocation tests |
| REQ-005 | When protocol conformance runs, it shall exercise TLS, signature/token binding, expiry, replay, idempotency, revision, retry, event deduplication, outage, and reconciliation against a separate fixture process. | End-to-end conformance environment |

## Scope

- In: provider registry/control plane, production cryptographic profile,
  guarded egress, pairwise identity, durable replay/audit state, quotas,
  revocation, conformance fixtures, observability, and docs.
- Out: delegating production game authority, migrating game snapshots, public
  self-service provider registration, provider-hosted UI, constitution changes,
  and Git delivery.

## Links

- Deferred until after the private-alpha cartridge/first-game milestones.
- Architecture: [ADR-0002](../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md)
