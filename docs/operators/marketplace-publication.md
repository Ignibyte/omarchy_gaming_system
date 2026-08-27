# Marketplace publication and offline-root operations

Status: the deterministic local publication tooling, offline handoff, immutable
activation, guarded mirror probe, and compromise/rollback drill are
implemented. Official domains, storage/CDN accounts, production monitoring,
real offline media or HSMs, dual-control staffing, and production keys are not
provisioned by this repository.

This runbook is for the marketplace release operator, not an owner-operated
community server administrator. Community servers consume a pinned marketplace
and independently decide which reviewed games to admit.

## Authority and custody

Keep these roles and workspaces separate:

- a publisher signs an inert cartridge release;
- reviewers record bounded plain-text review facts in the publication plan;
- the online publication host verifies the release and SDK, holds the catalog
  private key, signs policy and the snapshot, and prepares a public request;
- the offline custodian holds the mode-0600 root private key and signs only the
  canonical public request inside a network-less workspace;
- the online finalizer verifies that response and builds the immutable static
  tree; and
- hosting/mirror operators copy only finalized public bytes.

The catalog key cannot authorize a client package or its own successor. The
offline root does not sign cartridge publisher claims or server admission.
Never place either private key in a plan, prepared tree, hosted tree, command
output, ticket, or log. The commands require explicit absolute private-key
paths; the repository gate executes `offline-sign` successfully inside a
network namespace with no interfaces.

## Build the operator command

From a reviewed checkout:

```bash
cargo build --locked --release --package omarchygs-marketplace-publisher \
  --bin omarchygs-marketplace-publisher
publisher="$PWD/target/release/omarchygs-marketplace-publisher"
```

All JSON is canonical UTF-8 with no unknown fields. Times are explicit Unix
seconds so a ceremony does not silently inherit an incorrect host clock. The
plan's `publication_id` is `publication-` plus the 20-digit bundle version;
`created_at_unix` must equal `not_before_unix`.

The plan pins:

- channel/marketplace IDs, names, canonical HTTPS origins, bundle/snapshot
  versions, validity, and ceremony time;
- the complete ordered catalog key history and the prior trust digest;
- each reviewed release input directory, publisher key, hosted release path,
  lifecycle policy, reviewer, and review summary; and
- each native package input and hosted path, platform/architecture/version,
  exact filename, source revision/digest, and build-provenance digest.

Inputs live beneath one absolute owner-private directory. A release directory
contains `cartridge.ogsc`, `conformance.json`, and `release.signed.json`; the
publisher public key is a separate plan-pinned file. The SDK directory must be
an exact exported OmarchyGS SDK. Release and package paths are relative and may
not traverse or pass through symlinks.

## Prepare online

The online command holds the catalog key but never reads the root private key:

```bash
"$publisher" prepare \
  /absolute/publication-plan.json \
  /absolute/private-input-root \
  /absolute/exported-sdk \
  /absolute/catalog.private.json \
  /absolute/root.public.json \
  /absolute/previous/trust.signed.json \
  /absolute/new-prepared-directory
```

Use `-` instead of the previous trust path only for bundle 1. Preparation
re-verifies publisher signatures, reconstructed conformance, SDK/host
compatibility, review bounds, key lifecycle, package metadata, and every exact
byte before signing catalog policy and the snapshot. It emits only a private
working directory containing public static inputs, `prepared.json`, and
`offline-request.json`. Compare the receipt and transfer the request over the
approved one-way/removable-media procedure.

## Sign offline

In the isolated root-custody environment, independently review the complete
request and run:

```bash
"$publisher" offline-sign \
  /absolute/offline-request.json \
  /absolute/root.private.json \
  /absolute/offline-response.json
```

The root file must be a regular, owner-owned, single-link mode-0600 file. The
command revalidates the root identity, time window, complete key/package
payload, previous signed trust, and monotonic transition. It creates one new
public response and prints a secret-free JSON receipt. Return both public
outputs through the approved handoff. A root-key mismatch, expired request,
changed byte, rollback, symlink, hardlink, or existing output fails closed.

## Finalize and activate online

Import the response into a new or existing owner-private store:

```bash
"$publisher" finalize \
  /absolute/new-prepared-directory \
  /absolute/offline-response.json \
  /absolute/publication-store \
  2000000000
```

The receipt gives `bundle_version` and `publication_sha256`. The version name
is the 20-digit bundle version, a hyphen, and that digest. Verify and select it:

```bash
version=00000000000000000001-<publication_sha256>
"$publisher" verify /absolute/publication-store "$version" \
  /absolute/root.public.json 2000000000
"$publisher" activate /absolute/publication-store "$version" \
  /absolute/root.public.json 2000000000
"$publisher" verify /absolute/publication-store current \
  /absolute/root.public.json 2000000000
```

Finalization stages a complete private temporary tree, authenticates it, calls
`fsync`, and atomically renames it under `versions/`. Activation authenticates
both candidate and current history before atomically replacing only the
restricted `current` symlink. Failed work never changes `current`. Existing
versions are immutable evidence. The store refuses a seventeenth finalized
version; archive and disposition of older evidence require a separately
reviewed operator procedure rather than automatic deletion.

## Static hosting contract

Serve `current/channel/` at the plan's channel origin and
`current/marketplace/` at its marketplace origin:

```text
versions/<20-digit-bundle>-<manifest-sha256>/
  channel/
    publication.json                 application/json
    trust.signed.json                application/json
    packages/...                     application/vnd.archlinux.package
  marketplace/
    publication.json                 application/json
    snapshot.signed.json             application/json
    releases/.../cartridge.ogsc      application/octet-stream
    releases/.../conformance.json    application/json
    releases/.../release.signed.json application/json
current -> versions/<exact-version>
```

Do not rewrite, compress, transform, add to, hardlink, or partially upload a
version. Deploy the complete version under a new immutable hosting prefix,
verify it, and change the hosting pointer only after every intended mirror has
the exact tree. Caches must not mix files across prefixes.

## Probe and rollout

The production probe resolves only public HTTPS destinations and disables
ambient proxy use, redirects, referers, decompression, credentials, and
connection reuse. It bounds every body, streams packages through exact
size/SHA-256 verification, and authenticates the root, key lifecycle, snapshot,
catalog policies, publisher releases, SDK compatibility, and exact inventory.

Supply the last accepted minimum bundle/snapshot floors and expected
publication digest (or `-` only before the first observation), followed by one
or more channel/marketplace mirror pairs:

```bash
"$publisher" probe /absolute/root.public.json 2000000000 1 1 \
  <expected-publication-sha256-or-dash> \
  https://channel-mirror-a.example/v1/ https://market-mirror-a.example/v1/ \
  https://channel-mirror-b.example/v1/ https://market-mirror-b.example/v1/
```

Every pair must serve the same authenticated publication identity. Partial
rollout, stale bytes, split manifests, wrong media types, missing/extra files,
or alternate roots are unhealthy. Mirrors replicate availability; they do not
become another trust root and are not client-selected fallback authorities.
Keep the bounded JSON probe receipt with the deployment record.

## Catalog compromise and rollback drill

For a suspected catalog-key compromise:

1. stop new publication and preserve the current store and receipts;
2. generate a distinct successor catalog key in the online custody boundary;
3. create a higher bundle and snapshot plan that retains the affected key in
   history as `revoked` with its terminal snapshot, appends the successor as
   `active` at the next snapshot, advances package/bootstrap floors, and pins
   the previous trust digest;
4. repeat prepare, offline review/sign, finalize, verify, mirror probe, and
   atomic activation; and
5. prove the new publication is current, the old version remains historical
   evidence, stale activation is denied, and clients/servers reconcile the
   revocation before security-sensitive effects.

Never “roll back” by lowering a bundle/snapshot or editing `current` manually.
Recovery means activating a higher, complete, root-authorized publication. If
the offline root itself may be compromised, stop publication. This v1 protocol
does not authorize in-band root replacement: distributing a new root requires
a separately reviewed client-bootstrap/package release and incident plan.

## Production readiness still external

Before calling an official channel live, record owners and evidence for domain
and TLS control, immutable object storage/CDN behavior, monitoring/alerting,
catalog and root backup/custody, offline media or HSM recovery, dual control,
review staffing, release policy, retention, compromise communications, and a
real restore/rotation exercise. The repository proves the local protocol and
drill; it does not claim those people or services exist.
