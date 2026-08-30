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
  crates/server/src/server_modules.rs \
  || ! rg -q 'ProcessSupervisor::packaged_sibling\(\)' \
    crates/server/src/server_module_custom.rs; then
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

for ogs_required_custom_boundary in \
  'custom-module-import' \
  'custom-module-apply' \
  'OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC' \
  'I understand this module is unreviewed and unsupported by OmarchyGS.'; do
  if ! rg -F -q "$ogs_required_custom_boundary" \
    crates/server/src/server_module_custom.rs \
    crates/server/src/bin/omarchygs-admin.rs; then
    echo "Production custom-module boundary is missing: $ogs_required_custom_boundary" >&2
    exit 1
  fi
done

for ogs_required_custom_schema in \
  "artifact_custody = 'database_immutable'" \
  "provenance_class = 'operator_custom'" \
  'server_module_custom_operations_immutable_rows'; do
  if ! rg -F -q "$ogs_required_custom_schema" \
    migrations/0027_operator_custom_server_modules.sql; then
    echo "Production custom-module custody is missing: $ogs_required_custom_schema" >&2
    exit 1
  fi
done

if rg -n 'custom-module-(import|apply)|server_module_custom' crates/server/src/app.rs; then
  echo "Custom module administration escaped the database-local CLI" >&2
  exit 1
fi

./scripts/check-local-only-automation.sh

echo "Production server module conformance passed"
