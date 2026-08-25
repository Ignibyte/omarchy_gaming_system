#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_log_dir="$ogs_root/.dev"
ogs_mfa_key_file="$ogs_log_dir/mfa-encryption-key"
ogs_server_pid=""
ogs_qml_arguments=()
ogs_smoke_test=false

case "${1:-}" in
  "") ;;
  --smoke-test)
    export QT_QPA_PLATFORM="${QT_QPA_PLATFORM:-offscreen}"
    ogs_qml_arguments=(-- --smoke-test)
    ogs_smoke_test=true
    ;;
  *)
    echo "Usage: $0 [--smoke-test]" >&2
    exit 2
    ;;
esac

cleanup() {
  if [[ -n "$ogs_server_pid" ]] && kill -0 "$ogs_server_pid" 2>/dev/null; then
    kill "$ogs_server_pid"
    wait "$ogs_server_pid" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

for command_name in docker mise qml6 curl jq openssl python3; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

cd "$ogs_root"
mkdir -p "$ogs_log_dir"

if [[ -z "${OGS_MFA_ENCRYPTION_KEY:-}" ]]; then
  if [[ ! -s "$ogs_mfa_key_file" ]]; then
    (
      umask 077
      openssl rand -base64 32 \
        | tr '+/' '-_' \
        | tr -d '=\n' >"$ogs_mfa_key_file"
    )
  fi
  chmod 600 "$ogs_mfa_key_file"
  OGS_MFA_ENCRYPTION_KEY=$(tr -d '\n' <"$ogs_mfa_key_file")
  export OGS_MFA_ENCRYPTION_KEY
fi

mise install
docker compose up -d --wait db

export DATABASE_URL="${DATABASE_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/omarchy_gaming_system}"
export OGS_BIND_ADDRESS="${OGS_BIND_ADDRESS:-127.0.0.1:8080}"
export RUST_LOG="${RUST_LOG:-omarchy_gaming_system_server=debug,tower_http=debug}"

mise exec -- cargo run -p omarchy-gaming-system-server >"$ogs_log_dir/server.log" 2>&1 &
ogs_server_pid=$!

for _ in {1..90}; do
  if curl --fail --silent "http://$OGS_BIND_ADDRESS/health" >/dev/null; then
    break
  fi

  if ! kill -0 "$ogs_server_pid" 2>/dev/null; then
    echo "The server stopped during startup:" >&2
    tail -80 "$ogs_log_dir/server.log" >&2
    exit 1
  fi

  sleep 1
done

if ! curl --fail --silent "http://$OGS_BIND_ADDRESS/health" >/dev/null; then
  echo "The server did not become healthy. See $ogs_log_dir/server.log" >&2
  exit 1
fi

ogs_health_response=$(curl --fail --silent "http://$OGS_BIND_ADDRESS/health")
if ! jq -e \
  '.service == "omarchy-gaming-system" and .status == "ok" and .database == "ok"' \
  <<<"$ogs_health_response" >/dev/null; then
  echo "Health smoke returned an unexpected gaming-system identity" >&2
  exit 1
fi

if [[ "$ogs_smoke_test" == true ]]; then
  ogs_game_catalog=$(curl \
    --fail \
    --silent \
    "http://$OGS_BIND_ADDRESS/v1/games")
  if ! jq -e 'keys == ["games"] and .games == []' \
    <<<"$ogs_game_catalog" >/dev/null; then
    echo "Game catalog smoke advertised an unexpected production game" >&2
    exit 1
  fi

  ogs_registration_username="smoke_$(date +%s)_$$"
  ogs_registration_password="TEST-ONLY-registration-passphrase"
  ogs_registration_url="http://$OGS_BIND_ADDRESS/v1/accounts"
  ogs_registration_payload=$(printf \
    '{"username":"%s","password":"%s"}' \
    "$ogs_registration_username" \
    "$ogs_registration_password")

  ogs_registration_response=$(curl \
    --fail \
    --silent \
    --header "Content-Type: application/json" \
    --data "$ogs_registration_payload" \
    "$ogs_registration_url")

  if ! grep -Fq "\"username\":\"$ogs_registration_username\"" \
    <<<"$ogs_registration_response"; then
    echo "Registration smoke returned an unexpected response" >&2
    exit 1
  fi

  if grep -Fq 'password' <<<"$ogs_registration_response"; then
    echo "Registration smoke response exposed password-derived data" >&2
    exit 1
  fi

  ogs_duplicate_status=$(curl \
    --silent \
    --output "$ogs_log_dir/duplicate-registration.json" \
    --write-out '%{http_code}' \
    --header "Content-Type: application/json" \
    --data "$ogs_registration_payload" \
    "$ogs_registration_url")

  if [[ "$ogs_duplicate_status" != 409 ]] \
    || ! grep -Fq '"code":"username_taken"' \
      "$ogs_log_dir/duplicate-registration.json"; then
    echo "Duplicate registration smoke did not return username_taken (409)" >&2
    exit 1
  fi

  ogs_session_payload=$(printf \
    '{"username":"%s","password":"%s","device_name":"Pipeline smoke"}' \
    "$ogs_registration_username" \
    "$ogs_registration_password")
  ogs_session_response=$(curl \
    --fail \
    --silent \
    --header "Content-Type: application/json" \
    --data "$ogs_session_payload" \
    "http://$OGS_BIND_ADDRESS/v1/sessions")

  if grep -Eq 'token_hash|account_id' <<<"$ogs_session_response"; then
    echo "Session creation smoke exposed private persistence fields" >&2
    exit 1
  fi

  ogs_session_token=$(jq -er \
    '.token | select(startswith("ogs1_"))' \
    <<<"$ogs_session_response")
  ogs_session_id=$(jq -er '.session.id' <<<"$ogs_session_response")

  ogs_session_list=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/sessions")
  if ! jq -e \
    --arg session_id "$ogs_session_id" \
    '.sessions | any(.id == $session_id and .current == true)' \
    <<<"$ogs_session_list" >/dev/null; then
    echo "Session list smoke did not return the current device" >&2
    exit 1
  fi

  ogs_persona_handle="p$(date +%s)_$$"
  ogs_persona_updated_handle="${ogs_persona_handle}_u"
  ogs_persona_payload=$(jq -nc \
    --arg handle "$ogs_persona_handle" \
    '{handle: $handle, display_name: "Pipeline Player", bio: "Live smoke persona", status_message: "Ready"}')
  ogs_persona_response=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_persona_payload" \
    "http://$OGS_BIND_ADDRESS/v1/personas")

  if ! jq -e \
    --arg handle "$ogs_persona_handle" \
    '(.handle == $handle) and
     ((keys | sort) == ["bio", "created_at", "display_name", "handle", "id", "status_message", "updated_at"])' \
    <<<"$ogs_persona_response" >/dev/null; then
    echo "Persona creation smoke returned an unexpected public profile" >&2
    exit 1
  fi
  if grep -Eq 'account_id|password|session_id|token(_hash)?' \
    <<<"$ogs_persona_response"; then
    echo "Persona creation smoke exposed private fields" >&2
    exit 1
  fi
  ogs_persona_id=$(jq -er '.id' <<<"$ogs_persona_response")
  ogs_sync_baseline=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/sync")
  if ! jq -e \
    '.events == [] and .next_cursor == 0 and
     .has_more == false and .reset_required == false' \
    <<<"$ogs_sync_baseline" >/dev/null; then
    echo "Persona sync smoke did not return an empty baseline" >&2
    exit 1
  fi
  ogs_sync_cursor=$(jq -er '.next_cursor' <<<"$ogs_sync_baseline")

  ogs_public_persona=$(curl \
    --fail \
    --silent \
    "http://$OGS_BIND_ADDRESS/v1/personas/by-handle/${ogs_persona_handle^^}")
  if [[ "$ogs_public_persona" != "$ogs_persona_response" ]]; then
    echo "Public persona lookup did not return the created profile" >&2
    exit 1
  fi

  ogs_persona_list=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas")
  if ! jq -e \
    --arg persona_id "$ogs_persona_id" \
    '.personas | any(.id == $persona_id)' \
    <<<"$ogs_persona_list" >/dev/null; then
    echo "Persona inventory smoke did not return the owned persona" >&2
    exit 1
  fi
  if grep -Eq 'account_id|password|session_id|token(_hash)?' \
    <<<"$ogs_persona_list"; then
    echo "Persona inventory smoke exposed private fields" >&2
    exit 1
  fi

  ogs_persona_update_payload=$(jq -nc \
    --arg handle "$ogs_persona_updated_handle" \
    '{handle: $handle, display_name: "Updated Pipeline Player", status_message: "In a match"}')
  ogs_persona_updated=$(curl \
    --fail \
    --silent \
    --request PATCH \
    --header "Authorization: Bearer $ogs_session_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_persona_update_payload" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id")
  if ! jq -e \
    --arg handle "$ogs_persona_updated_handle" \
    '.handle == $handle and .display_name == "Updated Pipeline Player" and .status_message == "In a match"' \
    <<<"$ogs_persona_updated" >/dev/null; then
    echo "Persona update smoke returned an unexpected profile" >&2
    exit 1
  fi

  ogs_old_handle_status=$(curl \
    --silent \
    --output "$ogs_log_dir/old-persona-handle.json" \
    --write-out '%{http_code}' \
    "http://$OGS_BIND_ADDRESS/v1/personas/by-handle/$ogs_persona_handle")
  if [[ "$ogs_old_handle_status" != 404 ]] \
    || ! grep -Fq '"code":"persona_not_found"' \
      "$ogs_log_dir/old-persona-handle.json"; then
    echo "Old persona handle remained reachable after update" >&2
    exit 1
  fi

  ogs_updated_public_persona=$(curl \
    --fail \
    --silent \
    "http://$OGS_BIND_ADDRESS/v1/personas/by-handle/${ogs_persona_updated_handle^^}")
  if [[ "$ogs_updated_public_persona" != "$ogs_persona_updated" ]]; then
    echo "Updated public persona lookup did not match the owner edit" >&2
    exit 1
  fi

  ogs_peer_username="peer_$(date +%s)_$$"
  ogs_peer_password="TEST-ONLY-peer-passphrase"
  ogs_peer_registration_payload=$(jq -nc \
    --arg username "$ogs_peer_username" \
    --arg password "$ogs_peer_password" \
    '{username: $username, password: $password}')
  curl \
    --fail \
    --silent \
    --output /dev/null \
    --header "Content-Type: application/json" \
    --data "$ogs_peer_registration_payload" \
    "$ogs_registration_url"
  ogs_peer_session_payload=$(jq -nc \
    --arg username "$ogs_peer_username" \
    --arg password "$ogs_peer_password" \
    '{username: $username, password: $password, device_name: "Pipeline peer"}')
  ogs_peer_session=$(curl \
    --fail \
    --silent \
    --header "Content-Type: application/json" \
    --data "$ogs_peer_session_payload" \
    "http://$OGS_BIND_ADDRESS/v1/sessions")
  ogs_peer_token=$(jq -er '.token | select(startswith("ogs1_"))' \
    <<<"$ogs_peer_session")
  ogs_peer_handle="peerp$(date +%s)_$$"
  ogs_peer_persona_payload=$(jq -nc \
    --arg handle "$ogs_peer_handle" \
    '{handle: $handle, display_name: "Pipeline Peer"}')
  ogs_peer_persona=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_peer_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_peer_persona_payload" \
    "http://$OGS_BIND_ADDRESS/v1/personas")
  ogs_peer_persona_id=$(jq -er '.id' <<<"$ogs_peer_persona")

  ogs_connection_request=$(curl \
    --fail \
    --silent \
    --request PUT \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/connection-requests/$ogs_peer_persona_id")
  if ! jq -e \
    --arg peer_id "$ogs_peer_persona_id" \
    '.persona.id == $peer_id and
     ((keys | sort) == ["created_at", "persona"]) and
     ((.persona | keys | sort) == ["bio", "created_at", "display_name", "handle", "id", "status_message", "updated_at"])' \
    <<<"$ogs_connection_request" >/dev/null; then
    echo "Connection request smoke returned an unexpected safe response" >&2
    exit 1
  fi
  ogs_request_sync=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/sync?after=$ogs_sync_cursor")
  if ! jq -e \
    '[.events[].type] == ["connection_requests_changed"] and
     .has_more == false and .reset_required == false' \
    <<<"$ogs_request_sync" >/dev/null; then
    echo "Connection request sync smoke returned unexpected events" >&2
    exit 1
  fi
  ogs_sync_cursor=$(jq -er '.next_cursor' <<<"$ogs_request_sync")

  ogs_peer_requests=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_peer_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_peer_persona_id/connection-requests")
  if ! jq -e \
    --arg persona_id "$ogs_persona_id" \
    '.incoming | any(.persona.id == $persona_id)' \
    <<<"$ogs_peer_requests" >/dev/null; then
    echo "Connection request inventory smoke lost the incoming request" >&2
    exit 1
  fi

  ogs_connection=$(curl \
    --fail \
    --silent \
    --request PUT \
    --header "Authorization: Bearer $ogs_peer_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_peer_persona_id/connections/$ogs_persona_id")
  if ! jq -e \
    --arg persona_id "$ogs_persona_id" \
    '.persona.id == $persona_id and has("connected_at")' \
    <<<"$ogs_connection" >/dev/null; then
    echo "Connection acceptance smoke returned an unexpected response" >&2
    exit 1
  fi
  ogs_acceptance_sync=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/sync?after=$ogs_sync_cursor")
  if ! jq -e \
    '[.events[].type] ==
       ["connection_requests_changed", "connections_changed", "conversation_changed"] and
     (.events[2].conversation_id | type) == "string"' \
    <<<"$ogs_acceptance_sync" >/dev/null; then
    echo "Connection acceptance sync smoke returned unexpected events" >&2
    exit 1
  fi
  ogs_sync_cursor=$(jq -er '.next_cursor' <<<"$ogs_acceptance_sync")

  ogs_connections=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/connections")
  if ! jq -e \
    --arg peer_id "$ogs_peer_persona_id" \
    '.connections | any(.persona.id == $peer_id)' \
    <<<"$ogs_connections" >/dev/null; then
    echo "Connection inventory smoke lost the accepted peer" >&2
    exit 1
  fi

  ogs_challenge_key=$(python3 -c 'import uuid; print(uuid.uuid4())')
  ogs_challenge_payload=$(jq -nc \
    --arg idempotency_key "$ogs_challenge_key" \
    --arg challenged_persona_id "$ogs_peer_persona_id" \
    '{idempotency_key: $idempotency_key,
      challenged_persona_id: $challenged_persona_id,
      game_key: "smoke_game",
      game_version: 1}')
  ogs_challenge_status=$(curl \
    --silent \
    --output "$ogs_log_dir/unavailable-game-challenge.json" \
    --write-out '%{http_code}' \
    --header "Authorization: Bearer $ogs_session_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_challenge_payload" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/game-challenges")
  if [[ "$ogs_challenge_status" != 409 ]] \
    || ! grep -Fq '"code":"game_unavailable"' \
      "$ogs_log_dir/unavailable-game-challenge.json"; then
    echo "Empty-registry game challenge did not fail closed" >&2
    exit 1
  fi
  ogs_challenge_sync=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/sync?after=$ogs_sync_cursor")
  if ! jq -e \
    '.events == [] and .has_more == false and .reset_required == false' \
    <<<"$ogs_challenge_sync" >/dev/null; then
    echo "Rejected game challenge emitted a partial sync event" >&2
    exit 1
  fi

  ogs_conversations=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/conversations")
  if ! jq -e \
    --arg peer_id "$ogs_peer_persona_id" \
    '.conversations | length == 1 and
     .[0].other_persona.id == $peer_id and
     .[0].unread_count == 1 and
     .[0].latest_message.type == "system" and
     .[0].latest_message.system.type == "connection_accepted"' \
    <<<"$ogs_conversations" >/dev/null; then
    echo "Conversation inventory smoke lost the typed acceptance event" >&2
    exit 1
  fi
  ogs_conversation_id=$(jq -er '.conversations[0].id' <<<"$ogs_conversations")

  ogs_message_payload=$(jq -nc '{body: "Live pipeline hello"}')
  ogs_message=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_peer_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_message_payload" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_peer_persona_id/conversations/$ogs_conversation_id/messages")
  if ! jq -e \
    --arg peer_id "$ogs_peer_persona_id" \
    '.type == "user" and .body == "Live pipeline hello" and
     .sender.id == $peer_id and has("id") and has("sequence") and has("created_at")' \
    <<<"$ogs_message" >/dev/null; then
    echo "Inbox message smoke returned an unexpected user message" >&2
    exit 1
  fi
  ogs_message_id=$(jq -er '.id' <<<"$ogs_message")

  ogs_history=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/conversations/$ogs_conversation_id/messages")
  if ! jq -e \
    '.messages | length == 2 and
     .[0].type == "system" and .[1].type == "user" and
     .[0].sequence < .[1].sequence' \
    <<<"$ogs_history" >/dev/null; then
    echo "Inbox history smoke lost typed ascending messages" >&2
    exit 1
  fi

  ogs_read=$(curl \
    --fail \
    --silent \
    --request PUT \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/conversations/$ogs_conversation_id/read/$ogs_message_id")
  if ! jq -e \
    --arg message_id "$ogs_message_id" \
    '.through_message_id == $message_id and .unread_count == 0' \
    <<<"$ogs_read" >/dev/null; then
    echo "Inbox read smoke did not clear the private unread count" >&2
    exit 1
  fi
  ogs_inbox_sync=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/sync?after=$ogs_sync_cursor")
  if ! jq -e \
    --arg conversation_id "$ogs_conversation_id" \
    '[.events[].type] == ["conversation_changed", "conversation_changed"] and
     all(.events[]; .conversation_id == $conversation_id) and
     (tostring | contains("Live pipeline hello") | not)' \
    <<<"$ogs_inbox_sync" >/dev/null; then
    echo "Inbox sync smoke returned unexpected or non-minimal events" >&2
    exit 1
  fi
  ogs_sync_cursor=$(jq -er '.next_cursor' <<<"$ogs_inbox_sync")

  ogs_connection_remove_status=$(curl \
    --silent \
    --output /dev/null \
    --write-out '%{http_code}' \
    --request DELETE \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/connections/$ogs_peer_persona_id")
  if [[ "$ogs_connection_remove_status" != 204 ]]; then
    echo "Connection removal smoke did not return 204" >&2
    exit 1
  fi

  curl \
    --fail \
    --silent \
    --output /dev/null \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/conversations/$ogs_conversation_id/messages"
  ogs_disconnected_send_status=$(curl \
    --silent \
    --output "$ogs_log_dir/disconnected-inbox-send.json" \
    --write-out '%{http_code}' \
    --header "Authorization: Bearer $ogs_session_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_message_payload" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/conversations/$ogs_conversation_id/messages")
  if [[ "$ogs_disconnected_send_status" != 409 ]] \
    || ! grep -Fq '"code":"conversation_unavailable"' \
      "$ogs_log_dir/disconnected-inbox-send.json"; then
    echo "Disconnected inbox send did not fail while history remained readable" >&2
    exit 1
  fi

  ogs_block=$(curl \
    --fail \
    --silent \
    --request PUT \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/blocks/$ogs_peer_persona_id")
  if ! jq -e \
    --arg peer_id "$ogs_peer_persona_id" \
    '.persona.id == $peer_id and has("created_at")' \
    <<<"$ogs_block" >/dev/null; then
    echo "Persona block smoke returned an unexpected response" >&2
    exit 1
  fi

  ogs_blocked_request_status=$(curl \
    --silent \
    --output "$ogs_log_dir/blocked-connection-request.json" \
    --write-out '%{http_code}' \
    --request PUT \
    --header "Authorization: Bearer $ogs_peer_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_peer_persona_id/connection-requests/$ogs_persona_id")
  if [[ "$ogs_blocked_request_status" != 409 ]] \
    || ! grep -Fq '"code":"connection_unavailable"' \
      "$ogs_log_dir/blocked-connection-request.json"; then
    echo "Blocked connection request did not fail privately" >&2
    exit 1
  fi

  ogs_blocks=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/blocks")
  if ! jq -e \
    --arg peer_id "$ogs_peer_persona_id" \
    '.blocks | any(.persona.id == $peer_id)' \
    <<<"$ogs_blocks" >/dev/null; then
    echo "Private block inventory smoke lost the blocked peer" >&2
    exit 1
  fi

  ogs_unblock_status=$(curl \
    --silent \
    --output /dev/null \
    --write-out '%{http_code}' \
    --request DELETE \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/blocks/$ogs_peer_persona_id")
  if [[ "$ogs_unblock_status" != 204 ]]; then
    echo "Persona unblock smoke did not return 204" >&2
    exit 1
  fi

  curl \
    --fail \
    --silent \
    --output /dev/null \
    --request PUT \
    --header "Authorization: Bearer $ogs_peer_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_peer_persona_id/connection-requests/$ogs_persona_id"
  ogs_request_cancel_status=$(curl \
    --silent \
    --output /dev/null \
    --write-out '%{http_code}' \
    --request DELETE \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/connections/$ogs_peer_persona_id")
  if [[ "$ogs_request_cancel_status" != 204 ]]; then
    echo "Pending connection cancellation smoke did not return 204" >&2
    exit 1
  fi
  ogs_social_sync=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/personas/$ogs_persona_id/sync?after=$ogs_sync_cursor")
  if ! jq -e \
    '[.events[].type] ==
       ["connection_requests_changed", "connections_changed", "blocks_changed",
        "blocks_changed", "connection_requests_changed",
        "connection_requests_changed", "connections_changed"] and
     .has_more == false and .reset_required == false' \
    <<<"$ogs_social_sync" >/dev/null; then
    echo "Social sync smoke lost cursor order or mutation invalidations" >&2
    exit 1
  fi

  ogs_mfa_enrollment_payload=$(jq -nc \
    --arg password "$ogs_registration_password" \
    '{password: $password}')
  ogs_mfa_enrollment=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_mfa_enrollment_payload" \
    "http://$OGS_BIND_ADDRESS/v1/account/mfa")
  ogs_mfa_secret=$(jq -er '.secret' <<<"$ogs_mfa_enrollment")
  if ! jq -e \
    '.provisioning_uri | startswith("otpauth://totp/OmarchyGS%3A")' \
    <<<"$ogs_mfa_enrollment" >/dev/null; then
    echo "MFA enrollment did not return the OmarchyGS provisioning contract" >&2
    exit 1
  fi

  ogs_totp_code=$(
    python3 - "$ogs_mfa_secret" <<'PY'
import base64
import hashlib
import hmac
import struct
import sys
import time

secret = base64.b32decode(sys.argv[1])
counter = int(time.time()) // 30
digest = hmac.new(secret, struct.pack(">Q", counter), hashlib.sha1).digest()
offset = digest[-1] & 0x0F
value = struct.unpack(">I", digest[offset:offset + 4])[0] & 0x7FFFFFFF
print(f"{value % 1_000_000:06d}")
PY
  )
  ogs_mfa_confirmation_payload=$(jq -nc \
    --arg code "$ogs_totp_code" \
    '{code: $code}')
  ogs_mfa_confirmation=$(curl \
    --fail \
    --silent \
    --header "Authorization: Bearer $ogs_session_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_mfa_confirmation_payload" \
    "http://$OGS_BIND_ADDRESS/v1/account/mfa/confirm")
  ogs_first_recovery_code=$(jq -er '.recovery_codes[0]' <<<"$ogs_mfa_confirmation")
  ogs_second_recovery_code=$(jq -er '.recovery_codes[1]' <<<"$ogs_mfa_confirmation")

  ogs_mfa_login=$(curl \
    --fail \
    --silent \
    --header "Content-Type: application/json" \
    --data "$ogs_session_payload" \
    "http://$OGS_BIND_ADDRESS/v1/sessions")
  ogs_mfa_challenge=$(jq -er \
    '.challenge_token | select(startswith("ogm1_"))' \
    <<<"$ogs_mfa_login")
  if jq -e 'has("token") or has("session")' <<<"$ogs_mfa_login" >/dev/null; then
    echo "MFA-gated primary login created a session before factor verification" >&2
    exit 1
  fi

  ogs_mfa_completion_payload=$(jq -nc \
    --arg challenge_token "$ogs_mfa_challenge" \
    --arg code "$ogs_first_recovery_code" \
    '{challenge_token: $challenge_token, code: $code}')
  ogs_mfa_session=$(curl \
    --fail \
    --silent \
    --header "Content-Type: application/json" \
    --data "$ogs_mfa_completion_payload" \
    "http://$OGS_BIND_ADDRESS/v1/sessions/mfa")
  jq -er '.token | select(startswith("ogs1_"))' \
    <<<"$ogs_mfa_session" >/dev/null

  ogs_replay_login=$(curl \
    --fail \
    --silent \
    --header "Content-Type: application/json" \
    --data "$ogs_session_payload" \
    "http://$OGS_BIND_ADDRESS/v1/sessions")
  ogs_replay_challenge=$(jq -er '.challenge_token' <<<"$ogs_replay_login")
  ogs_replay_payload=$(jq -nc \
    --arg challenge_token "$ogs_replay_challenge" \
    --arg code "$ogs_first_recovery_code" \
    '{challenge_token: $challenge_token, code: $code}')
  ogs_replay_status=$(curl \
    --silent \
    --output "$ogs_log_dir/replayed-mfa-code.json" \
    --write-out '%{http_code}' \
    --header "Content-Type: application/json" \
    --data "$ogs_replay_payload" \
    "http://$OGS_BIND_ADDRESS/v1/sessions/mfa")
  if [[ "$ogs_replay_status" != 401 ]] \
    || ! grep -Fq '"code":"invalid_mfa_code"' \
      "$ogs_log_dir/replayed-mfa-code.json"; then
    echo "Used MFA recovery code was accepted again" >&2
    exit 1
  fi

  ogs_mfa_disable_payload=$(jq -nc \
    --arg password "$ogs_registration_password" \
    --arg code "$ogs_second_recovery_code" \
    '{password: $password, code: $code}')
  ogs_mfa_disable_status=$(curl \
    --silent \
    --output /dev/null \
    --write-out '%{http_code}' \
    --request DELETE \
    --header "Authorization: Bearer $ogs_session_token" \
    --header "Content-Type: application/json" \
    --data "$ogs_mfa_disable_payload" \
    "http://$OGS_BIND_ADDRESS/v1/account/mfa")
  if [[ "$ogs_mfa_disable_status" != 204 ]]; then
    echo "MFA disable smoke did not return 204" >&2
    exit 1
  fi

  ogs_post_mfa_login=$(curl \
    --fail \
    --silent \
    --header "Content-Type: application/json" \
    --data "$ogs_session_payload" \
    "http://$OGS_BIND_ADDRESS/v1/sessions")
  if ! jq -e '.token | startswith("ogs1_")' \
    <<<"$ogs_post_mfa_login" >/dev/null; then
    echo "Password-only login was not restored after MFA disablement" >&2
    exit 1
  fi

  ogs_revoke_status=$(curl \
    --silent \
    --output /dev/null \
    --write-out '%{http_code}' \
    --request DELETE \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/sessions/$ogs_session_id")
  if [[ "$ogs_revoke_status" != 204 ]]; then
    echo "Session revocation smoke did not return 204" >&2
    exit 1
  fi

  ogs_revoked_status=$(curl \
    --silent \
    --output "$ogs_log_dir/revoked-session.json" \
    --write-out '%{http_code}' \
    --header "Authorization: Bearer $ogs_session_token" \
    "http://$OGS_BIND_ADDRESS/v1/sessions")
  if [[ "$ogs_revoked_status" != 401 ]] \
    || ! grep -Fq '"code":"invalid_session"' \
      "$ogs_log_dir/revoked-session.json"; then
    echo "Revoked session smoke remained usable" >&2
    exit 1
  fi
fi

echo "Server ready at http://$OGS_BIND_ADDRESS"
echo "Closing the QML window will stop the Rust server; PostgreSQL stays running."
qml6 "$ogs_root/client/qml/Main.qml" "${ogs_qml_arguments[@]}"
