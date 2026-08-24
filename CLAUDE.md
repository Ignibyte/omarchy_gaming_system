# Omarchy BBS — Claude development guide

Read [CONSTITUTION.md](CONSTITUTION.md) before pipeline work. It defines the
binding gates, phases, architecture boundaries, and local knowledge process.

## Product

Omarchy BBS is an API-first social gaming BBS:

- `crates/server` — Rust/Axum server and PostgreSQL integration
- `client/qml` — keyboard-first Qt/QML connector for Omarchy
- `migrations` — forward-only PostgreSQL schema
- `docs/product-charter.md` — first-playable product scope

## Work pipeline

Code ships through:

```text
/work → /pipeline:plan → /pipeline:design → /pipeline:implement
      → /pipeline:inspect → /pipeline:validate
      → /pipeline:complete → /commit
```

| Command | Result |
|---|---|
| `/work` | Preflight, bulletins, and local knowledge recall |
| `/pipeline:plan` | Ticket, EARS spec, notes, and open AAR |
| `/pipeline:design` | Architecture/file manifest and regression plan |
| `/pipeline:implement` | Code matching the approved design |
| `/pipeline:inspect` | Adversarial review and disposition ledger |
| `/pipeline:validate` | Tests run and `bin/gate.sh --diff` green |
| `/pipeline:complete` | AC audit, docs, submitted AAR, ticket/archive |
| `/commit` | Fresh receipt, staged review, commit, optional push/PR |

Use `/brainstorm` for research and rough ideas that are not ready to enter the
pipeline. A user may explicitly waive the phase ceremony for a small task, but
the quality gate and evidence rules remain.

## Quality commands

```bash
bin/gate.sh --fast   # short development loop; no commit receipt
bin/gate.sh --diff   # delivery loop; writes the commit receipt
./scripts/dev.sh     # PostgreSQL + Rust server + visible QML client
```

Never run cargo commands concurrently and never kill a running cargo process.

## Local knowledge

Before designing or implementing:

1. Search `docs/planning/knowledge/INDEX.md` for relevant `PR-`, `BF-`, and
   `AD-` entries.
2. Read the nearest notes under `docs/planning/pipeline/completed/`.
3. Read relevant `docs/architecture/*.md`.
4. Inspect the actual callers and tests before changing code.

Record durable lessons in the current AAR and append every new ID to the
knowledge register at completion.
