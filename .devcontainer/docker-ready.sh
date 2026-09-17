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

# Refresh only the command's group credentials, retaining its non-root UID.
# sg accepts a shell command: quote every argument, including empty arguments,
# literal quotes and newlines, rather than interpreting caller-supplied text.
command='exec'
for argument in "$@"; do
  command+=" '${argument//\'/\'\\\'\'}'"
done
exec sg "$group_name" -c "$command"
