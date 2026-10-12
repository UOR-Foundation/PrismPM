#!/usr/bin/env bash
# Runner-local OCI transport for real native binary qualification; never publish.
set -euo pipefail
test "$#" -eq 4 || { echo 'usage: binary-sdk-qualify.sh OCI_TAR ARCH SOURCE_COMMIT NEW_EVIDENCE_DIR' >&2; exit 64; }
archive=$1
architecture=$2
revision=$3
evidence=$4
root=$(cd "$(dirname "$0")/.." && pwd -P)
case "$(uname -m):$architecture" in x86_64:amd64|aarch64:arm64) ;; *) exit 64 ;; esac
[[ $revision =~ ^[0-9a-f]{40}$ ]] || exit 64
test "$(git -C "$root" rev-parse HEAD)" = "$revision"
test -z "$(git -C "$root" status --porcelain)"
test ! -e "$evidence" && test ! -L "$evidence"
# Reuse the existing reviewed OCI layout/config parser and pinned ORAS transport.
bash "$root/scripts/sdk-candidate.sh" inspect "$archive" "$architecture" "$revision" "$evidence"
evidence=$(cd "$evidence" && pwd -P)
digest=$(cat "$evidence/digest.txt")
[[ $digest =~ ^sha256:[0-9a-f]{64}$ ]] || exit 64
nonce="$$-${RANDOM}"
registry="prismpm-binary-qualification-${nonce}"
volume="prismpm-binary-config-${nonce}"
scratch=$(mktemp -d)
cleanup() {
  docker container rm --force "$registry" >/dev/null 2>&1 || true
  docker volume rm "$volume" >/dev/null 2>&1 || true
  rm -r -- "$scratch"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
printf '%s\n' '{"distSpecVersion":"1.1.1","http":{"address":"0.0.0.0","port":5000},"log":{"level":"warn"},"storage":{"rootDirectory":"/tmp/zot"}}' > "$scratch/config.json"
docker volume create "$volume" >/dev/null
docker container create --name "$registry" --publish 127.0.0.1::5000 \
  --read-only --tmpfs /tmp:rw,nosuid,nodev --volume "$volume:/config" \
  ghcr.io/project-zot/zot@sha256:cd2aea942f428630bcb4190542be6abd35e14177aab84fc7ccad0dca8ecb363d \
  serve /config/config.json >/dev/null
docker container cp "$scratch/config.json" "$registry:/config/config.json"
docker container start "$registry" >/dev/null
endpoint=$(docker container port "$registry" 5000/tcp | sed -n '1p')
[[ $endpoint =~ ^127\.0\.0\.1:[0-9]+$ ]] || exit 64
ready=0
for attempt in $(seq 1 20); do
  if curl --fail --silent --max-time 2 "http://$endpoint/v2/" >/dev/null; then ready=1; break; fi
  sleep 0.25
done
if test "$ready" -ne 1; then docker logs "$registry" >&2; exit 1; fi
oras cp --from-oci-layout --to-plain-http "$archive@$digest" "$endpoint/sdk:binary-qualification"
test "$(oras resolve --plain-http "$endpoint/sdk:binary-qualification")" = "$digest"
image="$endpoint/sdk@$digest"
printf '%s\n' "$image" > "$evidence/image.txt"
# Test the exact native OCI bytes; no host checkout/cache is mounted into this run.
bash "$root/scripts/binary-sdk-check.sh" "$image" "$revision" "$evidence/accepted" > "$evidence/binary-sdk.log" 2>&1
cat "$evidence/binary-sdk.log"
