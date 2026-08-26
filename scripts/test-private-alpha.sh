#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"
ogs_database="omarchygs_private_alpha_${BASHPID}"
ogs_admin_url="${OGS_TEST_POSTGRES_ADMIN_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/postgres}"
ogs_database_url="postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/$ogs_database"
ogs_server_binary="$ogs_root/target/debug/omarchy-gaming-system-server"
ogs_admin_binary="$ogs_root/target/debug/omarchygs-admin"
ogs_server_pid=""
ogs_server_log="$ogs_temp/server.log"
ogs_request_file="$ogs_temp/request.json"
ogs_password="TEST-ONLY-private-alpha-registration-passphrase"

stop_server() {
  if [[ -n "$ogs_server_pid" ]] && kill -0 "$ogs_server_pid" 2>/dev/null; then
    kill "$ogs_server_pid" 2>/dev/null || true
    wait "$ogs_server_pid" 2>/dev/null || true
  fi
  ogs_server_pid=""
}

cleanup() {
  local ogs_status=$?
  stop_server
  if ((ogs_status != 0)) && [[ -s "$ogs_server_log" ]]; then
    echo "private-alpha server output:" >&2
    sed -n '1,180p' "$ogs_server_log" >&2
  fi
  psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
    -c "DROP DATABASE IF EXISTS $ogs_database WITH (FORCE)" >/dev/null 2>&1 || true
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo cmp curl docker jq mktemp openssl psql python3 rg; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required private-alpha command is unavailable: $ogs_command" >&2
    exit 1
  }
done
[[ "$ogs_database" =~ ^[a-z0-9_]+$ ]]

reserve_port() {
  python3 - <<'PY'
import socket

with socket.socket() as listener:
    listener.bind(("127.0.0.1", 0))
    print(listener.getsockname()[1])
PY
}

write_private_json() {
  local ogs_path="$1"
  local ogs_document="$2"
  (umask 077; printf '%s\n' "$ogs_document" >"$ogs_path")
}

issue_invitation() {
  local ogs_label="$1"
  local ogs_command_file="$ogs_temp/issue-$(python3 -c 'import uuid; print(uuid.uuid4())').json"
  local ogs_operation_id
  local ogs_document
  local ogs_receipt

  ogs_operation_id=$(python3 -c 'import uuid; print(uuid.uuid4())')
  ogs_document=$(jq -nc \
    --arg operation "$ogs_operation_id" \
    --arg label "$ogs_label" \
    '{command: "issue_registration_invite",
      idempotency_key: $operation,
      label: $label,
      valid_for_hours: 24,
      actor: "private-alpha-drill",
      reason: "Exercise isolated invite-only admission"}')
  write_private_json "$ogs_command_file" "$ogs_document"
  ogs_receipt=$(DATABASE_URL="$ogs_database_url" \
    "$ogs_admin_binary" apply "$ogs_command_file")
  rm -f -- "$ogs_command_file"
  if ! jq -e '
      (keys | sort) == [
        "action", "created_at", "expires_at", "first_delivery", "id",
        "invite_code", "label", "operation_id", "previous_state",
        "resulting_state", "target_id", "target_kind"
      ] and
      .action == "issue_registration_invite" and
      .target_kind == "registration_invite" and
      .previous_state == "absent" and .resulting_state == "issued" and
      .first_delivery == true and
      (.invite_code | startswith("ogsi_") and length == 48)' \
      <<<"$ogs_receipt" >/dev/null; then
    echo "private-alpha invitation issue returned an unexpected receipt" >&2
    return 1
  fi
  printf '%s\n' "$ogs_receipt"
}

post_registration() {
  local ogs_code="$1"
  local ogs_username="$2"
  local ogs_registration_password="$3"
  local ogs_output="$4"
  local ogs_document

  ogs_document=$(jq -nc \
    --arg invite_code "$ogs_code" \
    --arg username "$ogs_username" \
    --arg password "$ogs_registration_password" \
    '{invite_code: $invite_code, username: $username, password: $password}')
  write_private_json "$ogs_request_file" "$ogs_document"
  curl --silent \
    --output "$ogs_output" \
    --write-out '%{http_code}' \
    --header 'Content-Type: application/json' \
    --data-binary "@$ogs_request_file" \
    "http://127.0.0.1:$ogs_port/v1/accounts"
  rm -f -- "$ogs_request_file"
}

cd "$ogs_root"
docker compose up -d --wait db
cargo build -p omarchy-gaming-system-server --bins
[[ -x "$ogs_server_binary" ]]
[[ -x "$ogs_admin_binary" ]]

psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE $ogs_database OWNER omarchy_gaming_system" >/dev/null

ogs_mfa_key=$(openssl rand -base64 32 | tr '+/' '-_' | tr -d '=\n')
ogs_port=$(reserve_port)
env \
  DATABASE_URL="$ogs_database_url" \
  OGS_BIND_ADDRESS="127.0.0.1:$ogs_port" \
  OGS_MFA_ENCRYPTION_KEY="$ogs_mfa_key" \
  RUST_LOG=omarchy_gaming_system_server=warn \
  "$ogs_server_binary" >"$ogs_server_log" 2>&1 &
ogs_server_pid=$!
# A cold 17-migration database can exceed ten seconds after the gate's
# compile/provider load. Keep the wait bounded without making that load a
# false admission failure.
for _ in {1..300}; do
  if ! kill -0 "$ogs_server_pid" 2>/dev/null; then
    echo "private-alpha server stopped during startup" >&2
    exit 1
  fi
  if curl --fail --silent "http://127.0.0.1:$ogs_port/health" >/dev/null; then
    break
  fi
  sleep 0.1
done
curl --fail --silent "http://127.0.0.1:$ogs_port/health" >/dev/null

ogs_first_issue=$(issue_invitation "First external tester")
ogs_first_invite=$(jq -er '.invite_code' <<<"$ogs_first_issue")
ogs_first_invite_id=$(jq -er '.target_id' <<<"$ogs_first_issue")
ogs_username="alpha_$(python3 -c 'import uuid; print(uuid.uuid4().hex[:12])')"

ogs_created_status=$(post_registration \
  "$ogs_first_invite" "$ogs_username" "$ogs_password" "$ogs_temp/created.json")
if [[ "$ogs_created_status" != 201 ]] \
  || ! jq -e \
    --arg username "$ogs_username" \
    '(keys | sort) == ["id", "username"] and .username == $username' \
    "$ogs_temp/created.json" >/dev/null; then
  echo "invited private-alpha registration did not create the expected account" >&2
  exit 1
fi

ogs_replay_status=$(post_registration \
  "$ogs_first_invite" "${ogs_username^^}" "$ogs_password" "$ogs_temp/replay.json")
if [[ "$ogs_replay_status" != 200 ]] \
  || ! cmp -s "$ogs_temp/created.json" "$ogs_temp/replay.json"; then
  echo "private-alpha registration replay did not recover the original receipt" >&2
  exit 1
fi

for ogs_changed_value in changed_username changed_password; do
  if [[ "$ogs_changed_value" == changed_username ]]; then
    ogs_changed_username="${ogs_username}_other"
    ogs_changed_password="$ogs_password"
  else
    ogs_changed_username="$ogs_username"
    ogs_changed_password="TEST-ONLY-private-alpha-changed-passphrase"
  fi
  ogs_changed_status=$(post_registration \
    "$ogs_first_invite" "$ogs_changed_username" "$ogs_changed_password" \
    "$ogs_temp/$ogs_changed_value.json")
  if [[ "$ogs_changed_status" != 403 ]] \
    || ! jq -e \
      '(keys == ["error"]) and
       (.error | keys | sort) == ["code", "message"] and
       .error.code == "invalid_invitation"' \
      "$ogs_temp/$ogs_changed_value.json" >/dev/null; then
    echo "used invitation disclosed or admitted changed registration intent" >&2
    exit 1
  fi
done

ogs_login_document=$(jq -nc \
  --arg username "$ogs_username" \
  --arg password "$ogs_password" \
  '{username: $username, password: $password, device_name: "Private alpha drill"}')
write_private_json "$ogs_request_file" "$ogs_login_document"
ogs_login_status=$(curl --silent \
  --output "$ogs_temp/login.json" \
  --write-out '%{http_code}' \
  --header 'Content-Type: application/json' \
  --data-binary "@$ogs_request_file" \
  "http://127.0.0.1:$ogs_port/v1/sessions")
rm -f -- "$ogs_request_file"
if [[ "$ogs_login_status" != 201 ]] \
  || ! jq -e '.token | startswith("ogs1_")' "$ogs_temp/login.json" >/dev/null; then
  echo "invited account could not complete ordinary sign-in" >&2
  exit 1
fi

ogs_second_issue=$(issue_invitation "Revoked external tester")
ogs_second_invite=$(jq -er '.invite_code' <<<"$ogs_second_issue")
ogs_second_invite_id=$(jq -er '.target_id' <<<"$ogs_second_issue")
ogs_revoke_file="$ogs_temp/revoke.json"
ogs_revoke_operation=$(python3 -c 'import uuid; print(uuid.uuid4())')
ogs_revoke_document=$(jq -nc \
  --arg operation "$ogs_revoke_operation" \
  --arg invite_id "$ogs_second_invite_id" \
  '{command: "revoke_registration_invite",
    idempotency_key: $operation,
    invite_id: $invite_id,
    actor: "private-alpha-drill",
    reason: "Prove invitation revocation before delivery"}')
write_private_json "$ogs_revoke_file" "$ogs_revoke_document"
ogs_revoke_receipt=$(DATABASE_URL="$ogs_database_url" \
  "$ogs_admin_binary" apply "$ogs_revoke_file")
rm -f -- "$ogs_revoke_file"
if ! jq -e \
  --arg invite_id "$ogs_second_invite_id" \
  '.target_kind == "registration_invite" and .target_id == $invite_id and
   .action == "revoke_registration_invite" and
   .previous_state == "issued" and .resulting_state == "revoked"' \
  <<<"$ogs_revoke_receipt" >/dev/null; then
  echo "private-alpha invitation revocation returned an unexpected receipt" >&2
  exit 1
fi
ogs_revoked_status=$(post_registration \
  "$ogs_second_invite" "revoked_alpha" "$ogs_password" "$ogs_temp/revoked.json")
if [[ "$ogs_revoked_status" != 403 ]] \
  || ! jq -e '.error.code == "invalid_invitation"' "$ogs_temp/revoked.json" >/dev/null; then
  echo "revoked invitation remained usable" >&2
  exit 1
fi

ogs_inventory=$(DATABASE_URL="$ogs_database_url" "$ogs_admin_binary" invites all 10)
if ! jq -e \
  --arg used_id "$ogs_first_invite_id" \
  --arg revoked_id "$ogs_second_invite_id" \
  --arg username "$ogs_username" \
  '(keys == ["invitations"]) and (.invitations | length == 2) and
   (.invitations | any(.id == $used_id and .state == "used" and
                       .redeemed_username == $username)) and
   (.invitations | any(.id == $revoked_id and .state == "revoked" and
                       .redeemed_username == null)) and
   all(.invitations[];
       (keys | sort) == ["created_at", "expires_at", "id", "label",
                         "redeemed_username", "revoked_at", "state", "used_at"])' \
  <<<"$ogs_inventory" >/dev/null; then
  echo "private-alpha invitation inventory was not exact" >&2
  exit 1
fi
for ogs_secret in "$ogs_first_invite" "$ogs_second_invite" code_hash password_hash token_hash; do
  if grep -Fq -- "$ogs_secret" <<<"$ogs_inventory"; then
    echo "private-alpha invitation inventory exposed secret material" >&2
    exit 1
  fi
done

ogs_counts=$(psql "$ogs_database_url" -v ON_ERROR_STOP=1 -Atc \
  "SELECT (SELECT count(*) FROM accounts),
          (SELECT count(*) FROM registration_invites),
          (SELECT count(*) FROM registration_invites WHERE used_at IS NOT NULL),
          (SELECT count(*) FROM registration_invites WHERE revoked_at IS NOT NULL),
          (SELECT count(*) FROM operator_audit_events
             WHERE target_registration_invite_id IS NOT NULL),
          (SELECT min(octet_length(code_hash)) FROM registration_invites),
          (SELECT max(octet_length(code_hash)) FROM registration_invites)" | tr '|' ' ')
if [[ "$ogs_counts" != "1 2 1 1 3 32 32" ]]; then
  echo "unexpected private-alpha persistence evidence: $ogs_counts" >&2
  exit 1
fi
for ogs_secret in "$ogs_first_invite" "$ogs_second_invite" "$ogs_password"; do
  if rg --fixed-strings -- "$ogs_secret" "$ogs_server_log" >/dev/null; then
    echo "private-alpha server log exposed submitted secret material" >&2
    exit 1
  fi
done

echo "invite-only private-alpha admission drill passed"
