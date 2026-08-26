---
type: "Reference"
title: "Codex work pipeline"
openwiki_generated: true
sources:
  - id: openwiki-source-547854e5b8fbf28ebc0107fe
    resource: repo://.agents/skills/omarchy-workflow/references/phases.md
  - id: openwiki-source-0f2091252a9c3383cef44ad0
    resource: repo://.agents/skills/openwiki/SKILL.md
  - id: openwiki-source-d35335fb117842a01dcf6199
    resource: repo://.codex/hooks.json
  - id: openwiki-source-0c48ed5751117180c90b59f2
    resource: repo://.codex/hooks/enforce-commit-gate.sh
  - id: openwiki-source-f33c50e4e23c68fe7d1ede8b
    resource: repo://.codex/hooks/enforce-phase-gate.sh
  - id: openwiki-source-08555f2e71e5c713fd517049
    resource: repo://.codex/hooks/enforce-secrets.sh
  - id: openwiki-source-51bf78cd3a2dccc0b4b3d2bc
    resource: repo://.codex/hooks/lib-hook-helpers.sh
  - id: openwiki-source-8037e2358a2c4f9b2c722a11
    resource: repo://AGENTS.md
  - id: openwiki-source-0bb8016edf4f4744d3a09cf4
    resource: repo://bin/gate.sh
  - id: openwiki-source-0118ed911c8f8689e6c1c0a1
    resource: repo://bin/lib-gate.sh
  - id: openwiki-source-937883bc0b4873d5f0200c46
    resource: repo://CONSTITUTION.md
  - id: openwiki-source-d35448de763d92d5820dbaad
    resource: repo://scripts/check-pipeline-tools.sh
  - id: openwiki-source-ff3f60a113327d3006289ed7
    resource: repo://scripts/mcp-openwiki.sh
  - id: openwiki-source-4b4a00bf508755956f6f4de6
    resource: repo://scripts/selftest-hooks.sh
  - id: openwiki-source-037d6d04880b10f227f0ac17
    resource: repo://scripts/setup-pipeline-tools.sh
generated: {by: "codex", at: "2026-08-25T01:37:12.518Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-26T17:59:41.119Z
---

# Codex work pipeline

## Lifecycle

Every non-trivial feature, fix, migration, infrastructure, or workflow change
uses one active ticketed pipeline:

```text
recall → plan → design → implement → inspect → validate → complete → delivery
```

The active spec is the durable phase source of truth. Each pipeline has EARS
acceptance criteria, running notes, a ticket, and an AAR. Only one active
spec/notes pair may exist, so finish or disposition it before opening the next
roadmap slice.

## Evidence tools

CodeGraph is required once the design is stable and again after the last
implementation/inspection edit. Its topology and blast-radius results guide
review, but unsupported files and tests still require direct inspection.
Worktree-bound receipts prevent a stale CodeGraph pass from supporting a later
phase claim.

OpenWiki runs during completion. Its lifecycle prepares generated documentation,
maintains Grounded Claims, and finalizes indexes and provenance. The completion
claim requires a successful finish receipt for the same pipeline and gated
worktree.

The ignored pipeline-tool installation is itself fail-closed. CodeGraph setup
verifies repository-reviewed wrapper and platform SHA-512 values before a
script-disabled install, permits only the reviewed package pair and executable
link, checks the exact package tree, and records provenance that readiness
validates before execution. OpenWiki setup separately pins a reviewed commit
and exact upstream pnpm integrity, uses the frozen pnpm lock with install scripts
disabled, applies the reviewed Codex-only source patch, and records hashes for
the bootstrap, lock, patch, and build. Stale or missing provenance stops the
integration instead of silently rebuilding or executing different bytes.

## Hook guardrails

Before design passes, edit paths are canonicalized against the Git worktree.
Relative traversal, absolute aliases, and in-tree symlinks therefore resolve to
the same gated destination, while outside or unresolvable paths fail closed.
The repository self-tests those aliases alongside an ordinary documentation
edit.

The commit hook permits only exact standalone `git commit --help`,
`git commit -h`, and `git commit --dry-run` inquiries. Help or dry-run text
inside a compound command cannot exempt a mutating commit. These checks
strengthen the workflow boundary, but they do not change the rule that hooks
are guardrails and a matching worktree-bound gate receipt is delivery proof.

The shared delivery-state hash, commit classification, and changed-file secret
scanner consume Git paths as NUL-delimited exact bytes. A newline-bearing gated
file therefore changes the receipt and remains a gated commit input rather than
becoming a quoted pseudo-path. The scanner checks regular non-symlink changed
and untracked files for the project's high-signal provider families, including
OpenAI project and service-account keys, and passes every path after `grep`'s
option terminator. Hook self-tests cover stale receipt detection through a
newline path, secrets in newline, space, and dash-prefixed filenames, both
OpenAI families, an existing provider family, a clean scan, and a short
near-miss.

Project-local hooks still require the operator to review and trust the project
and exact hook definitions. Treat that grant as local code trust: repository
hooks are useful lifecycle guardrails, not a standalone security or delivery
boundary. The independently executed, worktree-bound diff gate remains the
canonical proof.

## Validation and delivery

`bin/gate.sh` is the load-bearing proof. Hooks enforce phase claims, secret
scanning, and matching receipts, but they are discipline guardrails rather than
a security boundary. A diff/full gate receipt becomes stale after any gated
edit.

Completion archives the spec and notes, closes the ticket, submits the AAR, and
updates the knowledge register. Delivery remains separate: do not commit, push,
or open a pull request unless the user explicitly authorizes that action.

## Durable memory

Use these sources in order of relevance:

- generated OpenWiki pages for engineering navigation;
- `docs/planning/knowledge/INDEX.md` and linked AARs for durable failures,
  prevention rules, and decisions;
- the nearest completed pipeline for implementation context;
- architecture and product documents for binding boundaries;
- source and tests as final authority.
