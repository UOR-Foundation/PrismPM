#!/usr/bin/env bash
# Construction/resource diagnostic only: P(runtime + archive), or runtime A/B.
# This runner never publishes and never qualifies the six-image release family.
set -euo pipefail
umask 077
export LC_ALL=C LANG=C
test "$#" -ge 6 && test "$#" -le 7
phase=$1 arch=$2 revision=$3 work_end=$4 whole_end=$5 output=$6 expected=${7:-}
case "$phase:$arch" in production:amd64|production:arm64|pair:amd64|pair:arm64) ;; *) exit 2;; esac
[[ "$revision" =~ ^[0-9a-f]{40}$ && "$work_end" =~ ^[0-9]{13}$ && "$whole_end" =~ ^[0-9]{13}$ ]]
test "${GITHUB_ACTIONS:-}" = true
test "${GITHUB_REPOSITORY:-}" = UOR-Foundation/PrismPM
test "${GITHUB_EVENT_NAME:-}" = workflow_dispatch
test "${GITHUB_REF:-}" = refs/heads/fix/521-release-build-sets
test "${GITHUB_SHA:-}" = "$revision" && test "${GITHUB_WORKFLOW_SHA:-}" = "$revision"
[[ "${GITHUB_RUN_ID:-}" =~ ^[0-9]+$ && "${GITHUB_RUN_ATTEMPT:-}" =~ ^[1-9][0-9]*$ ]]
checkout=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
helper=$checkout/scripts/native-resource-diagnostic.mjs
monitor=$checkout/scripts/native-resource-monitor.py
image=moby/buildkit@sha256:de10faf919fc71ba4eb1dd7bd6449566d012b0c9436b1c61bfee21d621b009aa
store_bytes=42949672960
admission_bytes=57982058496
label_key=org.prismpm.native-resource-owner
sock=unix:///var/run/docker.sock
/usr/bin/python3 -I -B - "$output" "$expected" "$phase" "$work_end" "$whole_end" <<'PY'
import os,sys,time
out,expected,phase,work,whole=sys.argv[1:]; work=int(work);whole=int(whole)
assert int(time.time()*1000)<work<whole and whole-work==1800000
assert whole-int(time.time()*1000)<=21600000
assert os.path.isabs(out) and os.path.realpath(out)==out and not os.path.lexists(out)
assert os.path.isdir(os.path.dirname(out))
if phase=='pair':
 assert os.path.isabs(expected) and os.path.realpath(expected)==expected and os.path.isdir(expected)
else: assert not expected
PY
mkdir -- "$output"
scratch=$(mktemp -d "${RUNNER_TEMP:?}/native-resource-$phase-$arch.XXXXXXXX")
scratch_identity=$(stat --format='%d:%i' -- "$scratch")
store=$scratch/store backing=$scratch/store.ext4
mkdir -- "$store"
loop= mounted= owner_cid= owner_name= owner_builder= owner_directory= owner_identity= owner_plan= owner_finish= owner_stop= owner_samples= owner_snapshot= owner_terminal_sequence=0 outside_baseline_path= outside_baseline_sha= monitor_pid= watch_id= watch_result= watch_finish_deadline= controller_pid= controller_cancel= controller_id= controller_deadline=
cleanup_mode=0 cleanup_complete=0 complete=0 reason=incomplete command_sequence=0 supervision_lost=0 resource_failure=0
control=$output/.control
mkdir -- "$control"
capture=$control/stdout result=$control/command-result.json
# Persist the exact inherited global dispatch policy before any experiment IO.
node "$helper" admit "$revision" "$work_end" "$whole_end" "$output/dispatch-plan.json"
# Commands run through the existing private-FD Linux subreaper. The shell
# requests cancellation through an exact fresh receipt, never a numeric signal.
clock_ms() { /usr/bin/python3 -I -B -c 'import time;print(int(time.time()*1000))'; }
cancel_controller() {
  test -n "$controller_pid" && test -n "$controller_cancel" || return 0
  test ! -e "$controller_cancel" || return 0
  /usr/bin/python3 -I -B - "$controller_cancel" "$controller_id" "$revision" "$1" <<'PYTHON'
import json,os,sys
path,command,revision,reason=sys.argv[1:]
with open(path,'x') as f:json.dump(dict(schema='prismpm/native-resource-cancel/1',source_revision=revision,run_id=os.environ['GITHUB_RUN_ID'],run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],command_id=command,reason=reason),f)
PYTHON
}
retirement_proven() {
  /usr/bin/python3 -I -B - "$1" "$2" "$revision" <<'PYTHON'
import json,os,stat,sys
s=os.lstat(sys.argv[1]);assert stat.S_ISREG(s.st_mode) and s.st_nlink==1 and s.st_size<=1048576
r=json.load(open(sys.argv[1]));assert r['retirement']['descendants_absent'] is True and r['retirement']['uncertainty'] is None
assert r['command_id']==sys.argv[2] and r['source_revision']==sys.argv[3]
assert r['run_id']==os.environ['GITHUB_RUN_ID'] and r['run_attempt']==os.environ['GITHUB_RUN_ATTEMPT']
PYTHON
}
wait_controller() {
  local deadline=$1 guard=$2 status=0 failed=0 ticks=0 cancelled_at=0 now
  while kill -0 "$controller_pid" 2>/dev/null; do
    ticks=$((ticks + 1)); now=$(clock_ms)
    if test "$now" -ge "$deadline"; then cancel_controller deadline || :; cancelled_at=$now; failed=1; break; fi
    if test "$guard" = 1 && test -n "$monitor_pid"; then
      if test -e "$owner_stop" || ! kill -0 "$monitor_pid" 2>/dev/null || { test "$((ticks % 10))" = 0 && ! monitor_healthy; }; then
        cancel_controller resource-monitor-unavailable || :
        test "$cancelled_at" != 0 || cancelled_at=$now
        failed=1
      fi
    fi
    if test "$cancelled_at" != 0 && test "$((now - cancelled_at))" -ge 6000; then break; fi
    sleep 0.1
  done
  if kill -0 "$controller_pid" 2>/dev/null; then supervision_lost=1; return 1; fi
  wait "$controller_pid" || status=$?
  controller_pid=
  if ! retirement_proven "$result" "$controller_id"; then supervision_lost=1; return 1; fi
  test "$failed" = 0 || return 1
  return "$status"
}
monitor_healthy() {
  test -n "$monitor_pid" || return 0
  test ! -e "$owner_stop" && kill -0 "$monitor_pid" 2>/dev/null && test -s "$owner_samples" || return 1
  /usr/bin/python3 -I -B - "$owner_samples" <<'PYTHON'
import os,stat,sys,time
s=os.lstat(sys.argv[1]);assert stat.S_ISREG(s.st_mode) and s.st_nlink==1 and s.st_size<=64*1024**2
assert -1<=time.time()-s.st_mtime<=10
PYTHON
}
write_command_request() {
  /usr/bin/python3 -I -B - "$1" "$2" "$revision" "$3" "$4" "$5" "$6" "$7" "$8" "$9" "${@:10}" <<'PYTHON'
import json,os,sys
path,command,revision,deadline,maximum,log,capture,result,cancel,stream,*argv=sys.argv[1:]
with open(path,'x') as f:json.dump(dict(schema='prismpm/native-resource-command/1',source_revision=revision,run_id=os.environ['GITHUB_RUN_ID'],run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],command_id=command,deadline_ms=int(deadline),maximum_ms=int(maximum),log_path=log,capture_path=capture,result_path=result,cancel_file=cancel,stream_output=stream=='1',argv=argv),f)
PYTHON
}
run_limit() {
  local seconds=$1; shift
  local deadline=$work_end log=$output/commands.log guard=1
  if test "$cleanup_mode" = 1; then deadline=$whole_end; log=$output/cleanup.log; guard=0; else monitor_healthy || return 1; fi
  test -z "$controller_pid" || return 1
  command_sequence=$((command_sequence + 1)); controller_id=cmd-$command_sequence
  capture=$control/$controller_id.stdout result=$control/$controller_id.result.json
  controller_cancel=$control/$controller_id.cancel.json
  local request=$control/$controller_id.request.json
  write_command_request "$request" "$controller_id" "$deadline" "$((seconds * 1000))" "$log" "$capture" "$result" "$controller_cancel" "${stream_capture:-0}" "$@"
  node "$helper" command "$request" > /dev/null 2>&1 & controller_pid=$!
  controller_deadline=$deadline
  wait_controller "$deadline" "$guard"
}
run() { run_limit 0 "$@"; }
docker_cmd() { run docker --host "$sock" "$@"; }
stream() { local stream_capture=1; run "$@"; }
docker_stream() { stream docker --host "$sock" "$@"; }
identity() { stat --format='%d:%i' -- "$1"; }
verify_backing() {
  local actual
  actual=$(stat --format='%d:%i:%s:%b' -- "$backing") || return 1
  /usr/bin/python3 -I -B - "$actual" "$backing_identity" "$store_bytes" <<'PY' || return 1
import sys
actual,expected,size=sys.argv[1:];dev,ino,length,blocks=map(int,actual.split(':'))
assert f'{dev}:{ino}'==expected and length==int(size) and blocks*512>=int(size)
PY
  test ! -L "$backing"
}
verify_loop() {
  run sudo losetup --json --list --output NAME,BACK-FILE,OFFSET,SIZELIMIT -- "$loop" || return 1
  /usr/bin/python3 -I -B - "$capture" "$loop" "$backing" <<'PY'
import json,sys
rows=json.load(open(sys.argv[1]))['loopdevices'];assert len(rows)==1
r=rows[0];assert r['name']==sys.argv[2] and r['back-file']==sys.argv[3] and r['offset']==0 and r['sizelimit']==0
PY
}
verify_mount() {
  run findmnt --json --mountpoint "$store" --output TARGET,SOURCE,FSTYPE,OPTIONS || return 1
  /usr/bin/python3 -I -B - "$capture" "$store" "$loop" <<'PY'
import json,sys
rows=json.load(open(sys.argv[1]))['filesystems'];assert len(rows)==1
r=rows[0];opts=set(r['options'].split(','))
assert r['target']==sys.argv[2] and r['source']==sys.argv[3] and r['fstype']=='ext4'
assert {'nodev','nosuid'}.issubset(opts) and 'discard' not in opts
PY
}
make_plan() {
  local destination=$1
  /usr/bin/python3 -I -B - "$destination" "$phase" "$arch" "$revision" "$work_end" "$whole_end" "$output" "$store" "$backing" "$backing_identity" "$loop" "$docker_root" "$owner_name" "$owner_cid" "$owner_uuid" "$image_id" "$memory" "$owner_directory" "$BUILDX_CONFIG" "$owner_finish" "$outside_baseline_path" "$outside_baseline_sha" <<'PY'
import json,os,sys
(dest,phase,arch,sha,work,whole,out,store,backing,identity,loop,docker_root,name,cid,uuid,image,memory,directory,config,finish,baseline_path,baseline_sha)=sys.argv[1:]
dev,ino=map(int,identity.split(':'))
owner=None if not cid else dict(name=name,cid=cid,label_key='org.prismpm.native-resource-owner',label_value=uuid,image_id=image,memory_bytes=int(memory),state=directory+'/state',run=directory+'/run',config_root=config)
baseline=None if not baseline_path else dict(path=baseline_path,sha256=baseline_sha)
plan=dict(outside_baseline=baseline,schema='prismpm/native-resource-plan/1',phase=phase,arch=arch,source_revision=sha,run_id=os.environ['GITHUB_RUN_ID'],run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],work_end_ms=int(work),whole_end_ms=int(whole),output_root=out,store_root=store,backing=dict(path=backing,dev=dev,ino=ino,size=42949672960,blocks_min=42949672960),loop_device=loop,docker_root=docker_root,participating_paths=[store,store+'/tmp',store+'/roots',store+'/exports',store+'/builders'],owner=owner,finish_file=finish or out+'/store-finish.json')
with open(dest,'x') as f: json.dump(plan,f,separators=(',',':'))
PY
}
owned_container() {
  docker_cmd inspect --type container --format '{"Id":{{json .Id}},"Name":{{json .Name}},"Image":{{json .Image}},"Config":{"Image":{{json .Config.Image}},"Labels":{{json .Config.Labels}}},"HostConfig":{{json .HostConfig}},"Mounts":{{json .Mounts}}}' "$owner_cid" || return 1
  /usr/bin/python3 -I -B - "$capture" "$owner_cid" "$owner_name" "$owner_uuid" "$image_id" "$owner_directory" "$memory" "$control/image.json" <<'PY'
import json,sys
path,cid,name,uuid,image,directory,memory,image_path=sys.argv[1:];r=json.load(open(path));h=r['HostConfig'];c=r['Config']
assert r['Id']==cid and r['Name']=='/'+name and r['Image']==image
defaults=json.load(open(image_path))['Config']
assert c['Image']==image
labels=dict(defaults.get('Labels') or {});labels['org.prismpm.native-resource-owner']=uuid
assert c['Labels']==labels
assert h['Privileged'] is True and h['ReadonlyRootfs'] is True and h['Init'] is True
assert h['NanoCpus']==2000000000 and h['Memory']==int(memory) and h['MemorySwap']==int(memory) and h['PidsLimit']==4096
assert h['LogConfig']==dict(Type='local',Config={'max-size':'16m','max-file':'2'})
assert h['NetworkMode']=='bridge' and h['IpcMode'] in ('private','') and h['PidMode']=='' and h['CgroupParent']==''
for key in ('Binds','VolumesFrom','Devices','DeviceRequests','CapAdd','CapDrop'): assert not h.get(key)
mounts=r['Mounts'];assert len(mounts)==2
assert {(m['Type'],m['Source'],m['Destination'],m['RW']) for m in mounts}=={('bind',directory+'/state','/var/lib/buildkit',True),('bind',directory+'/run','/run',True)}
assert h['Tmpfs']=={'/tmp':'rw,noexec,nosuid,nodev,size=268435456'}
PY
}
finish_watch() {
  if test -n "$monitor_pid"; then
    if test ! -e "$owner_stop" && test ! -e "$owner_finish"; then
      /usr/bin/python3 -I -B - "$owner_finish" "$phase" "$revision" <<'PY'
import json,os,sys
with open(sys.argv[1],'x') as f: json.dump(dict(schema='prismpm/native-resource-finish/1',phase=sys.argv[2],source_revision=sys.argv[3],run_id=os.environ['GITHUB_RUN_ID'],run_attempt=os.environ['GITHUB_RUN_ATTEMPT']),f)
PY
    fi
    local status=0 now
    if test -z "$watch_finish_deadline"; then watch_finish_deadline=$(/usr/bin/python3 -I -B - "$whole_end" "$work_end" "$cleanup_mode" <<'PYTHON'
import sys,time
print(min(int(sys.argv[1] if sys.argv[3]=='1' else sys.argv[2]),int(time.time()*1000)+40000))
PYTHON
); fi
    while kill -0 "$monitor_pid" 2>/dev/null; do
      now=$(/usr/bin/python3 -I -B -c 'import time;print(int(time.time()*1000))')
      if test "$now" -ge "$watch_finish_deadline"; then supervision_lost=1; return 1; fi
      sleep 0.1
    done
    wait "$monitor_pid" || status=$?
    monitor_pid=
    if ! retirement_proven "$watch_result" "$watch_id"; then supervision_lost=1; return 1; fi
    test "$status" = 0 && test ! -e "$owner_stop"
  fi
}
retire_owner() {
  test -n "$owner_cid" || return 0
  local watch_status=0
  finish_watch || watch_status=1
  test "$supervision_lost" = 0 || return 1
  if test -n "$monitor_pid"; then
    owned_container || return 1
    docker_cmd stop --time 10 "$owner_cid" || return 1
    return 1
  fi
  # Capacity failure never authorizes deleting an uncertain or replaced owner.
  owned_container || return 1
  docker_cmd stop --time 10 "$owner_cid" || return 1
  owned_container || return 1
  test -n "$owner_snapshot" && test -f "$owner_snapshot" || return 1
  owner_terminal_sequence=$((owner_terminal_sequence + 1))
  local terminal=$output/$owner_builder-retirement-$owner_terminal_sequence.json
  run sudo -n /usr/bin/python3 -I -B "$monitor" retire "$owner_plan" "$owner_snapshot" "$terminal" || return 1
  local terminal_failure
  terminal_failure=$(/usr/bin/python3 -I -B - "$terminal" <<'PYTHON'
import json,sys
r=json.load(open(sys.argv[1]));assert r['complete'] is True
fields=('owner_oom_killed','resource_failure','resource_observation_complete')
unknown=any(type(r.get(key)) is not bool for key in fields)
print(1 if unknown or r.get('owner_oom_killed') is True or r.get('resource_failure') is True else 0)
PYTHON
) || return 1
  if test "$terminal_failure" = 1; then resource_failure=1; watch_status=1; fi
  docker_cmd rm "$owner_cid" || return 1
  docker_cmd container ls --all --no-trunc --filter "id=$owner_cid" --format '{{.ID}}' || return 1
  test ! -s "$capture" || return 1
  docker_cmd buildx rm --keep-state "$owner_builder" || return 1
  docker_cmd volume ls --format '{{.Name}}' || return 1
  ! grep -Fxq "${owner_name}_state" "$capture" || return 1
  test "$(identity "$owner_directory")" = "$owner_identity" && test ! -L "$owner_directory" || return 1
  run sudo rm -rf -- "$owner_directory" || return 1
  owner_cid= owner_name= owner_builder= owner_directory= owner_identity= owner_plan= owner_finish= owner_stop= owner_samples= owner_snapshot= watch_id= watch_result= watch_finish_deadline=
  test "$watch_status" = 0
}
cleanup() {
  local original=$? failed=0
  trap - EXIT TERM INT
  cleanup_mode=1
  if test -n "$controller_pid"; then
    cancel_controller interrupted || :
    local controller_wait
    controller_wait=$(/usr/bin/python3 -I -B -c 'import sys,time;print(min(int(sys.argv[1]),int(time.time()*1000)+6000))' "$whole_end")
    wait_controller "$controller_wait" 0 || original=1
  fi
  if test "$supervision_lost" = 1; then
    failed=1
    if test -n "$monitor_pid"; then finish_watch || :; fi
  fi
  if test "$failed" = 0 && test -n "$owner_cid"; then
    if ! retire_owner; then original=1; test -z "$owner_cid" || failed=1; fi
  fi
  if test -n "$owner_cid"; then failed=1; fi
  if test "$failed" = 0 && test "$mounted" = 1; then
    if verify_backing && verify_loop && verify_mount; then
      if run sudo umount -- "$store"; then
        mounted=
        if run findmnt --noheadings --mountpoint "$store"; then failed=1; else test "$(/usr/bin/python3 -I -B -c 'import json,sys;print(json.load(open(sys.argv[1]))["status"])' "$result")" = 1 || failed=1; fi
      else failed=1; fi
    else failed=1; fi
  fi
  if test "$failed" = 0 && test -n "$loop"; then
    if verify_backing && verify_loop && run sudo losetup --detach "$loop"; then
      if run sudo losetup --json --list --output NAME,BACK-FILE -- "$loop"; then
        /usr/bin/python3 -I -B - "$capture" <<'PY' || failed=1
import json,sys
assert json.load(open(sys.argv[1]))['loopdevices']==[]
PY
      else failed=1; fi
      loop=
    else failed=1; fi
  fi
  if test "$failed" = 0 && test -e "$backing"; then
    if verify_backing; then run rm -- "$backing" || failed=1; else failed=1; fi
  fi
  if test "$failed" = 0 && test -z "$loop" && test -z "$mounted" && test -z "$owner_cid"; then
    if test "$(identity "$scratch")" = "$scratch_identity" && test ! -L "$scratch"; then
      if run rm -rf -- "$scratch"; then cleanup_complete=1; else failed=1; fi
    else failed=1; fi
  fi
  if test "$original" != 0; then reason=command-or-resource-failure; fi
  if test "$failed" != 0; then reason=cleanup-incomplete; fi
  if test "$original" != 0 || test "$failed" != 0 || test "$resource_failure" != 0; then complete=0; fi
  /usr/bin/python3 -I -B - "$output/result.json" "$phase" "$arch" "$revision" "$complete" "$cleanup_complete" "$reason" "$resource_failure" <<'PY'
import json,os,sys
path,phase,arch,sha,complete,cleanup,reason,resource_failure=sys.argv[1:]
with open(path,'x') as f: json.dump(dict(schema='prismpm/native-resource-result/1',phase=phase,arch=arch,source_revision=sha,run_id=os.environ['GITHUB_RUN_ID'],run_attempt=os.environ['GITHUB_RUN_ATTEMPT'],complete=complete=='1' and cleanup=='1',cleanup_complete=cleanup=='1',resource_failure=resource_failure=='1',reason=reason,scope='runtime-and-cli-production-plus-independent-runtime-pair;not-six-family-or-registry-fit'),f)
PY
  if test "$original" != 0 || test "$failed" != 0 || test "$complete" != 1; then exit 1; fi
}
trap cleanup EXIT
trap 'reason=interrupted; exit 1' TERM INT
run git -C "$checkout" rev-parse HEAD; test "$(cat "$capture")" = "$revision"
run git -C "$checkout" status --porcelain --untracked-files=normal; test ! -s "$capture"
run uname -m; case "$arch:$(cat "$capture")" in amd64:x86_64|arm64:aarch64) ;; *) exit 1;; esac
docker_cmd buildx version; grep -Eq '^github.com/docker/buildx v0\.28\.0([[:space:]]|$)' "$capture"
# Pull immutable native image content before admission/baseline. No later pull
# may consume unaccounted host storage.
docker_cmd pull "$image"
docker_cmd image inspect "$image"
image_id=$(/usr/bin/python3 -I -B - "$capture" "$image" "$arch" "$control/image.json" <<'PY'
import json,re,sys
rows=json.load(open(sys.argv[1]));assert len(rows)==1
r=rows[0];assert sys.argv[2] in r['RepoDigests'] and r['Architecture']==sys.argv[3] and r['Os']=='linux'
assert re.fullmatch('sha256:[0-9a-f]{64}',r['Id'])
with open(sys.argv[4],'x') as f:json.dump(r,f)
print(r['Id'])
PY
)
docker_cmd info --format '{{.DockerRootDir}}'; docker_root=$(cat "$capture")
[[ "$docker_root" = /* ]] && test -d "$docker_root" && test ! -L "$docker_root"
run df --block-size=1 --output=avail -- "$scratch"
available=$(tail -n 1 "$capture" | tr -d ' '); [[ "$available" =~ ^[0-9]+$ ]] && test "$available" -ge "$admission_bytes"
(set -o noclobber; : > "$backing")
backing_identity=$(identity "$backing")
run fallocate --length "$store_bytes" -- "$backing"
verify_backing
run sudo mkfs.ext4 -F -E nodiscard,lazy_itable_init=0,lazy_journal_init=0 -- "$backing"
verify_backing
run sudo losetup --find --show --nooverlap -- "$backing"; loop=$(cat "$capture"); [[ "$loop" =~ ^/dev/loop[0-9]+$ ]]
verify_loop
run sudo mount -t ext4 -o nodiscard,nodev,nosuid -- "$loop" "$store"; mounted=1
verify_mount; verify_backing
run sudo chown "$(id -u):$(id -g)" -- "$store"
mkdir -- "$store/tmp" "$store/roots" "$store/exports" "$store/builders"
# Preserve inherited Docker credential/proxy selectors; only owned mutable
# Buildx configuration and temporary data move into the bounded filesystem.
export TMPDIR=$store/tmp
BUILDX_CONFIG=$store/builders
export BUILDX_CONFIG
memory=$(/usr/bin/python3 -I -B - <<'PY'
values={r.split(':')[0]:int(r.split()[1])*1024 for r in open('/proc/meminfo') if ':' in r and r.split()[1].isdigit()}
assert values['MemTotal']>3*1024**3
print(min(12*1024**3,values['MemTotal']-2*1024**3))
PY
)
owner_uuid= owner_finish=
make_plan "$output/baseline-plan.json"
run sudo -n /usr/bin/python3 -I -B "$monitor" baseline "$output/baseline-plan.json" "$output/outside-baseline.json"
outside_baseline_path=$output/outside-baseline.json
run sha256sum -- "$outside_baseline_path"
outside_baseline_sha=$(/usr/bin/python3 -I -B - "$capture" <<'PYTHON'
import re,sys
value=open(sys.argv[1]).read().split()[0];assert re.fullmatch('[0-9a-f]{64}',value);print(value)
PYTHON
)
make_plan "$output/store-plan.json"
run sudo -n /usr/bin/python3 -I -B "$monitor" snapshot "$output/store-plan.json" "$output/store-before.json"
# The deliberately oversized allocation must fail with ENOSPC, never timeout.
run df --block-size=1 --output=avail -- "$store"; before=$(tail -n 1 "$capture" | tr -d ' '); [[ "$before" =~ ^[0-9]+$ ]] && test "$before" -ge 1073741824 && test "$before" -le "$store_bytes"
probe=$store/.exclusive-exhaustion-probe
(set -o noclobber; : > "$probe"); probe_identity=$(identity "$probe")
if run_limit 30 fallocate --length "$((before + 1073741824))" -- "$probe"; then exit 1; fi
/usr/bin/python3 -I -B - "$result" <<'PY'
import json,sys
r=json.load(open(sys.argv[1]));assert r['status']==1 and r['reason'] is None and ('No space left on device' in r['stderr'] or 'ENOSPC' in r['stderr'])
PY
test "$(identity "$probe")" = "$probe_identity" && test ! -L "$probe"; rm -- "$probe"
run sync --file-system -- "$store"
verify_backing; verify_loop; verify_mount
run df --block-size=1 --output=avail -- "$store"; after=$(tail -n 1 "$capture" | tr -d ' '); [[ "$after" =~ ^[0-9]+$ ]] && test "$after" -ge "$before"
run sudo -n /usr/bin/python3 -I -B "$monitor" snapshot "$output/store-plan.json" "$output/store-after-probe.json"
new_owner() {
  local purpose=$1
  owner_builder="nr-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT-$arch-$phase-$purpose"
  owner_name="buildx_buildkit_${owner_builder}0"
  docker_cmd container ls --all --no-trunc --filter "name=^/${owner_name}$" --format '{{.ID}}'; test ! -s "$capture"
  docker_cmd volume ls --format '{{.Name}}'; ! grep -Fxq "${owner_name}_state" "$capture"
  owner_directory=$store/builders/$purpose
  mkdir -- "$owner_directory" "$owner_directory/state" "$owner_directory/run" "$owner_directory/config"
  owner_identity=$(identity "$owner_directory")
  BUILDX_CONFIG=$owner_directory/config
  owner_uuid=$(cat /proc/sys/kernel/random/uuid)
  owner_finish=$output/$purpose-finish.json owner_stop=$output/$purpose-stop.json owner_plan=$output/$purpose-plan.json
  docker_cmd create --pull=never --name "$owner_name" --cidfile "$owner_directory/cid" --label "$label_key=$owner_uuid" \
    --privileged --init --read-only --cpus 2 --memory "$memory" --memory-swap "$memory" --pids-limit 4096 \
    --log-driver local --log-opt max-size=16m --log-opt max-file=2 \
    --mount "type=bind,src=$owner_directory/state,dst=/var/lib/buildkit" \
    --mount "type=bind,src=$owner_directory/run,dst=/run" --tmpfs /tmp:rw,noexec,nosuid,nodev,size=268435456 "$image_id"
  owner_cid=$(cat "$owner_directory/cid"); [[ "$owner_cid" =~ ^[0-9a-f]{64}$ ]] && test "$(cat "$capture")" = "$owner_cid"
  owned_container
  docker_cmd start "$owner_cid"
  make_plan "$owner_plan"
  owner_snapshot=$output/$purpose-before-bootstrap.json
  run sudo -n /usr/bin/python3 -I -B "$monitor" snapshot "$owner_plan" "$owner_snapshot"
  owner_samples=$output/$purpose-samples.jsonl
  watch_id=watch-$purpose
  local watch_request=$control/$watch_id.request.json
  watch_result=$control/$watch_id.result.json
  watch_finish_deadline=
  # The quiet monitor owns cleanup.log only until its private-FD controller
  # proves all scanner descendants absent. Cleanup commands start afterwards.
  write_command_request "$watch_request" "$watch_id" "$work_end" 0 "$output/cleanup.log" "$control/$watch_id.stdout" "$watch_result" "$control/$watch_id.cancel.json" 0 sudo -n /usr/bin/python3 -I -B "$monitor" watch "$owner_plan" "$owner_samples" "$owner_stop"
  node "$helper" command "$watch_request" > /dev/null 2>&1 & monitor_pid=$!
  for _ in {1..50}; do
    test ! -e "$owner_stop" && kill -0 "$monitor_pid" 2>/dev/null || return 1
    test ! -s "$owner_samples" || break
    sleep 0.1
  done
  monitor_healthy
  docker_cmd buildx create --name "$owner_builder" --driver docker-container \
    --driver-opt "image=$image,memory=$memory,memory-swap=$memory,cpu-period=100000,cpu-quota=200000"
  docker_cmd buildx inspect --bootstrap "$owner_builder"
  owned_container
  run sudo -n /usr/bin/python3 -I -B "$monitor" snapshot "$owner_plan" "$output/$purpose-after-bootstrap.json"
  /usr/bin/python3 -I -B - "$output/$purpose-before-bootstrap.json" "$output/$purpose-after-bootstrap.json" <<'PYTHON'
import json,sys
before,after=[json.load(open(path)) for path in sys.argv[1:]]
assert before['complete'] is True and after['complete'] is True
fields=('cid','pid','pid_start_ticks','cgroup_path','cgroup_dev','cgroup_ino','config_sha256','namespace_sha256')
assert {key:before['fast']['owner'][key] for key in fields}=={key:after['fast']['owner'][key] for key in fields}
assert before['fast']['path_identities']==after['fast']['path_identities']
assert before['fast']['owner']['bind_identities']==after['fast']['owner']['bind_identities']
PYTHON
  owner_snapshot=$output/$purpose-after-bootstrap.json
  docker_cmd exec "$owner_cid" buildkitd --version; grep -Eq 'buildkitd github.com/moby/buildkit v0\.26\.2([[:space:]]|$)' "$capture"
}
clone_root() {
  local destination=$1
  run git -c core.hooksPath=/dev/null clone --no-hardlinks --no-checkout -- "$checkout" "$destination"
  run git -C "$destination" -c core.hooksPath=/dev/null checkout --detach "$revision"
  run git -C "$destination" rev-parse HEAD; test "$(cat "$capture")" = "$revision"
}
# Tiny disposable export proves the exact readonly/cgroup/mount arrangement.
new_owner preflight
mkdir -- "$store/preflight"; printf 'FROM scratch\nCOPY proof /proof\n' > "$store/preflight/Dockerfile"
printf 'bounded-native-resource-preflight\n' > "$store/preflight/proof"
docker_stream buildx build --builder "$owner_builder" --no-cache --provenance=false --sbom=false \
  --output "type=local,dest=$store/exports/preflight" "$store/preflight"
cmp -- "$store/preflight/proof" "$store/exports/preflight/proof"
retire_owner
if test "$phase" = production; then
  clone_root "$store/roots/root-p"
  new_owner production
  run node "$helper" production-plan "$revision" "$arch" "$owner_builder" "$store/exports/production"
  plan=$(cat "$capture")
  stream node "$store/roots/root-p/scripts/sdk-image-inputs.mjs" build-set "$store/roots/root-p" "$revision" "$plan"
  run node "$helper" metadata "$store/exports/production/runtime.oci" "$arch" "$revision" "$output/production-$arch.json"
  run node "$helper" archive "$store/exports/production/cli" "$revision" "$arch" "$output/archive-$arch.json"
  retire_owner
else
  for root in a b; do
    clone_root "$store/roots/root-$root"
    new_owner "root-$root"
    stream node "$store/roots/root-$root/scripts/sdk-image-inputs.mjs" build "$store/roots/root-$root" "$revision" runtime \
      --builder "$owner_builder" --no-cache --platform "linux/$arch" --provenance=false --sbom=false --build-arg SOURCE_DATE_EPOCH=0 \
      --label org.opencontainers.image.created=1970-01-01T00:00:00Z --label "org.opencontainers.image.revision=$revision" \
      --label org.opencontainers.image.source=https://github.com/UOR-Foundation/PrismPM --label org.opencontainers.image.version=0.4.0 \
      --output "type=oci,tar=false,dest=$store/exports/root-$root.oci,rewrite-timestamp=true,oci-mediatypes=true"
    retire_owner
  done
  # Both complete raw directories remain present until the unchanged verifier
  # has checked their actual native production subject and current custody.
  run node "$helper" verify "$expected" "$arch" "$revision" "$store/exports/root-a.oci" "$store/exports/root-b.oci" "$output/pair-$arch.json"
fi
run sudo -n /usr/bin/python3 -I -B "$monitor" snapshot "$output/store-plan.json" "$output/store-final.json"
complete=1 reason=completed
