#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"
ogs_provider_pid=""
ogs_database="ogs_relay_forge_${$}"
ogs_admin_url="${DATABASE_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/omarchy_gaming_system}"

cleanup() {
  if [[ -n "$ogs_provider_pid" ]] && kill -0 "$ogs_provider_pid" 2>/dev/null; then
    kill -INT "$ogs_provider_pid" 2>/dev/null || true
    wait "$ogs_provider_pid" 2>/dev/null || true
  fi
  psql "$ogs_admin_url" -v ON_ERROR_STOP=1 -qAt \
    -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$ogs_database'" \
    >/dev/null 2>&1 || true
  psql "$ogs_admin_url" -v ON_ERROR_STOP=1 -qAt \
    -c "DROP DATABASE IF EXISTS \"$ogs_database\"" >/dev/null 2>&1 || true
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in base64 cargo docker jq mise openssl psql rg shuf; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

port() {
  local ogs_port
  for _ in {1..100}; do
    ogs_port="$(shuf -i 20000-50000 -n 1)"
    if ! (exec 9<>"/dev/tcp/127.0.0.1/$ogs_port") 2>/dev/null; then
      printf '%s\n' "$ogs_port"
      return 0
    fi
  done
  return 1
}

wait_for_port() {
  local ogs_port="$1"
  for _ in {1..100}; do
    if (exec 9<>"/dev/tcp/127.0.0.1/$ogs_port") 2>/dev/null; then
      return 0
    fi
    sleep 0.05
  done
  return 1
}

base64url_text() {
  printf '%s' "$1" | base64 -w0 | tr '+/' '-_' | tr -d '='
}

cd "$ogs_root"
docker compose up -d --wait db
export DATABASE_URL="$ogs_admin_url"

mise exec -- cargo test -p omarchygs-provider-starter --test persistence \
  -- --ignored --test-threads=1
cargo build -p omarchygs-provider-conformance --bins

ogs_sdk_patch="patch.crates-io.omarchygs-provider-sdk.path=\"$ogs_root/crates/provider-sdk\""
ogs_starter_patch="patch.crates-io.omarchygs-provider-starter.path=\"$ogs_root/crates/provider-starter\""
cargo build --locked --manifest-path examples/provider-relay-forge/Cargo.toml \
  --features conformance --target-dir "$ogs_root/target/relay-forge-conformance" \
  --config "$ogs_sdk_patch" --config "$ogs_starter_patch"

psql "$ogs_admin_url" -v ON_ERROR_STOP=1 -qAt \
  -c "CREATE DATABASE \"$ogs_database\""
ogs_provider_url="${ogs_admin_url%/*}/$ogs_database"
[[ "$ogs_provider_url" != "$ogs_admin_url" ]]

OMARCHYGS_RELAY_FORGE_BIN="$ogs_root/target/relay-forge-conformance/debug/relay-forge-provider" \
OMARCHYGS_RELAY_FORGE_DATABASE_URL="$ogs_provider_url" \
  mise exec -- cargo test -p omarchy-game-provider \
    --features provider-conformance --test starter_integration \
    -- --ignored --test-threads=1

ogs_provider_port="$(port)"
ogs_callback_port="$(port)"
[[ "$ogs_provider_port" != "$ogs_callback_port" ]]
ogs_release_id='45454545-4545-4545-8545-454545454545'

openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -keyout "$ogs_temp/provider-key.pem" -out "$ogs_temp/provider-cert.pem" \
  -subj '/CN=relay.example.test' -addext 'subjectAltName=DNS:relay.example.test' \
  -addext 'basicConstraints=critical,CA:FALSE' \
  -addext 'keyUsage=critical,digitalSignature,keyEncipherment' \
  -addext 'extendedKeyUsage=serverAuth' \
  >/dev/null 2>&1
openssl x509 -in "$ogs_temp/provider-cert.pem" -outform DER \
  -out "$ogs_temp/provider-cert.der"
openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -keyout "$ogs_temp/callback-key.pem" -out "$ogs_temp/callback-cert.pem" \
  -subj '/CN=callback.example.test' -addext 'subjectAltName=DNS:callback.example.test' \
  -addext 'basicConstraints=critical,CA:FALSE' \
  -addext 'keyUsage=critical,digitalSignature,keyEncipherment' \
  -addext 'extendedKeyUsage=serverAuth' \
  >/dev/null 2>&1
openssl x509 -in "$ogs_temp/callback-cert.pem" -outform DER \
  -out "$ogs_temp/callback-cert.der"
chmod 600 "$ogs_temp/provider-key.pem" "$ogs_temp/callback-key.pem"

ogs_grant_seed="$(base64url_text '33333333333333333333333333333333')"
ogs_message_seed="$(base64url_text '44444444444444444444444444444444')"
ogs_provider_seed="$(base64url_text '55555555555555555555555555555555')"
ogs_pairwise_secret="$(base64url_text '<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<')"
ogs_public_keys="$({
  jq -cn \
    --arg grant_seed_base64 "$ogs_grant_seed" \
    --arg message_seed_base64 "$ogs_message_seed" \
    --arg provider_seed_base64 "$ogs_provider_seed" \
    '{grant_seed_base64:$grant_seed_base64,message_seed_base64:$message_seed_base64,provider_seed_base64:$provider_seed_base64}'
} | target/debug/omarchygs-provider-conformance-keys)"
ogs_grant_public="$(jq -r '.grant_public_key_base64' <<<"$ogs_public_keys")"
ogs_message_public="$(jq -r '.message_public_key_base64' <<<"$ogs_public_keys")"
ogs_provider_public="$(jq -r '.provider_public_key_base64' <<<"$ogs_public_keys")"
ogs_provider_root="$(base64 -w0 "$ogs_temp/provider-cert.der" | tr '+/' '-_' | tr -d '=')"
ogs_callback_root="$(base64 -w0 "$ogs_temp/callback-cert.der" | tr '+/' '-_' | tr -d '=')"

jq -cn \
  --arg authority "relay.example.test:$ogs_provider_port" \
  --arg bind_address "127.0.0.1:$ogs_provider_port" \
  --arg callback_socket_override "127.0.0.1:$ogs_callback_port" \
  --arg callback_tls_root_der_base64 "$ogs_callback_root" \
  --arg callback_url "https://callback.example.test:$ogs_callback_port/v1/provider-events/$ogs_release_id" \
  --arg cartridge_digest "$(printf 'a%.0s' {1..64})" \
  --arg database_url "$ogs_provider_url" \
  --arg platform_grant_public_key_base64 "$ogs_grant_public" \
  --arg platform_message_public_key_base64 "$ogs_message_public" \
  --arg provider_message_signing_seed_base64 "$ogs_provider_seed" \
  --arg release_id "$ogs_release_id" \
  --arg tls_certificate "$ogs_temp/provider-cert.pem" \
  --arg tls_private_key "$ogs_temp/provider-key.pem" \
  '{authority:$authority,bind_address:$bind_address,callback_socket_override:$callback_socket_override,callback_tls_root_der_base64:$callback_tls_root_der_base64,callback_url:$callback_url,cartridge_digest:$cartridge_digest,command_response_delay_ms:200,database_url:$database_url,platform_grant_key_id:"platform-grant-1",platform_grant_public_key_base64:$platform_grant_public_key_base64,platform_message_key_id:"platform-message-1",platform_message_public_key_base64:$platform_message_public_key_base64,provider_message_key_id:"provider-message-1",provider_message_signing_seed_base64:$provider_message_signing_seed_base64,release_id:$release_id,tls_certificate:$tls_certificate,tls_private_key:$tls_private_key}' | tr -d '\n' \
  >"$ogs_temp/provider-config.json"
chmod 600 "$ogs_temp/provider-config.json"

jq -cn \
  --arg authority "relay.example.test:$ogs_provider_port" \
  --arg callback_authority "callback.example.test:$ogs_callback_port" \
  --arg callback_bind_address "127.0.0.1:$ogs_callback_port" \
  --arg callback_certificate_pem "$ogs_temp/callback-cert.pem" \
  --arg callback_path "/v1/provider-events/$ogs_release_id" \
  --arg callback_private_key_pem "$ogs_temp/callback-key.pem" \
  --arg cartridge_digest "$(printf 'a%.0s' {1..64})" \
  --arg endpoint "https://relay.example.test:$ogs_provider_port/omarchygs/provider/v1/" \
  --arg pairwise_secret_base64 "$ogs_pairwise_secret" \
  --arg platform_grant_seed_base64 "$ogs_grant_seed" \
  --arg platform_message_seed_base64 "$ogs_message_seed" \
  --arg provider_message_public_key_base64 "$ogs_provider_public" \
  --arg provider_root_der_base64 "$ogs_provider_root" \
  --arg provider_socket_override "127.0.0.1:$ogs_provider_port" \
  --arg release_id "$ogs_release_id" \
  --arg subject "$(printf 'Q%.0s' {1..43})" \
  '{authority:$authority,callback_authority:$callback_authority,callback_bind_address:$callback_bind_address,callback_certificate_pem:$callback_certificate_pem,callback_path:$callback_path,callback_private_key_pem:$callback_private_key_pem,cartridge_digest:$cartridge_digest,endpoint:$endpoint,game_key:"relay-forge",normal_timeout_ms:3000,pairwise_secret_base64:$pairwise_secret_base64,platform_grant_key_id:"platform-grant-1",platform_grant_seed_base64:$platform_grant_seed_base64,platform_message_key_id:"platform-message-1",platform_message_seed_base64:$platform_message_seed_base64,provider_id:"relay-labs",provider_message_key_id:"provider-message-1",provider_message_public_key_base64:$provider_message_public_key_base64,provider_root_der_base64:$provider_root_der_base64,provider_socket_override:$provider_socket_override,release_id:$release_id,rules_version:1,subject:$subject,unknown_outcome_timeout_ms:30}' | tr -d '\n' \
  >"$ogs_temp/conformance-config.json"
chmod 600 "$ogs_temp/conformance-config.json"

start_provider() {
  "$ogs_root/target/relay-forge-conformance/debug/relay-forge-provider" "$ogs_temp/provider-config.json" \
    >"$ogs_temp/provider.stdout" 2>"$ogs_temp/provider.stderr" &
  ogs_provider_pid=$!
  if ! wait_for_port "$ogs_provider_port"; then
    cat "$ogs_temp/provider.stderr" >&2
    return 1
  fi
}

stop_provider() {
  kill -INT "$ogs_provider_pid"
  wait "$ogs_provider_pid"
  ogs_provider_pid=""
}

start_provider
target/debug/omarchygs-provider-conformance "$ogs_temp/conformance-config.json" \
  >"$ogs_temp/receipt-one.json"
stop_provider

start_provider
target/debug/omarchygs-provider-conformance "$ogs_temp/conformance-config.json" \
  >"$ogs_temp/receipt-two.json"
stop_provider

for ogs_receipt in "$ogs_temp/receipt-one.json" "$ogs_temp/receipt-two.json"; do
  jq -e '.format == "omarchygs.provider-conformance-receipt/v1" and (.cases | length == 15) and all(.cases[]; .passed)' \
    "$ogs_receipt" >/dev/null
done

ogs_sessions="$(psql "$ogs_provider_url" -v ON_ERROR_STOP=1 -qAt \
  -c 'SELECT count(*) FROM provider_starter_sessions')"
ogs_receipts="$(psql "$ogs_provider_url" -v ON_ERROR_STOP=1 -qAt \
  -c 'SELECT count(*) FROM provider_starter_operation_receipts')"
[[ "$ogs_sessions" == '3' ]]
((ogs_receipts >= 20))
[[ "$(psql "$ogs_admin_url" -v ON_ERROR_STOP=1 -qAt \
  -c "SELECT to_regclass('public.provider_starter_sessions') IS NULL")" == 't' ]]

! rg -a '(persona_id|account_id|device_credential|DATABASE_URL)' \
  "$ogs_temp/receipt-one.json" "$ogs_temp/receipt-two.json"

echo 'provider starter TLS conformance, callback recovery, and durable restart passed'
