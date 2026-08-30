---
title: Provider starter, conformance kit, and second clean-room game
pipeline_id: 956c841d-af4b-4e55-a13f-e6a9d143a231
status: Phase 5 — Complete PASS
ticket: TICKET-045
ticket_doc: docs/planning/tickets/closed/TICKET-045-provider-starter-conformance-and-second-game.md
aar: docs/planning/knowledge/aar/AAR-045-provider-starter-conformance-and-second-game.md
created: 2026-08-30
completed: 2026-08-30
---

# Provider starter, conformance kit, and second clean-room game — spec

## Intent

Turn the public protocol preview into a reusable backend development surface by
shipping game-agnostic durable operation handling, a portable conformance/fault
runner, and a distinct clean-room game proof, while preserving OmarchyGS as the
only provider-admission authority.

## Scope

- In:
  - public starter transport/persistence/runtime and deterministic rules seam;
  - public conformance library/CLI, fixtures, fault injection, and receipts;
  - Relay Forge clean-room provider with separate database and keys;
  - reproducible export, packaged clean-clone builds, real broker integration;
  - focused and canonical local evidence.
- Out:
  - co-located sidecar profile and service deployment operations;
  - provider onboarding, public admission/discovery, or new player routes;
  - hosted publication or CI/CD;
  - changing Door Legends production authority.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the starter receives launch, command, or reconcile traffic, it shall authenticate and validate the exact SDK contract before invoking game rules, persist expected-revision and whole-operation idempotency state in its own PostgreSQL database, and produce stable retry receipts and authenticated callbacks without platform database access or compiled fallback. | Starter unit/PostgreSQL tests and real broker callback/outage/restart/reconcile exercise. |
| REQ-002 | When a developer supplies game rules, the starter shall keep deterministic game state transitions behind a narrow game trait and keep transport, grant/signature verification, compatibility, persistence, callback delivery, configuration, and process lifecycle game-agnostic. | Rule-only unit tests, two distinct game implementations, dependency review, and substitution tests. |
| REQ-003 | When the portable conformance kit targets a provider, it shall exercise valid compatibility/launch/command/reconcile/callback flow and a bounded published fault corpus covering replay, changed intent, stale revision, timeout/unknown outcome, outage/restart, signature/digest/context mismatch, malformed/oversized input, callback deduplication, and recovery. | Standalone CLI/library runs with bounded machine-readable receipts against the starter and clean-room provider. |
| REQ-004 | When Relay Forge is built from a clean source tree, it shall consume only packaged public Provider SDK/starter/conformance artifacts, own distinct rules, keys, process, and PostgreSQL state, pass the public conformance kit, and integrate through the real OmarchyGS broker with no repository path dependency or private platform hook. | Two clean clones, dependency/path scans, deterministic rule tests, real TLS/broker run, and database separation assertions. |
| REQ-005 | When the expanded public developer kit is exported twice, its sources, templates, migrations, schemas, fixtures, checksums, provenance, and builds shall be byte-identical and verify from fresh repositories without OmarchyGS source, platform credentials, or private keys. | Exact inventories, signed release verification, two-export comparison, packaged dependency trees, and clean-clone builds. |
| REQ-006 | When starter or conformance documentation describes identity and authority, it shall state that providers receive only pairwise game-scoped subjects and scoped grants and never gain account/persona identity, reusable device credentials, platform database access, arbitrary egress, client executable privilege, direct client connectivity, registration, activation, discovery, trust, or publication authority. | Documentation contract assertions, serialized traffic scan, public API diff, and operator-boundary review. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Relay Forge is the second game: a small deterministic resource-building state machine with commands and terminal completion unlike Door Legends exploration. | It forces rule substitution and expected-revision behavior without adding presentation or product scope. |
| 2 | Starter, conformance, and example artifacts are project-owned local preview packages and are not published to crates.io or another hosted registry. | Clean-clone reproducibility is sufficient technical release proof without inventing legal or external operations authority. |
| 3 | The starter owns only provider-side transport, persistence, callbacks, and lifecycle; the SDK remains the protocol owner and OmarchyGS remains registry, broker, egress, quota, audit, and admission owner. | It preserves the Ticket 044 public/platform split and prevents a starter from becoming a shadow platform. |
| 4 | The clean-room integration may be registered only inside ephemeral conformance databases and tests; Door Legends remains the sole production provider. | Real broker proof does not authorize an external or second production release. |
| 5 | Co-located transport and operational deployment remain a separate Ticket 046 slice. | They require a dedicated threat-model decision and should not inherit test-only loopback allowances. |

## Linked artifacts

- Ticket: [TICKET-045](../../tickets/closed/TICKET-045-provider-starter-conformance-and-second-game.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Intake: [Public Provider SDK](../../intake/public-provider-sdk-starter-and-sidecar.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, second-game selection | autonomous scope lock plus tool readiness |
| 2 Design | Starter/rules/persistence, conformance/fault, export, integration manifests | worktree-bound CodeGraph receipt plus direct unsupported-file review |
| 3 Implement | Public packages, Relay Forge, clean-clone and real-broker proof | focused compile/tests and self-review |
| 3.5 Inspect | Correctness, security, persistence, authority, compatibility, simplicity | fixes plus fresh CodeGraph receipt |
| 4 Validate | Focused suites and complete local diff gate | matching worktree receipt |
| 5 Complete | AC audit, OpenWiki, AAR, roadmap/intake reconciliation, archive | no silent drops |
| Delivery | Staged review, authorized commit, push, and remote readback | matching local receipt and remote commit |
