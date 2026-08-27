#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
ogs_temp="$(mktemp -d)"
trap 'rm -rf -- "$ogs_temp"' EXIT INT TERM

for ogs_command in bwrap cargo python3; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "Missing marketplace publication test command: $ogs_command" >&2
    exit 1
  }
done

cd "$ogs_root"
cargo test --quiet --locked --package omarchygs-marketplace-publisher
cargo build --quiet --locked --package omarchygs-marketplace-publisher \
  --bin omarchygs-marketplace-publisher

ogs_publisher="$ogs_root/target/debug/omarchygs-marketplace-publisher"
if "$ogs_publisher" >"$ogs_temp/stdout" 2>"$ogs_temp/stderr"; then
  echo "Marketplace publisher accepted a missing command." >&2
  exit 1
fi
python3 - "$ogs_temp/stdout" "$ogs_temp/stderr" <<'PY'
import json
import pathlib
import sys

stdout = pathlib.Path(sys.argv[1]).read_bytes()
stderr = pathlib.Path(sys.argv[2]).read_bytes()
assert stdout == b""
assert json.loads(stderr) == {
    "format": "omarchygs.marketplace-publication-error/v1",
    "ok": False,
    "code": "marketplace_publication_invalid_input",
}
PY

echo "static marketplace publication and offline-root drill passed"
