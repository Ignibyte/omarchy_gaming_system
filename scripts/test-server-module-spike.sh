#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_manifest="$ogs_root/crates/server-module-spike/Cargo.toml"
ogs_target="$ogs_root/crates/server-module-spike/target"
ogs_components_a="$ogs_target/proof-components-a"
ogs_components_b="$ogs_target/proof-components-b"
ogs_bin="$ogs_target/debug"

cd "$ogs_root"

cargo fmt --manifest-path "$ogs_manifest" --all --check
cargo clippy --manifest-path "$ogs_manifest" --all-targets -- -D warnings
cargo test --manifest-path "$ogs_manifest"
cargo build --manifest-path "$ogs_manifest" --bins
env RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path "$ogs_manifest" --no-deps

"$ogs_bin/fixture-builder" "$ogs_components_a"
"$ogs_bin/fixture-builder" "$ogs_components_b"

for ogs_fixture in \
  valid noop unauthorized trap loop memory-hog forbidden-import wrong-interface; do
  cmp "$ogs_components_a/$ogs_fixture.wasm" "$ogs_components_b/$ogs_fixture.wasm"
done

run_scenario() {
  local ogs_scenario="$1"
  local ogs_fixture="$2"
  local ogs_expected="$3"
  local ogs_output

  ogs_output=$("$ogs_bin/supervisor" \
    --scenario "$ogs_scenario" "$ogs_components_a/$ogs_fixture.wasm")
  jq -e \
    --arg scenario "$ogs_scenario" \
    --arg expected "$ogs_expected" \
    '.scenario == $scenario
      and .result == $expected
      and (.containment == "systemd-user-scope+bubblewrap" or .containment == "bubblewrap")
      and .startup_ms < 30000
      and .execution_ms <= 750
      and (.host_rss_kib == null or (.host_rss_kib > 0 and .host_rss_kib <= 262144))' \
    <<<"$ogs_output" >/dev/null

  if [[ "$ogs_expected" != "startup_rejected" ]]; then
    jq -e '.ready.component_ready
      and .ready.home_absent
      and .ready.passwd_absent
      and .ready.server_environment_absent
      and .ready.loopback_only' <<<"$ogs_output" >/dev/null
  fi
}

run_scenario valid valid core_committed_allowlisted_intent
run_scenario noop noop noop
run_scenario unauthorized unauthorized unauthorized_intent_rejected
run_scenario trap trap module_failure_contained
run_scenario loop loop module_failure_contained
run_scenario memory-hog memory-hog startup_rejected
run_scenario forbidden-import forbidden-import startup_rejected
run_scenario wrong-interface wrong-interface startup_rejected
run_scenario tamper valid request_rejected
run_scenario forged-context valid request_rejected
run_scenario host-exit valid host_exit_contained
run_scenario host-hang valid host_timeout_contained

# A fresh exact-release process must recover after the preceding crash/hang.
run_scenario valid valid core_committed_allowlisted_intent

./scripts/check-local-only-automation.sh

ogs_hostile_root=$(mktemp -d)
trap 'rm -rf -- "$ogs_hostile_root"' EXIT
mkdir -p "$ogs_hostile_root/.github/workflows"
touch "$ogs_hostile_root/.github/workflows/ci.yml"
if ./scripts/check-local-only-automation.sh "$ogs_hostile_root"; then
  echo "Hosted automation hostile fixture was accepted" >&2
  exit 1
fi

if rg -n \
  'omarchygs-server-module-spike|ModuleRuntime|module-host|server_module' \
  crates/server client migrations Cargo.toml compose.yaml \
  --glob '!server-module-spike/**'; then
  echo "Production module loader/configuration unexpectedly exists" >&2
  exit 1
fi

echo "Server module isolation architecture proof passed"
