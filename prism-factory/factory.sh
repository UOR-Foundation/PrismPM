#!/usr/bin/env bash
# The Prism factory's entry point on a laptop, a runner, or any CI.
#
#   factory.sh plan|request|run|workflow|route-record [options]   the host CLI (see `prism-factory`)
#   factory.sh ci                     plan and run this host's obligations against the receipts branch
#   factory.sh actions-route          a release run's factory job: run host obligations, write routes
#   factory.sh actions-receipts       a release run's receipts job: record and push its receipts
#   factory.sh site DIR               build the GitHub Pages site into DIR
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
  # Adds this run's receipts to the branch's newest head, reweighs the schedule
  # by this run's outcomes, and pushes; another run may push first, so retry.
  local fresh=$1 attempt index tree commit parent blob receipt
  for attempt in 1 2 3 4 5; do
    index=$(mktemp)
    parent=()
    if git fetch --quiet --depth=1 origin "refs/heads/$branch" 2>/dev/null; then
      GIT_INDEX_FILE=$index git read-tree FETCH_HEAD
      parent=(-p "$(git rev-parse FETCH_HEAD)")
    else
      rm -f "$index"
    fi
    for receipt in "$fresh"/*.receipt; do
      [ -e "$receipt" ] || continue
      blob=$(git hash-object -w "$receipt")
      GIT_INDEX_FILE=$index git update-index --add --cacheinfo "100644,$blob,${receipt##*/}"
    done
    { GIT_INDEX_FILE=$index git cat-file blob :weights 2>/dev/null || true; cat "$fresh/outcomes" 2>/dev/null || true; } \
      | "$host" reweigh > "$fresh/weights"
    blob=$(git hash-object -w "$fresh/weights")
    GIT_INDEX_FILE=$index git update-index --add --cacheinfo "100644,$blob,weights"
    # The index is what the Pages site and a laptop read: receipt lines, then weights.
    GIT_INDEX_FILE=$index git ls-files -- '*.receipt' | while read -r receipt; do
      GIT_INDEX_FILE=$index git cat-file blob ":$receipt" | head -n 1
    done > "$fresh/index"
    cat "$fresh/weights" >> "$fresh/index"
    blob=$(git hash-object -w "$fresh/index")
    GIT_INDEX_FILE=$index git update-index --add --cacheinfo "100644,$blob,index"
    tree=$(GIT_INDEX_FILE=$index git write-tree)
    rm -f "$index"
    commit=$(git -c user.name=prism-factory -c user.email=prism-factory@users.noreply.github.com \
      commit-tree "$tree" "${parent[@]}" -m "Receipts of run ${GITHUB_RUN_ID:-local} for $(git rev-parse HEAD)")
    if git push --quiet origin "$commit:refs/heads/$branch"; then return 0; fi
    sleep $((attempt * 5))
  done
  echo "factory.sh: could not push receipts to $branch" >&2
  return 1
}

command=${1:-}
case "$command" in
  ci)
    # A pull request or a laptop: run this host's obligations against the
    # published receipts. Only release runs write receipts.
    build
    fetch_receipts
    status=0
    "$host" run --receipts "$store" --context mode=rehearsal --context "run=local-$$" || status=$?
    "$host" plan --receipts "$store" --context mode=rehearsal --context "run=local-$$" | grep -E '^(obligation|releasable) ' || true
    exit "$status"
    ;;
  actions-route)
    build
    fetch_receipts
    : "${FACTORY_MODE:?}" "${GITHUB_RUN_ID:?}" "${GITHUB_RUN_ATTEMPT:?}"
    "$host" run --receipts "$store" --only factory-model \
      --context "mode=$FACTORY_MODE" --context "run=$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
    exec "$host" actions-route --receipts "$store"
    ;;
  actions-receipts)
    build
    fresh=target/factory/run-receipts
    rm -rf "$fresh"; mkdir -p "$fresh"
    "$host" actions-receipts --receipts "$fresh"
    publish_receipts "$fresh"
    ;;
  site)
    out=${2:?usage: factory.sh site DIR}
    rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
    cargo rustc --quiet --release --locked --offline --lib --crate-type cdylib --target wasm32-unknown-unknown --manifest-path "$manifest"
    rm -rf "$out"; mkdir -p "$out"
    cp prism-factory/host/target/wasm32-unknown-unknown/release/prism_factory_host.wasm "$out/factory.wasm"
    cp prism-factory/site/index.html "$out/index.html"
    ;;
  plan|request|run|workflow|route-record)
    build
    shift
    exec "$host" "$command" "$@"
    ;;
  *)
    sed -n '2,9p' "$0" >&2
    exit 64
    ;;
esac
