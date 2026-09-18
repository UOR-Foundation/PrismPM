#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'devcontainer Docker initialization: %s\n' "$*" >&2
  exit 1
}

test "$#" -gt 0 || fail 'a command is required'
test "$(id -u)" -ne 0 || fail 'the command must run as the non-root remote user'
remote_user=$(id -un)
deadline=$((SECONDS + 30))

# Dev Containers may start its remote shell before the root entrypoint's
# usermod finishes. Query NSS for readiness, not this shell's stale groups.
while :; do
  test -S /var/run/docker.sock || fail 'the mounted Docker socket is missing'
  socket_gid=$(stat -c '%g' /var/run/docker.sock)
  case " $(id -G "$remote_user") " in
    *" $socket_gid "*) break ;;
  esac
  test "$SECONDS" -lt "$deadline" || fail 'timed out waiting for socket group membership'
  sleep 0.1
done

case " $(id -G) " in
  *" $socket_gid "*) exec "$@" ;;
esac

group_name=$(getent group "$socket_gid" | cut -d: -f1)
test -n "$group_name" || fail 'the socket group cannot be resolved'

# usermod updates group and shadow authorization separately. NSS visibility
# alone does not guarantee that sg can refresh an already-running shell.
# Probe authorization noninteractively within the same initialization deadline;
# never retry the caller's command or consume its standard input.
waiting_for_authorization=false
while :; do
  remaining=$((deadline - SECONDS))
  test "$remaining" -gt 0 || fail 'timed out waiting for socket group authorization'
  if timeout --signal=TERM --kill-after=1 "${remaining}s" \
    sg "$group_name" -c true </dev/null >/dev/null 2>&1; then
    break
  fi
  if ! "$waiting_for_authorization"; then
    printf 'devcontainer Docker initialization: waiting for socket group authorization\n' >&2
    waiting_for_authorization=true
  fi
  test "$SECONDS" -lt "$deadline" || fail 'timed out waiting for socket group authorization'
  sleep 0.1
done

# Refresh only the command's group credentials, retaining its non-root UID.
# sg accepts a shell command: quote every argument, including empty arguments,
# literal quotes and newlines, rather than interpreting caller-supplied text.
command='exec'
for argument in "$@"; do
  command+=" '${argument//\'/\'\\\'\'}'"
done
exec sg "$group_name" -c "$command"
