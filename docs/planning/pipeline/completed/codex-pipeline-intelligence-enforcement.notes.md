---
title: Codex pipeline intelligence enforcement — notes
pipeline_id: b896c489-44bd-40e9-91c0-ff6c9ba3ecd3
---

# Codex pipeline intelligence enforcement — running notes

## Phase 1 — Plan

- User directive: add CodeGraph and OpenWiki to the pipeline with enforced use;
  all agent integration remains Codex-only.
- Recall: ADR-0001 and AAR-002 established repository-native Codex hooks,
  content-bound receipts, an offline canonical gate, and a recursive residual
  audit that includes untracked files.
- Preflight: no active pipeline existed; both CLIs were absent; Node 26 and npm
  11 satisfy OpenWiki's Node 22+ requirement.
- Upstream inspection: CodeGraph 1.5.0 supports Codex project config and a local
  SQLite index. OpenWiki 0.3.3 supports a Codex MCP lifecycle and project skill,
  but its repository setup currently writes two root agent guides.

## Phase 2 — Design

- Architecture: Codex loads two project MCP servers through wrapper scripts.
  CodeGraph supplies topology/impact evidence in design and inspection.
  OpenWiki supplies claims-backed durable documentation reconciliation at
  completion. A PostToolUse hook writes phase-scoped receipts under `.git`; the
  Stop hook checks those receipts before accepting phase claims.
- Security/privacy: both tools are pinned, installed into ignored local state,
  and launched with telemetry opt-outs. No credentials or generated databases
  are committed. Project hooks and MCP config remain subject to Codex trust.
- Compatibility: a deterministic bootstrap patch narrows OpenWiki's managed
  agent-guide list to `AGENTS.md`; the committed repository continues rejecting
  obsolete agent files and text.
- CodeGraph evidence: initialized 1.5.0 with telemetry disabled and ran an
  `explore` query over the server health-to-database flow. The graph returned
  the Axum route, database query, response constructor, server startup caller,
  config source, and blast-radius/test hints from current on-disk code.
- Tool limit: CodeGraph indexed Rust and YAML, not the shell hook implementation
  changed by this ticket. The design therefore uses CodeGraph for repository
  topology and augments it with direct inspection of the affected shell/JSON
  producers and consumers; enforcement records use without pretending it
  replaces unsupported-language review.
- Bootstrap correction: the registry artifact named OpenWiki 0.3.3 does not
  contain the Codex lifecycle server present in the inspected repository at the
  same version. Setup therefore pins commit `a525ed8`, installs its development
  dependencies in ignored local state, applies the Codex-only source patch, and
  builds that exact checkout.

### File manifest

| Path | Purpose |
|---|---|
| `.codex/config.toml` | Register project-scoped CodeGraph and OpenWiki MCP servers. |
| `.codex/hooks.json` | Wire usage recording into Codex PostToolUse. |
| `.codex/hooks/record-pipeline-tool-use.sh` | Recognize successful tool calls and issue phase-bound receipts. |
| `.codex/hooks/enforce-stop-claims.sh` | Require the correct fresh receipt for Phase 2, 3.5, and 5 claims. |
| `.codex/hooks/lib-hook-helpers.sh` | Compute and inspect pipeline tool receipt state. |
| `.agents/skills/omarchy-workflow/*` | Make the two tool calls mandatory in the phase method. |
| `.agents/skills/openwiki/*` | Provide the Codex-native OpenWiki lifecycle workflow. |
| `scripts/setup-pipeline-tools.sh` | Install, patch, verify, and initialize pinned local tools. |
| `scripts/mcp-codegraph.sh` | Launch CodeGraph MCP from local generated state. |
| `scripts/mcp-openwiki.sh` | Launch OpenWiki Codex MCP from local generated state. |
| `scripts/check-pipeline-tools.sh` | Report local tool versions, patch state, and graph readiness. |
| `scripts/check-pipeline.sh` | Validate committed wiring without third-party executables. |
| `scripts/selftest-hooks.sh` | Prove receipt issuance, staleness, and claim denial behavior. |
| `AGENTS.md`, `CONSTITUTION.md`, `README.md`, ADR-0001 | Document the enforced runtime and bootstrap contract. |
| Ticket/spec/notes/AAR/index records | Preserve local planning and completion evidence. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Bootstrap/check script smoke; exact package and MCP config assertions. |
| REQ-002 | Stop claim denied without design receipt, allowed after qualifying CodeGraph use. |
| REQ-003 | Receipt goes stale after a gated edit; inspect claim requires another qualifying use. |
| REQ-004 | Completion claim denied without a successful OpenWiki finish and allowed with it after archive. |
| REQ-005 | OpenWiki patch assertion plus the existing recursive residual audit. |
| REQ-006 | Pipeline structure check and canonical gate from a worktree whose tool cache is ignored. |

## Phase 3 — Implement

- Built: pinned local bootstrap and readiness checks; CodeGraph CLI/MCP and
  OpenWiki MCP launchers; Codex project MCP config; a Codex-native OpenWiki
  skill; phase-method updates; PostToolUse evidence recording; Stop-claim
  enforcement; new gated-path coverage; isolated regressions; and operator,
  constitution, and ADR documentation.
- Bootstrap verification: setup installed CodeGraph 1.5.0, cloned OpenWiki at
  `a525ed8`, applied both Codex-only source changes, built the lifecycle server,
  synchronized the existing graph, and passed the readiness check.
- Focused checks: Bash syntax, hook JSON parsing, all three skill validators,
  pipeline structure, tool readiness, and the isolated hook suite passed.
- Deviations: the design initially targeted the registry package for OpenWiki.
  Its 0.3.3 artifact lacked the GitHub repository's Codex lifecycle code, so the
  implementation pins and builds the inspected commit instead. This preserves
  the requested capability rather than accepting a misleading version match.
- Restart compatibility: Codex intentionally does not hot-load changed trusted
  hooks or MCP definitions. The sanctioned CodeGraph CLI fallback now records
  its own receipt only after a successful `explore`, so the pipeline can reach
  inspection without fabricating hook input; OpenWiki still requires a restart
  because its host lifecycle is intrinsically MCP-based.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Dependency correctness | The registry artifact named 0.3.3 lacked the requested Codex lifecycle found at the upstream repository's current commit. | high | Pin the inspected Git commit, build it locally, and verify the lifecycle output exists. |
| 2 | Portability | MCP config initially relied on Codex starting at the repository root. | medium | Resolve both launchers from `git rev-parse --show-toplevel`; `codex mcp list` parses and reports both enabled. |
| 3 | Evidence integrity | Recognizing a CodeGraph command inside arbitrary Bash text could accept an `echo` as use. | high | PostToolUse now accepts only the exact MCP tool; the fallback wrapper issues evidence itself only after successful `explore`. The self-test proves unrelated Bash cannot issue a receipt. |
| 4 | Enforcement | The inherited `stop_hook_active` escape let a repeated unsupported phase claim pass on the continuation turn. | high | Removed the bypass and proved a repeated claim remains blocked; an honest blocked handoff can stop without claiming success. |
| 5 | Supply-chain adaptation | A broad generated-source substitution could silently patch a changed upstream shape. | medium | Require either the exact upstream or already-patched line and reject any tracked source diff beyond the two reviewed Codex-only changes. |
| 6 | Coverage interpretation | CodeGraph surfaced the correct health flow and embedded test functions but still warned that the flow had no covering tests. | low | Treat graph coverage as a review hint and preserve direct test/gate verification; record this as a durable prevention rule. |

- Final CodeGraph pass: synchronized an already-current index and re-explored
  the Axum health → SQL query → response/startup flow after the last gated fix.
  The inspect receipt matches the current gated worktree.

## Phase 4 — Validate

- Tests run: all three repository skills passed the bundled validator; local
  setup and readiness passed twice, including idempotency; both MCP launchers
  cleanly handled a stdio EOF smoke; `codex mcp list` parsed both servers as
  enabled; pipeline structure and isolated hook regressions passed.
- Gate run: `bin/gate.sh --diff` passed all 11 checks: Rust formatting, Clippy,
  three unit tests, rustdoc, Compose, shell syntax, pipeline structure, secret
  scan, hook self-tests, whitespace, and PostgreSQL/Rust/QML smoke. It printed
  `GATE GREEN [diff]` and wrote a matching worktree receipt.
- Skips or pre-existing failures: none. The headless QML smoke emitted the
  existing non-fatal EGL warnings and passed.

## Phase 5 — Complete

- Acceptance-criteria audit:

| Requirement | Verdict | Evidence |
|---|---|---|
| REQ-001 | satisfied | Idempotent setup and readiness checks proved CodeGraph 1.5.0, OpenWiki `a525ed8`/0.3.3, telemetry opt-outs, Codex MCP config, the Codex-only patch, and a complete local graph. |
| REQ-002 | satisfied | Isolated hooks denied the design claim without evidence, ignored an errored tool call, and accepted a matching successful CodeGraph receipt. |
| REQ-003 | satisfied | Isolated hooks proved design evidence becomes stale after a gated edit and inspection needs a fresh successful CodeGraph call. |
| REQ-004 | satisfied | The first real OpenWiki init and a formatting-repair update both finished successfully; PostToolUse wrote a receipt matching this pipeline and gated worktree. |
| REQ-005 | satisfied | OpenWiki generated only `AGENTS.md` integration plus `openwiki/`; no obsolete guide or scheduled workflow exists, and the recursive residual audit passed. |
| REQ-006 | satisfied | Pipeline structure validation remained independent of generated tool binaries; the canonical gate passed every committed-wiring check. |

- Docs: initialized five claims-backed OpenWiki pages covering task routing,
  runtime, product boundaries, development/validation, and the Codex workflow.
  The first post-generation gate caught four trailing blank lines; a compliant
  OpenWiki update repaired them before the final gate.
- AAR: AAR-003 submitted with the artifact/source drift failure, advisory graph
  coverage rule, and pipeline-intelligence architecture decision registered.
- Archive: ticket closed and the active spec/notes pair moved to completed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The registry package lacked the inspected source tree's Codex lifecycle despite sharing its version. | Published artifact and repository HEAD had drifted. | Pin and build the inspected Git commit; verify lifecycle output. | `BF-omarchy-bbs-openwiki-release-source-drift-001` |
| 2 | Generated wiki pages failed the whitespace gate on their first pass. | Initial authored patches left an extra blank line at EOF. | Repair inside an OpenWiki update lifecycle and rerun finalization. | Keep generated Markdown inside the same canonical whitespace gate. |
