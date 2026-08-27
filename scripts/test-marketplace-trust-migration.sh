#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
ogs_schema="ogs_t36_upgrade_${BASHPID}_${RANDOM}"
[[ "$ogs_schema" =~ ^ogs_t36_upgrade_[0-9]+_[0-9]+$ ]]

cleanup_upgrade_schema() {
  PGOPTIONS="-c client_min_messages=warning" \
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 \
    -c "DROP SCHEMA IF EXISTS $ogs_schema CASCADE" >/dev/null
}
trap cleanup_upgrade_schema EXIT INT TERM

psql "$DATABASE_URL" -v ON_ERROR_STOP=1 \
  -c "CREATE SCHEMA $ogs_schema" >/dev/null

while IFS= read -r ogs_migration; do
  [[ "$(basename "$ogs_migration")" == 0023_* ]] && break
  PGOPTIONS="-c search_path=$ogs_schema" \
    psql "$DATABASE_URL" -1 -v ON_ERROR_STOP=1 \
      -f "$ogs_migration" >/dev/null
done < <(find "$ogs_root/migrations" -maxdepth 1 -type f -name '00*.sql' | LC_ALL=C sort)

PGOPTIONS="-c search_path=$ogs_schema" \
  psql "$DATABASE_URL" -1 -v ON_ERROR_STOP=1 >/dev/null <<'SQL'
INSERT INTO marketplace_sync_state (
    marketplace_origin, authority_id, key_id, marketplace_name,
    snapshot_version, snapshot_sha256, signed_snapshot, marketplace_key
) VALUES (
    'https://market.example.test/v1/', 'official-marketplace', 'catalog-1',
    'Official Marketplace', 2, repeat('2', 64), decode('02', 'hex'),
    '{"authority_id":"official-marketplace","key_id":"catalog-1"}'::jsonb
);

INSERT INTO marketplace_releases (
    game_key, publisher_id, publisher_key, rules_version, cartridge_version,
    archive_sha256, signed_identity_sha256, display_name, release_path,
    reviewed_by, review_summary, signed_policy, policy_version, policy_status,
    policy_reason, compatible, imported, first_seen_snapshot_version,
    last_seen_snapshot_version
) VALUES (
    'retained-game', 'publisher', '{}'::jsonb, 1, 1,
    repeat('a', 64), repeat('b', 64), 'Retained Game',
    'releases/retained-game/1/', 'reviewer', 'Reviewed.', '{}'::jsonb,
    1, 'active', 'Was present.', TRUE, TRUE, 1, 1
);
SQL

PGOPTIONS="-c search_path=$ogs_schema" \
  psql "$DATABASE_URL" -1 -v ON_ERROR_STOP=1 \
    -f "$ogs_root/migrations/0023_marketplace_trust_key_rotation.sql" >/dev/null

ogs_versions="$(
  PGOPTIONS="-c search_path=$ogs_schema" \
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -Atc \
      "SELECT policy_snapshot_version || ':' || last_seen_snapshot_version FROM marketplace_releases WHERE game_key = 'retained-game'"
)"
[[ "$ogs_versions" == "1:1" ]] || {
  echo "Marketplace trust migration did not preserve historical policy provenance." >&2
  exit 1
}

echo "marketplace trust historical upgrade migration passed"
