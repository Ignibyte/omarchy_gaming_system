#!/usr/bin/env bash
set -Eeuo pipefail

bbs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "$bbs_root/bin/gate.sh" --fast
