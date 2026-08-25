#!/usr/bin/env bash
set -euo pipefail

ogs_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_fixture="$ogs_root/crates/game-cartridge-renderer/tests/fixtures/rich"
ogs_temp="$(mktemp -d)"
ogs_qml_pid=""

cleanup() {
  if [[ -n "$ogs_qml_pid" ]]; then
    kill "$ogs_qml_pid" 2>/dev/null || true
    wait "$ogs_qml_pid" 2>/dev/null || true
  fi
  rm -rf -- "$ogs_temp"
}
trap cleanup EXIT INT TERM

for ogs_command in base64 cargo cp mkdir ps python3 qml6 rg sed seq sleep stat tr; do
  command -v "$ogs_command" >/dev/null 2>&1 || {
    echo "required command is unavailable: $ogs_command" >&2
    exit 1
  }
done

ogs_reference_cpu=""
if command -v taskset >/dev/null 2>&1; then
  ogs_reference_cpu="$(python3 -c 'import os; print(min(os.sched_getaffinity(0)))')"
fi

cd "$ogs_root"
cargo test -p omarchygs-game-cartridge-renderer --all-targets
cargo build -p omarchygs-game-cartridge --bin omarchygs-cartridge
cargo build -p omarchygs-game-cartridge-renderer --bin omarchygs-cartridge-preview

ogs_cartridge_bin="$ogs_root/target/debug/omarchygs-cartridge"
ogs_preview_bin="$ogs_root/target/debug/omarchygs-cartridge-preview"
ogs_private_key="$ogs_temp/publisher.private.json"
ogs_public_key="$ogs_temp/publisher.public.json"

"$ogs_cartridge_bin" keygen ignibyte ignibyte-renderer-v1 \
  "$ogs_private_key" "$ogs_public_key" >"$ogs_temp/keygen.json"

prepare_source() {
  local ogs_name="$1"
  local ogs_source="$ogs_temp/source-$ogs_name"
  cp -R -- "$ogs_fixture" "$ogs_source"
  base64 --decode "$ogs_source/assets/pixel.png.base64" >"$ogs_source/assets/pixel.png"
  base64 --decode "$ogs_source/assets/tick.wav.base64" >"$ogs_source/assets/tick.wav"
  rm -- "$ogs_source/assets/pixel.png.base64" "$ogs_source/assets/tick.wav.base64"
  printf '%s\n' "$ogs_source"
}

ogs_base_source="$(prepare_source base)"
ogs_core_source="$(prepare_source core)"
ogs_rich_source="$(prepare_source rich)"
ogs_budget_source="$(prepare_source budget)"
ogs_overbudget_source="$(prepare_source overbudget)"

python3 - "$ogs_core_source" "$ogs_rich_source" "$ogs_budget_source" \
  "$ogs_overbudget_source" "$ogs_temp" <<'PY'
import binascii
import json
import pathlib
import struct
import sys
import zlib

core = pathlib.Path(sys.argv[1])
rich = pathlib.Path(sys.argv[2])
budget = pathlib.Path(sys.argv[3])
overbudget = pathlib.Path(sys.argv[4])
temporary = pathlib.Path(sys.argv[5])

def read(path):
    return json.loads(path.read_text())

def write(path, value):
    path.write_text(json.dumps(value, separators=(",", ":")))

def png_chunk(kind, payload):
    return (
        struct.pack(">I", len(payload))
        + kind
        + payload
        + struct.pack(">I", binascii.crc32(kind + payload) & 0xFFFFFFFF)
    )

def replace_raster(root, width):
    height = width
    scanline = b"\0" + b"\0" * (width * 4)
    png = (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(
            b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
        )
        + png_chunk(b"IDAT", zlib.compress(scanline * height, 9))
        + png_chunk(b"IEND", b"")
    )
    (root / "assets/pixel.png").write_bytes(png)
    manifest = read(root / "manifest.json")
    manifest["assets"][0].update({
        "decoded_bytes": width * height * 4,
        "width": width,
        "height": height,
    })
    write(root / "manifest.json", manifest)
    presentation = read(root / "presentation.json")
    nodes = presentation["screens"][0]["nodes"]
    nodes = [node for node in nodes if node["kind"] not in {"image", "sprite"}]
    nodes.append({
        "kind": "image",
        "id": "raster-profile-boundary",
        "asset": "assets/pixel.png",
        "accessible_label": f"{width} pixel profile boundary",
    })
    presentation["screens"][0]["nodes"] = nodes
    write(root / "presentation.json", presentation)

core_presentation = read(core / "presentation.json")
core_screen = core_presentation["screens"][0]
core_screen["nodes"] = [
    {
        "kind": "terminal", "id": f"log-{index}", "text_binding": "log.text",
        "accessible_label": f"Game log {index}"
    }
    for index in range(220)
] + [
    {
        "kind": "grid", "id": "board", "rows": 32, "columns": 32,
        "cells_binding": "board.cells", "action": "move", "accessible_label": "Game board"
    },
    {
        "kind": "status", "id": "status", "text_binding": "status.text",
        "accessible_label": "Game status"
    },
    {
        "kind": "button", "id": "end-turn", "label_binding": "button.label",
        "action": "end_turn", "accessible_label": "End turn"
    },
    {
        "kind": "meter", "id": "health", "value_binding": "meter.value",
        "minimum": 0, "maximum": 100, "accessible_label": "Health"
    },
] + [
    {
        "kind": "image", "id": f"portrait-{index}", "asset": "assets/pixel.png",
        "accessible_label": f"Portrait {index}"
    }
    for index in range(32)
]
write(core / "presentation.json", core_presentation)

core_schema = read(core / "schemas/view.schema.json")
cells = core_schema["properties"]["board"]["properties"]["cells"]
cells["minItems"] = 1024
cells["maxItems"] = 1024
write(core / "schemas/view.schema.json", core_schema)
core_view = read(core / "view.json")
core_view["board"]["cells"] = [str(index % 10) for index in range(1024)]
write(core / "view.json", core_view)

rich_presentation = read(rich / "presentation.json")
rich_screen = rich_presentation["screens"][0]
rich_screen["nodes"] = [
    {
        "kind": "terminal", "id": "log", "text_binding": "log.text",
        "accessible_label": "Game log"
    },
    {
        "kind": "grid", "id": "board", "rows": 2, "columns": 2,
        "cells_binding": "board.cells", "action": "move", "accessible_label": "Game board"
    },
    {
        "kind": "status", "id": "status", "text_binding": "status.text",
        "accessible_label": "Game status"
    },
    {
        "kind": "button", "id": "end-turn", "label_binding": "button.label",
        "action": "end_turn", "accessible_label": "End turn"
    },
    {
        "kind": "meter", "id": "health", "value_binding": "meter.value",
        "minimum": 0, "maximum": 100, "accessible_label": "Health"
    },
] + [
    {
        "kind": "image", "id": f"portrait-{index}", "asset": "assets/pixel.png",
        "accessible_label": f"Portrait {index}"
    }
    for index in range(64)
] + [
    {
        "kind": "sprite", "id": f"hero-{index}", "asset": "assets/pixel.png",
        "frame_width": 1, "frame_height": 1, "frame_count": 1,
        "frames_per_second": 12, "accessible_label": f"Hero {index}"
    }
    for index in range(127)
] + [
    {
        "kind": "particle_field", "id": "stars", "particle_count": 2048,
        "preset": "stars", "accessible_label": "Star field"
    }
] + [
    {
        "kind": "audio_cue", "id": f"tick-{index}", "asset": "assets/tick.wav",
        "looped": False, "accessible_label": f"Turn sound {index}"
    }
    for index in range(16)
]
write(rich / "presentation.json", rich_presentation)

accessibility_preferences = read(rich / "preferences.json")
accessibility_preferences.update({
    "scale": 2.0,
    "high_contrast": True,
    "reduced_motion": True,
    "muted_audio": True,
})
write(temporary / "preferences-accessibility.json", accessibility_preferences)
replace_raster(budget, 2048)
replace_raster(overbudget, 4096)
PY

for ogs_name in base core rich budget overbudget; do
  ogs_source_variable="ogs_${ogs_name}_source"
  ogs_source="${!ogs_source_variable}"
  cp -- "$ogs_source/view.json" "$ogs_temp/view-$ogs_name.json"
  cp -- "$ogs_source/preferences.json" "$ogs_temp/preferences-$ogs_name.json"
  rm -- "$ogs_source/view.json" "$ogs_source/preferences.json"
  "$ogs_cartridge_bin" pack "$ogs_source" "$ogs_private_key" \
    "$ogs_temp/$ogs_name.ogsc" >"$ogs_temp/pack-$ogs_name.json"
done

run_preview() {
  local ogs_name="$1"
  local ogs_profile="$2"
  local ogs_state="$3"
  local ogs_variant="${4:-$ogs_name}"
  local ogs_source_variable="ogs_${ogs_name}_source"
  local ogs_source="${!ogs_source_variable}"
  local ogs_output="$ogs_temp/output-$ogs_name-$ogs_variant-$ogs_state"
  local ogs_receipt="$ogs_temp/receipt-$ogs_name-$ogs_variant-$ogs_state.json"
  mkdir -m 700 -- "$ogs_output"

  env \
    DATABASE_URL='postgres://unusable.invalid/no-access' \
    OMARCHYGS_DEVICE_TOKEN='must-not-be-read' \
    HTTP_PROXY='http://127.0.0.1:1' \
    HTTPS_PROXY='http://127.0.0.1:1' \
    "$ogs_preview_bin" prepare "$ogs_temp/$ogs_name.ogsc" "$ogs_public_key" \
      "$ogs_profile" "$ogs_temp/view-$ogs_name.json" "$ogs_state" \
      "$ogs_temp/preferences-$ogs_variant.json" "$ogs_output" >"$ogs_receipt"

  rg --fixed-strings '"provider_contacted":false' "$ogs_receipt" >/dev/null
  rg --fixed-strings '"database_required":false' "$ogs_receipt" >/dev/null
  rg --fixed-strings '"platform_credentials_read":false' "$ogs_receipt" >/dev/null

  local ogs_plan="$ogs_output/render-plan.json"
  local ogs_asset_root="$ogs_output/assets"
  local ogs_log="$ogs_temp/qml-$ogs_name-$ogs_variant-$ogs_state.log"
  local -a ogs_qml_command=(qml6)
  if [[ -n "$ogs_reference_cpu" ]]; then
    ogs_qml_command=(taskset -c "$ogs_reference_cpu" qml6)
  fi
  QML_XHR_ALLOW_FILE_READ=1 QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software \
    QT_FORCE_STDERR_LOGGING=1 QT_LOGGING_RULES='qml=true;*.warning=true' \
    "${ogs_qml_command[@]}" "$ogs_root/client/qml/cartridge/CartridgePreview.qml" -- \
      --smoke-test --plan="$ogs_plan" --asset-root="$ogs_asset_root" \
      >"$ogs_log" 2>&1 &
  ogs_qml_pid=$!

  local ogs_peak_rss=0
  local ogs_timed_out=true
  for _ in $(seq 1 300); do
    local ogs_process_state
    ogs_process_state="$(ps -o stat= -p "$ogs_qml_pid" 2>/dev/null | tr -d '[:space:]' || true)"
    if [[ -z "$ogs_process_state" || "$ogs_process_state" == Z* ]]; then
      ogs_timed_out=false
      break
    fi
    local ogs_rss
    ogs_rss="$(ps -o rss= -p "$ogs_qml_pid" 2>/dev/null | tr -d '[:space:]' || true)"
    if [[ "$ogs_rss" =~ ^[0-9]+$ ]] && (( ogs_rss > ogs_peak_rss )); then
      ogs_peak_rss=$ogs_rss
    fi
    sleep 0.05
  done
  if [[ "$ogs_timed_out" == true ]]; then
    kill "$ogs_qml_pid" 2>/dev/null || true
  fi
  if ! wait "$ogs_qml_pid"; then
    echo "trusted cartridge QML failed: $ogs_name/$ogs_state" >&2
    sed -n '1,240p' "$ogs_log" >&2
    return 1
  fi
  ogs_qml_pid=""

  if rg 'ReferenceError|TypeError|Required property|Unable to assign|is not a type|failed to load component' \
    "$ogs_log" >/dev/null; then
    echo "trusted cartridge QML emitted a runtime contract error: $ogs_name/$ogs_state" >&2
    sed -n '1,240p' "$ogs_log" >&2
    return 1
  fi
  local ogs_metrics
  ogs_metrics="$(rg --only-matching 'OGS_CARTRIDGE_RENDER_METRICS state=[a-z_]+ nodes=[0-9]+ frames=120 average_ms=[0-9.]+ max_ms=[0-9.]+' "$ogs_log")"
  [[ -n "$ogs_metrics" ]] || {
    echo "trusted cartridge QML emitted no metrics: $ogs_name/$ogs_state" >&2
    sed -n '1,240p' "$ogs_log" >&2
    return 1
  }
  python3 - "$ogs_metrics" <<'PY'
import re
import sys
match = re.search(r"average_ms=([0-9.]+)", sys.argv[1])
if match is None or float(match.group(1)) > 33.3:
    raise SystemExit(1)
PY
  if [[ "$ogs_state" == "ready" ]]; then
    rg --fixed-strings 'OGS_CARTRIDGE_INPUT_METRICS expected=2 exercised=2 focus=true' \
      "$ogs_log" >/dev/null || {
      echo "trusted cartridge QML did not prove input/focus behavior" >&2
      sed -n '1,240p' "$ogs_log" >&2
      return 1
    }
  fi
  local ogs_hard_rss=524288
  if [[ "$ogs_profile" == "core" ]]; then
    ogs_hard_rss=393216
  fi
  (( ogs_peak_rss <= ogs_hard_rss )) || {
    echo "trusted cartridge QML exceeded profile RSS: $ogs_peak_rss KiB" >&2
    return 1
  }
  echo "$ogs_metrics peak_rss_kib=$ogs_peak_rss profile=$ogs_profile fixture=$ogs_name/$ogs_variant affinity_cpu=${ogs_reference_cpu:-unconstrained}"
}

run_preview core core ready
run_preview rich rich2d ready
run_preview budget rich2d ready
run_preview base rich2d ready accessibility
for ogs_state in loading offline stale empty protocol_error unsupported_capability revoked; do
  run_preview base rich2d "$ogs_state"
done

ogs_overbudget_output="$ogs_temp/output-overbudget-rejected"
ogs_overbudget_receipt="$ogs_temp/receipt-overbudget-rejected.json"
mkdir -m 700 -- "$ogs_overbudget_output"
if "$ogs_preview_bin" prepare "$ogs_temp/overbudget.ogsc" "$ogs_public_key" \
  rich2d "$ogs_temp/view-overbudget.json" ready \
  "$ogs_temp/preferences-overbudget.json" "$ogs_overbudget_output" \
  >"$ogs_overbudget_receipt"; then
  echo "Rich-2D accepted the 4096px raster availability trigger" >&2
  exit 1
fi
rg --fixed-strings '"code":"renderer_budget_exceeded"' \
  "$ogs_overbudget_receipt" >/dev/null
[[ ! -e "$ogs_overbudget_output/render-plan.json" ]] || {
  echo "rejected raster trigger published a render plan" >&2
  exit 1
}

python3 - "$ogs_temp/output-core-core-ready/render-plan.json" "$ogs_temp/rejected-core-plan.json" <<'PY'
import json
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2])
plan = json.loads(source.read_text())
plan["nodes"][0] = {
    "kind": "particle_field",
    "id": "over-core-budget",
    "particle_count": 1,
    "preset": "stars",
    "running": True,
    "accessible_label": "Must be rejected",
}
destination.write_text(json.dumps(plan, separators=(",", ":")))
PY

ogs_rejected_log="$ogs_temp/qml-rejected-core-budget.log"
ogs_rejected_command=(qml6)
if [[ -n "$ogs_reference_cpu" ]]; then
  ogs_rejected_command=(taskset -c "$ogs_reference_cpu" qml6)
fi
if QML_XHR_ALLOW_FILE_READ=1 QT_QPA_PLATFORM=offscreen QT_QUICK_BACKEND=software \
  "${ogs_rejected_command[@]}" "$ogs_root/client/qml/cartridge/CartridgePreview.qml" -- \
    --smoke-test --plan="$ogs_temp/rejected-core-plan.json" \
    --asset-root="$ogs_temp/output-core-core-ready/assets" \
    >"$ogs_rejected_log" 2>&1; then
  echo "trusted cartridge QML accepted a plan above the Core aggregate budget" >&2
  sed -n '1,240p' "$ogs_rejected_log" >&2
  exit 1
fi

echo "trusted game cartridge renderer passed"
