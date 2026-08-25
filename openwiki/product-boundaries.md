---
type: "Reference"
title: "Product and architecture boundaries"
openwiki_generated: true
sources:
  - id: openwiki-source-0d99cc708822fd795c83ba12
    resource: repo://client/qml/cartridge/CartridgePreview.qml
  - id: openwiki-source-c566a55d52a9744f7b26b7c4
    resource: repo://client/qml/cartridge/TrustedCartridgeSurface.qml
  - id: openwiki-source-937883bc0b4873d5f0200c46
    resource: repo://CONSTITUTION.md
  - id: openwiki-source-37af4c6b51c86b62db25f85f
    resource: repo://crates/game-cartridge-renderer/Cargo.toml
  - id: openwiki-source-fdf115002c4aabad0babec70
    resource: repo://crates/game-cartridge-renderer/src/lib.rs
  - id: openwiki-source-877fd8b6ed8717d54fa8c17a
    resource: repo://crates/game-cartridge/Cargo.toml
  - id: openwiki-source-b4a2591d7d7f80d847ef95ed
    resource: repo://crates/game-cartridge/src/contract.rs
  - id: openwiki-source-a1b45828c3f97dd0a06fb618
    resource: repo://crates/game-cartridge/src/release.rs
  - id: openwiki-source-111e4189516b7f457a68f043
    resource: repo://crates/game-cartridge/src/sdk.rs
  - id: openwiki-source-71f8ccb7a1e293121205a368
    resource: repo://crates/game-cartridge/src/secure_store.rs
  - id: openwiki-source-07e2881dc5e4740f35a238ee
    resource: repo://crates/game-cartridge/src/store.rs
  - id: openwiki-source-30e12d7dfe374ac923c8ddbd
    resource: repo://crates/game-runtime/src/lib.rs
  - id: openwiki-source-66facc66e34ad7f2a74321e1
    resource: repo://crates/server/src/accounts.rs
  - id: openwiki-source-e61b285fcaa489b63922f43f
    resource: repo://crates/server/src/app.rs
  - id: openwiki-source-a3892e0554790e3efc606fe1
    resource: repo://crates/server/src/challenges.rs
  - id: openwiki-source-4b133589ca70bd174cf19eb9
    resource: repo://crates/server/src/connections.rs
  - id: openwiki-source-a243b385d49ea9224173d77a
    resource: repo://crates/server/src/game_api_tests.rs
  - id: openwiki-source-26aac996689c040c6aab6825
    resource: repo://crates/server/src/games.rs
  - id: openwiki-source-b2c7af59f511c4ed8a004fb0
    resource: repo://crates/server/src/inbox_api_tests.rs
  - id: openwiki-source-4773699be275375a3bb0c216
    resource: repo://crates/server/src/inboxes.rs
  - id: openwiki-source-83e16151ac88c29a31cb79d2
    resource: repo://crates/server/src/mfa.rs
  - id: openwiki-source-54f6da1456b2b76d94d11b0e
    resource: repo://crates/server/src/personas.rs
  - id: openwiki-source-d943a78fae758ed47e30a12a
    resource: repo://crates/server/src/sessions.rs
  - id: openwiki-source-e7a72df5b89c1ac350ffe062
    resource: repo://crates/server/src/sync.rs
  - id: openwiki-source-98b4fef5bee3b5a0d880f16b
    resource: repo://docs/api.md
  - id: openwiki-source-408aa68caebee417a5a319b8
    resource: repo://docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md
  - id: openwiki-source-c22435ddb0c3a9abfe95d9af
    resource: repo://docs/architecture/game-cartridges.md
  - id: openwiki-source-872141f77f71851168245852
    resource: repo://docs/architecture/system-overview.md
  - id: openwiki-source-c3d1d450d3a3561b368e5307
    resource: repo://docs/planning/ROADMAP.md
  - id: openwiki-source-85dba8f87dd5947de337aca5
    resource: repo://docs/product-charter.md
  - id: openwiki-source-674113ba65eebb6f842b2dda
    resource: repo://migrations/0008_conversation_local_message_sequences.sql
  - id: openwiki-source-8df9ad1a3495f8360740ff03
    resource: repo://scripts/test-game-cartridge-sdk.sh
generated: {by: "codex", at: "2026-08-25T15:17:54.717Z"}
---

# Product and architecture boundaries

## Product focus

Omarchy Gaming System is game-first. Connections, private inboxes, challenges,
server-authoritative matches, and persistent results define the first playable.
Public message boards remain outside the current identity and private-alpha
scope; they may be reconsidered later only as a complementary community
surface.

## Authority and identity

The Rust server is authoritative for authentication, authorization, game state,
turns, time, randomness, and rewards. Transport handlers translate requests and
responses; domain modules own permissions and invariants.

Accounts and personas are deliberately different identities. Accounts own
credentials, sessions, and administrative status. Personas are the public
identity shown to social and game surfaces. New APIs must not leak account
ownership through persona responses.

The device-session API authorizes account-level work but returns neither account
ownership nor token digests. Raw Bearer tokens appear only at creation and do
not create a public persona identity.

Optional TOTP MFA also belongs to private account authentication, not persona
identity. An enabled account's password login returns a temporary challenge;
only successful TOTP or unused recovery-code verification creates the new
device session. Enabling or disabling MFA does not rename personas, reveal
account ownership, or revoke existing sessions.

That account authority now creates, inventories, and edits personas without
accepting a client owner field. Inventory filters by the authenticated account,
and mutation predicates on both account and persona IDs. Persona responses
contain only seven public profile fields. Exact canonical handle lookup is
intentionally public, but neither it nor an authenticated response reveals the
owning account or session.

The implemented social graph also stays on the persona side of this boundary.
Every connection or block command derives the private account principal from a
validated device session and owner-scopes the acting persona; same-account
personas cannot create social edges. Requests are directional until the named
addressee accepts, after which the connection is mutual. Blocks remain private
and directional, remove relationship state atomically, and suppress requests in
both directions with one generic error. The block row and inventory are not
directly disclosed, but interaction denial is not treated as protection against
every inference a caller may draw about block direction.

Private inboxes remain on the persona side too. One accepted pair owns one
durable conversation. Only either participant can inventory or read it, and
responses omit account identity and the peer's read position. History remains
available after removal or blocking, while new sends require an accepted,
unblocked connection. Public history sequences are local to one conversation,
so unrelated private activity cannot appear as gaps.

Game challenges connect the social and game boundaries without moving either
one. An owned persona may challenge only a connected, unblocked,
different-account peer to one exact registered two-person game version. The
challenge is durable participant-private history and its lifecycle appears as
typed system messages in the pair's existing conversation. Acceptance is
reserved for the challenged persona and creates one server-authoritative
version-pinned session in the same transaction; decline, cancellation, and
expiry retain terminal history without a session.

## Durable recovery

REST/JSON is the durable command and query surface. The persona synchronization
feed returns a bounded baseline or retained changes after a monotonic cursor and
requires an explicit reset when continuity cannot be proven. WebSockets only
notify an authenticated owner that something changed; clients always return to
REST after reconnect or a lag hint. Game-challenge and game-session events
carry only their participant-authorized UUID; durable state remains behind the
same persona-participant REST boundary. Challenge and game commands use REST as
their durable mutation and replay surfaces; their WebSocket effect remains
only an advisory participant-local invalidation. Both transports preserve
persona ownership without exposing private account identity.

## Ordered identity work

The four roadmap identity outcomes are intentionally sequenced:

1. Account registration establishes normalized account identity and Argon2id
   password storage. It creates neither a session nor a persona.
2. Revocable device sessions now establish account authentication without
   storing raw tokens.
3. Opt-in TOTP MFA adds encrypted authenticator secrets, single-use recovery
   codes, replay-resistant login challenges, and bounded factor attempts to
   private account authentication.
4. Persona creation, editing, handle lookup, and privacy rules expose public
   identity without exposing account identity.

All four identity outcomes are implemented as independently auditable ticketed
pipelines. Persona connection requests, acceptance, removal, blocking, and the
private conversation/message/unread slice and durable persona synchronization
are also implemented on top of that boundary. The compiled registry and durable
exact-version session foundation and revision-checked idempotent commands are
now implemented as well. Connected-persona challenge creation, terminal
history, and atomic acceptance into those sessions are also implemented.
Playable rules, completed matches, and results remain later slices on top of
the same synchronization rule without treating WebSocket delivery as durable
truth.

The current server is a local development slice. Bearer tokens require
production TLS in transit, and public login requires distributed attempt
throttling. The in-process four-job Argon2 limit bounds memory-heavy work but is
not a substitute for either deployment control. TOTP also requires protected,
replicated encryption-key management and does not provide phishing resistance.

Public registration currently distinguishes an existing canonical username
with HTTP 409. That makes account-name enumeration a known, temporarily
accepted private-alpha risk. A public deployment must either accept that
contract deliberately or introduce a separately designed verifiable private
registration channel before replacing it with a generic response; MFA does not
remove the registration-side disclosure.

## Later boundaries

Synchronization and games build on the implemented persona connection and
private inbox boundaries. Conversation-local message sequences remain history
cursors, while the persona synchronization cursor spans visible resources and
drives reconnect recovery.

The current compiled game interface receives only a human-player count for
initialization or the current state, actor seat, and bounded object command for
a transition. It returns deterministic bounded object JSON and cannot query
PostgreSQL, inspect accounts or sessions, read the clock, use the network, or
draw ambient randomness. Server orchestration owns the transaction, persona
authorization, durable version and snapshot, ordered seats, revision and replay
identity, and participant sync events.
Session reads expose public personas and stored state only to a participating
owned persona and do not consult today's registry, so a newer process cannot
silently relabel old state. The participant command route returns only session
ID, revision, and state; receipts and conflict-side current revisions stay
private. The transport has no direct arbitrary session-creation route:
challenge acceptance owns the public path into an exact two-person session.
Completed results, playable production rules, and game UI remain later
boundaries.

Compiled Rust crates are the initial extension model. Loading user-supplied
native code is outside the first-alpha scope and would require a separate
sandbox decision.

### Portable game direction

ADR-0002 accepts a staged portable frontend named the **OmarchyGS Game
Cartridge**. A cartridge is immutable, publisher-signed data: manifest,
declarative screens, schemas, localization, and bounded assets. Trusted
platform QML renders that vocabulary. A cartridge cannot supply QML,
JavaScript, native code, shell commands, arbitrary shaders, imports, dynamic
remote assets, filesystem paths, or a network client.

Production now includes a local library and CLI for canonical stored-only
packing, Ed25519 verification, strict typed-content validation, compatibility,
and a read-only content-addressed store with fail-closed revocation. A separate
production renderer validates a bounded view against the authenticated pinned
schema, applies typed capability fallbacks and trusted preferences, and emits
only inert Core/Rich-2D plan tags. Ticket 017 adds deterministic SDK export,
signed reproducible release and catalog-policy verification, and secure local
import. The package and renderer crates have no HTTP, SQL, dynamic-loader, or
platform-credential dependency.

A signature authenticates publisher identity and exact bytes; it does not grant
resource or UI trust. Rust incrementally enforces plan/node/effect budgets plus
per-raster and decoded-scene ceilings before publishing a node or asset, caches
each authenticated asset digest, and requires exact Grid/Button action payload
shapes. The QML boundary recounts aggregate profile totals and maps only fixed
tags to platform-owned Components. OmarchyGS retains origin and failure chrome,
accessibility/focus behavior, trusted display preferences, and all future action
dispatch authority. The current preview emits only unconfirmed action requests
and has no server/provider path.

The Ticket 015 store and prepared preview directory remain same-user developer
boundaries. Ticket 017's Linux secure store is descriptor-relative, rejects
fixed directories not owned by the effective user or writable by group/other,
serializes monotonic signed-policy transitions, and persists denial policy
before enforcement. The exact store UID is still authoritative, so a later
privileged or shared launcher needs a dedicated service identity or equivalent
external monotonic authority. The main client still does not browse or launch
cartridges, and the server does not ingest them into its public game catalog.

This direction does not change present authority. Production still uses the
compiled runtime and OmarchyGS-owned PostgreSQL snapshot/revision described
above. Remote provider authority requires a later production protocol,
migration, and explicit Constitution §10 amendment that assigns each gameplay
revision to exactly one durable authority.

If that later mode is approved, OmarchyGS remains the authenticated broker and
platform authority for accounts, sessions, MFA, personas and avatar
projections, social state, catalog and launch policy, achievements,
notifications, audit, suspension, and the platform session envelope. A
registered provider may own only its game rules and private gameplay state.
The provider receives a short-lived audience/game/version/session/scope-bound
pairwise persona grant—never account identity, credentials, reusable device
tokens, or database access. See [Game Cartridges](game-cartridges.md) for the
full trust, graphics, failure, and rollout model.
