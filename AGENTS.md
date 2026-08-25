# Omarchy Gaming System — Codex project guide

Read [CONSTITUTION.md](CONSTITUTION.md) before modifying this repository. It
defines the binding gates, phases, architecture boundaries, and local knowledge
process.

## Product

Omarchy Gaming System is an API-first social gaming platform:

- `crates/server` — Rust/Axum server and PostgreSQL integration
- `client/qml` — keyboard-first Qt/QML connector for Omarchy
- `migrations` — forward-only PostgreSQL schema
- `docs/product-charter.md` — first-playable product scope

## Work routing

- For non-trivial feature, fix, migration, infrastructure, or workflow changes,
  use the repository skill `$omarchy-workflow`. If the current session started
  before the skill was added or changed, read
  `.agents/skills/omarchy-workflow/SKILL.md` directly or restart Codex.
- For product exploration that should not create a ticket or application code,
  use `$omarchy-brainstorm`.
- Answer read-only questions and perform read-only diagnosis directly.
- A user may explicitly waive phase ceremony for a small change. The quality,
  test, secret, and delivery-receipt rules still apply.
- Never keep more than one active spec/notes pair. Resume or disposition an
  existing active pipeline before opening another.
- Do not commit, push, open a pull request, or otherwise publish changes unless
  the user explicitly authorizes that delivery action.

The normal pipeline is:

```text
recall → plan → design → implement → inspect → validate → complete → delivery
```

## Quality commands

```bash
bin/gate.sh --fast   # short development loop; no receipt
bin/gate.sh --diff   # full current delivery loop; writes a receipt
./scripts/dev.sh     # PostgreSQL + Rust server + visible QML client
```

Run Cargo commands sequentially. Do not terminate another Cargo process.

## Local knowledge

Before designing or implementing:

1. Search `docs/planning/knowledge/INDEX.md` for relevant `PR-`, `BF-`, and
   `AD-` entries.
2. Read `openwiki/quickstart.md` and the affected OpenWiki pages when they
   exist.
3. Read the nearest notes under `docs/planning/pipeline/completed/`.
4. Read relevant `docs/architecture/*.md`.
5. Use CodeGraph to inspect relevant symbols, flows, callers, and blast radius;
   directly inspect unsupported file types and tests before changing code.

During Phase 5, run the `$openwiki` lifecycle to reconcile durable engineering
documentation, then record lessons in the current AAR and append every new ID
to the knowledge register.

## Codex enforcement

Repository lifecycle hooks live in `.codex/hooks.json` and `.codex/hooks/`.
Codex runs project hooks only after the project and exact hook definitions have
been reviewed and trusted. Project MCP servers live in `.codex/config.toml` and
use pinned, ignored local installs prepared by `scripts/setup-pipeline-tools.sh`.
Hooks require worktree-bound CodeGraph evidence at design and inspection plus a
completed OpenWiki lifecycle at completion. Hooks are guardrails;
`bin/gate.sh` and its worktree-bound receipt are the delivery proof.

<!-- OPENWIKI:START -->

## OpenWiki

This repository has a generated `openwiki/` evidence index. It is optional just-in-time context, not required startup reading.

- Treat source code and tests as authoritative. A brief's unknowns and review items are verification gaps, not automatic requirements.
- Prefer the narrowest quiet validation that proves the changed behavior. Preserve complete failure output.

The scheduled OpenWiki GitHub Actions workflow refreshes the repository wiki. Do not hand-edit generated OpenWiki pages unless explicitly asked; prefer updating source code/docs and letting OpenWiki regenerate.

<!-- OPENWIKI:END -->
