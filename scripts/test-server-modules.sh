#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_host="$ogs_root/target/debug/omarchygs-module-host"

cd "$ogs_root"

cargo fmt --all --check
cargo clippy -p omarchygs-server-module-runtime --all-targets -- -D warnings
cargo test -p omarchygs-server-module-runtime --all-targets
env RUSTDOCFLAGS="-D warnings" cargo doc \
  -p omarchygs-server-module-runtime --no-deps
cargo build -p omarchygs-server-module-runtime \
  --bin omarchygs-module-host

OGS_MODULE_HOST_TEST_BINARY="$ogs_host" \
  cargo test -p omarchygs-server-module-runtime \
    --test conformance \
    real_process_is_contained_and_recovers_after_failure \
    -- --ignored --exact --nocapture

if ! rg -q 'ProcessSupervisor::packaged_sibling\(\)' \
  crates/server/src/server_modules.rs; then
  echo "Production module startup is not bound to the packaged sibling host" >&2
  exit 1
fi

if rg -n \
  'OGS_MODULE_(HOST|COMPONENT|RELEASE|WASM|WIT)_(PATH|URL)|reviewed_path\(' \
  crates/server/src/server_modules.rs crates/server/src/config.rs crates/server/src/main.rs; then
  echo "Production server module accepts an unreviewed executable or interface path" >&2
  exit 1
fi

if rg -n '"/[^" ]*modules?[^" ]*"' crates/server/src/app.rs; then
  echo "A public server-module administration route was added" >&2
  exit 1
fi

if rg -n '(^|[[:space:]])(reqwest|sqlx|hyper|tonic)[[:space:]]*=' \
  crates/server-module-runtime/Cargo.toml; then
  echo "The isolated module host acquired a database or network client dependency" >&2
  exit 1
fi

if rg -n 'fn[[:space:]]+(install|upload|import)_.*module|custom.*module.*(path|bytes)' \
  crates/server/src/server_modules.rs crates/server/src/bin/omarchygs-admin.rs; then
  echo "Custom production module installation appeared before its gated ticket" >&2
  exit 1
fi

./scripts/check-local-only-automation.sh

echo "Production server module conformance passed"
