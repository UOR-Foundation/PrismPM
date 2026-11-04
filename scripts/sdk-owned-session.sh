#!/usr/bin/env bash
# Private CI transport ownership; never application or SDK acceptance evidence.

sdk_owned_begin() {
  SDK_SESSION_OWNER="prismpm-sdk-$$-$RANDOM-$RANDOM"
  declare -gA SDK_SESSION_IMAGES=() SDK_SESSION_IDS=() SDK_SESSION_BINDINGS=()
  declare -gA SDK_SESSION_VOLUMES=() SDK_SESSION_REMOVED=()
  SDK_SESSION_CONTAINER='' SDK_SESSION_VOLUME=''
}

sdk_owned_docker() {
  timeout --signal=TERM --kill-after=5s 30s docker "$@"
}

sdk_owned_absent() {
  local kind=$1 name=$2 observed
  if test "$kind" = container; then
    observed=$(sdk_owned_docker container ls --all --no-trunc --filter "name=^/${name}$" --format '{{.ID}}') || return 1
  else
    observed=$(sdk_owned_docker volume ls --filter "name=^${name}$" --format '{{.Name}}') || return 1
  fi
  test -z "$observed"
}

sdk_owned_volume_identity() {
  local name=$1 observed volume_name driver owner created scope options
  observed=$(sdk_owned_docker volume inspect "$name" --format '{{.Name}}|{{.Driver}}|{{index .Labels "io.prismpm.sdk-session"}}|{{.CreatedAt}}|{{.Scope}}|{{json .Options}}') || return 1
  [[ $observed != *$'\n'* && $observed != *$'\r'* ]] || return 1
  IFS='|' read -r volume_name driver owner created scope options <<< "$observed"
  test "$volume_name" = "$name" && test "$driver" = local && test "$owner" = "$SDK_SESSION_OWNER" &&
    test -n "$created" && test "$scope" = local && { test "$options" = null || test "$options" = '{}'; } || return 1
  if test -n "${SDK_SESSION_VOLUMES[$name]}"; then
    test "$observed" = "${SDK_SESSION_VOLUMES[$name]}" || return 1
  else
    SDK_SESSION_VOLUMES[$name]=$observed
  fi
}

sdk_owned_volume() {
  local role=$1 name="$SDK_SESSION_OWNER-$1" observed
  [[ $role =~ ^[a-z][a-z0-9-]{0,40}$ ]] || return 1
  sdk_owned_absent volume "$name" || return 1
  # Register after proven absence, before a possibly ambiguous daemon write.
  SDK_SESSION_VOLUMES[$name]=''
  observed=$(sdk_owned_docker volume create --driver local --label "io.prismpm.sdk-session=$SDK_SESSION_OWNER" "$name") || return "$?"
  test "$observed" = "$name" || return 1
  sdk_owned_volume_identity "$name" || return 1
  SDK_SESSION_VOLUME=$name
}

sdk_owned_container_identity() {
  local name=$1 observed id actual_name image_id image owner mounts extra expected_image_id
  observed=$(sdk_owned_docker container inspect "$name" --format '{{.Id}}|{{.Name}}|{{.Image}}|{{.Config.Image}}|{{index .Config.Labels "io.prismpm.sdk-session"}}|{{range .Mounts}}{{.Type}}:{{.Name}}:{{.Destination}}:{{.RW}};{{end}}') || return 1
  [[ $observed != *$'\n'* && $observed != *$'\r'* ]] || return 1
  IFS='|' read -r id actual_name image_id image owner mounts extra <<< "$observed"
  [[ $id =~ ^[0-9a-f]{64}$ && $image_id =~ ^sha256:[0-9a-f]{64}$ ]] &&
    test "$actual_name" = "/$name" && test "$image" = "${SDK_SESSION_IMAGES[$name]}" &&
    test "$owner" = "$SDK_SESSION_OWNER" && test -z "$extra" || return 1
  # Only the original declared private tmpfs and owned config volume are allowed.
  mounts=${mounts//tmpfs::\/tmp:true;/}
  test "$mounts" = "${SDK_SESSION_BINDINGS[$name]}" || return 1
  if test -n "${SDK_SESSION_IDS[$name]}"; then
    test "${SDK_SESSION_IDS[$name]}" = "$id|$image_id" || return 1
  else
    expected_image_id=$(sdk_owned_docker image inspect "$image" --format '{{.Id}}') || return 1
    test "$image_id" = "$expected_image_id" || return 1
    SDK_SESSION_IDS[$name]="$id|$image_id"
  fi
  SDK_SESSION_CONTAINER=$id
}

sdk_owned_container() {
  local role=$1 image=$2 volume=$3 name="$SDK_SESSION_OWNER-$1" observed
  shift 3
  [[ $role =~ ^[a-z][a-z0-9-]{0,40}$ && $image =~ @sha256:[0-9a-f]{64}$ ]] || return 1
  sdk_owned_absent container "$name" || return 1
  if test -n "$volume"; then
    test -v "SDK_SESSION_VOLUMES[$volume]" && sdk_owned_volume_identity "$volume" || return 1
  fi
  SDK_SESSION_IMAGES[$name]=$image
  SDK_SESSION_IDS[$name]=''
  SDK_SESSION_BINDINGS[$name]="${volume:+volume:$volume:/config:true;}"
  # Keep registration even when create fails after its daemon-side effect.
  observed=$(sdk_owned_docker container create --name "$name" --label "io.prismpm.sdk-session=$SDK_SESSION_OWNER" "$@") || return "$?"
  [[ $observed =~ ^[0-9a-f]{64}$ ]] || return 1
  sdk_owned_container_identity "$name" || return 1
  test "$observed" = "$SDK_SESSION_CONTAINER"
}

sdk_owned_retire_name() {
  local name=$1 id image_id state running oom exec_ids exit_code error absent
  if test "${SDK_SESSION_REMOVED[$name]:-}" = true; then return 0; fi
  if sdk_owned_absent container "$name"; then
    # A failed creation may have made no resource. Known IDs still need absence.
    if test -n "${SDK_SESSION_IDS[$name]}"; then
      id=${SDK_SESSION_IDS[$name]%%|*}
      absent=$(sdk_owned_docker container ls --all --no-trunc --filter "id=$id" --format '{{.ID}}') || return 1
      test -z "$absent" || return 1
    fi
    SDK_SESSION_REMOVED[$name]=true
    return 0
  fi
  sdk_owned_container_identity "$name" || return 1
  id=$SDK_SESSION_CONTAINER
  state=$(sdk_owned_docker container inspect "$id" --format '{{.State.Running}}|{{.State.OOMKilled}}|{{json .ExecIDs}}|{{.State.ExitCode}}|{{json .State.Error}}') || return 1
  [[ $state != *$'\n'* && $state != *$'\r'* ]] || return 1
  IFS='|' read -r running oom exec_ids exit_code error <<< "$state"
  { test "$running" = true || test "$running" = false; } &&
    { test "$exec_ids" = null || test "$exec_ids" = '[]'; } || return 1
  if test "$running" = true; then
    sdk_owned_docker container stop --time 5 "$id" >/dev/null || return 1
    sdk_owned_container_identity "$name" || return 1
    state=$(sdk_owned_docker container inspect "$id" --format '{{.State.Running}}|{{.State.OOMKilled}}|{{json .ExecIDs}}|{{.State.ExitCode}}|{{json .State.Error}}') || return 1
    [[ $state != *$'\n'* && $state != *$'\r'* ]] || return 1
    IFS='|' read -r running oom exec_ids exit_code error <<< "$state"
  fi
  test "$running" = false && { test "$exec_ids" = null || test "$exec_ids" = '[]'; } &&
    { test "$oom" = true || test "$oom" = false; } && [[ $exit_code =~ ^[0-9]+$ ]] &&
    [[ $error == \"*\" ]] || return 1
  sdk_owned_docker container rm "$id" >/dev/null || return 1
  absent=$(sdk_owned_docker container ls --all --no-trunc --filter "id=$id" --format '{{.ID}}') || return 1
  test -z "$absent" && sdk_owned_absent container "$name" || return 1
  SDK_SESSION_REMOVED[$name]=true
  printf '{"schema":"prismpm/ci-owned-retirement/1","kind":"container","id":"%s","name":"%s","exit_code":%s,"oom_killed":%s,"exec_ids":%s,"absent":true}\n' "$id" "$name" "$exit_code" "$oom" "$exec_ids" >&2
  test "$oom" = false && test "$error" = '""'
}

sdk_owned_retire_container() {
  local wanted=$1 name
  for name in "${!SDK_SESSION_IDS[@]}"; do
    if test "${SDK_SESSION_IDS[$name]%%|*}" = "$wanted"; then
      sdk_owned_retire_name "$name"
      return "$?"
    fi
  done
  return 1
}

sdk_owned_cleanup() {
  local failed=0 name observed
  for name in "${!SDK_SESSION_IDS[@]}"; do
    if ! sdk_owned_retire_name "$name"; then
      printf 'SDK cleanup uncertain; retained container identity: %s\n' "$name" >&2
      failed=1
    fi
  done
  for name in "${!SDK_SESSION_VOLUMES[@]}"; do
    if sdk_owned_absent volume "$name"; then continue; fi
    if ! sdk_owned_volume_identity "$name"; then
      printf 'SDK cleanup uncertain; retained volume identity: %s\n' "$name" >&2
      failed=1
      continue
    fi
    if ! sdk_owned_docker volume rm "$name" >/dev/null || ! sdk_owned_absent volume "$name"; then
      printf 'SDK owned volume retirement failed: %s\n' "$name" >&2
      failed=1
    else
      printf '{"schema":"prismpm/ci-owned-retirement/1","kind":"volume","name":"%s","absent":true}\n' "$name" >&2
    fi
  done
  return "$failed"
}
