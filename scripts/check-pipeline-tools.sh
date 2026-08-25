#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_prefix="$ogs_root/.dev/pipeline-tools"
ogs_pnpm_prefix="$ogs_prefix/pnpm"
ogs_codegraph_version="1.5.0"
ogs_codegraph_integrity_hex="fe5d49315390f561902eb73f208b8083b220afde38b7bf7fa0d0894dc9c6918b48790c765c522a234868fbd2deb3f62ee3329f98f535ee1214b210fac8fd5f2b"
ogs_openwiki_commit="a525ed88fe1f189d08e0f0acf12f42caec2b600e"
ogs_pnpm_version="10.33.2"
ogs_pnpm_integrity_hex="a90faf6feeab71ad6c6e57f94e0fe1a12f5dcc22cd754db40ae9593eb6a3e0b6b12e3540218bb37ae083404b1f2ce6db2a4121e979829b4aff94b99f49da1cf8"
ogs_pnpm_package_manager="pnpm@$ogs_pnpm_version+sha512.$ogs_pnpm_integrity_hex"
ogs_codegraph_package="$ogs_prefix/codegraph/node_modules/@colbymchenry/codegraph/package.json"
ogs_codegraph_provenance="$ogs_prefix/codegraph-install.provenance"
ogs_pnpm_package="$ogs_pnpm_prefix/node_modules/pnpm/package.json"
ogs_openwiki_source="$ogs_prefix/openwiki"
ogs_openwiki_package="$ogs_openwiki_source/package.json"
ogs_openwiki_provenance="$ogs_prefix/openwiki-build.provenance"
ogs_openwiki_mode="$ogs_openwiki_source/src/ingestion/code-mode.ts"
ogs_openwiki_session="$ogs_openwiki_source/src/integrations/core/session-manager.ts"
ogs_expected_changes=$'src/ingestion/code-mode.ts\nsrc/integrations/core/session-manager.ts'

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    ogs_codegraph_platform="linux-x64"
    ogs_codegraph_platform_integrity_hex="c13d4eecd910a15eb33e2ab7e9dd6a06aa081cbd994673a414c8d0ec9270592eca1e73f7daa1c277fa4f692c25f1c351946624612697dd6543a73d81695f8da3"
    ogs_codegraph_expected_tree_sha256="0c41ea51125f3838779ddd9b7f2455d77974bd08a8865a94099bcdc7d5440584"
    ;;
  Linux-aarch64 | Linux-arm64)
    ogs_codegraph_platform="linux-arm64"
    ogs_codegraph_platform_integrity_hex="06ae4681becfa0b17b9a73a21a5fd4c4cac382b3c201050cb25ec1a4c900c6af05ba5bd5129ec4fc6ac8422876601df6bd6af5f0aa4dda2d95a112c01cb20353"
    ogs_codegraph_expected_tree_sha256="eda3f714c9a44e5b91bae1b57633b191e79f980e71c6f92e89479d5892bfd024"
    ;;
  *)
    echo "Pipeline readiness has no reviewed CodeGraph artifact for this platform." >&2
    exit 1
    ;;
esac
ogs_codegraph_package_pin="@colbymchenry/codegraph@$ogs_codegraph_version"
ogs_codegraph_platform_package_pin="@colbymchenry/codegraph-$ogs_codegraph_platform@$ogs_codegraph_version"
ogs_codegraph_platform_package="$ogs_prefix/codegraph/node_modules/@colbymchenry/codegraph-$ogs_codegraph_platform/package.json"

ogs_tree_digest() {
  local ogs_tree="$1"

  find "$ogs_tree" -type f -print0 \
    | LC_ALL=C sort -z \
    | xargs -0 -r sha256sum \
    | sha256sum \
    | awk '{print $1}'
}

ogs_relative_tree_digest() {
  local ogs_tree="$1"

  (
    cd "$ogs_tree"
    find . -type f -print0 \
      | LC_ALL=C sort -z \
      | xargs -0 -r sha256sum --zero -- \
      | sha256sum \
      | awk '{print $1}'
  )
}

ogs_provenance_value() {
  local ogs_key="$1"

  sed -n "s/^${ogs_key}=//p" "$ogs_openwiki_provenance" | head -1
}

ogs_codegraph_provenance_value() {
  local ogs_key="$1"

  sed -n "s/^${ogs_key}=//p" "$ogs_codegraph_provenance" | head -1
}

[[ -f "$ogs_codegraph_package" && -f "$ogs_codegraph_platform_package" \
  && -f "$ogs_codegraph_provenance" && -f "$ogs_pnpm_package" \
  && -f "$ogs_openwiki_package" && -f "$ogs_openwiki_provenance" ]] || {
  echo "Pipeline tools are not installed. Run scripts/setup-pipeline-tools.sh." >&2
  exit 1
}

ogs_codegraph_version=$(node -p "require(process.argv[1]).version" "$ogs_codegraph_package")
ogs_installed_pnpm_version=$(node -p "require(process.argv[1]).version" "$ogs_pnpm_package")
ogs_openwiki_version=$(node -p "require(process.argv[1]).version" "$ogs_openwiki_package")
[[ "$ogs_codegraph_version" == "1.5.0" ]] || {
  echo "Expected CodeGraph 1.5.0, found $ogs_codegraph_version." >&2
  exit 1
}
ogs_installed_codegraph_platform_version=$(
  node -p "require(process.argv[1]).version" "$ogs_codegraph_platform_package"
)
[[ "$ogs_installed_codegraph_platform_version" == "$ogs_codegraph_version" ]] || {
  echo "Expected CodeGraph platform $ogs_codegraph_version, found $ogs_installed_codegraph_platform_version." >&2
  exit 1
}
ogs_codegraph_installed_packages=$(
  find "$ogs_prefix/codegraph/node_modules/@colbymchenry" \
    -mindepth 1 -maxdepth 1 -type d -printf '%f\n' \
    | LC_ALL=C sort
)
ogs_codegraph_expected_packages=$(
  printf '%s\n' codegraph "codegraph-$ogs_codegraph_platform" | LC_ALL=C sort
)
ogs_codegraph_tree_sha256=$(
  ogs_relative_tree_digest "$ogs_prefix/codegraph/node_modules/@colbymchenry"
)
[[ "$ogs_codegraph_installed_packages" == "$ogs_codegraph_expected_packages" \
  && "$ogs_codegraph_tree_sha256" == "$ogs_codegraph_expected_tree_sha256" \
  && "$(ogs_codegraph_provenance_value version)" == "1" \
  && "$(ogs_codegraph_provenance_value package)" == "$ogs_codegraph_package_pin" \
  && "$(ogs_codegraph_provenance_value package_sha512)" == "$ogs_codegraph_integrity_hex" \
  && "$(ogs_codegraph_provenance_value platform_package)" == "$ogs_codegraph_platform_package_pin" \
  && "$(ogs_codegraph_provenance_value platform_sha512)" == "$ogs_codegraph_platform_integrity_hex" \
  && "$(ogs_codegraph_provenance_value tree_sha256)" == "$ogs_codegraph_tree_sha256" \
  && -L "$ogs_prefix/codegraph/node_modules/.bin/codegraph" \
  && "$(readlink "$ogs_prefix/codegraph/node_modules/.bin/codegraph")" \
    == "../@colbymchenry/codegraph/npm-shim.js" ]] || {
  echo "CodeGraph install or integrity provenance is stale; rerun scripts/setup-pipeline-tools.sh." >&2
  exit 1
}
[[ "$ogs_openwiki_version" == "0.3.3" ]] || {
  echo "Expected OpenWiki 0.3.3, found $ogs_openwiki_version." >&2
  exit 1
}
[[ "$ogs_installed_pnpm_version" == "$ogs_pnpm_version" ]] || {
  echo "Expected pnpm $ogs_pnpm_version, found $ogs_installed_pnpm_version." >&2
  exit 1
}
[[ "$(node -p 'require(process.argv[1]).packageManager' "$ogs_openwiki_package")" \
  == "$ogs_pnpm_package_manager" ]] || {
  echo "OpenWiki package-manager integrity no longer matches the reviewed pin." >&2
  exit 1
}
[[ "$(git -C "$ogs_openwiki_source" rev-parse HEAD)" == "$ogs_openwiki_commit" ]] || {
  echo "OpenWiki checkout is not at pinned commit $ogs_openwiki_commit." >&2
  exit 1
}
[[ -f "$ogs_openwiki_source/dist/integrations/core/session-manager.js" ]] || {
  echo "OpenWiki Codex lifecycle build is missing." >&2
  exit 1
}
[[ -f "$ogs_openwiki_source/node_modules/.modules.yaml" \
  && ! -e "$ogs_openwiki_source/package-lock.json" ]] || {
  echo "OpenWiki dependencies were not installed from the frozen pnpm lock." >&2
  exit 1
}

grep -Fxq 'const CODE_MODE_AGENT_FILES = ["AGENTS.md"];' "$ogs_openwiki_mode" || {
  echo "OpenWiki Codex-only agent-guide patch is missing." >&2
  exit 1
}
grep -Fq 'createWorkflow: false' "$ogs_openwiki_session" || {
  echo "OpenWiki Codex-only scheduled-workflow patch is missing." >&2
  exit 1
}
[[ "$(git -C "$ogs_openwiki_source" diff --name-only | LC_ALL=C sort)" == "$ogs_expected_changes" ]] || {
  echo "OpenWiki generated checkout has unexpected tracked changes." >&2
  exit 1
}

[[ "$(ogs_provenance_value version)" == "1" \
  && "$(ogs_provenance_value commit)" == "$ogs_openwiki_commit" \
  && "$(ogs_provenance_value package_manager)" == "$ogs_pnpm_package_manager" \
  && "$(ogs_provenance_value pnpm_tree_sha256)" == "$(ogs_tree_digest "$ogs_pnpm_prefix/node_modules/pnpm")" \
  && "$(ogs_provenance_value lock_sha256)" == "$(sha256sum "$ogs_openwiki_source/pnpm-lock.yaml" | awk '{print $1}')" \
  && "$(ogs_provenance_value patch_sha256)" == "$(git -C "$ogs_openwiki_source" diff --binary | sha256sum | awk '{print $1}')" \
  && "$(ogs_provenance_value dist_sha256)" == "$(ogs_tree_digest "$ogs_openwiki_source/dist")" ]] || {
  echo "OpenWiki install or build provenance is stale; rerun scripts/setup-pipeline-tools.sh." >&2
  exit 1
}

cd "$ogs_root"
ogs_status=$(CODEGRAPH_TELEMETRY=0 DO_NOT_TRACK=1 \
  "$ogs_prefix/codegraph/node_modules/.bin/codegraph" status --json)
jq -e '
  .initialized == true and
  .version == "1.5.0" and
  .index.state == "complete"
' <<<"$ogs_status" >/dev/null

echo "Pipeline tools ready: CodeGraph $ogs_codegraph_version; OpenWiki $ogs_openwiki_version via verified pnpm $ogs_installed_pnpm_version; Codex-only patch and build provenance active."
