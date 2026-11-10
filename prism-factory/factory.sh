#!/usr/bin/env bash
# The Prism factory's entry point on a laptop, a runner, or any CI.
#
#   factory.sh plan|request|run|workflow [options]   the host CLI (see `prism-factory`)
#   factory.sh ci                                    plan and run against the receipts branch
#   factory.sh site DIR                              build the GitHub Pages site into DIR
#
# Decisions are made by the certified core in prism-factory/certified; this
# script only builds the host and moves receipts.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
manifest=prism-factory/host/Cargo.toml
host=prism-factory/host/target/release/prism-factory
store=target/factory/receipts
branch=${FACTORY_RECEIPTS_BRANCH:-factory-receipts}

build() { cargo build --quiet --release --locked --offline --manifest-path "$manifest"; }

fetch_receipts() {
  rm -rf "$store"; mkdir -p "$store"
  if git fetch --quiet --depth=1 origin "refs/heads/$branch" 2>/dev/null; then
    git archive FETCH_HEAD | tar -x -C "$store"
  fi
}

publish_receipts() {
  # One commit holding every receipt so far (the fetched ones plus this run's).
  # The index is what the Pages site and a laptop read: receipt lines, then weights.
  { cat "$store"/*.receipt 2>/dev/null | grep '^receipt ' || true; cat "$store/weights" 2>/dev/null || true; } > "$store/index"
  local scratch; scratch=$(mktemp -d)
  cp -R "$store"/. "$scratch"/
  git -C "$scratch" init --quiet
  git -C "$scratch" add -A
  git -C "$scratch" -c user.name=prism-factory -c user.email=prism-factory@users.noreply.github.com \
    commit --quiet -m "Receipts for $(git rev-parse HEAD)"
  git fetch --quiet "$scratch" HEAD
  git push --quiet --force origin "FETCH_HEAD:refs/heads/$branch"
  rm -rf "$scratch"
}

command=${1:-}
case "$command" in
  ci)
    build
    fetch_receipts
    status=0
    "$host" run --receipts "$store" || status=$?
    if [ "${FACTORY_PUBLISH:-false}" = true ]; then publish_receipts; fi
    "$host" plan --receipts "$store" | grep -E '^(obligation|releasable) ' || true
    exit "$status"
    ;;
  site)
    out=${2:?usage: factory.sh site DIR}
    rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
    cargo rustc --quiet --release --locked --offline --lib --crate-type cdylib --target wasm32-unknown-unknown --manifest-path "$manifest"
    rm -rf "$out"; mkdir -p "$out"
    cp prism-factory/host/target/wasm32-unknown-unknown/release/prism_factory_host.wasm "$out/factory.wasm"
    cp prism-factory/site/index.html "$out/index.html"
    ;;
  plan|request|run|workflow)
    build
    shift
    exec "$host" "$command" "$@"
    ;;
  *)
    sed -n '2,9p' "$0" >&2
    exit 64
    ;;
esac
