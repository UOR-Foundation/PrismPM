#!/usr/bin/env bash
# Basic host Docker infrastructure only; SDK commands execute in pinned tools.
set -euo pipefail
test "$#" -eq 4 || { echo 'usage: sdk-vv-outer.sh IMAGE@sha256:DIGEST SOURCE_COMMIT ARCH FRESH_OUTPUT' >&2; exit 64; }
sdk_image=$1 revision=$2 architecture=$3 destination=$4
[[ $sdk_image =~ ^[a-z0-9][a-z0-9./:_-]*@sha256:[0-9a-f]{64}$ ]] || exit 64
[[ $revision =~ ^[0-9a-f]{40}$ ]] || exit 64
case "$(uname -m):$architecture" in x86_64:amd64|aarch64:arm64) ;; *) exit 64 ;; esac
root=$(cd "$(dirname "$0")/.." && pwd -P)
workspace=$(pwd -P)
test "$(git -C "$root" rev-parse HEAD)" = "$revision"
test -z "$(git -C "$root" status --porcelain)"
test "$root" = "$workspace/root-a"
[[ $destination =~ ^sdk-(amd64|arm64)-full-sdk-vv$ ]] || exit 64
test "$destination" = "sdk-$architecture-full-sdk-vv"
test ! -e "$workspace/$destination"
evidence="$workspace/$destination-outer-owner"
mkdir -m 0700 "$evidence"
owner="prism-sdk-outer-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT-$architecture"
tag="prism-sdk-outer-tools:$revision-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT-$architecture"
label=org.uor.prismpm.sdk-outer
container_id= image_id= cleaning=0 cleanup_deadline=0
controlled() {
  local name=$1 bound=$2 status remaining
  shift 2
  if (( cleaning )); then
    remaining=$((cleanup_deadline-SECONDS)); (( remaining>0 )) || return 124
    (( remaining>=bound )) || bound=$remaining
  fi
  if timeout --signal=TERM --kill-after=2s "${bound}s" docker "$@" >"$evidence/$name.stdout" 2>"$evidence/$name.stderr"; then status=0; else status=$?; fi
  printf '%s\n' "$status" >"$evidence/$name.status"
  cat "$evidence/$name.stdout"
  return "$status"
}
absent() {
  local name=$1 reference=$2 status diagnostic
  if controlled "$name" 10 container inspect "$reference"; then return 1; else status=$?; fi
  test "$status" = 1 || return 1
  diagnostic=$(cat "$evidence/$name.stderr")
  [[ $diagnostic =~ [Nn]o[[:space:]]such[[:space:]](object|container) ]]
}
cleanup() {
  local status=$? actual
  trap - EXIT
  cleaning=1 cleanup_deadline=$((SECONDS+60))
  # Reconcile a lost create response by its unique label, then use the actual
  # immutable container ID. Never remove an unrelated replacement by name.
  if controlled survivor 10 container inspect "$owner"; then
    actual=$(controlled survivor-id 10 container inspect "$owner" --format '{{.Id}}')
    [[ $actual =~ ^[a-f0-9]{64}$ ]] || exit 1
    test "$(controlled survivor-label 10 container inspect "$actual" --format '{{index .Config.Labels "org.uor.prismpm.sdk-outer"}}')" = "$owner" || exit 1
    if test -n "$container_id"; then test "$actual" = "$container_id" || exit 1; fi
    container_id=$actual
    controlled removal 10 container rm --force --volumes "$container_id" || exit 1
  fi
  absent name-absence "$owner" || exit 1
  if test -n "$container_id"; then absent id-absence "$container_id" || exit 1; fi
  if test -n "$image_id"; then
    test "$(controlled tag-identity 10 image inspect "$tag" --format '{{.Id}}')" = "$image_id" || exit 1
    controlled tag-removal 10 image rm "$tag" || exit 1
  fi
  printf '%s\n' "$status" >"$evidence/cleanup.status"
  exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
absent prior "$owner"
# This infrastructure stage is source-independent and small. Keep the original
# 12GiB SDK reserve plus a separate16GiB build allowance BEFORE construction.
available=$(df --output=avail -B1 "$workspace" | tail -n 1)
(( available >= 28*1024*1024*1024 )) || { echo 'insufficient space for tools and original SDK reserve' >&2; exit 1; }
controlled tools-build 1200 buildx build --file "$root/sdk/Dockerfile" --target registry_qualification_tools \
  --platform "linux/$architecture" --load --tag "$tag" "$root"
image_id=$(controlled tools-id 10 image inspect "$tag" --format '{{.Id}}')
[[ $image_id =~ ^sha256:[a-f0-9]{64}$ ]] || exit 1
controlled tools-inspect 10 image inspect "$image_id" >/dev/null
socket_group=$(stat -c '%g' /var/run/docker.sock)
container_id=$(controlled create 30 container create --init --name "$owner" --cidfile "$evidence/container.id" \
  --label "$label=$owner" --pull=never --platform "linux/$architecture" \
  --read-only --user "$(id -u):$(id -g)" --group-add "$socket_group" \
  --cpus 2 --memory 1g --memory-swap 1g --pids-limit 256 --cap-drop ALL --security-opt no-new-privileges \
  --tmpfs /tmp:rw,exec,nosuid,nodev,size=16g,mode=1777 \
  --mount "type=bind,source=$workspace,target=$workspace" \
  --mount "type=bind,source=$root,target=$root,readonly" \
  --mount type=bind,source=/var/run/docker.sock,target=/var/run/docker.sock \
  --workdir "$workspace" --env GITHUB_ACTIONS --env GITHUB_SHA --env RUNNER_ENVIRONMENT --env RUNNER_OS \
  --env RUNNER_ARCH --env GITHUB_RUN_ID --env GITHUB_RUN_ATTEMPT --entrypoint /bin/bash "$image_id" \
  --noprofile --norc -euo pipefail -c \
  'node root-a/scripts/sdk-vv-check.mjs tests; node root-a/scripts/sdk-vv-check.mjs run "$1" "$2" "$3" "$4"' \
  sdk-outer "$sdk_image" "$revision" "$architecture" "$destination")
[[ $container_id =~ ^[a-f0-9]{64}$ ]] || exit 1
test "$(cat "$evidence/container.id")" = "$container_id"
# The original360-minute job and inner two-run4-hour deadline remain unchanged.
if docker container start --attach "$container_id" >"$evidence/actor.stdout" 2>"$evidence/actor.stderr"; then attached=0; else attached=$?; fi
printf '%s\n' "$attached" >"$evidence/actor.status"
controlled terminal 10 container inspect "$container_id" >/dev/null
test "$attached" = 0
test "$(controlled actor-running 10 container inspect "$container_id" --format '{{.State.Running}}')" = false
test "$(controlled actor-exit 10 container inspect "$container_id" --format '{{.State.ExitCode}}')" = 0
test "$(controlled actor-oom 10 container inspect "$container_id" --format '{{.State.OOMKilled}}')" = false
