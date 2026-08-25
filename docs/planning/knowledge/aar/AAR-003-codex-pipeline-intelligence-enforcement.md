---
aar: AAR-003-codex-pipeline-intelligence-enforcement
ticket: TICKET-003
pipeline: codex-pipeline-intelligence-enforcement
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-003-codex-pipeline-intelligence-enforcement

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-bbs-agent-work-pipeline-001` | Read ADR-0001 and the completed Codex migration notes. | Yes — kept enforcement repository-native and retained the independent delivery gate. |
| `PR-omarchy-bbs-quality-gates-include-untracked-001` | Read AAR-002 and the knowledge register. | Yes — tool and residual audits include untracked committable content. |
| CodeGraph and OpenWiki upstream source | Inspected the pinned releases before designing integration. | Yes — exposed telemetry defaults, project-level Codex support, and OpenWiki's incompatible second guide file. |

## What happened

CodeGraph and OpenWiki were added as pinned, project-local Codex integrations.
The workflow now requires worktree-bound CodeGraph evidence in design and
inspection and a successful OpenWiki lifecycle at completion. Focused setup
testing exposed that the registry artifact sharing OpenWiki's source version did
not yet contain the Codex lifecycle, so setup moved to a pinned Git revision and
a verified local build. Review then removed Bash-text spoofing, a repeated-stop
escape, relative-MCP-path assumptions, and over-broad patching.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-bbs-openwiki-release-source-drift-001` | A registry artifact and the upstream repository reported the same version while exposing materially different integration capabilities. | First bootstrap smoke could not find the expected lifecycle server. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Structural graph coverage hints supplement but never replace direct test inspection and executed gate evidence. | CodeGraph returned the relevant embedded tests as source while simultaneously reporting no covering tests for the flow. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-bbs-codex-pipeline-intelligence-001` | Require pinned CodeGraph analysis during design/inspection and a claims-backed OpenWiki lifecycle during completion, enforced through Codex tool hooks and gated-worktree receipts. | `docs/architecture/adr-0001-agent-work-pipeline.md` |

## Effectiveness

5/5. Prior worktree-receipt and untracked-file rules directly shaped the
evidence design. Inspecting upstream source before installation exposed two
Codex-only incompatibilities, and focused bootstrap testing caught source versus
artifact drift before the integration could be declared ready. The first real
OpenWiki lifecycle then exercised Grounded Claims, deterministic finalization,
PostToolUse evidence, and the canonical gate rather than relying on mocks alone.
