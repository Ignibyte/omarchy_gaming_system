---
type: "Reference"
title: "Omarchy Gaming System engineering quickstart"
openwiki_generated: true
sources:
  - id: openwiki-source-d392f8f0962c50f0d66e0629
    resource: repo://client/qml/Main.qml
  - id: openwiki-source-25c2deb1d0664370b4037c40
    resource: repo://crates/game-provider/src/lib.rs
  - id: openwiki-source-30e12d7dfe374ac923c8ddbd
    resource: repo://crates/game-runtime/src/lib.rs
  - id: openwiki-source-df8490db5b51be8096630e7e
    resource: repo://crates/game-signal-siege/src/lib.rs
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
  - id: openwiki-source-76060b846b9222af2c790243
    resource: repo://crates/server/src/signal_siege_api_tests.rs
  - id: openwiki-source-e7a72df5b89c1ac350ffe062
    resource: repo://crates/server/src/sync.rs
  - id: openwiki-source-408aa68caebee417a5a319b8
    resource: repo://docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md
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
  - id: openwiki-source-513cfb82a80f03b4b9a1484e
    resource: repo://scripts/test-provider-conformance.sh
generated: {by: "codex", at: "2026-08-26T00:20:22.247Z"}
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
publishes exactly Signal Siege v1, a deterministic one-human tactical game
against a server-owned bot, and supports idempotent owner-scoped launch,
bounded play, durable completion, exact final-command replay, and retained
history. Production also includes canonical signed Game Cartridges, an
isolated trusted Core/Rich-2D renderer/preview CLI, a deterministic public SDK
export, signed release and catalog-policy verification, and a secure local
cartridge importer. When the optional provider runtime is configured, the
server also exposes the operator-pinned Door Legends v1 release and routes its
player operations to a separate provider process and database. The main QML
connector now handles server selection, account registration, password or MFA
sign-in, and owned-persona creation or selection before entering an
authenticated home. From there it can manage persona connections and private
blocks, browse private conversations, page history, send messages, and clear
unread state. It does not yet browse game definitions, create challenges,
launch gameplay, render cartridges in the main shell, or subscribe to live
WebSocket hints.

The product is game-first: connections, private inboxes, challenges, and
persistent game history define the intended experience. A public message board
may complement that system later, but it is not the current identity or
private-alpha focus.

The server-side first playable now spans account, authentication, persona,
connection, private inbox, durable synchronization, challenge-to-session
orchestration, and one completed asynchronous solo match with an explicit
outcome. The flagship QML connector now covers the account-to-persona-to-social
and private-inbox path, but still has no challenge, game catalog, launch, or
gameplay flow. Compiled Signal Siege outcome-derived achievements and rewards
remain future work.

ADR-0002 now accepts the **Game Cartridge** as the staged portable-game
direction: a publisher-signed, data-only presentation package rendered by
trusted OmarchyGS QML components. Ticket 015 now implements deterministic v1
packing, strict verification/conformance, compatibility reporting, and a
same-user content-addressed local store. Ticket 016 implements the bounded
render-plan compiler, fixed trusted QML vocabulary, private preview output, and
measured Core/Rich-2D profile. Ticket 017 implements a deterministic public SDK,
signed reproducible release and five-state catalog policy, a Linux
descriptor-relative secure importer, and a clean-clone first-party repository
proof. Ticket 018 adds the production provider security crate and durable
schema for operator-pinned registration, signed grants and messages, guarded
egress, replay, quotas, leases, lifecycle, and audit. Ticket 019 connects that
foundation to one narrowly authorized first-party pilot: compiled Signal Siege
sessions retain OmarchyGS rules authority, while a Door Legends session pins
one exact provider release as its only durable rules/state/revision authority.
External providers, server-side cartridge ingestion, and main-client launch
remain later work.

## Task routing

| Engineering intent | Read first | Primary source entrypoints | Narrow validation |
|---|---|---|---|
| Change server startup, configuration, migrations, or health behavior | [Runtime foundation](runtime-foundation.md) | `crates/server/src/main.rs`, `config.rs`, `app.rs`; `migrations/` | `cargo test -p omarchy-gaming-system-server`; health smoke |
| Change accounts, device sessions, MFA, personas, or connections | [Runtime foundation](runtime-foundation.md) | `accounts.rs`, `credentials.rs`, `sessions.rs`, `mfa.rs`, `personas.rs`, `connections.rs`; `docs/api.md` | Domain tests plus multi-account PostgreSQL evidence |
| Change QML endpoint selection, account access, MFA sign-in, persona onboarding, connections, blocks, or private inbox | [Runtime foundation](runtime-foundation.md) and [Development and validation](development-and-validation.md) | `client/qml/Main.qml`, `ApiClient.qml`, `OnboardingController.qml`, `SocialController.qml`, `client/qml/screens/` | `scripts/test-qml-onboarding.sh`; live QML smoke in `scripts/dev.sh --smoke-test` |
| Change inbox, challenges, synchronization, or game behavior | [Runtime foundation](runtime-foundation.md) and [Product boundaries](product-boundaries.md) | `inboxes.rs`, `challenges.rs`, `sync.rs`, `games.rs`, `crates/game-runtime`, `crates/game-signal-siege`; migrations `0007`–`0013`; challenge, game, Signal Siege, inbox, and sync API tests | Participant privacy, relationship policy, exact-version state, lifecycle, expiry, transition and revision races, retry effects, cursor/reconnect, and PostgreSQL evidence |
| Change cartridge packaging, trusted rendering, SDK portability, or provider integration | [Game Cartridges](game-cartridges.md) and [Product boundaries](product-boundaries.md) | `crates/game-cartridge`; `crates/game-cartridge-renderer`; `crates/game-provider`; `crates/server/src/provider_games.rs`; `client/qml/cartridge`; migrations `0014`–`0015`; ADR-0002; Tickets 015–019 | `scripts/test-game-cartridge.sh`; `scripts/test-game-cartridge-renderer.sh`; `scripts/test-game-cartridge-sdk.sh`; `scripts/test-provider-conformance.sh`; `scripts/test-provider-authority-pilot.sh`; threat/authority review and constitutional authority check |
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

The keyboard-first QML connector now exercises that complete entry path. It
accepts a bare server origin, allows HTTP only for exact loopback hosts, and
requires HTTPS remotely. An exact healthy OmarchyGS response unlocks account
registration or sign-in; successful password or MFA authentication then loads
owned personas, permits creation when needed, and requires explicit selection
before the authenticated home. Bearer tokens and MFA challenges remain only in
process memory and are cleared on endpoint changes, logout, challenge expiry,
terminal authentication failures, invalid sessions, or malformed authenticated
success responses.

The same shell exposes explicit Social and Inbox routes only after a valid
owned persona is selected. One bearer-owning transport stays behind the
onboarding authority controller; the social controller receives a gated
request function and derives actor paths from that selected persona. Social
entry manually refreshes incoming/outgoing requests, accepted connections, and
the actor's private block inventory. Inbox entry manually refreshes at most 100
conversations, loads ascending bounded message pages, prepends older pages by
the conversation-local cursor, sends trimmed control-safe text, and advances
unread state through the latest loaded message. Exact public profiles and
allowlisted user/system message shapes render as plain text. Malformed,
oversized, stale, or invalid-session responses fail closed; the last case also
clears bearer, personas, selection, and social state. The connector does not
poll or open the persona-sync WebSocket yet, so screen entry, completed actions,
and the visible refresh command recover durable REST truth.

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

The current game boundary is executable. `GET /v1/games` always returns stable
compiled Signal Siege v1 metadata and, when the optional all-or-none provider
configuration is present, adds only an active operator-pinned Door Legends v1
manifest. Every catalog record declares `platform_compiled` or
`registered_provider` authority and an optional exact provider release. An
authenticated account may launch either admitted one-human definition for an
owned persona with a durable UUID receipt. The persona-root transaction checks
exact replay before current admission, admits at most 25 active solo starts,
and creates only the human seat; the deterministic Signal Siege bot has no
account or persona row.

Connected, unblocked personas may separately use the existing challenge flow
for an exact registered two-player version. Challenge creation is bounded and
idempotent, and acceptance atomically fixes challenger/challenged at seats 0/1.
All participants read durable snapshots without consulting today's registry
and submit bounded commands with a session-wide idempotency UUID and expected
revision. A transition now returns both state and authoritative active or
completed lifecycle. Completion stores its timestamp and explicit outcome;
the exact final command replays after completion, while new commands conflict.
List/detail history and payload-minimal sync invalidations remain reconnect
safe. Door Legends commands retain their idempotency key and expected provider
revision through the broker, while explicit reconciliation recovers unknown
outcomes. Provider session reads expose only the authority-tagged platform
envelope, last authenticated bounded view, availability, and optional
allowlisted result—not provider-private rules state. The QML game discovery,
launch, and gameplay flow remains a later slice.

The production cartridge crates can independently pack and verify a canonical
signed `.ogsc`, report host compatibility, install/revoke it in a bounded
same-user local store, validate a schema-conforming view, and compile a bounded
Core or Rich-2D plan for platform-owned QML components. They also export and
self-verify a deterministic public SDK, create and verify signed reproducible
release attestations, enforce signed five-state lifecycle policy, and import a
release through a Linux descriptor-relative secure store. The preview CLI
writes only read-only plan/assets into a caller-created private directory and
reports no provider, database, or credential use. The current main QML connector
does not browse or launch cartridges, and the server does not ingest cartridge
files directly. The `omarchy-game-provider` crate implements operator-pinned
releases, signed pairwise grants and messages, public-only pinned HTTPS egress,
and durable replay/quota/lease/audit controls. The optional production bridge
instantiates it only for the Door Legends pilot. Migration 0015 prevents dual
authority: compiled sessions require local object state and no provider release,
whereas provider sessions require a release pin and null local rules state.
Authenticated callbacks become results, achievements, views, audit, and sync
effects only through one policy-checked projection transaction.

Current runtime identifiers use the gaming-system namespace; see [Runtime
foundation](runtime-foundation.md) for the narrow local compatibility window
retained for old bind configuration and session tokens.
