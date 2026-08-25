# System overview

Omarchy Gaming System is a modular Rust monolith backed by PostgreSQL. The QML
connector is the flagship client but uses the same versioned public API as any
future terminal, web, mobile, or embedded connector.

```text
QML connector
  ├─ HTTPS/JSON commands and queries
  └─ WebSocket notifications
             ↓
Rust application
  ├─ auth and accounts
  ├─ personas and social connections
  ├─ conversations and notifications
  └─ server-authoritative game runtime
             ↓
PostgreSQL
  ├─ durable domain state
  ├─ game events and snapshots
  └─ synchronization cursors
```

## Boundaries

- HTTP and WebSocket handlers translate transport data and call domain code.
- Domain modules own authorization and invariants.
- Game modules accept validated commands and return deterministic transitions;
  they do not query the database directly.
- Transactions append events, update snapshots/revisions, and create durable
  notifications atomically.
- WebSockets signal that data changed. A cursor API repairs missed events after
  reconnect and remains the synchronization source of truth.

## Current slices

The executable connects to PostgreSQL, applies embedded migrations, and exposes
the first identity HTTP surfaces:

- `GET /health` proves database readiness; the QML connector consumes it and
  displays a connected, offline, or protocol-error state.
- `POST /v1/accounts` delegates to the account domain, which canonicalizes the
  private account username, bounds the password, hashes it with salted Argon2id
  off the async executor, and relies on PostgreSQL for unique insertion.
- `POST /v1/sessions` verifies account credentials with bounded Argon2id work
  and either issues an opaque device Bearer token or, for an MFA-enabled
  account, creates one of at most ten independent five-minute digest-only
  challenges without a session. The cap returns HTTP 429 without invalidating
  another device's live challenge.
- authenticated `/v1/account/mfa` enrollment, status, confirmation, and
  disablement routes keep TOTP settings behind the private account boundary.
  TOTP secrets are random 160-bit values encrypted with AES-256-GCM and account
  UUID associated data under the operator's `OGS_MFA_ENCRYPTION_KEY`. Ten
  120-bit recovery codes are returned once and only their SHA-256 digests are
  stored.
- `POST /v1/sessions/mfa` consumes the login challenge and either a current
  unused RFC 6238 code or unused recovery code in the same transaction that
  creates the device session. Authenticator-wide failure state prevents new
  challenges from resetting the five-attempt, five-minute throttle.
- authenticated `GET /v1/sessions` and `DELETE /v1/sessions/{session_id}` use
  PostgreSQL to enforce active account status, idle/absolute timeout,
  account-scoped inventory, last use, and immediate revocation.
- authenticated `POST /v1/personas`, `GET /v1/personas`, and
  `PATCH /v1/personas/{persona_id}` derive the private account principal from a
  validated session. The domain owns canonical profile validation and uses
  account-scoped SQL for inventory and mutation.
- public `GET /v1/personas/by-handle/{handle}` performs exact canonical handle
  lookup and returns only the explicit public profile fields.
- authenticated persona-scoped connection routes create and inventory pending
  requests, let only the addressee accept, list the resulting mutual
  connection, and let either participant idempotently remove pending or
  accepted state. They embed only the public persona profile. Mutation-time
  limits cap each persona at 100 incoming and 100 outgoing pending requests,
  with the existing ordered persona locks making boundary races deterministic.
- authenticated directional block routes keep block inventories private. A
  block and relationship mutation lock both persona rows in UUID order; the
  block is inserted and any pending or accepted relationship is deleted in one
  transaction. Requests in either direction then receive the same
  non-disclosing error until the blocker explicitly unblocks. Direct block
  state remains private, while interaction outcomes are explicitly not treated
  as protection against every indirect block-direction inference.
- authenticated inbox routes expose one durable conversation per canonical
  persona pair, bounded ascending history, tagged user/system messages, and a
  private monotonic read cursor for each participant. Message sequences are
  local to the conversation, so unrelated private activity cannot be inferred
  from gaps. The actual connection
  acceptance transition appends a server-authored `connection_accepted`
  message in the same transaction; retries do not duplicate it. Sending locks
  the persona pair before the conversation and requires a currently accepted,
  unblocked connection, while history remains readable after removal or block.
- authenticated persona sync routes expose an independent durable invalidation
  cursor for each owned persona. Mutations append exact social or conversation
  event types and prune beyond 10,000 retained rows inside the same PostgreSQL
  transaction as domain state. An expired cursor explicitly requires a fresh
  baseline and REST snapshot instead of silently skipping changes.
- the header-authenticated `/sync/live` WebSocket sends only a ready cursor and
  change/recovery hints. PostgreSQL `LISTEN/NOTIFY` publishes hints after commit
  across server instances; a bounded process-local hub fans them out and tells
  lagging sockets to resynchronize through REST. Decoder limits reject client
  payloads above 1 KiB before application allocation, admission is bounded per
  persona, account, and process, and no-touch authority checks close sockets
  after session revocation/expiry or account deactivation.
- a database-free compiled game runtime validates canonical, versioned public
  manifests and resolves only exact rule versions. The production registry
  contains Signal Siege v1, a deterministic one-human duel whose bot policy
  receives only the pre-command game state. Test routers can inject additional
  deterministic fixture definitions.
- public `GET /v1/games` inventories only compiled manifest metadata. Durable
  game sessions pin one game key/version, revision-zero object snapshot,
  active/completed status, completion timestamp, and ordered human persona
  seats. An owner-scoped public start route creates one-human sessions through
  an idempotent durable receipt and caps each persona at 25 active solo starts;
  a persona row lock serializes both duplicate and final-capacity races.
  Challenge acceptance invokes the same crate-private creation primitive for
  two-human games. Participant-owned routes expose bounded inventory/detail.
- durable two-person challenges pin one exact game key/version between a
  connected, unblocked, different-account persona pair. A challenger-scoped
  UUID makes creation retry-safe, a partial uniqueness constraint prevents an
  equivalent pending request, and canonical persona-root locks serialize the
  100-incoming/100-outgoing limits and every lifecycle race.
- challenge state is monotonic from `pending` to `accepted`, `declined`,
  `cancelled`, or `expired`. The server owns a seven-day expiry and resolves it
  under persona locks on reads and mutations. Terminal history is retained;
  accepted rows alone link one exact session.
- challenge creation and first terminal transitions append one typed message
  to the pair's durable conversation plus minimal challenge/conversation sync
  invalidations for both personas. Acceptance calls the session primitive in
  the same transaction, so the exact initial snapshot, ordered seats, accepted
  link, inbox record, and all invalidations either commit together or not at
  all. WebSocket hints carry none of those resource details.
- creating a session appends a minimal `game_session_changed` invalidation for
  every participant in the same transaction. Reads use the stored game version
  and state directly, so a changed process registry cannot silently relabel an
  old session and sync/WebSocket payloads never carry the game snapshot.
- participant command POSTs lock the durable session and check a session-wide
  UUID receipt before optimistic revision enforcement. Matching retries return
  the committed receipt, including the terminal receipt after completion or
  compiled-registry drift; collisions and stale/future revisions change
  nothing. New commands cannot mutate a completed session.
  A first-use command executes only the stored exact compiled rules version
  with bounded object state, actor seat, and bounded object command. Snapshot,
  one-step revision, status/completion timestamp, receipt, and one minimal
  invalidation per participant commit atomically. Compiled rules receive no
  database, network, clock, account/session identity, or ambient randomness.
  Signal Siege resolves each human action and deterministic bot response in
  that one transition, terminates by core destruction or round 12, and stores
  the explicit winner/draw outcome without creating a bot identity row.

Registration returns only the new account ID and canonical username. It does
not authenticate the caller or create a public persona. Session responses never
expose account ownership or token digests, and raw tokens are returned only at
creation. Persona responses are built from a public model that does not contain
`account_id` or authentication material. Accounts may own multiple personas;
public handle enumeration never reveals that ownership relationship.

Social, inbox, challenge, and game-session queries identify the acting persona
in the path, but it is always constrained to the account derived from the
Bearer session. A canonical
unordered pair owns at most one pending or accepted relationship row, while
blocks remain directional and one conversation persists independently of live
relationship state. Same-account personas cannot manufacture social edges.
Idempotent `PUT` and `DELETE` commands are safe for client retry. The inbox
persists typed local messages and read state, while the persona sync feed makes
those and social changes reconnect-safe without exposing private resource data
on WebSockets. The same feed identifies changed game sessions by UUID, while
the participant-authorized REST resource remains the only state source.
Challenge events likewise carry only the participant-authorized challenge
UUID; challenge REST is the status source. Game command POSTs use explicit
session-wide idempotency keys and expected revisions; WebSockets never accept
durable game mutations.

Credential hashing and verification share a four-job limit so memory-hard work
cannot grow with request concurrency. This is not distributed login throttling,
and the local HTTP server does not establish production TLS; both controls are
required before public deployment.

TOTP uses six HMAC-SHA-1 digits every 30 seconds, accepts at most one step of
past/future clock drift, and stores the last accepted step to reject replay.
Enrollment remains pending until a code confirms it, and security-state
disablement requires a valid Bearer session, the account password, and an
unused factor. TOTP is not phishing-resistant; a future passkey/WebAuthn slice
would provide a stronger verifier-bound authenticator.

## Product identity and transition

The product is game-first and uses **OmarchyGS** as its human shorthand:
connections, private inboxes, challenges, and server-authoritative matches
define the first playable. A public message board may become a complementary
community feature later, but it is not the current identity or private-alpha
focus.

Current runtime identifiers use `omarchy-gaming-system`, the `ogs` shell and
configuration namespace, and the `ogs1_` opaque-session prefix. During the
local pre-alpha transition, the server continues to accept structurally valid
`bbs1_` session tokens and uses `BBS_BIND_ADDRESS` only when
`OGS_BIND_ADDRESS` is absent. PostgreSQL stores token hashes rather than token
prefixes, so legacy sessions require no schema migration and still follow the
same expiry and revocation rules. Forward-only migrations and completed
planning records retain their historical names.

## Portable game direction

The accepted [OmarchyGS Game Cartridge](game-cartridges.md) model lets
independently versioned games ship a signed declarative presentation package
rendered by trusted OmarchyGS QML components. The v1 verifier, conformance SDK,
same-user store, render-plan compiler, and first-party separate-repository proof
exist; Signal Siege rules remain compiled into the trusted server for the
private alpha. A future registered provider may own server-side gameplay only
after the broker, operational controls, ADR, and Constitution amendment defined
by that architecture are approved. Raw third-party QML, JavaScript, native
plugins, device tokens, account identity, and direct database access remain
outside the boundary.
