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

This runs Rust formatting, Clippy, tests, and Docker Compose validation.

Run the full server/database/QML health path without opening a window:

```bash
./scripts/dev.sh --smoke-test
```

## Layout

```text
client/qml/        QML connector
crates/server/     Rust API service
docs/              Product and architecture decisions
migrations/        PostgreSQL schema migrations
scripts/           Local development commands
```
