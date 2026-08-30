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
ogs_package_target="$ogs_temp/package-target"
cargo package -p omarchygs-provider-sdk --allow-dirty --no-verify \
  --target-dir "$ogs_package_target" >/dev/null
cargo package -p omarchygs-provider-sdk --allow-dirty --list \
  >"$ogs_temp/package-files.txt"

if rg '(^|/)(broker|egress|registry|migrations?|admin)(/|\.|$)' \
  "$ogs_temp/package-files.txt" >/dev/null; then
  echo 'Provider SDK package contains a platform-only path' >&2
  exit 1
fi

ogs_crate="$(find "$ogs_package_target/package" -maxdepth 1 -type f \
  -name 'omarchygs-provider-sdk-*.crate' -print -quit)"
[[ -n "$ogs_crate" ]]
mkdir -m 700 -- "$ogs_temp/package"
tar -xzf "$ogs_crate" -C "$ogs_temp/package"
ogs_sdk="$(find "$ogs_temp/package" -mindepth 1 -maxdepth 1 -type d \
  -name 'omarchygs-provider-sdk-*' -print -quit)"
[[ -n "$ogs_sdk" ]]

if rg -n '(use (sqlx|reqwest|tokio|tracing)|pub mod (broker|egress|registry))' \
  "$ogs_sdk/src" >/dev/null \
  || rg -n 'path\s*=\s*"(\.\.|/)' "$ogs_sdk/Cargo.toml" >/dev/null; then
  echo 'Provider SDK package contains a forbidden platform dependency or module' >&2
  exit 1
fi

ogs_source="$ogs_temp/source"
cp -R -- examples/provider-sdk-consumer "$ogs_source"
ogs_patch="patch.crates-io.omarchygs-provider-sdk.path=\"$ogs_sdk\""
cargo generate-lockfile --manifest-path "$ogs_source/Cargo.toml" \
  --config "$ogs_patch" >/dev/null
git -C "$ogs_source" init --quiet
git -C "$ogs_source" config user.name 'OmarchyGS Provider SDK Conformance'
git -C "$ogs_source" config user.email 'provider-sdk-conformance@invalid.example'
git -C "$ogs_source" add --all
env GIT_AUTHOR_DATE='2026-01-01T00:00:00Z' \
  GIT_COMMITTER_DATE='2026-01-01T00:00:00Z' \
  git -C "$ogs_source" commit --quiet -m 'Independent Provider SDK consumer'
git clone --quiet --no-hardlinks "$ogs_source" "$ogs_temp/clone-one"
git clone --quiet --no-hardlinks "$ogs_source" "$ogs_temp/clone-two"

for ogs_clone in "$ogs_temp/clone-one" "$ogs_temp/clone-two"; do
  ! rg 'path\s*=' "$ogs_clone/Cargo.toml"
  cargo run --quiet --locked --manifest-path "$ogs_clone/Cargo.toml" \
    --config "$ogs_patch" -- "$ogs_clone/export" >"$ogs_clone/identity.txt"
  cargo tree --locked --manifest-path "$ogs_clone/Cargo.toml" \
    --config "$ogs_patch" -p omarchygs-provider-sdk -e normal \
    >"$ogs_clone/dependencies.txt"
  if rg '(^| )((sqlx|reqwest|tokio|tracing|url) v|omarchy-game-provider)' \
    "$ogs_clone/dependencies.txt" >/dev/null; then
    echo 'Provider SDK consumer pulled a platform-only dependency' >&2
    exit 1
  fi
  if rg -a --fixed-strings -- "$ogs_root" "$ogs_clone/export" >/dev/null; then
    echo 'Provider SDK export leaked the OmarchyGS source path' >&2
    exit 1
  fi
done

cmp "$ogs_temp/clone-one/identity.txt" "$ogs_temp/clone-two/identity.txt"
diff -ru "$ogs_temp/clone-one/export" "$ogs_temp/clone-two/export"

find "$ogs_temp/clone-one/export" -type f -print0 \
  | LC_ALL=C sort -z \
  | while IFS= read -r -d '' ogs_file; do
      printf '%s  %s\n' "$(sha256sum "$ogs_file" | awk '{print $1}')" \
        "${ogs_file#"$ogs_temp/clone-one/export/"}"
    done >"$ogs_temp/export-checksums.txt"
[[ -s "$ogs_temp/export-checksums.txt" ]]

echo 'public Provider SDK deterministic release passed'
