#!/usr/bin/env bash
set -euo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"

cleanup() {
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo cmp find python3 rg stat; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

cd "$ogs_root"
cargo test -p omarchygs-game-cartridge --all-targets
cargo build -p omarchygs-game-cartridge --bin omarchygs-cartridge

ogs_bin="$ogs_root/target/debug/omarchygs-cartridge"
ogs_source_one="$ogs_temp/source-one"
ogs_source_two="$ogs_temp/source-two"
cp -R -- "$ogs_root/crates/game-cartridge/tests/fixtures/valid" "$ogs_source_one"
cp -R -- "$ogs_root/crates/game-cartridge/tests/fixtures/valid" "$ogs_source_two"
find "$ogs_source_two" -type f -exec touch -t 202001010000 {} +
find "$ogs_source_two" -type f -exec chmod 600 {} +

"$ogs_bin" keygen ignibyte ignibyte-primary-v1 \
  "$ogs_temp/publisher.private.json" "$ogs_temp/publisher.public.json" \
  >"$ogs_temp/keygen.json"
"$ogs_bin" pack "$ogs_source_one" "$ogs_temp/publisher.private.json" \
  "$ogs_temp/first.ogsc" >"$ogs_temp/pack-first.json"
"$ogs_bin" pack "$ogs_source_two" "$ogs_temp/publisher.private.json" \
  "$ogs_temp/second.ogsc" >"$ogs_temp/pack-second.json"
cmp "$ogs_temp/first.ogsc" "$ogs_temp/second.ogsc"

env \
  DATABASE_URL='postgres://unusable.invalid/no-access' \
  OMARCHYGS_DEVICE_TOKEN='must-not-be-read' \
  HTTP_PROXY='http://127.0.0.1:1' \
  HTTPS_PROXY='http://127.0.0.1:1' \
  "$ogs_bin" conform "$ogs_temp/first.ogsc" "$ogs_temp/publisher.public.json" \
  >"$ogs_temp/conformance.json"
rg --fixed-strings '"conformant":true' "$ogs_temp/conformance.json" >/dev/null
rg --fixed-strings '"installed":false' "$ogs_temp/conformance.json" >/dev/null
rg --fixed-strings '"provider_contacted":false' "$ogs_temp/conformance.json" >/dev/null
rg --fixed-strings '"database_required":false' "$ogs_temp/conformance.json" >/dev/null
rg --fixed-strings '"platform_credentials_read":false' "$ogs_temp/conformance.json" >/dev/null

"$ogs_bin" install "$ogs_temp/first.ogsc" "$ogs_temp/publisher.public.json" \
  "$ogs_temp/store" >"$ogs_temp/install.json"
ogs_digest="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["activation"]["archive_sha256"])' "$ogs_temp/install.json")"
ogs_blob="$ogs_temp/store/blobs/sha256/$ogs_digest.ogsc"
[[ -f "$ogs_blob" ]]
[[ "$(stat -c '%a' "$ogs_blob")" == "444" ]]
[[ ! -x "$ogs_blob" ]]

"$ogs_bin" revoke "$ogs_temp/store" "$ogs_digest" "conformance withdrawal" \
  >"$ogs_temp/revoke.json"
rg --fixed-strings '"revoked":true' "$ogs_temp/revoke.json" >/dev/null

echo "production game cartridge contract passed"
echo "OGS_CARTRIDGE_V1 archive_sha256=$ogs_digest archive_bytes=$(stat -c '%s' "$ogs_blob")"
