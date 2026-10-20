#!/usr/bin/env bash
# Independent post-build closure test; never consumes the external-oracle image override.
set -euo pipefail
test "$#" -eq 2 -o "$#" -eq 3 || { echo 'usage: browser-api-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT [EXPLICIT_REGISTRY_CA]' >&2; exit 64; }
image=$1
revision=$2
root=$(cd "$(dirname "$0")/.." && pwd -P)
helper="$root/scripts/browser-api-sdk-check.mjs"
if test "$#" -eq 3; then
  node "$root/scripts/sdk-qualification-trust.mjs" "$3"
  export NODE_EXTRA_CA_CERTS=$3
fi
[[ $image =~ ^[a-z0-9][a-z0-9./:_-]*@sha256:[0-9a-f]{64}$ ]] || exit 64
[[ $revision =~ ^[0-9a-f]{40}$ ]] || exit 64
test "$(git -C "$root" rev-parse HEAD)" = "$revision"
test -z "$(git -C "$root" status --porcelain)"
case "$(uname -m)" in x86_64) architecture=amd64 ;; aarch64) architecture=arm64 ;; *) exit 64 ;; esac
sdk_work=$(mktemp -d)
container=''
container_name=''
container_role=''
image_id=''
volume=''
volume_nonce=''
volume_key=org.uor.prismpm.browser-sdk-private
owned_volume() {
  local observed
  test -n "$volume" && test -n "$volume_nonce" || return 1
  observed=$(docker volume inspect --format '{{.Name}} {{.Driver}} {{.Scope}} {{len .Options}} {{len .Labels}} {{index .Labels "org.uor.prismpm.browser-sdk-private"}}' "$volume") || return 1
  test "$observed" = "$volume local local 0 1 $volume_nonce"
}
owned_container() {
  local observed id name image nonce role running pid extra
  test -n "$container_name" || return 1
  observed=$(docker container inspect --format '{{.Id}} {{.Name}} {{.Image}} {{index .Config.Labels "org.uor.prismpm.browser-sdk-private"}} {{index .Config.Labels "org.uor.prismpm.browser-sdk-role"}} {{.State.Running}} {{.State.Pid}}' "$container_name") || return 1
  [[ $observed != *$'\n'* ]] || return 1
  read -r id name image nonce role running pid extra <<< "$observed"
  [[ $id =~ ^[0-9a-f]{64}$ && $pid =~ ^[0-9]+$ ]] && test -z "$extra" || return 1
  test "$name" = "/$container_name" && test "$image" = "$image_id" && test "$nonce" = "$volume_nonce" && test "$role" = "$container_role" || return 1
  test -z "$container" || test "$container" = "$id" || return 1
  [[ $running = true || $running = false ]] || return 1
  container=$id; container_running=$running; container_pid=$pid
}
new_container() {
  local role=$1 requested prior created
  shift
  test -z "$container" && test -z "$container_name"
  case "$role" in copy|init|owner) ;; *) return 1 ;; esac
  requested="prismpm-browser-sdk-$volume_nonce-$role"
  prior=$(docker container ls --all --filter "name=^/$requested$" --format '{{.Names}}')
  test -z "$prior"
  container_name=$requested; container_role=$role
  created=$(docker container create --name "$container_name" --label "$volume_key=$volume_nonce" --label "org.uor.prismpm.browser-sdk-role=$role" "$@")
  [[ $created =~ ^[0-9a-f]{64}$ ]] || return 1
  container=$created
  owned_container
}
retire_container() {
  local remaining
  owned_container || return 1
  if test "$container_running" = true; then docker container stop --timeout 5 "$container" || return 1; owned_container || return 1; fi
  test "$container_running" = false && test "$container_pid" = 0 || return 1
  docker container rm "$container" || return 1
  remaining=$(docker container ls --all --filter "id=$container" --format '{{.ID}}') || return 1
  test -z "$remaining" || return 1
  remaining=$(docker container ls --all --filter "name=^/$container_name$" --format '{{.Names}}') || return 1
  test -z "$remaining" || return 1
  container=''; container_name=''; container_role=''
}
cleanup_resources() {
  local failed=0 remaining
  if test -n "$container_name"; then retire_container || failed=1; fi
  # Never mutate a volume with unproved ownership or a possibly live owner.
  if test -n "$volume"; then
    if test -z "$container_name" && owned_volume && docker volume rm "$volume"; then
      if remaining=$(docker volume ls --filter "name=$volume" --format '{{.Name}}') && test -z "$remaining"; then volume=''; else failed=1; fi
    else failed=1; fi
  fi
  if test "$failed" -eq 0; then
    chmod -R u+rwX -- "$sdk_work" && rm -r -- "$sdk_work" || failed=1
  fi
  return "$failed"
}
on_exit() {
  local original=$? cleanup_status=0
  trap - EXIT
  cleanup_resources || cleanup_status=$?
  if test "$cleanup_status" -ne 0; then printf 'browser SDK owned cleanup failed; work=%s volume=%s container=%s\n' "$sdk_work" "$volume" "$container_name/$container" >&2; fi
  if test "$original" -ne 0; then exit "$original"; fi
  exit "$cleanup_status"
}
trap on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
node "$helper" capture "$root" "$revision" > "$sdk_work/source.json"
docker pull --platform "linux/$architecture" "$image"
docker image inspect "$image" | node "$helper" image "$image" "$architecture" "$revision"
image_id=$(docker image inspect --format '{{.Id}}' "$image")
[[ $image_id =~ ^sha256:[0-9a-f]{64}$ ]] || exit 1
volume_nonce=$(cat /proc/sys/kernel/random/uuid)
[[ $volume_nonce =~ ^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$ ]] || exit 1
volume_nonce=${volume_nonce//-/}
new_container copy --network none --read-only --entrypoint /usr/bin/true "$image"
mkdir "$sdk_work/source" "$sdk_work/browser"
while IFS= read -r path; do
  mkdir -p "$sdk_work/source/$(dirname "$path")"
  # Defer read-only directory modes until descendants are written.
  docker container cp "$container:/opt/prismpm/share/conformance-root/$path" - | tar --extract --no-same-owner --no-same-permissions --delay-directory-restore --file - --directory "$sdk_work/source/$(dirname "$path")"
done < <(node "$helper" roots)
node "$helper" verify "$sdk_work/source" "$sdk_work/source.json"
# Defer read-only directory modes until descendants are written.
docker container cp "$container:/opt/prismpm/browser/." - | tar --extract --no-same-owner --no-same-permissions --delay-directory-restore --file - --directory "$sdk_work/browser"
node "$helper" modules "$root" "$sdk_work/browser"
mkdir -p "$sdk_work/share/account-genesis"
# Defer read-only directory modes until descendants are written.
docker container cp "$container:/opt/prismpm/share/account-genesis/." - | tar --extract --no-same-owner --no-same-permissions --delay-directory-restore --file - --directory "$sdk_work/share/account-genesis"
node "$helper" account-construction "$sdk_work" "$sdk_work/source.json"
docker container cp "$container:/usr/local/bin/prismpm-devcontainer-init" "$sdk_work/entrypoint.sh"
cmp "$root/sdk/devcontainer-init.sh" "$sdk_work/entrypoint.sh"
retire_container
# A private same-daemon disk volume supplies real compiler storage. No host
# source/dependency mount exists; every original 12 GiB reserve guard remains.
requested_volume="prismpm-browser-sdk-$volume_nonce"
prior_volumes=$(docker volume ls --filter "name=$requested_volume" --format '{{.Name}}')
test -z "$prior_volumes"
volume=$requested_volume
created_volume=$(docker volume create --driver local --label "$volume_key=$volume_nonce" "$volume")
test "$created_volume" = "$volume"
owned_volume
new_container init --user 0:0 --read-only --network none --cap-drop ALL --cpus 1 --memory 256m --memory-swap 256m --pids-limit 64 \
  --security-opt no-new-privileges --mount "type=volume,source=$volume,target=/tmp,volume-nocopy" \
  --entrypoint /usr/local/bin/node "$image" --input-type=module -e 'import assert from "node:assert/strict";
import {lstatSync,readdirSync,chmodSync} from "node:fs";
const s=lstatSync("/tmp");assert(s.isDirectory()&&!s.isSymbolicLink()&&s.uid===0,"owned fresh volume root");
assert.deepEqual(readdirSync("/tmp"),[],"new no-copy volume must be empty");
chmodSync("/tmp",0o1777);assert.equal(lstatSync("/tmp").mode&0o7777,0o1777);'
owned_volume
owned_container
timeout --signal=TERM --kill-after=5s 30s docker container start --attach "$container"
observed_exit=$(docker container inspect --format '{{.State.ExitCode}}' "$container")
test "$observed_exit" = 0
retire_container
owned_volume
new_container owner --user 1000:1000 --read-only --network none --cap-drop ALL \
  --security-opt no-new-privileges --mount "type=volume,source=$volume,target=/tmp,volume-nocopy" \
  --env PRISMPM_EPHEMERAL_HOME=1 --env CARGO_NET_OFFLINE=true \
  --workdir /opt/prismpm/share/conformance-root "$image" \
  /bin/sh -ec 'node --input-type=module -e "$1"; exec node scripts/browser-api-sdk-check.mjs run' private-sdk-storage   'import assert from "node:assert/strict";import {statfsSync} from "node:fs";
const s=statfsSync("/tmp",{bigint:true}),available=s.bavail*s.bsize;
process.stdout.write("browser SDK private storage "+JSON.stringify({available_bytes:String(available),required_bytes:String(12n*1024n**3n)})+"\n");
assert(available>=12n*1024n**3n,"12 GiB private compiler reserve before original installed owners");'
owned_volume
owned_container
docker container start --attach "$container"
observed_exit=$(docker container inspect --format '{{.State.ExitCode}}' "$container")
test "$observed_exit" = 0
retire_container
node "$helper" verify "$root" "$sdk_work/source.json"
if cleanup_resources; then trap - EXIT; else trap - EXIT; exit 1; fi
printf 'browser SDK closure passed: source=%s image=%s platform=linux/%s\n' "$revision" "$image" "$architecture"
