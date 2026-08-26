#!/usr/bin/env bash
set -Eeuo pipefail
export LC_ALL=C

ogs_check_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"

if (( $# > 1 )) || [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  echo "Usage: $0 [source-root]" >&2
  exit 2
fi

ogs_source_candidate="${1:-$ogs_check_root}"
if [[ ! -d "$ogs_source_candidate" || -L "$ogs_source_candidate" ]]; then
  echo "Client package source root must be a non-symlink directory." >&2
  exit 1
fi
ogs_source_root="$(cd -- "$ogs_source_candidate" && pwd -P)"
ogs_manifest="$ogs_source_root/packaging/arch/client-runtime-files.txt"
ogs_temp="$(mktemp -d)"
trap 'rm -rf -- "$ogs_temp"' EXIT INT TERM

fail() {
  echo "Client package source check failed: $*" >&2
  exit 1
}

for ogs_required_path in \
  Cargo.toml \
  packaging/arch/PKGBUILD \
  packaging/arch/client-runtime-files.txt \
  packaging/arch/com.ignibyte.OmarchyGS.desktop \
  packaging/arch/omarchygs; do
  if [[ ! -f "$ogs_source_root/$ogs_required_path" \
    || -L "$ogs_source_root/$ogs_required_path" ]]; then
    fail "$ogs_required_path must be a non-symlink regular file"
  fi
done

if [[ -n "$(tail -c 1 -- "$ogs_manifest")" ]]; then
  fail "runtime manifest must end with a newline"
fi

ogs_previous=""
ogs_records=0
: >"$ogs_temp/listed"
while IFS= read -r ogs_path || [[ -n "$ogs_path" ]]; do
  ((ogs_records += 1))
  [[ -n "$ogs_path" ]] || fail "runtime manifest contains an empty record"
  [[ "$ogs_path" =~ ^client/qml/[A-Za-z0-9_-]+([./][A-Za-z0-9_-]+)*(\.qml)?$ ]] \
    || fail "runtime manifest contains an unsafe path"
  [[ "$ogs_path" != *"//"* \
    && "$ogs_path" != *"/./"* \
    && "$ogs_path" != *"/../"* \
    && "$ogs_path" != */. \
    && "$ogs_path" != */.. \
    && "$ogs_path" != client/qml/tests/* ]] \
    || fail "runtime manifest path escapes the production QML inventory"
  if [[ -n "$ogs_previous" && "$ogs_path" < "$ogs_previous" ]]; then
    fail "runtime manifest must be sorted"
  fi
  if [[ "$ogs_path" == "$ogs_previous" ]]; then
    fail "runtime manifest contains a duplicate path"
  fi
  [[ -f "$ogs_source_root/$ogs_path" && ! -L "$ogs_source_root/$ogs_path" ]] \
    || fail "$ogs_path must be a non-symlink regular file"
  printf '%s\n' "$ogs_path" >>"$ogs_temp/listed"
  ogs_previous="$ogs_path"
done <"$ogs_manifest"
((ogs_records > 0)) || fail "runtime manifest is empty"

: >"$ogs_temp/actual"
while IFS= read -r -d '' ogs_absolute_path; do
  ogs_relative_path="${ogs_absolute_path#"$ogs_source_root/"}"
  [[ "$ogs_relative_path" =~ ^client/qml/[A-Za-z0-9_-]+([./][A-Za-z0-9_-]+)*(\.qml)?$ ]] \
    || fail "production QML tree contains an unsafe path"
  [[ -f "$ogs_absolute_path" && ! -L "$ogs_absolute_path" ]] \
    || fail "$ogs_relative_path must be a non-symlink regular file"
  printf '%s\n' "$ogs_relative_path" >>"$ogs_temp/actual"
done < <(
  find "$ogs_source_root/client/qml" \
    -path "$ogs_source_root/client/qml/tests" -prune -o \
    ! -type d -print0
)
sort -o "$ogs_temp/actual" -- "$ogs_temp/actual"

if ! cmp -s -- "$ogs_temp/listed" "$ogs_temp/actual"; then
  diff -u -- "$ogs_temp/listed" "$ogs_temp/actual" >&2 || true
  fail "runtime manifest does not exactly match the production QML tree"
fi

ogs_workspace_version="$({
  awk '
    /^\[workspace\.package\]$/ { in_workspace_package = 1; next }
    /^\[/ { in_workspace_package = 0 }
    in_workspace_package && $1 == "version" {
      value = $3
      gsub(/"/, "", value)
      print value
      exit
    }
  ' "$ogs_source_root/Cargo.toml"
})"
ogs_package_version="$({
  awk -F= '$1 == "pkgver" { print $2; exit }' \
    "$ogs_source_root/packaging/arch/PKGBUILD"
})"
[[ -n "$ogs_workspace_version" && "$ogs_package_version" == "$ogs_workspace_version" ]] \
  || fail "PKGBUILD pkgver must equal the workspace package version"

bash -n "$ogs_source_root/packaging/arch/omarchygs" \
  || fail "client launcher is not valid Bash"
if command -v desktop-file-validate >/dev/null 2>&1; then
  desktop-file-validate \
    "$ogs_source_root/packaging/arch/com.ignibyte.OmarchyGS.desktop" \
    || fail "desktop entry is invalid"
fi

echo "Client package source contract passed ($ogs_records runtime files, version $ogs_workspace_version)"
