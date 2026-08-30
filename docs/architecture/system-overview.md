# System overview

Omarchy Gaming System is a modular Rust monolith backed by PostgreSQL. The QML
connector is the flagship client but uses the same versioned public API as any
future terminal, web, mobile, or embedded connector.

The long-term deployment unit is an independently owner-operated community.
An individual or group runs the standard OmarchyGS server, curates its game
catalog, and invites players into server-local accounts, personas, social
state, and history. Multiple processes behind one origin may implement one
logical deployment; different server origins are separate trust and identity
domains unless a future federation design says otherwise.

```text
QML connector
  ├─ HTTPS/JSON commands and queries
  └─ WebSocket notifications
             ↓
Rust application
  ├─ auth and accounts
  ├─ personas and social connections
  ├─ conversations and notifications
  ├─ reports and operator containment
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
- Portable frontend, game-backend, and general server-extension contracts are
  distinct. Cartridges are inert data rendered by trusted QML; portable game
  rules use the brokered provider protocol; server modules use the
  ADR-0004 process-isolated no-WASI Component Model host, exact WIT contracts,
  and capability-scoped typed hooks/intents. Core reauthorizes every proposal;
  modules cannot shortcut domain authorization or become gameplay authority.
- The production module base uses all-or-none runtime keys. It can register the
  compiled-in reviewed Sentinel release and can dispatch up to eight privately
  admitted, server-bound operator-custom module identities. Both durably
  observe minimized report events and may propose only a core-reauthorized
  `priority_review` label. It exposes no public mutation route, configurable
  host/artifact path, arbitrary egress, client code, or gameplay authority.
- A server operator may admit explicitly operator-custom content and may later
  admit marketplace-vetted modules after that separate gate. Provenance and
  support expectations differ, but the
  official client never accepts raw server-supplied QML, JavaScript, native
  code, credentials, or arbitrary network destinations from either class.
- The player-device deployment unit is a native Arch package containing only
  the exact platform-owned QML client, command launcher, desktop entry, and
  non-secret build provenance. It consumes Omarchy's system Qt runtime and is
  separate from community-server, provider, and cartridge deployment units.

## Current slices

The executable connects to PostgreSQL, applies embedded migrations, and exposes
the first identity HTTP surfaces:

- `GET /health` proves database readiness; the QML connector consumes its exact
  identity before enabling account access and distinguishes connecting,
  ready, offline, configuration-error, and protocol-error states.
- `POST /v1/accounts` requires an operator-issued 256-bit invitation, then
  delegates to the account domain, which canonicalizes the private username,
  bounds the password, rejects unavailable invitation digests before Argon2id
  work, and atomically inserts the salted-Argon2id account plus one invitation
  consumption. A credential-proven exact retry returns the immutable public
  receipt; changed intent receives the uniform invitation denial.
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
- authenticated `POST /v1/personas/{persona_id}/reports` owner-scopes the
  reporter, accepts one bounded persona target/category/detail under a UUID,
  caps each reporter at 25 open reports, and returns only an exact private
  receipt. Reports do not generate sync hints or notify the subject.
- when the exact first-party report module is explicitly enabled, the same
  report transaction appends a privacy-minimized durable observation. A bounded
  dispatcher executes one no-WASI component in a fresh OS-contained process,
  then core independently reauthorizes and receipts its single typed label
  proposal. Host faults retry and dead-letter without rolling back committed
  reports. An inactive module or saturated queue never rejects a core report;
  the same transaction records an aggregate reason, count, and timestamp
  without growing the queue.
- the separate `omarchygs-admin` process is a PostgreSQL-local operator
  adapter, not an HTTP route or reusable administrator credential. It lists a
  bounded report queue and invitation inventory, issues digest-only expiring
  one-account codes, revokes unused codes, and applies only reversible account
  suspension/reactivation or terminal report disposition. Target locks,
  operation UUIDs, same-transaction state changes, and insert-only audit
  serialize every action. Raw invitations appear only in the first issue
  receipt; reactivation cannot resurrect old tokens, and the stronger
  `disabled` state remains outside this command.
- the keyboard-first QML access shell composes those unchanged REST endpoints
  through one finite connection/access/MFA/persona/home state machine. Its API
  object serializes one bounded request generation, rejects stale completions,
  validates exact response shapes, and is the only client object that retains
  a raw bearer. Invitation, password, and factor fields are masked and clear
  on submission or terminal form transitions; bearer and MFA challenge values
  remain in process memory only and are cleared with every
  terminal authority transition. Remote endpoints require HTTPS while
  loopback HTTP remains the explicit development exception. The shell and all
  ten routed screens share repository-owned semantic theme, heading, status,
  card, section, and control primitives. Their state remains understandable
  without color alone; visible focus, accessible names, deterministic initial
  focus, reversible traversal, Enter/Escape behavior, explicit plain text, and
  scrollable minimum-size layouts provide keyboard-only play at 640×420 and
  above without changing controller authority. One persistent shell-owned
  EXIT control requests the normal application-window close lifecycle on every
  route; it does not log out, revoke the durable device session, or dispatch
  API, social, game, or cartridge actions.
- the private-alpha Omarchy package installs that same production QML root
  under `/usr/share/omarchy-gaming-system/qml`, exposes `/usr/bin/omarchygs`,
  and registers a non-terminal Game application entry. A fail-closed manifest
  binds the complete non-test runtime tree, and the package gate proves two
  identical Arch builds, exact metadata/payload/modes/provenance, hostile
  source rejection, and an offscreen launch from the extracted artifact. The
  package carries no server code or credential; public repository/signing and
  persistent OS-keyring login remain separate future boundaries.
- a dedicated QML social controller uses that same credential-owning API
  object only through a session-gated request function and completion signal;
  it never receives the bearer. Every connection, block, conversation,
  history, send, read, and report path derives its actor from the currently
  selected owned persona. Exact schema allowlists reject partial, extra,
  unknown, or oversized social/inbox/report responses while plain-text
  presentation keeps peer, report, and system content out of the QML rich-text
  boundary.
- social and inbox screens refresh authoritative REST state on entry or
  explicit player action. They expose exact-handle connection requests,
  accept/decline/cancel/remove and private block lifecycle, bounded
  conversations, ascending older-page recovery, body-only private sends, and
  monotonic read acknowledgements, plus a bounded exact-handle report form that
  clears player text only after an exact successful receipt. They deliberately
  do not start polling or
  subscribe to `/sync/live`; concurrent live-hint lifetime and recovery remain
  a separately reviewed client transport slice.
- a dedicated QML game controller follows the same bearer-owning session
  boundary and derives every persona, challenge target, session revision, and
  command from validated client authority rather than accepting raw route
  input from a screen. Games, challenges, and gameplay use explicit REST
  refresh with strict catalog/challenge/session/state allowlists. Transport-
  uncertain mutations retain their exact idempotency identity for an explicit
  retry, while revision conflicts refetch the participant-authorized session.
  The platform-owned Signal Siege presenter receives only a derived view model
  and can emit only `strike`, `guard`, or `charge`; it does not claim signed
  Game Cartridge provenance or execute publisher QML/JavaScript.
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
  receives only the pre-command game state, and immutable Signal Siege v2 for
  exactly two humans with alternating visible turns. Test routers can inject
  additional deterministic fixture definitions.
- public `GET /v1/games` inventories stable compiled manifest metadata and,
  only when the optional provider runtime is fully configured and its exact
  first-party release is active, the operator-pinned Door Legends manifest.
  Durable
  game sessions pin one game key/version, revision-zero object snapshot,
  active/completed status, completion timestamp, and ordered human persona
  seats. An owner-scoped public start route creates one-human sessions through
  an idempotent durable receipt and caps each persona at 25 active solo starts;
  a persona row lock serializes both duplicate and final-capacity races.
  Challenge acceptance invokes the same crate-private creation primitive for
  two-human games. Participant-owned routes expose bounded inventory/detail.
- authenticated `GET /v1/cartridges` inventories only effective exact
  marketplace-vetted presentation releases selected by this server. A pinned
  signed marketplace snapshot and verified immutable bytes feed PostgreSQL
  reviewed inventory; an independent expected-state operator transaction owns
  activation, deactivation, upgrade, rollback, revision, and immutable audit.
  Lifecycle suspension, removal, or incompatibility fails closed without
  choosing another version. When the optional distribution runtime has its
  exact manual key or root-authenticated trust bundle plus secure-store root,
  discovery advertises the authenticated
  current and historical exact acquisition routes plus session-presentation
  capabilities. The current route requires today's selected release; the
  participant route resolves only the immutable session pin through normalized
  retained signed snapshot/release evidence. Both apply current lifecycle
  authority and self-verify the canonical response.
  The packaged loopback Rust companion requires a locally provisioned manual
  marketplace key or explicitly enrolled offline-root channel, rejects any
  server-envelope trust substitution, and independently verifies initial/final
  catalog admission plus historical/current-policy marketplace keys and every
  publisher, lifecycle, SDK, and byte claim before
  atomically staging shared content and an exact server-UUID-scoped read-only mount.
  Each profile can retain up to 128 game/digest/admission-revision mounts, each
  with exact evidence/policy key fingerprints and snapshot versions. The QML Games screen exposes
  explicit install, update, and local removal. Eligible new compiled or
  registered-provider sessions pin one exact current release and admission
  revision in an immutable presentation row. The native companion resolves only
  the matching canonical origin/UUID mount under client-controlled trust. A
  missing mount is explicit and can be installed from the immutable pin only by
  a participant; the companion checks the session before and after acquisition.
  It compiles the requested signed screen into an inert plan and exposes bounded
  digest assets through an ephemeral loopback capability. Trusted QML validates
  exact screen/navigation metadata, keeps bounded Back/Entry history locally,
  and never sends reserved navigation actions as gameplay. Declared gameplay
  actions include the accepted screen and return to the server for participant
  authorization, signed current-screen validation, durable admission, and
  dispatch through the session's existing sole gameplay authority.
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
  Signal Siege v2 instead alternates seat-scoped human actions, terminates by
  core destruction or turn 24, and records a seat-named terminal outcome while
  retaining v1 semantics unchanged.

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

Each database now also owns exactly one random immutable server UUID. The
public discovery endpoint combines it with the operator-configured display
name and a deterministic protocol/capability document. The flagship client
stores only that public metadata for multiple canonical origins, clears all
live authority before an origin change, and pins remembered origins to their
UUID. These profiles are isolated server choices, not federation or shared
identity.

## Portable game direction

The accepted [OmarchyGS Game Cartridge](game-cartridges.md) model lets
independently versioned games ship a signed declarative presentation package
rendered by trusted OmarchyGS QML components. The v1 verifier, conformance SDK,
descriptor-relative store, render-plan compiler, first-party separate-repository
proof, guarded marketplace sync, reviewed server inventory, local catalog
control, metadata-only player catalog, independently trusted client
acquisition/multi-release mounting, immutable session presentation pins,
historical pin recovery, trusted multi-screen launch, and screen-bound action
admission exist. Offline-root trust enrollment, monotonic key
rotation/revocation, dual-key acquisition v2, and authenticated native-package
staging also exist without granting installer authority. A non-SDK publication
tool now composes those contracts into deterministic immutable channel and
marketplace trees through separate online catalog-signing and network-less
offline-root steps. One atomic local pointer selects a fully verified version;
guarded probes authenticate exact bytes across operator-supplied mirrors
without adding client fallback or another authority. Official hosting, human
root custody, and in-band root replacement remain deployment work outside the
repository. Signal Siege rules remain compiled into the trusted server for the
private alpha. The registered Door Legends pilot is the first portable
playable and owns server-side gameplay only through the separately authenticated
broker boundary. Raw
third-party QML, JavaScript, native plugins, device tokens, account identity,
and direct database access remain outside the boundary.

[ADR-0003](adr-0003-owner-operated-server-and-extension-boundary.md) extends
that direction to owner-operated communities. A vetted marketplace can publish
exact signed releases, and each server operator now chooses what to synchronize
and independently admit. Players can acquire/cache current selected cartridges
or explicitly recover an exact old session pin for trusted mounted rendering. A
public-only Provider SDK preview now packages the brokered model, authenticated
exact-v1 compatibility, signing/grant/message helpers, and reproducible release
contract without platform implementation or admission authority. Its starter,
portable conformance kit, second clean-room backend, and sidecar/operations
profile remain future slices. The separate
module base now supplies its first observation-only typed hook through one
shared process-isolated runtime for reviewed and operator-custom provenance.
Operator-custom server behavior is visibly distinct, remains server-bound and
unsupported, and cannot weaken any official-client cartridge bound.
