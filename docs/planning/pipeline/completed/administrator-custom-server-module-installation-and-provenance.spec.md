---
title: Administrator custom server-module installation and provenance
pipeline_id: e07910b9-995b-4767-b464-c86ba883bd5a
status: Phase 5 — Complete PASS
ticket: TICKET-041
ticket_doc: docs/planning/tickets/closed/TICKET-041-administrator-custom-server-module-installation-and-provenance.md
aar: docs/planning/knowledge/aar/AAR-041-administrator-custom-server-module-installation-and-provenance.md
created: 2026-08-27
---

# Administrator custom server-module installation and provenance — spec

## Intent

Ship the deliberate owner-operated escape hatch above Ticket 040: a local
administrator can import and lifecycle-manage an exact publisher-signed Wasm
component while OmarchyGS preserves its existing WIT, capability, process,
state, receipt, recovery, gameplay-authority, and client-code boundaries. Any
server running operator-custom behavior discloses that support boundary to its
players without publishing executable bytes or private operator inventory.

## Scope

- In:
  - bounded database-local operator-custom module import, immutable artifact
    custody, explicit publisher/key/capability review, and server-bound custom
    provenance;
  - expected-revision enable, disable, suspend, recover, upgrade, immediate
    rollback, terminal removal, restore review, audit, and retained evidence;
  - the shared Ticket 040 no-WASI host, exact WIT, typed hook/intent,
    capability, state, dispatcher, receipt, and containment policy;
  - bounded public discovery and trusted-QML warning surfaces for active
    operator-custom behavior;
  - operator terms, privacy/telemetry, security contact, patching,
    backup/recovery, incident, and support-responsibility documentation.
- Out:
  - remote or public module administration, self-service marketplace approval,
    arbitrary egress or hostcalls, admission hooks, native/QML/JavaScript
    delivery, direct protected-state mutation, game-provider substitution,
    automatic trust, and federation.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an administrator imports a custom module, the system shall require a bounded private database-local command, exact publisher integrity, operator-custom provenance, explicit unreviewed-code acknowledgement, requested/granted capability review, compatible WIT/component bytes, immutable staging, actor/reason, and an idempotent operation UUID. | Admin integration plus malformed/path/key/capability/replay hostile corpus. |
| REQ-002 | When a custom release is enabled, upgraded, rolled back, disabled, suspended, removed, or recovered, the system shall require expected lifecycle/state revisions, host readiness, compatible atomic namespace migration, retained rollback, immutable audit, and no delivery outside the exact active admission. | Concurrent lifecycle, migration, crash, rollback, and recovery drills. |
| REQ-003 | When a custom module affects server behavior visible to players, discovery and the trusted client shall disclose stable server identity, operator-custom status, module-behavior capability summary, and support boundary without sending component bytes, raw code, private inventory, or signing authority to the client. | API/QML schema, privacy, keyboard/accessibility, and hostile provenance tests. |
| REQ-004 | When custom and marketplace-vetted modules execute, the system shall apply the same WIT, capability, resource, state, dispatcher, receipt, sandbox, and conformance rules while keeping their attestations, warnings, audit, and support claims distinct. | Shared conformance matrix and trust-claim audit. |
| REQ-005 | When an operator bypasses marketplace review, the system shall not claim OmarchyGS review/support, expose server/database credentials, permit arbitrary native/QML/JavaScript/client delivery, allow direct protected-state mutation, or let a general hook become a game provider. | Security review, source/route/client inventory, and real containment drill. |
| REQ-006 | When the slice completes, it shall document terms, privacy/telemetry, security contact, patching, backup/recovery, incident, and operator responsibility expectations and pass security/CodeGraph/OpenWiki/local gate evidence. | Documentation audit and local diff gate. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Installation and lifecycle mutation remain bounded database-local CLI operations over owner-private descriptor-validated files; no HTTP administration route is added. | The server owner needs an escape hatch without exposing a remote executable-upload surface. |
| 2 | A custom release is one exact publisher-signed Component Model artifact using the existing production WIT major and no-WASI host; publisher integrity, operator trust, core admission, and measured containment remain separate evidence. | A valid signature or local acknowledgement must never imply capability or runtime authority. |
| 3 | The administrator explicitly acknowledges the unreviewed-code warning, exact publisher-key fingerprint, requested hooks/capabilities, and the granted subset; no requested power is auto-granted. | Custom trust is a conscious server-owner decision and cannot manufacture marketplace review. |
| 4 | Bounded canonical component and trust documents are retained immutably in PostgreSQL; runtime paths are core-created private materializations, never operator-selected execution paths. | Database custody makes backup/restore and exact-digest recovery self-contained while avoiding a path-substitution loader. |
| 5 | Upgrade stages a new immutable release, migrates an isolated candidate namespace, proves readiness, and atomically changes the exact admission. Rollback targets only the retained immediate predecessor/snapshot. Terminal removal stops execution but preserves artifact, provenance, receipt, state-disposition, and audit tombstones. | Exact recovery evidence is more important than deleting a small bounded artifact, and arbitrary downgrade graphs are deferred. |
| 6 | Any active operator-custom module produces a bounded public aggregate disclosure tied to the stable server UUID and a persistent trusted-client warning; component bytes, module IDs, private configuration/state, operator identity, and signing material remain private. | Players need informed consent about server behavior without receiving an executable channel or an inventory oracle. |
| 7 | The Ticket 040 dispatcher, typed effect, state, receipt, sandbox, resource, shutdown, and restore invariants are shared across provenance classes; provenance changes claims and warnings, not runtime power. | Custom modules must not gain a weaker execution path. |
| 8 | The only initially admitted hook/intent family remains `persona_reported` → `moderation_add_label`; additional hooks, admission decisions, egress, or gameplay authority require separate tickets. | This ticket proves custom custody and lifecycle rather than silently expanding the extension authority model. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-041-administrator-custom-server-module-installation-and-provenance.md`
- Architecture: `docs/architecture/adr-0004-process-isolated-wasm-server-modules.md`; `docs/architecture/server-modules.md`
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope and decisions recorded |
| 2 Design | Architecture, file manifest, regression plan | CodeGraph design receipt |
| 3 Implement | Code matching the design | focused compile/tests and self-review |
| 3.5 Inspect | Independent findings ledger and fixes | fresh CodeGraph inspection receipt |
| 4 Validate | Tests run and delivery gate green | matching gate receipt |
| 5 Complete | AC audit, OpenWiki, docs, submitted AAR, archive | matching completion receipt and no silent drops |
| Delivery | Fresh gate, staged review, authorized commit/push | matching receipt and remote readback |
