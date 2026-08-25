---
title: Codex-native agent workflow — notes
pipeline_id: 2720b6c4-0b38-48be-838f-0c8a99b21ac1
---

# Codex-native agent workflow — running notes

## Phase 1 — Plan

- User decision: Omarchy BBS is Codex-only; all agent workflow and enforcement
  must be Codex-native.
- Recall: the prior pipeline established one-active-ticket discipline, local
  memory, EARS criteria, a real vertical-slice gate, and a content-bound receipt.

## Phase 2 — Design

- Architecture: Codex loads project guidance from `AGENTS.md`, repository skills
  from `.agents/skills`, and trusted project hooks from `.codex/hooks.json`.
- File manifest: replace the previous project guide and hook tree; update the
  gate library, structure check, hook self-tests, README, constitution, ADR,
  roadmap, and historical workflow records.
- Enforcement: file-edit hooks use active-spec status, commit hooks use the
  gated worktree hash, and stop hooks verify completion claims plus secrets.
- Compatibility: transcript parsing and previous-agent task-count enforcement
  are removed because those interfaces are not Codex lifecycle contracts.

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Required-file audit and skill validator |
| REQ-002 | Phase and commit receipt self-tests |
| REQ-003 | Stop-claim and secret self-tests |
| REQ-004 | Residual-integration audit and `bin/gate.sh --fast` |

## Phase 3 — Implement

- Built: replaced the old project guide and command tree with `AGENTS.md`,
  `$omarchy-workflow`, `$omarchy-brainstorm`, `.codex/hooks.json`, four
  worktree/state-based hooks, shared helpers, and rewritten self-tests.
- Updated: the gate's gated-path inventory, shell and secret checks, pipeline
  structure audit, README, constitution, ADR, roadmap, templates, and prior
  workflow records.
- Deviations: task-count and read-history checks were not ported because their
  previous transcript/tool interfaces are not Codex contracts. The workflow
  skill carries recall and planning duties; durable phase state and receipts
  carry enforcement.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness | A literal rename would leave hooks parsing an incompatible transcript and file-edit schema. | high | Replaced transcript parsing with active-spec status, `apply_patch` input, last-message claims, and worktree receipt checks. |
| 2 | Delivery evidence | The first residual-integration audit used `git grep`, which would omit newly created untracked files. | medium | Reused the standing untracked-file lesson and changed the audit to recursively inspect all non-generated workspace content. |
| 3 | Enforcement | Codex project hooks require review/trust and are guardrails rather than a complete security boundary. | — | Documented trust behavior in `AGENTS.md` and README; retained `bin/gate.sh` as the independent load-bearing proof. |
| 4 | Scope | The prior notes implied a remote CI confirmation, but GitHub still has no `main`. | low | Corrected the record and added `BUL-001-initial-push-pending`; remote delivery remains out of scope without authorization. |

## Phase 4 — Validate

- Both repository skills passed the bundled skill validator when invoked with
  Python; the validator file itself lacks executable mode on this machine.
- Bash syntax, hook JSON shape, the pipeline structure check, and all isolated
  hook behavior tests passed.
- `bin/gate.sh --diff` passed all 11 checks: Rust formatting, Clippy, three
  tests, rustdoc, Compose, shell syntax, pipeline structure, secret scan, hook
  self-tests, whitespace, and the PostgreSQL/Rust/QML smoke path.
- The live QML smoke emitted non-fatal headless EGL warnings and passed. No
  tests or failures were skipped. A matching worktree receipt was written.

## Phase 5 — Complete

| Requirement | Verdict | Evidence |
|---|---|---|
| REQ-001 | satisfied | `AGENTS.md` plus two valid repo skills in `.agents/skills`; structure audit passed. |
| REQ-002 | satisfied | Self-tests proved pre-design edit denial, active-pipeline commit denial, missing/stale receipt denial, and matching/staged receipt allowance. |
| REQ-003 | satisfied | Self-tests proved false Phase 4/5 claims are stopped and changed-file secret fixtures are detected. |
| REQ-004 | satisfied | Obsolete files and content were removed; recursive residual audit and the canonical gate passed. |

- Docs updated: AGENTS guide, constitution, README, ADR-0001, roadmap,
  bulletins, pipeline templates, and prior workflow records.
- AAR-002 submitted; no new durable failure/prevention IDs were required. The
  existing untracked-file prevention rule directly caught the inspect issue.
- Ticket moved to `closed/`; spec/notes pair moved to `completed/`; active
  pipeline is empty.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first obsolete-integration audit omitted untracked files. | `git grep` only inspected tracked worktree content. | Replaced it with a recursive non-generated workspace scan. | `PR-omarchy-bbs-quality-gates-include-untracked-001` |
