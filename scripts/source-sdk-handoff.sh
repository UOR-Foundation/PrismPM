#!/usr/bin/env bash
# Run inside the source devcontainer, using its actual Docker daemon.
set -euo pipefail
if test "$#" -lt 6; then
  echo 'usage: source-sdk-handoff.sh IMAGE REV ARCH REF EVENT COMMAND...' >&2
  exit 64
fi
image=$1 revision=$2 architecture=$3 ref=$4 event=$5
shift 5
root=$(cd "$(dirname "$0")/.." && pwd -P)
# Leave stdin for the original command, including a caller's here-document.
node "$root/scripts/source-sdk-handoff.mjs" "$image" "$revision" "$architecture" "$ref" "$event" </dev/null
export PRISMPM_TEST_SDK_IMAGE="$image"
exec "$@"
