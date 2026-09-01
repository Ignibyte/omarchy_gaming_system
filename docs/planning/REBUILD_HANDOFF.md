# Pre-rebuild handoff — 2026-09-01

This checkpoint preserves the platform and separate Usurper provider before the
development machine is rebuilt. The two repositories are coupled for local
development but remain separate trust, process, persistence, and publication
boundaries.

## Repositories

| Repository | Branch | Role | Delivery boundary |
|---|---|---|---|
| `https://github.com/Ignibyte/omarchy_gaming_system` | `main` | OmarchyGS platform, provider SDK/conformance, cartridge tooling, and trusted QML renderer | Existing public repository |
| `https://github.com/Ignibyte/omarchygs_usurper` | `main` | Separate deterministic Usurper v0.20e Rust provider and inert cartridge | Keep private until the Provider SDK has an explicit compatible public copyright grant |

The repositories should normally be cloned as sibling directories named
`omarchy_bbs` and `omarchygs_usurper`. The Usurper scripts accept
`OMARCHYGS_PLATFORM_ROOT` when another layout is required.

## Workflow state

- Tickets 047 through 058 and their AAR/spec/notes records are complete in the
  platform repository.
- No pipeline or local ticket remains active. Ticket 058 began this handoff at
  Phase 3.5, then passed validation and completion before delivery.
- Ticket 058 implements rules/state/cartridge v11 and the exact level-six
  normal dungeon band in the separate Usurper repository. Focused crate tests,
  signed-cartridge/QML smoke, CodeGraph inspection, and a standard security
  scan were recorded before this handoff. Phase 4 full validation, visible
  preview confirmation, Phase 5 OpenWiki reconciliation, AAR submission, and
  pipeline archival were unfinished when the handoff began and subsequently
  completed. The completed notes are authoritative for the final disposition.

The authoritative completed records are:

- [`TICKET-058`](tickets/closed/TICKET-058-usurper-level-six-dungeon-band.md)
- [completed spec](pipeline/completed/usurper-level-six-dungeon-band.spec.md)
- [completed notes](pipeline/completed/usurper-level-six-dungeon-band.notes.md)
- [`AAR-058`](knowledge/aar/AAR-058-usurper-level-six-dungeon-band.md)

## Rebuild and verification

1. Clone both repositories as siblings and install the toolchains described by
   the platform setup documentation.
2. Reconstruct `omarchygs_usurper/upstream/v0.20e/` from the immutable sources
   and identities in `docs/UPSTREAM_PROVENANCE.md` in the Usurper repository.
   The upstream corpus is reference evidence and is intentionally ignored.
3. From the Usurper repository, run `scripts/prepare-provider-kit.sh` to export
   the adjacent platform SDK/starter packages into the ignored local kit.
4. Run `scripts/test.sh`, `scripts/test-cartridge.sh`, and
   `scripts/test-provider.sh` sequentially. The provider test requires Docker,
   PostgreSQL client tools, and the platform repository.
5. From the platform repository, run `scripts/check-pipeline-tools.sh`, restore
   any required CodeGraph evidence through the documented workflow, and run
   `bin/gate.sh --diff` after the last gated edit.
6. Treat Ticket 058 as completed history. Start any new non-trivial change
   through the normal workflow and record only commands and previews that
   actually run on the rebuilt machine.

## Deliberately untracked state

The following Usurper paths are rebuildable or sensitive and must remain
untracked: `target/`, `upstream/`, `.omarchygs-provider-kit/`, `.preview/`,
`var/`, `.cargo/`, `.env*`, private keys, certificates, and PKCS#12 files.
Platform build output, workflow receipts beneath `.git`, local databases,
containers, generated development credentials, and open preview processes are
also not part of the handoff.

No production registration, admission, deployment, public Usurper release, or
shared-realm persistence is authorized by this checkpoint.
