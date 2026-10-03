#!/usr/bin/env bash
# Additional native-profile evidence, never full SDK or product acceptance.
set -euo pipefail
test "$#" -eq 2 || { echo 'usage: library-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT' >&2; exit 64; }
image=$1
revision=$2
root=$(cd "$(dirname "$0")/.." && pwd -P)
helper="$root/scripts/library-sdk-check.mjs"
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
docker container cp "$container:/opt/prismpm/share/inventory.json" "$sdk_work/inventory.json"
cmp "$root/sdk/devcontainer-init.sh" "$sdk_work/entrypoint.sh"
docker container rm "$container" >/dev/null
container=''
# Online metadata acquisition runs the source-bound SDK helper. Only optional
# read-only registry credentials enter this phase; no host helper can execute.
# Start Node directly: the SDK initializer's root-to-user transition requires
# capabilities deliberately absent here. No Cargo/runtime initialization occurs.
credential_mount=()
registry_directory="${DOCKER_CONFIG:-${HOME}/.docker}"
if test -d "$registry_directory"; then
  registry_directory=$(cd "$registry_directory" && pwd -P)
  case "$registry_directory" in *,*|*$'\n'*|*$'\r'*) exit 64 ;; esac
  credential_mount=(--mount "type=bind,source=$registry_directory,target=/run/prismpm-registry-auth,readonly")
fi
container=$(docker container create --entrypoint node --user "$(id -u):$(id -g)" --read-only --network bridge --cap-drop ALL \
  --security-opt no-new-privileges --memory 512m --pids-limit 128 --tmpfs /tmp:rw,nosuid,nodev,size=64m \
  "${credential_mount[@]}" --env DOCKER_CONFIG=/run/prismpm-registry-auth \
  --workdir /opt/prismpm/share/conformance-root "$image" \
  scripts/library-sdk-check.mjs acquire-lock "$image")
[[ $container =~ ^[0-9a-f]{64}$ ]] || exit 1
docker container start --attach "$container" > "$sdk_work/lock.json"
test "$(docker container inspect --format '{{.State.ExitCode}}' "$container")" = 0
docker container rm "$container" >/dev/null
container=''
node "$helper" binding "$sdk_work/lock.json" "$image" "$architecture" "$root/standards.lock" "$sdk_work/inventory.json" > "$sdk_work/binding.json"
# Execution has no network, credentials, host implementation, or writable mount.
# The independently captured lock is the only explicit stdin data.
container=$(docker container create --interactive --user 1000:1000 --read-only --network none --cap-drop ALL \
  --security-opt no-new-privileges --tmpfs /tmp:rw,exec,nosuid,nodev,size=8g \
  --env PRISMPM_EPHEMERAL_HOME=1 --env CARGO_NET_OFFLINE=true \
  --workdir /opt/prismpm/share/conformance-root "$image" \
  node scripts/library-sdk-check.mjs run "$image")
[[ $container =~ ^[0-9a-f]{64}$ ]] || exit 1
docker container start --attach --interactive "$container" < "$sdk_work/lock.json" > "$sdk_work/result.json"
test "$(docker container inspect --format '{{.State.ExitCode}}' "$container")" = 0
node "$helper" result "$sdk_work/result.json" "$sdk_work/lock.json" "$image" "$architecture" "$root/standards.lock" "$sdk_work/inventory.json"
cat "$sdk_work/result.json"
docker container rm "$container" >/dev/null
container=''
node "$helper" verify "$root" "$sdk_work/source.json"
printf 'native-library SDK closure passed: source=%s image=%s platform=linux/%s\n' "$revision" "$image" "$architecture"
