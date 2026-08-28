---
aar: AAR-040-production-server-module-base-and-observation-hooks
ticket: TICKET-040
pipeline: production-server-module-base-and-observation-hooks
status: submitted
opened: 2026-08-27
submitted: 2026-08-27
effectiveness: effective
---

# AAR-040-production-server-module-base-and-observation-hooks

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-process-isolated-wasm-server-modules-001` | Ticket 039 ADR and architecture recall | Yes; fixes the artifact, process, WIT, typed-intent, and core-reauthorization boundary. |
| `PR-omarchy-gaming-system-enforce-artifact-bounds-during-file-read-001` | Ticket 039 security finding | Yes; executable bytes must be bounded before allocation and compilation. |
| `PR-omarchy-gaming-system-prune-empty-bounded-state-001` | Ticket 039 dispatcher finding | Yes; bounded live work is insufficient if partition indexes grow forever. |
| `PR-omarchy-gaming-system-use-absolute-containment-helper-paths-001` | Ticket 039 process-boundary finding | Yes; the production launcher must not resolve privileged helpers through inherited `PATH`. |
| `PR-omarchy-gaming-system-apply-host-limits-at-supported-layers-001` | Ticket 039 runtime proof | Yes; each claimed ceiling must be exercised on the real production host path. |
| `PR-omarchy-gaming-system-bind-operator-provenance-to-admitted-server-001` | Ticket 039 cross-document binding review | Yes by boundary, though operator-custom provenance remains deferred to Ticket 041. |
| `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001` | Provider/dispatcher concurrency recall | Yes; first-delivery receipt idempotency needs a pre-existing lock root. |
| Ticket 039 completed notes and OpenWiki server-module page | Nearest pipeline and generated-memory recall | Yes; preserves the exact observation-before-admission rollout and production-loader authorization line. |

## What happened

Ticket 040 turned ADR-0004's isolated proof into the first production module
slice without creating a general plugin loader. The server can opt into one
exact compiled-in Sentinel release. A persona-report transaction emits a
metadata-only observation to a bounded PostgreSQL outbox, a separately
contained no-WASI host may propose one numeric moderation label, and core
reauthorizes and records the effect. Release/admission inventory, namespaced
configuration and state, lifecycle, retries, circuit behavior, immutable
receipts, observation-gap evidence, recovery commands, and restore review all
remain core owned. With no module configuration—or with persisted policy that
keeps the optional module inactive—the ordinary server remains available.

Implementation and inspection found several plausible-looking boundaries that
were not yet durable enough: receipt identity varied by retry attempt, stop
state did not always survive restart, a pairwise subject was trusted instead
of re-derived, readiness errors could leak a child, operation UUIDs were
action-local, optional observation failures could reject core reports, graceful
drain could race dispatcher shutdown, local command files were path-trusted,
pruning could remove request preimages, and readiness finalization did not
initially recheck every revision. Independent post-patch review then found the
remaining inactive-startup denial and confirmed the shutdown race. All were
fixed and covered by focused or full-gate evidence.

The first OpenWiki finish occurred while the durable spec still recorded
Phase 3.5, so the hook correctly retained Ticket 039's completion receipt. The
existing `PR-omarchy-gaming-system-advance-durable-phase-before-phase-tools-001`
identified the sequencing error: Phase 4 was recorded, stale generated claims
and duplicate claims were reconciled, and a clean follow-up lifecycle issued
the matching Ticket 040 receipt.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-module-receipt-attempt-identity-drift-001` | A legitimate at-least-once retry changed the immutable request digest because the one-based transport attempt was part of receipt identity. | Focused PostgreSQL replay test. |
| `BF-omarchy-gaming-system-module-degradation-reactivation-gap-001` | Circuit degradation changed lifecycle but left activation permission true, allowing restart to bypass explicit recovery. | Lifecycle/restart inspection and fault test. |
| `BF-omarchy-gaming-system-module-pairwise-subject-sink-trust-001` | Core apply trusted the pairwise partition stored at claim time instead of deriving it again from the authoritative report subject. | Authorization source-to-sink inspection. |
| `BF-omarchy-gaming-system-module-readiness-child-leak-001` | Invalid readiness propagated before the spawned host was explicitly terminated and reaped. | Process-error-path inspection. |
| `BF-omarchy-gaming-system-module-operation-id-action-alias-001` | One data-operation UUID could be reused for a different action because uniqueness and replay lookup were action-scoped. | Idempotency/schema inspection. |
| `BF-omarchy-gaming-system-optional-observation-core-availability-coupling-001` | Inactive or saturated optional observation delivery could reject the authoritative report transaction. | Codex Security availability trace. |
| `BF-omarchy-gaming-system-module-shutdown-drain-race-001` | HTTP graceful drain could complete its notification edge before the module dispatcher synchronously observed shutdown and stopped claiming work. | Codex Security and independent post-patch concurrency review. |
| `BF-omarchy-gaming-system-module-admin-command-file-substitution-001` | Administrator mutation files were bounded but could be symlinked, shared, permissively readable, or replaced across the read. | Codex Security local-input trace. |
| `BF-omarchy-gaming-system-module-delivery-preimage-pruning-loss-001` | Delivery receipts retained response bytes but only a request digest, preventing independent reconstruction after outbox pruning. | Codex Security audit trace. |
| `BF-omarchy-gaming-system-module-readiness-finalization-race-001` | Host readiness ran outside SQL and finalization did not initially compare every configuration/state revision used by the signed admission. | Phase 3.5 concurrency review. |
| `BF-omarchy-gaming-system-inactive-module-core-startup-denial-001` | Persisted degraded, suspended, or restore-review state was mapped to fatal startup even though the module is optional. | Independent post-patch availability review. |
| `BF-omarchy-gaming-system-module-saturation-documentation-drift-001` | The system overview still said queue saturation rejects reports after implementation changed the contract to fail-open aggregate gap evidence. | Independent post-patch documentation review. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bind-receipt-identity-to-stable-semantics-001` | Bind an immutable receipt to stable semantic request facts and explicitly exclude mutable delivery-attempt metadata. | At-least-once retries must reconcile to the first durable effect instead of becoming conflicts. |
| `PR-omarchy-gaming-system-persist-extension-stop-state-001` | Every circuit, suspension, or restore stop state must persist an explicit activation gate that startup cannot infer away. | Lifecycle naming alone does not prevent restart from reactivating work. |
| `PR-omarchy-gaming-system-rederive-opaque-identities-at-effect-sink-001` | Re-derive pairwise or opaque identifiers from authoritative roots at the protected effect sink. | Transport partitions and cached claims are not authorization facts. |
| `PR-omarchy-gaming-system-own-child-cleanup-after-spawn-001` | Every error edge after spawning a child must explicitly terminate and reap that exact child before returning. | Structured error propagation does not own process cleanup automatically. |
| `PR-omarchy-gaming-system-scope-operation-uuid-to-whole-command-001` | Treat an operation UUID as the identity of the entire command and compare its action plus digest on replay. | Method-local idempotency namespaces permit cross-action aliasing. |
| `PR-omarchy-gaming-system-fail-open-optional-observation-hooks-001` | Optional post-commit observation modules must not decide core startup or authoritative transaction availability; retain bounded aggregate gap evidence when observation is unavailable. | Optional extension outages must remain visible without becoming a core-service denial. |
| `PR-omarchy-gaming-system-signal-extension-shutdown-before-http-drain-001` | Signal extension dispatchers synchronously at the HTTP graceful-drain edge, then await their bounded workers during service teardown. | An intermediary task or channel acknowledgement can leave a claim window after drain begins. |
| `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001` | Open local mutation documents once with no-follow semantics and verify regular type, effective ownership, exact private mode, single link, bounds, and stable descriptor metadata before and after reading. | A bounded pathname read is still vulnerable to alias, sharing, and replacement attacks. |
| `PR-omarchy-gaming-system-retain-module-request-preimages-before-pruning-001` | Persist bounded canonical request and response preimages plus the authorized target before pruning replayable transport rows. | Digests alone cannot reconstruct or independently audit an external effect. |
| `PR-omarchy-gaming-system-reauthorize-readiness-under-finalization-lock-001` | After out-of-transaction readiness work, lock the stable roots and compare every lifecycle, configuration, state, namespace, activation, restore, and signed-admission revision before finalizing. | Readiness is stale the moment any authority-bearing input changes. |
| `PR-omarchy-gaming-system-reconcile-restored-modules-before-server-start-001` | Run an audited module restore reconciliation against a copied database before any restored server startup, leaving modules disabled pending explicit review and fresh readiness. | PostgreSQL cannot intrinsically distinguish a restored copy from the original live database. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-observation-only-production-server-module-base-001` | Implement the first production module as one opt-in compiled-in reviewed no-WASI observation component with durable fail-open gaps and core-reauthorized typed effects; keep custom installation and admission hooks separately gated. | `../../../architecture/adr-0004-process-isolated-wasm-server-modules.md`; `../../../architecture/server-modules.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Ticket 039's process, artifact, capability, and authority rules
prevented this slice from becoming a generic loader, direct database plugin,
second gameplay backend, or client executable channel. Real PostgreSQL,
process-containment, restore, and full-gate evidence exposed issues that unit
contracts alone could not. The initial OpenWiki receipt sequencing mistake also
showed that an existing workflow rule was not recalled at the moment it was
needed; the corrective lifecycle both applied that rule and reconciled stale
database/QML counts rather than accepting a warning-bearing completion.
