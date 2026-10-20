#!/bin/sh
set -eu

# Generated Lean modules require more than the common 8 MiB process default.
# Keep this resource bound explicit and inherited by sanitized child processes;
# changing only compiler environment variables would be lost at that boundary.
ulimit -S -s 65536

# This image intentionally starts as root only long enough to adapt an optional
# host Docker socket.  Ensure the environment follows the permanent SDK user
# before any cache setup or requested command executes.
if [ "$(id -u)" = 0 ] || [ "$(id -u)" = 1000 ]; then
  export HOME=/home/vscode
  export USER=vscode
  export LOGNAME=vscode
fi

# GitHub runners can have any numeric uid. Give those short-lived action
# containers a private writable Cargo cache, seeded from the SDK's immutable
# locked cache, instead of relying on permissions in /home/vscode.
if [ "${PRISMPM_EPHEMERAL_HOME:-}" = 1 ]; then
  cache_root=${TMPDIR:-/tmp}/prismpm-cargo-${UID:-$(id -u)}
  if [ ! -e "$cache_root/.seeded" ]; then
    mkdir -p "$cache_root"
    cp -a /opt/prismpm/cargo-home/. "$cache_root/"
    chmod -R u+rwX "$cache_root"
    : > "$cache_root/.seeded"
  fi
  export CARGO_HOME=$cache_root

  # Private ephemeral defaults leave explicit caller paths and the exec handoff intact.
  if [ -z "${XDG_CACHE_HOME:-}" ] || [ -z "${XDG_CONFIG_HOME:-}" ] || [ -z "${XDG_RUNTIME_DIR:-}" ]; then
    xdg_parent=$(readlink -f -- "${TMPDIR:-/tmp}")
    test -d "$xdg_parent"
    xdg_root=$(umask 077; mktemp -d "$xdg_parent/prismpm-xdg.XXXXXXXXXX")
    chmod 0700 "$xdg_root"
    install -d -m 0700 "$xdg_root/cache" "$xdg_root/config" "$xdg_root/runtime"
    if [ "$(id -u)" = 0 ]; then
      chown 1000:1000 "$xdg_root/cache" "$xdg_root/config" "$xdg_root/runtime" "$xdg_root"
    fi
    export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$xdg_root/cache}"
    export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$xdg_root/config}"
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-$xdg_root/runtime}"
  fi
fi

if [ "$(id -u)" = 0 ] && [ -S /var/run/docker.sock ]; then
  socket_gid=$(stat -c '%g' /var/run/docker.sock)
  group_name=$(getent group "$socket_gid" | cut -d: -f1 || true)
  if [ -z "$group_name" ]; then
    group_name=host-docker
    /usr/sbin/groupadd --gid "$socket_gid" "$group_name"
  fi
  /usr/sbin/usermod --append --groups "$group_name" vscode
fi

if [ "$#" -gt 0 ]; then
  if [ "$(id -u)" = 0 ]; then
    exec setpriv --reuid=1000 --regid=1000 --init-groups "$@"
  fi
  exec "$@"
fi
if [ "$(id -u)" = 0 ]; then
  exec setpriv --reuid=1000 --regid=1000 --init-groups sleep infinity
fi
exec sleep infinity
