#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

hosted_paths=(
  .buildkite
  .circleci
  .drone.yml
  .github/workflows
  .gitlab-ci.yml
  .woodpecker.yml
  Jenkinsfile
  azure-pipelines.yml
)

for ogs_path in "${hosted_paths[@]}"; do
  if [[ -f "$ogs_root/$ogs_path" ]]; then
    echo "Local-only automation check failed: hosted automation file exists: $ogs_path" >&2
    exit 1
  fi

  if [[ -d "$ogs_root/$ogs_path" ]] \
    && find "$ogs_root/$ogs_path" -type f -print -quit | grep -q .; then
    echo "Local-only automation check failed: hosted automation files exist under: $ogs_path" >&2
    exit 1
  fi
done

echo "Local-only automation check passed"
