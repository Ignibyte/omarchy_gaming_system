---
aar: AAR-047-usurper-v020e-provenance-and-rust-port-map
ticket: TICKET-047
pipeline: usurper-v020e-provenance-and-rust-port-map
status: submitted
opened: 2026-08-30
submitted: 2026-08-30
effectiveness: effective
---

# AAR-047-usurper-v020e-provenance-and-rust-port-map

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Product charter and ADR-0002 | Historical-port scope and separate-repository review | Yes — require verified source/assets and independent game history. |
| AARs 017, 019, and 045–046 | Cartridge/provider knowledge search | Yes — supply the clean-repository, provider-authority, conformance, and deployment boundaries. |
| `PR-omarchy-gaming-system-gate-independent-executable-source-trees-001` | Knowledge register search | Yes — future game-source changes must invalidate matching delivery evidence. |
| Usurper v0.20e archive and source commit | Upstream provenance research | Yes — establish the original-author baseline and expose bundled-code/asset audit needs. |

## What happened

Ticket 047 established Usurper v0.20e as the first historical BBS port target
without beginning the port. The original 3,323,989-byte release archive is now
available in an ignored adjacent workspace, pinned by SHA-256, alongside a
detached clean clone of the publisher-linked parentless source commit. A
twelve-artifact manifest, provenance report, and Pascal-to-Rust build map are
the only tracked outputs in the game workspace.

Direct archive comparison found that `Source20e.zip` contains two game-source
trees: the outer 132-file tree matches the Git commit, while the nested
131-file copy changes six units and omits one. License review also found that a
distribution-level GPL notice does not settle every bundled unit or artist-
credited ANSI asset. The selected policy uses GPL-marked original game logic,
reimplements historical infrastructure, and keeps binaries and art as
reference-only until individually cleared.

The port map preserves Borland scalar widths, integer/RNG/call ordering,
canonical seed tables, state transitions, and the fixed maintenance sequence
inside deterministic reducers with injected clock and random sources. It
assigns rules and realm persistence to a separate provider database and leaves
the platform as identity, admission, session, and trusted-rendering authority.
The first build slice is one deterministic BBS day. Shared realm state, trusted
alias entry, public toolkit licensing, mature-content policy, and art clearance
remain explicit later prerequisites.

All acquisition/inventory checks and the complete local diff gate passed.
OpenWiki completed without factual-page changes because no platform behavior
was implemented; documenting the proposed port there would have confused
future intent with current capability. No upstream bytes entered the platform,
and neither repository was committed or published.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-usurper-nested-source-baseline-ambiguity-001` | The original release's source bundle contains two game trees with the same apparent lineage but different behavior: the nested copy changes six units and omits `CHESTLO.PAS`. | Safe extraction, file-count comparison, and byte comparison against the publisher-linked Git tree. |
| `BF-omarchy-gaming-system-usurper-distribution-license-overbreadth-001` | A top-level GPL distribution notice could be mistaken for complete clearance even though 30 source/assembly units lack the Usurper header and ANSI art has artist credits without a per-asset grant. | File-header, bundled-component, and `USUTEXT.DAT` provenance review. |
| `BF-omarchy-gaming-system-usurper-provider-state-topology-mismatch-001` | The generic Provider starter's database-free 32 KiB session state can prove a solo slice but cannot own Usurper's eventual shared king, market, NPC, social, and daily-maintenance realm. | CodeGraph trace and direct Provider SDK/starter contract review against the Pascal state map. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-authenticate-duplicate-upstream-trees-001` | Authenticate every duplicate or nested upstream source tree against an immutable publisher-linked commit before choosing a port baseline. | Archive layout and directory names are not evidence that two historical copies are equivalent. |
| `PR-omarchy-gaming-system-classify-bundled-corpus-rights-by-artifact-001` | Classify source units, bundled libraries, binaries, generated records, text, and art separately; a distribution license is not blanket proof for every included artifact. | Faithful ports often inherit mixed historical provenance even when the principal source has a clear license. |
| `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` | Before selecting a generic provider starter, compare its state size, transaction, identity, input, and shared-realm seams with the target game's authoritative topology and stage mismatches explicitly. | A deterministic single-session proof can conceal a later shared-world authority redesign. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Port canonical Usurper v0.20e in a separate GPL-2.0-or-later repository as a deterministic rules/provider system with independent PostgreSQL state and an inert OmarchyGS cartridge; begin with one complete solo BBS day and require a reviewed realm seam before shared-world milestones. | `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md`; Ticket 047 completed notes; adjacent `docs/RUST_PORT_MAP.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective (5/5). The provenance-first slice prevented the stale nested source,
unresolved infrastructure, binaries, and ANSI assets from becoming accidental
port inputs. Direct Pascal review exposed the deterministic arithmetic, RNG,
state, and maintenance obligations before Rust types or migrations could lock
in a weaker model. Platform graph review also found the precise solo/shared-
realm and alias-input gaps early enough to stage them honestly. Every ticket
requirement is backed by reproducible local evidence, while implementation,
admission, publication, and unsupported rights claims remain outside scope.
