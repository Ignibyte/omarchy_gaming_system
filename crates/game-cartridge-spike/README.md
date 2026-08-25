# Game Cartridge architecture proof

This isolated workspace is the executable proof for Ticket 014. It is not a
production service, public SDK, or supported cartridge format.

The proof demonstrates four boundaries:

- a data-only cartridge is indexed, signed by an ephemeral publisher key, and
  rejected if its content, capability vocabulary, path shape, or resource
  bounds change;
- a trusted broker derives a pairwise persona subject and issues short-lived,
  audience-, game-, version-, session-, and scope-bound grants;
- a separate provider process owns the gameplay revision and returns signed
  views with idempotent command receipts; and
- trusted QML renders only the broker-validated `terminal`, `grid`, and
  `status` node vocabulary. Cartridge QML, JavaScript, URLs, native code, and
  network access do not exist in the package format.

Run the complete proof from the repository root:

```bash
scripts/test-game-cartridge-spike.sh
```

The script creates temporary signing keys and a temporary signed fixture,
starts provider and broker on loopback-only ephemeral ports, verifies the HTTP
flow with a Rust probe, and runs the trusted QML surface offscreen long enough
to record frame timing. Temporary material and child processes are removed on
exit.

All endpoint and key configuration is intentionally environment-driven and
loopback-restricted. Do not deploy these binaries or treat their JSON envelope
as the final production authentication profile; the associated ADR specifies
the additional TLS, registry, rotation, callback, quota, persistence, and
operational work required before remote providers can be enabled.
