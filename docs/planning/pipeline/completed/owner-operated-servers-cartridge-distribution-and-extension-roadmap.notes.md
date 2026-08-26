---
title: Owner-operated servers, cartridge distribution, and extension roadmap — notes
pipeline_id: c2474cc0-716a-4db5-8223-1f67cea48059
---

# Owner-operated servers, cartridge distribution, and extension roadmap — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: no critical bulletin or active pipeline blocked Ticket 027. Ticket
  026 is completed but intentionally remains uncommitted with its validated
  EXIT-control changes; this documentation slice preserves that worktree and
  requires a fresh combined delivery receipt after any gated documentation
  edit.
- Recall: the product charter currently makes federation and user-supplied
  native plugins private-alpha non-goals. The roadmap places the installer and
  operator controls before external provider onboarding.
- Recall: ADR-0002 and the cartridge architecture already require signed inert
  presentation packages, trusted host QML, one game authority, server-brokered
  provider traffic, and no direct client/provider credential path.
- Recall: production tooling already proves a canonical `.ogsc`, signed
  release/catalog policy, secure local import, trusted Core/Rich-2D rendering,
  a public protocol/model crate, provider security controls, and the sole Door
  Legends first-party remote authority pilot. The main client does not yet
  acquire or mount signed packages.
- Decision: document the owner-operated server/product direction now without
  authorizing marketplace APIs, external providers, federation, arbitrary
  client code, or a server plugin runtime.

## Phase 2 — Design

- Product and authority model:
  - An OmarchyGS deployment is a first-class owner-operated community. The
    operator runs the standard server architecture and curates its catalog;
    players choose that server and see its accounts, personas, relationships,
    games, achievements, and history. Independent servers do not silently
    merge identities or policy, and federation remains a separate future
    system.
  - A vetted marketplace is a distribution and provenance service, not a
    gameplay authority. The server operator imports and activates an exact
    signed release. The server advertises only its admitted releases and their
    provenance to clients. A client acquires the exact `.ogsc` bytes from a
    server-approved distribution path, verifies them, and caches them locally
    for trusted rendering.
  - A cartridge is the portable frontend artifact: manifest, declarative
    presentation, schemas, localization, and bounded assets. "QML side" means
    the official client maps inert nodes into platform-owned QML components;
    it never means publisher-provided QML or executable frontend code.
  - Game backend code is separate. A compiled first-party game may remain in
    OmarchyGS, while a portable backend conforms to the brokered provider
    protocol. The future public Provider SDK packages the existing protocol
    model, starter service, conformance suite, operational contract, and
    version negotiation so the core server need not know game rules.
- Marketplace and sideload flow:
  - Vetted path: marketplace publisher signature and review → server operator
    import/activation and signed catalog policy → server-scoped player catalog
    → exact client acquisition and local content-addressed verification →
    trusted render plan → actions through the selected OmarchyGS server →
    compiled runtime or registered provider.
  - Custom path: an administrator explicitly enables a server-local trust
    domain, signs/imports an inert cartridge under an operator-controlled key,
    and may install separately packaged server extension code. Catalog/API/UI
    provenance must label it as operator-custom rather than marketplace-vetted.
    Bypassing marketplace review never bypasses package bounds, client schema
    checks, trusted rendering, or the ban on direct cartridge networking.
  - Server-side custom code is trusted by and operationally owned by the
    administrator of that machine. The project does not monitor or operate
    independent deployments. Operator/player disclosures and reviewed terms
    are release requirements, but prose disclaimers are not substitutes for
    client isolation, authentication, audit, revocation, or resource bounds.
- Server extension direction:
  - General modules and game backends are different extension families. Game
    rules use the Provider SDK. Server modules use a future versioned module
    manifest, capability grants, typed lifecycle/domain hooks, bounded failure
    behavior, compatibility negotiation, configuration/state namespaces,
    audit, disable/upgrade/rollback policy, and conformance fixtures.
  - Hooks may observe allowlisted events or submit typed intents back through
    core authorization; they do not receive raw credentials, unrestricted
    database handles, or authority to rewrite protected state behind domain
    services. The architecture spike must choose external-process RPC, Wasm,
    statically compiled modules, or another isolation boundary before
    executable extensions are authorized. An unstable dynamic Rust ABI and
    client-side plugins are not the baseline.
- Database and migration consequences: none in this slice. Future catalog,
  server identity, module registry, hook receipts, and provenance fields will
  require forward-only designs in their own tickets.
- API compatibility: no endpoint changes now. Future work requires versioned
  server identity/capability discovery, server-scoped catalog release and
  provenance records, exact bounded cartridge transfer, module administration,
  and provider SDK version negotiation. Existing `/v1/games` and client
  behavior remain unchanged.
- Exact file manifest:
  - `docs/product-charter.md` — make the owner-operated community and curated
    game-library promise explicit without expanding private-alpha scope.
  - `docs/planning/ROADMAP.md` — add ordered marketplace, local acquisition,
    backend SDK, custom-content disclosure, module-spike, and module-system
    outcomes.
  - `docs/architecture/adr-0003-owner-operated-server-and-extension-boundary.md`
    — record the accepted direction and deferred implementation gates.
  - `docs/architecture/system-overview.md` — place deployment ownership and
    extension families in the overall platform topology.
  - `docs/architecture/game-cartridges.md` — define server marketplace import,
    client acquisition/cache, server-local sideload provenance, and the strict
    cartridge/backend/module split.
  - `docs/operators/owner-operated-servers.md` — capture responsibility,
    provenance, custom-content, player disclosure, and pre-public legal-review
    requirements without pretending to supply legal advice.
  - Ticket/spec/notes/AAR, knowledge register, and generated OpenWiki evidence
    — workflow and durable recall only.
- Regression and review matrix:

| Requirement | Evidence |
|---|---|
| REQ-001 | Product charter, ADR-0003, and system overview agree on independent owner-operated communities and no implicit federation. |
| REQ-002 | Game-cartridge marketplace flow distinguishes operator import, server catalog activation, exact client acquisition, verification, cache, and trusted rendering. |
| REQ-003 | Constitution and both cartridge ADRs consistently prohibit raw publisher QML/executable frontend/network access; terminology review rejects "QML cartridge code." |
| REQ-004 | Roadmap and ADR specify a public Provider SDK/starter/conformance outcome downstream of the existing first-party protocol pilot. |
| REQ-005 | ADR, cartridge architecture, and operator guide permit explicit operator-local trust while retaining hard client validation and honest provenance. |
| REQ-006 | Roadmap and ADR specify module base/hooks/capabilities/lifecycle/audit plus a required isolation spike, with no client plugin or dynamic Rust ABI authorization. |

- Risks and mitigations:
  - "Self-hosted" can be mistaken for federation or shared global identity;
    documents state that identity, session, policy, and history are server-local.
  - "Custom code" can be mistaken for safe or marketplace-supported; custom
    provenance is explicit and the operator owns server risk and support.
  - A disclaimer can be mistaken for a security boundary; the official client
    retains inert packages and validated trusted rendering for every server.
  - A general plugin system can become an authority bypass; hooks are typed,
    capability-scoped, audited, and routed through core services.
  - Choosing an ABI prematurely could lock the project to unsafe in-process
    code; this slice requires a separate isolation spike.
  - A marketplace outage or withdrawal must not silently mutate an installed
    release; exact digests remain pinned and lifecycle policy controls future
    launch/update behavior.
- Alternatives rejected or deferred: global mandatory marketplace authority;
  federation as a side effect of multi-server support; cartridges containing
  backend binaries; raw QML/JavaScript/native client plugins; arbitrary URLs;
  unsigned/unbounded sideloads; provider direct-to-client credentials; a
  general plugin hook as a shortcut around the provider protocol; and choosing
  Wasm, subprocess RPC, or an in-process ABI without a dedicated spike.
- Rollback: revert this documentation/ADR slice. It has no runtime, schema,
  package, key, or operator-state consequences.
- CodeGraph evidence: the worktree-bound design exploration traced the current
  `GameRegistry` compiled authority, `GameSession` authority discriminator,
  `ProviderBroker` launch/command/reconcile operations, pairwise grant issuer,
  and their server/API/test dependents. It confirms that the current runtime
  already has two explicit backend authority paths and that a future Provider
  SDK should package the provider protocol rather than broaden
  `GameDefinition` into a network/plugin trait. The query did not surface the
  filesystem cartridge store or QML graph, so direct inspection of the
  canonical package, secure importer, trusted renderer, generated OpenWiki,
  and existing architecture remains authoritative for those documentation
  claims. Receipt pipeline `c2474cc0-716a-4db5-8223-1f67cea48059`, gated state
  `6e32ad0b1b41c6060adac2a10066f9ebb1028e4186e66b292f94f9372b663554`.

## Phase 3 — Implement

- Built `ADR-0003` as the accepted product direction while explicitly leaving
  marketplace, external-provider, operator-custom, federation, and module
  implementation behind later security/operations tickets.
- Updated the product charter and system overview so independently
  owner-operated standard deployments are first-class communities with
  server-local identity, policy, catalog, and history.
- Updated the cartridge architecture with the operator import → server catalog
  → exact client acquisition/cache → trusted renderer → selected-server broker
  flow. The cartridge is now unambiguously frontend-only and a custom server
  cannot use its catalog to send executable QML/code to the official client.
- Added a separate Provider SDK outcome built on the existing public protocol
  seam and kept game backends distinct from the future general server module
  base and typed hook system.
- Added the operator-custom trust class, local signing/import direction,
  provenance labels, module isolation gate, and owner responsibility/support
  boundary. Added an operator guide that calls for reviewed legal/privacy terms
  without claiming engineering prose is legal advice or a security control.
- Expanded the roadmap with saved isolated server profiles, server marketplace
  synchronization, client cartridge acquisition/mounting, Provider SDK,
  custom cartridge trust, module isolation spike, module/hook implementation,
  custom module administration, and terms/disclosure work.
- Focused checks: `git diff --check`, `./scripts/check-pipeline.sh`, and nonempty
  manifest checks passed. No application code, schema, API, or runtime behavior
  changed.
- Deviations: none from the approved documentation manifest. OpenWiki and the
  knowledge/AAR registration remain Phase 5 lifecycle work.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Current-state correctness | `system-overview.md` still said `/v1/games` listed only compiled metadata, and `game-cartridges.md` still called the provider crate uninstantiated after Ticket 019 activated the optional Door Legends bridge. | medium | Resolved: both now distinguish the normal compiled catalog from the optional, all-or-none operator-pinned pilot. Capture `BF-omarchy-gaming-system-provider-activation-documentation-drift-001` and `PR-omarchy-gaming-system-reconcile-foundation-docs-when-activated-001`. |
| 2 | Supply-chain/trust semantics | The first marketplace flow compressed publisher integrity, marketplace review, and server admission into an ambiguous "catalog verification" step. | medium | Resolved: ADR-0003 and the cartridge lifecycle now define three independent attestations; operator-custom content explicitly omits marketplace review. Capture `BF-omarchy-gaming-system-cartridge-distribution-trust-conflation-001` and `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001`. |
| 3 | Self-hosted backend feasibility | The first Provider SDK description covered remote backends but not an owner running custom code beside their OmarchyGS service. | medium hypothesis | Resolved: a future co-located sidecar profile remains a separately identified service with its own state/credentials and requires a reviewed authenticated local transport; it cannot reuse the conformance loopback escape hatch or platform database. |
| 4 | Client safety | Marketplace bypass could be read as bypassing package verification or allowing raw QML on a player's device. | high hypothesis | Closed with no finding: every product, ADR, architecture, and operator surface keeps custom cartridges signed, inert, bounded, content-addressed, and rendered only through trusted QML; arbitrary client code/network remains prohibited. |
| 5 | Authorization/extensibility | A general plugin hook could bypass provider revisions or protected platform domain services. | high hypothesis | Closed with no finding: Provider SDK and server modules are distinct; hooks may observe allowlisted events or submit typed intents through core authorization, and no runtime/ABI is authorized before a separate isolation spike. |
| 6 | Responsibility/disclosure | A blanket "not responsible" statement could be mistaken for legal advice or a sufficient security control. | medium hypothesis | Closed with no finding: the operator guide assigns engineering responsibilities, calls for reviewed terms/privacy language before public distribution, explicitly disclaims legal advice, and preserves technical containment. |
| 7 | Roadmap sequencing | The first edit placed the "later" Provider SDK section above the owner-operated marketplace work. | low | Resolved: ordered owner-operated catalog/acquisition/custom-content outcomes before the remaining provider SDK/onboarding path. |
| 8 | Runtime/blast radius | Fresh CodeGraph inspection found the immutable compiled `GameRegistry`, optional `ProviderRuntime`/`ProviderBroker`, provider catalog tests, and no general plugin loader. | — | PASS: the documentation describes current versus future behavior honestly and changes no runtime, API, schema, authority, QML, package, or hook implementation. Current inspect receipt is bound to pipeline `c2474cc0-716a-4db5-8223-1f67cea48059`. |

- Security/privacy review: independent server origins retain separate account,
  persona, token, catalog, and history authority. The chosen server necessarily
  sees data created there but receives no new client capability. Publisher,
  marketplace, server, provider, module, and client trust are not collapsed;
  neither custom provenance nor administrator ownership bypasses current
  authentication, credential, egress, replay, rendering, or database
  boundaries.
- Simplification review: retained the existing cartridge verifier/store,
  trusted renderer, provider protocol, and standard server architecture as the
  seams. No speculative endpoint, migration, hook list, ABI, or marketplace
  service design was invented.
- Final direct review: every concept requested by the user appears in at least
  one authoritative product/architecture document and the roadmap. Focused
  whitespace and pipeline-structure checks remain green after remediation.

## Phase 4 — Validate

- Tests run: the first `bin/gate.sh --diff` invocation completed the focused,
  Rust, QML, secret, pipeline, and clean-clone stages but did not produce a
  green delivery result because one PostgreSQL MFA integration test returned
  `401` instead of `200` at
  `mfa_api_tests::enrollment_encrypts_secret_confirms_recovery_and_scopes_status:164`.
  The exact failed test was then rerun against the healthy PostgreSQL service
  and passed (`1 passed; 0 failed`).
- Gate run: a fresh full diff gate is required after that focused diagnosis;
  the first run is not delivery proof. The fresh run then completed all 18
  stages with `GATE GREEN [diff]`: 45 PostgreSQL integration tests, 40 QML
  fixture cases plus four live API scenarios, remote-provider conformance, and
  the clean-clone Door Legends authority pilot all passed. Its gated state was
  `6e32ad0b1b41c6060adac2a10066f9ebb1028e4186e66b292f94f9372b663554`.
- Skips or pre-existing failures: the docs-only Ticket 027 did not touch MFA.
  CodeGraph inspection showed the test deliberately generates a previous-step
  TOTP and the server accepts current, previous, and next steps. If execution
  crosses a 30-second boundary after code generation, that code is two steps
  old and is correctly rejected. The isolated immediate pass is evidence of a
  boundary-sensitive pre-existing test, not permission to ignore the red gate.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — the product charter, system overview, ADR-0003, operator
    guide, roadmap, and OpenWiki define independent owner-operated communities
    with server-local identity/policy/history and no implicit federation.
  - REQ-002 PASS — ADR-0003 and the cartridge architecture define marketplace
    publication, administrator import/admission, server-scoped catalog,
    bounded exact client acquisition, verification, cache, and trusted local
    rendering as separate lifecycle steps.
  - REQ-003 PASS — all authoritative surfaces define a cartridge as signed
    inert presentation data interpreted by platform QML and prohibit publisher
    QML/JavaScript, backend code, credentials, arbitrary destinations, or
    direct networking.
  - REQ-004 PASS — the roadmap and ADR specify a future public Provider SDK,
    starter backend, protocol/version helpers, conformance/fault fixtures, and
    operations guidance without expanding current provider authorization.
  - REQ-005 PASS — operator-custom cartridge and server-code paths are
    explicitly labeled local trust, preserve official-client validation, and
    assign independent-server operations/disclosure responsibility to the
    administrator without treating a disclaimer as containment.
  - REQ-006 PASS — the roadmap and ADR define a separate module base with
    versioned manifests, capabilities, typed hooks, namespaced state,
    compatibility, audit, lifecycle, and a required isolation spike; no client
    plugin or unstable in-process Rust ABI is authorized.
- Docs: OpenWiki update run `f2c4cdc3-c907-40cf-9ea8-244a44685836`
  authored the three affected pages and completed with warnings because nine
  older claim references had become stale. Recovery run
  `ed7d0885-f3fe-46a2-a211-92cea066c74e` reconciled those references, reapplied
  the new owner-operated claims, and returned `status: complete` with no
  warning. Generated quickstart, product-boundary, and Game Cartridge pages now
  distinguish implemented behavior from the accepted future direction.
- AAR: submitted at effectiveness 5 with two documentation failures, two
  prevention rules, and ADR-0003 registered as durable knowledge.
- Archive: OpenWiki completion receipt matched pipeline
  `c2474cc0-716a-4db5-8223-1f67cea48059` at gated state
  `f2bd29a8dbb4d7f135a37c5b921131c638f06a454f260f98dd97832e982c2cce`;
  ticket, spec, and notes moved to their closed/completed paths. The final
  post-archive `bin/gate.sh --diff` completed all 18 stages with `GATE GREEN
  [diff]` at the same state: 45/45 PostgreSQL tests, 40/40 QML fixture cases,
  all live API scenarios, provider conformance, and the clean-clone Door
  Legends pilot passed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Foundation architecture still described the provider foundation as dormant and the catalog as compiled-only after Ticket 019. | Later runtime activation did not reconcile every durable current-state summary. | Updated system overview, cartridge architecture, and OpenWiki current-state prose. | `PR-omarchy-gaming-system-reconcile-foundation-docs-when-activated-001` |
| 2 | The first marketplace wording collapsed publisher integrity, marketplace review, and server admission into one ambiguous verification step. | Distribution provenance and local authorization were treated as one generic trust decision. | Defined three independent attestations and the reduced claim set for operator-custom content. | `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` |
| 3 | The first full gate rejected a deliberately previous-window TOTP with `401`; its exact isolated rerun and the fresh full gate passed. | The pre-existing test can cross a 30-second boundary after constructing a previous-step code, making it two steps old while the server correctly accepts only ±1. | Preserved the red result, inspected the verifier path, reran the exact case, then required a fully fresh green gate. | Do not waive a red gate as a flake; diagnose it and obtain new end-to-end evidence. |
| 4 | The first OpenWiki finish completed with nine stale-evidence warnings and withheld affected claim sidecars. | Earlier QML and architecture edits shifted or changed sources referenced by existing generated claims. | Ran a recovery lifecycle that updated every stale reference and completed without warnings. | Treat OpenWiki completion warnings that withhold claims as unfinished durable reconciliation. |
