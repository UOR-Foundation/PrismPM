#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# The external-oracle corpus must execute in the SDK image, including on the
# first clean source gate before a release image exists. Build an exact local
# image, publish it to an isolated local registry, and pass its distribution
# manifest digest to the tests. A Docker config image ID is not an OCI
# distribution identity. A caller may instead supply an already verified
# digest (for example, the released image in a consumer repository).
if test -z "${PRISMPM_TEST_SDK_IMAGE:-}"; then
  zot_image='ghcr.io/project-zot/zot@sha256:cd2aea942f428630bcb4190542be6abd35e14177aab84fc7ccad0dca8ecb363d'
  nonce="$$-${RANDOM}"
  registry="prismpm-vv-registry-${nonce}"
  config_volume="prismpm-vv-registry-config-${nonce}"
  scratch=$(mktemp -d)
  sdk_tag=''

  cleanup_registry() {
    docker container rm --force "$registry" >/dev/null 2>&1 || true
    docker volume rm --force "$config_volume" >/dev/null 2>&1 || true
    if test -n "$sdk_tag"; then
      docker image rm "$sdk_tag" >/dev/null 2>&1 || true
    fi
    rm -r -- "$scratch"
  }
  trap cleanup_registry EXIT

  printf '%s\n' '{"distSpecVersion":"1.1.1","http":{"address":"0.0.0.0","port":5000},"log":{"level":"warn"},"storage":{"rootDirectory":"/tmp/zot"}}' >"$scratch/config.json"
  docker volume create "$config_volume" >/dev/null
  docker container create \
    --name "$registry" \
    --publish 127.0.0.1::5000 \
    --read-only \
    --tmpfs /tmp:rw,nosuid,nodev \
    --volume "$config_volume:/config" \
    "$zot_image" \
    serve /config/config.json >/dev/null
  docker container cp "$scratch/config.json" "$registry:/config/config.json"
  docker container start "$registry" >/dev/null

  endpoint=$(docker container port "$registry" 5000/tcp | sed -n '1p')
  case "$endpoint" in
    127.0.0.1:[0-9]*) ;;
    *) printf 'SDK gate registry returned an invalid endpoint: %s\n' "$endpoint" >&2; exit 1 ;;
  esac

  docker build \
    --build-arg SOURCE_DATE_EPOCH=0 \
    --file sdk/Dockerfile \
    --target runtime \
    --tag prismpm-vv-sdk:gate \
    .

  sdk_tag="$endpoint/prismpm-vv-sdk:gate"
  docker image tag prismpm-vv-sdk:gate "$sdk_tag"
  for attempt in $(seq 1 20); do
    if docker image push "$sdk_tag"; then
      break
    fi
    if test "$attempt" -eq 20; then
      printf 'SDK gate registry was not ready after %s attempts\n' "$attempt" >&2
      exit 1
    fi
    sleep 0.25
  done

  sdk_reference=$(docker image inspect "$sdk_tag" --format '{{range .RepoDigests}}{{println .}}{{end}}' \
    | awk -v prefix="$endpoint/prismpm-vv-sdk@sha256:" 'index($0, prefix) == 1 { print; exit }')
  case "$sdk_reference" in
    "$endpoint"/prismpm-vv-sdk@sha256:????????????????????????????????????????????????????????????????) ;;
    *) printf 'SDK gate push returned an invalid manifest reference: %s\n' "$sdk_reference" >&2; exit 1 ;;
  esac
  export PRISMPM_TEST_SDK_IMAGE="$sdk_reference"
fi

cargo xtask vv
