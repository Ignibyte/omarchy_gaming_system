#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
ogs_temp="$(mktemp -d)"
trap 'rm -rf -- "$ogs_temp"' EXIT INT TERM

for ogs_command in cargo cmp date python3 sha256sum stat; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "Missing marketplace trust test command: $ogs_command" >&2
    exit 1
  }
done

cd "$ogs_root"
cargo build --quiet --locked \
  --package omarchygs-marketplace-trust \
  --bin omarchygs-marketplace-channel
cargo build --quiet --locked \
  --package omarchygs-game-cartridge \
  --bin omarchygs-cartridge

ogs_channel="$ogs_root/target/debug/omarchygs-marketplace-channel"
ogs_cartridge="$ogs_root/target/debug/omarchygs-cartridge"
"$ogs_channel" generate-root root-1 official \
  "$ogs_temp/root.private.json" "$ogs_temp/root.public.json" >/dev/null
(
  cd -- "$ogs_temp"
  if "$ogs_channel" generate-root root-relative official \
    root-relative.private.json root-relative.public.json >/dev/null 2>&1; then
    echo "Marketplace root tooling accepted a relative private-key path." >&2
    exit 1
  fi
)
"$ogs_cartridge" catalog-keygen official-marketplace catalog-1 \
  "$ogs_temp/catalog.private.json" "$ogs_temp/catalog.public.json" >/dev/null

[[ "$(stat -c '%a' "$ogs_temp/root.private.json")" == 600 ]]
[[ "$(stat -c '%a' "$ogs_temp/root.public.json")" == 644 ]]

ogs_now="$(date +%s)"
python3 - "$ogs_temp/catalog.public.json" "$ogs_temp/payload.json" "$ogs_now" <<'PY'
import hashlib
import json
import pathlib
import sys

catalog = json.loads(pathlib.Path(sys.argv[1]).read_bytes())
catalog_bytes = json.dumps(catalog, separators=(",", ":")).encode()
now = int(sys.argv[3])
payload = {
    "format": "omarchygs.marketplace-trust-channel/v2",
    "channel_id": "official",
    "channel_name": "Official OmarchyGS",
    "channel_origin": "https://packages.example.test/v1/",
    "marketplace_origin": "https://market.example.test/v1/",
    "marketplace_authority_id": "official-marketplace",
    "bundle_version": 1,
    "current_snapshot_version": 1,
    "not_before_unix": now - 10,
    "expires_at_unix": now + 3600,
    "keys": [{
        "key": catalog,
        "key_sha256": hashlib.sha256(catalog_bytes).hexdigest(),
        "status": "active",
        "first_snapshot_version": 1,
    }],
    "packages": [],
}
pathlib.Path(sys.argv[2]).write_bytes(
    json.dumps(payload, separators=(",", ":")).encode()
)
PY

"$ogs_channel" sign "$ogs_temp/payload.json" "$ogs_temp/root.private.json" \
  "$ogs_temp/channel-one.signed.json" >/dev/null
"$ogs_channel" sign "$ogs_temp/payload.json" "$ogs_temp/root.private.json" \
  "$ogs_temp/channel-two.signed.json" >/dev/null
cmp -- "$ogs_temp/channel-one.signed.json" "$ogs_temp/channel-two.signed.json"
"$ogs_channel" verify "$ogs_temp/channel-one.signed.json" \
  "$ogs_temp/root.public.json" official \
  https://packages.example.test/v1/ "$ogs_now" >/dev/null

python3 - "$ogs_temp/channel-one.signed.json" "$ogs_temp/tampered.signed.json" <<'PY'
import json
import pathlib
import sys

document = json.loads(pathlib.Path(sys.argv[1]).read_bytes())
document["signature"] = ("A" if document["signature"][0] != "A" else "B") + document["signature"][1:]
pathlib.Path(sys.argv[2]).write_bytes(
    json.dumps(document, separators=(",", ":")).encode()
)
PY
if "$ogs_channel" verify "$ogs_temp/tampered.signed.json" \
  "$ogs_temp/root.public.json" official \
  https://packages.example.test/v1/ "$ogs_now" >/dev/null 2>&1; then
  echo "Tampered marketplace trust channel was accepted." >&2
  exit 1
fi

"$ogs_channel" bootstrap "$ogs_temp/root.public.json" \
  https://packages.example.test/v1/ trust.signed.json \
  1 1 arch-linux x86_64 0.1.0-1 "$ogs_temp/bootstrap.json" >/dev/null
"$ogs_channel" verify-bootstrap "$ogs_temp/bootstrap.json" >/dev/null
printf '\n' >>"$ogs_temp/bootstrap.json"
if "$ogs_channel" verify-bootstrap "$ogs_temp/bootstrap.json" >/dev/null 2>&1; then
  echo "Non-canonical marketplace bootstrap was accepted." >&2
  exit 1
fi

echo "root-signed marketplace trust channel passed"
