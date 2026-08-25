#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_prefix="$ogs_root/.dev/pipeline-tools"
ogs_codegraph_prefix="$ogs_prefix/codegraph"
ogs_pnpm_prefix="$ogs_prefix/pnpm"
ogs_openwiki_source="$ogs_prefix/openwiki"
ogs_openwiki_provenance="$ogs_prefix/openwiki-build.provenance"
ogs_codegraph_provenance="$ogs_prefix/codegraph-install.provenance"
ogs_codegraph_version="1.5.0"
ogs_codegraph_integrity_hex="fe5d49315390f561902eb73f208b8083b220afde38b7bf7fa0d0894dc9c6918b48790c765c522a234868fbd2deb3f62ee3329f98f535ee1214b210fac8fd5f2b"
ogs_openwiki_version="0.3.3"
ogs_openwiki_commit="a525ed88fe1f189d08e0f0acf12f42caec2b600e"
ogs_pnpm_version="10.33.2"
ogs_pnpm_integrity_hex="a90faf6feeab71ad6c6e57f94e0fe1a12f5dcc22cd754db40ae9593eb6a3e0b6b12e3540218bb37ae083404b1f2ce6db2a4121e979829b4aff94b99f49da1cf8"
ogs_pnpm_package_manager="pnpm@$ogs_pnpm_version+sha512.$ogs_pnpm_integrity_hex"
ogs_openwiki_mode="$ogs_openwiki_source/src/ingestion/code-mode.ts"
ogs_openwiki_session="$ogs_openwiki_source/src/integrations/core/session-manager.ts"
ogs_bootstrap_dir=""
ogs_secondary_guide=$(printf '%s%s.md' 'CLAU' 'DE')
ogs_agent_files_original="const CODE_MODE_AGENT_FILES = [\"AGENTS.md\", \"$ogs_secondary_guide\"];"
ogs_agent_files_codex='const CODE_MODE_AGENT_FILES = ["AGENTS.md"];'

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
    echo "Pipeline tool setup has no reviewed CodeGraph artifact for this platform." >&2
    exit 1
    ;;
esac
ogs_codegraph_package="@colbymchenry/codegraph@$ogs_codegraph_version"
ogs_codegraph_platform_package="@colbymchenry/codegraph-$ogs_codegraph_platform@$ogs_codegraph_version"

cleanup() {
  [[ -n "$ogs_bootstrap_dir" && "$ogs_bootstrap_dir" == /tmp/* ]] \
    && rm -rf -- "$ogs_bootstrap_dir"
}
trap cleanup EXIT

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

command -v node >/dev/null 2>&1 || {
  echo "Pipeline tool setup requires Node.js 22 or newer." >&2
  exit 1
}
command -v npm >/dev/null 2>&1 || {
  echo "Pipeline tool setup requires npm to fetch the integrity-pinned pnpm bootstrap." >&2
  exit 1
}
command -v sha256sum >/dev/null 2>&1 \
  && command -v sha512sum >/dev/null 2>&1 || {
  echo "Pipeline tool setup requires sha256sum and sha512sum." >&2
  exit 1
}

ogs_node_major=$(node -p 'Number(process.versions.node.split(".")[0])')
((ogs_node_major >= 22)) || {
  echo "Pipeline tool setup requires Node.js 22 or newer." >&2
  exit 1
}

ogs_bootstrap_dir=$(mktemp -d)
ogs_codegraph_tarball_name=$(
  DO_NOT_TRACK=1 npm pack \
    --silent \
    --pack-destination "$ogs_bootstrap_dir" \
    "$ogs_codegraph_package" \
    | tail -1
)
ogs_codegraph_platform_tarball_name=$(
  DO_NOT_TRACK=1 npm pack \
    --silent \
    --pack-destination "$ogs_bootstrap_dir" \
    "$ogs_codegraph_platform_package" \
    | tail -1
)
for ogs_tarball_name in \
  "$ogs_codegraph_tarball_name" \
  "$ogs_codegraph_platform_tarball_name"; do
  [[ -n "$ogs_tarball_name" && "$ogs_tarball_name" != */* \
    && "$ogs_tarball_name" != *$'\n'* \
    && -f "$ogs_bootstrap_dir/$ogs_tarball_name" ]] || {
    echo "A reviewed CodeGraph tarball was not downloaded safely." >&2
    exit 1
  }
done
ogs_codegraph_tarball="$ogs_bootstrap_dir/$ogs_codegraph_tarball_name"
ogs_codegraph_platform_tarball="$ogs_bootstrap_dir/$ogs_codegraph_platform_tarball_name"
[[ "$(sha512sum "$ogs_codegraph_tarball" | awk '{print $1}')" \
  == "$ogs_codegraph_integrity_hex" ]] || {
  echo "The downloaded CodeGraph wrapper does not match the pinned SHA-512 integrity." >&2
  exit 1
}
[[ "$(sha512sum "$ogs_codegraph_platform_tarball" | awk '{print $1}')" \
  == "$ogs_codegraph_platform_integrity_hex" ]] || {
  echo "The downloaded CodeGraph platform binary does not match the pinned SHA-512 integrity." >&2
  exit 1
}

[[ "$ogs_codegraph_prefix" == "$ogs_root/.dev/pipeline-tools/codegraph" ]] || {
  echo "Refusing to replace an unexpected CodeGraph bootstrap path." >&2
  exit 1
}
rm -rf -- "$ogs_codegraph_prefix"
mkdir -p "$ogs_codegraph_prefix"
DO_NOT_TRACK=1 npm install \
  --prefix "$ogs_codegraph_prefix" \
  --no-save \
  --package-lock=false \
  --ignore-scripts \
  --no-audit \
  --no-fund \
  "$ogs_codegraph_tarball" \
  "$ogs_codegraph_platform_tarball"

ogs_codegraph_installed_packages=$(
  find "$ogs_codegraph_prefix/node_modules/@colbymchenry" \
    -mindepth 1 -maxdepth 1 -type d -printf '%f\n' \
    | LC_ALL=C sort
)
ogs_codegraph_expected_packages=$(
  printf '%s\n' codegraph "codegraph-$ogs_codegraph_platform" | LC_ALL=C sort
)
[[ "$ogs_codegraph_installed_packages" == "$ogs_codegraph_expected_packages" ]] || {
  echo "The verified CodeGraph install contains unexpected packages." >&2
  exit 1
}
ogs_codegraph_tree_sha256=$(
  ogs_relative_tree_digest "$ogs_codegraph_prefix/node_modules/@colbymchenry"
)
[[ "$ogs_codegraph_tree_sha256" == "$ogs_codegraph_expected_tree_sha256" ]] || {
  echo "The verified CodeGraph install has an unexpected package tree." >&2
  exit 1
}
[[ -L "$ogs_codegraph_prefix/node_modules/.bin/codegraph" \
  && "$(readlink "$ogs_codegraph_prefix/node_modules/.bin/codegraph")" \
    == "../@colbymchenry/codegraph/npm-shim.js" ]] || {
  echo "The verified CodeGraph executable link is invalid." >&2
  exit 1
}
ogs_codegraph_provenance_temp=$(
  mktemp "$ogs_prefix/.codegraph-install-provenance.XXXXXX"
)
{
  printf 'version=1\n'
  printf 'package=%s\n' "$ogs_codegraph_package"
  printf 'package_sha512=%s\n' "$ogs_codegraph_integrity_hex"
  printf 'platform_package=%s\n' "$ogs_codegraph_platform_package"
  printf 'platform_sha512=%s\n' "$ogs_codegraph_platform_integrity_hex"
  printf 'tree_sha256=%s\n' "$ogs_codegraph_tree_sha256"
} >"$ogs_codegraph_provenance_temp"
mv "$ogs_codegraph_provenance_temp" "$ogs_codegraph_provenance"

if [[ ! -d "$ogs_openwiki_source/.git" ]]; then
  [[ ! -e "$ogs_openwiki_source" ]] || {
    echo "OpenWiki setup path exists but is not the expected Git checkout: $ogs_openwiki_source" >&2
    exit 1
  }
  git clone --no-checkout https://github.com/langchain-ai/openwiki.git "$ogs_openwiki_source"
  git -C "$ogs_openwiki_source" checkout --detach "$ogs_openwiki_commit"
fi

ogs_installed_commit=$(git -C "$ogs_openwiki_source" rev-parse HEAD)
[[ "$ogs_installed_commit" == "$ogs_openwiki_commit" ]] || {
  echo "OpenWiki checkout is not at pinned commit $ogs_openwiki_commit." >&2
  exit 1
}

ogs_declared_package_manager=$(
  node -p 'require(process.argv[1]).packageManager' "$ogs_openwiki_source/package.json"
)
[[ "$ogs_declared_package_manager" == "$ogs_pnpm_package_manager" ]] || {
  echo "OpenWiki package-manager integrity changed; refusing an unreviewed bootstrap." >&2
  exit 1
}

[[ -f "$ogs_openwiki_mode" && -f "$ogs_openwiki_session" ]] || {
  echo "OpenWiki package layout changed; refusing to apply the Codex-only patch." >&2
  exit 1
}

grep -Fxq "$ogs_agent_files_original" "$ogs_openwiki_mode" \
  || grep -Fxq "$ogs_agent_files_codex" "$ogs_openwiki_mode" || {
  echo "OpenWiki agent-guide source changed; refusing an unreviewed patch." >&2
  exit 1
}
grep -Fq 'createWorkflow: input.mode === "init"' "$ogs_openwiki_session" \
  || grep -Fq 'createWorkflow: false' "$ogs_openwiki_session" || {
  echo "OpenWiki workflow source changed; refusing an unreviewed patch." >&2
  exit 1
}

# The pinned release maintains more than one root agent guide and generates an
# unattended provider workflow. This project permits only its trusted Codex
# surface, so narrow both behaviors in ignored, generated dependency state.
sed -i \
  's/^const CODE_MODE_AGENT_FILES = .*;$/const CODE_MODE_AGENT_FILES = ["AGENTS.md"];/' \
  "$ogs_openwiki_mode"
sed -i \
  's/createWorkflow: input.mode === "init"/createWorkflow: false/' \
  "$ogs_openwiki_session"

DO_NOT_TRACK=1 npm pack \
  --silent \
  --pack-destination "$ogs_bootstrap_dir" \
  "pnpm@$ogs_pnpm_version" >/dev/null
ogs_pnpm_tarball=$(
  find "$ogs_bootstrap_dir" -maxdepth 1 -type f -name 'pnpm-*.tgz' -print -quit
)
[[ -n "$ogs_pnpm_tarball" ]] || {
  echo "The integrity-pinned pnpm tarball was not downloaded." >&2
  exit 1
}
ogs_downloaded_integrity=$(sha512sum "$ogs_pnpm_tarball" | awk '{print $1}')
[[ "$ogs_downloaded_integrity" == "$ogs_pnpm_integrity_hex" ]] || {
  echo "The downloaded pnpm tarball does not match the pinned SHA-512 integrity." >&2
  exit 1
}

[[ "$ogs_pnpm_prefix" == "$ogs_root/.dev/pipeline-tools/pnpm" ]] || {
  echo "Refusing to replace an unexpected pnpm bootstrap path." >&2
  exit 1
}
rm -rf -- "$ogs_pnpm_prefix"
mkdir -p "$ogs_pnpm_prefix"
DO_NOT_TRACK=1 npm install \
  --prefix "$ogs_pnpm_prefix" \
  --no-save \
  --ignore-scripts \
  --no-audit \
  --no-fund \
  "$ogs_pnpm_tarball"
ogs_pnpm_bin="$ogs_pnpm_prefix/node_modules/.bin/pnpm"
[[ "$("$ogs_pnpm_bin" --version)" == "$ogs_pnpm_version" ]] || {
  echo "The verified pnpm bootstrap has an unexpected version." >&2
  exit 1
}

ogs_openwiki_modules="$ogs_openwiki_source/node_modules"
[[ "$ogs_openwiki_modules" == "$ogs_root/.dev/pipeline-tools/openwiki/node_modules" ]] || {
  echo "Refusing to replace an unexpected OpenWiki dependency path." >&2
  exit 1
}
rm -rf -- "$ogs_openwiki_modules"
rm -f -- "$ogs_openwiki_source/package-lock.json"
DO_NOT_TRACK=1 OPENWIKI_TELEMETRY_DISABLED=1 "$ogs_pnpm_bin" \
  --dir "$ogs_openwiki_source" \
  install \
  --frozen-lockfile \
  --ignore-scripts \
  --prod=false

"$ogs_openwiki_source/node_modules/.bin/tsc" \
  -p "$ogs_openwiki_source/tsconfig.json"
"$ogs_openwiki_source/node_modules/.bin/tsc" \
  -p "$ogs_openwiki_source/tsconfig.client.json"
node "$ogs_openwiki_source/scripts/copy-visualize-assets.cjs"
chmod +x "$ogs_openwiki_source/dist/cli/cli.js"

grep -Fxq "$ogs_agent_files_codex" "$ogs_openwiki_mode" || {
  echo "OpenWiki Codex-only agent-guide patch did not apply." >&2
  exit 1
}

ogs_source_changes=$(git -C "$ogs_openwiki_source" diff --name-only | LC_ALL=C sort)
[[ "$ogs_source_changes" == $'src/ingestion/code-mode.ts\nsrc/integrations/core/session-manager.ts' ]] || {
  echo "OpenWiki generated checkout has unexpected tracked changes:" >&2
  printf '%s\n' "$ogs_source_changes" >&2
  exit 1
}
grep -Fq 'createWorkflow: false' "$ogs_openwiki_session" || {
  echo "OpenWiki Codex-only scheduled-workflow patch did not apply." >&2
  exit 1
}

ogs_provenance_temp=$(mktemp "$ogs_prefix/.openwiki-build-provenance.XXXXXX")
{
  printf 'version=1\n'
  printf 'commit=%s\n' "$ogs_openwiki_commit"
  printf 'package_manager=%s\n' "$ogs_pnpm_package_manager"
  printf 'pnpm_tree_sha256=%s\n' "$(ogs_tree_digest "$ogs_pnpm_prefix/node_modules/pnpm")"
  printf 'lock_sha256=%s\n' "$(sha256sum "$ogs_openwiki_source/pnpm-lock.yaml" | awk '{print $1}')"
  printf 'patch_sha256=%s\n' "$(git -C "$ogs_openwiki_source" diff --binary | sha256sum | awk '{print $1}')"
  printf 'dist_sha256=%s\n' "$(ogs_tree_digest "$ogs_openwiki_source/dist")"
} >"$ogs_provenance_temp"
mv "$ogs_provenance_temp" "$ogs_openwiki_provenance"

cd "$ogs_root"
if [[ -d .codegraph ]]; then
  scripts/codegraph.sh sync
else
  scripts/codegraph.sh init
fi

scripts/check-pipeline-tools.sh
echo "Pipeline tools are ready. Restart Codex so it reviews and loads .codex/config.toml and .codex/hooks.json."
