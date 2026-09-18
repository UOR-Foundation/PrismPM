#!/bin/sh
set -eu
if [ -n "${NODE_OPTIONS:-}" ] || [ -n "${NODE_PATH:-}" ]; then
  printf '%s\n' 'AsyncAPI runtime rejected: ambient module configuration is forbidden' >&2
  exit 1
fi
printf '%s  %s\n' '87f962245c8ad4fa95a972f8f67ed9804df22ee385197ae403dafae8b375995e' \
  '/opt/prismpm/asyncapi-official/scripts/launcher.mjs' | /usr/bin/sha256sum --check --strict >/dev/null
exec /usr/local/bin/node /opt/prismpm/asyncapi-official/scripts/launcher.mjs "$@"
