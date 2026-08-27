#!/usr/bin/env bash
set -Eeuo pipefail
export LC_ALL=C.UTF-8

ogs_test_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
ogs_temp="$(mktemp -d)"
ogs_fixture_pid=""
ogs_channel_build_pid=""

cleanup() {
  local ogs_status=$?
  if [[ -n "$ogs_fixture_pid" ]] && kill -0 "$ogs_fixture_pid" 2>/dev/null; then
    kill "$ogs_fixture_pid" 2>/dev/null || true
    wait "$ogs_fixture_pid" 2>/dev/null || true
  fi
  if [[ -n "$ogs_channel_build_pid" ]] \
    && kill -0 "$ogs_channel_build_pid" 2>/dev/null; then
    kill "$ogs_channel_build_pid" 2>/dev/null || true
    wait "$ogs_channel_build_pid" 2>/dev/null || true
  fi
  rm -rf -- "$ogs_temp"
  exit "$ogs_status"
}
trap cleanup EXIT INT TERM

for ogs_command in \
  bsdtar cargo cmp curl desktop-file-validate flock git jq makepkg pacman python3 \
  qml6 sha256sum stat timeout truncate; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "Missing client package test command: $ogs_command" >&2
    exit 1
  }
done

fail() {
  echo "Client package test failed: $*" >&2
  exit 1
}

copy_source() {
  local ogs_destination="$1"
  mkdir -p -- "$ogs_destination"
  cp -a -- \
    "$ogs_test_root/Cargo.lock" \
    "$ogs_test_root/Cargo.toml" \
    "$ogs_test_root/client" \
    "$ogs_test_root/crates" \
    "$ogs_test_root/packaging" \
    "$ogs_destination/"
}

expect_source_rejection() {
  local ogs_name="$1"
  local ogs_case_root="$ogs_temp/source-$ogs_name"
  copy_source "$ogs_case_root"
  shift
  "$@" "$ogs_case_root"
  if "$ogs_test_root/scripts/check-client-package-source.sh" \
    "$ogs_case_root" >"$ogs_temp/$ogs_name.out" 2>"$ogs_temp/$ogs_name.err"; then
    fail "$ogs_name source fixture was accepted"
  fi
}

remove_manifest_entry() {
  sed -i '1d' "$1/packaging/arch/client-runtime-files.txt"
}

add_unlisted_runtime() {
  printf '%s\n' 'import QtQuick' >"$1/client/qml/Unlisted.qml"
}

duplicate_manifest_entry() {
  local ogs_first
  ogs_first="$(sed -n '1p' "$1/packaging/arch/client-runtime-files.txt")"
  printf '%s\n' "$ogs_first" >>"$1/packaging/arch/client-runtime-files.txt"
}

add_traversal_entry() {
  printf '%s\n' 'client/qml/../Cargo.toml' \
    >>"$1/packaging/arch/client-runtime-files.txt"
}

reverse_manifest() {
  tac "$1/packaging/arch/client-runtime-files.txt" \
    >"$1/packaging/arch/client-runtime-files.txt.reversed"
  mv -- \
    "$1/packaging/arch/client-runtime-files.txt.reversed" \
    "$1/packaging/arch/client-runtime-files.txt"
}

remove_manifest_terminator() {
  truncate -s -1 -- "$1/packaging/arch/client-runtime-files.txt"
}

symlink_runtime_entry() {
  rm -f -- "$1/client/qml/ApiClient.qml"
  ln -s -- Main.qml "$1/client/qml/ApiClient.qml"
}

"$ogs_test_root/scripts/check-client-package-source.sh" "$ogs_test_root"
expect_source_rejection missing remove_manifest_entry
expect_source_rejection extra add_unlisted_runtime
expect_source_rejection duplicate duplicate_manifest_entry
expect_source_rejection traversal add_traversal_entry
expect_source_rejection unsorted reverse_manifest
expect_source_rejection unterminated remove_manifest_terminator
expect_source_rejection symlink symlink_runtime_entry

if "$ogs_test_root/scripts/build-client-package.sh" \
  --source-root "$ogs_temp/source-traversal" \
  --output "$ogs_temp/rejected-build" \
  >"$ogs_temp/rejected-build.out" 2>"$ogs_temp/rejected-build.err"; then
  fail "builder accepted an invalid source manifest"
fi
[[ ! -e "$ogs_temp/rejected-build" ]] \
  || fail "invalid source created a package output directory"

git -C "$ogs_test_root" status --porcelain=v1 -z >"$ogs_temp/status-before"
"$ogs_test_root/scripts/build-client-package.sh" --output "$ogs_temp/build-one"
"$ogs_test_root/scripts/build-client-package.sh" --output "$ogs_temp/build-two"
git -C "$ogs_test_root" status --porcelain=v1 -z >"$ogs_temp/status-after"
cmp -- "$ogs_temp/status-before" "$ogs_temp/status-after" \
  || fail "package build modified source-tree status"

cargo run --quiet --locked \
  --manifest-path "$ogs_test_root/Cargo.toml" \
  --package omarchygs-marketplace-trust \
  --bin omarchygs-marketplace-channel -- \
  generate-root package-root package-channel \
  "$ogs_temp/package-root.private.json" \
  "$ogs_temp/package-root.public.json" >/dev/null
cargo run --quiet --locked \
  --manifest-path "$ogs_test_root/Cargo.toml" \
  --package omarchygs-marketplace-trust \
  --bin omarchygs-marketplace-channel -- \
  bootstrap "$ogs_temp/package-root.public.json" \
  https://packages.example.test/v1/ trust.signed.json 1 1 \
  arch-linux x86_64 0.1.0-1 "$ogs_temp/package-bootstrap.json" >/dev/null
cp -- "$ogs_temp/package-bootstrap.json" "$ogs_temp/expected-package-bootstrap.json"
"$ogs_test_root/scripts/build-client-package.sh" \
  --channel-bootstrap "$ogs_temp/package-bootstrap.json" \
  --output "$ogs_temp/build-channel" \
  >"$ogs_temp/build-channel.out" 2>"$ogs_temp/build-channel.err" &
ogs_channel_build_pid=$!
ogs_channel_lock="/tmp/omarchygs-client-package-build-$(id -u)/build.lock"
ogs_channel_lock_observed=false
for _ in {1..400}; do
  if ! flock -n "$ogs_channel_lock" true 2>/dev/null; then
    ogs_channel_lock_observed=true
    break
  fi
  kill -0 "$ogs_channel_build_pid" 2>/dev/null \
    || fail "channel build stopped before acquiring its stable build lock"
  sleep 0.05
done
[[ "$ogs_channel_lock_observed" == true ]] \
  || fail "channel build did not acquire its stable build lock"
printf '\n' >>"$ogs_temp/package-bootstrap.json"
if ! wait "$ogs_channel_build_pid"; then
  ogs_channel_build_pid=""
  cat "$ogs_temp/build-channel.err" >&2
  fail "channel build did not retain its verified bootstrap snapshot"
fi
ogs_channel_build_pid=""
cp -- "$ogs_temp/expected-package-bootstrap.json" "$ogs_temp/tampered-bootstrap.json"
printf '\n' >>"$ogs_temp/tampered-bootstrap.json"
if "$ogs_test_root/scripts/build-client-package.sh" \
  --channel-bootstrap "$ogs_temp/tampered-bootstrap.json" \
  --output "$ogs_temp/rejected-channel-build" \
  >"$ogs_temp/rejected-channel-build.out" \
  2>"$ogs_temp/rejected-channel-build.err"; then
  fail "builder accepted a non-canonical channel bootstrap"
fi
[[ ! -e "$ogs_temp/rejected-channel-build" ]] \
  || fail "invalid channel bootstrap created a package output directory"

mapfile -d '' -t ogs_first_packages < <(
  find "$ogs_temp/build-one" -maxdepth 1 -type f \
    -name '*.pkg.tar.*' ! -name '*.sig' ! -name '*.sha256' -print0
)
mapfile -d '' -t ogs_second_packages < <(
  find "$ogs_temp/build-two" -maxdepth 1 -type f \
    -name '*.pkg.tar.*' ! -name '*.sig' ! -name '*.sha256' -print0
)
(( ${#ogs_first_packages[@]} == 1 && ${#ogs_second_packages[@]} == 1 )) \
  || fail "reproducibility builds did not each emit one package"
ogs_package="${ogs_first_packages[0]}"
ogs_package_name="$(basename -- "$ogs_package")"
[[ "$(basename -- "${ogs_second_packages[0]}")" == "$ogs_package_name" ]] \
  || fail "reproducibility builds emitted different package names"
cmp -- "$ogs_package" "${ogs_second_packages[0]}" \
  || fail "two package builds were not byte-identical"

ogs_package_digest="$(sha256sum -- "$ogs_package" | awk '{print $1}')"
grep -Fxq -- "$ogs_package_digest  $ogs_package_name" "$ogs_package.sha256" \
  || fail "package SHA-256 sidecar does not bind the artifact"
cmp -- "$ogs_package.sha256" "${ogs_second_packages[0]}.sha256" \
  || fail "reproducibility builds emitted different SHA-256 sidecars"

mapfile -d '' -t ogs_channel_packages < <(
  find "$ogs_temp/build-channel" -maxdepth 1 -type f \
    -name '*.pkg.tar.*' ! -name '*.sig' ! -name '*.sha256' -print0
)
(( ${#ogs_channel_packages[@]} == 1 )) \
  || fail "channel build did not emit exactly one package"
mkdir -p -- "$ogs_temp/extracted-channel"
bsdtar -xf "${ogs_channel_packages[0]}" -C "$ogs_temp/extracted-channel"
ogs_packaged_bootstrap="$ogs_temp/extracted-channel/usr/share/omarchy-gaming-system/marketplace-channel-bootstrap.json"
cmp -- "$ogs_temp/expected-package-bootstrap.json" "$ogs_packaged_bootstrap" \
  || fail "channel package did not preserve the verified bootstrap"
[[ "$(stat -c '%a' "$ogs_packaged_bootstrap")" == 644 ]] \
  || fail "packaged channel bootstrap mode is not 0644"
ogs_bootstrap_digest="$(sha256sum -- "$ogs_packaged_bootstrap" | awk '{print $1}')"
grep -Fxq "channel_bootstrap_sha256=$ogs_bootstrap_digest" \
  "$ogs_temp/extracted-channel/usr/share/doc/omarchy-gaming-system-client/BUILD-PROVENANCE" \
  || fail "channel package provenance does not bind its bootstrap"

pacman -Qip -- "$ogs_package" >/dev/null \
  || fail "pacman rejected client package metadata"
bsdtar -xOf "$ogs_package" .PKGINFO >"$ogs_temp/PKGINFO"
grep -Fxq 'pkgname = omarchy-gaming-system-client' "$ogs_temp/PKGINFO" \
  || fail "package name is incorrect"
grep -Fxq 'pkgver = 0.1.0-1' "$ogs_temp/PKGINFO" \
  || fail "package version is incorrect"
grep -Fxq 'arch = x86_64' "$ogs_temp/PKGINFO" \
  || fail "package architecture is incorrect"
for ogs_dependency in gcc-libs glibc qt6-declarative; do
  grep -Fxq "depend = $ogs_dependency" "$ogs_temp/PKGINFO" \
    || fail "$ogs_dependency runtime dependency is missing"
done
[[ "$(grep -c '^depend = ' "$ogs_temp/PKGINFO")" == 3 ]] \
  || fail "client package declares an unexpected runtime dependency"

mkdir -p -- "$ogs_temp/extracted"
bsdtar -xf "$ogs_package" -C "$ogs_temp/extracted"
if find "$ogs_temp/extracted/usr" -type l -print -quit | grep -q .; then
  fail "client package payload contains a symbolic link"
fi
if find "$ogs_temp/extracted/usr" ! -type d ! -type f -print -quit | grep -q .; then
  fail "client package payload contains a non-regular object"
fi

: >"$ogs_temp/expected-payload"
printf '%s\n' \
  usr/bin/omarchygs-cartridge-companion \
  usr/bin/omarchygs \
  usr/share/applications/com.ignibyte.OmarchyGS.desktop \
  usr/share/doc/omarchy-gaming-system-client/BUILD-PROVENANCE \
  >>"$ogs_temp/expected-payload"
while IFS= read -r ogs_runtime_path; do
  printf 'usr/share/omarchy-gaming-system/qml/%s\n' \
    "${ogs_runtime_path#client/qml/}" \
    >>"$ogs_temp/expected-payload"
done <"$ogs_test_root/packaging/arch/client-runtime-files.txt"
sort -o "$ogs_temp/expected-payload" -- "$ogs_temp/expected-payload"
find "$ogs_temp/extracted/usr" -type f -printf '%P\n' \
  | sed 's#^#usr/#' | sort >"$ogs_temp/actual-payload"
if ! cmp -s -- "$ogs_temp/expected-payload" "$ogs_temp/actual-payload"; then
  diff -u -- "$ogs_temp/expected-payload" "$ogs_temp/actual-payload" >&2 || true
  fail "client package payload does not match the exact manifest"
fi

for ogs_executable in omarchygs omarchygs-cartridge-companion; do
  [[ "$(stat -c '%a' "$ogs_temp/extracted/usr/bin/$ogs_executable")" == 755 ]] \
    || fail "$ogs_executable mode is not 0755"
done
while IFS= read -r ogs_payload_path; do
  [[ "$ogs_payload_path" == usr/bin/omarchygs \
    || "$ogs_payload_path" == usr/bin/omarchygs-cartridge-companion ]] && continue
  [[ "$(stat -c '%a' "$ogs_temp/extracted/$ogs_payload_path")" == 644 ]] \
    || fail "$ogs_payload_path mode is not 0644"
done <"$ogs_temp/actual-payload"

ogs_provenance="$ogs_temp/extracted/usr/share/doc/omarchy-gaming-system-client/BUILD-PROVENANCE"
grep -Fxq 'format=omarchygs-client-build-provenance-v1' "$ogs_provenance" \
  || fail "package provenance format is missing"
grep -Eq '^source_revision=([0-9a-f]{40}|unversioned)$' "$ogs_provenance" \
  || fail "package provenance revision is invalid"
grep -Eq '^source_dirty=(true|false)$' "$ogs_provenance" \
  || fail "package provenance dirty state is invalid"
grep -Eq '^source_sha256=[0-9a-f]{64}$' "$ogs_provenance" \
  || fail "package provenance source digest is invalid"
grep -Fxq 'channel_bootstrap_sha256=none' "$ogs_provenance" \
  || fail "manual client package falsely claims a public channel bootstrap"
if grep -Fq -- "$ogs_test_root" "$ogs_provenance"; then
  fail "package provenance leaked the source path"
fi

ogs_desktop="$ogs_temp/extracted/usr/share/applications/com.ignibyte.OmarchyGS.desktop"
desktop-file-validate "$ogs_desktop" \
  || fail "packaged desktop entry is invalid"
grep -Fxq 'Exec=omarchygs' "$ogs_desktop" \
  || fail "desktop entry does not resolve the packaged launcher"
grep -Fxq 'Terminal=false' "$ogs_desktop" \
  || fail "desktop entry unexpectedly launches a terminal"
grep -Fxq 'Categories=Game;' "$ogs_desktop" \
  || fail "desktop entry categories are incorrect"

ogs_port_file="$ogs_temp/fixture.port"
ogs_fixture_log="$ogs_temp/fixture.log"
python3 "$ogs_test_root/client/qml/tests/fixture_server.py" \
  "$ogs_port_file" normal >"$ogs_fixture_log" 2>&1 &
ogs_fixture_pid=$!
for _ in {1..100}; do
  [[ -s "$ogs_port_file" ]] && break
  kill -0 "$ogs_fixture_pid" 2>/dev/null \
    || fail "package smoke fixture stopped during startup"
  sleep 0.05
done
[[ -s "$ogs_port_file" ]] || fail "package smoke fixture did not publish a port"
ogs_fixture_url="http://127.0.0.1:$(<"$ogs_port_file")"
install -d -m0700 -- "$ogs_temp/runtime" "$ogs_temp/data" "$ogs_temp/home"

ln -s -- "$ogs_temp/missing-marketplace-key.json" "$ogs_temp/untrusted-marketplace-key.json"
if env \
  HOME="$ogs_temp/home" \
  OGS_CLIENT_MARKETPLACE_PUBLIC_KEY="$ogs_temp/untrusted-marketplace-key.json" \
  PATH=/usr/bin \
  QT_QPA_PLATFORM=offscreen \
  QT_QUICK_BACKEND=software \
  XDG_DATA_HOME="$ogs_temp/data" \
  XDG_RUNTIME_DIR="$ogs_temp/runtime" \
  timeout 5 \
  "$ogs_temp/extracted/usr/bin/omarchygs" \
    --smoke-test \
    "--server-url=$ogs_fixture_url" \
    >"$ogs_temp/untrusted-marketplace.out" \
    2>"$ogs_temp/untrusted-marketplace.err"; then
  fail "extracted client accepted a symlinked marketplace trust key"
fi
grep -Fq 'trusted marketplace public key is missing or invalid' \
  "$ogs_temp/untrusted-marketplace.err" \
  || fail "invalid marketplace trust key did not fail explicitly"
if find "$ogs_temp/runtime" -mindepth 1 -print -quit | grep -q .; then
  fail "invalid marketplace trust configuration left runtime state"
fi

# Leave room for cold QML parsing under host I/O pressure. Main.qml keeps its
# independent 15-second post-load smoke deadline.
env \
  HOME="$ogs_temp/home" \
  PATH=/usr/bin \
  QT_QPA_PLATFORM=offscreen \
  QT_QUICK_BACKEND=software \
  XDG_DATA_HOME="$ogs_temp/data" \
  XDG_RUNTIME_DIR="$ogs_temp/runtime" \
  timeout 120 \
  "$ogs_temp/extracted/usr/bin/omarchygs" \
    --smoke-test \
    "--server-url=$ogs_fixture_url"

[[ -d "$ogs_temp/data/omarchy-gaming-system/cartridges" \
  && "$(stat -c '%a' "$ogs_temp/data/omarchy-gaming-system/cartridges")" == 700 ]] \
  || fail "extracted client did not create the private cartridge cache"
if find "$ogs_temp/runtime" -mindepth 1 -print -quit | grep -q .; then
  fail "extracted client left companion runtime state after exit"
fi

ogs_fixture_status="$(curl --fail --silent "$ogs_fixture_url/__fixture__/status")"
jq -e '
  .violations == [] and
  .calls == ["GET /.well-known/omarchygs", "GET /__fixture__/status"]
' <<<"$ogs_fixture_status" >/dev/null \
  || fail "extracted client package violated the discovery request contract"

echo "native Omarchy client package passed"
printf 'OGS_CLIENT_PACKAGE_TEST artifact=%s sha256=%s runtime_files=%s\n' \
  "$ogs_package_name" \
  "$ogs_package_digest" \
  "$(wc -l <"$ogs_test_root/packaging/arch/client-runtime-files.txt")"
