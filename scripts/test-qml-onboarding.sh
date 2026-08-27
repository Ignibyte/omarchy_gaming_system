#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_temp_dir="$(mktemp -d)"
ogs_config_dir="$ogs_root/.dev/qml-onboarding"
ogs_fixture_config="$ogs_config_dir/fixture-config.json"
declare -a ogs_fixture_pids=()
declare -A ogs_fixture_urls=()
declare -a ogs_fixture_logs=()

cleanup() {
  local ogs_status=$?
  local ogs_pid
  local ogs_log

  for ogs_pid in "${ogs_fixture_pids[@]}"; do
    if kill -0 "$ogs_pid" 2>/dev/null; then
      kill "$ogs_pid" 2>/dev/null || true
    fi
    wait "$ogs_pid" 2>/dev/null || true
  done
  if ((ogs_status != 0)); then
    for ogs_log in "${ogs_fixture_logs[@]}"; do
      if [[ -s "$ogs_log" ]]; then
        echo "QML fixture output from $ogs_log:" >&2
        sed -n '1,240p' "$ogs_log" >&2
      fi
    done
  fi
  rm -f -- "$ogs_fixture_config"
  rm -rf -- "$ogs_temp_dir"
}

trap cleanup EXIT INT TERM

for ogs_command in python3 curl jq qmake6 flock; do
  if ! command -v "$ogs_command" >/dev/null 2>&1; then
    echo "Missing required QML onboarding test command: $ogs_command" >&2
    exit 1
  fi
done

ogs_qt_bins=$(qmake6 -query QT_INSTALL_BINS)
ogs_qml_test_runner="$ogs_qt_bins/qmltestrunner"
if [[ ! -x "$ogs_qml_test_runner" ]]; then
  echo "Qt Quick Test runner not found at $ogs_qml_test_runner" >&2
  exit 1
fi

start_fixture() {
  local ogs_name="$1"
  local ogs_mode="$2"
  local ogs_port_file="$ogs_temp_dir/$ogs_name.port"
  local ogs_log_file="$ogs_temp_dir/$ogs_name.log"
  local ogs_pid

  python3 "$ogs_root/client/qml/tests/fixture_server.py" \
    "$ogs_port_file" "$ogs_mode" >"$ogs_log_file" 2>&1 &
  ogs_pid=$!
  ogs_fixture_pids+=("$ogs_pid")
  ogs_fixture_logs+=("$ogs_log_file")

  for _ in {1..100}; do
    if [[ -s "$ogs_port_file" ]]; then
      break
    fi
    if ! kill -0 "$ogs_pid" 2>/dev/null; then
      echo "QML fixture $ogs_name stopped during startup" >&2
      return 1
    fi
    sleep 0.05
  done
  if [[ ! -s "$ogs_port_file" ]]; then
    echo "QML fixture $ogs_name did not publish a port" >&2
    return 1
  fi

  ogs_fixture_urls["$ogs_name"]="http://127.0.0.1:$(<"$ogs_port_file")"
}

cd "$ogs_root"
python3 "$ogs_root/scripts/check-qml-style.py"
mkdir -p "$ogs_config_dir"
chmod 0700 "$ogs_config_dir"
exec 9>"$ogs_config_dir/fixture.lock"
flock 9
start_fixture normal normal
start_fixture server_two server_two
start_fixture catalog_only catalog_only
start_fixture identity_changed identity_changed
start_fixture incompatible incompatible
start_fixture malformed malformed
start_fixture wrong_identity wrong_identity
start_fixture slow slow
start_fixture oversized oversized

export QT_QPA_PLATFORM=offscreen
export QT_QUICK_BACKEND=software
export QML_XHR_ALLOW_FILE_READ=1
export XDG_CONFIG_HOME="$ogs_temp_dir/xdg-config"
mkdir -p "$XDG_CONFIG_HOME"
chmod 0700 "$XDG_CONFIG_HOME"

jq -nc \
  --arg server_url "${ogs_fixture_urls[normal]}" \
  --arg server_two_url "${ogs_fixture_urls[server_two]}" \
  --arg catalog_only_url "${ogs_fixture_urls[catalog_only]}" \
  --arg identity_changed_url "${ogs_fixture_urls[identity_changed]}" \
  --arg incompatible_url "${ogs_fixture_urls[incompatible]}" \
  --arg malformed_url "${ogs_fixture_urls[malformed]}" \
  --arg wrong_identity_url "${ogs_fixture_urls[wrong_identity]}" \
  --arg slow_url "${ogs_fixture_urls[slow]}" \
  --arg oversized_url "${ogs_fixture_urls[oversized]}" \
  '{server_url: $server_url, server_two_url: $server_two_url,
    catalog_only_url: $catalog_only_url,
    identity_changed_url: $identity_changed_url, incompatible_url: $incompatible_url,
    malformed_url: $malformed_url,
    wrong_identity_url: $wrong_identity_url, slow_url: $slow_url,
    oversized_url: $oversized_url}' \
  | python3 "$ogs_root/client/qml/tests/fixture_server.py" \
      --write-config "$ogs_fixture_config"

"$ogs_qml_test_runner" \
  -input "$ogs_root/client/qml/tests/profiles-write" \
  -import "$ogs_root/client/qml" \
  -eventdelay 0 \
  -keydelay 0

"$ogs_qml_test_runner" \
  -input "$ogs_root/client/qml/tests/profiles-read" \
  -import "$ogs_root/client/qml" \
  -eventdelay 0 \
  -keydelay 0

"$ogs_qml_test_runner" \
  -input "$ogs_root/client/qml/tests/fixture" \
  -import "$ogs_root/client/qml" \
  -eventdelay 0 \
  -keydelay 0

for ogs_name in normal server_two catalog_only identity_changed incompatible malformed wrong_identity slow oversized; do
  if ! curl --fail --silent "${ogs_fixture_urls[$ogs_name]}/__fixture__/status" \
    | jq -e '.violations == []' >/dev/null; then
    echo "QML fixture $ogs_name observed a request-contract violation" >&2
    curl --fail --silent "${ogs_fixture_urls[$ogs_name]}/__fixture__/status" >&2 || true
    exit 1
  fi
done

echo "keyboard-first QML onboarding fixture passed"
