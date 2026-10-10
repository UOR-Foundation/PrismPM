#!/usr/bin/env bash
# Review-only pinned formatter output. Source checkout and remote stay untouched.
set -euo pipefail
test "$#" -eq 2 || { echo 'usage: binary-sdk-format.sh SOURCE_COMMIT NEW_EVIDENCE_DIR' >&2; exit 64; }
revision=$1
evidence=$2
image='ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21'
root=$(cd "$(dirname "$0")/.." && pwd -P)
[[ $revision =~ ^[0-9a-f]{40}$ ]] || exit 64
test "$(uname -m)" = x86_64
test "$(git -C "$root" rev-parse HEAD)" = "$revision"
test -z "$(git -C "$root" status --porcelain)"
test ! -e "$evidence" && test ! -L "$evidence"
mkdir "$evidence"
evidence=$(cd "$evidence" && pwd -P)
scratch=$(mktemp -d)
container=''
cleanup() {
  if test -n "$container"; then docker container rm --force "$container" >/dev/null 2>&1 || true; fi
  chmod -R u+rwX -- "$scratch"
  rm -r -- "$scratch"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
docker pull --platform linux/amd64 "$image"
docker image inspect "$image" > "$evidence/image.json"
node "$root/scripts/binary-sdk-check.mjs" format-image "$evidence/image.json" "$image"
mkdir "$scratch/source"
git -C "$root" archive "$revision" | tar -x -C "$scratch/source"
git -C "$scratch/source" init --quiet
git -C "$scratch/source" add --all --force
container=$(docker container create --user 1000:1000 --read-only --network none --cap-drop ALL \
  --security-opt no-new-privileges --tmpfs /tmp:rw,exec,nosuid,nodev,size=4g \
  --env PRISMPM_EPHEMERAL_HOME=1 --env CARGO_NET_OFFLINE=true \
  --workdir /tmp "$image" sleep infinity)
[[ $container =~ ^[0-9a-f]{64}$ ]] || exit 1
docker container start "$container" >/dev/null
docker exec "$container" mkdir /tmp/source
# tar is executed as the bounded SDK user, so imported bytes need no root chown.
git -C "$root" archive "$revision" | docker exec --interactive "$container" tar -x -C /tmp/source
docker exec "$container" /usr/local/bin/prismpm-devcontainer-init /bin/bash -euo pipefail -c '
  test "$(id -u)" = 1000
  cd /tmp/source
  rustfmt --version
  cargo --version
  cargo fmt --all
  cargo fmt --all --check
' > "$evidence/formatter.log" 2>&1
docker container cp "$container:/opt/prismpm/share/inventory.json" "$evidence/inventory.json"
docker container cp "$container:/tmp/source/." "$scratch/source/"
test -z "$(git -C "$scratch/source" ls-files --others)"
git -C "$scratch/source" diff --binary > "$evidence/format.patch"
git -C "$scratch/source" diff --name-only > "$evidence/changed-paths.txt"
node "$root/scripts/binary-sdk-check.mjs" format-evidence "$evidence" "$revision" "$image"
test "$(git -C "$root" rev-parse HEAD)" = "$revision"
test -z "$(git -C "$root" status --porcelain)"
# A nonempty review patch is an ordinary strict source-format failure. It cannot
# authorize image qualification until the reviewed bytes enter a new commit.
if test -s "$evidence/format.patch"; then
  echo 'Source formatting differs; review and import format.patch before qualification.' >&2
  exit 1
fi
printf 'Pinned source format check passed: source=%s image=%s\n' "$revision" "$image"
