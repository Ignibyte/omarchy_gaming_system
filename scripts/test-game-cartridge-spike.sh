#!/usr/bin/env bash
set -euo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_manifest="$ogs_root/crates/game-cartridge-spike/Cargo.toml"
ogs_target="$ogs_root/target/game-cartridge-spike"
ogs_temp="$(mktemp -d)"
ogs_provider_pid=""
ogs_broker_pid=""
ogs_qml_pid=""

cleanup() {
  if [[ -n "$ogs_qml_pid" ]]; then
    kill "$ogs_qml_pid" 2>/dev/null || true
    wait "$ogs_qml_pid" 2>/dev/null || true
  fi
  if [[ -n "$ogs_broker_pid" ]]; then
    kill "$ogs_broker_pid" 2>/dev/null || true
    wait "$ogs_broker_pid" 2>/dev/null || true
  fi
  if [[ -n "$ogs_provider_pid" ]]; then
    kill "$ogs_provider_pid" 2>/dev/null || true
    wait "$ogs_provider_pid" 2>/dev/null || true
  fi
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in cargo curl du ps python3 qml6 rg; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

export CARGO_TARGET_DIR="$ogs_target"
cargo fmt --manifest-path "$ogs_manifest" --all -- --check
cargo clippy --manifest-path "$ogs_manifest" --all-targets -- -D warnings
cargo test --manifest-path "$ogs_manifest" --all-targets
cargo build --manifest-path "$ogs_manifest" --bins
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path "$ogs_manifest" --no-deps

ogs_bin="$ogs_target/debug"
ogs_cartridge="$ogs_temp/cartridge"
cp -R -- "$ogs_root/crates/game-cartridge-spike/fixtures/cartridge" "$ogs_cartridge"

"$ogs_bin/cartridge-tool" keygen "$ogs_temp/platform.private" "$ogs_temp/platform.public"
"$ogs_bin/cartridge-tool" keygen "$ogs_temp/provider.private" "$ogs_temp/provider.public"
"$ogs_bin/cartridge-tool" keygen "$ogs_temp/publisher.private" "$ogs_temp/publisher.public"
ogs_digest="$("$ogs_bin/cartridge-tool" sign "$ogs_cartridge" "$ogs_temp/publisher.private")"
ogs_verified_digest="$("$ogs_bin/cartridge-tool" verify "$ogs_cartridge" "$ogs_temp/publisher.public")"
[[ "$ogs_digest" == "$ogs_verified_digest" ]] || {
  echo "signed cartridge digest changed during verification" >&2
  exit 1
}

free_port() {
  python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()'
}

wait_for_health() {
  local ogs_url="$1"
  local ogs_log="$2"
  for _ in $(seq 1 60); do
    if curl --fail --silent --show-error --max-time 1 "$ogs_url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.1
  done
  echo "service did not become healthy: $ogs_url" >&2
  sed -n '1,160p' "$ogs_log" >&2
  return 1
}

ogs_provider_port="$(free_port)"
ogs_broker_port="$(free_port)"
while [[ "$ogs_broker_port" == "$ogs_provider_port" ]]; do
  ogs_broker_port="$(free_port)"
done

env \
  OGS_SPIKE_PROVIDER_BIND="127.0.0.1:$ogs_provider_port" \
  OGS_SPIKE_PLATFORM_PUBLIC_KEY="$ogs_temp/platform.public" \
  OGS_SPIKE_PROVIDER_PRIVATE_KEY="$ogs_temp/provider.private" \
  OGS_SPIKE_PROVIDER_ID="fixture-provider" \
  OGS_SPIKE_GAME_KEY="retro-grid" \
  OGS_SPIKE_GAME_VERSION="1" \
  OGS_SPIKE_CARTRIDGE_DIGEST="$ogs_digest" \
  "$ogs_bin/provider" >"$ogs_temp/provider.log" 2>&1 &
ogs_provider_pid=$!
wait_for_health "http://127.0.0.1:$ogs_provider_port/health" "$ogs_temp/provider.log"

env \
  OGS_SPIKE_BROKER_BIND="127.0.0.1:$ogs_broker_port" \
  OGS_SPIKE_PROVIDER_URL="http://127.0.0.1:$ogs_provider_port/" \
  OGS_SPIKE_PUBLISHER_ID="ignibyte" \
  OGS_SPIKE_PROVIDER_ID="fixture-provider" \
  OGS_SPIKE_PLATFORM_PRIVATE_KEY="$ogs_temp/platform.private" \
  OGS_SPIKE_PROVIDER_PUBLIC_KEY="$ogs_temp/provider.public" \
  OGS_SPIKE_PUBLISHER_PUBLIC_KEY="$ogs_temp/publisher.public" \
  OGS_SPIKE_CARTRIDGE_DIR="$ogs_cartridge" \
  OGS_SPIKE_PAIRWISE_SECRET="temporary-proof-secret-is-at-least-thirty-two-bytes" \
  "$ogs_bin/broker" >"$ogs_temp/broker.log" 2>&1 &
ogs_broker_pid=$!
wait_for_health "http://127.0.0.1:$ogs_broker_port/health" "$ogs_temp/broker.log"

"$ogs_bin/probe" "http://127.0.0.1:$ogs_broker_port/v1/proof" >"$ogs_temp/probe.json"
rg --fixed-strings '"raw_persona_disclosed": false' "$ogs_temp/probe.json" >/dev/null
rg --fixed-strings '"device_token_disclosed": false' "$ogs_temp/probe.json" >/dev/null
rg --fixed-strings '"database_access_disclosed": false' "$ogs_temp/probe.json" >/dev/null

QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software \
  QT_FORCE_STDERR_LOGGING=1 QT_LOGGING_RULES='qml=true;*.warning=true' \
  qml6 "$ogs_root/crates/game-cartridge-spike/qml/CartridgeProof.qml" -- \
  --smoke-test --broker-url="http://127.0.0.1:$ogs_broker_port" \
  >"$ogs_temp/qml.log" 2>&1 &
ogs_qml_pid=$!
ogs_qml_peak_rss=0
ogs_qml_timed_out=true
for _ in $(seq 1 300); do
  ogs_qml_state="$(ps -o stat= -p "$ogs_qml_pid" 2>/dev/null | tr -d '[:space:]' || true)"
  if [[ -z "$ogs_qml_state" || "$ogs_qml_state" == Z* ]]; then
    ogs_qml_timed_out=false
    break
  fi
  ogs_qml_rss="$(ps -o rss= -p "$ogs_qml_pid" 2>/dev/null | tr -d '[:space:]' || true)"
  if [[ "$ogs_qml_rss" =~ ^[0-9]+$ ]] && (( ogs_qml_rss > ogs_qml_peak_rss )); then
    ogs_qml_peak_rss=$ogs_qml_rss
  fi
  sleep 0.05
done
if [[ "$ogs_qml_timed_out" == true ]]; then
  kill "$ogs_qml_pid" 2>/dev/null || true
fi
if ! wait "$ogs_qml_pid"; then
  echo "trusted QML proof failed" >&2
  sed -n '1,200p' "$ogs_temp/qml.log" >&2
  sed -n '1,200p' "$ogs_temp/broker.log" >&2
  sed -n '1,200p' "$ogs_temp/provider.log" >&2
  exit 1
fi
ogs_qml_pid=""
echo "OGS_CARTRIDGE_MEMORY_METRICS peak_rss_kib=$ogs_qml_peak_rss" >>"$ogs_temp/qml.log"
if rg 'ReferenceError|TypeError|Required property|Unable to assign|is not a type' \
  "$ogs_temp/qml.log" >/dev/null; then
  echo "trusted QML proof emitted a runtime contract error" >&2
  sed -n '1,200p' "$ogs_temp/qml.log" >&2
  exit 1
fi
ogs_metrics="$(rg --only-matching 'OGS_CARTRIDGE_FRAME_METRICS frames=120 average_ms=[0-9.]+ max_ms=[0-9.]+' "$ogs_temp/qml.log")"
ogs_memory="$(rg --only-matching 'OGS_CARTRIDGE_MEMORY_METRICS peak_rss_kib=[0-9]+' "$ogs_temp/qml.log")"
[[ -n "$ogs_metrics" ]] || {
  echo "trusted QML proof did not emit frame metrics" >&2
  sed -n '1,200p' "$ogs_temp/qml.log" >&2
  exit 1
}

echo "game cartridge proof passed"
echo "$ogs_metrics"
echo "$ogs_memory"
echo "OGS_CARTRIDGE_PACKAGE_METRICS files=$(find "$ogs_cartridge" -type f | wc -l) expanded_bytes=$(du -sb "$ogs_cartridge" | cut -f1)"
