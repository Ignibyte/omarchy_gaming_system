---
title: Local-only automation state reconciliation — notes
pipeline_id: 7f356f37-bd05-43f0-a123-98a74a6c99ba
---

# Local-only automation state reconciliation — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 042 was delivered at `0a9b8bd`, the four remaining roadmap
  outcomes were committed as promotion-ready intakes at `30509cf`, local and
  `origin/main` matched, and the worktree began clean with no active pipeline
  or open ticket.
- Recall: the owner explicitly authorized autonomous commits and pushes and
  reaffirmed that GitHub/hosted CI/CD must be absent; all quality gates remain
  local.
- Recall: Ticket 039 deleted `.github/workflows/ci.yml`, added
  `scripts/check-local-only-automation.sh`, wired it into
  `scripts/check-pipeline.sh` and the canonical gate, and added a hostile
  `.github/workflows/ci.yml` fixture to the server-module architecture proof.
- Recall: current repository inventory contains no GitHub Actions, GitLab,
  CircleCI, Buildkite, Drone, Woodpecker, Jenkins, or Azure pipeline
  definition. README, Constitution, and ADR-0001 already name the local gate as
  sole delivery proof.
- Failure observed: current GitHub API readback returned `enabled: true` with
  zero workflows, contradicting Ticket 039's external-state claim. The owner
  instruction authorized an immediate settings correction; the subsequent API
  readback returned `enabled: false` and `total_count: 0`.
- Failure observed: `AGENTS.md` still claims that a scheduled OpenWiki GitHub
  Actions workflow refreshes the wiki, contradicting both the repository state
  and the local-only Constitution.
- Decision: preserve the existing local checker and hostile test; this ticket
  repairs the external setting and stale contributor guidance, then proves the
  complete local enforcement still works.
- Decision: remote readback is completion evidence but never a local-gate
  dependency. GitHub availability cannot decide whether a worktree is safe to
  commit.
- `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0, OpenWiki
  0.3.3, verified pnpm, patch/build provenance, and Codex-only tooling ready.
- No critical bulletin blocks work.
- Phase 1 is PASS.

## Phase 2 — Design

- Architecture and evidence flow:
  1. `bin/gate.sh` remains the only quality entrypoint. Its stage 7 invokes
     `scripts/check-pipeline.sh`, which directly invokes
     `scripts/check-local-only-automation.sh`; stages 23 and 24 repeat the
     same boundary through the server-module proofs.
  2. `scripts/check-local-only-automation.sh` accepts an optional root and
     rejects files under `.github/workflows` plus common Buildkite, CircleCI,
     Drone, GitLab, Woodpecker, Jenkins, and Azure pipeline locations. The
     Ticket 039 hostile fixture creates a fresh GitHub workflow under a
     temporary root and requires rejection.
  3. GitHub repository Actions permission is external configuration, not
     source. It was disabled through the GitHub API and is verified by a
     separate readback. The local gate intentionally performs no network call
     and remains valid when GitHub is unavailable.
  4. The reviewed local OpenWiki integration is the authoritative owner of the
     managed `AGENTS.md` block. Extend its pinned source patch and readiness
     assertions so deterministic setup writes the mandatory local lifecycle
     and hosted-automation prohibition. README, Constitution §0/§15, and
     ADR-0001 already state the correct architecture and need no semantic edit.
  5. Phase 5 uses the project-local OpenWiki MCP lifecycle to reconcile any
     generated quickstart/workflow claims; no generated file is hand-edited.
- Exact file manifest:

  | Path | Purpose |
  |---|---|
  | `scripts/setup-pipeline-tools.sh` | Extend the reviewed pinned OpenWiki patch and generate correct local-only contributor guidance. |
  | `scripts/check-pipeline-tools.sh` | Fail closed unless the reviewed sentence exists in both OpenWiki source and build output. |
  | `AGENTS.md` | Lifecycle-generated result of the corrected local setup; never hand-edited. |
  | Ticket/spec/notes/AAR/index paths | Preserve the numbered workflow, findings, decisions, acceptance audit, and archive. |
  | `openwiki/*` | Only lifecycle-generated reconciliation if OpenWiki identifies affected durable pages or claims. |

  No Rust, QML, SQL, Cargo, Compose, package, hook, or gate script is changed.
  The GitHub setting change has no repository path.
- Database, API, client, and compatibility consequences: none. No migration,
  runtime configuration, route, schema, client behavior, or executable
  dependency-pin changes. Existing application development and gate commands
  are byte-for-byte unchanged.
- Requirement-to-evidence map:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | GitHub API reads `enabled: false`, `total_count: 0` before completion and after delivery. |
  | REQ-002 | Direct tracked/residual path inventory, positive local checker, hostile temporary GitHub workflow rejection, server-module proof, and full local gate. |
  | REQ-003 | Exact reviewed OpenWiki source/build patch, lifecycle-generated AGENTS result, plus unchanged correct README, Constitution, ADR-0001, and generated-page audit. |
  | REQ-004 | Shell syntax, pipeline structure, local-only checker, secret/whitespace checks, complete `bin/gate.sh --diff`, and staged dependency review. |
  | REQ-005 | Matching design/inspection CodeGraph receipts, OpenWiki completion receipt, AAR/index entries, ticket closure, and archived pair. |

- Security, operations, and failure risks:
  - A GitHub setting can drift without a Git commit. Completion therefore
    records fresh API readback, but local validation never trusts or depends on
    that mutable external setting.
  - An empty workflow inventory plus enabled Actions is dormant today but
    violates policy and would execute a future accidentally pushed workflow;
    both the permission and repository file boundary must be enforced.
  - Removing the stale sentence must not imply generated OpenWiki pages are
    hand-maintained. The corrected text requires the local lifecycle and keeps
    generated files lifecycle-owned.
  - The GitHub CLI credential is supplied by the existing system keyring; it is
    never printed unmasked, copied into Git, or passed to the local gate.
- Material alternatives rejected:
  - Adding GitHub settings readback to `bin/gate.sh` was rejected because it
    would make the local quality proof network- and account-dependent.
  - Rewriting the checker or adding another hostile harness was rejected
    because direct inspection and Ticket 039 evidence show the exact GitHub
    workflow path is already rejected in the canonical gate.
  - Leaving Actions enabled because there are zero workflows was rejected: it
    preserves an unnecessary remote execution capability and contradicts the
    owner's explicit policy.
  - Hand-editing generated OpenWiki output was rejected; Phase 5 owns any
    claims-backed reconciliation.
- CodeGraph design evidence: exploration for the gate/checker/receipt flow
  issued a matching design receipt for pipeline
  `7f356f37-bd05-43f0-a123-98a74a6c99ba` and gated state
  `6d1febb47c50f51bc9cd584bc1d0f54f7cf944fcb6182858fcafa9f6e21e62f0`.
  CodeGraph does not index the controlling shell/config/docs surfaces and
  returned only irrelevant token-name matches in production Rust; direct
  source review above is authoritative and confirms no runtime blast radius.
- Phase 2 is PASS.

### Design amendment after Phase 5 lifecycle finding

- The first OpenWiki update began successfully and immediately rewrote the
  managed `AGENTS.md` block back to the upstream sentence claiming a scheduled
  GitHub Actions refresh. The lifecycle, not the earlier direct edit, is the
  authoritative owner of that block.
- Root cause: Ticket 039's reviewed ignored-dependency patch changes
  `createWorkflow` to `false`, but leaves `createCodeModeAgentsSnippet()` at
  its upstream hosted-workflow wording. Every deterministic `openwiki_begin`
  therefore reintroduces guidance that contradicts the already-patched runtime
  behavior.
- The initial no-op documentation lifecycle completed with status `complete`;
  generated pages already describe local-only automation accurately. It also
  removed two stale quickstart source projections during Claims finalization.
  Its completion receipt will be invalidated by the corrective implementation
  and replaced by a fresh lifecycle after the tool patch.
- Revised file and ownership design:
  1. `scripts/setup-pipeline-tools.sh` shall accept only the exact reviewed
     upstream or already-local contributor sentence, replace it with one
     local-lifecycle/hosted-CI-prohibited sentence in the ignored pinned source,
     build OpenWiki, and verify the source transformation.
  2. `scripts/check-pipeline-tools.sh` shall require the reviewed sentence in
     both the pinned source and compiled distribution and reject the upstream
     hosted-workflow sentence, alongside the existing disabled-workflow check.
  3. The setup script shall regenerate the managed `AGENTS.md` block through
     OpenWiki's deterministic repository setup; no direct edit owns it.
  4. A fresh OpenWiki update shall prove that a subsequent lifecycle preserves
     the corrected block and produce the final completion receipt.
- The existing expected OpenWiki source-change inventory remains exactly
  `src/ingestion/code-mode.ts` and
  `src/integrations/core/session-manager.ts`; the additional patch is in the
  already-reviewed `code-mode.ts` path. No dependency pin, lockfile, network
  gate, runtime, application, database, API, or QML surface changes.
- Focused proof expands to setup/readiness execution, exact source/dist/managed
  block assertions, a repeated OpenWiki lifecycle, and the existing local-only
  positive/hostile checks before the full local diff gate.
- Fresh CodeGraph design exploration issued the matching design receipt for
  pipeline `7f356f37-bd05-43f0-a123-98a74a6c99ba` and amended gated state
  `c08b2ffa7bab7bc5e2b6b86dd383ec88a3a94f64381b16883621be0e117f6054`.
  CodeGraph again cannot model the controlling shell or ignored TypeScript
  dependency source and returned unrelated indexed Rust symbols; the complete
  direct source and ownership inspection above is authoritative.
- The amended Phase 2 design is PASS.

## Phase 3 — Implement

- Built:
  - Disabled GitHub Actions through the authenticated repository API. Immediate
    readback returned `enabled: false`, and workflow inventory returned
    `total_count: 0`.
  - The initial direct replacement of the stale managed `AGENTS.md` sentence
    was superseded after OpenWiki correctly asserted ownership and regenerated
    its upstream text during the first completion attempt.
  - Preserved `bin/gate.sh`, `scripts/check-pipeline.sh`,
    `scripts/check-local-only-automation.sh`, and the hostile fixture without
    modification; direct inspection found no missing repository path class for
    the reported GitHub Actions case.
- Focused evidence:
  - `git diff --check` passed.
  - `bash -n` passed for the local-only checker, pipeline check,
    server-module proof, and canonical gate.
  - `scripts/check-local-only-automation.sh` passed on the worktree.
  - `scripts/check-pipeline.sh` passed.
  - `scripts/test-server-module-spike.sh` passed formatting, warnings-denied
    Clippy, all 21 nested-workspace tests, binaries, warnings-denied rustdoc,
    deterministic fixtures, 13 process scenarios, the positive local-only
    check, and the hostile temporary `.github/workflows/ci.yml` rejection.
- Initial deviation: the first implementation treated the managed `AGENTS.md`
  block as ordinary contributor documentation. The first OpenWiki lifecycle
  exposed that ownership mistake and returned the work to design before
  delivery.

### Implementation amendment

- Extended `scripts/setup-pipeline-tools.sh` so the reviewed pinned OpenWiki
  patch accepts only the exact upstream or already-local contributor sentence,
  replaces the upstream hosted-workflow claim with project-local lifecycle and
  hosted-CI prohibition text, compiles it, and verifies both source and build
  lack the obsolete sentence.
- Extended `scripts/check-pipeline-tools.sh` to fail closed unless that exact
  local sentence exists in both the reviewed source and compiled distribution;
  the existing pin, frozen lock, exact source-path inventory, patch digest, and
  build digest checks remain intact.
- Ran the verified setup twice. The first transformed the upstream sentence;
  the second proved the already-local path is idempotent. Both installs used
  the reviewed OpenWiki commit, integrity-pinned pnpm, frozen lockfile,
  disabled install scripts, exact source-change inventory, rebuilt
  distribution, and fresh provenance.
- A fresh post-rebuild OpenWiki MCP process began twice and retained the exact
  local-only managed block. The second run finalized with
  `{"status":"complete"}` and removed its temporary plan. No managed
  `AGENTS.md` content was hand-edited in the corrected implementation.
- OpenWiki Claims finalization removed two stale quickstart source projections;
  no generated prose or material proposition needed changing because
  `quickstart.md`, `codex-workflow.md`, and `development-and-validation.md`
  already describe the local-only architecture accurately.
- Amended focused evidence:
  - `git diff --check` and shell syntax passed.
  - `scripts/check-pipeline-tools.sh`,
    `scripts/check-local-only-automation.sh`, and
    `scripts/check-pipeline.sh` passed.
  - Exact residual assertions found the reviewed local sentence in source,
    compiled distribution, and lifecycle-generated `AGENTS.md`, and found the
    obsolete scheduled-GitHub-Action sentence in none of them.
  - Direct hosted-workflow path inventory remained empty.
- No application, database, API, QML, dependency pin, gate-stage, hook, or
  hosted automation change was introduced.
- The amended Phase 3 is PASS.

## Phase 3.5 — Initial inspect ledger (superseded)

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | External configuration / least privilege | GitHub Actions permission was enabled despite an empty workflow inventory and the committed local-only policy. A future accidental workflow push would regain remote execution. | Medium policy drift | Fixed before repository edits: Actions disabled through the authenticated API; two independent readbacks returned `enabled: false` and zero workflows. Final post-delivery readback remains required. |
| 2 | Correctness / contributor guidance | `AGENTS.md` described a scheduled GitHub Actions OpenWiki refresh that did not exist and contradicted the Constitution. | Low | Fixed with exact local `$openwiki` lifecycle language, hosted-CI prohibition, local rejection, and lifecycle-owned generation. Residual search found no equivalent claim. |
| 3 | Enforcement / regression coverage | The existing checker already covers `.github/workflows` and common equivalent providers; its canonical hostile fixture creates and rejects a temporary GitHub workflow. | — | Retained without duplication. Focused proof passed the positive and hostile paths. |
| 4 | Secrets / privacy | GitHub CLI used the pre-existing system-keyring credential; no token, key, account secret, API body, or remote credential entered the diff or local gate. | — | Secret hook passed; staged review must retain only public settings outcomes. |
| 5 | Runtime / database / QML | The sole gated diff is contributor guidance; no production source, dependency, migration, configuration, route, package, or client surface changed. | — | No runtime test expansion required. Full canonical gate still supplies integration regression proof. |
| 6 | Simplification / reuse | Adding remote API checks to the gate or another checker/test would duplicate existing coverage and introduce network coupling. | — | Rejected. Keep external readback as explicit completion/delivery evidence only. |

- Fresh CodeGraph inspection issued the matching receipt for pipeline
  `7f356f37-bd05-43f0-a123-98a74a6c99ba` and post-implementation gated state
  `53bfa2550060217125e4ec68aa2b267efc87c818d944fe3cf9bca8c188ec171d`.
  Shell, config, and documentation remain graph-unsupported; the exploration
  returned unrelated Rust token matches and no relevant production path.
  Direct source, residual-path, and external-state review therefore remains
  authoritative for the complete blast radius.
- This inspection and its receipt predated the lifecycle finding and the two
  tool-script edits. It is preserved as historical evidence but is no longer
  the Phase 3.5 exit proof.

## Phase 3.5 — Fresh inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness / ownership | Directly editing the managed `AGENTS.md` block was not durable because `openwiki_begin` deterministically owns and rewrites it. | Medium workflow defect | Fixed at the reviewed pinned generator; two fresh post-rebuild begins retained the corrected local-only text. |
| 2 | Supply chain / fail closed | A free-form replacement against an unexpected future OpenWiki sentence could patch the wrong bytes or silently do nothing. | — | Setup accepts only the exact reviewed upstream or already-local sentence, retains the exact commit/lock/source-path inventory, and verifies source plus compiled output. |
| 3 | Readiness / build drift | Source-only validation could leave the running compiled distribution stale. | — | Readiness now requires the local sentence in both source and `dist/ingestion/code-mode.js`, rejects the upstream sentence in both, and still matches patch/build provenance. |
| 4 | Process lifecycle | The already-running MCP process retained its pre-rebuild module in memory. | Expected operational boundary | Completed that run, started a fresh MCP server from the verified rebuilt install, repeated begin after an interrupted test-client cleanup, and finalized successfully. Setup continues to tell operators to restart Codex after rebuilding. |
| 5 | Local-only enforcement | No hosted workflow file, alternative provider definition, remote gate call, or quality-stage change entered the repository. | — | Existing positive checker, hostile GitHub workflow fixture, pipeline wiring, direct path inventory, and later full gate remain authoritative. |
| 6 | Runtime / secrets | No Rust, SQL, QML, application configuration, dependency pin, credential, or player/operator behavior changed. | — | Scope remains two local tool scripts, lifecycle-owned guidance/metadata, planning records, and external GitHub permission state. Secret scan remains required by the gate. |
| 7 | Generated documentation | OpenWiki removed two stale quickstart source projections while leaving its already-accurate local-only workflow prose unchanged. | — | Lifecycle-owned Claims finalization completed; no generated prose was hand-edited and navigation remains intact. |

- Direct review covered the complete setup/readiness diff, shell expansion and
  `sed` safety, exact accepted inputs, source/build/provenance ordering,
  repeated setup, repeated fresh-process lifecycle behavior, residual hosted
  workflow paths, generated diffs, and unchanged runtime boundaries.
- Fresh CodeGraph inspection issued the matching receipt for pipeline
  `7f356f37-bd05-43f0-a123-98a74a6c99ba` and post-amendment gated state
  `66e1f0f3963b45c81127818c0f5c09a6921de6efe9e41325c243ee04eec297b1`.
  Shell, generated documentation, and ignored dependency state remain
  graph-unsupported; CodeGraph returned unrelated indexed Rust symbols. Direct
  source, lifecycle, and residual review is authoritative for this blast radius.
- No unresolved correctness, security, supply-chain, operations,
  documentation, or simplification finding remains.
- The amended Phase 3.5 is PASS.

## Phase 4 — Initial validation (superseded)

- Tests run:
  - `git diff --check` passed.
  - `bash -n` passed for `scripts/check-local-only-automation.sh`,
    `scripts/check-pipeline.sh`, `scripts/test-server-module-spike.sh`, and
    `bin/gate.sh`.
  - `scripts/check-local-only-automation.sh` passed on the worktree.
  - `scripts/check-pipeline.sh` passed.
  - `scripts/test-server-module-spike.sh` passed its formatting, Clippy,
    21-test nested workspace, build, rustdoc, deterministic-fixture,
    13-process-scenario, local-only positive, and hostile hosted-workflow
    checks.
- Gate run: `bin/gate.sh --diff` passed every configured local stage and wrote
  `.git/omarchy-gaming-system-gate-receipt` with gated-state hash
  `53bfa2550060217125e4ec68aa2b267efc87c818d944fe3cf9bca8c188ec171d`,
  exactly matching the current gated worktree and the Phase 3.5 inspection
  receipt.
- Skips or pre-existing failures: none. Cargo reported the already-packaged
  `chacha20 0.10.1` yanked-version warning as informational; it did not fail
  dependency, build, or test validation.
- The matching receipt proved the initial gated state only. It became stale
  when the OpenWiki generator/readiness fix changed gated scripts and cannot
  support delivery.

## Phase 4 — Fresh validation

- Focused validation passed:
  - `git diff --check` and shell syntax for the changed tool scripts plus the
    local checker, pipeline check, and canonical gate;
  - two complete verified `scripts/setup-pipeline-tools.sh` rebuilds, including
    original-to-local and already-local input paths;
  - `scripts/check-pipeline-tools.sh`,
    `scripts/check-local-only-automation.sh`, and
    `scripts/check-pipeline.sh`;
  - exact source/build/managed-guidance positive and obsolete-guidance negative
    assertions; and
  - two fresh-process OpenWiki begins plus one successful finish.
- `bin/gate.sh --diff` passed all configured local stages 1–24: Rust format,
  warnings-denied Clippy/tests/rustdoc, Compose, shell, pipeline/readiness,
  secret, hooks, whitespace, cartridge contract/renderer/SDK/architecture,
  native source/trust/publication/package, 82 PostgreSQL-backed database cases,
  the live API/QML smoke, provider security and Door Legends authority,
  operator restore, private-alpha admission, and both server-module proofs.
- Gate 23 explicitly passed the positive local-only check and rejected its
  hostile temporary `.github/workflows/ci.yml` fixture.
- The final gate, fresh inspection, and current gated worktree all match
  `66e1f0f3963b45c81127818c0f5c09a6921de6efe9e41325c243ee04eec297b1`.
- Skips or failures: none. Test cases intentionally marked ignored in ordinary
  workspace mode were executed by their dedicated gate stages. The packaged
  `chacha20 0.10.1` yanked-version warning remains informational and did not
  fail dependency, build, or test validation.
- The amended Phase 4 is PASS.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — authenticated GitHub readback before completion returned
    `enabled: false`, `sha_pinning_required: false`, and an empty workflow
    inventory with `total_count: 0`; the same readback remains required after
    delivery.
  - REQ-002 PASS — tracked/residual provider inventory is empty, the direct
    checker and pipeline check passed, and gate 23 explicitly rejected the
    hostile temporary `.github/workflows/ci.yml` fixture.
  - REQ-003 PASS — reviewed OpenWiki source and compiled output own the exact
    local-lifecycle guidance, two fresh begins preserved the generated
    `AGENTS.md` block, and README, Constitution, ADR-0001, quickstart,
    Codex-workflow, and development/validation guidance remain consistent.
  - REQ-004 PASS — every focused and canonical validation ran locally; no
    network check entered the gate; `GATE GREEN [diff]` and its receipt match
    state `66e1f0f3963b45c81127818c0f5c09a6921de6efe9e41325c243ee04eec297b1`.
  - REQ-005 PASS — revised design and inspection CodeGraph evidence, successful
    OpenWiki completion, matching completion receipt, submitted AAR, five
    registered knowledge IDs, closed ticket, and completed pipeline pair are
    mutually linked and current.
- Docs: the local OpenWiki lifecycle finalized successfully. Generated workflow
  prose was already accurate; Claims finalization removed two stale quickstart
  source projections and refreshed lifecycle metadata. The managed AGENTS block
  is now produced by the reviewed local generator rather than a direct edit.
- AAR: `AAR-043-local-only-automation-state-reconciliation.md` is submitted as
  effective with both failures, both prevention rules, and the reaffirmed
  local-only architecture decision appended to the knowledge register.
- Archive: Ticket 043 is closed and indexed; this spec/notes pair is moved to
  `docs/planning/pipeline/completed/` with no active pair remaining.
- Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | GitHub Actions permission was enabled again even though no workflow definition remained. | Repository settings can drift independently of committed local enforcement, and no fresh external readback was made after Ticket 039 delivery. | Disabled Actions through the GitHub API and retained zero workflows. | Verify the remote hosted-automation setting after policy delivery without making the local gate depend on it. |
| 2 | The OpenWiki-managed `AGENTS.md` block described a scheduled GitHub Actions refresh that did not exist and restored it after a direct edit. | The reviewed patch disabled workflow creation but did not update the pinned generator sentence that owned future contributor guidance. | Extend the reviewed source patch and source/build readiness assertions; rebuild and repeat the lifecycle in a fresh process. | Audit authoritative generators and built output whenever automation ownership changes, then exercise the owning lifecycle. |
