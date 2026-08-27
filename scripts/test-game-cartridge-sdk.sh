#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"

cleanup() {
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo cmp cp diff find git mkdir python3 rg sha256sum stat; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

cd "$ogs_root"
cargo test -p omarchygs-game-cartridge --test sdk_release
cargo build -p omarchygs-game-cartridge --bin omarchygs-cartridge
cargo build -p omarchygs-game-cartridge-renderer --bin omarchygs-cartridge-preview

ogs_tools="$ogs_temp/tools"
mkdir -m 700 -- "$ogs_tools"
cp -- target/debug/omarchygs-cartridge "$ogs_tools/omarchygs-cartridge"
cp -- target/debug/omarchygs-cartridge-preview "$ogs_tools/omarchygs-cartridge-preview"
chmod 500 "$ogs_tools/omarchygs-cartridge" "$ogs_tools/omarchygs-cartridge-preview"
ogs_cartridge="$ogs_tools/omarchygs-cartridge"
ogs_preview="$ogs_tools/omarchygs-cartridge-preview"

ogs_sdk_one="$ogs_temp/sdk-one"
ogs_sdk_two="$ogs_temp/sdk-two"
mkdir -m 700 -- "$ogs_sdk_one" "$ogs_sdk_two"
"$ogs_cartridge" sdk-export "$ogs_sdk_one" >"$ogs_temp/sdk-one.json"
"$ogs_cartridge" sdk-export "$ogs_sdk_two" >"$ogs_temp/sdk-two.json"
diff -r --no-dereference "$ogs_sdk_one" "$ogs_sdk_two"
"$ogs_cartridge" sdk-verify "$ogs_sdk_one" >"$ogs_temp/sdk-verify.json"
rg --fixed-strings '"ok":true' "$ogs_temp/sdk-verify.json" >/dev/null

ogs_source="$ogs_temp/source"
cp -R -- examples/first-party-door-legends "$ogs_source"
git -C "$ogs_source" init --quiet
git -C "$ogs_source" config user.name 'OmarchyGS Conformance'
git -C "$ogs_source" config user.email 'conformance@invalid.example'
git -C "$ogs_source" add --all
env GIT_AUTHOR_DATE='2026-01-01T00:00:00Z' \
  GIT_COMMITTER_DATE='2026-01-01T00:00:00Z' \
  git -C "$ogs_source" commit --quiet -m 'First-party Door Legends cartridge'
ogs_revision="$(git -C "$ogs_source" rev-parse --verify HEAD)"
git clone --quiet --no-hardlinks "$ogs_source" "$ogs_temp/clone-one"
git clone --quiet --no-hardlinks "$ogs_source" "$ogs_temp/clone-two"

"$ogs_cartridge" keygen ignibyte ignibyte-primary-v1 \
  "$ogs_temp/publisher.private.json" "$ogs_temp/publisher.public.json" \
  >"$ogs_temp/publisher-keygen.json"

for ogs_clone in clone-one clone-two; do
  env \
    DATABASE_URL='postgres://unusable.invalid/no-access' \
    OMARCHYGS_DEVICE_TOKEN='must-not-be-read' \
    OMARCHYGS_MFA_ENCRYPTION_KEY='must-not-be-read' \
    HTTP_PROXY='http://127.0.0.1:1' \
    HTTPS_PROXY='http://127.0.0.1:1' \
    OMARCHYGS_CARTRIDGE_CLI="$ogs_cartridge" \
    OMARCHYGS_CARTRIDGE_SDK="$ogs_sdk_one" \
    OMARCHYGS_PUBLISHER_KEY="$ogs_temp/publisher.private.json" \
    "$ogs_temp/$ogs_clone/build-release.sh" "$ogs_temp/$ogs_clone/release" \
    >"$ogs_temp/$ogs_clone/build-report.jsonl"
  [[ "$(git -C "$ogs_temp/$ogs_clone" rev-parse --verify HEAD)" == "$ogs_revision" ]]
done

for ogs_file in cartridge.ogsc conformance.json release.signed.json; do
  cmp "$ogs_temp/clone-one/release/$ogs_file" "$ogs_temp/clone-two/release/$ogs_file"
  [[ "$(stat -c '%a' "$ogs_temp/clone-one/release/$ogs_file")" == "444" ]]
done
"$ogs_cartridge" verify-release "$ogs_temp/clone-one/release" \
  "$ogs_temp/publisher.public.json" "$ogs_sdk_one" \
  >"$ogs_temp/release-verification.json"
rg --fixed-strings '"reproducible_inputs":true' "$ogs_temp/release-verification.json" >/dev/null
rg --fixed-strings "\"source_revision\":\"$ogs_revision\"" \
  "$ogs_temp/release-verification.json" >/dev/null
rg --fixed-strings '"database_required":false' "$ogs_temp/release-verification.json" >/dev/null
rg --fixed-strings '"provider_contacted":false' "$ogs_temp/release-verification.json" >/dev/null
rg --fixed-strings '"platform_credentials_read":false' "$ogs_temp/release-verification.json" >/dev/null

"$ogs_cartridge" catalog-keygen omarchygs catalog-primary-v1 \
  "$ogs_temp/catalog.private.json" "$ogs_temp/catalog.public.json" \
  >"$ogs_temp/catalog-keygen.json"
"$ogs_cartridge" catalog-policy "$ogs_temp/clone-one/release" \
  "$ogs_temp/publisher.public.json" "$ogs_sdk_one" \
  "$ogs_temp/catalog.private.json" 1 active 'first-party release approved' \
  "$ogs_temp/catalog-policy.signed.json" >"$ogs_temp/catalog-policy.json"
mkdir -m 700 -- "$ogs_temp/store"
"$ogs_cartridge" secure-import "$ogs_temp/clone-one/release" \
  "$ogs_temp/publisher.public.json" "$ogs_sdk_one" \
  "$ogs_temp/catalog-policy.signed.json" "$ogs_temp/catalog.public.json" \
  "$ogs_temp/store" >"$ogs_temp/secure-import.json"
rg --fixed-strings '"descriptor_relative":true' "$ogs_temp/secure-import.json" >/dev/null
rg --fixed-strings '"authoritative_policy_verified":true' "$ogs_temp/secure-import.json" >/dev/null

ogs_digest="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["activation"]["archive_sha256"])' "$ogs_temp/secure-import.json")"
cmp "$ogs_temp/clone-one/release/cartridge.ogsc" \
  "$ogs_temp/store/blobs/sha256/$ogs_digest.ogsc"

printf '%s\n' '{"scale":1.0,"high_contrast":false,"reduced_motion":false,"muted_audio":true}' \
  >"$ogs_temp/preferences.json"
mkdir -m 700 -- "$ogs_temp/prepared-preview"
"$ogs_preview" prepare "$ogs_temp/clone-one/release/cartridge.ogsc" \
  "$ogs_temp/publisher.public.json" core "$ogs_temp/clone-one/view.json" ready \
  "$ogs_temp/preferences.json" "$ogs_temp/prepared-preview" \
  >"$ogs_temp/preview.json"
rg --fixed-strings '"asset_count":0' "$ogs_temp/preview.json" >/dev/null
[[ "$(python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["nodes"]))' "$ogs_temp/prepared-preview/render-plan.json")" == "4" ]]
rg --fixed-strings '"game_key":"door-legends"' "$ogs_temp/prepared-preview/render-plan.json" >/dev/null

if rg --fixed-strings -- "$ogs_root" \
  "$ogs_temp/clone-one/release" "$ogs_temp/clone-two/release" \
  "$ogs_temp/release-verification.json" >/dev/null; then
  echo 'clean-room release leaked a platform source-tree path' >&2
  exit 1
fi

echo "production Game Cartridge SDK release passed"
echo "OGS_CARTRIDGE_SDK_RELEASE source_revision=$ogs_revision archive_sha256=$ogs_digest sdk_lock_sha256=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["identity"]["lock_sha256"])' "$ogs_temp/sdk-verify.json")"
