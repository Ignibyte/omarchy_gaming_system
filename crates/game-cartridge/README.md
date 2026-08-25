# OmarchyGS Game Cartridge

This production workspace crate defines the local `.ogsc` v1 package boundary.
It packages and verifies signed **data only**; it never executes publisher code,
contacts a provider, reads platform credentials, or opens the OmarchyGS
database.

The v1 archive is a deterministic, stored-only ZIP containing:

```text
integrity.signed.json
manifest.json
presentation.json
schemas/<name>.schema.json
locales/<tag>.json                 # optional
assets/<name>.png|wav              # optional
```

Every JSON document is strict and canonical. The signed integrity index binds
the path, media type, length, and SHA-256 digest of every other entry. The
verifier also reconstructs the canonical ZIP and requires byte equality, so an
alternate archive encoding cannot represent the same release.

Every screen pins one declared view schema. The trusted v1 presentation
vocabulary is `terminal`, `grid`, `status`, `button`, `image`, `meter`,
`sprite`, `particle_field`, and `audio_cue`; every node requires an exact host
capability and any optional node uses a typed, node-compatible fallback. Raw
QML, JavaScript, native code, shaders, remote assets, and network destinations
are not valid cartridge content.

Successful verification retains the authenticated payload behind read-only
accessors. Downstream renderers therefore consume the same bytes whose length,
digest, canonical form, schema/media profile, and signature were checked; an
external caller cannot mutate the verified manifest or presentation.

## CLI

```text
omarchygs-cartridge keygen <publisher-id> <key-id> <private.json> <public.json>
omarchygs-cartridge pack <source-dir> <private.json> <output.ogsc>
omarchygs-cartridge conform <archive.ogsc> <public.json> [host-profile.json]
omarchygs-cartridge install <archive.ogsc> <public.json> <store-root> [host-profile.json]
omarchygs-cartridge revoke <store-root> <archive-sha256> <reason>
omarchygs-cartridge sdk-export <empty-sdk-directory>
omarchygs-cartridge sdk-verify <sdk-directory>
omarchygs-cartridge release <source-dir> <private.json> <sdk-directory> <git-revision> <empty-release-directory> [host-profile.json]
omarchygs-cartridge verify-release <release-directory> <public.json> <sdk-directory> [host-profile.json]
omarchygs-cartridge catalog-keygen <authority-id> <key-id> <private.json> <public.json>
omarchygs-cartridge catalog-policy <release-directory> <publisher-public.json> <sdk-directory> <catalog-private.json> <version> <status> <reason> <output.json> [host-profile.json]
omarchygs-cartridge secure-import <release-directory> <publisher-public.json> <sdk-directory> <policy.json> <catalog-public.json> <existing-store-root> [host-profile.json]
```

Commands emit one machine-readable JSON document. `conform` exits `0` for a
compatible valid cartridge, `3` for a valid but incompatible cartridge, and
`2` for rejection. Key and package output paths are create-only to prevent
accidental overwrite.

`sdk-export` emits the versioned, language-neutral schema and lock surface used
by a separate game repository. `release` signs provenance that binds the source
revision, exact builder binary/version, SDK lock, publisher identity, archive,
and conformance report. `secure-import` independently verifies the publisher
release and the platform catalog policy, then performs every descendant store
operation relative to already-open Linux directory descriptors. The older
`install`/`revoke` commands remain a same-user developer boundary only.
The secure store accepts only directories owned by its effective user and
rejects every root or fixed child writable by group/other. Lifecycle policy
read/compare/replace transitions are descriptor-relative and serialized across
processes; the highest authenticated policy is persisted even when its launch
decision is denial, so restart or concurrent import cannot reopen an older
release.

Run the focused production contract gate from the repository root:

```bash
scripts/test-game-cartridge.sh
scripts/test-game-cartridge-sdk.sh
```
