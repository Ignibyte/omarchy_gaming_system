---
type: "Reference"
title: "Product and architecture boundaries"
openwiki_generated: true
sources:
  - id: openwiki-source-937883bc0b4873d5f0200c46
    resource: repo://CONSTITUTION.md
  - id: openwiki-source-30e12d7dfe374ac923c8ddbd
    resource: repo://crates/game-runtime/src/lib.rs
  - id: openwiki-source-66facc66e34ad7f2a74321e1
    resource: repo://crates/server/src/accounts.rs
  - id: openwiki-source-e61b285fcaa489b63922f43f
    resource: repo://crates/server/src/app.rs
  - id: openwiki-source-4b133589ca70bd174cf19eb9
    resource: repo://crates/server/src/connections.rs
  - id: openwiki-source-a243b385d49ea9224173d77a
    resource: repo://crates/server/src/game_api_tests.rs
  - id: openwiki-source-26aac996689c040c6aab6825
    resource: repo://crates/server/src/games.rs
  - id: openwiki-source-b2c7af59f511c4ed8a004fb0
    resource: repo://crates/server/src/inbox_api_tests.rs
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
  - id: openwiki-source-872141f77f71851168245852
    resource: repo://docs/architecture/system-overview.md
  - id: openwiki-source-c3d1d450d3a3561b368e5307
    resource: repo://docs/planning/ROADMAP.md
  - id: openwiki-source-85dba8f87dd5947de337aca5
    resource: repo://docs/product-charter.md
  - id: openwiki-source-674113ba65eebb6f842b2dda
    resource: repo://migrations/0008_conversation_local_message_sequences.sql
generated: {by: "codex", at: "2026-08-25T01:37:12.518Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-25T01:37:12.518Z
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

## Durable recovery

REST/JSON is the durable command and query surface. The persona synchronization
feed returns a bounded baseline or retained changes after a monotonic cursor and
requires an explicit reset when continuity cannot be proven. WebSockets only
notify an authenticated owner that something changed; clients always return to
REST after reconnect or a lag hint. A game-session event carries only the
participant-authorized session UUID; durable state remains behind the same
persona-participant REST boundary. Game commands also use REST as the durable
mutation and replay surface; their WebSocket effect remains only an advisory
participant-local invalidation. Both transports preserve persona ownership
without exposing private account identity.

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
now implemented as well. Challenge orchestration, results, and playable rules
remain later slices on top of the same synchronization rule without treating
WebSocket delivery as durable truth.

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
private. The transport still has no public creation route; challenges, results,
playable rules, and game UI remain later boundaries.

Compiled Rust crates are the initial extension model. Loading user-supplied
native code is outside the first-alpha scope and would require a separate
sandbox decision.
