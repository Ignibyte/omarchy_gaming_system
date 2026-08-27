#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp="$(mktemp -d)"
ogs_source_db="omarchygs_operator_source_${BASHPID}"
ogs_restore_db="omarchygs_operator_restore_${BASHPID}"
ogs_admin_url="${OGS_TEST_POSTGRES_ADMIN_URL:-postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/postgres}"
ogs_source_url="postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/$ogs_source_db"
ogs_restore_url="postgres://omarchy_gaming_system:omarchy_gaming_system@127.0.0.1:5432/$ogs_restore_db"
ogs_server_binary="$ogs_root/target/debug/omarchy-gaming-system-server"
ogs_admin_binary="$ogs_root/target/debug/omarchygs-admin"
ogs_active_server_pid=""
ogs_report_id="a0000000-0000-4000-8000-000000000001"
ogs_subject_account_id="20000000-0000-4000-8000-000000000001"
ogs_session_id="70000000-0000-4000-8000-000000000001"
ogs_raw_token="ogs1_UlJSUlJSUlJSUlJSUlJSUlJSUlJSUlJSUlJSUlJSUlI"
ogs_source_server_id=""

stop_server() {
  if [[ -n "$ogs_active_server_pid" ]] && kill -0 "$ogs_active_server_pid" 2>/dev/null; then
    kill "$ogs_active_server_pid" 2>/dev/null || true
    wait "$ogs_active_server_pid" 2>/dev/null || true
  fi
  ogs_active_server_pid=""
}

cleanup() {
  stop_server
  psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
    -c "DROP DATABASE IF EXISTS $ogs_source_db WITH (FORCE)" \
    -c "DROP DATABASE IF EXISTS $ogs_restore_db WITH (FORCE)" >/dev/null 2>&1 || true
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo cmp curl cut docker jq mktemp openssl pg_dump pg_restore psql python3 sha256sum; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required operator recovery command is unavailable: $ogs_command" >&2
    exit 1
  }
done

[[ "$ogs_source_db" =~ ^[a-z0-9_]+$ ]]
[[ "$ogs_restore_db" =~ ^[a-z0-9_]+$ ]]

reserve_port() {
  python3 - <<'PY'
import socket

with socket.socket() as listener:
    listener.bind(("127.0.0.1", 0))
    print(listener.getsockname()[1])
PY
}

start_server() {
  local ogs_database_url="$1"
  local ogs_port="$2"
  local ogs_log="$3"

  stop_server
  env \
    DATABASE_URL="$ogs_database_url" \
    OGS_BIND_ADDRESS="127.0.0.1:$ogs_port" \
    OGS_MFA_ENCRYPTION_KEY="$ogs_mfa_key" \
    RUST_LOG=omarchy_gaming_system_server=warn \
    "$ogs_server_binary" >"$ogs_log" 2>&1 &
  ogs_active_server_pid=$!
  # A cold 19-migration database can exceed ten seconds after the gate's
  # compile/provider load. Keep the wait bounded without making that load a
  # false recovery failure.
  for _ in {1..300}; do
    if ! kill -0 "$ogs_active_server_pid" 2>/dev/null; then
      echo "operator recovery server stopped during startup" >&2
      sed -n '1,160p' "$ogs_log" >&2
      return 1
    fi
    if curl --fail --silent "http://127.0.0.1:$ogs_port/health" >/dev/null; then
      return 0
    fi
    sleep 0.1
  done
  echo "operator recovery server did not become healthy" >&2
  sed -n '1,160p' "$ogs_log" >&2
  return 1
}

write_table_counts() {
  local ogs_database_url="$1"
  local ogs_output="$2"
  local ogs_table
  local ogs_count

  : >"$ogs_output"
  while IFS= read -r ogs_table; do
    [[ "$ogs_table" =~ ^[a-z0-9_]+$ ]]
    ogs_count=$(psql "$ogs_database_url" -v ON_ERROR_STOP=1 -Atc \
      "SELECT count(*) FROM \"$ogs_table\"")
    printf '%s\t%s\n' "$ogs_table" "$ogs_count" >>"$ogs_output"
  done < <(
    psql "$ogs_database_url" -v ON_ERROR_STOP=1 -Atc \
      "SELECT tablename FROM pg_tables WHERE schemaname = 'public' AND tablename <> '_sqlx_migrations' ORDER BY tablename"
  )
}

assert_scalar() {
  local ogs_database_url="$1"
  local ogs_query="$2"
  local ogs_expected="$3"
  local ogs_label="$4"
  local ogs_actual

  ogs_actual=$(psql "$ogs_database_url" -v ON_ERROR_STOP=1 -Atc "$ogs_query")
  if [[ "$ogs_actual" != "$ogs_expected" ]]; then
    echo "unexpected $ogs_label: expected=$ogs_expected actual=$ogs_actual" >&2
    exit 1
  fi
}

cd "$ogs_root"
docker compose up -d --wait db
cargo build -p omarchy-gaming-system-server --bins

[[ -x "$ogs_server_binary" ]]
[[ -x "$ogs_admin_binary" ]]

psql "$ogs_admin_url" -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE $ogs_source_db OWNER omarchy_gaming_system" \
  -c "CREATE DATABASE $ogs_restore_db OWNER omarchy_gaming_system" >/dev/null

ogs_mfa_key=$(openssl rand -base64 32 | tr '+/' '-_' | tr -d '=\n')
ogs_source_port=$(reserve_port)
start_server "$ogs_source_url" "$ogs_source_port" "$ogs_temp/source-server.log"
ogs_source_server_id=$(curl --fail --silent \
  "http://127.0.0.1:$ogs_source_port/.well-known/omarchygs" | jq -er '.server_id')
stop_server

ogs_token_digest=$(printf '%s' "$ogs_raw_token" | sha256sum | cut -d ' ' -f 1)
psql "$ogs_source_url" -v ON_ERROR_STOP=1 \
  -v token_digest="$ogs_token_digest" <<'SQL' >/dev/null
INSERT INTO accounts (id, username, password_hash) VALUES
    ('10000000-0000-4000-8000-000000000001', 'recovery_reporter', 'test-only-password-hash'),
    ('20000000-0000-4000-8000-000000000001', 'recovery_subject', 'test-only-password-hash'),
    ('30000000-0000-4000-8000-000000000001', 'recovery_friend', 'test-only-password-hash');

INSERT INTO personas (id, account_id, handle, display_name, bio, status_message) VALUES
    ('40000000-0000-4000-8000-000000000001', '10000000-0000-4000-8000-000000000001', 'recovery_reporter', 'Recovery Reporter', 'Restore drill reporter', 'Reviewing'),
    ('50000000-0000-4000-8000-000000000001', '20000000-0000-4000-8000-000000000001', 'recovery_subject', 'Recovery Subject', 'Restore drill subject', 'Playing'),
    ('60000000-0000-4000-8000-000000000001', '30000000-0000-4000-8000-000000000001', 'recovery_friend', 'Recovery Friend', 'Restore drill friend', 'Connected');

INSERT INTO account_sessions (
    id, account_id, token_hash, device_name, expires_at
) VALUES (
    '70000000-0000-4000-8000-000000000001',
    '20000000-0000-4000-8000-000000000001',
    decode(:'token_digest', 'hex'),
    'Recovery drill device',
    now() + interval '30 days'
);

INSERT INTO persona_connections (
    persona_low_id, persona_high_id, requester_id, addressee_id,
    status, accepted_at
) VALUES (
    '40000000-0000-4000-8000-000000000001',
    '60000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    '60000000-0000-4000-8000-000000000001',
    'accepted', now()
);

INSERT INTO persona_blocks (blocker_id, blocked_id) VALUES (
    '50000000-0000-4000-8000-000000000001',
    '60000000-0000-4000-8000-000000000001'
);

INSERT INTO inbox_conversations (
    id, persona_low_id, persona_high_id
) VALUES (
    '80000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    '60000000-0000-4000-8000-000000000001'
);

INSERT INTO inbox_messages (
    id, message_sequence, conversation_id, sender_persona_id,
    message_type, user_body
) VALUES (
    '81000000-0000-4000-8000-000000000001', 1,
    '80000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    'user', 'Recovery drill message'
);

UPDATE inbox_conversations
SET last_message_sequence = 1, low_last_read_sequence = 1, updated_at = now()
WHERE id = '80000000-0000-4000-8000-000000000001';

INSERT INTO game_sessions (
    id, game_key, game_version, revision, status, state, authority
) VALUES (
    '90000000-0000-4000-8000-000000000001',
    'signal_siege', 1, 1, 'active', '{"fixture":"restored","turn":1}'::jsonb,
    'platform_compiled'
);

INSERT INTO game_session_participants (game_session_id, persona_id, seat) VALUES (
    '90000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001', 0
);

INSERT INTO game_session_starts (
    persona_id, idempotency_key, game_session_id, game_key, game_version
) VALUES (
    '50000000-0000-4000-8000-000000000001',
    '91000000-0000-4000-8000-000000000001',
    '90000000-0000-4000-8000-000000000001',
    'signal_siege', 1
);

INSERT INTO game_session_commands (
    game_session_id, idempotency_key, actor_persona_id,
    expected_revision, applied_revision, command, state, session_status
) VALUES (
    '90000000-0000-4000-8000-000000000001',
    '92000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001',
    0, 1, '{"kind":"fixture","action":"charge"}'::jsonb,
    '{"fixture":"restored","turn":1}'::jsonb, 'active'
);

INSERT INTO game_challenges (
    id, idempotency_key, challenger_persona_id, challenged_persona_id,
    game_key, game_version, status, expires_at, resolved_at
) VALUES (
    '93000000-0000-4000-8000-000000000001',
    '94000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001',
    'signal_siege', 2, 'declined', now() + interval '1 day', now()
);

INSERT INTO persona_sync_state (persona_id, last_event_sequence) VALUES
    ('50000000-0000-4000-8000-000000000001', 4);

INSERT INTO persona_sync_events (
    persona_id, event_sequence, event_type, conversation_id,
    game_session_id, game_challenge_id
) VALUES
    ('50000000-0000-4000-8000-000000000001', 1, 'connections_changed', NULL, NULL, NULL),
    ('50000000-0000-4000-8000-000000000001', 2, 'conversation_changed', '80000000-0000-4000-8000-000000000001', NULL, NULL),
    ('50000000-0000-4000-8000-000000000001', 3, 'game_session_changed', NULL, '90000000-0000-4000-8000-000000000001', NULL),
    ('50000000-0000-4000-8000-000000000001', 4, 'game_challenge_changed', NULL, NULL, '93000000-0000-4000-8000-000000000001');

INSERT INTO persona_reports (
    id, reporter_persona_id, subject_persona_id,
    idempotency_key, category, detail
) VALUES (
    'a0000000-0000-4000-8000-000000000001',
    '40000000-0000-4000-8000-000000000001',
    '50000000-0000-4000-8000-000000000001',
    'a1000000-0000-4000-8000-000000000001',
    'harassment', 'Recovery drill report detail'
);

INSERT INTO marketplace_sync_state (
    marketplace_origin, authority_id, key_id, marketplace_name,
    snapshot_version, snapshot_sha256
) VALUES (
    'https://market.example.test/v1/', 'omarchygs-marketplace',
    'marketplace-primary-v1', 'OmarchyGS Marketplace', 7,
    'cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc'
);

INSERT INTO marketplace_releases (
    id, game_key, publisher_id, publisher_key, rules_version,
    cartridge_version, archive_sha256, signed_identity_sha256,
    display_name, release_path, reviewed_by, review_summary,
    signed_policy, policy_version, policy_status, policy_reason,
    compatible, imported, first_seen_snapshot_version,
    last_seen_snapshot_version
) VALUES (
    'c0000000-0000-4000-8000-000000000001',
    'door-legends', 'ignibyte', '{"key_id":"publisher-primary-v1"}'::jsonb,
    1, 2,
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
    'Door Legends', 'releases/door-legends/2/', 'review-team',
    'Recovery drill reviewed release.', '{"policy":"recovery-fixture"}'::jsonb,
    5, 'active', 'Current reviewed release.', TRUE, TRUE, 3, 7
);

INSERT INTO server_cartridge_catalogs (
    id, game_key, active_release_id, admission_revision
) VALUES (
    'c1000000-0000-4000-8000-000000000001', 'door-legends',
    'c0000000-0000-4000-8000-000000000001', 3
);

INSERT INTO cartridge_catalog_audit_events (
    id, operation_id, catalog_id, action, actor, reason,
    previous_archive_sha256, resulting_archive_sha256,
    admission_revision, previous_provenance_class,
    resulting_provenance_class
) VALUES (
    'c2000000-0000-4000-8000-000000000001',
    'c3000000-0000-4000-8000-000000000001',
    'c1000000-0000-4000-8000-000000000001',
    'activate_cartridge', 'recovery-drill-sysop',
    'Prove cartridge admission survives restore', NULL,
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 3,
    NULL, 'marketplace_vetted'
);

INSERT INTO operator_custom_authority (
    server_id, operator_name, authority_id, key_id, public_key, key_sha256
) SELECT
    id, 'Recovery Fixture Operator', 'recovery-community', 'custom-primary-v1',
    '{"format_version":1,"algorithm":"ed25519","authority_id":"recovery-community","key_id":"custom-primary-v1","verifying_key":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"}'::jsonb,
    'dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd'
FROM server_identity WHERE singleton;

INSERT INTO operator_custom_releases (
    id, import_operation_id, game_key, publisher_id, publisher_key,
    rules_version, cartridge_version, archive_sha256,
    signed_identity_sha256, display_name, operator_key,
    operator_key_sha256, operator_name, signed_operator_attestation,
    attestation_version, warning, signed_policy, policy_version,
    policy_status, policy_reason, compatible, imported
) VALUES (
    'd0000000-0000-4000-8000-000000000001',
    'd1000000-0000-4000-8000-000000000001',
    'recovery-custom', 'ignibyte', '{"key_id":"publisher-primary-v1"}'::jsonb,
    1, 1,
    'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
    'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff',
    'Recovery Custom',
    '{"authority_id":"recovery-community","key_id":"custom-primary-v1"}'::jsonb,
    'dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd',
    'Recovery Fixture Operator', '{"attestation":"recovery-fixture"}'::jsonb,
    1,
    'Operator-custom content: not reviewed or supported by the OmarchyGS marketplace.',
    '{"policy":"recovery-fixture"}'::jsonb, 2, 'retired',
    'Retained for an active historical session.', TRUE, TRUE
);

INSERT INTO operator_custom_audit_events (
    id, operation_id, release_id, action, actor, reason,
    previous_policy_version, previous_policy_status,
    resulting_policy_version, resulting_policy_status
) VALUES (
    'd2000000-0000-4000-8000-000000000001',
    'd1000000-0000-4000-8000-000000000001',
    'd0000000-0000-4000-8000-000000000001',
    'import_custom_cartridge', 'recovery-drill-sysop',
    'Prove operator-custom provenance survives restore',
    NULL, NULL, 2, 'retired'
);

INSERT INTO server_cartridge_catalogs (
    id, game_key, active_custom_release_id, admission_revision
) VALUES (
    'd3000000-0000-4000-8000-000000000001', 'recovery-custom',
    'd0000000-0000-4000-8000-000000000001', 4
);

INSERT INTO cartridge_catalog_audit_events (
    id, operation_id, catalog_id, action, actor, reason,
    previous_archive_sha256, resulting_archive_sha256,
    admission_revision, previous_provenance_class,
    resulting_provenance_class
) VALUES (
    'd4000000-0000-4000-8000-000000000001',
    'd5000000-0000-4000-8000-000000000001',
    'd3000000-0000-4000-8000-000000000001',
    'activate_cartridge', 'recovery-drill-sysop',
    'Prove custom admission survives restore', NULL,
    'eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee',
    4, NULL, 'operator_custom'
);
SQL

jq -nc \
  --arg account_id "$ogs_subject_account_id" \
  '{command: "set_account_status",
    idempotency_key: "b0000000-0000-4000-8000-000000000001",
    account_id: $account_id,
    status: "suspended",
    actor: "recovery-drill-sysop",
    reason: "Prove transactional containment survives restore"}' \
  >"$ogs_temp/suspend.json"
jq -nc \
  --arg report_id "$ogs_report_id" \
  '{command: "set_report_status",
    idempotency_key: "b1000000-0000-4000-8000-000000000001",
    report_id: $report_id,
    status: "resolved",
    actor: "recovery-drill-sysop",
    reason: "Prove report disposition survives restore"}' \
  >"$ogs_temp/resolve.json"
chmod 600 "$ogs_temp/suspend.json" "$ogs_temp/resolve.json"

DATABASE_URL="$ogs_source_url" "$ogs_admin_binary" apply "$ogs_temp/suspend.json" \
  >"$ogs_temp/suspend-receipt.json"
DATABASE_URL="$ogs_source_url" "$ogs_admin_binary" apply "$ogs_temp/resolve.json" \
  >"$ogs_temp/resolve-receipt.json"
jq -e \
  '.target_kind == "account" and .resulting_state == "suspended" and
   ((keys | sort) == ["action", "created_at", "id", "operation_id", "previous_state", "resulting_state", "target_id", "target_kind"])' \
  "$ogs_temp/suspend-receipt.json" >/dev/null
jq -e \
  '.target_kind == "report" and .resulting_state == "resolved" and
   ((keys | sort) == ["action", "created_at", "id", "operation_id", "previous_state", "resulting_state", "target_id", "target_kind"])' \
  "$ogs_temp/resolve-receipt.json" >/dev/null

assert_scalar "$ogs_source_url" \
  "SELECT status FROM accounts WHERE id = '$ogs_subject_account_id'" \
  suspended "source account status"
assert_scalar "$ogs_source_url" \
  "SELECT count(*) FROM account_sessions WHERE id = '$ogs_session_id' AND revoked_at IS NOT NULL" \
  1 "source revoked session"
assert_scalar "$ogs_source_url" \
  "SELECT status || ':' || (closed_at IS NOT NULL)::text FROM persona_reports WHERE id = '$ogs_report_id'" \
  resolved:true "source report disposition"
assert_scalar "$ogs_source_url" \
  "SELECT count(*) FROM operator_audit_events" 2 "source operator audit count"
assert_scalar "$ogs_source_url" \
  "SELECT snapshot_version || ':' || snapshot_sha256 FROM marketplace_sync_state WHERE singleton" \
  "7:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" \
  "source marketplace snapshot"
assert_scalar "$ogs_source_url" \
  "SELECT r.archive_sha256 || ':' || c.admission_revision FROM server_cartridge_catalogs c JOIN marketplace_releases r ON r.id = c.active_release_id WHERE c.game_key = 'door-legends'" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:3" \
  "source cartridge selection"
assert_scalar "$ogs_source_url" \
  "SELECT count(*) FROM cartridge_catalog_audit_events" 2 \
  "source cartridge catalog audit count"
assert_scalar "$ogs_source_url" \
  "SELECT r.archive_sha256 || ':' || r.policy_status || ':' || c.admission_revision FROM server_cartridge_catalogs c JOIN operator_custom_releases r ON r.id = c.active_custom_release_id WHERE c.game_key = 'recovery-custom'" \
  "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee:retired:4" \
  "source operator-custom selection"
assert_scalar "$ogs_source_url" \
  "SELECT count(*) FROM operator_custom_audit_events" 1 \
  "source operator-custom audit count"

write_table_counts "$ogs_source_url" "$ogs_temp/source-counts.tsv"
pg_dump "$ogs_source_url" --format=custom --file="$ogs_temp/platform.backup"
pg_restore --exit-on-error --no-owner --dbname="$ogs_restore_url" \
  "$ogs_temp/platform.backup"
write_table_counts "$ogs_restore_url" "$ogs_temp/restore-counts.tsv"

if ! cmp --silent "$ogs_temp/source-counts.tsv" "$ogs_temp/restore-counts.tsv"; then
  echo "restored application-table counts differ from the source" >&2
  diff -u "$ogs_temp/source-counts.tsv" "$ogs_temp/restore-counts.tsv" >&2 || true
  exit 1
fi

assert_scalar "$ogs_restore_url" \
  "SELECT status FROM accounts WHERE id = '$ogs_subject_account_id'" \
  suspended "restored account status"
assert_scalar "$ogs_restore_url" \
  "SELECT count(*) FROM account_sessions WHERE id = '$ogs_session_id' AND revoked_at IS NOT NULL" \
  1 "restored revoked session"
assert_scalar "$ogs_restore_url" \
  "SELECT status || ':' || (closed_at IS NOT NULL)::text FROM persona_reports WHERE id = '$ogs_report_id'" \
  resolved:true "restored report disposition"
assert_scalar "$ogs_restore_url" \
  "SELECT count(*) FROM operator_audit_events WHERE target_account_id = '$ogs_subject_account_id' OR target_report_id = '$ogs_report_id'" \
  2 "restored linked operator audit"
assert_scalar "$ogs_restore_url" \
  "SELECT (SELECT count(*) FROM persona_connections) || ':' || (SELECT count(*) FROM inbox_messages) || ':' || (SELECT count(*) FROM game_session_commands) || ':' || (SELECT count(*) FROM persona_sync_events)" \
  1:1:1:4 "restored social inbox game and sync history"
assert_scalar "$ogs_restore_url" \
  "SELECT snapshot_version || ':' || snapshot_sha256 FROM marketplace_sync_state WHERE singleton" \
  "7:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" \
  "restored marketplace snapshot"
assert_scalar "$ogs_restore_url" \
  "SELECT r.archive_sha256 || ':' || c.admission_revision FROM server_cartridge_catalogs c JOIN marketplace_releases r ON r.id = c.active_release_id WHERE c.game_key = 'door-legends'" \
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:3" \
  "restored cartridge selection"
assert_scalar "$ogs_restore_url" \
  "SELECT count(*) FROM cartridge_catalog_audit_events WHERE catalog_id = 'c1000000-0000-4000-8000-000000000001'" \
  1 "restored cartridge catalog audit"
assert_scalar "$ogs_restore_url" \
  "SELECT r.archive_sha256 || ':' || r.policy_status || ':' || c.admission_revision FROM server_cartridge_catalogs c JOIN operator_custom_releases r ON r.id = c.active_custom_release_id WHERE c.game_key = 'recovery-custom'" \
  "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee:retired:4" \
  "restored operator-custom selection"
assert_scalar "$ogs_restore_url" \
  "SELECT count(*) FROM operator_custom_audit_events WHERE release_id = 'd0000000-0000-4000-8000-000000000001'" \
  1 "restored operator-custom audit"

if psql "$ogs_restore_url" -v ON_ERROR_STOP=1 \
  -c "UPDATE operator_audit_events SET reason = 'forbidden'" >/dev/null 2>&1; then
  echo "restored operator audit accepted mutation" >&2
  exit 1
fi
if psql "$ogs_restore_url" -v ON_ERROR_STOP=1 \
  -c "DELETE FROM persona_reports WHERE id = '$ogs_report_id'" >/dev/null 2>&1; then
  echo "restored report accepted deletion" >&2
  exit 1
fi
if psql "$ogs_restore_url" -v ON_ERROR_STOP=1 \
  -c "UPDATE cartridge_catalog_audit_events SET reason = 'forbidden'" >/dev/null 2>&1; then
  echo "restored cartridge catalog audit accepted mutation" >&2
  exit 1
fi
if psql "$ogs_restore_url" -v ON_ERROR_STOP=1 \
  -c "UPDATE operator_custom_releases SET archive_sha256 = repeat('0', 64) WHERE id = 'd0000000-0000-4000-8000-000000000001'" >/dev/null 2>&1; then
  echo "restored operator-custom release accepted identity mutation" >&2
  exit 1
fi
if psql "$ogs_restore_url" -v ON_ERROR_STOP=1 \
  -c "DELETE FROM operator_custom_audit_events WHERE id = 'd2000000-0000-4000-8000-000000000001'" >/dev/null 2>&1; then
  echo "restored operator-custom audit accepted deletion" >&2
  exit 1
fi

ogs_restore_port=$(reserve_port)
start_server "$ogs_restore_url" "$ogs_restore_port" "$ogs_temp/restore-server.log"
ogs_restore_server_id=$(curl --fail --silent \
  "http://127.0.0.1:$ogs_restore_port/.well-known/omarchygs" | jq -er '.server_id')
if [[ "$ogs_restore_server_id" != "$ogs_source_server_id" ]]; then
  echo "restored server identity differs from the source" >&2
  exit 1
fi
ogs_auth_status=$(curl --silent --output "$ogs_temp/restored-auth.json" \
  --write-out '%{http_code}' \
  --header "Authorization: Bearer $ogs_raw_token" \
  "http://127.0.0.1:$ogs_restore_port/v1/personas")
if [[ "$ogs_auth_status" != 401 ]] \
  || ! jq -e '.error.code == "invalid_session"' "$ogs_temp/restored-auth.json" >/dev/null; then
  echo "restored production server accepted a pre-suspension token" >&2
  exit 1
fi
stop_server

echo "platform operator backup and restore drill passed"
