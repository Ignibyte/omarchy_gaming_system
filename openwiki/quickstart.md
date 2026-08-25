---
type: "Reference"
title: "Omarchy Gaming System engineering quickstart"
openwiki_generated: true
sources:
  - id: openwiki-source-d392f8f0962c50f0d66e0629
    resource: repo://client/qml/Main.qml
  - id: openwiki-source-a1b45828c3f97dd0a06fb618
    resource: repo://crates/game-cartridge/src/release.rs
  - id: openwiki-source-111e4189516b7f457a68f043
    resource: repo://crates/game-cartridge/src/sdk.rs
  - id: openwiki-source-71f8ccb7a1e293121205a368
    resource: repo://crates/game-cartridge/src/secure_store.rs
  - id: openwiki-source-30e12d7dfe374ac923c8ddbd
    resource: repo://crates/game-runtime/src/lib.rs
  - id: openwiki-source-e61b285fcaa489b63922f43f
    resource: repo://crates/server/src/app.rs
  - id: openwiki-source-a3892e0554790e3efc606fe1
    resource: repo://crates/server/src/challenges.rs
  - id: openwiki-source-4b133589ca70bd174cf19eb9
    resource: repo://crates/server/src/connections.rs
  - id: openwiki-source-26aac996689c040c6aab6825
    resource: repo://crates/server/src/games.rs
  - id: openwiki-source-b2c7af59f511c4ed8a004fb0
    resource: repo://crates/server/src/inbox_api_tests.rs
  - id: openwiki-source-a13fe4db1eee073d0a7e2c4d
    resource: repo://crates/server/src/main.rs
  - id: openwiki-source-83e16151ac88c29a31cb79d2
    resource: repo://crates/server/src/mfa.rs
  - id: openwiki-source-54f6da1456b2b76d94d11b0e
    resource: repo://crates/server/src/personas.rs
  - id: openwiki-source-d943a78fae758ed47e30a12a
    resource: repo://crates/server/src/sessions.rs
  - id: openwiki-source-e7a72df5b89c1ac350ffe062
    resource: repo://crates/server/src/sync.rs
  - id: openwiki-source-408aa68caebee417a5a319b8
    resource: repo://docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md
  - id: openwiki-source-872141f77f71851168245852
    resource: repo://docs/architecture/system-overview.md
  - id: openwiki-source-85dba8f87dd5947de337aca5
    resource: repo://docs/product-charter.md
  - id: openwiki-source-cb6494f7cbf0d5d23ffe082a
    resource: repo://migrations/0012_game_challenges.sql
  - id: openwiki-source-d69dbacb0ae7fe382ee46161
    resource: repo://scripts/test-game-cartridge-renderer.sh
  - id: openwiki-source-8df9ad1a3495f8360740ff03
    resource: repo://scripts/test-game-cartridge-sdk.sh
  - id: openwiki-source-68106a790eb8acc94f8d3540
    resource: repo://scripts/test-game-cartridge.sh
generated: {by: "codex", at: "2026-08-25T15:17:54.717Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-25T15:17:54.717Z
---

# Omarchy Gaming System engineering quickstart

Omarchy Gaming System is an API-first social gaming system with a keyboard-first QML
connector as its flagship client. The implemented runtime now starts
PostgreSQL, applies migrations, exposes database-backed `/health`, accepts
account registration at `POST /v1/accounts`, provides revocable Bearer device
sessions, offers opt-in TOTP two-factor authentication with single-use recovery
codes, supports account-owned personas with public exact-handle lookup, and
supports persona-scoped connection requests, accepted connections, and private
directional blocks. Accepted persona pairs also own durable private
conversations with typed messages and per-participant unread state. Every
persona also has a retained, monotonic synchronization cursor: REST recovers
durable changes and an authenticated WebSocket supplies owner-scoped wakeup
hints. Connected personas can now create retry-safe, exact-version two-person
game challenges, receive their lifecycle as typed private inbox history, retain
terminal challenge history, and accept one into an atomic version-pinned game
session. A database-free compiled game registry validates exact rules
versions, public metadata, deterministic bounded initialization, and bounded
deterministic commands. PostgreSQL stores version-pinned sessions with ordered
persona participants, while participant-private REST routes read the durable
snapshot and apply revision-checked, session-idempotent commands. Production
honestly publishes an empty game catalog until a playable definition is
compiled. Production also includes canonical signed Game Cartridges, an
isolated trusted Core/Rich-2D renderer/preview CLI, a deterministic public SDK
export, signed release and catalog-policy verification, and a secure local
cartridge importer. The main QML connector still renders health and does not yet
browse or launch cartridges.

The product is game-first: connections, private inboxes, challenges, and
persistent game history define the intended experience. A public message board
may complement that system later, but it is not the current identity or
private-alpha focus.

The first-playable product is broader than the current code. Account,
authentication, persona, connection, private inbox, durable synchronization,
and challenge-to-session orchestration now exist. A completed asynchronous
match and recorded result remain roadmap intent; the implemented game slice
establishes discovery, version-pinned initialization, participant-private
reads, revision-checked commands, and reconnect invalidation, but it still has
no playable production rules, result lifecycle, or game UI flow.

ADR-0002 now accepts the **Game Cartridge** as the staged portable-game
direction: a publisher-signed, data-only presentation package rendered by
trusted OmarchyGS QML components. Ticket 015 now implements deterministic v1
packing, strict verification/conformance, compatibility reporting, and a
same-user content-addressed local store. Ticket 016 implements the bounded
render-plan compiler, fixed trusted QML vocabulary, private preview output, and
measured Core/Rich-2D profile. Ticket 017 implements a deterministic public SDK,
signed reproducible release and five-state catalog policy, a Linux
descriptor-relative secure importer, and a clean-clone first-party repository
proof. No server catalog-ingestion or main-client launch route and no provider
network are connected. Ticket 014's broker/provider/QML work remains an isolated
proof, and the compiled Rust runtime plus OmarchyGS-owned PostgreSQL game
snapshot remain authoritative until later provider migration, security, and
Constitution-amendment pipelines complete.

## Task routing

| Engineering intent | Read first | Primary source entrypoints | Narrow validation |
|---|---|---|---|
| Change server startup, configuration, migrations, or health behavior | [Runtime foundation](runtime-foundation.md) | `crates/server/src/main.rs`, `config.rs`, `app.rs`; `migrations/` | `cargo test -p omarchy-gaming-system-server`; health smoke |
| Change accounts, device sessions, MFA, personas, or connections | [Runtime foundation](runtime-foundation.md) | `accounts.rs`, `credentials.rs`, `sessions.rs`, `mfa.rs`, `personas.rs`, `connections.rs`; `docs/api.md` | Domain tests plus multi-account PostgreSQL evidence |
| Change inbox, challenges, synchronization, or game behavior | [Runtime foundation](runtime-foundation.md) and [Product boundaries](product-boundaries.md) | `inboxes.rs`, `challenges.rs`, `sync.rs`, `games.rs`, `crates/game-runtime`; migrations `0007`–`0012`; challenge, game, inbox, and sync API tests | Participant privacy, relationship policy, exact-version state, expiry, transition and revision races, retry effects, cursor/reconnect, and PostgreSQL evidence |
| Change cartridge packaging, trusted rendering, SDK portability, or future provider integration | [Game Cartridges](game-cartridges.md) and [Product boundaries](product-boundaries.md) | `crates/game-cartridge`; `crates/game-cartridge-renderer`; `client/qml/cartridge`; ADR-0002; isolated `crates/game-cartridge-spike`; Tickets 015–019 | `scripts/test-game-cartridge.sh`; `scripts/test-game-cartridge-renderer.sh`; `scripts/test-game-cartridge-sdk.sh`; provider proof, threat/authority review, and constitutional authority check |
| Run or diagnose the local stack and quality gate | [Development and validation](development-and-validation.md) | `scripts/dev.sh`; `bin/gate.sh`; `client/qml/Main.qml` | `bin/gate.sh --fast` or `--diff` |
| Start or resume a non-trivial change | [Codex workflow](codex-workflow.md) | `AGENTS.md`; `$omarchy-workflow`; active pipeline | Phase receipts and canonical gate |

## Current boundary

The database-backed health, account-registration, revocable-device-session,
opt-in TOTP MFA, and persona slices are executable today. Registration creates
no session or persona implicitly: clients exchange credentials for an opaque
token, then use that account authority to manage its devices, optional MFA,
and one or more personas. Once MFA is enabled, correct primary credentials
return a short-lived challenge rather than a session; a TOTP or unused recovery
code must complete that challenge before a new device token is issued.
Persona responses expose only public profile fields. Exact canonical handle
lookup is public, while the owning account remains private.

Connection commands authenticate the device session and require the acting
persona to belong to that account. Requests are directional until the
addressee accepts; accepted connections are mutual. Blocks remain private and
directional, atomically remove any existing relationship, and prevent new
requests in either direction until the blocker explicitly unblocks. Responses
embed only the public persona profile.

An actual connection-acceptance transition creates or reuses one private
conversation and appends one server-authored `connection_accepted` message.
Only either participant can inventory or read that conversation. User sends
require the pair to remain accepted and unblocked, while existing history stays
readable after removal or blocking. Message order and unread positions are
conversation-local and monotonic. A separate persona cursor recovers retained
cross-resource changes through REST; hint-only WebSockets tell clients when to
query it.

The current game boundary is foundational but executable. `GET /v1/games`
returns compiled manifest metadata in stable order and is empty in production
until the first playable game ships. An owned persona may challenge a connected,
unblocked peer to one exact registered two-player version. Creation is bounded,
challenger-idempotent, and delivered through typed private inbox history plus
payload-minimal sync invalidations; participant-only reads retain accepted,
declined, cancelled, and expired history. Only the challenged persona may
accept, which calls the trusted session primitive in the same PostgreSQL
transaction and fixes challenger/challenged at seats 0/1. That operation stores
deterministic revision-zero object state and one minimal session invalidation
for each participant. Authenticated participants can then read the durable
session without consulting today's registry and submit bounded commands with a
session-wide idempotency UUID and expected revision. Matching challenge and
command retries return durable results without duplicate effects. Direct
arbitrary session creation, completed results, playable production rules, and
game UI remain later slices.

The production cartridge crates can independently pack and verify a canonical
signed `.ogsc`, report host compatibility, install/revoke it in a bounded
same-user local store, validate a schema-conforming view, and compile a bounded
Core or Rich-2D plan for platform-owned QML components. They also export and
self-verify a deterministic public SDK, create and verify signed reproducible
release attestations, enforce signed five-state lifecycle policy, and import a
release through a Linux descriptor-relative secure store. The preview CLI
writes only read-only plan/assets into a caller-created private directory and
reports no provider, database, or credential use. The current main QML connector
does not browse or launch cartridges, and the server does not ingest cartridges
or contact a provider.

Current runtime identifiers use the gaming-system namespace; see [Runtime
foundation](runtime-foundation.md) for the narrow local compatibility window
retained for old bind configuration and session tokens.
