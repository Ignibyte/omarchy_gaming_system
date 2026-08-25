---
title: Production remote-provider security foundation
pipeline_id: 6f1b77ba-06f4-4c58-b908-171f00197018
status: Phase 5 — Complete PASS
ticket: TICKET-018
ticket_doc: docs/planning/tickets/closed/TICKET-018-production-remote-provider-security-foundation.md
aar: docs/planning/knowledge/aar/AAR-018-production-remote-provider-security-foundation.md
created: 2026-08-25
---

# Production remote-provider security foundation — spec

## Intent

Ship the dormant, production-grade trust and protocol foundation that a later
registered remote game provider will require: operator-controlled immutable
release identity, asymmetric grants and messages, pairwise persona identity,
guarded egress, durable replay/quota/audit state, fail-closed lifecycle policy,
and a separate-process conformance environment. This pipeline deliberately
does not delegate gameplay authority or expose a player-facing remote launch.

## Scope

- In: all five Ticket 018 requirements; provider and exact-release registry;
  approved HTTPS destination, provider keys, scopes, quotas, lifecycle and
  rotation/revocation history; short-lived sender-authenticated grants;
  pairwise persona subjects; canonical authenticated requests, responses, and
  callbacks; guarded DNS/IP/redirect/body/deadline/concurrency policy; durable
  replay and quota enforcement; sanitized errors and audit evidence;
  suspension/revocation admission policy; separate TLS fixture process;
  fault/conformance corpus; PostgreSQL migration and tests; operator docs;
  OpenWiki reconciliation; security inspection; AAR.
- Out: remote ownership of production game state, player-facing provider
  launch/command/callback routes, provider-hosted UI, direct client-provider
  traffic, public/self-service registration, migrating snapshots, accepting
  provider achievements into the platform ledger, Constitution changes,
  multi-provider federation, and Git delivery.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an operator registers or changes a provider release, the platform shall durably pin provider, game, rules, cartridge, HTTPS endpoint, TLS identity, message keys, scopes, quotas, lifecycle, active-session policy, and append-only rotation/revocation history. | PostgreSQL registry, immutability, rotation, lifecycle, and audit tests plus operator CLI smoke |
| REQ-002 | When the broker prepares or sends a provider operation, it shall use a guarded registered destination and a short-lived sender-authenticated grant bound to audience, exact release, platform session, one scope, pairwise persona subject, expiry, and replay identity, while disclosing no account ID, credential, reusable device token, or database capability. | Grant/message contract, pairwise identity, egress, and privacy tests |
| REQ-003 | When DNS, redirects, requests, responses, callbacks, bodies, signatures, replay state, deadlines, concurrency, or quotas violate policy, the broker shall fail closed with bounded non-disclosing errors and durable operator-visible evidence. | SSRF, signature, replay, quota, timeout, redirect, body-limit, concurrency, and audit corpus |
| REQ-004 | When an operator suspends or revokes a key, provider, release, or capability, the platform shall stop new grants and launch admission according to the pinned active-session policy without relying on WebSocket delivery. | Transactional key/provider/release/scope lifecycle and admission tests |
| REQ-005 | When protocol conformance runs, the platform shall exercise real TLS, grant and message signature binding, expiry, replay, idempotency, expected revision, retry, event deduplication, outage, and reconciliation against a separately spawned fixture provider. | Canonical provider-conformance script and full diff gate |

## Locked product decisions

| # | Decision | Reason |
|---|---|---|
| 1 | Ticket 018 installs a dormant production security boundary but does not connect it to current player/game routes. | Constitution §10 still assigns all production gameplay authority to OmarchyGS until Ticket 019 proposes and passes the explicit amendment and migration. |
| 2 | Provider control-plane mutations are operator-only and auditable; no public or cartridge-selected endpoint registration exists. | Endpoint, key, scope, quota, and lifecycle authority are security policy, not publisher-controlled gameplay data. |
| 3 | An exact provider release identity is immutable after registration; changes create a new release or append an explicit key/lifecycle event. | Stored sessions and receipts must never be relabeled through mutable endpoint, key, or version metadata. |
| 4 | The broker remains the only network principal crossing the provider boundary; clients and cartridges receive no endpoint or reusable provider credential. | Centralized egress, privacy, quota, replay, audit, and revocation controls are the accepted ADR-0002 boundary. |
| 5 | Every authenticated operation binds exact contextual identity and is recoverable from durable PostgreSQL receipts; WebSockets remain hints only. | Retry, replay, reconciliation, and revocation cannot depend on process memory or live notification delivery. |

## Phase 2 decisions

- A new production workspace crate, `omarchy-game-provider`, owns the dormant
  provider control plane, protocol, guarded client, durable receipts, and
  operator CLI. The current Axum player API does not depend on or instantiate
  it; Ticket 019 must cross that boundary explicitly after the Constitution
  amendment and single-authority migration pass.
- PostgreSQL migration 0014 normalizes providers, immutable exact releases,
  release scopes, append-only operational keys, grant records, quota windows,
  expiring concurrency leases, operation receipts, inbound message receipts,
  and append-only security audit events. Database constraints and triggers
  prevent release/key identity rewrites and audit mutation; operator changes
  lock the provider root and append a bounded reasoned event.
- Exact release identity pins provider ID, release ID, game/rules/cartridge
  identity, canonical HTTPS origin/base path, lifecycle, active-session
  policy, scopes, and bounded quotas. Message-key and TLS-root rotations append
  new immutable key rows with overlap windows; revocation is immediate and a
  terminal release/provider revocation cannot be reversed.
- Grants use Ed25519 over retained canonical JSON bytes with a domain-separated
  v1 prefix. Each grant has one scope and binds issuer, provider audience,
  exact release/game/rules/cartridge, platform session, HMAC-SHA-256 pairwise
  persona subject, issued/expiry time, and token UUID. Lifetime is at most 60
  seconds. Grant rows retain only pairwise identity, never account or raw
  persona identity.
- HTTP requests, responses, and callback-shaped events use a strict fixed
  OmarchyGS profile of RFC 9421 with Ed25519 and RFC 9530 `Content-Digest`.
  Covered components include method/authority/path or status, request context
  for responses, content digest/type, provider ID, release UUID, and message
  UUID. Signature parameters bind created/expiry, nonce, key ID, algorithm,
  and protocol tag. Unknown, duplicate, malformed, stale, future, mismatched,
  or unregistered components fail closed.
- Guarded egress accepts a canonical HTTPS DNS endpoint only. It performs one
  bounded resolution, rejects the whole result if any address is loopback,
  private, link-local, multicast, unspecified, documentation, benchmarking,
  reserved, or otherwise non-global, and pins the accepted socket addresses
  into a one-operation client. Proxies, redirects, referers, transparent
  decompression, TLS early data, and non-HTTPS requests are disabled. The
  client trusts only active operator-registered DER TLS roots, bounds connect,
  read, and total deadlines, and stops response bodies while streaming.
- PostgreSQL quota windows cover grants, outbound requests, and callbacks.
  Expiring database concurrency leases provide a cross-process ceiling and
  crash recovery. Durable operation receipts bind one idempotency UUID to the
  exact release/session/scope/expected revision/request digest before the
  network call; exact retry reuses it, while changed input conflicts. Inbound
  event receipts resolve exact replay before applying a new disposition and
  reject the same ID with different authenticated bytes.
- Lifecycle admission is a pure, exhaustively tested matrix over provider,
  release, key, scope, operation class, new/existing session, and the pinned
  active-session policy. Suspension always stops new launches. Revocation
  stops all new grants and operations immediately; suspended existing sessions
  may only reconcile or continue commands when their explicit pinned policy
  permits it. No WebSocket state participates.
- The provider conformance test spawns a distinct TLS fixture process with
  ephemeral keys and state outside the repository. The production profile
  cannot authorize loopback; a compile-time conformance-only profile admits
  only the exact loopback socket and generated trust root. The corpus covers
  signatures, grants, TLS trust, expiry, replay, idempotency, revision,
  retry-after-timeout, event deduplication, redirect, oversized streaming
  bodies, outage, and reconciliation.

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-018-production-remote-provider-security-foundation.md`
- Architecture: `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md`, `docs/architecture/game-cartridges.md`
- Predecessor proof: `docs/planning/pipeline/completed/portable-games-sdk-and-remote-hosting-spike.notes.md`
- Intake: none; Ticket 018 was opened by the accepted Ticket 014 follow-up sequence.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket reconciliation, bounded security-foundation scope, active spec/notes, open AAR | authority boundary and exclusions fixed |
| 2 Design | Data/protocol/egress/lifecycle design, file manifest, threat model, regression table | actionable design plus worktree-bound CodeGraph receipt |
| 3 Implement | Registry, protocol, guarded client, durable controls, fixture, migrations, docs, tests | focused loop green |
| 3.5 Inspect | Correctness, crypto, SSRF, privacy, concurrency, database, operations, and simplification ledger | findings resolved plus fresh CodeGraph receipt |
| 4 Validate | Complete failure/conformance corpus and canonical diff gate | matching delivery receipt |
| 5 Complete | Requirement audit, OpenWiki, docs, submitted AAR, ticket close and archive | matching OpenWiki and delivery receipts |
| Delivery | Authorized commit/push only | explicit user authorization |
