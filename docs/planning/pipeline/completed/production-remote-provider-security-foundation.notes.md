---
title: Production remote-provider security foundation — notes
pipeline_id: 6f1b77ba-06f4-4c58-b908-171f00197018
---

# Production remote-provider security foundation — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: no active bulletin or pipeline blocked selection. Ticket 019 depends
  on Ticket 018, so the existing open Ticket 018 is the highest-priority
  unblocked pipeline. The pinned CodeGraph 1.5.0 and OpenWiki 0.3.3 local tools
  passed `scripts/check-pipeline-tools.sh`.
- Recall: ADR-0002 authorizes only staged provider plumbing. Constitution §10
  still makes the compiled OmarchyGS runtime and PostgreSQL snapshot
  authoritative; this pipeline must not wire remote gameplay authority into
  production routes.
- Recall: `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` and
  the Ticket 014 proof establish broker-only traffic, pairwise persona
  identity, exact audience/game/release/session/scope/expiry binding, signed
  results, stable idempotency/revision, data-only presentation, and REST/cursor
  recovery.
- Recall: `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001`
  requires every cartridge, grant, request, receipt, and event to bind the
  registered provider and exact release/session/subject/scope/expiry context.
- Recall: `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001`
  requires response ceilings during streaming, before complete buffering.
- Recall: `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001`
  and `PR-omarchy-gaming-system-check-replay-before-current-revision-001`
  require genuine retries to resolve durable receipts before mutable current
  policy and revision checks where the authenticated identity still matches.
- Recall: Ticket 014 explicitly warns that its ephemeral keys, loopback HTTP,
  in-memory replay state, and compact proof limits are not production controls
  and must not be copied as if they were.
- Decision: scope is the complete dormant control-plane/protocol foundation in
  Ticket 018. Player-facing provider gameplay, result/achievement projection,
  and the authority amendment remain Ticket 019 work.

## Phase 2 — Design

- Architecture and data flow:
  1. An operator invokes `omarchygs-provider-admin apply <document>` with a
     local database credential. The control plane validates the complete
     command before opening a transaction, locks the provider root, changes
     only lifecycle/config state explicitly allowed by the command, and
     appends a non-secret immutable audit event with actor and reason.
  2. A future platform caller supplies an owned persona UUID only to the local
     grant issuer. The issuer loads and locks the exact registered release,
     evaluates lifecycle/scope/key policy, charges the durable grant quota,
     derives the pairwise subject, signs one short-lived one-scope grant, and
     stores its digest without the raw persona or account identity.
  3. The broker prepares canonical JSON, assigns a stable message and
     idempotency identity, stores the exact operation identity before I/O,
     resolves and validates the registered DNS name, acquires a durable quota
     and expiring concurrency lease, builds a one-operation HTTPS client pinned
     to those addresses and active registered TLS roots, signs the HTTP
     message, and streams the bounded response.
  4. Response verification authenticates RFC 9421/9530 fields and exact
     response bytes with an active registered provider key, then validates the
     body's provider/release/game/session/subject/idempotency/revision binding.
     A database transaction records the resulting receipt and audit outcome.
     Timeout remains unknown; exact retry reuses the durable idempotency
     identity and retrieves the provider's stable receipt.
  5. The dormant callback validator authenticates a callback-shaped request,
     charges callback quota, resolves exact event replay in PostgreSQL, and
     records only a bounded disposition. It applies no result, achievement, or
     notification projection in this ticket. Ticket 019 will place a thin
     transport route in front of this boundary when remote authority is
     constitutionally enabled.
  6. Current compiled games, `game_sessions.state`, Axum routes, persona sync,
     and QML remain unchanged. The new crate and schema are production-built
     and operator-usable but have no player-facing activation path.
- Cryptographic and HTTP profile:
  - Grant signatures are Ed25519 over `omarchygs-provider-grant-v1\0` plus the
    exact retained payload bytes. Payload and signature use unpadded base64url;
    input is bounded and parsed only after signature verification.
  - HTTP signatures use label `ogs` and a fixed RFC 9421 ordered component
    list. Requests cover `@method`, `@authority`, `@path`, `content-digest`,
    `content-type`, `x-ogs-provider`, `x-ogs-release`, and
    `x-ogs-message-id`. Responses additionally cover `@status` and the
    originating request method/authority/path through `;req`. Parameters are
    fixed to `created`, `expires`, `nonce`, `keyid`, `alg="ed25519"`, and
    `tag="omarchygs-provider-v1"`; duplicate fields/labels/components reject.
  - `Content-Digest` is exactly RFC 9530 SHA-256 over the transmitted bytes.
    Signature validity is exclusive at expiry, allows at most five seconds of
    future clock skew, and lasts no more than 30 seconds.
  - Provider message and TLS trust material are independent, public, immutable
    operational keys. Multiple active rows allow safe rotation overlap;
    revocation immediately removes a key from acceptance.
- Database and migration consequences:
  - `provider_registrations`: canonical provider ID, display name, lifecycle,
    terminal revocation and timestamps.
  - `provider_releases`: UUID plus immutable release/game/rules/cartridge and
    HTTPS endpoint identity, mutable audited lifecycle, active-session policy,
    config revision, and strictly bounded request/response/deadline/rate/
    concurrency limits.
  - `provider_release_scopes`: exact allowlisted scope plus active/suspended/
    revoked lifecycle; rows are never deleted.
  - `provider_release_keys`: immutable key ID/kind/material/digest/validity for
    Ed25519 message verification or DER TLS roots, with audited lifecycle and
    no delete/material rewrite.
  - `provider_grants`: token/release/session/pairwise subject/scope, claims
    digest and exclusive expiry; no account ID or raw persona ID.
  - `provider_quota_windows` and `provider_concurrency_leases`: serialized
    cross-process admission with bounded windows and expiring crash cleanup.
  - `provider_operations`: exact idempotency identity and request digest,
    status, attempt count, provider revision, bounded authenticated receipt,
    and timestamps; conflicting reuse rejects before network I/O.
  - `provider_message_receipts`: inbound response/callback message ID and
    authenticated digest/disposition for exact replay/deduplication.
  - `provider_security_audit_events`: append-only control, denial, failure,
    rotation, revocation, and protocol evidence with safe enums, bounded actor,
    correlation UUID, digest/code metadata, and no raw body or credential.
  - Migration 0014 is forward-only. Rollback is operational suspension or
    revocation; no production gameplay path depends on these tables yet and no
    downgrade deletes evidence.
- API and compatibility contract:
  - No public REST, WebSocket, QML, catalog, or compiled game behavior changes.
  - The Rust library exposes validated operator commands, grant issuance,
    operation preparation/execution, response verification, callback
    ingestion, reconciliation, and explicit sanitized error codes.
  - The operator CLI is the only control-plane adapter. It accepts a bounded
    tagged JSON command, reads `DATABASE_URL`, never accepts private provider
    signing material, emits bounded JSON, and never prints database credentials
    or platform signing/pairwise secrets.
  - Production egress has no private-network override. The separately compiled
    `provider-conformance` feature exposes only an exact-socket loopback policy
    for the canonical fixture and is never linked into the server.
- Exact file manifest:

  | File | Purpose |
  |---|---|
  | `Cargo.toml`, `Cargo.lock` | Add the production provider crate and reviewed transport/TLS/test dependencies. |
  | `crates/game-provider/Cargo.toml` | Define the library, operator binary, conformance-only fixture binary/feature, and dependencies. |
  | `crates/game-provider/README.md` | State the dormant authority boundary, operator workflow, security profile, and narrow validation commands. |
  | `crates/game-provider/src/lib.rs` | Export the bounded production provider APIs and shared error/result types. |
  | `crates/game-provider/src/model.rs` | Define and validate exact provider/release/key/scope/quota/lifecycle/operator inputs. |
  | `crates/game-provider/src/protocol.rs` | Implement pairwise subjects, signed grants, strict RFC 9421/9530 profile, exact message contracts, and context validation. |
  | `crates/game-provider/src/registry.rs` | Implement PostgreSQL registration, rotation, lifecycle, scope/quota policy, grants, admission, and safe audit reads/writes. |
  | `crates/game-provider/src/broker.rs` | Implement durable idempotency, quota/lease admission, guarded HTTPS execution, streamed bounds, response/callback receipts, and reconciliation. |
  | `crates/game-provider/src/egress.rs` | Canonicalize endpoints, classify DNS/IP results, pin validated addresses/TLS roots, and build fail-closed clients. |
  | `crates/game-provider/src/bin/omarchygs-provider-admin.rs` | Provide the operator-only bounded JSON control-plane adapter. |
  | `crates/game-provider/src/bin/omarchygs-provider-fixture.rs` | Run the compile-time-gated separate TLS provider with fault, idempotency, revision, event, and reconciliation behavior. |
  | `crates/game-provider/tests/protocol.rs` | Exercise grants, HTTP signatures/digests, binding, expiry, malformed fields, replay identity, and privacy. |
  | `crates/game-provider/tests/egress.rs` | Exercise endpoint canonicalization, private/special IP/DNS rejection, redirect policy, TLS trust, and bounds helpers. |
  | `crates/game-provider/tests/registry.rs` | Exercise PostgreSQL registration, immutability, rotation, quotas, concurrency, admission, audit, revocation, replay, and races. |
  | `crates/game-provider/tests/conformance.rs` | Spawn the real TLS fixture process and exercise the complete protocol/failure/retry/reconciliation path against PostgreSQL. |
  | `migrations/0014_provider_security_foundation.sql` | Add constrained provider control-plane, receipt, quota/lease, and immutable audit storage. |
  | `scripts/test-provider-conformance.sh` | Run the ignored separate-process TLS conformance test against the canonical PostgreSQL service. |
  | `bin/gate.sh`, `CONSTITUTION.md` | Ratchet provider conformance into DIFF/FULL delivery evidence and document the new gate. |
  | `docs/architecture/game-cartridges.md` | Ratify the production protocol, registry, egress, lifecycle, and still-dormant authority boundary. |
  | `docs/operators/provider-security.md` | Document registration/rotation/suspension/revocation, audit, quotas, recovery, and safe operator commands. |
  | Ticket/spec/notes/AAR/knowledge/OpenWiki | Preserve requirement evidence, findings, durable lessons, and completion receipts. |
- Regression and evidence plan:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Model/unit validation plus PostgreSQL tests register one exact release, reject malformed/duplicate/mutated identity, rotate overlapping message/TLS keys, update quotas with revision, enforce terminal revocation, and prove append-only reasoned audit; CLI smoke applies and reads a safe command. |
  | REQ-002 | Protocol tests prove one-scope short-lived grants and RFC 9421/9530 messages bind audience/provider/release/game/rules/cartridge/session/subject/expiry/replay; pairwise subjects differ by provider/game; serialized fixture traffic and durable rows contain no account ID, device token, raw persona, credential, or database capability. |
  | REQ-003 | Egress/protocol/registry/conformance corpus rejects non-HTTPS or noncanonical endpoints, userinfo/query/fragment, IP literals, every private/special IPv4/IPv6 class, mixed DNS answers, redirect, wrong TLS root, stale/future/tampered/mismatched signatures, malformed/oversized request/response/callback bodies, expired/replayed/conflicting identities, timeout, quota, and concurrency races; each failure returns a bounded public code and safe durable audit event. |
  | REQ-004 | PostgreSQL transition and admission matrix tests suspend/revoke provider, release, message/TLS key, and scope; assert new launch/grant denial, immediate terminal revocation, explicit existing-session policy, active-key overlap, no revival after revocation, and no WebSocket dependency. |
  | REQ-005 | `scripts/test-provider-conformance.sh` runs a separately spawned TLS fixture and proves valid launch/command, message binding, exact idempotent replay, stale revision, commit-then-timeout retry, signed event dedupe, outage, authenticated reconciliation, redirect/body/signature failures, and wrong TLS trust. The full diff gate runs this after PostgreSQL integration. |
- Security, privacy, concurrency, reconnect, and rollback risks:
  - SSRF/DNS rebinding: validate every answer, cap answers, then override DNS in
    the exact HTTPS client so no second resolution can change the destination.
  - TLS confusion: canonical host/SNI remains the registered DNS name; trust
    only active registered roots and reject system roots/proxies/redirects.
  - Signature confusion: fixed components, order, label, algorithm, tag,
    lengths, clock window, and exact registered key/context; hash the bytes
    actually sent/parsed and never sign reserialized untrusted JSON.
  - Replay/idempotency: transactionally resolve authenticated exact receipts
    before current revision/quota admission where safe; different bytes under
    the same identity conflict; revocation remains an immediate security gate.
  - Quota races: lock database windows and release roots; use unique expiring
    leases so process crashes recover without cluster-wide over-admission.
  - Audit amplification/privacy: callback/request quota is charged before
    expensive verification where an exact release header can be safely parsed;
    events store enums, UUIDs, digests, and bounded safe details, never bodies,
    grants, credentials, subjects from unauthenticated input, or network errors
    containing endpoint secrets.
  - Key rotation races: accept multiple active validity-overlapping public keys
    by exact key ID; a revoked key is never accepted even for a previously
    created message. Platform grants remain brokered and expire within 60s.
  - Timeout/reconnect: a timeout is unknown; durable operation identity survives
    retry and reconciliation compares signed provider revision/status/receipt,
    never timestamps. WebSockets do not enter the proof.
  - Migration/rollback: schema is additive and forward-only. Suspend/revoke the
    dormant release on operational failure; preserve all audit/receipt state.
- Decisions and material alternatives rejected:
  - Rejected copying the Ticket 014 envelope verbatim: it is not bound to HTTP
    method/authority/path/body digest and uses in-memory replay over plain HTTP.
  - Rejected a generic pluggable HTTP-signature parser: the first protocol has
    one fixed profile, so strict allowlisted serialization and parsing is a
    smaller ambiguity surface while remaining RFC 9421/9530 shaped.
  - Rejected system DNS after validation: a second lookup reopens rebinding.
    The selected client pins only the already validated socket set.
  - Rejected system TLS roots plus a stored fingerprint advisory: the client
    trusts only active operator-registered DER roots for the exact host.
  - Rejected process-local semaphores and counters as the only quota boundary:
    multiple server instances would exceed provider policy. PostgreSQL windows
    and expiring leases serialize the production ceiling.
  - Rejected a public admin API or cartridge-selected provider URL: provider
    registration is an operator control-plane action and credentials must not
    be distributed into the player plane.
  - Rejected wiring callbacks or remote launch into Axum now: that would create
    a partial production-authority path before Ticket 019's Constitution and
    single-owner migration gates.
- CodeGraph evidence:
  - The Phase 2 server-flow exploration found `AppState`/router construction in
    `app.rs`, migration/startup ownership in `main.rs`, authentication in
    `sessions.rs`, and the current game/challenge durable mutation boundaries.
    Because this ticket exposes no player route, those callers remain outside
    the implementation blast radius; migration execution and the canonical
    database gate are the only current server integration points.
  - The spike-flow exploration traced grant creation, pairwise derivation,
    broker streaming, provider grant consumption, receipt-before-revision
    ordering, signed results, event dedupe, and separate-process proof. It also
    confirmed the production gaps: loopback HTTP, ephemeral/in-memory state,
    no real TLS identity, no durable audit/quota/replay, and no HTTP component
    signature binding.
  - Direct inspection covered SQL migrations, shell gate/test harnesses, JSON
    fixtures, QML exclusions, ADR-0002, Game Cartridges architecture, OpenWiki,
    and the Ticket 014 completed evidence because CodeGraph does not own those
    source types.
  - Design receipt: the successful MCP explorations issued the worktree-bound
    `design` receipt for pipeline
    `6f1b77ba-06f4-4c58-b908-171f00197018` at gated-state hash
    `1c06b41584bf898b3b1d94d7fb1c784190668445579b5782162e68cccd27ec83`.

## Phase 3 — Implement

- Built: the new `omarchy-game-provider` workspace crate, migration 0014,
  strict provider/release/lifecycle/key/scope/quota models, operator JSON CLI,
  Ed25519 grants and pairwise subjects, the fixed RFC 9421/9530-shaped HTTP
  signature profile, DNS/IP/TLS guarded egress, PostgreSQL-backed grant/quota/
  lease/operation/message/audit controls, broker response and callback
  authentication, and sanitized stable errors. The crate remains absent from
  the player-facing server dependency graph.
- Built: a feature-gated TLS fixture binary with durable provider-side
  sessions/receipts and a canonical script that generates ephemeral trust
  material, spawns the provider separately, and proves launch, commands,
  exact replay, changed-intent conflict, expected-revision conflict, signed
  event dedupe, commit-then-timeout recovery, invalid signatures, redirects,
  streaming body ceilings, wrong TLS trust, outage, restart, and authenticated
  reconciliation.
- Built: PostgreSQL tests for immutable identities/audit, key overlap and
  terminal revocation, suspended active-session policy, scope revocation,
  pairwise privacy, grant/request quotas, concurrency leases, and a real
  simultaneous admission race; public protocol/egress hostile-input tests; an
  operator CLI subprocess smoke; provider README; operator runbook;
  architecture reconciliation; and DIFF/FULL gate 17.
- Focused evidence: `scripts/test-provider-conformance.sh` passed 12 library
  tests, 3 egress tests, 4 public protocol tests, 1 CLI test, 1 separate-process
  TLS conformance test, and 4 PostgreSQL registry tests. Focused all-target
  Clippy with `provider-conformance` passed with warnings denied. Rustdoc,
  whitespace, and pipeline-structure checks passed.
- Deviations: the design named only registry/conformance integration files;
  implementation added `tests/admin_cli.rs` so the stated CLI smoke is actual
  subprocess evidence rather than an inferred library check. No authority or
  public-route scope was added.
- Defects fixed during the loop: post-admission URL/signature/parse failures
  originally could leave an attempt/lease without immediate failure closure;
  rejected callbacks originally lacked a safe denial event; both now use
  common cleanup/audit paths. The first TLS run also exposed two cryptographic
  integration defects: conflicting Rustls crypto-provider features and random
  UUID nonces that sometimes violated the leading-letter identifier rule. The
  dependency graph is now ring-only and every generated nonce has an explicit
  `n-` prefix.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Concurrency / replay | `record_callback_receipt` tried to lock a receipt that might not exist. Concurrent exact first deliveries could both observe absence, race on the message/event unique indexes, and return `Conflict` for the loser instead of `Duplicate`. | Medium | Fixed: callback receipt transactions first lock the guaranteed release root, then read/insert the receipt. The separately spawned TLS conformance now delivers the same signed event concurrently and requires exactly one `Accepted` plus one `Duplicate`. |
| 2 | Network / SSRF | The IPv6 egress denylist omitted the RFC 8215 local-use NAT64 prefix and admitted reserved address space outside the allocated global-unicast block. A future NAT64 deployment could translate an accepted address to non-public IPv4. | Medium | Fixed: IPv6 production egress now requires `2000::/3` global-unicast allocation and rejects the current IANA special-purpose prefixes inside it. Unit coverage includes local-use NAT64, 6to4, documentation, AS112, ORCHID/DRIP, SRv6, and reserved-space examples. |
| 3 | Security scan / reachability | The frozen pre-fix Codex Security scan validated both defects but correctly found no currently reachable reportable vulnerability because the crate remains absent from the player server. | Informational | Both defects were still remediated before activation. A fresh post-fix scan (`c997858b-db23-4845-9048-3b0fef787b8a`) reviewed all 13 changed source surfaces and completed with zero findings. |
| 4 | Correctness / privacy / database | Exact release/grant/message bindings, current admission, key/lifecycle transitions, quota/lease locking, signed-response parsing order, pairwise-only provider identity, strict payload fields, and append-only audit were traced through all changed source and migration surfaces. | — | No additional confirmed defect. |
| 5 | Simplification / integration | CodeGraph found no player-server caller for the new broker and confirmed the only consumer is the feature-gated conformance path. Direct review covered SQL, shell, CLI, fixtures, and tests that graph coverage does not fully classify. | — | Dormant authority boundary preserved; no scope expansion or missed runtime integration found. |

- Post-fix evidence: `scripts/test-provider-conformance.sh` passed all 12
  library tests, 3 egress tests, 4 public protocol tests, the operator CLI
  subprocess, the separate TLS process test, and all 4 PostgreSQL registry
  tests. The TLS test includes simultaneous delivery of one signed callback.
- Security evidence: the pre-fix scan
  `7cd54afb-0a4c-42ac-8178-9a8ea3f400c2` preserved the two validated defects;
  the fresh post-fix scan `c997858b-db23-4845-9048-3b0fef787b8a` completed
  with zero findings over the new frozen snapshot. TAC status was unavailable
  because its connector was not authenticated; this did not reduce scan
  execution or source coverage.
- CodeGraph inspection receipt: pipeline
  `6f1b77ba-06f4-4c58-b908-171f00197018`, gated state
  `5e2c817423348d5b26dcdd90ca864f786313141c807646a2199bf9c225f3a7ce`.

## Phase 4 — Validate

- Tests run: `scripts/test-provider-conformance.sh` passed independently after
  the inspection fixes. `bin/gate.sh --diff` then repeated the complete
  production workspace, PostgreSQL, QML, cartridge, renderer, SDK, architecture
  proof, and provider conformance evidence.
- Gate run: `bin/gate.sh --diff` printed `GATE GREEN [diff]` across all 17
  stages and wrote the delivery receipt for gated state
  `5e2c817423348d5b26dcdd90ca864f786313141c807646a2199bf9c225f3a7ce`.
- Skips or pre-existing failures: the ordinary workspace stage reported the
  expected ignored PostgreSQL/provider tests; dedicated stages 15 and 17 ran
  those tests against the healthy PostgreSQL container with no failures. The
  QML smoke emitted the existing software-rendering `libEGL` warnings and
  passed.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — registry and operator CLI evidence proves one immutable
    provider/release identity, exact HTTPS/key/scope/quota policy, overlapping
    rotation, config revisions, lifecycle and terminal revocation, and
    append-only reasoned audit history.
  - REQ-002 PASS — grant, protocol, broker, and separate-process evidence
    proves a 60-second one-scope audience/release/session-bound grant, exact
    signed request/response context, guarded registered egress, and pairwise
    persona disclosure without account ID, raw persona ID, reusable device
    credential, or database capability.
  - REQ-003 PASS — the public and conformance corpora reject noncanonical
    endpoints, private/special/mixed DNS results, wrong roots, redirects,
    stale/future/tampered/mismatched signatures, malformed or oversized bodies,
    replay collisions, timeouts, and quota/lease races with bounded safe errors
    and durable audit evidence. Concurrent first callback delivery now proves
    exactly one acceptance and one duplicate.
  - REQ-004 PASS — PostgreSQL lifecycle tests suspend or revoke provider,
    release, scope, and independent message/TLS keys; new admission fails
    closed, terminal states cannot revive, active-session policy is explicit,
    and no WebSocket delivery is required.
  - REQ-005 PASS — `scripts/test-provider-conformance.sh` launches a separate
    TLS fixture process and proves grant/message binding, exact replay,
    revision conflict, commit-then-timeout retry, event deduplication, outage,
    restart, and authenticated reconciliation against migrated PostgreSQL.
- OpenWiki update run `cdf1091a-04dd-414d-8c6d-dd8684883c80` reconciled
  `game-cartridges.md`, `development-and-validation.md`, and `quickstart.md`,
  repaired the prior Ticket 017 evidence path, and finished with `status:
  complete` and no warnings. The completion receipt names pipeline
  `6f1b77ba-06f4-4c58-b908-171f00197018`, tool
  `mcp__openwiki__openwiki_finish`, and gated state
  `acbbb4f207642e022848a1fe4fb9ba943fc8a5ece4a7033b0b09bed2332f8f55`.
- Hand-maintained architecture and the new operator runbook describe the
  immutable registry, cryptographic profile, egress controls, lifecycle,
  failure recovery, and still-dormant authority boundary.
- AAR-018 is submitted with two captured failures, two standing prevention
  rules, and the remote-provider security-foundation architecture decision;
  every new ID is registered in `docs/planning/knowledge/INDEX.md`.
- The final post-OpenWiki `bin/gate.sh --diff` passed all 17 stages and printed
  `GATE GREEN [diff]`: ordinary workspace checks, all 43 sequential PostgreSQL
  tests, the live PostgreSQL/Rust API/Signal Siege/QML smoke, every Cartridge
  contract/renderer/SDK/proof stage, and the complete provider conformance
  suite passed. The delivery, OpenWiki completion, and current gated hashes all
  match `acbbb4f207642e022848a1fe4fb9ba943fc8a5ece4a7033b0b09bed2332f8f55`.
- Ticket 018 is closed and this spec/notes pair is archived. No requirement was
  deferred or silently dropped, and no provider authority route was enabled.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Concurrent exact callback delivery could produce `Conflict` instead of durable duplicate success. | `SELECT ... FOR UPDATE` cannot serialize on an absent message/event receipt. | Lock the existing release root before checking and inserting callback identities; add concurrent TLS-process evidence. | `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001` |
| 2 | Local-use NAT64 and other reserved IPv6 space passed the nominal public-only classifier. | A special-use denylist was incomplete and had no positive global-unicast allocation boundary. | Require the allocated IPv6 global-unicast block and reject current IANA special-purpose prefixes within it; expand the hostile corpus. | `PR-omarchy-gaming-system-classify-provider-egress-by-global-allocation-001` |
