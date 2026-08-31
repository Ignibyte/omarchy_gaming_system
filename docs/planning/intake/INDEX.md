# Project completion intake index

Status as of 2026-08-30: Tickets 044 and 045 implement the public Provider SDK,
starter, portable conformance/fault kit, and Relay Forge clean-room integration.
Ticket 046 is the active locally actionable sidecar/operations slice; external
intakes remain blocked on real people, systems, accounts, or operating proof.

Candidate intake documents are preparation, not completion evidence. Promote
only one shippable slice at a time through the repository workflow.

| Order | Roadmap outcome | Intake | Promotion dependency |
|---|---|---|---|
| 1 | First external two-clean-installation private-alpha acceptance run | [External private-alpha acceptance](external-two-clean-installation-private-alpha-acceptance.md) | Ticket 042 delivery; public HTTPS deployment; two clean external Omarchy installations; two testers and an operator. |
| 2 | Official hosted marketplace origins, custody, monitoring, staffing, retention, and recovery | [Official marketplace operations](official-marketplace-hosting-custody-and-operations.md) | Explicit external account/budget/write authority and named custodians/operators; separately gated root-replacement bootstrap design before its live exercise. |
| 3 | Public Provider SDK, starter backend, negotiation, conformance, sidecar, and operations guide | [Public Provider SDK](public-provider-sdk-starter-and-sidecar.md) | Ticket 042 delivery; SDK release ownership; protocol compatibility policy; selected second clean-room game; sidecar threat-model decision. |
| 4 | Reviewed external providers | [Reviewed external provider onboarding](reviewed-external-provider-onboarding.md) | Public Provider SDK complete; real review/operations/recovery/suspension/support capacity proven; consenting candidate provider selected. |

## Promotion rules

1. Preserve the roadmap order unless the earliest item is externally blocked
   and the reason for promoting the next locally actionable slice is recorded
   in its Phase 1 notes.
2. Do not use local fixtures, containers, VMs, or deterministic rehearsals as
   evidence for external people, clean systems, custody, hosting, paging, or
   an operating observation window.
3. Do not provision paid services, register domains, contact external people,
   publish releases/incidents, or create production keys without explicit
   authority for that external action.
4. Do not treat SDK publication as provider review or admission. Do not treat
   marketplace publication as server admission. Do not let either path grant
   executable authority to the client.
5. Deliver or explicitly disposition the current completed work before opening
   the next active spec/notes pair in the primary worktree.
