---
aar: AAR-043-local-only-automation-state-reconciliation
ticket: TICKET-043
pipeline: local-only-automation-state-reconciliation
status: submitted
opened: 2026-08-30
submitted: 2026-08-30
effectiveness: effective
---

# AAR-043-local-only-automation-state-reconciliation

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Ticket 039 notes and AAR | Search for hosted CI/CD and local-only automation | Yes — proved the workflow definition, local checker, hostile fixture, and prior remote disablement already existed. |
| Constitution §0 and §15 | Mandatory workflow recall | Yes — fixed local gate/receipt as delivery proof and prohibited hosted workflows. |
| `scripts/check-local-only-automation.sh` and server-module spike | Direct shell inspection | Yes — showed current enforcement already covers GitHub Actions and common equivalents. |
| Current GitHub API readback | Owner-directed external-state verification | Yes — exposed enabled Actions permissions despite an empty workflow inventory. |

## What happened

Ticket 043 reconciled three independent local-only automation surfaces. GitHub
Actions permissions had drifted back to enabled even though the repository had
zero workflows, so the setting was disabled through the authenticated API and
read back with an empty workflow inventory. The existing local checker,
pipeline wiring, and hostile temporary GitHub workflow fixture remained the
repository enforcement boundary; no network dependency entered the gate.

The first implementation corrected the stale managed `AGENTS.md` sentence
directly. Phase 5 immediately exposed that this was not durable: the pinned
OpenWiki lifecycle owned the block and regenerated the upstream claim that a
scheduled GitHub Actions workflow existed. The work returned to design. The
reviewed ignored-dependency patch now transforms that exact sentence alongside
its existing Codex-only agent-file and disabled-workflow changes, while setup
and readiness fail closed unless both source and compiled distribution contain
the local-lifecycle text and lack the hosted-workflow claim. Two verified
rebuilds and repeated fresh-process OpenWiki begins proved original-to-local,
already-local, process-reload, and lifecycle persistence behavior.

Generated OpenWiki workflow pages already described local-only delivery
accurately. Claims finalization removed two stale quickstart source projections
and completed without a prose rewrite. A fresh CodeGraph inspection, the
OpenWiki completion receipt, and every stage of `bin/gate.sh --diff` matched the
same gated worktree hash. Gate 23 explicitly rejected its hostile
`.github/workflows/ci.yml` fixture. Final GitHub readback again returned Actions
disabled and zero workflows.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-github-actions-permission-drift-001` | GitHub Actions permissions were enabled even though the repository and prior completion record required local-only automation. | Fresh GitHub repository settings readback. |
| `BF-omarchy-gaming-system-openwiki-hosted-workflow-guidance-drift-001` | The pinned OpenWiki generator retained its upstream scheduled-GitHub-Action sentence after local workflow creation was disabled, so every lifecycle begin could reintroduce false contributor guidance. | First Phase 5 lifecycle begin after the direct documentation edit. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-read-back-hosted-automation-settings-after-policy-delivery-001` | After changing hosted-automation policy, read back the remote setting and workflow inventory while keeping that network check outside the local delivery gate. | Repository settings can drift independently of committed source, but remote availability must not become a local quality dependency. |
| `PR-omarchy-gaming-system-reconcile-contributor-guidance-after-automation-ownership-change-001` | When automation ownership changes, search contributor, generator, built-tool, generated-wiki, operator, architecture, and README surfaces for stale execution claims, then fix the authoritative owner and repeat its lifecycle. | Removing a workflow file or directly editing generated guidance does not correct the template that owns future refreshes. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-local-only-delivery-evidence-reaffirmed-001` | Hosted CI/CD remains disabled and absent; `bin/gate.sh` plus its worktree receipt is the sole delivery-quality proof, and remote settings readback is supplementary external evidence rather than a gate dependency. | `docs/architecture/adr-0001-agent-work-pipeline.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective (5/5). Ticket 039 recall prevented duplicate checker or hosted-state
logic from entering the offline gate, while the Phase 5 ownership failure
proved why lifecycle execution must be part of completion rather than a
documentation formality. Returning to design produced a durable source/build
patch and process-reload regression instead of shipping a sentence that the
next OpenWiki begin would erase. Exact external readback, repeated verified
tool setup, fresh-process completion, matching structural/delivery receipts,
and the complete local gate closed the settings, generator, generated-output,
and repository-enforcement loops.
