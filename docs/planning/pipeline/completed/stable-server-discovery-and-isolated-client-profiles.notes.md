---
title: Stable server discovery and isolated client profiles — notes
pipeline_id: de9c08be-fa26-488b-a6f7-0b068add0761
---

# Stable server discovery and isolated client profiles — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 030 shipped and remote `main` is clean at
  `4b229d40e843c6d6841ea87010ffb0a6ddeacd74`; there is no active pipeline or
  blocking bulletin. The real external two-clean-installation acceptance event
  remains open because it requires people and machines outside this workspace.
- Recall: ADR-0003 defines one owner-operated server origin as one independent
  community trust domain, explicitly rejects implicit federation, and orders
  stable server identity/capability discovery plus isolated profiles before
  marketplace synchronization.
- Recall: the current QML connector stores one editable `serverUrl` in process,
  validates exact loopback HTTP or remote HTTPS origins, recognizes the server
  through `/health`, and clears all bearer/persona/dependent controller state
  when returning to server selection. It persists no credentials today.
- Recall: `PR-omarchy-bbs-verify-the-vertical-slice-001` requires the real
  migration, endpoint, and QML consumer to run together. Ticket 022 rules also
  require exact client response bounds, production-root QML compilation, and
  protected test-only secret handoffs.
- Recall: `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0 and
  OpenWiki 0.3.3; PostgreSQL is healthy in the local Compose project.
- Decision: skip rather than falsely complete the external-human roadmap item,
  and take the next independently executable engineering outcome as Ticket 031.
- Decision: persist a random singleton UUID with server data. The public
  operator name may change, but hostname, certificate, and name changes cannot
  silently change community identity.
- Decision: profiles are bounded non-secret convenience records. The client
  retains the existing no-persisted-token contract and clears every live
  authority before a new-origin request.
- Decision: an origin/UUID mismatch is an explicit identity-change failure,
  not an automatic profile update. The user must remove the prior profile
  before intentionally trusting the replacement.

## Phase 2 — Design

- CodeGraph design evidence traced `Config::from_environment` into `main`, the
  router-helper fan-in into `router_with_provider_runtime`, `AppState`, `/health`,
  and the test callers. The constructor change affects `main`, three app test
  helpers, and the registered-provider API fixture; the normal `router` helper
  has 27 API-test callers and therefore retains a stable default-name wrapper.
- Direct inspection covered QML because CodeGraph does not model these QML
  object/function flows reliably. `OnboardingController` currently clears its
  bearer, MFA challenge, persona inventory, selection, and expected request
  generation before `ApiClient.configure`; dependent social/game controllers
  derive authority from that selected persona and already clear on actor loss.
- Qt 6.5 exposes `QtCore.Settings` with `category`, `location`, `value`,
  `setValue`, and `sync`. The design uses a project-unique category and an exact
  single JSON value; fixture runs receive an isolated `XDG_CONFIG_HOME` and two
  separate test-runner processes prove persistence without touching user state.
- The discovery endpoint is `GET /.well-known/omarchygs`, public and no-store,
  with exactly five fields. It does not expand `/health`, expose software or
  database details beyond the existing health contract, or accept request data.
- Migration design uses a checked boolean singleton key, random UUID default,
  initial insert, and immutable update/delete/truncate triggers. Recovery
  evidence compares the UUID before and after the existing full database dump
  and restore drill.
- Profile design caps the inventory at 16 records/16 KiB, validates exact public
  keys and canonical origins, rejects duplicate origins and UUIDs, and never
  auto-connects from persisted content. Direct entry of a remembered origin
  still enforces the stored UUID pin.
- Compatibility design accepts protocol 1 only, requires invite registration,
  device sessions, and personas, retains bounded unknown capability strings,
  and exposes a fixed incompatible state for missing required capabilities.
- The complete implementation/file manifest, hostile regression matrix, and UX
  behavior are recorded in the active spec. Phase 2 is PASS.

## Phase 3 — Implement

- Added forward-only migration `0018_server_identity.sql`: one checked singleton
  UUID row plus update/delete/truncate rejection. The real operator recovery
  drill now reads discovery before backup and after restore and requires exact
  UUID equality.
- Added bounded `OGS_SERVER_NAME`, the dedicated no-store
  `GET /.well-known/omarchygs` contract, deterministic truthful capability
  advertisement, generic database-unavailable handling, and focused unit/API
  coverage. `/health` is unchanged.
- Added `ServerProfiles.qml` with an explicit platform-config INI location,
  exact public-only schema, 16-record/16-KiB caps, canonical-origin and UUID
  deduplication, rejection/reset of hostile saved state, and no persisted
  credentials or authority.
- Replaced health-based QML recognition with exact discovery negotiation.
  Direct connect-once, save-and-connect, pinned saved selection, explicit
  removal, compatible unknown capabilities, incompatible protocol state, and
  origin/UUID replacement rejection are wired through the keyboard-first
  connection screen.
- Extended fixtures to two compatible servers, future capability, incompatible
  protocol, identity replacement, wrong service, malformed, slow, and oversized
  discovery. Separate QML test-runner processes prove two public profiles
  survive restart before fixture tests clear the isolated test store.
- Updated package manifests/smoke, development smoke, API/architecture/operator
  documentation, environment example, and restore guidance.
- Focused validation:
  - configuration unit tests: PASS, 6 tests;
  - discovery unit/PostgreSQL tests with explicit `DATABASE_URL`: PASS, 4 tests;
  - QML onboarding/profile suite: PASS, 44 tests;
  - native client package: PASS, deterministic artifact, 38 runtime files;
  - operator PostgreSQL backup/restore drill: PASS, including server UUID.
- One initial discovery test invocation omitted `DATABASE_URL`: its two pure
  tests passed and two PostgreSQL harness cases did not execute. The explicit
  configured rerun passed all four.
- The first `bin/gate.sh --fast` run was RED only because the new real-PostgreSQL
  `sqlx::test` lacked the repository-standard ignore marker and the portable
  unit stage intentionally has no `DATABASE_URL`. All other 14 fast stages
  passed. The test was marked `requires PostgreSQL; run scripts/test-database.sh`
  after its explicit database run had already passed.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness / blast radius | Final CodeGraph inspection traced discovery document construction through the Axum route and confirmed the router-constructor callers affected by the new public-name state. QML, SQL, and shell surfaces were inspected directly because CodeGraph does not model them completely. | None | PASS. |
| 2 | Security / privacy | Codex Security diff scan `dee3a3f5-1af4-4ef8-95dd-be55c07ede12` reviewed the frozen snapshot `codex-security-snapshot/v1:sha256:2cdee0fd96ddab2ff27cde6928de7fc8a3f0974508127d85066eaf6bec55b95a`, closed all 11 generated items plus QML/Python/package surfaces, and reported zero findings. | None | PASS; sealed report at `/tmp/codex-security-scans-gVuCBs/omarchy_bbs/4b229d40e843c6d6841ea87010ffb0a6ddeacd74_20260826T204548Z_c_nr99b1/report.md`. |
| 3 | Origin / authority isolation | Direct inspection confirmed every new-origin path cancels the prior request generation and clears bearer, MFA, username, persona, and dependent authority before configuration or network access. Typed remembered origins cannot bypass the saved UUID pin. | None | PASS; hostile and cross-server QML fixtures cover the boundary. |
| 4 | Persistence / identity | The server UUID is database-owned and immutable through normal DML/truncate; saved client state is exact, bounded, public-only, deduplicated, revalidated before use, and never auto-connects. | None | PASS; database, two-process QML, package, and recovery evidence present. |
| 5 | Compatibility / UX | Discovery requires the implemented protocol-1 onboarding subset, tolerates bounded future capabilities, rejects incompatible or replaced identities before account UI, and exposes explicit keyboard-accessible connection choices. | None | PASS. |

Phase 3.5 is PASS with no unresolved findings.

## Phase 4 — Validate

- `./scripts/test-database.sh`: PASS. The canonical PostgreSQL suite executed
  58 tests across the server, operator admin, and operator CLI targets with
  zero failures. This includes the migration-backed discovery identity,
  exactness, stability, and immutability test.
- `bin/gate.sh --fast`: PASS after the database-only test received the standard
  ignore marker; all 15 stages were green.
- `bin/gate.sh --diff`: PASS across all 22 stages before Phase 5 edits, with
  worktree receipt `9dc09fe8d12e550d29593cf225553b660425696ef78337a4840102782d899f76`.
  It repeated the PostgreSQL suite, two-process profile proof, 44-case fixture,
  live QML/API flows, package, provider, recovery, and admission drills.

## Phase 5 — Complete

- EARS audit:

  | Requirement | Evidence | Result |
  |---|---|---|
  | REQ-001 | Migration singleton/immutability API test and source-versus-restored UUID drill | PASS |
  | REQ-002 | Exact no-store discovery success, provider capability, unavailable database, and field-absence tests | PASS |
  | REQ-003 | Six configuration tests plus UUID-stable name-change API case | PASS |
  | REQ-004 | Exact bounded `ServerProfiles.qml` schema and two-process persistence proof | PASS |
  | REQ-005 | Typed/saved origin UUID pin and identity-replacement fixture | PASS |
  | REQ-006 | Pre-request bearer/MFA/username/persona clearing and dependent-controller fixture | PASS |
  | REQ-007 | Credential-like, extra-key, duplicate, unsupported, and oversized saved-state fixtures | PASS |
  | REQ-008 | Protocol/capability compatibility matrix with bounded unknown capability | PASS |
  | REQ-009 | Saved/direct/remove actions in the 44-case keyboard, accessibility, plain-text, and 640×420 corpus | PASS |
  | REQ-010 | Two compatible profiles across separate QML processes plus the complete 22-stage gate | PASS |
- OpenWiki run `fde9d225-3a6c-4c8e-9611-45a7a1d08550` returned
  `status: complete` and reconciled quickstart, runtime, product-boundary, and
  validation pages. Its warnings were the broad pages' pre-existing unrelated
  Claims evidence debt; no lifecycle action was skipped.
- AAR-031 is submitted with three failure IDs, three prevention rules, and one
  architecture decision, all appended to the knowledge register.
- The owner-operated-server roadmap outcome is checked, Ticket 031 is closed,
  and this spec/notes pair is the only pipeline pair moved from active to
  completed. The external two-installation human acceptance event remains open.
- Phase 5 is PASS. Delivery will rerun the diff gate after these completion
  edits so the final worktree receipt binds the exact committed tree.
