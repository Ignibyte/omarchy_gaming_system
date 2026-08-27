# OmarchyGS server-module isolation spike

This nested workspace is executable architecture evidence for Ticket 039. It
is not linked into the production OmarchyGS workspace and does not authorize a
production module loader.

The proof deterministically componentizes inert core Wasm fixtures from the
exact checked-in WIT contract, then exercises one completed component release
in a separate Bubblewrap/systemd-contained host process. The runtime accepts
only completed components; it does not transform publisher input. The
component receives no WASI or other imports. Core-side code verifies distinct
release, provenance, and admission evidence; bounds the framed RPC;
re-authorizes typed intents; and owns replay, lifecycle, configuration, and
state decisions.

The complete proof also disables/rejects hosted CI/CD definitions. Repository
quality and delivery evidence run locally through `bin/gate.sh`.

Run the complete proof from the repository root:

```bash
scripts/test-server-module-spike.sh
```

The proof limits are deliberately small measurements, not ratified production
defaults. General server modules remain disabled in the production server.
