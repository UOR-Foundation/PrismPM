#!/bin/sh
# Print a writable Cargo home seeded from the SDK's immutable locked offline
# cache (/opt/prismpm/cargo-home) for the identity that will run Cargo.
#
# A root caller seeds for UID/GID 1000: the SDK entrypoint drops root to that
# identity before executing the requested command, and a root login shell can
# still write a 1000-owned home. Ephemeral homes (PRISMPM_EPHEMERAL_HOME=1) are
# private per final UID under TMPDIR; otherwise the home is /home/vscode/.cargo,
# which the full SDK ships already seeded and the build-only SDK ships empty.
# Idempotent: a seeded home is reused as is, and the seed is never written.
set -eu

seed=/opt/prismpm/cargo-home
uid=$(id -u)
owner=
if [ "$uid" = 0 ]; then
  uid=1000
  owner=1000:1000
fi

if [ "${PRISMPM_EPHEMERAL_HOME:-}" = 1 ]; then
  home=${TMPDIR:-/tmp}/prismpm-cargo-$uid
else
  home=/home/vscode/.cargo
fi

if [ ! -e "$home/.seeded" ] && [ ! -d "$home/registry" ]; then
  test -d "$seed"
  mkdir -p "$home"
  # Copy entries, not "$seed/.": the home directory may belong to another
  # identity (the image's 1000-owned, world-writable home), and its own
  # ownership and times must not be rewritten.
  for entry in "$seed"/* "$seed"/.[!.]* "$seed"/..?*; do
    if [ -e "$entry" ]; then
      cp -a "$entry" "$home/"
      chmod -R u+rwX "$home/${entry##*/}"
    fi
  done
  : > "$home/.seeded"
  if [ -n "$owner" ]; then
    chown -R "$owner" "$home"
  fi
fi

printf '%s\n' "$home"
