# Bulletins — Omarchy Gaming System

Cross-session notices read during workflow recall. A critical bulletin blocks new work;
warnings and information must be acknowledged. Dismissed entries move to the
archive with a reason.

## Active

| ID | Severity | Posted | Expires | Bulletin |
|---|---|---|---|---|

Severity: `critical`, `warn`, or `info`.

## Archive

| ID | Severity | Posted | Dismissed | Bulletin | Reason |
|---|---|---|---|---|---|
| `BUL-001-initial-push-pending` | warn | 2026-08-24 | 2026-08-25 | GitHub has no `main` branch yet; the two existing commits and current workflow conversion are local, so CI remains remotely unconfirmed. | Remote `main` was created and verified at commit `56965c7115fc35b2d0eaf11378bbe60ee1022ce1`. |
