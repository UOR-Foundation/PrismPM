#!/usr/bin/env bash
# Observed allocated bytes, not a descriptor-based bound or acceptance receipt.
set -euo pipefail
test "$#" -ge 2
label=$1 output=$2
shift 2
test "${GITHUB_ACTIONS:-}" = true
test ! -e "$output"
(
  printf 'stage=%s utc=%s\n' "$label" "$(date -u +%FT%TZ)"
  df --block-size=1 / "${RUNNER_TEMP:?}"
  daemon_root=$(docker info --format '{{.DockerRootDir}}')
  test -n "$daemon_root"
  # Includes unpacked snapshots and content. This runner owns its daemon.
  sudo du --summarize --one-file-system --block-size=1 -- "$daemon_root"
  for path in "$@"; do
    if test -e "$path"; then
      test ! -L "$path"
      du --summarize --one-file-system --block-size=1 -- "$path"
    fi
  done
  docker system df
) > "$output"
cat "$output"
