# Omarchy Gaming System OpenWiki brief

Build a durable engineering map for the API-first social gaming system. Organize
documentation around runtime domains and end-to-end workflows rather than the
directory tree.

Prioritize:

- the PostgreSQL migration → Rust/Axum API → QML connector vertical slice;
- the game-first product boundary, with public boards treated only as a
  possible later complement rather than the current identity;
- server authority, account/persona separation, REST recovery, and notification
  boundaries established in the product and architecture documents;
- local development, validation, delivery receipts, and failure diagnosis;
- the Codex-only work pipeline, including CodeGraph design/inspection evidence,
  OpenWiki completion, tickets, AARs, and knowledge recall;
- focused source and test anchors that help a future change find its owner and
  narrowest verification path.

Keep roadmap intent distinct from implemented behavior. Call out current
limitations, security boundaries, and operational assumptions without turning
planned features into present-tense facts. Do not introduce another agent
runtime, guide, integration target, or provider-driven scheduled workflow.
