---
type: "Reference"
title: "Runtime foundation"
openwiki_generated: true
sources:
  - id: openwiki-source-490417654c55d88090cb369e
    resource: repo://client/qml/components/OgsScreenHeader.qml
  - id: openwiki-source-f4ccc0eff8d8cee134cf3ed5
    resource: repo://client/qml/components/OgsStatusBanner.qml
  - id: openwiki-source-998b0f5a7b56d7475101b7a2
    resource: repo://client/qml/components/OgsTheme.qml
  - id: openwiki-source-da678ac479c336e5e6fc1d04
    resource: repo://client/qml/GameController.qml
  - id: openwiki-source-d392f8f0962c50f0d66e0629
    resource: repo://client/qml/Main.qml
  - id: openwiki-source-f73ad44f40942d16dc369861
    resource: repo://client/qml/OnboardingController.qml
  - id: openwiki-source-4f5334e859a4d83e2a196fcf
    resource: repo://client/qml/SocialController.qml
  - id: openwiki-source-fc035ef77d2451c6e8138211
    resource: repo://client/qml/tests/fixture/tst_accessibility.qml
  - id: openwiki-source-30e12d7dfe374ac923c8ddbd
    resource: repo://crates/game-runtime/src/lib.rs
  - id: openwiki-source-df8490db5b51be8096630e7e
    resource: repo://crates/game-signal-siege/src/lib.rs
  - id: openwiki-source-66facc66e34ad7f2a74321e1
    resource: repo://crates/server/src/accounts.rs
  - id: openwiki-source-e61b285fcaa489b63922f43f
    resource: repo://crates/server/src/app.rs
  - id: openwiki-source-ba203ea2e600f294ab58ef02
    resource: repo://crates/server/src/bin/omarchygs-admin.rs
  - id: openwiki-source-2c054a2481343f8aacaf65ae
    resource: repo://crates/server/src/challenge_api_tests.rs
  - id: openwiki-source-a3892e0554790e3efc606fe1
    resource: repo://crates/server/src/challenges.rs
  - id: openwiki-source-b691fa90e62f9509a0c1869a
    resource: repo://crates/server/src/config.rs
  - id: openwiki-source-4b133589ca70bd174cf19eb9
    resource: repo://crates/server/src/connections.rs
  - id: openwiki-source-17c0b29fc571b7362a541c89
    resource: repo://crates/server/src/credentials.rs
  - id: openwiki-source-a243b385d49ea9224173d77a
    resource: repo://crates/server/src/game_api_tests.rs
  - id: openwiki-source-26aac996689c040c6aab6825
    resource: repo://crates/server/src/games.rs
  - id: openwiki-source-a13fe4db1eee073d0a7e2c4d
    resource: repo://crates/server/src/main.rs
  - id: openwiki-source-1f3bbf6debbcae2e3b3c3b61
    resource: repo://crates/server/src/mfa_api_tests.rs
  - id: openwiki-source-83e16151ac88c29a31cb79d2
    resource: repo://crates/server/src/mfa.rs
  - id: openwiki-source-94ddb58f2dc1a71ed1959533
    resource: repo://crates/server/src/operator_admin.rs
  - id: openwiki-source-54f6da1456b2b76d94d11b0e
    resource: repo://crates/server/src/personas.rs
  - id: openwiki-source-0e10f198b5749ecebf761185
    resource: repo://crates/server/src/provider_games.rs
  - id: openwiki-source-e4423ee4de83f38bd240bf8b
    resource: repo://crates/server/src/reports.rs
  - id: openwiki-source-d943a78fae758ed47e30a12a
    resource: repo://crates/server/src/sessions.rs
  - id: openwiki-source-76060b846b9222af2c790243
    resource: repo://crates/server/src/signal_siege_api_tests.rs
  - id: openwiki-source-46fb4135d6a71efad1062c0d
    resource: repo://crates/server/src/sync_api_tests.rs
  - id: openwiki-source-e7a72df5b89c1ac350ffe062
    resource: repo://crates/server/src/sync.rs
  - id: openwiki-source-72d1d6cc0f49cdc59cc77234
    resource: repo://migrations/0001_identity_foundation.sql
  - id: openwiki-source-cc81194d2be7a8c404889b15
    resource: repo://migrations/0002_canonical_account_usernames.sql
  - id: openwiki-source-358c515ea0211d4b7f4f722a
    resource: repo://migrations/0003_device_session_metadata.sql
  - id: openwiki-source-80556fe53504b4a8da1cd08a
    resource: repo://migrations/0004_canonical_persona_handles.sql
  - id: openwiki-source-0a7e68dc488e993e49a7053c
    resource: repo://migrations/0005_totp_two_factor_authentication.sql
  - id: openwiki-source-db59744a1b86bb366d2bc988
    resource: repo://migrations/0006_persona_connections_and_blocks.sql
  - id: openwiki-source-c873ebda6240c16d74d77455
    resource: repo://migrations/0009_persona_sync_events.sql
  - id: openwiki-source-48796f93204ecc1ec11191f8
    resource: repo://migrations/0010_game_registry_and_sessions.sql
  - id: openwiki-source-c79a682b6a5bf55579dee651
    resource: repo://migrations/0011_idempotent_revision_checked_game_commands.sql
  - id: openwiki-source-cb6494f7cbf0d5d23ffe082a
    resource: repo://migrations/0012_game_challenges.sql
  - id: openwiki-source-926664a4167297129df76802
    resource: repo://migrations/0013_signal_siege_and_solo_sessions.sql
  - id: openwiki-source-4331166a21e12c8c40994c1e
    resource: repo://migrations/0016_operator_reporting_and_audit.sql
  - id: openwiki-source-a5928e7ee39885995efdc170
    resource: repo://scripts/dev.sh
generated: {by: "codex", at: "2026-08-26T21:07:26.522Z"}
---

# Runtime foundation

## Startup flow

`crates/server/src/main.rs` owns process startup. It initializes tracing, loads
environment-backed configuration, connects a PostgreSQL pool, applies embedded
SQLx migrations, starts the PostgreSQL-backed synchronization listener and
in-process notification hub, constructs the production registry containing
Signal Siege v1 and v2, binds the configured listener, and serves the Axum router with
graceful shutdown. A failure to connect, migrate, subscribe, bind, or serve
carries context and stops startup instead of exposing a partially ready
process. Shutdown also aborts the listener task.

After migrations, startup may also construct a production `ProviderRuntime`
and inject it into the router. The provider runtime is absent when all provider
environment values are absent. It is enabled only when the grant-signing seed,
pairwise secret, message-signing seed, and callback authority are all present
and valid; a partial or malformed set stops startup rather than exposing a
partially configured broker.

Configuration lives in `crates/server/src/config.rs`. `DATABASE_URL` and
`OGS_BIND_ADDRESS` can override development defaults; `BBS_BIND_ADDRESS` is a
transitional fallback only when the new variable is absent. The defaults point
only at loopback and use the `omarchy_gaming_system` development database.
`OGS_SERVER_NAME` supplies the public community label and defaults to
`OmarchyGS Community`; an explicit value must contain 1–64 trimmed,
non-control characters or startup stops before binding.
Startup also requires `OGS_MFA_ENCRYPTION_KEY` as an unpadded base64url-encoded
32-byte value. All replicas and restored instances that need to verify enrolled
authenticators must receive the same protected key. Keep network, key, and
credential policy explicit when introducing a non-local deployment profile.
The optional provider secrets use unpadded base64url and the callback authority
must be a bounded lowercase DNS authority with an optional nonzero port.

## Health flow

`app::router` installs `GET /health` and shares the database pool through Axum
state. The handler executes `SELECT 1`:

- a result of `1` produces HTTP 200 and reports both service and database as
  `ok`;
- any other result or database error produces HTTP 503 and reports a degraded
  service with the database unavailable.

Both documents identify the service as `omarchy-gaming-system`. The response
construction has focused identity, healthy, and degraded unit tests. The
delivery smoke supplies the live proof that migrations, PostgreSQL, HTTP
serialization, and the QML consumer work together.

## Server discovery flow

`app::router` also installs unauthenticated, no-store
`GET /.well-known/omarchygs`. The exact successful document contains only
`service`, `server_id`, `server_name`, `protocol_version`, and `capabilities`.
Protocol 1 publishes one lexically ordered set of currently implemented
versioned capabilities; the registered-provider capability appears only when
the optional provider runtime exists. A missing durable identity returns a
generic `503 server_discovery_unavailable` rather than database detail. This
compatibility contract is separate from `/health`, which remains operational
liveness.

Migration `0018_server_identity.sql` creates one checked singleton row whose
UUID is randomly generated by PostgreSQL. Update, delete, and truncate triggers
reject ordinary mutation, so the identifier follows the application database
through restart, backup, and restore. It is public continuity metadata rather
than a secret or cryptographic server identity proof; production remote origins
still depend on HTTPS authentication. Copying or forking the database also
copies the UUID until a future explicit fork/rotation workflow exists.

## Account registration flow

`app::router` also installs `POST /v1/accounts` with a 1 KiB request-body cap.
The exact request fields are `invite_code`, `username`, and `password`. The
handler delegates the admission and credential work to
`accounts::register_account`, returns only the account ID and canonical
username, and marks both successful and error responses `no-store`.

The account domain owns the sensitive work:

- usernames are trimmed, ASCII-lowercased, and restricted to a 3–32 byte ASCII
  namespace beginning with a letter or digit;
- passwords are not trimmed and must contain 12–128 bytes;
- invitation codes must be the canonical 48-character `ogsi_` bearer format;
- shared credential code runs password hashing through `spawn_blocking` with an
  OS-random salt and Argon2id v19 parameters `m=19456`, `t=2`, and `p=1`;
- the invitation is resolved by its SHA-256 digest, and raw codes are never
  persisted;
- account insertion and invitation consumption commit in one transaction under
  an invitation row lock, so concurrent consumers create exactly one account;
- the resulting PHC string is inserted into PostgreSQL, and only a valid,
  unused invitation can reach the named username uniqueness check and its
  public `409` conflict outcome. That conflict rolls back without consuming the
  invitation.

The first successful consumption returns `201`. Repeating the same used code
with the exact canonical username and password performs Argon2 verification and
recovers the same account receipt with `200`; a different username still takes
the same password-verification path before denial. Malformed, absent, expired,
revoked, concurrent-losing, and changed-credential invitations all collapse to
the same `403 invalid_invitation` response. Validation failures become stable
`422` errors. Unexpected database, task, or hashing failures collapse to a
generic `500` response rather than exposing internal or password-derived data.

Registration and login share a four-permit semaphore before entering their
memory-hard blocking work. Missing-account login still performs a dummy Argon2
operation, while wrong-password, suspended, and disabled accounts verify the
stored hash before returning the same `invalid_credentials` response.

## Device-session flow

`POST /v1/sessions` accepts credentials and a bounded device name. When MFA is
disabled, the session domain generates 32 OS-random bytes, encodes them as an
`ogs1_` base64url token, stores only `SHA-256(token)`, and returns the raw token
once in a `Cache-Control: no-store` response. When MFA is enabled, the same
primary-credential check returns a challenge and creates no session until the
factor succeeds.

During the local pre-alpha transition, parsing also accepts structurally valid
legacy `bbs1_` tokens. It hashes the complete presented token, so an existing
stored digest still resolves without a migration and then passes through the
same account-status, revocation, absolute-expiry, and idle-expiry predicates as
a new token. New sessions never emit the legacy prefix.

`GET /v1/sessions` and `DELETE /v1/sessions/{session_id}` accept that value only
as an `Authorization: Bearer` credential. Authentication atomically advances
last use only if PostgreSQL finds an active account and a session that is:

- not revoked;
- inside its 30-day absolute expiry;
- active within the last seven days.

Inventory filters by the authenticated account and marks the presenting device
as current. Revocation predicates on both the authenticated account and target
session ID, so foreign and absent UUIDs share the same not-found result. The
current device may revoke itself, after which the token fails immediately.

## Opt-in TOTP MFA flow

The account-level MFA routes are deliberately separate from public persona
identity:

- authenticated `POST /v1/account/mfa` rechecks the current password, creates a
  random 160-bit TOTP secret, and returns the Base32 secret and `otpauth` URI;
- `POST /v1/account/mfa/confirm` must receive a valid code within ten minutes
  before the authenticator becomes active and ten independently generated
  recovery codes are returned once;
- `GET /v1/account/mfa` exposes only enabled state and unused recovery-code
  count;
- `DELETE /v1/account/mfa` requires the current password plus a valid TOTP or
  unused recovery code, then removes the authenticator, recovery codes, and
  outstanding challenges without revoking existing device sessions.

The recoverable TOTP secret is AES-256-GCM ciphertext with a random nonce and
the account UUID as associated data. Recovery codes and login challenges are
stored only as SHA-256 digests. All enrollment, confirmation, status, challenge,
and completed-session responses that contain authentication state are marked
`Cache-Control: no-store`.

An enabled account's password login creates a five-minute `ogm1_` challenge
instead of a device session. The account row is locked during issuance, so up
to ten unexpired challenges can coexist for independent device attempts; an
additional valid-password request receives HTTP 429 without consuming or
replacing any live challenge. Expired and consumed challenges are cleaned up
during later issuance.

`POST /v1/sessions/mfa` locks the selected challenge, account, and authenticator
in one transaction. A successful six-digit TOTP or unused recovery code is
consumed together with the challenge before the new device session is issued.
The RFC 6238 profile uses HMAC-SHA-1, a 30-second step, and one adjacent step on
either side for clock drift; `last_used_step` prevents replay. Recovery codes
are also single-use. Five consecutive factor failures across challenges lock
verification for five minutes, while successful verification clears the
failure state. These local controls do not replace distributed public-edge
throttling or TLS, and TOTP is not phishing-resistant.

## Persona flow

Authenticated `POST /v1/personas` and `GET /v1/personas` create and list
personas for the account derived from the presented device session.
Authenticated `PATCH /v1/personas/{persona_id}` accepts only handle, display
name, bio, and status fields and predicates the update on both the persona UUID
and derived account UUID. A foreign, absent, or malformed persona ID therefore
shares the same not-found response, and an unauthenticated malformed ID still
fails authentication first.

The persona domain owns the profile rules:

- handles are trimmed, ASCII-lowercased, and restricted to a 3–24 byte ASCII
  namespace beginning with a letter or digit;
- display names contain 1–64 trimmed, non-control Unicode characters;
- bios contain at most 1,000 Unicode characters and allow tabs and newlines but
  no other controls;
- status messages contain at most 160 trimmed, non-control characters;
- empty edits are rejected, and the named handle-uniqueness conflict becomes a
  stable `409` without revealing the existing owner.

Public `GET /v1/personas/by-handle/{handle}` canonicalizes one exact handle and
returns the same not-found result for invalid and absent values. Persistence
rows are first narrowed into a domain model and then into an explicit transport
DTO containing only ID, handle, display name, bio, status message, and created/
updated timestamps. Account IDs and authentication data are not fields in
either response model. Successful authenticated persona responses carry
`Cache-Control: no-store`; public lookup is intentionally enumerable by handle.

## QML onboarding client flow

`client/qml/Main.qml` is now a keyboard-first onboarding shell rather than a
health-only connector. Its controller moves through connection, account access,
optional MFA, persona inventory or creation, and an authenticated home. Account
registration includes a masked invitation field, transmits the bearer only in
the registration request, and clears it after completion, mode changes, and
server changes. Registration deliberately returns to sign-in; session creation immediately
loads the owned persona inventory; and either an owned selection or successful
persona creation establishes the active persona for the home screen.

The shell and all ten routed screens share `OgsTheme`, `OgsScreenHeader`, and
`OgsStatusBanner` rather than defining independent palettes or status grammar.
Every route exposes a semantic heading, a visible non-color-prefixed state, a
keyboard navigation hint, and an explicit initial focus target. The shell adds
an accessible application-state rail, persistent keyboard legend, and one
shell-owned EXIT button without moving navigation, data, or session authority
out of the existing controllers. EXIT calls only the normal
`ApplicationWindow.close()` lifecycle; it does not log out or revoke the
durable device session. Under `scripts/dev.sh`, the QML process returning lets
the launcher's existing cleanup trap stop its child Rust server while leaving
PostgreSQL running. Visual text explicitly selects `Text.PlainText`; layout,
focus, keyboard/pointer close, and session-preservation assertions cover both
the 920×600 default and 640×420 minimum window.

`ApiClient.qml` accepts only a bare HTTP or HTTPS origin. HTTP is limited to
`localhost`, `127.0.0.1`, and `[::1]`; every remote host requires HTTPS. Each
request cancels and supersedes the prior generation, has a ten-second timeout
and a 256 KiB response limit, and rejects a final response URL outside the exact
selected origin and path. The controller parses only nonempty JSON objects and
accepts exact bounded response shapes, including the OmarchyGS discovery
identity and compatibility document, token formats, future session and MFA
expiry, public persona fields, and a small allowlist of stable public error
codes.

`ServerProfiles.qml` persists one exact JSON array through Qt `Settings` at the
platform configuration location. It permits at most sixteen records and 16 KiB
total, and every record contains only canonical origin, server UUID, public
name, protocol version, and an ordered bounded capability array. Unknown keys,
credentials, malformed or noncanonical values, duplicate origins or UUIDs, and
oversized state reset the inventory to empty. Persisted state never triggers an
automatic connection.

Direct entry offers connect-once or save-and-connect, while saved rows support
explicit connect and removal. Typing an already saved canonical origin still
loads its expected UUID. Discovery must use protocol 1 and include invitation
registration, device sessions, and personas before account access; bounded
future capabilities are retained. A UUID mismatch fails closed until the old
profile is explicitly removed.

Bearer tokens and MFA challenge values live only in the QML process. Changing
the server first cancels the prior request generation and clears bearer, MFA,
username hints, persona inventory, and persona selection before configuring or
requesting another origin. Logging out, canceling or expiring MFA, receiving a
terminal MFA failure, receiving `invalid_session`, or rejecting an authenticated success
clears the applicable authority before returning to account access. A
superseded request generation cannot update current state. This is a local
client boundary, not a substitute for TLS certificate policy, server-side
authentication and authorization, or public-edge rate limiting.

After a valid owned persona is selected, the same authority controller
allowlists home, social, inbox, games, challenges, and gameplay navigation and
exposes a player-prefixed request gateway without exposing its bearer.
`SocialController` and `GameController` receive that gateway and the selected
public persona, derive every authenticated actor path from that persona ID, and
cancel and clear state when the actor changes. The production root owns one of
each controller and refreshes the appropriate durable REST inventory when a
corresponding screen becomes active.

Social refresh serially loads incoming/outgoing requests, accepted
connections, and the actor's private block inventory. Exact public-handle
lookup is performed without a bearer but only from an established player
session; a successful exact profile is then used for the UUID-scoped connection
command. Accepted requests, pending/accepted removal, block, and unblock all
reuse the server's durable REST commands and refresh the affected inventories.
The client neither models another persona's private block direction nor
interprets generic relationship-policy failures as hidden state.

Inbox refresh loads at most 100 conversation summaries. Opening one replaces
local history with an ascending page of at most 50 exact tagged messages;
`next_before` is the only older-history cursor, and both page-local and
cross-page sequence checks must pass before older messages are prepended. User
sends contain only a trimmed, control-safe body of at most 4,000 Unicode
characters. The committed response must be a user message from the selected
actor with a newer sequence before it is appended, and unread state advances
only through a validated loaded message ID.

Every relationship, conversation, message, read receipt, public profile, and
error response has an exact bounded client schema. Unknown system variants,
extra private fields, unsafe integers, invalid timestamps, malformed JSON, and
oversized responses are rejected without partially accepting state. Messages
are converted only to allowlisted plain text. Request completion must match
both generation and operation; a valid `401 invalid_session` clears the social
controller before the authority owner clears bearer, persona inventory, and
selection. The social and game client slices refresh durable REST truth on
entry, action, or user request and deliberately introduce no polling or
WebSocket lifetime. The game controller serially loads the bounded catalog and
session or connection/challenge inventories, keeps an uncertain mutation's
exact idempotency identity for explicit retry, and refetches the session after
a committed command or revision conflict. Catalog, challenge, session, command,
and exact Signal Siege v1/v2 state documents must pass closed schemas,
participant uniqueness/cardinality, actor-direction, lifecycle, and cross-field
checks before presentation.

The Games and Challenges screens expose keyboard-first catalog, history,
connection, and lifecycle controls. Gameplay maps only a validated compiled
Signal Siege view model into platform-owned plain-text, meter, status, and
button components. That surface is trusted application UI: it does not wrap
the state in `omarchygs.render-plan/v1`, claim an authenticated cartridge
origin, or make provider-owned sessions executable.

## Persona connection and block flow

Connection and block routes are persona-scoped, but path identity never grants
authority by itself. The `connections` domain authenticates the Bearer token,
derives the private account ID, and requires the acting persona to belong to
that account. A missing, malformed, or foreign actor returns the same
`persona_not_found` response. Targets must exist on another account; invalid,
same-account, or blocked targets share `connection_unavailable` for
state-creating commands.

`PUT /v1/personas/{actor}/connection-requests/{target}` creates one outgoing
pending request and safely returns the existing request on retry. Inventories
split pending rows into stable incoming and outgoing arrays. Creation enforces
at most 100 pending rows in either direction: it checks the requester's
outgoing count and the addressee's incoming count after locking both persona
roots, so concurrent boundary attempts serialize and an existing outgoing
request remains retryable at the limit. Only the addressee can accept through
`PUT /v1/personas/{actor}/connections/{requester}`; the accepted row then
appears as one mutual connection to both participants. Either participant can
use `DELETE` on the connection path to cancel pending state or remove accepted
state. Authenticated delete retries intentionally return `204` even when the
target state is absent.

Every pair mutation locks both extant persona rows in ascending UUID order
before checking ownership, target policy, blocks, or relationship state. This
serializes opposite requests, concurrent acceptance, removal, and blocking
without cross-table invariant windows. One canonical pair row stores either
`pending` requester/addressee direction or an `accepted_at` timestamp; database
checks bind those fields to the same canonical pair.

Blocks use a separate directional `(blocker_id, blocked_id)` row and are listed
only for the blocker. Creating or retrying a block deletes any pending or
accepted pair row in the same transaction. Requests then fail with the same
generic result in both directions. This keeps block rows and inventories from
direct disclosure, but a caller may still infer direction by comparing a
denied interaction with its own block inventory; the product does not promise
to conceal that residual inference. Unblock is idempotent and does not restore
an earlier request or connection. Social responses carry
`Cache-Control: no-store` and embed the existing seven-field public persona
shape rather than account ownership or persistence identifiers.

## Private inbox flow

Only a real pending-to-accepted transition calls the inbox domain inside the
connection transaction. It creates or reuses one conversation for the
canonical persona pair and appends exactly one typed, server-authored
`connection_accepted` message. Retrying an already accepted command returns the
connection without appending another event.

The inbox routes expose bounded conversation inventory, bounded message
history, body-only user sends, and monotonic read acknowledgement. Every
operation authenticates the device session and owner-scopes the acting persona;
conversation inventory and history additionally require that persona to be one
of the durable participants. Responses embed public persona profiles but never
the peer's account ownership or private read cursor. Pages default to 50 and
cap at 100. The inbox router applies `Cache-Control: no-store` to successful
documents, domain failures, and request-extractor rejections.

Sending first locks both persona roots through the connection domain, verifies
that the pair is currently accepted and unblocked, then locks the conversation
row and appends. Removal and blocking use the same persona-root order, so a
send either commits before the social mutation or observes its denial. The
message remains durable in the former case. Existing history stays readable
after removal or block; unblock alone does not restore send permission.

Each conversation has its own positive message sequence. The conversation row
lock allocates the next value and serializes concurrent user and system
messages. The sender's read position advances with its new message; explicit
read acknowledgement uses `GREATEST`, so older retries cannot move a cursor
backward. This sequence is a conversation history cursor only, not the
cross-resource persona synchronization cursor.

## Game challenge flow

The authenticated challenge routes are nested below an owned acting persona.
Creation accepts a UUID idempotency key, another persona, and one exact game
key/version. A new challenge requires a different-account peer whose connection
is still accepted and unblocked, and the selected compiled manifest must admit
the two people in this v1 flow. Canonical persona-root locks serialize
relationship policy, equivalent pending requests, lazy expiry, and the fixed
limits of 100 unexpired outgoing plus 100 unexpired incoming challenges per
persona. The server fixes expiry at seven days.

The challenger-scoped idempotency identity is durable. Reusing it for another
target, key, or version conflicts; an exact retry returns the retained
participant-authorized representation without another challenge, inbox
message, or synchronization event. Because this is recovery of a committed
write rather than a new interaction, it remains replayable after the pair's
relationship changes or that version leaves the current process registry.
Concurrent first requests that miss the initial lookup still converge under
the pair locks and a second transaction-local replay check.

Inventory and detail authenticate ownership and return only rows in which the
acting persona is challenger or challenged. Pages are newest-first, default to
50, cap at 100, and include both pending and terminal history. Reads and
mutations resolve due pending rows to `expired` inside their transaction.
Responses contain the exact game identity, incoming/outgoing direction,
status, both public persona projections, optional accepted session ID, and
timestamps; they omit account IDs, the idempotency key, relationship/block
state, registry internals, and game state.

Only the challenged persona may accept or decline; only the challenger may
cancel. Decline and cancellation create terminal history without a session.
Acceptance additionally rechecks that the pair is connected and unblocked,
then invokes `games::create_session` with challenger seat 0 and challenged seat
1 in the same PostgreSQL transaction. The session snapshot and seats, accepted
challenge link, typed inbox event, and challenge, conversation, and session
invalidations therefore commit together or roll back together. A retry of the
same terminal operation is effect-free, and competing terminal transitions
have one winner.

Creation and first terminal transitions append one server-authored message to
the pair's existing private conversation. The variants are
`game_challenge_created`, `game_challenge_accepted`,
`game_challenge_declined`, and `game_challenge_cancelled`; every variant names
the public actor and challenge ID, while acceptance alone also names the game
session. Challenge synchronization similarly carries only
`game_challenge_changed` plus the challenge UUID. The inbox record is immutable
history, REST challenge state is current truth, and WebSockets remain generic
wakeup hints.

## Durable persona synchronization and live hints

Every committed social, inbox, game-challenge, game-session, or game-command
mutation that changes visible state appends a typed row to each affected
persona's monotonic synchronization stream in the same transaction as the
domain write. It then emits a PostgreSQL notification carrying only the persona
ID; the durable cursor remains in REST. No-op retries append nothing. Retention
pruning is transactional too, so the oldest retained cursor defines the
recovery boundary. Game invalidations carry only the challenge or session UUID,
never state, participants, or conversation details.

`GET /v1/personas/{persona_id}/sync` is the durable source of truth. Without an
`after` cursor it returns a bounded baseline; with one it returns strictly newer
events in order. A cursor outside retained history, ahead of the persona, or
separated from the first returned row produces `reset_required` instead of a
silently incomplete page. Authentication and persona ownership are checked
before state is disclosed.

`GET /v1/personas/{persona_id}/sync/live` upgrades only a header-authenticated
owner; query-string tokens are rejected. The socket sends `ready`, advisory
`changed`, and lag-recovery `resync_required` messages, never durable domain
payloads. Incoming frames are capped at 1 KiB, and permits cap connections at
five per persona, twenty per account, and 256 per process. The prepared socket
retains UUIDs rather than a raw token and revalidates the session without
extending idle lifetime before readiness, before hints, and every 30 seconds;
revoked, expired, or inactive sessions close fail-closed.

## Compiled game registry, versioned sessions, and commands

`crates/game-runtime` is a database-free boundary for trusted compiled game
definitions. A manifest uses one canonical key, a positive exact version, a
bounded control-free display name, and human-player limits within the global
eight-seat cap. Registry construction rejects invalid manifests and duplicate
`(key, version)` definitions and stores them in deterministic key/version
order. Production constructs a compiled registry containing immutable Signal
Siege v1 and v2; tests may inject fixture versions or an empty registry to prove
retained history and replay. The public `GET /v1/games` route always projects
that compiled metadata and, when the optional provider runtime is enabled,
merges only active provider-pilot manifests. Every record identifies its
`platform_compiled` or `registered_provider` authority and optional pinned
provider release.

A definition receives only the human-player count when initializing and only
the current object state, actor seat, and object command when transitioning. It
has no pool, network, clock, session, account, or ambient-randomness capability
in the interface. The registry resolves exactly the requested version, checks
that the count fits its manifest, and rejects initialization errors, non-object
JSON, or serialized initial state above 64 KiB. Command execution also rejects
state above 64 KiB, command input above 16 KiB, an out-of-range actor seat,
trusted-rules rejection, and non-object or over-64-KiB output through stable
typed errors. A successful definition returns a bounded next snapshot and a
closed `active|completed` lifecycle; identical initialization or command
inputs are expected to be deterministic.

`games::create_session` is the platform-compiled crate-private transaction primitive invoked by
challenge acceptance and the owner-scoped solo-start transaction. It rejects
empty, duplicate, or over-eight participant sets before persistence;
initializes the exact rules version;
locks all existing persona roots in canonical UUID order; then stores the
immutable game key/version, revision-zero active object snapshot, and
caller-ordered seats. It appends one `game_session_changed` event for every
participant inside the same caller-owned transaction. Any error must be
propagated so state, seats, and invalidations roll back together.

`POST /v1/personas/{persona_id}/game-sessions` is the narrow public solo-start
policy. It accepts a UUID idempotency key and exact game key/version, then
authenticates and owner-scopes the persona before locking its root. A matching
durable receipt is returned before current registry or inventory admission, so
replay survives completion and registry removal; a changed identity conflicts.
New work must resolve to a manifest requiring exactly one human, and at most 25
receipt-backed solo sessions for that persona may remain active. Session,
seat-zero participant, receipt, and one minimal sync invalidation commit
together. The server inserts no bot account, persona, or participant.

Signal Siege v1 is the production one-human definition. Human and bot begin
with eight core and two energy, choose among strike, guard, and charge, and
resolve each round simultaneously. The bot chooses from pre-command durable
state only. Cross-field state validation rejects inconsistent round, phase,
combatant, last-round, and outcome shapes. Core destruction or round 12
produces an explicit bounded winner/reason/final-state outcome and the compiled
`completed` lifecycle.

Signal Siege v2 is the exact two-human challenge definition. It preserves v1
unchanged, admits exactly two participants, initializes seat zero as active,
and alternates one strike, guard, or charge command per turn. Guard persists as
a bounded block until the opponent's strike or that player acts again; charge
restores bounded energy. Cross-field validation binds turn parity, seats,
last-turn evidence, guard and damage relationships, active seat, outcome, and
lifecycle. Core destruction or turn 24 produces a bounded terminal outcome;
otherwise authority passes to the other seat.

Authenticated inventory and detail routes first validate the Bearer session,
then require the acting persona to belong to its derived account and to each
returned game session. Inventory defaults to 50, caps at 100, and orders newest
first. Responses expose the durable key, version, revision, active/completed
status, authority, optional provider release and availability, state or
authenticated provider view, optional allowlisted provider result, completion
time, timestamps, seats, and the existing public persona shape; they contain no
account ownership, provider endpoint, credential, grant, or private provider
rules state.
Foreign, malformed, and absent session IDs share the same not-found result.
Reads come directly from PostgreSQL, so a process registry that has gained,
lost, or replaced versions cannot silently reinterpret an old session.

`POST /v1/personas/{persona_id}/game-sessions/{game_session_id}/commands`
accepts a body-capped object containing a UUID idempotency key, nonnegative
expected revision, and object command. The handler returns only the game
session ID, committed revision, lifecycle, and state with
`Cache-Control: no-store`. Authentication owner-scopes the acting persona; the
transaction then locks the session only through that persona's participant
row, keeping malformed, absent, and non-participant sessions
indistinguishable.

While holding the session lock, the domain checks the session-wide replay
receipt before enforcing lifecycle or current revision. A receipt replays only
when its actor, expected revision, and PostgreSQL-`JSONB`-semantic command
match; it returns stored revision, lifecycle, and state without rerunning rules
or appending another event, including after the first application completed the
session. Any identity mismatch is an idempotency conflict. A new key requires
an active session at the current revision and the stored exact game version to
remain compiled. Success atomically writes the bounded state and lifecycle,
sets completion time when terminal, increments one revision, preserves a
monotonic `updated_at`, inserts the lifecycle-bearing receipt, and appends one
minimal invalidation per canonically ordered participant. Rules rejection,
malformed input, unavailable rules, completed lifecycle, revision conflict,
and later transaction failure leave state, receipt, and invalidations
unchanged.

## Registered-provider game flow

Ticket 019 adds one optional registered-provider path for the operator-pinned
Door Legends v1 release. An owner-scoped start first locks the persona root and
resolves a durable start receipt before current catalog admission. New work
persists a `registered_provider` session envelope with a null local state,
release pin, `provisioning` availability, seat-zero participant, start receipt,
and sync invalidation, then commits before sending the signed launch operation.
The platform therefore has a durable recovery root even if the first network
outcome is unknown, without storing a writable copy of provider rules state.

Provider command and reconcile routes authenticate the same owned participant
and dispatch through `ProviderBroker`; they never invoke `GameRegistry`. Each
operation retains one idempotency UUID and expected provider revision. The
broker issues a fresh short-lived pairwise grant, enforces registered endpoint,
TLS, key, scope, lifecycle, body, timeout, replay, quota, and lease policy, and
authenticates the exact response. The platform then conditionally projects only
the bounded Door Legends view, revision, lifecycle, and availability in a new
transaction. A timeout or outage marks an explicit recovery state, and an
explicit reconcile operation asks the provider for authoritative state. There
is no compiled fallback.

Provider callbacks first validate the fixed callback authority and path,
registered identity, pairwise subject, exact signed bytes, key, bounds, quota,
and current pilot lifecycle. The projection transaction locks release, pilot,
and session roots in one order, claims the durable callback receipt, validates
event revision and platform-pinned result/achievement policy, and commits the
view, allowlisted result or awards, audit, and persona-sync invalidation
together. Invalid signatures consume no authenticated callback quota;
authenticated events outside policy are durably ignored and audited without a
platform effect; exact replays are effect-free.

Suspension removes the pilot from the catalog, denies launches, commands, and
callbacks, preserves participant-private reads, and allows explicit
reconciliation to establish the authoritative remote state. Reactivation
keeps sessions non-ready until that reconciliation succeeds. Retirement is
terminal. The provider database remains independent of the OmarchyGS database;
the tested recovery procedure backs it up and restores it separately.

## Player reporting and operator containment

`POST /v1/personas/{persona_id}/reports` authenticates the device session and
owner-scopes the reporter before disclosing target validity. The report domain
locks that persona, resolves an exact UUID replay before current admission,
rejects self-reporting, accepts only `harassment`, `spam`, `cheating`, or
`other`, bounds trimmed control-safe detail to 1–1,000 characters, and admits
at most 25 open reports per reporter. The player receives only the report ID,
idempotency key, immutable creation status, and timestamp; the route emits no
subject notification or persona synchronization event.

The QML Social screen resolves the existing public exact-handle resource and
submits through the bearer-owning onboarding request gateway. The social
controller never receives the raw token, preserves one operation UUID only for
the same uncertain submission, rejects extra or malformed receipt fields, and
clears the report form only after an exact success. Fixed guidance and player
content remain plain text, and the report controls participate in the existing
keyboard, accessibility, and 640×420 containment checks.

`omarchygs-admin` is a separate PostgreSQL-local executable, not an Axum route,
account role, administrator token, or listener. It reads only `DATABASE_URL`,
lists a filtered newest-first queue of at most 100 reports or invitations, or
applies one bounded non-symlink regular JSON command file. Account actions permit only
`active` ↔ `suspended`; suspension revokes every live device session in the
same transaction, reactivation never clears `revoked_at`, and `disabled`
remains outside this reversible command. Report actions permit one `open` →
`resolved` or `dismissed` transition. Both target roots are locked, exact
operation retries return the original receipt, conflicting retries or terminal
state changes fail, and the state transition plus audit append commit together.

Invitation actions issue 1–720 hour bearer codes, revoke only a live unused
invitation, and cap the community at 500 simultaneously live invitations. Issue
is serialized and idempotent by operation UUID: exactly one first delivery
contains the raw code, while replays and inventory expose only bounded metadata
and lifecycle state. Revocation is also exact-replay idempotent, and used,
expired, or already-revoked invitations cannot move into another terminal
state.

The operator audit records only the bounded actor/reason, target, action,
previous/resulting state, operation UUID, and timestamp—not report detail or
credentials. PostgreSQL rejects audit update/deletion and report deletion. The
operator guide owns database credential, report privacy, MFA-key custody,
backup, isolated restore, and rollback guidance; remote administrator accounts,
roles, appeals, evidence attachments, content deletion, and scheduled backup
infrastructure remain out of scope.

## Identity-ready schema

The forward-only `0001_identity_foundation.sql` migration creates three
separate persistence roots:

- `accounts` stores a case-insensitively unique username, password hash,
  lifecycle status, and timestamps;
- `account_sessions` belongs to an account and stores only a unique token hash,
  expiry, last-used, revocation, and creation timestamps;
- `personas` belongs to an account and stores a case-insensitively unique handle
  plus bounded public profile fields.

The forward-only `0002_canonical_account_usernames.sql` migration additionally
enforces the account namespace for every database writer. Migration `0003`
adds bounded device names and requires 32-byte stored token digests. Migration
`0004` enforces the same canonical persona-handle namespace for every database
writer, and the persona endpoints implement the existing profile tables.
Migration `0005` adds one TOTP authenticator per account, hashed single-use
recovery codes, and hashed expiring login challenges. Its constraints bound
ciphertext and nonce lengths, digest sizes, device names, failure counts, and
challenge expiry. Migration `0006` adds the canonical pending/accepted
connection pair and directional block tables, with foreign keys, direction and
status checks, and indexes for both request directions, both connection sides,
and reverse block checks. Migration `0007` adds one canonical-pair conversation,
per-participant read positions, exact user/system message constraints, and an
accepted-pair backfill. Forward migration `0008` converts the initial global
identity values and existing read/latest positions into conversation-local
sequences, then enforces unique `(conversation_id, message_sequence)` values
and a matching composite latest-message foreign key. Migration `0009` adds one
cursor state row per persona plus bounded retained synchronization events and
indexes for ordered recovery and pruning. Migration `0010` adds exact-version
game sessions, revision-zero object snapshots, unique ordered persona seats,
and the shaped `game_session_changed` UUID payload in retained persona events.
Migration `0011` adds session-wide command receipts, binds each actor to a
durable participant, allows one receipt per applied revision, requires the
applied revision to be exactly one beyond its nonnegative expected revision,
and enforces object command and state shapes. The database enforces object
shape and canonical identity; the compiled runtime applies the serialized
state and command bounds before persistence. Migration `0012` adds durable
two-person challenges with immutable exact-game identity, challenger-scoped
idempotency, one equivalent pending request, participant-history indexes, and
status/session/resolution constraints. It also extends inbox messages with
typed challenge/session references and retained persona events with an exact
payload-minimal challenge variant. Migration `0013` adds the consistent
active/completed session and timestamp shape, stores the applied lifecycle in
every command receipt, and adds persona/idempotency/session-linked solo-start
receipts with exact game identity and participant integrity. Migration `0014`
adds the registered-provider security/control-plane foundation. Migration
`0015` adds the session authority discriminator and exclusive state shape,
provider release and availability, the singleton pilot, authenticated bounded
views, immutable result and achievement projections, and terminal lifecycle
guards. Migration `0016` adds retained persona reports with reporter-scoped
idempotency, fixed category/status and terminal-time constraints, plus
exact-target operator audit with target-scoped operation uniqueness. Database
triggers make audit append-only and deny report deletion. Migration `0017`
adds digest-only registration invitations with exact expiry, use, revocation,
and single-terminal-state constraints and extends the immutable operator audit
target/action contract to invitation issue and revocation. Migration `0018`
adds one randomly generated singleton server UUID and immutable row/table
triggers so ordinary database backup and restore preserve community continuity.
Add later
capabilities through domain modules and thin handlers rather than placing policy
directly in SQL or transport code.

## Safe change path

Start with the owning domain behavior and tests, then update thin routes and
persistence. Never rewrite a migration after it may have run; add a numbered
forward migration. Validate database-sensitive behavior against PostgreSQL and
finish with `bin/gate.sh --diff`. Registration's narrow proof is the account
and invitation unit tests plus three ignored SQLx router scenarios for first
use/replay, unavailable-state equivalence/conflict rollback, and concurrent
consumption. The full admission proof is `scripts/test-private-alpha.sh`.
Session changes additionally use the dedicated multi-account lifecycle tests.
Persona changes use `persona_api_tests.rs`, whose migrated router tests cover
multiple owners, exact public field sets, foreign-object denial, public lookup,
handle movement and conflicts, input allowlists, and storage preservation.
MFA changes use `mfa_api_tests.rs`, whose migrated router tests cover encrypted
enrollment, status privacy, TOTP/recovery/challenge replay, independent bounded
challenge issuance, cross-challenge attempt locking, dual-proof disablement,
and restoration of password-only login. Connection changes use
`connection_api_tests.rs`: its five migrated multi-account cases cover
direction and response privacy, race-safe request caps, participant-only
acceptance/removal, atomic block behavior under a request race, and
serialization of opposite requests and concurrent acceptance. Inbox changes
use `inbox_api_tests.rs`: its five migrated cases cover transition-only
conversation creation, typed body-only messages, private monotonic unread
state, conversation-local ordering, bounded durable history, lifecycle send
denial, no-store failures, and concurrent send/read behavior. Synchronization
changes use `sync_api_tests.rs`: its migrated cases cover baseline and
incremental recovery, retention resets, owner privacy, transaction-coupled
invalidations, real TCP WebSocket authentication and hints, principal-scoped
quotas, frame bounds, permit release, and session revocation, expiry,
inactivity, and no-touch revalidation.
Report changes use `report_api_tests.rs`: its migrated cases cover exact
receipts, no-store success and errors, authentication precedence, owner scope,
self/absent targets, input bounds, open-cap and concurrent-first-write
serialization, idempotency collision, and immutable replay after disposition.
Operator-state changes use the focused database tests in `operator_admin.rs`
plus the real executable test under `crates/server/tests`; together they cover
inventory privacy, bounded file input, exact replay, conflicting and concurrent
decisions, account suspension/reactivation, session containment, terminal
report disposition, and append-only linked audit. Platform restore changes
must also pass `scripts/test-operator-recovery.sh`.
Game-runtime changes use its five unit tests for manifest validation, stable
exact-version lookup, and bounded deterministic initialization and commands.
Signal Siege changes use its ten v1/v2 rule tests plus
`signal_siege_api_tests.rs`: one local catalog/body-limit case and four migrated
PostgreSQL cases cover owner scope, exact start replay, registry drift,
active-cap concurrency, completion/final replay/history, no bot identity,
privacy-minimal sync, and rollback. The challenge suite adds a real v2
alternation and terminal-outcome case. General game transport or persistence
changes also use `game_api_tests.rs`: two local router cases and five migrated
PostgreSQL cases cover stable catalog projection,
body bounds, atomic creation and command transitions, ordered seats, semantic
replay and isolated collision axes, revision conflicts, rollback silence,
minimal per-participant sync, bounded private reads, indistinguishable foreign
and absent objects, registry-independent stored history, and one-winner command
concurrency.
Challenge changes use `challenge_api_tests.rs`: one local body-limit case and
seven migrated PostgreSQL cases cover participant privacy, exact creation replay
and collisions, typed inbox and minimal sync payloads, exact-version acceptance
and seat order, terminal history and lazy expiry, pending limits, initializer
and block rollback, production Signal Siege v2 alternation/completion, and
one-winner terminal races. QML client changes first prove two public profiles
survive separate writer and reader processes, then run through the 44-case
fixture corpus and four live scenarios in `scripts/dev.sh`; those
prove contrast, semantic headings and status, deterministic focus, reversible
Tab traversal, Escape authority, minimum-width containment, strict hostile-
envelope rejection, report retry and receipt handling, retained game retry
identity, revision refetch, authority cleanup, active-seat enforcement,
terminal completion, and fresh-controller recovery.
Server discovery or identity changes additionally use
`server_discovery_api_tests.rs`, the configuration unit tests, the focused QML
compatibility/identity fixtures, and `scripts/test-operator-recovery.sh` for
source-versus-restored UUID equality.
Provider player-route changes use `provider_game_api_tests.rs` plus
`scripts/test-provider-authority-pilot.sh`. The clean-clone proof covers the
independent TLS process and database, protocol-only dependency, mixed catalog,
exact start and command replay, expected-revision races, unknown-outcome
reconciliation, callback authentication/deduplication/policy, participant
privacy, lifecycle containment, restart, and separate provider backup/restore.
