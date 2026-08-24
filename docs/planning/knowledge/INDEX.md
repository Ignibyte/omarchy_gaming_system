# Knowledge register — Omarchy BBS

Search this file before planning or implementation, then read the linked AAR,
pipeline notes, or architecture document. New IDs belong both in the run's AAR
and in this register.

## Standing rules

| ID | Rule | Source |
|---|---|---|
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | A foundation is not complete until the real database migration, HTTP endpoint, and QML consumer run together. | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-secret-fixtures-must-not-match-source-001` | Secret-scanner fixtures must be assembled inside the sandbox rather than stored as a matching literal in source. | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-quality-gates-include-untracked-001` | Pre-staging quality gates must inspect committable untracked files explicitly. | `aar/AAR-001-initial-foundation-and-pipeline.md` |

## Register

| ID | Kind | Source |
|---|---|---|
| `BF-omarchy-bbs-postgres18-volume-layout-001` | failure | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `BF-omarchy-bbs-secret-selftest-self-match-001` | failure | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `BF-omarchy-bbs-untracked-whitespace-blindspot-001` | failure | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | rule | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-secret-fixtures-must-not-match-source-001` | rule | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `PR-omarchy-bbs-quality-gates-include-untracked-001` | rule | `aar/AAR-001-initial-foundation-and-pipeline.md` |
| `AD-omarchy-bbs-agent-work-pipeline-001` | decision | `../../architecture/adr-0001-agent-work-pipeline.md` |
