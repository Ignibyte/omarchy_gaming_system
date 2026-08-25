# OmarchyGS trusted Game Cartridge renderer

This production crate compiles one verified cartridge screen and one bounded,
schema-valid JSON view into `omarchygs.render-plan/v1`. The plan contains only
typed inert tags, plain strings, declared action IDs, numeric values, and
SHA-256 asset tokens. It never contains QML, JavaScript, markup, arbitrary
URLs, or publisher filesystem paths.

Profiles:

- `core`: Terminal, Grid, Status, Button, Image, and Meter; 256 active nodes,
  1,024 grid cells, 32 images, 256 KiB view, 1 MiB plan, 1,024 px / 4 MiB per
  raster, and 16 MiB of referenced decoded raster data per scene.
- `rich2d`: Core plus Sprite, ParticleField, and AudioCue; 512 active nodes,
  4,096 grid cells, 64 images, 128 sprites, 2,048 particles, 16 audio cues,
  128 animations, 512 KiB view, 2 MiB plan, 2,048 px / 16 MiB per raster, and
  64 MiB of referenced decoded raster data per scene.

Every Image or Sprite instance is charged before its bytes are published.
Trusted QML images decode asynchronously at a host-selected source size; the
focused gate renders the largest accepted Rich-2D raster and rejects the prior
4,096 px availability trigger before publishing a plan.

The CLI prepares an isolated same-user developer preview:

```text
omarchygs-cartridge-preview prepare \
  <archive.ogsc> <publisher-public.json> <core|rich2d> \
  <view.json> <ready|loading|offline|stale|empty|protocol_error|unsupported_capability|revoked> \
  <preferences.json> <empty-private-output-directory>
```

The output directory must already exist, be empty, and have no group/other
permissions on Unix. The command writes a read-only `render-plan.json` and an
`assets/` directory containing only digest-named authenticated PNG/WAV files.
It emits one JSON receipt and never connects to a provider or database or reads
platform credentials.

Run the complete Rust/CLI/QML/profile matrix from the repository root:

```bash
scripts/test-game-cartridge-renderer.sh
```
