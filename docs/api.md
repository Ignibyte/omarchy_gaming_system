# HTTP API

Omarchy Gaming System uses versioned JSON endpoints for durable commands and queries.
The server remains authoritative for validation and persistence.

## Register an account

`POST /v1/accounts`

Request:

```json
{
  "username": "Player_One",
  "password": "a-long-private-passphrase"
}
```

Usernames are trimmed and converted to ASCII lowercase. The canonical value
must be 3–32 bytes, begin with an ASCII letter or digit, and contain only ASCII
letters, digits, underscores, or hyphens. Passwords are not trimmed and must be
12–128 bytes. Registration request bodies are limited to 1 KiB.

A successful request returns `201 Created`:

```json
{
  "id": "58ee076d-0216-422c-b1e2-48ee7fa648bb",
  "username": "player_one"
}
```

The response never contains the password or its hash. Passwords are stored as
uniquely salted Argon2id v19 PHC strings with `m=19456`, `t=2`, and `p=1`.

Validation failures return `422 Unprocessable Entity`, and canonical username
conflicts return `409 Conflict`. Errors use a stable envelope:

```json
{
  "error": {
    "code": "username_taken",
    "message": "username is already registered"
  }
}
```

Current registration error codes are `invalid_username`, `invalid_password`,
`username_taken`, and `internal_error`.

## Create a device session

`POST /v1/sessions`

Request:

```json
{
  "username": "player_one",
  "password": "a-long-private-passphrase",
  "device_name": "Omarchy laptop"
}
```

Device names are trimmed and must contain 1–64 non-control characters. A
successful request returns `201 Created`, includes `Cache-Control: no-store`,
and returns the opaque Bearer token exactly once:

```json
{
  "token": "ogs1_<base64url-encoded-random-value>",
  "session": {
    "id": "6de4bfbd-c15b-4aa8-ae4e-599b430f9b32",
    "device_name": "Omarchy laptop",
    "created_at": "2026-08-24T14:00:00.000Z",
    "last_used_at": "2026-08-24T14:00:00.000Z",
    "expires_at": "2026-09-23T14:00:00.000Z",
    "revoked_at": null,
    "current": true
  }
}
```

Tokens contain 256 OS-random bits after the `ogs1_` prefix. PostgreSQL stores
only the 32-byte SHA-256 digest; the raw token and digest are never returned
together or logged. Unknown usernames, wrong passwords, and inactive accounts
all return the same `invalid_credentials` response with HTTP 401.

When TOTP MFA is enabled, correct primary credentials return `202 Accepted`
under `Cache-Control: no-store` and create no device session yet:

```json
{
  "mfa_required": true,
  "challenge_token": "ogm1_<base64url-encoded-random-value>",
  "expires_at": "2026-08-24T14:05:00.000Z"
}
```

The opaque challenge contains 256 random bits and expires after five minutes.
PostgreSQL stores only its SHA-256 digest. An account may have at most ten live
challenges so overlapping device logins remain independent and bounded; an
additional password login returns HTTP 429 `mfa_rate_limited` without
invalidating any live challenge. Complete a challenge through
`POST /v1/sessions/mfa` as documented below.

Tokens issued before the gaming-system rebrand may use the legacy `bbs1_`
prefix. They remain accepted until their ordinary expiry or revocation, but
new sessions always receive `ogs1_` tokens.

## List device sessions

`GET /v1/sessions`

Send the token only in the authorization header:

```text
Authorization: Bearer ogs1_<token>
```

The response contains every session owned by the authenticated account and
marks the presented session as `current`. It never contains `account_id`, raw
tokens, or token digests, and it carries `Cache-Control: no-store`. Successful
authentication advances `last_used_at`.

Sessions become invalid after seven idle days, 30 absolute days, explicit
revocation, or account suspension/disablement. Those rules are enforced by the
server on each authenticated request. Established live-sync sockets recheck
the same authority without advancing `last_used_at`, so leaving a socket open
does not extend the session's idle lifetime.

## Revoke a device session

`DELETE /v1/sessions/{session_id}`

Use a valid Bearer token belonging to the same account. An owned session—also
including the current session—returns `204 No Content` and is immediately
unusable. Repeating revocation of an owned session remains successful. Missing,
malformed, and foreign IDs all return `session_not_found` with HTTP 404.

Missing, malformed, expired, inactive, or revoked Bearer tokens return
`invalid_session` with HTTP 401 and `WWW-Authenticate: Bearer`.

## Read two-factor status

`GET /v1/account/mfa`

Use a valid device Bearer token. The no-store response reports only whether
TOTP is enabled and how many unused recovery codes remain:

```json
{
  "enabled": false,
  "recovery_codes_remaining": 0
}
```

Pending enrollment is reported as disabled. The response never contains the
account ID, TOTP secret, encrypted secret, nonce, code digests, failure count,
or lock timestamp.

## Begin TOTP enrollment

`POST /v1/account/mfa`

Use a valid device Bearer token and re-enter the account password:

```json
{
  "password": "a-long-private-passphrase"
}
```

Correct proof returns `201 Created` and `Cache-Control: no-store`:

```json
{
  "secret": "BASE32ENCODEDTOTPSECRET",
  "provisioning_uri": "otpauth://totp/OmarchyGS%3Aplayer%5Fone?secret=BASE32ENCODEDTOTPSECRET&issuer=OmarchyGS&algorithm=SHA1&digits=6&period=30"
}
```

Transfer the secret or URI to a trusted authenticator application. The server
persists the random 160-bit secret only as AES-256-GCM ciphertext, with the
account UUID bound as associated data. Beginning again replaces an unconfirmed
enrollment; it cannot replace an enabled authenticator. Enrollment expires
after ten minutes and does not affect login until confirmed. Wrong passwords
return `invalid_credentials`; an enabled account returns
`mfa_already_enabled` with HTTP 409.

## Confirm TOTP enrollment

`POST /v1/account/mfa/confirm`

Use the same Bearer-authenticated account and submit the six-digit code shown
by the newly provisioned authenticator:

```json
{
  "code": "123456"
}
```

Successful confirmation returns `200 OK`, `Cache-Control: no-store`, and ten
recovery codes:

```json
{
  "recovery_codes": [
    "OGS-ABCD-EFGH-IJKL-MNOP-QRST-UVWX"
  ]
}
```

The actual response contains ten independently generated codes with 120 random
bits apiece. Store them outside the OmarchyGS installation. They are returned
only by this successful confirmation and cannot be retrieved later;
PostgreSQL stores only SHA-256 digests. A code can complete one future login or
authorize MFA disablement, then is unusable.

TOTP follows RFC 6238's six-digit HMAC-SHA-1, 30-second profile and accepts the
current step plus one past or future step for clock drift. A successfully used
time step is not accepted again. Invalid confirmation returns
`invalid_mfa_code`; missing or expired enrollment returns
`mfa_enrollment_not_found` with HTTP 409.

## Complete an MFA device login

`POST /v1/sessions/mfa`

Send the opaque challenge from the `202` password-login response and either a
six-digit TOTP or one unused recovery code:

```json
{
  "challenge_token": "ogm1_<base64url-encoded-random-value>",
  "code": "123456"
}
```

A successful request consumes the challenge and factor atomically and returns
the ordinary `201 Created` device-session response under
`Cache-Control: no-store`. Replaying the challenge returns
`invalid_mfa_challenge`; replaying the factor with a new challenge returns
`invalid_mfa_code`. Expired, consumed, malformed, or inactive-account
challenges create no session.

Each challenge allows five failed factor attempts. Failure state also belongs
to the authenticator, so requesting new challenges cannot reset it. Five
consecutive failures lock factor verification for five minutes and return
`mfa_rate_limited` with HTTP 429. Successful verification resets that state.
The independent ten-challenge cap separately bounds password-valid challenge
issuance and cannot consume or replace a legitimate in-progress challenge.
This local control does not replace distributed login throttling at a public
edge.

## Disable TOTP MFA

`DELETE /v1/account/mfa`

Use a valid device Bearer token, re-enter the account password, and supply an
unused TOTP or recovery code:

```json
{
  "password": "a-long-private-passphrase",
  "code": "OGS-ABCD-EFGH-IJKL-MNOP-QRST-UVWX"
}
```

Success returns `204 No Content` and atomically removes the encrypted
authenticator, recovery codes, and outstanding MFA challenges. Existing
device sessions keep their independent expiry/revocation lifecycle; future
password login returns the ordinary `201` session response. Wrong passwords,
invalid factors, rate limits, and disabled-state conflicts do not change MFA
state.

The server requires `OGS_MFA_ENCRYPTION_KEY` at startup as an unpadded,
base64url-encoded 32-byte value. Operators must provide the same protected key
across replicas and restarts and include it in backup/restore planning. Loss of
the key makes enrolled TOTP secrets unverifiable. TOTP codes and recovery codes
must travel only over TLS in production. TOTP is not phishing-resistant;
WebAuthn/passkeys are outside the current slice.

## Create a persona

`POST /v1/personas`

Use a valid device Bearer token. The account owner is always derived from that
session; clients cannot provide or change it. An account may own multiple
personas.

```json
{
  "handle": "Player_One",
  "display_name": "Player One",
  "bio": "Usually up for a strategy game.",
  "status_message": "Ready to play"
}
```

`bio` and `status_message` may be omitted and default to empty strings. Handles
are trimmed and converted to ASCII lowercase, then must be 3–24 bytes, begin
with an ASCII letter or digit, and contain only ASCII letters, digits,
underscores, or hyphens. Display names are trimmed and contain 1–64 Unicode
non-control characters. Bios contain at most 1,000 Unicode characters and may
use tabs and newlines but no other controls. Status messages are trimmed and
contain at most 160 Unicode non-control characters. Persona write bodies are
limited to 8 KiB.

A successful request returns `201 Created`, `Cache-Control: no-store`, and one
public profile:

```json
{
  "id": "dc26c171-f3bd-4752-ac22-dd8b3e81920f",
  "handle": "player_one",
  "display_name": "Player One",
  "bio": "Usually up for a strategy game.",
  "status_message": "Ready to play",
  "created_at": "2026-08-24T15:00:00.000Z",
  "updated_at": "2026-08-24T15:00:00.000Z"
}
```

This is the complete public persona shape. It never contains `account_id`,
credentials, sessions, tokens, or token digests. Validation errors return 422;
canonical handle conflicts return `handle_taken` with HTTP 409.

## List owned personas

`GET /v1/personas`

Use a valid device Bearer token. The response carries `Cache-Control: no-store`
and wraps every persona owned by the authenticated account in `personas`. It
never returns personas owned by another account or reveals the account link.

## Look up a public persona

`GET /v1/personas/by-handle/{handle}`

This endpoint is public and intentionally enumerable by exact handle. Lookup
trims and ASCII-lowercases the handle and returns the public profile shape
above. Invalid and absent handles return the same `persona_not_found` response
with HTTP 404. Account ownership remains private.

## Edit an owned persona

`PATCH /v1/personas/{persona_id}`

Use a valid device Bearer token and include one or more of `handle`,
`display_name`, `bio`, or `status_message`. Unknown fields are rejected, and an
empty patch returns `empty_persona_patch` with HTTP 422. A successful owner edit
returns the updated public profile with `Cache-Control: no-store` and advances
`updated_at`.

The server checks both the persona UUID and the authenticated account in the
same update. Missing, malformed, and foreign persona IDs therefore return the
same `persona_not_found` response with HTTP 404 and do not change storage.
Changing a handle moves public lookup immediately: the old handle returns 404
and the new canonical handle resolves the updated profile.

Bearer tokens grant account-level authority. Production transport must protect
them with TLS, and a public deployment must add distributed login throttling;
neither deployment control is supplied by the current local slice.

## Request a persona connection

`PUT /v1/personas/{persona_id}/connection-requests/{other_persona_id}`

Use a valid Bearer token. `persona_id` must belong to that token's account and
`other_persona_id` must be a persona owned by a different account. The command
has no request body. A new request returns `201 Created`; retrying the same
outgoing request returns `200 OK` with the same resource. Both responses carry
`Cache-Control: no-store`:

```json
{
  "persona": {
    "id": "88c08898-4f83-4a0a-8929-475c5b56db44",
    "handle": "player_two",
    "display_name": "Player Two",
    "bio": "Usually up for a strategy game.",
    "status_message": "Ready",
    "created_at": "2026-08-24T18:00:00.000Z",
    "updated_at": "2026-08-24T18:00:00.000Z"
  },
  "created_at": "2026-08-24T18:05:00.000Z"
}
```

There is at most one pending or accepted row for an unordered persona pair.
If the other persona has already requested the actor, the command returns 409
`connection_request_pending`; use the acceptance command instead. An accepted
pair returns 409 `connection_already_exists`. A missing, malformed,
same-account, or blocked target returns the same 409
`connection_unavailable`, including when the target privately blocked the
actor. The API never discloses which unavailable condition applies. New
requests are also unavailable when either the requester's 100 outgoing pending
requests or the addressee's 100 incoming pending requests is full. Retrying an
existing outgoing request remains idempotent at the limit.

## List pending connection requests

`GET /v1/personas/{persona_id}/connection-requests`

The acting persona must belong to the authenticated account. The no-store
response separates incoming and outgoing requests, orders each by creation
time and persona ID, and embeds only public persona profiles:

```json
{
  "incoming": [
    {
      "persona": { "id": "...", "handle": "player_two" },
      "created_at": "2026-08-24T18:05:00.000Z"
    }
  ],
  "outgoing": []
}
```

The abbreviated persona above represents the same complete seven-field public
shape returned by persona lookup; no account ID, session data, or block state
is added. Each direction contains at most 100 entries. A malformed, absent, or
foreign-owned acting persona returns the same 404 `persona_not_found` response.

## Accept a connection request

`PUT /v1/personas/{persona_id}/connections/{requester_persona_id}`

Only the owner of the pending request's addressee persona can accept it. A
successful transition returns `200 OK`, `Cache-Control: no-store`, and the
requester's public profile with the durable acceptance time:

```json
{
  "persona": { "id": "...", "handle": "player_one" },
  "connected_at": "2026-08-24T18:06:00.000Z"
}
```

Retrying after acceptance returns the same connection. The requester cannot
self-accept an outgoing request. A missing, wrong-direction, or foreign pair
returns 404 `connection_request_not_found`; invalid acting identity remains
404 `persona_not_found`.

## List accepted connections

`GET /v1/personas/{persona_id}/connections`

The owner-scoped, no-store response contains one entry per mutual accepted
connection in acceptance-time and persona-ID order:

```json
{
  "connections": [
    {
      "persona": { "id": "...", "handle": "player_two" },
      "connected_at": "2026-08-24T18:06:00.000Z"
    }
  ]
}
```

Each nested persona is the complete public seven-field profile. Connection
state is mutual, so both participants see the other persona with the same
`connected_at` value.

## Remove or cancel a connection

`DELETE /v1/personas/{persona_id}/connections/{other_persona_id}`

Either participant may remove an accepted connection, cancel an outgoing
request, or decline an incoming request. The command returns `204 No Content`
and is idempotent, including when the target or pair state is already absent.
The acting persona is still authenticated and owner-scoped first; a malformed,
absent, or foreign acting persona returns 404 `persona_not_found`.

## Block a persona

`PUT /v1/personas/{persona_id}/blocks/{other_persona_id}`

The acting persona must be owned by the authenticated account, and the target
must belong to another account. A new directional block returns `201 Created`;
retry returns `200 OK`. The no-store response contains the target's public
persona profile and block creation time:

```json
{
  "persona": { "id": "...", "handle": "player_two" },
  "created_at": "2026-08-24T18:10:00.000Z"
}
```

The server locks the persona pair, inserts the block, and deletes any pending
or accepted relationship in one transaction. While either direction is
blocked, both personas receive the same `connection_unavailable` response when
requesting the other. The target cannot query who blocked them. This protects
the block row and block inventory from direct disclosure; it does not promise
to conceal every indirect inference a person may draw from a denied
interaction with a known persona.

## List and remove private blocks

`GET /v1/personas/{persona_id}/blocks`

Returns the acting persona's directional blocks under `blocks`, ordered by
creation time and persona ID. Entries use the block response above. This
inventory is owner-scoped and `Cache-Control: no-store`; another persona's
block list is never returned.

`DELETE /v1/personas/{persona_id}/blocks/{other_persona_id}`

Returns `204 No Content` and is idempotent. Unblocking permits a future request
but never restores a removed request or connection. Connection and block pair
mutations lock both persona rows in UUID order so opposite requests and
request-versus-block races resolve to one database-serialized outcome.
Accepting an actual pending request also creates or reuses the pair's private
conversation and appends one typed acceptance message. Retrying acceptance does
not append another message. State-changing request, acceptance, removal, block,
and unblock operations also append the persona-local invalidations described in
the synchronization section below; successful no-op retries append no event.

## List private conversations

`GET /v1/personas/{persona_id}/conversations?limit=50`

The acting persona must belong to the authenticated account. The no-store
response returns at most 100 conversations in most-recently-active order. Each
entry contains the other participant's complete public persona, the actor's
private unread count, and the latest tagged message. It never exposes either
account ID or the other participant's read position:

```json
{
  "conversations": [
    {
      "id": "2b7ae665-98ac-4b30-9ea0-d4f57ae949b4",
      "other_persona": { "id": "...", "handle": "player_two" },
      "unread_count": 1,
      "latest_message": {
        "type": "system",
        "id": "...",
        "sequence": 12,
        "system": {
          "type": "connection_accepted",
          "actor": { "id": "...", "handle": "player_two" }
        },
        "created_at": "2026-08-24T18:06:00.000Z"
      },
      "created_at": "2026-08-24T18:06:00.000Z",
      "updated_at": "2026-08-24T18:06:00.000Z"
    }
  ]
}
```

The abbreviated personas above stand for the same seven-field public profile
used throughout the social API. Limits outside 1–100 return 422
`invalid_pagination`.

## Send a private user message

`POST /v1/personas/{persona_id}/conversations/{conversation_id}/messages`

The body accepts exactly one field:

```json
{ "body": "Ready for a match?" }
```

Text is trimmed, must contain 1–4,000 Unicode characters, and may contain tabs
and newlines but no other controls. A successful send returns `201 Created`,
`Cache-Control: no-store`, and the explicit user-message variant:

```json
{
  "type": "user",
  "id": "...",
  "sequence": 13,
  "sender": { "id": "...", "handle": "player_one" },
  "body": "Ready for a match?",
  "created_at": "2026-08-24T18:07:00.000Z"
}
```

Clients cannot supply type, sender, sequence, timestamp, or system content.
The sender is advanced through the new message; the peer sees it as unread.
Sending requires that the pair is currently connected and unblocked. Removal
or a block in either direction produces the same 409
`conversation_unavailable`; unblocking alone does not reconnect the pair.

## Read private message history

`GET /v1/personas/{persona_id}/conversations/{conversation_id}/messages?before=&limit=50`

Only either durable conversation participant may read history, even after the
connection is removed or blocked. Each page is ordered by ascending
conversation-local message sequence, defaults to 50, caps at 100, and returns
`next_before` for an older page. Activity in another conversation does not
create sequence gaps. User messages have the shape above. System messages use `type:
system` and a nested tagged `system` object; currently the only system variant
is `connection_accepted` with its public actor.

Malformed, absent, and non-participant conversation IDs all return 404
`conversation_not_found`. Invalid limits or non-positive `before` cursors
return 422 `invalid_pagination`.

## Mark a private conversation read

`PUT /v1/personas/{persona_id}/conversations/{conversation_id}/read/{message_id}`

The message must belong to the conversation. The command moves only the acting
participant's private read position forward and is safe to retry or race with
an older acknowledgement:

```json
{
  "through_message_id": "...",
  "unread_count": 0
}
```

A missing, malformed, or cross-conversation message returns 404
`message_not_found`. All inbox queries and writes require a valid Bearer token,
owner-scope the acting persona before disclosing objects, and return
`Cache-Control: no-store`, including on domain and request-parsing errors.

## List compiled games

`GET /v1/games`

This public endpoint returns stable `(key, version)`-ordered metadata for every
compiled production game definition:

```json
{
  "games": [
    {
      "key": "example_game",
      "version": 1,
      "display_name": "Example Game",
      "min_human_players": 1,
      "max_human_players": 2
    }
  ]
}
```

Until the first playable game is implemented, the production response is
honestly `{"games":[]}`. Tests inject compiled fixture definitions; those
fixtures are not production catalog entries.

## Read participating game sessions

`GET /v1/personas/{persona_id}/game-sessions?limit={1-100}`

`GET /v1/personas/{persona_id}/game-sessions/{game_session_id}`

Both routes require a valid device Bearer token that owns the acting persona,
and that persona must be an ordered participant in every returned session.
Inventory defaults to 50, caps at 100, and orders sessions newest-first. The
detail route returns one session. Private successes, authorization failures,
and extractor failures carry `Cache-Control: no-store`.

```json
{
  "id": "...",
  "game_key": "example_game",
  "game_version": 1,
  "revision": 0,
  "status": "active",
  "state": {},
  "participants": [
    {
      "seat": 0,
      "persona": {
        "id": "...",
        "handle": "player_one",
        "display_name": "Player One",
        "bio": "",
        "status_message": "",
        "created_at": "2026-08-24T22:00:00.000Z",
        "updated_at": "2026-08-24T22:00:00.000Z"
      }
    }
  ],
  "created_at": "2026-08-24T22:00:00.000Z",
  "updated_at": "2026-08-24T22:00:00.000Z"
}
```

Inventory wraps these documents as `{"sessions":[...]}`. The game key and
positive version are immutable: reading a session never substitutes a newer
compiled rules version. Malformed, absent, and non-participant session IDs all
return 404 `game_session_not_found`; a foreign acting persona uses the same 404
`persona_not_found` as an absent one. Invalid limits return 422
`invalid_pagination`. No public session-creation route exists; challenge
orchestration remains a separate roadmap slice.

## Apply a game command

`POST /v1/personas/{persona_id}/game-sessions/{game_session_id}/commands`

This route requires a valid device Bearer token that owns the acting persona,
and that persona must participate in the active session. The request body is
limited to 32 KiB, rejects unknown top-level fields, and contains a
session-wide UUID, the client's current session revision, and a bounded JSON
object command:

```json
{
  "idempotency_key": "8f5d8f1d-48df-4f5a-b6e7-ad26eb30ae88",
  "expected_revision": 0,
  "command": {
    "kind": "advance"
  }
}
```

The server locks the durable session, resolves only its stored exact compiled
game version, and applies the command to its current object snapshot using the
actor's durable seat. A first-use accepted command increments the revision by
exactly one and returns only the authoritative committed result:

```json
{
  "game_session_id": "...",
  "revision": 1,
  "state": {}
}
```

Replaying the same idempotency UUID with the same actor, original expected
revision, and semantically equal JSON command returns that original response
without executing or notifying again. Reusing the UUID for a different
participating actor, revision, or command returns 409
`game_idempotency_conflict`. A stale or future new command returns 409
`game_revision_conflict`; the response intentionally omits the current
revision, so the client must refetch the participant-authorized session.

An unavailable stored rules version returns 409 `game_unavailable`. Invalid
input returns 422 `invalid_game_command`, and a stable compiled-rules rejection
returns 422 `game_command_rejected`. Malformed, absent, and non-participant
sessions all return 404 `game_session_not_found`. Successful commands persist
the snapshot, revision, private replay receipt, timestamp, and one minimal
`game_session_changed` event per participant in one PostgreSQL transaction.
Conflicts, replays, rejections, and rollbacks append no event. All command
responses carry `Cache-Control: no-store`.

## Read durable persona changes

`GET /v1/personas/{persona_id}/sync?after={cursor}&limit={1-100}`

Use a valid device Bearer token that owns the acting persona. All responses,
including extractor and authorization errors, carry `Cache-Control: no-store`.
Omitting `after` captures an empty baseline at the persona's current cursor:

```json
{
  "events": [],
  "next_cursor": 42,
  "has_more": false,
  "reset_required": false
}
```

A reconnect-safe client captures this baseline, reads the authoritative social
and inbox REST resources, then requests events after the captured cursor. With
`after`, the endpoint returns events in ascending persona-local cursor order;
the default page size is 50 and the maximum is 100. Continue with the last
`next_cursor` while `has_more` is true.

The feed contains invalidations rather than resource data:

```json
{
  "events": [
    {
      "type": "connections_changed",
      "cursor": 43,
      "created_at": "2026-08-24T20:00:00.000Z"
    },
    {
      "type": "conversation_changed",
      "cursor": 44,
      "conversation_id": "58ee076d-0216-422c-b1e2-48ee7fa648bb",
      "created_at": "2026-08-24T20:00:01.000Z"
    },
    {
      "type": "game_session_changed",
      "cursor": 45,
      "game_session_id": "c48edcea-24dd-46f7-9eed-786968e31fa1",
      "created_at": "2026-08-24T20:00:02.000Z"
    }
  ],
  "next_cursor": 45,
  "has_more": false,
  "reset_required": false
}
```

The other event types are `connection_requests_changed` and `blocks_changed`.
Game-session events carry only the participant-authorized session UUID, never
state or participant data. Events never contain message bodies, profiles,
account identity, read counts, block direction, or credentials. Each persona
retains only its newest 10,000 events. If `after` is older than retained
history, the server returns an empty
page at the current cursor with `reset_required: true`; repeat the baseline and
authoritative REST snapshot flow. Negative or future cursors return
`invalid_sync_cursor` with HTTP 422. Invalid limits return `invalid_pagination`
with HTTP 422. Missing and foreign personas use the same `persona_not_found`
response.

## Subscribe to live persona change hints

`GET /v1/personas/{persona_id}/sync/live`

Upgrade this route to a WebSocket with the ordinary `Authorization: Bearer`
header belonging to the persona owner. Query-string tokens are not accepted.
The first server message captures the current durable cursor:

```json
{"type":"ready","cursor":44}
```

Later messages are only `{"type":"changed"}` or
`{"type":"resync_required"}`. A hint carries no domain data: fetch the durable
sync feed and then the affected REST resource. `resync_required` means the
process-local hint buffer was overrun and the client must repeat the baseline
and snapshot recovery flow. Duplicate or missed `changed` hints are safe
because PostgreSQL's retained event feed is authoritative.

The channel is server-to-client only. Client frames and assembled messages are
limited to 1 KiB; text or binary client messages close the channel, while
standard ping and close control frames remain supported. The server admits at
most five live sockets per persona, 20 per account, and 256 per process,
returning `sync_socket_limit_reached` with HTTP 429 before upgrade when a limit
is full. Session authority is rechecked before each persona hint and at least
every 30 seconds without advancing `last_used_at`; revocation, expiry, or an
inactive account closes an established socket.
