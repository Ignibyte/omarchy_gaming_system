#!/usr/bin/env bash
set -Eeuo pipefail
export LC_ALL=C

ogs_build_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
ogs_source_candidate="$ogs_build_root"
ogs_output_candidate="$ogs_build_root/target/packages"

usage() {
  echo "Usage: $0 [--source-root PATH] [--output PATH]" >&2
}

while (( $# > 0 )); do
  case "$1" in
    --source-root)
      (( $# >= 2 )) || { usage; exit 2; }
      ogs_source_candidate="$2"
      shift 2
      ;;
    --output)
      (( $# >= 2 )) || { usage; exit 2; }
      ogs_output_candidate="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

for ogs_command in flock git id makepkg sha256sum stat; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "Missing client package build command: $ogs_command" >&2
    exit 1
  }
done

"$ogs_build_root/scripts/check-client-package-source.sh" "$ogs_source_candidate"
ogs_source_root="$(cd -- "$ogs_source_candidate" && pwd -P)"

if [[ -e "$ogs_output_candidate" && -L "$ogs_output_candidate" ]]; then
  echo "Client package output directory must not be a symlink." >&2
  exit 1
fi
mkdir -p -- "$ogs_output_candidate"
ogs_output_root="$(cd -- "$ogs_output_candidate" && pwd -P)"
ogs_temp="$(mktemp -d)"
trap 'rm -rf -- "$ogs_temp"' EXIT INT TERM
mkdir -p -- "$ogs_temp/packages"

ogs_build_workspace="/tmp/omarchygs-client-package-build-$(id -u)"
if [[ -e "$ogs_build_workspace" \
  && ( ! -d "$ogs_build_workspace" || -L "$ogs_build_workspace" ) ]]; then
  echo "Stable client package build workspace is not a safe directory." >&2
  exit 1
fi
install -d -m0700 -- "$ogs_build_workspace"
if [[ "$(stat -c '%u' "$ogs_build_workspace")" != "$(id -u)" \
  || "$(stat -c '%a' "$ogs_build_workspace")" != 700 ]]; then
  echo "Stable client package build workspace has unsafe ownership or mode." >&2
  exit 1
fi
if [[ -e "$ogs_build_workspace/PKGBUILD" \
  && ( ! -f "$ogs_build_workspace/PKGBUILD" \
    || -L "$ogs_build_workspace/PKGBUILD" ) ]]; then
  echo "Stable client package PKGBUILD path is not a regular file." >&2
  exit 1
fi
exec 9>"$ogs_build_workspace/build.lock"
flock 9
cp -- "$ogs_source_root/packaging/arch/PKGBUILD" "$ogs_build_workspace/PKGBUILD"

ogs_digest_records="$ogs_temp/digest-records"
: >"$ogs_digest_records"
(
  cd -- "$ogs_source_root"
  for ogs_path in \
    Cargo.lock \
    Cargo.toml \
    packaging/arch/PKGBUILD \
    packaging/arch/client-runtime-files.txt \
    packaging/arch/com.ignibyte.OmarchyGS.desktop \
    packaging/arch/omarchygs; do
    ogs_hash="$(sha256sum -- "$ogs_path" | awk '{print $1}')"
    printf '%s\0%s\0' "$ogs_path" "$ogs_hash"
  done
  while IFS= read -r ogs_path; do
    ogs_hash="$(sha256sum -- "$ogs_path" | awk '{print $1}')"
    printf '%s\0%s\0' "$ogs_path" "$ogs_hash"
  done < <(
    find crates/client-cartridge-runtime crates/game-cartridge \
      -type f -print | LC_ALL=C sort
  )
  while IFS= read -r ogs_path; do
    ogs_hash="$(sha256sum -- "$ogs_path" | awk '{print $1}')"
    printf '%s\0%s\0' "$ogs_path" "$ogs_hash"
  done <packaging/arch/client-runtime-files.txt
) >"$ogs_digest_records"
ogs_source_digest="$(sha256sum -- "$ogs_digest_records" | awk '{print $1}')"

ogs_source_revision="$(git -C "$ogs_source_root" rev-parse HEAD 2>/dev/null || true)"
if [[ ! "$ogs_source_revision" =~ ^[0-9a-f]{40}$ ]]; then
  ogs_source_revision="unversioned"
fi
if [[ -n "$(git -C "$ogs_source_root" status --porcelain=v1 2>/dev/null || true)" ]]; then
  ogs_source_dirty="true"
else
  ogs_source_dirty="false"
fi
ogs_source_date_epoch="$(git -C "$ogs_source_root" log -1 --format=%ct 2>/dev/null || true)"
if [[ ! "$ogs_source_date_epoch" =~ ^[0-9]+$ ]]; then
  ogs_source_date_epoch=0
fi

(
  cd -- "$ogs_build_workspace"
  env \
    OMARCHYGS_SOURCE_ROOT="$ogs_source_root" \
    OMARCHYGS_SOURCE_DIGEST="$ogs_source_digest" \
    OMARCHYGS_SOURCE_REVISION="$ogs_source_revision" \
    OMARCHYGS_SOURCE_DIRTY="$ogs_source_dirty" \
    SOURCE_DATE_EPOCH="$ogs_source_date_epoch" \
    PKGDEST="$ogs_temp/packages" \
    makepkg --clean --cleanbuild --force --nodeps --noconfirm
)

mapfile -d '' -t ogs_packages < <(
  find "$ogs_temp/packages" -maxdepth 1 -type f \
    -name '*.pkg.tar.*' ! -name '*.sig' -print0
)
if (( ${#ogs_packages[@]} != 1 )); then
  echo "Client package build did not emit exactly one artifact." >&2
  exit 1
fi

ogs_package_name="$(basename -- "${ogs_packages[0]}")"
ogs_package_target="$ogs_output_root/$ogs_package_name"
ogs_package_temp="$(mktemp "$ogs_output_root/.omarchygs-package.XXXXXX")"
ogs_receipt_temp="$(mktemp "$ogs_output_root/.omarchygs-package-sha256.XXXXXX")"
trap 'rm -rf -- "$ogs_temp"; rm -f -- "$ogs_package_temp" "$ogs_receipt_temp"' EXIT INT TERM
install -m0644 -- "${ogs_packages[0]}" "$ogs_package_temp"
ogs_package_digest="$(sha256sum -- "$ogs_package_temp" | awk '{print $1}')"
printf '%s  %s\n' "$ogs_package_digest" "$ogs_package_name" >"$ogs_receipt_temp"
mv -f -- "$ogs_package_temp" "$ogs_package_target"
mv -f -- "$ogs_receipt_temp" "$ogs_package_target.sha256"

printf 'OGS_CLIENT_PACKAGE artifact=%s sha256=%s source_sha256=%s revision=%s dirty=%s\n' \
  "$ogs_package_target" \
  "$ogs_package_digest" \
  "$ogs_source_digest" \
  "$ogs_source_revision" \
  "$ogs_source_dirty"
