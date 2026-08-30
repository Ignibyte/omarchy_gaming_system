---
aar: AAR-042-packaged-reviewed-server-module-release-upgrade-and-rollback
ticket: TICKET-042
pipeline: packaged-reviewed-server-module-release-upgrade-and-rollback
status: submitted
opened: 2026-08-29
submitted: 2026-08-29
effectiveness: effective
---

# AAR-042-packaged-reviewed-server-module-release-upgrade-and-rollback

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-process-isolated-wasm-server-modules-001` | ADR-0004 and Ticket 039 recall | Yes; fixes the exact WIT, no-WASI host, typed-intent, and core-reauthorization boundary across releases. |
| `AD-omarchy-gaming-system-observation-only-production-server-module-base-001` | Ticket 040 AAR and nearest production base | Yes; the reviewed successor remains observation-only and cannot broaden the hook/capability set. |
| `AD-omarchy-gaming-system-operator-custom-server-module-boundary-001` | Ticket 041 AAR and shared runtime | Yes; lifecycle mechanics can be shared while packaged review and custom trust/warnings stay distinct. |
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Owner-operated extension architecture | Yes; packaged first-party review must not be mislabeled as marketplace review. |
| `PR-omarchy-gaming-system-reauthorize-readiness-under-finalization-lock-001` | Ticket 040 readiness race | Yes; upgrade/rollback readiness cannot authorize a later changed state. |
| `PR-omarchy-gaming-system-scope-operation-uuid-to-whole-command-001` | Ticket 040 lifecycle review | Yes; reviewed lifecycle replay must compare the complete action and body. |
| `PR-omarchy-gaming-system-reconcile-restored-modules-before-server-start-001` | Ticket 040 restore review | Yes; a restored or package-mismatched reviewed release remains stopped until exact recovery. |
| `PR-omarchy-gaming-system-fail-open-optional-observation-hooks-001` | Ticket 040 availability review | Yes; unavailable reviewed executable behavior records evidence without denying core service. |
| Ticket 041 completed notes and OpenWiki server-module page | Nearest pipeline and generated-memory recall | Yes; provides the candidate namespace, immediate rollback, stale-admission, shared host, and support-boundary baseline. |

## What happened

Ticket 042 turned the fixed reviewed Sentinel fixture into a bounded two-release
packaged catalog. Release `1.0.0` remains the only initial selection; release
`1.1.0` has a distinct component, release/review identity, and `state/v2`
schema while retaining the same WIT, hook, capability, budgets, process host,
dispatcher, receipt, and core effect authority. Startup registers and
byte-compares both releases, resolves the retained selection by exact UUID, and
never upgrades automatically.

The database-local `reviewed-module-apply` command now performs only the fixed
`1.0.0 → 1.1.0` upgrade or its one-use immediate rollback. It binds a
whole-command UUID and digest, all three mutable revisions, actor/reason, and a
complete bounded candidate state. Readiness runs in the contained host outside
SQL; finalization reacquires the shared registry lock and reauthorizes every
prepared release, lifecycle, configuration, state, schema, activation,
restore, and predecessor root before atomically publishing the new admission,
namespace, release selection, stale-work evidence, audit, snapshot, and
immutable operation receipt. Migration 0028 independently restricts retained
operation evidence to those exact release/schema edges.

Self-review and inspection caught four boundary gaps before completion. The
legacy reviewed state-maintenance path still labeled snapshots with the old
`state/v1` literal; startup could treat a packaged-catalog conflict as fatal;
post-readiness finalization did not separately compare the locked namespace
schema; and the operation table relied on Rust alone for the finite release
graph. The fixes made state maintenance schema-aware, kept exact-package
outages inside the optional-module availability boundary, added the independent
namespace-schema root to final reauthorization, and encoded the allowed edges
in PostgreSQL. A canonical CLI fixture ordering mistake was also corrected by
serializing the production command type.

The focused runtime, contained-host, PostgreSQL, and real CLI suites passed;
the complete database corpus passed twice, once standalone and once inside the
24-stage diff gate. A full changed-source security scan covered eleven files
and reported no vulnerabilities. Fresh CodeGraph inspection confirmed the
final locked transition flow. OpenWiki completed and reconciled the server-
module and quickstart pages; it retained a warning for unrelated pre-existing
quickstart evidence debt without blocking the completed lifecycle.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-reviewed-command-fixture-canonical-order-001` | An ad hoc JSON CLI fixture did not use the production command type's canonical field order and was rejected before exercising the intended transition. | First real reviewed-module CLI integration run. |
| `BF-omarchy-gaming-system-module-state-schema-literal-drift-001` | Legacy reviewed state migration persisted the historical `state/v1` literal after the selected release could legitimately use `state/v2`. | Phase 3 self-review and successor-schema maintenance test design. |
| `BF-omarchy-gaming-system-packaged-catalog-startup-conflict-fatal-001` | A packaged catalog contract conflict escaped the optional-runtime outage mapping and could terminate otherwise healthy core startup. | Phase 3 changed-catalog startup review. |
| `BF-omarchy-gaming-system-module-namespace-schema-finalization-race-001` | Post-readiness finalization compared the instance schema but not the independently mutable locked namespace schema. | Phase 3.5 concurrency and atomicity inspection. |
| `BF-omarchy-gaming-system-reviewed-operation-edge-database-gap-001` | The immutable reviewed-operation table did not encode the finite packaged release graph and relied on application validation for exact edge identity. | Phase 3.5 data-integrity inspection. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-build-canonical-command-fixtures-from-types-001` | Generate canonical command fixtures from the production typed structure before mutating individual hostile fields. | Ad hoc JSON key order can fail the canonical boundary before the intended behavior is tested. |
| `PR-omarchy-gaming-system-audit-persisted-schema-literals-on-version-bump-001` | When adding a schema version, inventory every persisted schema literal and bind maintenance/snapshot operations to coherent live instance, namespace, and release roots. | A literal that was valid for one release silently mislabels later state and can cross restoration boundaries. |
| `PR-omarchy-gaming-system-isolate-exact-package-outages-from-core-startup-001` | Map exact packaged-extension absence or contract mismatch into the optional-runtime outage boundary while keeping malformed configuration and internal database failures fatal. | Optional observation code must fail closed without coupling core player availability to package recovery. |
| `PR-omarchy-gaming-system-reauthorize-independent-persistence-roots-001` | After out-of-transaction readiness, re-lock and compare every independently mutable persistence root even when an earlier query required them to agree. | Preparation-time joins do not prove that separately stored roots remain coherent at final publication. |
| `PR-omarchy-gaming-system-encode-finite-operation-graphs-in-database-001` | When an immutable operation ledger has a finite state graph, encode the exact action/source/target/schema edges in database constraints as well as application validation. | Retained audit evidence should remain self-validating even if a future privileged writer bypasses the normal code path. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-packaged-reviewed-server-module-release-lifecycle-001` | Package a bounded exact reviewed-release catalog, preserve explicit initial selection, and allow only database-local readiness-checked atomic upgrade plus one-use immediate rollback without changing the module's authority or public surface. | `../../../architecture/adr-0004-process-isolated-wasm-server-modules.md`; `../../../architecture/server-modules.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalling exact package identity, whole-command replay,
post-readiness reauthorization, immediate-predecessor rollback, restore review,
and fail-open optional observation prevented the second release from becoming
automatic selection, arbitrary downgrade, remote administration, new module
authority, or a core availability dependency. Inspection found the places
where the old single-release model survived as literals or collapsed two
database roots into one conceptual invariant, and both the application and
database layers now enforce the finite lifecycle. The focused and complete
runtime/database/CLI evidence exercised real contained execution and recovery;
the security scan, CodeGraph readback, OpenWiki lifecycle, and full diff gate
closed the source, structural, documentation, and delivery-proof loops.
