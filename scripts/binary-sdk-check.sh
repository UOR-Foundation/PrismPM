#!/usr/bin/env bash
# Additional binary-package evidence, never full SDK or product acceptance.
set -euo pipefail
test "$#" -ge 2 && test "$#" -le 3 || { echo 'usage: binary-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT [NEW_EVIDENCE_DIR]' >&2; exit 64; }
image=$1
revision=$2
evidence=${3:-}
if test -n "$evidence"; then test ! -e "$evidence" && test ! -L "$evidence"; fi
root=$(cd "$(dirname "$0")/.." && pwd -P)
helper="$root/scripts/binary-sdk-check.mjs"
[[ $image =~ ^[a-z0-9][a-z0-9./:_-]*@sha256:[0-9a-f]{64}$ ]] || exit 64
[[ $revision =~ ^[0-9a-f]{40}$ ]] || exit 64
test "$(git -C "$root" rev-parse HEAD)" = "$revision"
test -z "$(git -C "$root" status --porcelain)"
case "$(uname -m)" in x86_64) architecture=amd64 ;; aarch64) architecture=arm64 ;; *) exit 64 ;; esac
sdk_work=$(mktemp -d)
container=''
cleanup() {
  if test -n "$container"; then docker container rm --force "$container" >/dev/null 2>&1 || true; fi
  chmod -R u+rwX -- "$sdk_work"
  rm -r -- "$sdk_work"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
node "$helper" capture "$root" "$revision" > "$sdk_work/source.json"
docker pull --platform "linux/$architecture" "$image"
docker image inspect "$image" | node "$helper" image "$image" "$architecture" "$revision"
container=$(docker container create --network none --read-only --entrypoint /usr/bin/true "$image")
[[ $container =~ ^[0-9a-f]{64}$ ]] || exit 1
mkdir "$sdk_work/source"
while IFS= read -r path; do
  mkdir -p "$sdk_work/source/$(dirname "$path")"
  docker container cp "$container:/opt/prismpm/share/conformance-root/$path" "$sdk_work/source/$path"
done < <(node "$helper" roots)
node "$helper" verify "$sdk_work/source" "$sdk_work/source.json"
docker container cp "$container:/usr/local/bin/prismpm-devcontainer-init" "$sdk_work/entrypoint.sh"
cmp "$root/sdk/devcontainer-init.sh" "$sdk_work/entrypoint.sh"
docker container cp "$container:/opt/prismpm/share/inventory.json" "$sdk_work/inventory.json"
source_sha256=$(node "$helper" hash "$sdk_work/source.json")
inventory_sha256=$(node "$helper" inventory "$sdk_work/inventory.json" "$revision")
docker container rm "$container" >/dev/null
container=''
# No host source, implementation, dependency or writable mount enters execution.
container=$(docker container create --user 1000:1000 --read-only --network none --cap-drop ALL \
  --security-opt no-new-privileges --tmpfs /tmp:rw,exec,nosuid,nodev,size=8g \
  --env PRISMPM_EPHEMERAL_HOME=1 --env CARGO_NET_OFFLINE=true \
  --workdir /opt/prismpm/share/conformance-root "$image" \
  /bin/sh -ec '
    node scripts/binary-sdk-check.mjs run "$@" > /tmp/binary-result.json
    exec tar -c -C /tmp binary-result.json prismpm-binary-evidence
  ' binary-sdk "$image" "$revision" "$source_sha256" "$inventory_sha256")
[[ $container =~ ^[0-9a-f]{64}$ ]] || exit 1
# Export result and proof in the process stdout while tmpfs is still mounted.
# The normal initialized command must pass before tar runs; its final exit is
# checked before anything is extracted or accepted by the outer verifier.
docker container start --attach "$container" > "$sdk_work/evidence.tar"
test "$(docker container inspect --format '{{.State.ExitCode}}' "$container")" = 0
tar -x -f "$sdk_work/evidence.tar" -C "$sdk_work" --no-same-owner --no-same-permissions binary-result.json prismpm-binary-evidence
mv "$sdk_work/binary-result.json" "$sdk_work/result.json"
mv "$sdk_work/prismpm-binary-evidence" "$sdk_work/proof"
node "$helper" result "$sdk_work/result.json" "$image" "$revision" "$architecture" "$source_sha256" "$inventory_sha256"
node "$helper" evidence "$sdk_work/proof" "$sdk_work/result.json"
if test -n "$evidence"; then
  mkdir "$evidence"
  cp -a "$sdk_work/proof" "$evidence/proof"
  cp "$sdk_work/source.json" "$sdk_work/inventory.json" "$sdk_work/result.json" "$evidence/"
fi
cat "$sdk_work/result.json"
docker container rm "$container" >/dev/null
container=''
node "$helper" verify "$root" "$sdk_work/source.json"
printf 'binary-package SDK closure passed: source=%s image=%s platform=linux/%s\n' "$revision" "$image" "$architecture"
