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

The API can also be checked directly:

```bash
curl http://127.0.0.1:8080/health
```

Expected response:

```json
{"service":"omarchy-gaming-system","version":"0.1.0","status":"ok","database":"ok"}
```

`OGS_BIND_ADDRESS` configures the listener. During the local rebrand
transition, `BBS_BIND_ADDRESS` remains a lower-priority fallback for existing
developer environments; new configuration should use only the gaming-system
name.

The server also requires `OGS_MFA_ENCRYPTION_KEY`, a base64url-encoded 32-byte
key used to encrypt opt-in TOTP secrets. `scripts/dev.sh` creates and reuses a
mode-0600 development key under ignored `.dev/` state when the variable is
absent. Deployments must supply and back up their own stable key; losing it
locks enrolled accounts out of TOTP verification.

Register an account through the versioned JSON API:

```bash
curl --header 'Content-Type: application/json' \
  --data '{"username":"player_one","password":"TEST-ONLY-change-this-passphrase"}' \
  http://127.0.0.1:8080/v1/accounts
```

See [the HTTP API reference](docs/api.md) for validation rules and response
contracts. Registration creates a private account only; device sessions and
public personas remain separate resources.

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

The first compiled-game foundation is also present. `GET /v1/games` exposes a
stable public catalog, which is intentionally empty until the first playable
game ships. The database can persist exact-version, revision-zero game sessions
with ordered persona participants and deterministic JSON state through a
trusted internal transaction boundary. Participating personas can list or read
those sessions without exposing account ownership. Participant commands execute
only through the stored exact rules version, use optimistic revisions and a
session-wide idempotency UUID, and atomically persist the next snapshot,
private replay receipt, and minimal participant invalidations. Public creation,
challenges, production game rules, and bots remain later roadmap slices.

## Development checks

```bash
./scripts/check.sh
```

This is an alias for the canonical fast gate. It runs Rust formatting, Clippy,
unit tests, documentation, Compose validation, hook tests, shell syntax, and
whitespace checks. The non-fast gate also runs isolated PostgreSQL integration
tests.

Run the full server/database/QML health path without opening a window:

```bash
./scripts/dev.sh --smoke-test
```

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
