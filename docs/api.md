# HTTP API

Omarchy Gaming System uses versioned JSON endpoints for durable commands and queries.
The server remains authoritative for validation and persistence.

## Discover a server

`GET /.well-known/omarchygs`

This public unauthenticated endpoint returns `Cache-Control: no-store` and an
exact compatibility document:

```json
{
  "service": "omarchy-gaming-system",
  "server_id": "58ee076d-0216-422c-b1e2-48ee7fa648bb",
  "server_name": "OmarchyGS Community",
  "protocol_version": 1,
  "capabilities": [
    "accounts.invite-registration.v1",
    "auth.device-sessions.v1",
    "auth.totp.v1",
    "games.cartridge-acquisition.v1",
    "games.cartridge-catalog.v1",
    "games.challenges.v1",
    "games.session-cartridge-acquisition.v1",
    "games.session-cartridge.v1",
    "games.sessions.v1",
    "identity.personas.v1",
    "social.connections.v1",
    "social.private-inbox.v1",
    "social.reporting.v1",
    "sync.cursor.v1",
    "sync.websocket-hints.v1"
  ]
}
```

`games.cartridge-acquisition.v1`, `games.session-cartridge.v1`, and
`games.session-cartridge-acquisition.v1` are present only when the server has a
complete reviewed cartridge-distribution configuration. Metadata-only servers
omit all three and do not register either acquisition route.

The UUID is generated once by migration `0018`, is immutable in ordinary
database operation, and survives PostgreSQL dump/restore. The public name is
the independently changeable `OGS_SERVER_NAME`. Capabilities are unique and
lexically ordered; `games.registered-provider.v1` is added only when that
runtime is configured. The response contains no account, credential,
provider-secret, database-location, or private operator data.

If the singleton identity cannot be read, the endpoint returns `503` with
`server_discovery_unavailable`. `/health` remains the operational liveness
contract and is not a compatibility API.

## Register an account

`POST /v1/accounts`

Request:

```json
{
  "invite_code": "ogsi_<base64url-encoded-random-value>",
  "username": "Player_One",
  "password": "a-long-private-passphrase"
}
```

Registration is invitation-only. A database-local operator issues one
`ogsi_` code for one account as described in the
[private-alpha runbook](operators/private-alpha.md). The 48-character code
contains 256 random bits; PostgreSQL stores only its 32-byte SHA-256 digest.
It expires after its operator-selected 1–720 hour lifetime and cannot be used
after revocation or consumption.

Usernames are trimmed and converted to ASCII lowercase. The canonical value
must be 3–32 bytes, begin with an ASCII letter or digit, and contain only ASCII
letters, digits, underscores, or hyphens. Passwords are not trimmed and must be
12–128 bytes. Registration request bodies are limited to 1 KiB.

A valid unused invitation and available username create and link one account
in the same transaction. Success returns `201 Created` under
`Cache-Control: no-store`:

```json
{
  "id": "58ee076d-0216-422c-b1e2-48ee7fa648bb",
  "username": "player_one"
}
```

The response never contains the invitation, password, or either digest/hash.
Passwords are stored as
uniquely salted Argon2id v19 PHC strings with `m=19456`, `t=2`, and `p=1`.

If the first response is lost, retrying the same code with the same canonical
username and password returns the original two-field receipt with `200 OK`.
That recovery does not make the code reusable: another username or password
returns `403 Forbidden` with `invalid_invitation`. A canonical username
conflict on an otherwise unused invitation returns 409 and leaves that
invitation unused so the player can choose another username.

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

Malformed, absent, expired, revoked, already-used, and changed-intent codes all
return the same `invalid_invitation` response and disclose no lifecycle or
operator metadata. Current registration error codes are `invalid_username`,
`invalid_password`, `invalid_invitation`, `username_taken`, and
`internal_error`.

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

## Report another persona

`POST /v1/personas/{reporter_persona_id}/reports`

Use a valid device Bearer token. The path persona must belong to the
authenticated account. Resolve the intended subject through exact public
handle lookup, then submit its UUID with a fresh operation UUID:

```json
{
  "idempotency_key": "3b569db7-1aaa-4d8c-afbb-a995c56c4e44",
  "subject_persona_id": "1f538dbf-bbe7-48fc-b9ca-31bc3af96f69",
  "category": "harassment",
  "detail": "Repeated unwanted messages after I asked them to stop."
}
```

Categories are exactly `harassment`, `spam`, `cheating`, or `other`. Detail is
trimmed, must contain 1–1,000 characters, and may contain ordinary tabs and line
breaks but no other control characters. A persona cannot report itself. Each
reporter may have at most 25 open reports; resolving or dismissing a report
through the local operator workflow releases capacity.

A new report returns `201 Created`, `Cache-Control: no-store`, and only this
receipt:

```json
{
  "id": "edceff52-2e75-4e3c-ae92-ab09b1f510f0",
  "idempotency_key": "3b569db7-1aaa-4d8c-afbb-a995c56c4e44",
  "status": "open",
  "created_at": "2026-08-26T16:30:00.000Z"
}
```

An exact retry returns the same receipt with HTTP 200, including after the
report has been dispositioned. Reusing the UUID with another subject, category,
or normalized detail returns HTTP 409 `report_idempotency_conflict`. Invalid
input and self-reporting return HTTP 422 `invalid_report`; an absent subject or
unowned/malformed reporter path returns HTTP 404 `persona_not_found` after
Bearer authentication; the open-report cap returns HTTP 429
`report_limit_reached`. No player route lists reports, exposes the subject's
account, reveals other reporters, or returns operator action/audit state.

## QML account and persona onboarding

The production QML connector composes the existing endpoints above without a
separate client protocol. It begins with exact `/health`, then uses
`POST /v1/accounts`, `POST /v1/sessions`, optional
`POST /v1/sessions/mfa`, authenticated `GET /v1/personas`, and authenticated
`POST /v1/personas`. Registration shows a masked invitation field only in
create-account mode and sends its value only in the JSON body to the already
admitted origin. The invitation and password fields clear on submission,
Escape, mode change, or server change. Registration does not implicitly authenticate; the
canonical returned username is carried back to the sign-in form and the player
must submit credentials explicitly.

The connector defaults to `http://127.0.0.1:8080`. A player can choose another
endpoint in the connection screen or pass `--server-url=<origin>` to `qml6`.
Only a bare HTTPS origin or loopback HTTP origin is accepted: userinfo, paths,
queries, fragments, invalid ports, and remote plaintext HTTP fail before any
credential request. Each operation uses one bounded XHR generation, rejects
stale completions and unexpected redirects, and validates exact success shapes
before changing client authority.

Passwords, TOTP/recovery input, bearer tokens, and MFA challenge tokens are not
written to QML `Settings`, LocalStorage, files, URLs, logs, or visible status
text. Secret fields are masked and cleared synchronously after submission. The
raw bearer is held only by the in-memory API client and appears only in the
`Authorization` header on authenticated persona requests. Local logout clears
the process state but does not claim remote session revocation; device-session
inventory/revocation and OS-keyring persistence remain later settings work.
An authenticated `401 invalid_session`, endpoint change, or authenticated
protocol-shape failure clears bearer, MFA, inventory, and selected persona and
returns to sign-in.

The player shell also exposes keyboard-first Games, Challenges, and Gameplay
screens through the same in-memory bearer authority. Their controller chains
the public catalog with participant-owned REST inventories, validates exact
challenge/session and Signal Siege state shapes, and derives mutation paths,
targets, revisions, and fresh non-secret idempotency UUIDs internally. It does
not poll or send gameplay over WebSockets. A revision conflict refetches the
session, and an uncertain transport outcome offers an explicit retry with the
same UUID and request intent.

Signal Siege is presented by trusted platform QML through a narrow derived
view model and the three allowlisted actions. This first-playable presenter is
not a signed cartridge and never invents a cartridge origin or digest;
publisher presentation remains restricted to verified installed packages.

The Social screen also resolves an exact public handle and submits the bounded
report above through the same credential-owning gateway. The QML controller
retains one operation UUID while the same form is retried after a transport or
protocol failure, validates the exact receipt, clears handle/detail only after
success, and clears the full player authority on a valid `invalid_session`.
Report text is presented as plain text and is never emitted through sync or
WebSocket hints.

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
create sequence gaps. User messages have the shape above. System messages use
`type: system` and a nested tagged `system` object. `connection_accepted`
contains its public actor. Challenge lifecycle variants are
`game_challenge_created`, `game_challenge_accepted`,
`game_challenge_declined`, and `game_challenge_cancelled`; each contains the
public actor and `challenge_id`, while acceptance alone also contains
`game_session_id`. These immutable references do not copy current challenge
status into inbox history.

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

## List admitted Game Cartridges

`GET /v1/cartridges`

This metadata-only endpoint requires a valid device-session Bearer token and
returns `Cache-Control: no-store` on success and failure. It lists only exact
server selections that are present in the current marketplace snapshot,
imported, compatible with this host, and currently `active` or `deprecated`:

```json
{
  "cartridges": [
    {
      "game_key": "door-legends",
      "publisher_id": "ignibyte",
      "rules_version": 1,
      "cartridge_version": 2,
      "display_name": "Door Legends",
      "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "signed_identity_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      "marketplace": {
        "provenance_class": "marketplace_vetted",
        "marketplace_id": "omarchygs-marketplace",
        "marketplace_name": "OmarchyGS Marketplace",
        "reviewed_by": "review-team",
        "review_summary": "Bounded first-party review passed.",
        "policy_version": 1,
        "lifecycle_status": "deprecated"
      },
      "server_admission": {"revision": 4},
      "warning": "Upgrade when practical."
    }
  ]
}
```

`warning` is present only for a deprecated release and contains its public
lifecycle reason. Suspended, denied, removed, incompatible, unimported, and
locally inactive releases are omitted; the server never substitutes another
version. The response deliberately excludes marketplace URLs, local paths,
public-key material, raw signed records, operator identities/reasons, and
download authority. Missing or invalid authentication returns the normal
`invalid_session` envelope. A database read failure returns 500
`internal_error`.

This catalog describes inert presentation releases selected by the server. It
does not replace public `GET /v1/games`, which describes currently implemented
gameplay authorities and rules versions. The packaged client uses its exact
digest and admission revision as the input to an explicit acquisition.

## Acquire one exact Game Cartridge

`GET /v1/cartridges/{game_key}/{archive_sha256}/acquisition`

This route exists only while `games.cartridge-acquisition.v1` is advertised and
requires a valid device-session Bearer token. The requested key and lowercase
SHA-256 digest must name the current effective catalog selection. Success is
canonical `application/json` with `Cache-Control: no-store`:

```json
{
  "format": "omarchygs.cartridge-acquisition/v1",
  "server_admission": {
    "server_id": "58ee076d-0216-422c-b1e2-48ee7fa648bb",
    "game_key": "door-legends",
    "publisher_id": "ignibyte",
    "rules_version": 1,
    "cartridge_version": 2,
    "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "signed_identity_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "admission_revision": 4
  },
  "marketplace_key": {
    "format_version": 1,
    "algorithm": "ed25519",
    "key_id": "marketplace-2026",
    "authority_id": "omarchygs-marketplace",
    "verifying_key": "<base64url public key>"
  },
  "signed_marketplace_snapshot": "<base64url exact signed snapshot>",
  "archive": "<base64url canonical .ogsc bytes>",
  "conformance": "<base64url canonical conformance record>",
  "release_attestation": "<base64url publisher release attestation>"
}
```

The server resolves the exact immutable store entry and self-verifies the
envelope before returning it. The response includes public verification
evidence but no marketplace URL, download redirect, filesystem destination,
credential, private key, operator reason, executable, raw QML, or backend
endpoint. The client independently verifies the selected-server admission,
requires `marketplace_key` to equal its complete locally provisioned
marketplace trust key, then verifies the snapshot signature, publisher
identity, lifecycle policy, SDK/host
compatibility, archive, conformance, and attestation, then re-reads the catalog
before mounting it. The response key is public evidence, not a trust-on-first-use
channel; clients must not learn their marketplace trust root from this route or
any other selected-server response.

Invalid identity syntax returns 422 `cartridge_acquisition_invalid_input`.
Absent, stale, denied, mismatched, or no-longer-effective exact releases return
404 `cartridge_acquisition_denied` without distinguishing the cause. Missing or
invalid authentication returns `invalid_session`; database or encoding failure
returns 500 `internal_error`.

## Acquire an exact cartridge pinned to a session

`GET /v1/personas/{persona_id}/game-sessions/{game_session_id}/cartridge-acquisition`

This route exists only while `games.session-cartridge-acquisition.v1` is
advertised. It requires a valid device-session Bearer that owns `persona_id`,
and that persona must participate in the requested session. Success returns the
same canonical `omarchygs.cartridge-acquisition/v1` document as the current
catalog route, with `Cache-Control: no-store`.

The release is selected only by the session's immutable presentation pin and
its retained signed marketplace snapshot/release evidence. Today's server
catalog selection is deliberately not consulted and cannot substitute another
digest. Current signed active-session policy still applies: retained historical
provenance proves origin and unchanged bytes, not present authorization.

The native companion reads the participant-visible session before acquisition,
derives the exact expected server admission, verifies the acquisition with its
client-controlled marketplace key, then reads the session again before
publishing the exact mount. A changed pin, server, lifecycle decision, digest,
revision, or evidence fails closed.

Malformed persona or session identity returns 422
`cartridge_acquisition_invalid_input`. Absent, foreign, unbound, unavailable,
or lifecycle-denied sessions return 404 `cartridge_acquisition_denied` without
distinguishing the cause. Missing or invalid authentication returns
`invalid_session`; database or encoding failure returns 500 `internal_error`.

## List games

`GET /v1/games`

This public endpoint returns stable `(key, version)`-ordered metadata for every
available compiled game and the one optional operator-enabled remote pilot:

```json
{
  "games": [
    {
      "key": "signal_siege",
      "version": 1,
      "display_name": "Signal Siege",
      "min_human_players": 1,
      "max_human_players": 1,
      "authority": "platform_compiled",
      "provider_release_id": null
    },
    {
      "key": "signal_siege",
      "version": 2,
      "display_name": "Signal Siege Versus",
      "min_human_players": 2,
      "max_human_players": 2,
      "authority": "platform_compiled",
      "provider_release_id": null
    }
  ]
}
```

Signal Siege v1 and v2 are immutable compiled production entries. When the
provider runtime and Door Legends pilot are both active, the response also
includes `door-legends` v1 with `authority: "registered_provider"` and its
immutable release UUID. Provider endpoints, keys, grants, subjects, and health
internals are never serialized. Tests may inject other compiled fixture
definitions; those fixtures are not production catalog entries.

### Signal Siege v1 rules

Signal Siege is a one-human game against an in-process deterministic bot. Both
sides begin with eight core and two energy; energy is capped at four. Each
human command chooses `strike`, `guard`, or `charge`. Strike and guard cost one
energy, strike deals two damage unless the opponent guards, and charge gains
two energy up to the cap. The server selects the bot action from the stored
pre-command state, then resolves both actions simultaneously.

A match completes when either core reaches zero or after round 12. The recorded
outcome compares remaining core, then remaining energy, and otherwise records a
draw. The bot has no account, persona, session, or participant row. Its policy
has no clock, database, network, or ambient-random input, so the same pinned
state and command always produce the same transition.

### Signal Siege v2 versus rules

Signal Siege v2 is the exact two-human challenge version. The challenger is
seat 0, the challenged persona is seat 1, and seat 0 takes the first turn.
Both players begin with eight core and two energy. Strike, guard, and charge
keep the v1 cost, damage, block, gain, and four-energy cap, but commands resolve
one visible seat at a time. A guard blocks the opponent's next strike and then
expires at the beginning of its owner's following turn.

The stored `active_seat` alternates after every accepted command. Commands from
the other participant, unaffordable actions, malformed state, or malformed
commands are rejected without advancing the session revision. A match ends
when either core reaches zero or after turn 24. The terminal outcome compares
core, then energy, then records a draw, and names `seat_0`, `seat_1`, or `draw`
without depending on clocks, networking, or ambient randomness.

## Start a solo game session

`POST /v1/personas/{persona_id}/game-sessions`

Use a valid device Bearer token that owns the acting persona. The request is
limited to 8 KiB, rejects unknown fields, and selects an exact compiled game
that admits exactly one human:

```json
{
  "idempotency_key": "91cc0000-0000-4000-8000-000000000001",
  "game_key": "signal_siege",
  "game_version": 1
}
```

A first start returns `201 Created` with the game-session representation; an
exact retry returns `200 OK` with that same durable session and creates no
additional state or sync event. Reusing the UUID for different game intent
returns 409 `game_idempotency_conflict`. Replay is resolved from the stored
receipt even when that game version is no longer in the current process
registry.

The persona becomes the sole participant at seat 0. A persona may have at most
25 active solo-started sessions; starts are serialized on that persona so
concurrent requests cannot exceed the boundary. A new request over the limit
returns 429 `too_many_active_game_sessions`, while exact retries still work.
Malformed idempotency keys return 422 `invalid_game_start`; unavailable game
versions return 409 `game_unavailable`; and a definition that does not admit
exactly one human returns 422 `invalid_game_participants`. Every response is
private and carries `Cache-Control: no-store`.

Every session now reports `authority`, `provider_release_id`, `availability`,
`presentation`, and `result`. A compiled session uses `platform_compiled`, a null release and
availability, and its existing authoritative `state`. A provider session uses
`registered_provider`, pins an exact release, and exposes a provider-reported
view through `state`; that view is presentation data, not platform-owned game
state. Provider sessions may report `provisioning`, `ready`, `reconciling`,
`unavailable`, `suspended`, `completed`, or `retired` availability. A validated
terminal callback adds only this public result projection:

```json
{
  "result": {
    "outcome": "escaped",
    "public_summary": { "ending": "sunlit_gate" },
    "provider_revision": 1,
    "projected_at": "2026-08-25T20:00:00.000Z"
  }
}
```

`presentation` is null for legacy, unconfigured, unmatched, or otherwise
unbound sessions. An eligible newly created session instead pins one exact
currently admitted release and returns this bounded participant-visible shape:

```json
{
  "format": "omarchygs.session-cartridge/v1",
  "publisher_id": "ignibyte",
  "game_key": "door-legends",
  "rules_version": 1,
  "cartridge_version": 2,
  "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "signed_identity_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "admission_revision": 4,
  "lifecycle_status": "active",
  "active_session_policy": "continue"
}
```

Deprecated bindings also include the signed bounded `warning`. Lifecycle can be
`active`, `deprecated`, `suspended`, `revoked`, or `retired`; the corresponding
active-session policy is `continue`, `suspend`, or `terminate`. The immutable
pin never follows a later catalog selection. Marketplace keys, operator reasons,
local paths, provider endpoints, grants, credentials, and internal release IDs
are not exposed.

## Reconcile a provider game session

`POST /v1/personas/{persona_id}/game-sessions/{game_session_id}/reconcile`

Use this participant-private route after an unknown provider outcome, outage,
or restore. The request is limited to the stable operation identity and the
last authenticated provider revision:

```json
{
  "idempotency_key": "91cc0000-0000-4000-8000-000000000003",
  "expected_revision": 1
}
```

The platform queries the pinned provider release and accepts only its signed
receipt. It never selects a winner by timestamp and never runs a compiled
fallback. A provider outage returns 503 `provider_unavailable`; retry with a
new reconciliation key after recovery. Exact operation replays remain stable.

## List provider achievements

`GET /v1/personas/{persona_id}/achievements`

This owner-scoped endpoint returns platform-approved achievements projected
from authenticated provider events. Each record contains only the pinned
definition (`key`, `display_name`, `description`), game and release identity,
session identity, provider revision, and award timestamp. Exact callback
replay cannot duplicate an award. Account IDs, pairwise subjects, grants,
provider endpoints, signatures, and private provider state are excluded.

## Create and read game challenges

`POST /v1/personas/{persona_id}/game-challenges`

The owned acting persona may challenge a different-account persona only while
the pair is connected and unblocked. The exact compiled game version must
admit two human players. Request bodies are limited to 8 KiB and accept only:

```json
{
  "idempotency_key": "91cc0000-0000-4000-8000-000000000001",
  "challenged_persona_id": "58ee076d-0216-422c-b1e2-48ee7fa648bb",
  "game_key": "example_game",
  "game_version": 1
}
```

A first request returns `201 Created`; an exact retry returns `200 OK` with the
same durable representation and no second inbox message or sync event. Reusing
the UUID for different intent returns `game_challenge_idempotency_conflict`.
Only one pending challenge for a directed pair and exact game version may
exist. Each persona is limited to 100 unexpired outgoing and 100 unexpired
incoming challenges. Expiry is server-owned at seven days.

`GET /v1/personas/{persona_id}/game-challenges?limit={1-100}&before={challenge_id}`

`GET /v1/personas/{persona_id}/game-challenges/{challenge_id}`

Inventory defaults to 50 records, is newest-first, retains terminal history,
and returns `next_before` only when another page exists. Detail and pagination
cursors are participant-authorized; malformed, missing, and foreign challenge
IDs share `game_challenge_not_found`, while an unusable cursor returns
`invalid_pagination`. A challenge response is allowlisted:

```json
{
  "id": "...",
  "game_key": "example_game",
  "game_version": 1,
  "direction": "incoming",
  "status": "pending",
  "challenger": { "id": "...", "handle": "player_one" },
  "challenged": { "id": "...", "handle": "player_two" },
  "game_session_id": null,
  "expires_at": "2026-09-01T20:00:00.000Z",
  "resolved_at": null,
  "created_at": "2026-08-25T20:00:00.000Z",
  "updated_at": "2026-08-25T20:00:00.000Z"
}
```

The abbreviated personas stand for the normal seven-field public projection.
Responses never contain the request idempotency key, account ownership,
connection/block direction, registry internals, or game state. Any read first
resolves a due pending row to retained `expired` history.

## Resolve a game challenge

`PUT /v1/personas/{persona_id}/game-challenges/{challenge_id}/accept`

`PUT /v1/personas/{persona_id}/game-challenges/{challenge_id}/decline`

`DELETE /v1/personas/{persona_id}/game-challenges/{challenge_id}`

Only the challenged persona may accept or decline; only the challenger may
cancel. Decline and cancel produce terminal history without a game session.
Acceptance additionally requires the pair still be connected and unblocked,
then creates exactly one session pinned to the invited game version with the
challenger at seat 0 and challenged persona at seat 1. Session creation,
challenge transition, typed inbox message, and participant invalidations
commit in one PostgreSQL transaction. Retrying the same completed operation
returns its existing representation without another effect; competing or
directionally invalid transitions return `game_challenge_transition_unavailable`.

Expired acceptance returns `game_challenge_expired`; unavailable targets,
games, and pending capacity return `challenge_target_unavailable`,
`game_unavailable`, and `game_challenge_limit_reached`. Creation or a first
terminal transition appends `game_challenge_changed` and
`conversation_changed` for both personas. Acceptance also receives the
`game_session_changed` events created by the existing session primitive. All
challenge routes and errors carry `Cache-Control: no-store`.

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
  "authority": "platform_compiled",
  "provider_release_id": null,
  "availability": null,
  "completed_at": null,
  "state": {},
  "presentation": null,
  "result": null,
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
`invalid_pagination`. Sessions are created by an accepted two-human challenge
or the owner-scoped solo start route. Completed sessions remain in inventory
with `status: "completed"`, a terminal `completed_at`, the final state, and
their immutable participant history.

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
    "kind": "play",
    "action": "strike"
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
  "status": "active",
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
the snapshot, revision, active/completed status, completion timestamp, private
replay receipt, update timestamp, and one minimal `game_session_changed` event
per participant in one PostgreSQL transaction. The command that completes a
game returns `status: "completed"`; its exact retry returns the original final
receipt even if the compiled rules are unavailable. Any new command on that
completed session returns 409 `game_completed`. Conflicts, replays, rejections,
and rollbacks append no event. All command responses carry
`Cache-Control: no-store`.

## Apply a signed Game Cartridge action

`POST /v1/personas/{persona_id}/game-sessions/{game_session_id}/cartridge-actions`

This 32 KiB participant-private route is the only gameplay path for an action
emitted by a trusted cartridge plan. It rejects unknown fields and accepts only
the selected session's current revision, exact pinned archive digest, accepted
signed screen, declared gameplay action, object payload, and a session-wide
idempotency UUID:

```json
{
  "idempotency_key": "8f5d8f1d-48df-4f5a-b6e7-ad26eb30ae88",
  "expected_revision": 0,
  "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "screen_id": "chronicle",
  "action": "enter",
  "payload": {}
}
```

The server owner-scopes the acting persona, requires participation in the exact
bound session, verifies current revision and digest, re-resolves the signed
release under active-session policy, and validates the action against the
exact signed `screen_id`. New clients always send this field. Its omission is a
compatibility path that means only the signed entry screen for older clients;
every new durable admission records an explicit screen. Button actions accept
exactly `{}`; Grid actions accept
only bounded integer `column` and `row` from the signed grid. The host—not QML
or the cartridge—translates that intent into the session's existing
`platform_compiled` or `registered_provider` command.

`navigate.<screen_id>` is reserved for trusted local presentation navigation
and is never accepted as gameplay. The verified cartridge must require
`presentation.navigation.v1`; each such action has an empty payload, names an
existing signed target, and is emitted by one unique Button. The companion
returns the current screen, entry screen, and accepted mapping alongside the
unchanged `omarchygs.render-plan/v1`; trusted QML may navigate cyclic screens
without a server or provider request.

Authorization is durably recorded before compiled execution or provider I/O.
An exact retry reuses the admitted host command even if the release is later
suspended or revoked; a changed actor, revision, digest, screen, action, or payload is
an idempotency conflict, and a fresh post-transition action is denied. Success
returns the existing command receipt plus the confirmed pinned digest:

```json
{
  "game_session_id": "...",
  "revision": 1,
  "status": "completed",
  "state": {},
  "authority": "registered_provider",
  "provider_release_id": "...",
  "availability": "completed",
  "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}
```

Malformed input returns 422 `invalid_game_command`; absent or non-participant
sessions return 404 `game_session_not_found`; stale revisions return 409
`game_revision_conflict`; changed replays return 409
`game_idempotency_conflict`; completed sessions return 409 `game_completed`;
and any pin, evidence, signed action, or lifecycle denial returns 409
`session_cartridge_unavailable`. All responses are `Cache-Control: no-store`.

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
    },
    {
      "type": "game_challenge_changed",
      "cursor": 46,
      "game_challenge_id": "edac755e-5a0b-4c9d-a652-f20a310fd22d",
      "created_at": "2026-08-24T20:00:03.000Z"
    }
  ],
  "next_cursor": 46,
  "has_more": false,
  "reset_required": false
}
```

The other event types are `connection_requests_changed` and `blocks_changed`.
Game-session and game-challenge events carry only their participant-authorized
resource UUID, never state, challenge details, or participant data. Events
never contain message bodies, profiles, account identity, read counts, block
direction, or credentials. Each persona retains only its newest 10,000 events.
If `after` is older than retained history, the server returns an empty
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
