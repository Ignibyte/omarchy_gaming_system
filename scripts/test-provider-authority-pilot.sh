#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"
ogs_provider_db="door_legends_pilot_${BASHPID}"
ogs_restore_db="door_legends_restore_${BASHPID}"
ogs_admin_url="${OGS_TEST_POSTGRES_ADMIN_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/postgres}"

cleanup() {
  psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
    -c "DROP DATABASE IF EXISTS $ogs_provider_db WITH (FORCE)" \
    -c "DROP DATABASE IF EXISTS $ogs_restore_db WITH (FORCE)" >/dev/null 2>&1 || true
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo cp docker find git pg_dump pg_restore psql rg tar; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

[[ "$ogs_provider_db" =~ ^[a-z0-9_]+$ ]]
[[ "$ogs_restore_db" =~ ^[a-z0-9_]+$ ]]

cd "$ogs_root"
docker compose up -d --wait db

ogs_package_target="$ogs_temp/package-target"
cargo package -p omarchygs-provider-sdk --allow-dirty --no-verify \
  --target-dir "$ogs_package_target" >/dev/null
ogs_crate="$(find "$ogs_package_target/package" -maxdepth 1 -type f -name 'omarchygs-provider-sdk-*.crate' -print -quit)"
[[ -n "$ogs_crate" ]]
mkdir -m 700 -- "$ogs_temp/protocol"
tar -xzf "$ogs_crate" -C "$ogs_temp/protocol"
ogs_protocol="$(find "$ogs_temp/protocol" -mindepth 1 -maxdepth 1 -type d -name 'omarchygs-provider-sdk-*' -print -quit)"
[[ -n "$ogs_protocol" ]]

ogs_source="$ogs_temp/source"
cp -R -- examples/first-party-door-legends "$ogs_source"
git -C "$ogs_source" init --quiet
git -C "$ogs_source" config user.name 'OmarchyGS Authority Conformance'
git -C "$ogs_source" config user.email 'authority-conformance@invalid.example'
git -C "$ogs_source" add --all
env GIT_AUTHOR_DATE='2026-01-01T00:00:00Z' \
  GIT_COMMITTER_DATE='2026-01-01T00:00:00Z' \
  git -C "$ogs_source" commit --quiet -m 'Door Legends cartridge and provider'
git clone --quiet --no-hardlinks "$ogs_source" "$ogs_temp/clone"

ogs_patch="patch.crates-io.omarchygs-provider-sdk.path=\"$ogs_protocol\""
cargo build --manifest-path "$ogs_temp/clone/provider/Cargo.toml" \
  --config "$ogs_patch" --locked --features conformance
cargo tree --manifest-path "$ogs_temp/clone/provider/Cargo.toml" \
  --config "$ogs_patch" -p omarchygs-provider-sdk -e features \
  >"$ogs_temp/protocol-tree.txt"
if rg '(^| )((sqlx|reqwest|tokio|tracing) v|omarchy-game-provider|provider-sdk feature "platform")' \
  "$ogs_temp/protocol-tree.txt" >/dev/null; then
  echo 'Door Legends pulled a platform-only provider dependency feature' >&2
  exit 1
fi

ogs_provider_binary="$ogs_temp/clone/provider/target/debug/door-legends-provider"
[[ -x "$ogs_provider_binary" ]]
if rg -a --fixed-strings -- "$ogs_root" "$ogs_provider_binary" >/dev/null; then
  echo 'clean-clone Door Legends binary leaked a platform source-tree path' >&2
  exit 1
fi

psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE $ogs_provider_db OWNER omarchy_gaming_system" \
  -c "CREATE DATABASE $ogs_restore_db OWNER omarchy_gaming_system" >/dev/null
ogs_provider_url="postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/$ogs_provider_db"
ogs_restore_url="postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/$ogs_restore_db"

env \
  DATABASE_URL="${DATABASE_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/omarchy_gaming_system}" \
  DOOR_LEGENDS_PROVIDER_BINARY="$ogs_provider_binary" \
  DOOR_LEGENDS_TEST_DATABASE_URL="$ogs_provider_url" \
  HTTP_PROXY='http://127.0.0.1:9' \
  HTTPS_PROXY='http://127.0.0.1:9' \
  ALL_PROXY='http://127.0.0.1:9' \
  http_proxy='http://127.0.0.1:9' \
  https_proxy='http://127.0.0.1:9' \
  all_proxy='http://127.0.0.1:9' \
  NO_PROXY='' \
  no_proxy='' \
  cargo test -p omarchy-gaming-system-server \
    provider_game_api_tests::clean_clone_door_legends_owns_state_restarts_and_projects_results \
    -- --ignored --exact --test-threads=1

pg_dump "$ogs_provider_url" --format=custom --file="$ogs_temp/door-legends.backup"
pg_restore --exit-on-error --no-owner --dbname="$ogs_restore_url" \
  "$ogs_temp/door-legends.backup"
ogs_restored_sessions="$(psql "$ogs_restore_url" -Atc 'SELECT count(*) FROM door_legends_sessions')"
ogs_restored_receipts="$(psql "$ogs_restore_url" -Atc 'SELECT count(*) FROM door_legends_operation_receipts')"
ogs_restored_events="$(psql "$ogs_restore_url" -Atc "SELECT count(*) FROM door_legends_event_outbox WHERE status = 'delivered'")"
if [[ "$ogs_restored_sessions" != "3" \
  || ( "$ogs_restored_receipts" != "10" && "$ogs_restored_receipts" != "11" ) \
  || "$ogs_restored_events" != "2" ]]; then
  echo "unexpected restored Door Legends evidence: sessions=$ogs_restored_sessions receipts=$ogs_restored_receipts delivered_events=$ogs_restored_events" >&2
  exit 1
fi

echo 'first-party remote-provider authority pilot passed'
