---
type: "Reference"
title: "Product and architecture boundaries"
openwiki_generated: true
sources:
  - id: openwiki-source-0d99cc708822fd795c83ba12
    resource: repo://client/qml/cartridge/CartridgePreview.qml
  - id: openwiki-source-c566a55d52a9744f7b26b7c4
    resource: repo://client/qml/cartridge/TrustedCartridgeSurface.qml
  - id: openwiki-source-0196de8872a3fef5b0b350d3
    resource: repo://client/qml/CartridgeController.qml
  - id: openwiki-source-a046e08cc1ba7740db940ad2
    resource: repo://client/qml/game/SignalSiegeSurface.qml
  - id: openwiki-source-da678ac479c336e5e6fc1d04
    resource: repo://client/qml/GameController.qml
  - id: openwiki-source-f73ad44f40942d16dc369861
    resource: repo://client/qml/OnboardingController.qml
  - id: openwiki-source-937883bc0b4873d5f0200c46
    resource: repo://CONSTITUTION.md
  - id: openwiki-source-2bc62522bf486443de88f261
    resource: repo://crates/client-cartridge-runtime/src/cache.rs
  - id: openwiki-source-939b835e7d6c679aae8394e7
    resource: repo://crates/client-cartridge-runtime/src/remote.rs
  - id: openwiki-source-bc8915a33f270bc28a270170
    resource: repo://crates/client-cartridge-runtime/src/service.rs
  - id: openwiki-source-37af4c6b51c86b62db25f85f
    resource: repo://crates/game-cartridge-renderer/Cargo.toml
  - id: openwiki-source-fdf115002c4aabad0babec70
    resource: repo://crates/game-cartridge-renderer/src/lib.rs
  - id: openwiki-source-877fd8b6ed8717d54fa8c17a
    resource: repo://crates/game-cartridge/Cargo.toml
  - id: openwiki-source-30abbd4fc5d09b185331836c
    resource: repo://crates/game-cartridge/src/acquisition.rs
  - id: openwiki-source-b4a2591d7d7f80d847ef95ed
    resource: repo://crates/game-cartridge/src/contract.rs
  - id: openwiki-source-71f8ccb7a1e293121205a368
    resource: repo://crates/game-cartridge/src/secure_store.rs
  - id: openwiki-source-07e2881dc5e4740f35a238ee
    resource: repo://crates/game-cartridge/src/store.rs
  - id: openwiki-source-30e12d7dfe374ac923c8ddbd
    resource: repo://crates/game-runtime/src/lib.rs
  - id: openwiki-source-df8490db5b51be8096630e7e
    resource: repo://crates/game-signal-siege/src/lib.rs
  - id: openwiki-source-66facc66e34ad7f2a74321e1
    resource: repo://crates/server/src/accounts.rs
  - id: openwiki-source-e61b285fcaa489b63922f43f
    resource: repo://crates/server/src/app.rs
  - id: openwiki-source-4b133589ca70bd174cf19eb9
    resource: repo://crates/server/src/connections.rs
  - id: openwiki-source-26aac996689c040c6aab6825
    resource: repo://crates/server/src/games.rs
  - id: openwiki-source-b2c7af59f511c4ed8a004fb0
    resource: repo://crates/server/src/inbox_api_tests.rs
  - id: openwiki-source-83e16151ac88c29a31cb79d2
    resource: repo://crates/server/src/mfa.rs
  - id: openwiki-source-94ddb58f2dc1a71ed1959533
    resource: repo://crates/server/src/operator_admin.rs
  - id: openwiki-source-54f6da1456b2b76d94d11b0e
    resource: repo://crates/server/src/personas.rs
  - id: openwiki-source-0e10f198b5749ecebf761185
    resource: repo://crates/server/src/provider_games.rs
  - id: openwiki-source-d943a78fae758ed47e30a12a
    resource: repo://crates/server/src/sessions.rs
  - id: openwiki-source-76060b846b9222af2c790243
    resource: repo://crates/server/src/signal_siege_api_tests.rs
  - id: openwiki-source-e7a72df5b89c1ac350ffe062
    resource: repo://crates/server/src/sync.rs
  - id: openwiki-source-98b4fef5bee3b5a0d880f16b
    resource: repo://docs/api.md
  - id: openwiki-source-408aa68caebee417a5a319b8
    resource: repo://docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md
  - id: openwiki-source-bfc109ee5d2c2f6c0f5c5f77
    resource: repo://docs/architecture/adr-0003-owner-operated-server-and-extension-boundary.md
  - id: openwiki-source-c22435ddb0c3a9abfe95d9af
    resource: repo://docs/architecture/game-cartridges.md
  - id: openwiki-source-872141f77f71851168245852
    resource: repo://docs/architecture/system-overview.md
  - id: openwiki-source-36d583174a7a0018316f71c7
    resource: repo://docs/operators/owner-operated-servers.md
  - id: openwiki-source-c3d1d450d3a3561b368e5307
    resource: repo://docs/planning/ROADMAP.md
  - id: openwiki-source-85dba8f87dd5947de337aca5
    resource: repo://docs/product-charter.md
  - id: openwiki-source-674113ba65eebb6f842b2dda
    resource: repo://migrations/0008_conversation_local_message_sequences.sql
  - id: openwiki-source-4331166a21e12c8c40994c1e
    resource: repo://migrations/0016_operator_reporting_and_audit.sql
  - id: openwiki-source-d85e6ea816d7c91e9828f7b2
    resource: repo://packaging/arch/omarchygs
  - id: openwiki-source-8df9ad1a3495f8360740ff03
    resource: repo://scripts/test-game-cartridge-sdk.sh
generated: {by: "codex", at: "2026-08-27T01:49:04.244Z"}
---

# Product and architecture boundaries

## Product focus

Omarchy Gaming System is game-first. Connections, private inboxes, challenges,
server-authoritative matches, and persistent results define the first playable.
Public message boards remain outside the current identity and private-alpha
scope; they may be reconsidered later only as a complementary community
surface.

## Authority and identity

OmarchyGS is authoritative for authentication, authorization, identities,
social state, catalog and launch policy, the participant-private game-session
envelope, public result and achievement policy/projections, audit, suspension,
and durable recovery. A `platform_compiled` session also keeps OmarchyGS as its
sole rules/state/revision owner. The Door Legends v1 pilot may instead pin one
operator-registered provider release as the sole owner of its scoped rules,
private gameplay state, turns, game time/randomness, revision, and outcome.
Transport handlers translate requests and responses; domain modules own
permissions and invariants.

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

Account admission is controlled by the community owner through the
database-local operator executable. It issues bounded, expiring invitation
bearer codes for trusted-channel delivery; the server stores only their
digests, consumes one atomically with account creation, and permits revocation
only before use. Invitation inventory and audit remain operator-only and never
become player-network administration routes.

That account authority now creates, inventories, and edits personas without
accepting a client owner field. Inventory filters by the authenticated account,
and mutation predicates on both account and persona IDs. Persona responses
contain only seven public profile fields. Exact canonical handle lookup is
intentionally public, but neither it nor an authenticated response reveals the
owning account or session.

Reporting stays on that same persona boundary. An authenticated owned persona
may file a bounded report about another public persona, but the player receipt
contains no account ownership, operator state, other reports, or report queue.
The subject account identifier and report detail are available only to the
trusted database-local operator command. Operator mutations are not network
API routes: private alpha permits reversible account suspension/reactivation,
terminal report disposition, and registration-invitation issue or pre-use
revocation with an immutable same-transaction audit event. Suspension revokes
current sessions without deleting personas,
messages, games, MFA state, reports, or provider state.

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

Platform backup is a separate operator responsibility from cursor recovery.
The implemented drill restores the PostgreSQL application schema and
representative identity, social, inbox, game, report, suspension, and audit
state into an isolated database, then starts the production server and proves a
pre-suspension token remains invalid. It also requires the restored server's
public UUID to match the source, because that continuity identity belongs to
the platform database. Copying or forking the database therefore copies the
UUID too; intentional fork or rotation tooling remains future work.
`OGS_MFA_ENCRYPTION_KEY` is outside the database and must be protected and
restored separately. Provider authority uses its own database and independent
recovery procedure.

## Ordered identity work

The four roadmap identity outcomes are intentionally sequenced:

1. Invitation-gated account registration atomically consumes a valid bearer,
   establishes normalized account identity and Argon2id password storage, and
   creates neither a session nor a persona.
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
Signal Siege v1 adds the immutable deterministic solo definition, while v2
adds exact two-person alternating play without relabeling existing v1 sessions.
The keyboard-first QML connector now covers catalog discovery, challenge
creation and acceptance, authoritative turns, terminal result, and refetch
recovery without treating WebSocket delivery as durable truth. Door Legends v1
adds one operator-pinned remote authority pilot with platform-owned result and
achievement projections. Player reporting, the database-local report and invitation queues,
reversible account containment, immutable platform audit, and an isolated
platform backup/restore proof are also implemented. The public discovery
contract and QML client now provide stable UUID continuity, protocol/capability
negotiation, and bounded public-only profiles for selecting independent
communities without sharing credentials or persona authority. Gate stage 22 proves the
software path for private-alpha admission, but it does not substitute for the
first human event's documented issue, trusted delivery, onboarding, gameplay,
safety, and evidence sequence. Marketplace synchronization, server admission,
and independently trusted player acquisition, caching, and server-profile
mounting are implemented. Federation, server identity fork/rotation, remote
administration, mounted-cartridge gameplay launch, and external-provider
onboarding remain later slices.

The current server is a local development slice. Bearer tokens require
production TLS in transit, and public login requires distributed attempt
throttling. The in-process four-job Argon2 limit bounds memory-heavy work but is
not a substitute for either deployment control. TOTP also requires protected,
replicated encryption-key management and does not provide phishing resistance.

Random registration callers cannot probe usernames because malformed, absent,
expired, revoked, and mismatched used invitations all receive the same generic
denial. A holder of a valid unused invitation can still distinguish an existing
canonical username through HTTP 409; the failed attempt does not consume the
invitation. That narrower account-name enumeration risk is accepted for the
controlled private alpha. Public deployment still requires deliberate edge
rate limits and a fresh decision about whether the conflict contract is
appropriate; MFA does not remove this registration-side disclosure.

## Game authority boundaries

Synchronization and games build on the implemented persona connection and
private inbox boundaries. Conversation-local message sequences remain history
cursors, while the persona synchronization cursor spans visible resources and
drives reconnect recovery.

The current compiled game interface receives only a human-player count for
initialization or the current state, actor seat, and bounded object command for
a transition. It returns deterministic bounded object JSON plus an
authoritative active/completed lifecycle and cannot query
PostgreSQL, inspect accounts or sessions, read the clock, use the network, or
draw ambient randomness. Server orchestration owns the transaction, persona
authorization, durable version and snapshot, ordered seats, revision and replay
identity, and participant sync events.
Session reads expose public personas and stored state only to a participating
owned persona and do not consult today's registry, so a newer process cannot
silently relabel old state. The participant command route returns only session
ID, revision, lifecycle, and state; receipts and conflict-side current
revisions stay private. Challenge acceptance owns the public path into an exact
two-person session. A separate owner-scoped route admits only an exact
one-human definition, checks durable replay before current registry and
active-cap policy, and creates seat zero for the owned persona. Signal Siege
represents its opponent entirely inside deterministic rules and bounded state,
so it creates no bot account, persona, credential, or participant row.
Signal Siege v2 separately admits exactly two people, alternates authoritative
seat turns, and completes on core destruction or a fixed turn bound. Completed
history, exact replay, and QML gameplay are implemented; result-derived
platform effects remain a later boundary.

The QML authority boundary keeps the bearer in `OnboardingController` and gives
`GameController` only the selected-persona request gateway. The game controller
validates exact catalog, challenge, session, participant, authority, and v1/v2
state relationships before deriving presentation. A transport timeout retains
the exact mutation identity for explicit retry; a revision conflict triggers an
authoritative refetch rather than silently rebasing the player's action. The
client currently refreshes on entry and action and opens no game polling or
WebSocket lifetime.

For registered-provider sessions, migration 0015 prohibits a writable local
gameplay snapshot: the platform envelope pins one exact release and keeps local
state null. The authenticated broker sends only short-lived scoped grants and
pairwise subjects, preserves operation idempotency and expected provider
revision, and projects only bounded authenticated views. There is no compiled
failback.

Provider callbacks have no platform effect until OmarchyGS authenticates the
exact message, rechecks lifecycle and pinned identity, claims the durable
receipt, validates allowlisted result and achievement policy, and commits the
projection, audit, and persona-sync invalidation atomically. Suspension denies
new launches, commands, and callbacks but retains private reads and explicit
reconciliation. Reactivation requires reconciliation before readiness;
retirement is terminal.

This authorization is limited to the operator-pinned Door Legends v1
first-party pilot. External or self-service provider registration, direct
client-provider networking, raw provider UI, and loading user-supplied native
code remain outside the first-alpha boundary and require separate decisions.

### Owner-operated deployment and future extensions

ADR-0003 accepts each independently owner-operated OmarchyGS origin as its own
community trust domain. Its standard server owns the accounts, personas,
social state, catalog and launch policy, platform envelopes, projections,
audit, and recovery created there. Compatible servers do not implicitly share
identity, moderation, or history; federation remains a separate future design.
The official client can now save and explicitly select up to sixteen exact
public-only profiles. A remembered canonical origin must present the same UUID
before account access, and switching origins clears all live bearer, MFA,
username, and persona authority before a request. These profiles are isolated
connection choices, not a global account or federated community layer.

The marketplace is distribution and review infrastructure rather than a global
gameplay or catalog authority. Publisher integrity, marketplace review, the
selected server's admission, and the player's configured marketplace trust key
remain separate decisions. The administrator imports and admits an exact
release; players see the server-scoped metadata catalog; and a separately
capability-advertised route serves only that exact release. The native client
verifies the full marketplace key against its own configured trust anchor,
rechecks every release and admission proof, and writes private cached content
plus an exact server-profile mount. The selected server cannot replace the
client's marketplace key. An operator-custom cartridge has no
marketplace-review claim, but it remains signed inert data subject to every
official-client package, schema, media, capability, digest, and trusted-render
check.

Executable extension families stay separate. Portable game rules use the
authenticated provider boundary and a future public Provider SDK; general
server behavior uses a separately versioned module base. Future module hooks
must be capability-scoped and typed, route protected mutations through core
authorization, and define isolation, resource, failure, audit, compatibility,
disable, upgrade, rollback, and recovery behavior. No general module runtime,
dynamic Rust plugin ABI, marketplace service, operator-custom installer, or
external-provider onboarding is implemented or authorized by this direction.

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
external monotonic authority. The main client now browses platform catalog
records, plays compiled Signal Siege, and separately acquires, privately caches,
updates, removes, and mounts exact admitted signed cartridges when independent
marketplace trust and server acquisition are available. A mount is only a
verified profile pointer into inert content: it does not create a game session,
prepare a trusted render plan, or grant executable frontend authority. The
server still does not ingest cartridge files into its public game catalog.
Signal Siege's platform-owned presenter reuses inert
repository components without claiming a signed origin, content digest, or
`omarchygs.render-plan/v1` provenance. The optional provider runtime instead
lists only an operator-enabled manifest already pinned in the provider registry.

The Door Legends mode keeps OmarchyGS as the authenticated broker and platform
authority for accounts, sessions, MFA, personas and avatar projections, social
state, catalog and launch policy, achievements, notifications, audit,
suspension, and the platform session envelope. The provider receives a
short-lived audience/game/version/session/scope-bound pairwise persona
grant—never account identity, credentials, reusable device tokens, or database
access. See [Game Cartridges](game-cartridges.md) for the full trust, graphics,
failure, and rollout model.
