---
title: Reviewed provider sidecar and deployment operations — notes
pipeline_id: 35105398-ffdc-433f-b83a-86e418471a07
---

# Reviewed provider sidecar and deployment operations — completed notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Tickets 018, 019, 044, and 045 establish exact registered endpoint
  identity, globally routed production egress, TLS roots, signed messages and
  grants, final locked lifecycle admission, aggregate deadlines, durable
  replay, independent provider state, and a test-only exact loopback override.
- Recall: Ticket 045 found that a resolver override must bind the DNS host,
  canonical authority, and exact port and must reject IP-literal bypasses.
- Routing: external acceptance, official marketplace operations, and reviewed
  provider onboarding still require real people, systems, accounts, custody,
  and operating proof. This final SDK sidecar/operations slice is locally
  actionable and does not counterfeit those prerequisites.
- Decision: use the intake's required separate threat model to select and test
  a production sidecar profile before implementation. Co-location may alter
  only the destination binding; it may not alter provider identity, protocol,
  authority, persistence, quota, audit, or lifecycle semantics.
- `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0, OpenWiki
  0.3.3, verified pnpm, reviewed Codex-only patch/build provenance, and local
  tools ready. The bulletin register contains no active warning or critical
  entry, and the repository PostgreSQL service is healthy. Phase 1 is PASS.

## Phase 2 — Design

- Architecture and data flow:
  1. Add a production `SidecarTarget` containing one non-nil release UUID and
     one nonzero loopback socket. `ProviderBroker` selects it only when the
     freshly loaded registered security material has that exact release; every
     other release keeps the existing public-DNS production path.
  2. The sidecar guarded client skips DNS resolution only for that matching
     release, requires the registered endpoint port to equal the configured
     socket port, and uses the existing canonical HTTPS URL, DNS SNI/Host,
     registered TLS roots, no-proxy/no-redirect client, finite body/deadline
     bounds, compatibility exchange, grants, signatures, replay, quota,
     lifecycle, and audit. No CIDR, hostname suffix, arbitrary private address,
     or caller-supplied operation URL is added.
  3. Server provider configuration gains two optional all-or-none public
     values: the exact sidecar release UUID and socket. Existing four secret/
     callback values remain all-or-none; the runtime constructs either the
     remote broker or the exact sidecar broker. Wrong/malformed/incomplete
     values fail configuration, while a release/registered-port mismatch fails
     the transport before I/O.
  4. Provider-to-platform callbacks gain a separately named production
     sidecar constructor/mode. It maps the exact canonical DNS callback URL to
     one matching loopback socket while retaining the registered platform TLS
     root, exact authority/path, and signed event. The conformance override
     remains feature-gated and distinct. Relay Forge and Door Legends expose
     explicit sidecar callback configuration and reject simultaneous sidecar
     and conformance overrides.
  5. The platform session envelope remains the sole platform-side durable
     authority and provider state remains null there. Commands are denied
     locally when an active provider session is `provisioning`, `reconciling`,
     or `unavailable`; a `suspended` or `retired` session returns the existing
     lifecycle conflict without transport. Reconciliation remains the only
     operation allowed to recover a non-ready session, still subject to
     registry lifecycle policy. Cached authenticated views stay readable and
     no compiled registry path is called.
  6. A separately supervised provider service uses its own OS identity,
     mode-0600 configuration/keys, PostgreSQL role/database, writable state,
     backups, and resource/network limits. It may listen and call back only on
     loopback; a local TLS reverse proxy is required for the canonical platform
     callback because the Axum application listener is HTTP. Remote deployment
     retains public DNS routing and the same application protocol.
- Threat model: `docs/security/provider-sidecar-threat-model.md` records the
  actors, assets, effective resources, boundary crossings, prioritized hostile
  stories, mitigations, assumptions, and severity calibration. It identifies
  local port ownership as availability only—not authentication—and requires
  exact release/socket/port binding, TLS and signed identity, separate process/
  database/credentials, non-ready command denial, and bounded operator
  evidence. No applicable `SECURITY.md` exists. The required architecture
  review was sequential rather than independent because this workflow does not
  authorize a separate review agent.
- Compatibility and persistence: no protocol, public player route, registry
  schema, provider database schema, or migration changes. Remote deployments
  and provider-disabled servers preserve current behavior. Sidecar config is
  additive and optional. Rollback removes the two sidecar environment values
  and restarts the services; immutable provider/session records remain valid
  because the registered canonical endpoint and wire bytes never changed.
- Exact file manifest:

  | Path | Purpose |
  |---|---|
  | `crates/game-provider/src/{egress,broker}.rs`, `tests/{egress,sidecar_integration}.rs` | Exact release/socket production transport plus hostile TLS/socket/release and real-process lifecycle proof. |
  | `crates/server/src/{config,provider_games}.rs`, provider API/config tests | All-or-none sidecar startup wiring and non-ready command/read-only recovery policy. |
  | `crates/provider-starter/src/callback.rs`, README/tests | Separately named production sidecar callback destination with exact DNS/port/path/TLS binding. |
  | `examples/provider-relay-forge/src/main.rs` and conformance fixtures | Explicit sidecar callback configuration for the clean-room process without enabling conformance. |
  | `examples/first-party-door-legends/provider/src/main.rs` and pilot script | Production sidecar callback option for the only admitted provider while preserving its conformance-only override. |
  | `deploy/provider-sidecar/**` | Reviewed systemd service and secret-free exact platform/provider configuration templates. |
  | `scripts/test-provider-sidecar.sh`, `bin/{gate,lib-gate}.sh` | Template/containment checks, real sidecar start/stop/crash/restart/restore/reconcile drill, locally signed bounded receipt, and mandatory delivery stage. |
  | `docs/security/provider-sidecar-threat-model.md`, `docs/operators/provider-deployment.md`, provider runbooks, architecture, README, roadmap/intake, OpenWiki, and pipeline artifacts | Threat model, remote/co-located operations, authority limits, delivery closure, and durable knowledge. |

  No migration, provider protocol change, public route, provider registration,
  catalog admission, hosted workflow, package publication, or external system
  action is in scope.
- Regression and evidence matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Sidecar target/unit tests, real broker/provider TLS process, callback round trip, existing provider security suites. |
  | REQ-002 | IP/non-loopback/zero/wrong-port tests, wrong-release public-egress disposition, wrong TLS/host/path tests, partial server/provider config tests, no-redirect assertions. |
  | REQ-003 | Static systemd/template validation, exact file modes in drill, process/database identity assertions, secret/output scan, containment review. |
  | REQ-004 | Non-ready server command regression plus sidecar provider stop, failed new launch, restart, independent dump/restore, authenticated reconcile, callback recovery, and no-platform-table assertion. |
  | REQ-005 | Remote/co-located operator guide contract check covering identity, endpoint, DB, secrets, rotation, limits, monitoring, restore, suspension/revocation, incident, upgrade, and EOL. |
  | REQ-006 | Route/catalog/registration source diff, Door Legends-only production regression, roadmap/intake audit, OpenWiki/AAR, full local gate, and remote delivery readback. |
- Security/privacy/concurrency/failure controls: sidecar selection happens only
  after the broker loads current release material; final locked admission and
  aggregate deadline behavior remain unchanged. Exact TLS roots and signed
  bodies defeat local port squatting from becoming impersonation. Callback
  target values bind a DNS host, exact release path, and equal port. Neither
  receipt nor logs contain keys, grants, subjects, database URLs, or bodies.
  Process and database restarts preserve whole-operation receipts; unknown
  outcomes become read-only/reconciling and never switch authority.
- Material alternatives rejected: a general loopback/private allowlist would
  weaken SSRF controls; Unix-domain HTTP would introduce a second connector and
  either discard TLS identity or require a new TLS-over-UDS stack; plaintext
  loopback would make port ownership authentication; sharing the platform
  reverse proxy, process, database, user, or credentials would collapse failure
  domains; making the test-only conformance override production would erase an
  auditable boundary; registering Relay Forge or adding onboarding would
  convert a deployment proof into unauthorized product admission.
- CodeGraph design evidence traced `ProviderRuntime` construction through
  `ProviderBroker`, compatibility and operation execution, registered endpoint
  validation and guarded HTTPS construction, provider-starter callback/runtime
  handling, platform callback projection, failure availability, and compiled/
  provider authority separation. Blast radius covers server config/main and
  provider API tests, game-provider conformance/egress/registry tests, starter
  persistence/callback code, and both provider process examples. Direct review
  covered shell orchestration, operator docs, deployment templates, migrations,
  Cargo configuration, and other unsupported files.
- The worktree-bound design receipt matches pipeline
  `35105398-ffdc-433f-b83a-86e418471a07` and gated state
  `67b59e0306d02de61f4b351a17e5f9c9085c8b41ea768de5a9b0936904b4318f`.
  Phase 2 is PASS.

## Phase 3 — Implement

- Built: exact-release `SidecarTarget` and production broker/guarded-client
  routing; all-or-none server environment configuration; exact production
  callback sidecar mode in the starter and both provider examples; local
  non-ready command denial with existing suspended/retired conflict semantics;
  separate-user/systemd/config/callback-proxy templates; remote/co-located
  operations and threat-model documentation; and mandatory signed lifecycle,
  hostile-peer, separate-database restore, containment, and runbook contract
  drill in gate stage 19a.
- Focused proof: game-provider sidecar/egress tests, provider-starter default and
  conformance unit tests, server provider configuration tests, Relay Forge
  standalone tests, the exact Door Legends callback unit test, the complete
  starter conformance script, the sidecar drill, and the first-party authority
  pilot passed. The last three run real TLS provider processes and independent
  PostgreSQL state; the sidecar drill also rejects a hostile TLS peer holding
  the configured port, denies launch during crash, restarts/reconciles, restores
  the provider database, and verifies an ephemeral Ed25519 drill signature.
- Deviations: the existing real-broker `starter_integration` test was extended
  instead of adding the design manifest's separate `sidecar_integration` file,
  keeping one canonical real-process flow. The first drill's 500 ms aggregate
  budget was too narrow for repeated local compatibility exchanges under test
  load; its explicit fixture quota is now 3000 ms without changing production
  defaults or semantics. The first authority-pilot rerun exposed that a blanket
  non-ready guard changed established suspended/retired conflicts from `409` to
  `503`; the guard now maps those terminal/lifecycle states locally to the
  existing `GameUnavailable` conflict while reserving `ProviderUnavailable`
  for provisioning/reconciling/outage states. The rerun passed.
- The required security diff inspection found two reportable hardening gaps and
  one product-correctness race: the callback Caddy template retained its default
  mutable admin API, Door Legends callback delivery could honor ambient proxy
  variables before its socket mapping, and command/reconcile admission was not
  linearized across the network operation. The fixes disable the admin API,
  force no-proxy callback transport, and add a durable database reservation with
  expiry recovery, response fencing, and a transaction-scoped PostgreSQL
  advisory fence held across provider transport. Migration `0029` was added
  because an in-memory lock would not survive a platform crash or protect
  multiple server processes; the advisory fence prevents a live process from
  being reclaimed, while the durable reservation retains crash recovery state.
- Fresh CodeGraph inspection traced server configuration into the sidecar
  broker, guarded transport, operation/replay path, callback transport, session
  availability, and main runtime construction. It found no alternate runtime
  caller or transport path; standalone example configuration and shell/systemd/
  documentation assets were inspected directly because they are outside its
  indexed Rust workspace. Phase 3 is PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security diff scan | The reviewed Caddy callback template left its mutable default loopback admin API reachable to the separately sandboxed provider user. | Medium | Fixed with a complete global `admin off` block, runbook contract, and drill assertion. |
| 2 | Security diff scan | Door Legends callback delivery could select ambient HTTP(S)/all-proxy configuration before applying its exact loopback DNS mapping. | Low | Fixed with unconditional reqwest `.no_proxy()` and the full pilot running under hostile proxy variables. |
| 3 | Correctness/security-policy inspection | Command readiness and outbound provider execution were not one linearized operation, allowing a queued command to outlive a recovery transition. | Blocking | Fixed with migration `0029`, one durable reservation, response UUID fencing, and deterministic concurrent reconcile/command proof. |
| 4 | Independent fix review | An expired reservation could be reclaimed while the original process remained live, and a later failure could overwrite operator `suspended`/`retired`. | Blocking | Fixed with a process-held PostgreSQL advisory fence, post-lock reservation revalidation, lifecycle-preserving failure SQL, forced-expiry proof, and suspension/failure race proof. |

- Codex Security diff scan
  `fd5b4a0d-2bf3-49f1-a372-72268fab8ebf` completed and sealed against
  snapshot
  `codex-security-snapshot/v1:sha256:6b542930912a6977ff8130f883ee81f2178199a4852b76df46ff824657449b4d`
  with complete changed-file coverage, one medium and one low reportable
  finding, and no unreviewed changed surface. Both findings are fixed above.
- The fresh fix investigator independently confirmed both reportable paths and
  the operation race. The single required patch review found the expiry/live-
  process and lifecycle-overwrite regressions; both were confirmed, fixed, and
  covered by the expanded authority pilot. Phase 3.5 is PASS.

## Phase 4 — Validate

- Tests run:
  - `cargo check -p omarchy-gaming-system-server --all-targets` — PASS after
    migration/reservation changes.
  - `scripts/test-provider-sidecar.sh` — PASS, including exact TLS loopback,
    hostile peer, crash/restart/reconcile, separate dump/restore, template
    assertions, and signed secret-free receipt.
  - `scripts/test-provider-authority-pilot.sh` — PASS after the final security
    fixes, including hostile ambient proxies, live operation fencing, forced
    reservation expiry, suspension-versus-failure ordering, callbacks, and
    independent provider restore.
- Gate run:
- `bin/gate.sh --diff` — PASS for all stages 1–24, including migrated
  PostgreSQL tests, QML smoke, remote-provider conformance, stage 19a sidecar
  operations, the Door Legends authority pilot, recovery, packaging, and
  server-module containment. The required Phase 5 documentation edits made
  that receipt stale by design; authorized delivery will use a final matching
  rerun after archival.
- Skips or pre-existing failures:
- None.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — `SidecarTarget`, broker selection, TLS/SNI/Host preservation,
    signed-protocol reuse, unit coverage, and the real sidecar drill bind one
    exact release to one exact loopback socket.
  - REQ-002 PASS — configuration, egress, hostile-peer, wrong TLS/port/release,
    redirect, proxy, and partial-value cases fail closed without a private-
    network allowlist.
  - REQ-003 PASS — reviewed systemd/config/Caddy templates plus the drill retain
    separate process identity, credentials, paths, limits, PostgreSQL state,
    backups, and lifecycle.
  - REQ-004 PASS — crash, outage, restart, callback recovery, independent
    dump/restore, non-ready command denial, durable reservations, advisory
    fencing, and authenticated reconciliation preserve single authority.
  - REQ-005 PASS — `docs/operators/provider-deployment.md` covers remote and
    co-located TLS/endpoint identity, database and secret custody, quotas,
    monitoring, rotation, suspension, incident, recovery, upgrade, and EOL.
  - REQ-006 PASS — route/catalog/registry review admits no second provider;
    roadmap and intake mark the local SDK/sidecar slice complete while external
    onboarding, public hosting, custody, and support remain separate.
- Docs:
  - Architecture, runbooks, threat model, README, product charter, roadmap, and
    intake are reconciled. OpenWiki update run
    `bea16410-03c7-44db-80c7-ce07779a0092` completed; it reported the existing
    cross-page unresolved-evidence debt as non-blocking warnings and left those
    older sidecars unchanged.
- AAR:
  - AAR-046 is submitted with five failure IDs, four prevention rules, two
    architecture decisions, and effectiveness 5/5; every new ID is registered
    in `docs/planning/knowledge/INDEX.md`.
- Archive:
  - TICKET-046 is closed, the spec/notes pair moves to `pipeline/completed`, the
    ticket moves to `tickets/closed`, and the ticket index has no locally
    actionable open item. The final external-only audit remains delivery work.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first sidecar drill returned `provider_unavailable` during a valid local compatibility exchange. | The fixture retained a 500 ms whole-operation budget while the expanded crash/hostile-peer real-process test adds scheduler and TLS setup pressure. | Raised only the fixture's registered aggregate deadline to 3000 ms and retained one bounded lease/deadline. | Size real-process integration budgets for repeatable local contention while keeping explicit upper bounds. |
| 2 | The first full authority-pilot rerun returned `503` instead of the established `409` for a suspended session command. | The new non-ready guard treated lifecycle `suspended`/`retired` states as transport outage states. | Return the existing local lifecycle conflict for suspended/retired and `ProviderUnavailable` only for provisioning/reconciling/unavailable; rerun the full pilot. | Preserve distinct policy-denial and transport-outage semantics when adding fail-closed short circuits. |
| 3 | Security inspection showed that a command could read `ready`, wait behind another provider operation, and reach the provider after that operation failed the session into recovery. | Availability was checked before an outbound request without reserving the session for the full request/response interval. | Persist one bounded command/reconcile reservation per provider session, fence response projection by its UUID, reject competing work locally, and require reconciliation after expiry. | Concurrency controls that span external effects must be durable and cover admission through authenticated projection, including crash recovery. |
| 4 | The first reservation-enabled authority-pilot run left a completed session's failed reconciliation reservation uncleared. | Failure cleanup inherited an `active`-session predicate even though reconciliation and its reservation also apply to completed provider sessions. | Clear the matching reservation for every registered-provider session while retaining lifecycle-safe availability updates; rerun the full pilot. | Cleanup predicates must cover every state in which the guarded operation can be admitted. |
| 5 | The passing authority test changed the provider restore receipt count from ten to nine in one valid concurrent schedule. | The new local reservation can reject the losing fresh command before the broker creates its previous revision-conflict receipt. | Accept nine or ten receipts for that effect-equivalent race, then ten or eleven after adding the lifecycle-race reconcile, while retaining the exact one-provider-mutation assertion. | Assert durable effects and bounded schedule-dependent evidence separately in concurrency tests. |
