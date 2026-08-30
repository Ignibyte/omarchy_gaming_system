#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"

cleanup() {
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo cmp cp diff find git rg sha256sum tar; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

cd "$ogs_root"

package_public_crates() {
  local ogs_target="$1"
  local ogs_sdk_patch="patch.crates-io.omarchygs-provider-sdk.path=\"$ogs_root/crates/provider-sdk\""

  cargo package -p omarchygs-provider-sdk --allow-dirty --no-verify \
    --target-dir "$ogs_target" >/dev/null
  cargo package -p omarchygs-provider-starter --allow-dirty --no-verify \
    --target-dir "$ogs_target" --config "$ogs_sdk_patch" >/dev/null
  cargo package -p omarchygs-provider-conformance --allow-dirty --no-verify \
    --target-dir "$ogs_target" --config "$ogs_sdk_patch" >/dev/null
}

package_public_crates "$ogs_temp/package-one"
package_public_crates "$ogs_temp/package-two"

for ogs_name in omarchygs-provider-sdk omarchygs-provider-starter omarchygs-provider-conformance; do
  cmp \
    "$ogs_temp/package-one/package/$ogs_name-0.1.0.crate" \
    "$ogs_temp/package-two/package/$ogs_name-0.1.0.crate"
done

mkdir -m 700 -- "$ogs_temp/packages"
for ogs_crate in "$ogs_temp/package-one/package/"*.crate; do
  tar -xzf "$ogs_crate" -C "$ogs_temp/packages"
done

ogs_sdk="$ogs_temp/packages/omarchygs-provider-sdk-0.1.0"
ogs_starter="$ogs_temp/packages/omarchygs-provider-starter-0.1.0"
ogs_conformance="$ogs_temp/packages/omarchygs-provider-conformance-0.1.0"

for ogs_package in "$ogs_sdk" "$ogs_starter" "$ogs_conformance"; do
  if rg -n 'path\s*=\s*"(\.\.|/)' "$ogs_package/Cargo.toml" >/dev/null; then
    echo 'public provider package retained a repository path dependency' >&2
    exit 1
  fi
done

if rg -n '(omarchy-game-provider|omarchy-gaming-system-server|pub mod (broker|egress|registry))' \
  "$ogs_starter/src" "$ogs_conformance/src" >/dev/null; then
  echo 'public provider package contains a private platform dependency or module' >&2
  exit 1
fi

ogs_source="$ogs_temp/relay-source"
mkdir -m 700 -- "$ogs_source" "$ogs_source/src"
cp -- examples/provider-relay-forge/Cargo.toml examples/provider-relay-forge/Cargo.lock \
  examples/provider-relay-forge/README.md "$ogs_source/"
cp -- examples/provider-relay-forge/src/*.rs "$ogs_source/src/"
git -C "$ogs_source" init --quiet
git -C "$ogs_source" config user.name 'Relay Forge Clean Room'
git -C "$ogs_source" config user.email 'relay-forge@invalid.example'
git -C "$ogs_source" add --all
env GIT_AUTHOR_DATE='2026-01-01T00:00:00Z' \
  GIT_COMMITTER_DATE='2026-01-01T00:00:00Z' \
  git -C "$ogs_source" commit --quiet -m 'Independent Relay Forge provider'
git clone --quiet --no-hardlinks "$ogs_source" "$ogs_temp/relay-one"
git clone --quiet --no-hardlinks "$ogs_source" "$ogs_temp/relay-two"

ogs_sdk_patch="patch.crates-io.omarchygs-provider-sdk.path=\"$ogs_sdk\""
ogs_starter_patch="patch.crates-io.omarchygs-provider-starter.path=\"$ogs_starter\""
for ogs_clone in "$ogs_temp/relay-one" "$ogs_temp/relay-two"; do
  ! rg 'path\s*=' "$ogs_clone/Cargo.toml"
  cargo test --quiet --locked --manifest-path "$ogs_clone/Cargo.toml" \
    --features conformance --config "$ogs_sdk_patch" --config "$ogs_starter_patch"
  cargo tree --locked --manifest-path "$ogs_clone/Cargo.toml" \
    --config "$ogs_sdk_patch" --config "$ogs_starter_patch" \
    >"$ogs_clone/dependencies.txt"
  if rg '(omarchy-game-provider|omarchy-gaming-system-server)' \
    "$ogs_clone/dependencies.txt" >/dev/null; then
    echo 'Relay Forge pulled a private platform crate' >&2
    exit 1
  fi
done

ogs_consumer="$ogs_temp/kit-consumer"
cp -R -- examples/provider-kit-consumer "$ogs_consumer"
ogs_conformance_patch="patch.crates-io.omarchygs-provider-conformance.path=\"$ogs_conformance\""
cargo generate-lockfile --manifest-path "$ogs_consumer/Cargo.toml" \
  --config "$ogs_sdk_patch" --config "$ogs_conformance_patch" >/dev/null
mkdir -m 700 -- "$ogs_consumer/export-one" "$ogs_consumer/export-two"

ogs_sdk_crate="$ogs_temp/package-one/package/omarchygs-provider-sdk-0.1.0.crate"
ogs_starter_crate="$ogs_temp/package-one/package/omarchygs-provider-starter-0.1.0.crate"
ogs_conformance_crate="$ogs_temp/package-one/package/omarchygs-provider-conformance-0.1.0.crate"
for ogs_export in "$ogs_consumer/export-one" "$ogs_consumer/export-two"; do
  cargo run --quiet --locked --manifest-path "$ogs_consumer/Cargo.toml" \
    --config "$ogs_sdk_patch" --config "$ogs_conformance_patch" -- \
    "$ogs_sdk_crate" "$ogs_starter_crate" "$ogs_conformance_crate" "$ogs_export" \
    >"$ogs_export.identity"
done

cmp "$ogs_consumer/export-one.identity" "$ogs_consumer/export-two.identity"
diff -ru "$ogs_consumer/export-one" "$ogs_consumer/export-two"

if rg -a --fixed-strings -- "$ogs_root" "$ogs_consumer/export-one" >/dev/null; then
  echo 'developer kit leaked the OmarchyGS source path' >&2
  exit 1
fi
if rg -a '(DATABASE_URL|PRIVATE KEY|platform_session|persona_id|account_id)' \
  "$ogs_consumer/export-one" >/dev/null; then
  echo 'developer kit contains a private credential or platform identity marker' >&2
  exit 1
fi

find "$ogs_consumer/export-one" -type f -print0 | LC_ALL=C sort -z \
  | while IFS= read -r -d '' ogs_file; do
      printf '%s  %s\n' "$(sha256sum "$ogs_file" | awk '{print $1}')" \
        "${ogs_file#"$ogs_consumer/export-one/"}"
    done >"$ogs_temp/developer-kit-checksums.txt"
[[ -s "$ogs_temp/developer-kit-checksums.txt" ]]

echo 'public provider developer kit and two clean Relay Forge builds passed'
