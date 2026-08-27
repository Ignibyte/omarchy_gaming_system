# Omarchy Gaming System

Omarchy Gaming System—**OmarchyGS** for short—is an API-first social gaming
system with a Rust server and a keyboard-first QML connector for Omarchy.
Connections, private inboxes, challenges, and server-authoritative games are
the core experience; a public message board may be considered later but is not
the product's current focus.

The first development slice proves the full local connection:

```text
QML connector -> GET /health -> Rust/Axum -> PostgreSQL
```

See [the product charter](docs/product-charter.md) for the first playable scope
and architectural commitments.

## Install the player client on Omarchy

The private-alpha client now builds as a native Arch package. From a reviewed
checkout on Omarchy:

```bash
./scripts/build-client-package.sh
cd target/packages
sha256sum --check omarchy-gaming-system-client-*.pkg.tar.zst.sha256
pacman -Qip omarchy-gaming-system-client-*.pkg.tar.zst
sudo pacman -U ./omarchy-gaming-system-client-*.pkg.tar.zst
```

Launch **Omarchy Gaming System** from the application menu or run
`omarchygs`. The x86_64 package contains the trusted QML player client and its
loopback Rust cartridge companion. It uses Omarchy's `qt6-declarative` runtime
and does not install the community server, PostgreSQL, Cargo, or Docker. These
locally built private-alpha artifacts are unsigned, so inspect the package and
verify its SHA-256 sidecar through the same trusted channel as the source
before installing it.

See [client installation](docs/client-installation.md) for server selection,
updates, removal, provenance, and the current trust boundaries.

## Quick start on Omarchy

Requirements currently present on a standard development machine:

- Docker with Compose
- `mise`
- Qt 6 QML runtime (`qml6`)
- `curl`
- `jq`
- OpenSSL
- Python 3

Start PostgreSQL, the Rust server, and the QML connector with one command:

```bash
./scripts/dev.sh
```

Closing the QML window stops the development server. PostgreSQL remains running
so later starts are quick. Stop it explicitly with:

```bash
docker compose down
```

The persistent **EXIT** button closes the client through the normal window
lifecycle without signing out or revoking its durable device session. From a
separate terminal at the repository root, the development client can also be
closed explicitly with:

```bash
pkill -TERM -f "^qml6 $(pwd)/client/qml/Main.qml$"
```

The API can also be checked directly:

```bash
curl http://127.0.0.1:8080/health
```

Expected response:

```json
{"service":"omarchy-gaming-system","version":"0.1.0","status":"ok","database":"ok"}
```

Players identify and negotiate a community through the separate public
discovery endpoint:

```bash
curl http://127.0.0.1:8080/.well-known/omarchygs
```

It returns the database-backed stable server UUID, `OGS_SERVER_NAME`, protocol
version, and implemented capabilities. The QML client can save up to 16 such
public profiles, connect once without saving, or deliberately select/remove a
saved server. It persists no password, invitation, bearer, MFA, account, or
persona authority. A saved origin that later reports another UUID fails closed
until the old profile is removed.

`OGS_BIND_ADDRESS` configures the listener. During the local rebrand
transition, `BBS_BIND_ADDRESS` remains a lower-priority fallback for existing
developer environments; new configuration should use only the gaming-system
name.

`OGS_SERVER_NAME` is the public 1–64 character community label and defaults to
`OmarchyGS Community`. Changing it does not change the server UUID. The UUID is
part of PostgreSQL state and must travel with normal database backup/restore;
it is not derived from a hostname, TLS certificate, or display name.

The server also requires `OGS_MFA_ENCRYPTION_KEY`, a base64url-encoded 32-byte
key used to encrypt opt-in TOTP secrets. `scripts/dev.sh` creates and reuses a
mode-0600 development key under ignored `.dev/` state when the variable is
absent. Deployments must supply and back up their own stable key; losing it
locks enrolled accounts out of TOTP verification.

Account creation is invitation-only. A trusted server operator first issues a
single-account code through the database-local procedure in the
[private-alpha runbook](docs/operators/private-alpha.md). Present that code
only in the versioned JSON registration body:

```bash
curl --header 'Content-Type: application/json' \
  --data '{"invite_code":"ogsi_<operator-issued-code>","username":"player_one","password":"TEST-ONLY-change-this-passphrase"}' \
  http://127.0.0.1:8080/v1/accounts
```

See [the HTTP API reference](docs/api.md) for validation rules and response
contracts. PostgreSQL retains only the invitation digest, and successful
registration atomically consumes it. Registration creates a private account
only; device sessions and public personas remain separate resources.

After registration, `POST /v1/sessions` exchanges the username and password for
a revocable device Bearer token. The API reference documents creation, listing,
expiry, and revocation; raw tokens are returned only once and should be handled
as secrets.

Accounts may opt into authenticator-app TOTP. Enrollment requires an existing
device session and the account password, returns ten one-time recovery codes at
confirmation, and changes future password logins into short-lived MFA
challenges. Up to ten independent live challenges support overlapping devices
without letting a later password login invalidate an earlier challenge. TOTP
improves password-compromise resistance but is not
phishing-resistant; passkeys remain a possible later authenticator.

Authenticated accounts can create and manage multiple personas through
`/v1/personas`. Exact public handle lookup exposes only the allowlisted profile
shape, while account ownership remains private. The API reference documents
the validation, privacy, and owner-authorization contract.

The shipped QML connector now provides the first keyboard-first player access
slice rather than only a health probe. It accepts HTTPS servers plus loopback
HTTP for development, distinguishes configuration/offline/protocol states,
supports invitation-only account registration, password login, existing TOTP
or recovery-code challenges, and owned-persona creation or selection.
Invitation, password, and factor input are masked and cleared after submission;
bearer and MFA challenge tokens live only in process memory and are erased on
logout, endpoint changes, invalid sessions, or protocol failure. Persistent
sign-in waits for a reviewed OS keyring boundary. The selected persona can now open keyboard-first social and
private-inbox screens through that same credential owner. Exact-handle
requests and reports, connection and private-block lifecycle, bounded ascending
history, body-only sends, and monotonic read receipts use explicit durable REST
refresh. Exact response allowlists and plain-text rendering reject protocol
confusion, while a validated `invalid_session` clears the full account/persona
authority boundary.

Owned personas can now send and accept idempotent connection requests, remove
connections, and privately block or unblock another persona. Pair mutations
are serialized in PostgreSQL so a block atomically removes pending or accepted
state, and each persona's pending inventory is capped at 100 requests per
direction. Each accepted pair also has one durable private conversation with
bounded user messages, typed server-authored acceptance events,
conversation-local stable history, and monotonic per-persona unread state.
Disconnecting or blocking preserves history while rejecting new sends; live
changes use a persona-local durable cursor feed plus an authenticated WebSocket
wake-up channel. The socket carries no game or inbox data: clients recover from
missed or duplicate hints through the bounded REST feed and authoritative
resource endpoints. Live sockets retain no raw credentials, recheck session
authority without extending idle expiry, reject client payloads above 1 KiB,
and enforce persona, account, and process connection budgets.

Players can file one of four bounded persona-report categories from the Social
screen. Reports are private to the server operator and return only a retry-safe
receipt; they do not notify the subject or enter the sync feed. A separate
database-local `omarchygs-admin` command lists the report queue, resolves or
dismisses reports, and reversibly suspends accounts. Suspension revokes all
live device sessions in the same transaction, reactivation never resurrects
them, and every operator mutation appends immutable audit. See
[operator safety and platform recovery](docs/operators/operator-safety-and-recovery.md)
for the command and backup/restore procedure.

The production catalog now includes **Signal Siege v1**, a deterministic
asynchronous duel against a server-side bot. An owned persona can start a
bounded, idempotent solo match, submit one human action per round, and receive
the simultaneous bot response until the durable session records its terminal
outcome. No bot account or persona is created. Exact start and command replays
remain available after registry drift, while new commands cannot mutate a
completed game. Participating personas can list or read exact-version sessions
without exposing account ownership; every accepted transition atomically
persists its snapshot, one-step revision, status, private replay receipt, and a
minimal sync invalidation. Connected, unblocked personas can also create
exact-version inbox challenges for games that admit two humans. Challenge
history, server-owned expiry, retry/race safety, typed inbox events, and
reconnect-safe invalidations are durable. The QML game screens can start solo
Signal Siege, create/respond to challenges, play the two-person versus rules,
and recover the exact terminal history through the authoritative REST paths.

## Synchronize and curate Game Cartridges

The database-local administrator command can synchronize one pinned
marketplace and curate a server-owned cartridge catalog. Marketplace review
does not activate a game: synchronization verifies and stages immutable bytes,
then a separate audited command admits one exact release for this community.

Provision an existing Linux directory owned by the account running the command
and inaccessible to group or other users, then configure the exact HTTPS
origin, Ed25519 public-key document, DER TLS root, and store root:

```bash
install -d -m 0700 /var/lib/omarchygs/cartridges
export OGS_MARKETPLACE_ORIGIN=https://marketplace.example.com/v1/
export OGS_MARKETPLACE_PUBLIC_KEY=/etc/omarchygs/marketplace-public.json
export OGS_MARKETPLACE_TLS_ROOT_DER=/etc/omarchygs/marketplace-root.der
export OGS_CARTRIDGE_STORE_ROOT=/var/lib/omarchygs/cartridges
```

The marketplace origin must be a canonical HTTPS domain origin. The sync
client accepts only the configured TLS root, public DNS destinations, relative
same-origin release paths, exact successful responses, and bounded bodies; it
uses no proxy, redirect, referer, decompression, or connection reuse.

Build the local command, synchronize, and inspect the complete operator
inventory:

```bash
cargo build -p omarchy-gaming-system-server --bin omarchygs-admin
DATABASE_URL="$DATABASE_URL" target/debug/omarchygs-admin marketplace-sync
DATABASE_URL="$DATABASE_URL" target/debug/omarchygs-admin cartridges
```

To activate an exact reviewed digest, place a bounded mode-0600 command in
`catalog-command.json`:

```json
{
  "idempotency_key": "8d8f9f79-539d-4fa3-80bd-1ca9ae111857",
  "game_key": "door-legends",
  "expected": {"state": "inactive"},
  "desired": {
    "state": "release",
    "archive_sha256": "<64-lowercase-hex-digest>"
  },
  "actor": "oncall-sysop",
  "reason": "Admit the reviewed release for this community"
}
```

Apply it with:

```bash
DATABASE_URL="$DATABASE_URL" target/debug/omarchygs-admin \
  catalog-apply ./catalog-command.json
```

Deactivation uses the current exact release as `expected` and
`{"state":"inactive"}` as `desired`. An upgrade or explicit rollback names
the current digest in `expected` and the chosen digest in `desired`; exact
replay returns the original receipt, while stale intent conflicts. A
marketplace suspension, removal, or incompatibility makes a selected release
ineffective without falling back to another version. Operators must explicitly
choose recovery after a later valid snapshot.

When `OGS_MARKETPLACE_PUBLIC_KEY` and `OGS_CARTRIDGE_STORE_ROOT` are both
present at server startup, discovery advertises exact acquisition support and
the authenticated acquisition route serves only the selected digest from the
retained signed snapshot and immutable store. Partial distribution
configuration fails startup; no alternate release is substituted.

The client package verifies that exact server admission, marketplace snapshot,
publisher release, lifecycle policy, SDK compatibility, archive, conformance,
and attestation through its loopback Rust companion. Marketplace verification
uses a client-controlled public key provisioned independently from the selected
server; a response-supplied replacement key is rejected. Verified bytes enter a
shared content-addressed cache, while read-only mount records remain isolated
by server UUID. Installation and update are explicit, failures retain the old
mount, and removal deletes only the selected profile mount—not remote state or
shared immutable bytes. Binding a mounted render plan to a live game session
and replacing the current platform-owned game screens remain a later slice.

See [owner-operated servers](docs/operators/owner-operated-servers.md) and
[operator safety and platform recovery](docs/operators/operator-safety-and-recovery.md)
for the authority, lifecycle, and backup boundaries.

## Development checks

```bash
./scripts/check.sh
```

This is an alias for the canonical fast gate. It runs Rust formatting, Clippy,
unit tests, documentation, Compose validation, hook tests, shell syntax, and
whitespace checks. The non-fast gate also runs isolated PostgreSQL integration
tests.

Run the full server/database/QML player path without opening a window:

```bash
./scripts/dev.sh --smoke-test
```

The smoke includes deterministic hostile HTTP fixtures, keyboard-only social,
report, and inbox interactions, a real locally issued invitation plus QML
registration/persona creation, an
enrolled MFA recovery login, and a migrated two-account QML connection/
conversation/message/report path before the API/game/social/reconnect checks
complete.

The non-fast delivery gate also proves a custom-format platform backup and
isolated restore, compares every application table, and rejects a
pre-suspension token against the restored production server:

```bash
./scripts/test-operator-recovery.sh
```

The invite-only admission rehearsal owns an isolated generated database and
drives the real operator CLI, production server, account registration/replay,
ordinary sign-in, revocation, metadata-only inventory, and audit boundary:

```bash
./scripts/test-private-alpha.sh
```

This deterministic rehearsal is software evidence, not a claim that the
external two-clean-installation acceptance run has occurred.

The delivery gate combines both levels and writes a worktree-bound commit
receipt:

```bash
bin/gate.sh --diff
```

## Codex work pipeline

Codex reads [AGENTS.md](AGENTS.md) and the binding
[CONSTITUTION.md](CONSTITUTION.md). Repository skills guide work through:

```text
recall → plan → design → implement → inspect → validate → complete → delivery
```

The repository keeps tickets, active and completed pipeline narratives,
architecture decisions, bulletins, and after-action lessons under
`docs/planning/`. The `$omarchy-workflow` skill runs or resumes non-trivial
work, while `$omarchy-brainstorm` explores ideas without opening a ticket.
Trusted project hooks enforce design readiness, completion claims, secret
scanning, and a matching delivery receipt before code commits. The canonical
gate remains usable by Codex, humans, and CI.

CodeGraph supplies indexed source topology during design and inspection.
OpenWiki maintains claims-backed engineering documentation during completion.
Prepare their exact project-local versions with:

```bash
scripts/setup-pipeline-tools.sh
```

Generated packages and the CodeGraph SQLite index stay under ignored `.dev/`
and `.codegraph/`. Both integrations run with telemetry disabled. The OpenWiki
checkout is patched in generated local state to expose only the Codex lifecycle,
maintain only `AGENTS.md`, and skip provider-driven scheduled workflows.

Codex reviews project-local hook and MCP definitions before running them. After
a fresh clone or a change to `.codex/hooks.json` or `.codex/config.toml`, review
and trust the definitions when Codex prompts, then restart Codex so both MCP
servers load. Runtime readiness can be checked with:

```bash
scripts/check-pipeline-tools.sh
```

## Layout

```text
client/qml/        QML connector
crates/server/     Rust API service
docs/              Product and architecture decisions
migrations/        PostgreSQL schema migrations
scripts/           Local development commands
.agents/skills/    Repository-scoped Codex workflow skills
.codex/            Codex MCP configuration and lifecycle hooks
openwiki/          Generated, claims-backed engineering wiki
bin/gate.sh        Canonical delivery gate
```
