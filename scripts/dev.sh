#!/usr/bin/env bash
set -Eeuo pipefail

bbs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bbs_log_dir="$bbs_root/.dev"
bbs_server_pid=""
bbs_qml_arguments=()

case "${1:-}" in
  "") ;;
  --smoke-test)
    export QT_QPA_PLATFORM="${QT_QPA_PLATFORM:-offscreen}"
    bbs_qml_arguments=(-- --smoke-test)
    ;;
  *)
    echo "Usage: $0 [--smoke-test]" >&2
    exit 2
    ;;
esac

cleanup() {
  if [[ -n "$bbs_server_pid" ]] && kill -0 "$bbs_server_pid" 2>/dev/null; then
    kill "$bbs_server_pid"
    wait "$bbs_server_pid" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

for command_name in docker mise qml6 curl; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

cd "$bbs_root"
mkdir -p "$bbs_log_dir"

mise install
docker compose up -d --wait db

export DATABASE_URL="${DATABASE_URL:-postgres://omarchy_bbs:omarchy_bbs@127.0.0.1:5432/omarchy_bbs}"
export BBS_BIND_ADDRESS="${BBS_BIND_ADDRESS:-127.0.0.1:8080}"
export RUST_LOG="${RUST_LOG:-omarchy_bbs_server=debug,tower_http=debug}"

mise exec -- cargo run -p omarchy-bbs-server >"$bbs_log_dir/server.log" 2>&1 &
bbs_server_pid=$!

for _ in {1..90}; do
  if curl --fail --silent "http://$BBS_BIND_ADDRESS/health" >/dev/null; then
    break
  fi

  if ! kill -0 "$bbs_server_pid" 2>/dev/null; then
    echo "The server stopped during startup:" >&2
    tail -80 "$bbs_log_dir/server.log" >&2
    exit 1
  fi

  sleep 1
done

if ! curl --fail --silent "http://$BBS_BIND_ADDRESS/health" >/dev/null; then
  echo "The server did not become healthy. See $bbs_log_dir/server.log" >&2
  exit 1
fi

echo "Server ready at http://$BBS_BIND_ADDRESS"
echo "Closing the QML window will stop the Rust server; PostgreSQL stays running."
qml6 "$bbs_root/client/qml/Main.qml" "${bbs_qml_arguments[@]}"
