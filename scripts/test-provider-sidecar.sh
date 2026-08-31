#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"
ogs_provider_db="ogs_provider_sidecar_${BASHPID}"
ogs_restore_db="ogs_provider_sidecar_restore_${BASHPID}"
ogs_admin_url="${OGS_TEST_POSTGRES_ADMIN_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/postgres}"

cleanup() {
  psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
    -c "DROP DATABASE IF EXISTS $ogs_provider_db WITH (FORCE)" \
    -c "DROP DATABASE IF EXISTS $ogs_restore_db WITH (FORCE)" >/dev/null 2>&1 || true
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo docker jq openssl pg_dump pg_restore psql rg sed; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

[[ "$ogs_provider_db" =~ ^[a-z0-9_]+$ ]]
[[ "$ogs_restore_db" =~ ^[a-z0-9_]+$ ]]

cd "$ogs_root"
docker compose up -d --wait db

ogs_sdk_patch="patch.crates-io.omarchygs-provider-sdk.path=\"$ogs_root/crates/provider-sdk\""
ogs_starter_patch="patch.crates-io.omarchygs-provider-starter.path=\"$ogs_root/crates/provider-starter\""
cargo build --locked --manifest-path examples/provider-relay-forge/Cargo.toml \
  --target-dir "$ogs_root/target/relay-forge-sidecar" \
  --config "$ogs_sdk_patch" --config "$ogs_starter_patch"

psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE $ogs_provider_db OWNER omarchy_gaming_system" \
  -c "CREATE DATABASE $ogs_restore_db OWNER omarchy_gaming_system" >/dev/null
ogs_provider_url="postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/$ogs_provider_db"
ogs_restore_url="postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/$ogs_restore_db"

env \
  DATABASE_URL="${DATABASE_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/omarchy_gaming_system}" \
  OMARCHYGS_RELAY_FORGE_BIN="$ogs_root/target/relay-forge-sidecar/debug/relay-forge-provider" \
  OMARCHYGS_RELAY_FORGE_DATABASE_URL="$ogs_provider_url" \
  cargo test -p omarchy-game-provider --features provider-conformance \
    --test starter_integration \
    -- --ignored --exact --test-threads=1

ogs_sessions="$(psql "$ogs_provider_url" -v ON_ERROR_STOP=1 -qAt \
  -c 'SELECT count(*) FROM provider_starter_sessions')"
ogs_receipts="$(psql "$ogs_provider_url" -v ON_ERROR_STOP=1 -qAt \
  -c 'SELECT count(*) FROM provider_starter_operation_receipts')"
ogs_delivered="$(psql "$ogs_provider_url" -v ON_ERROR_STOP=1 -qAt \
  -c "SELECT count(*) FROM provider_starter_event_outbox WHERE status = 'delivered'")"
[[ "$ogs_sessions" == '1' ]]
[[ "$ogs_receipts" == '6' ]]
[[ "$ogs_delivered" == '1' ]]

pg_dump "$ogs_provider_url" --format=custom --file="$ogs_temp/provider.backup"
pg_restore --exit-on-error --no-owner --dbname="$ogs_restore_url" \
  "$ogs_temp/provider.backup"
[[ "$(psql "$ogs_restore_url" -v ON_ERROR_STOP=1 -qAt \
  -c 'SELECT count(*) FROM provider_starter_sessions')" == "$ogs_sessions" ]]
[[ "$(psql "$ogs_restore_url" -v ON_ERROR_STOP=1 -qAt \
  -c 'SELECT count(*) FROM provider_starter_operation_receipts')" == "$ogs_receipts" ]]
[[ "$(psql "$ogs_restore_url" -v ON_ERROR_STOP=1 -qAt \
  -c "SELECT count(*) FROM provider_starter_event_outbox WHERE status = 'delivered'")" == "$ogs_delivered" ]]

ogs_unit="$ogs_root/deploy/provider-sidecar/omarchygs-provider-sidecar@.service"
for ogs_directive in \
  'User=omarchygs-provider-%i' \
  'NoNewPrivileges=yes' \
  'ProtectSystem=strict' \
  'IPAddressDeny=any' \
  'IPAddressAllow=localhost' \
  'RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6' \
  'ReadOnlyPaths=/etc/omarchygs/providers/%i' \
  'ReadWritePaths=/var/lib/omarchygs/providers/%i' \
  'MemoryMax=512M' \
  'TasksMax=64'; do
  rg -q --fixed-strings "$ogs_directive" "$ogs_unit"
done
! rg -q 'PrivateNetwork=yes|EnvironmentFile=' "$ogs_unit"
jq -e '
  .callback_sidecar_socket == "127.0.0.1:@PLATFORM_CALLBACK_TLS_PORT@" and
  .callback_socket_override == null and
  .command_response_delay_ms == 0 and
  .database_url == "@PROVIDER_ONLY_DATABASE_URL@" and
  .release_id == "@EXACT_REGISTERED_RELEASE_UUID@"
' deploy/provider-sidecar/provider-config.example.json >/dev/null
rg -q '^OGS_PROVIDER_SIDECAR_RELEASE_ID=' deploy/provider-sidecar/platform.env.example
rg -q '^OGS_PROVIDER_SIDECAR_SOCKET=127[.]0[.]0[.]1:' deploy/provider-sidecar/platform.env.example
rg -q '^\s*admin off$' deploy/provider-sidecar/platform-callback.Caddyfile.example
rg -q '^\s*bind 127[.]0[.]0[.]1$' deploy/provider-sidecar/platform-callback.Caddyfile.example
rg -Uq 'reqwest::Client::builder\(\)\n\s*\.https_only\(true\)\n\s*\.no_proxy\(\)' \
  examples/first-party-door-legends/provider/src/main.rs

ogs_runbook="$ogs_root/docs/operators/provider-deployment.md"
for ogs_topic in \
  'TLS identity' \
  'DNS/endpoint immutability' \
  'provider-only PostgreSQL role and database' \
  'least-privilege' \
  'Rotation' \
  'quotas' \
  'Monitor' \
  'Lost database/restore' \
  'Suspend' \
  'incident response' \
  'Upgrade' \
  'end-of-life'; do
  rg -qi --fixed-strings "$ogs_topic" "$ogs_runbook"
done

jq -cn \
  --arg profile 'exact-release-tls-loopback-v1' \
  --arg release_id '45454545-4545-4545-8545-454545454545' \
  --argjson sessions "$ogs_sessions" \
  --argjson receipts "$ogs_receipts" \
  --argjson callbacks "$ogs_delivered" \
  '{format:"omarchygs.provider-sidecar-drill-receipt/v1",profile:$profile,release_id:$release_id,checks:{exact_socket:true,wrong_tls_peer_rejected:true,crash_denied_launch:true,restart_reconciled:true,separate_database:true,backup_restored:true,templates_validated:true},provider:{sessions:$sessions,operation_receipts:$receipts,delivered_callbacks:$callbacks}}' \
  >"$ogs_temp/receipt.json"
openssl genpkey -algorithm ED25519 -out "$ogs_temp/receipt-key.pem" >/dev/null 2>&1
openssl pkey -in "$ogs_temp/receipt-key.pem" -pubout -out "$ogs_temp/receipt-public.pem" >/dev/null 2>&1
openssl pkeyutl -sign -rawin -inkey "$ogs_temp/receipt-key.pem" \
  -in "$ogs_temp/receipt.json" -out "$ogs_temp/receipt.sig"
openssl pkeyutl -verify -rawin -pubin -inkey "$ogs_temp/receipt-public.pem" \
  -in "$ogs_temp/receipt.json" -sigfile "$ogs_temp/receipt.sig" >/dev/null
! rg -a '(DATABASE_URL|postgres://|signing_seed|pairwise|subject|grant)' \
  "$ogs_temp/receipt.json" "$ogs_temp/receipt.sig"

echo 'provider sidecar exact transport, hostile peer, lifecycle, restore, templates, and signed receipt passed'
