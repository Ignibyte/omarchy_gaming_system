---
title: Stable server discovery and isolated client profiles
pipeline_id: de9c08be-fa26-488b-a6f7-0b068add0761
status: Phase 5 — Complete PASS
ticket: TICKET-031
ticket_doc: docs/planning/tickets/closed/TICKET-031-stable-server-discovery-and-isolated-client-profiles.md
aar: docs/planning/knowledge/aar/AAR-031-stable-server-discovery-and-isolated-client-profiles.md
created: 2026-08-26
---

# Stable server discovery and isolated client profiles — spec

## Intent

Make each owner-operated OmarchyGS community recognizable and selectable as a
stable independent system: the server publishes a durable identity and exact
compatibility document, while the flagship QML client saves only non-secret
profiles and prevents authority from crossing server boundaries.

## Scope

- In: all ten Ticket 031 requirements; singleton server identity, public name,
  discovery/capabilities, QML profile persistence and management, identity
  pinning, compatibility negotiation, cross-controller clearing, two-server
  evidence, documentation, and canonical gate.
- Out: federation, shared/global identity, persistent credentials, automatic
  login, certificate pinning/CA management, marketplace/catalog transfer,
  remote administration, and server identity rotation/fork operations.

## Acceptance criteria (EARS)

The binding requirements are REQ-001 through REQ-010 in
`TICKET-031-stable-server-discovery-and-isolated-client-profiles.md`.

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | The server UUID is generated once in PostgreSQL and survives restart, replication, backup, and restore; it is not derived from hostname, TLS material, or operator display name. | Community continuity follows durable server state, while hostnames and certificates can change operationally. |
| 2 | Discovery is a dedicated public versioned endpoint; `/health` remains the operational liveness contract. | Compatibility metadata and health have different consumers and evolution rules. |
| 3 | Capabilities are a deterministic bounded set of implemented versioned strings; protocol v1 clients require the onboarding subset and tolerate bounded unknown additions. | The document must support compatible evolution without advertising future or disabled behavior. |
| 4 | Saved profiles contain only canonical origin, UUID, public name, protocol version, and public capabilities; all credentials and selected account/persona state remain in process memory. | A convenience inventory must not become a credential store. |
| 5 | A saved origin is pinned to its server UUID. A mismatch fails closed until the player explicitly removes the old profile and reconnects as a new server. | Silent identity replacement would mix community trust and could expose credentials to a server now controlling a reused origin. |
| 6 | Multiple profiles mean isolated choices among independent communities, not federation or shared identity. | This preserves ADR-0003 and prevents multi-server support from implying cross-server authority. |
| 7 | The discovery contract is `GET /.well-known/omarchygs` with exact keys `service`, `server_id`, `server_name`, `protocol_version`, and `capabilities`; `/health` remains unchanged. | Recognition and compatibility need a public stable contract without coupling clients to operational database-health fields. |
| 8 | The profile inventory is an exact version-1 JSON array in Qt `Settings`, capped at 16 records and 16 KiB, deduplicated by both canonical origin and UUID, with no automatic connection from persisted state. | This is enough for deliberate community selection while bounding hostile local state and keeping credentials out of persistence. |
| 9 | Direct entry supports `CONNECT ONCE` and `SAVE & CONNECT`; selecting a saved row always supplies its pinned UUID. Typing an already-saved origin also enforces its pin. | A direct-entry path must not become an identity-pin bypass, while players still need an intentional non-persistent connection. |
| 10 | Protocol v1 requires invite registration, device sessions, and personas. Unknown bounded capabilities are retained and tolerated; missing required capabilities or another protocol version is incompatible. | This is the smallest implemented onboarding contract and permits compatible capability growth. |

## Linked artifacts

- Ticket: [TICKET-031](../../tickets/closed/TICKET-031-stable-server-discovery-and-isolated-client-profiles.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Roadmap: [owner-operated server ecosystem](../../ROADMAP.md)
- Prior direction: [Ticket 027 notes](../completed/owner-operated-servers-cartridge-distribution-and-extension-roadmap.notes.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | ticket, spec, notes, open AAR | scope and EARS complete |
| 2 Design | persistence/API/QML authority design, file manifest, regression plan | CodeGraph receipt and actionable design |
| 3 Implement | migration, discovery, client profiles, docs, and tests | focused multi-server evidence |
| 3.5 Inspect | correctness, auth/privacy, identity pinning, persistence, UX | resolved ledger and fresh CodeGraph receipt |
| 4 Validate | focused tests and canonical delivery gate | matching gate receipt |
| 5 Complete | EARS audit, OpenWiki, AAR, ticket/archive | no silent drops |
| Delivery | staged review and authorized commit/push | remote commit/tree readback |

## Phase 2 design

### Data and server boundary

- Migration `0018_server_identity.sql` creates a singleton row with a random
  PostgreSQL UUID. Primary/check constraints prevent a second row, and triggers
  reject update, delete, and truncate so normal backup/restore preserves the
  community identifier unchanged.
- `OGS_SERVER_NAME` is a trimmed, control-free 1–64 character public label.
  Absence selects `OmarchyGS Community`; invalid explicit values fail before
  listener bind. The name remains configuration, not identity authority.
- A `server_discovery` domain module owns protocol/capability constants,
  deterministic documents, and the singleton query. The Axum handler is public,
  no-store, unauthenticated, and returns a generic 503 document if durable
  identity cannot be read.
- Provider capability is advertised only when the registered-provider runtime
  is enabled. All other advertised strings map to currently routed behavior and
  remain lexically ordered.

### Client trust and persistence boundary

- `ServerProfiles.qml` is the sole persistence authority. It accepts only exact
  records containing `origin`, `server_id`, `server_name`, `protocol_version`,
  and `capabilities`; rejects invalid UTF-8-size bounds, unknown keys, malformed
  UUID/origin/name/capability values, duplicate origins/UUIDs, and unsupported
  protocol state; and rewrites rejected state to an empty array.
- `OnboardingController.qml` remains the connection and in-memory authority.
  It validates the exact discovery document before entering access, requires
  the supported onboarding subset, supplies remembered metadata to the profile
  store, and never persists bearer, MFA, invitation, password, account,
  persona, social, inbox, challenge, or game state.
- Every direct, saved, switch, or remove operation calls the existing authority
  clear before configuring or requesting a new origin. An origin already in the
  inventory automatically acquires its stored UUID expectation even when typed
  manually. A mismatch remains on the connection screen and never updates the
  profile.
- `ConnectionScreen.qml` presents bounded saved rows plus explicit connect,
  remove, connect-once, and save-and-connect actions. Rows and status messages
  use plain text, accessibility names, keyboard controls, and scrolling at the
  640×420 minimum.

### File manifest

| Area | Files |
|---|---|
| Durable identity | `migrations/0018_server_identity.sql`, `scripts/test-operator-recovery.sh` |
| Server contract | `crates/server/src/server_discovery.rs`, `config.rs`, `app.rs`, `main.rs`, `server_discovery_api_tests.rs`, provider-router callers |
| QML contract | `client/qml/ServerProfiles.qml`, `OnboardingController.qml`, `Main.qml`, `screens/ConnectionScreen.qml`, fixture server and QML tests |
| Packaging/dev | runtime manifest, QML/package tests, `scripts/dev.sh`, `.env.example` |
| Durable docs | API, README, owner-operated/operator recovery docs, roadmap/architecture/OpenWiki inputs as required |

### Regression and hostile cases

- Rust: exact success/no-store/no-auth, UUID stability across router/name
  changes, provider capability truthfulness, closed-pool 503, configuration
  bounds, migration singleton/immutability, and restore identity equality.
- QML: two saved profiles across separate test-runner processes; exact persisted
  public schema; malformed/oversized/extra-key/credential-like/duplicate state;
  compatible unknown capability; missing capability/version; malformed,
  oversized, timeout, wrong-service, and pinned-UUID replacement fixtures.
- Existing fixture, live vertical slice, package artifact, registration, MFA,
  persona, social, inbox, challenge, and game checks remain canonical gate
  regressions.
