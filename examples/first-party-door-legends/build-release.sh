#!/usr/bin/env bash
set -Eeuo pipefail

: "${OMARCHYGS_CARTRIDGE_CLI:?set to the pinned omarchygs-cartridge executable}"
: "${OMARCHYGS_CARTRIDGE_SDK:?set to the exported SDK v1 directory}"
: "${OMARCHYGS_PUBLISHER_KEY:?set to the publisher private key file}"

ogs_output="${1:?usage: build-release.sh OUTPUT_DIRECTORY}"
ogs_repo="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$ogs_repo"
ogs_revision="$(git rev-parse --verify HEAD)"

mkdir -- "$ogs_output"
"$OMARCHYGS_CARTRIDGE_CLI" sdk-verify "$OMARCHYGS_CARTRIDGE_SDK"
"$OMARCHYGS_CARTRIDGE_CLI" release cartridge "$OMARCHYGS_PUBLISHER_KEY" \
  "$OMARCHYGS_CARTRIDGE_SDK" "$ogs_revision" "$ogs_output"
