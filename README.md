# Omarchy BBS

An API-first social gaming BBS with a Rust server and a keyboard-first QML
connector for Omarchy.

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
{"service":"omarchy-bbs","version":"0.1.0","status":"ok","database":"ok"}
```

## Development checks

```bash
./scripts/check.sh
```

This is an alias for the canonical fast gate. It runs Rust formatting, Clippy,
tests, documentation, Compose validation, hook tests, shell syntax, and
whitespace checks.

Run the full server/database/QML health path without opening a window:

```bash
./scripts/dev.sh --smoke-test
```

The delivery gate combines both levels and writes a worktree-bound commit
receipt:

```bash
bin/gate.sh --diff
```

## Claude work pipeline

Claude Code reads [CLAUDE.md](CLAUDE.md) and the binding
[CONSTITUTION.md](CONSTITUTION.md). Feature work flows through:

```text
/work → /pipeline:plan → /pipeline:design → /pipeline:implement
      → /pipeline:inspect → /pipeline:validate
      → /pipeline:complete → /commit
```

The repository keeps tickets, active and completed pipeline narratives,
architecture decisions, bulletins, and after-action lessons under
`docs/planning/`. Claude hooks enforce phase ordering, knowledge recall, test
execution, secret scanning, and a matching delivery receipt before code
commits. The canonical gate remains usable without Claude.

## Layout

```text
client/qml/        QML connector
crates/server/     Rust API service
docs/              Product and architecture decisions
migrations/        PostgreSQL schema migrations
scripts/           Local development commands
.claude/           Claude commands and enforcement hooks
bin/gate.sh        Canonical delivery gate
```
