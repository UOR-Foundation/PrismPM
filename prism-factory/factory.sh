#!/usr/bin/env bash
# The Prism factory's entry point on a laptop, a runner, or any CI.
#
#   factory.sh plan|request|run|workflow|route-record [options]   the host CLI (see `prism-factory`)
#   factory.sh ci                     plan and run this host's obligations against the receipts branch
#   factory.sh actions-route          a release run's factory job: run host obligations, write routes
#   factory.sh actions-receipts       a release run's receipts job: record and push its receipts
#   factory.sh actions-cut            the release cut: tag vVERSION and dispatch its publication run
#                                     when the certified plan says the rehearsed commit is cuttable
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
  actions-cut)
    # The certified plan decides; this only reads GitHub and writes the tag.
    build
    fetch_receipts
    : "${CUT_REVISION:?}" "${GITHUB_REPOSITORY:?}"
    repo=$GITHUB_REPOSITORY
    version=$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version = "\([^"]*\)"$/\1/p' Cargo.toml)
    [[ $version =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] || { echo "factory: workspace version '$version'" >&2; exit 65; }
    tag=v$version minor="v${BASH_REMATCH[1]}.${BASH_REMATCH[2]}" patch=${BASH_REMATCH[3]}
    test "$(git rev-parse HEAD)" = "$CUT_REVISION"
    if gh api "repos/$repo/git/ref/tags/$tag" >/dev/null 2>&1; then
      echo "factory: $tag already exists; nothing to cut"
      exit 0
    fi
    plan=$("$host" plan --rev "$CUT_REVISION" --receipts "$store" --context mode=publish --context run=cut)
    grep -E '^(obligation|cuttable) ' <<<"$plan"
    if ! grep -qx 'cuttable yes' <<<"$plan"; then
      echo "factory: $CUT_REVISION is not cuttable: a run-independent obligation has no passing receipt"
      exit 0
    fi
    # A minor release ships when its milestone's planned work is done.
    milestone=
    if [ "$patch" = 0 ]; then
      milestone=$(gh api "repos/$repo/milestones?state=open&per_page=100" \
        --jq ".[] | select(.title | split(\" \") | index(\"$minor\")) | \"\(.number) \(.open_issues)\"")
      if [ -z "$milestone" ] || [ "$(wc -l <<<"$milestone")" != 1 ]; then
        echo "factory: no single open milestone names $minor; not cutting $tag"
        exit 0
      fi
      if [ "${milestone#* }" != 0 ]; then
        echo "factory: milestone $minor has ${milestone#* } open issues; not cutting $tag"
        exit 0
      fi
    fi
    object=$(gh api "repos/$repo/git/tags" -f tag="$tag" -f message="PrismPM $version" \
      -f object="$CUT_REVISION" -f type=commit --jq .sha)
    gh api "repos/$repo/git/refs" -f ref="refs/tags/$tag" -f sha="$object" >/dev/null
    echo "factory: tagged $tag at $CUT_REVISION"
    # The release body is the version's changelog entry, as release-please would
    # write it; the publication run appends RELEASE_NOTES.md and the assets.
    notes=$(mktemp)
    node --input-type=module -e "import {readFileSync} from 'node:fs'; import {entryFor} from './sdk/docs/lib/changelog.mjs';
      const body = entryFor(readFileSync('CHANGELOG.md', 'utf8'), process.argv[1]); if (!body) process.exit(65);
      process.stdout.write(body);" "$version" > "$notes"
    gh release create "$tag" --verify-tag --repo "$repo" --title "$tag" --notes-file "$notes"
    gh workflow run release.yml --repo "$repo" --ref "$tag" \
      -f version="$version" -f rehearsal=false -f publish-crates=true
    echo "factory: dispatched the publication run of $tag"
    if [ -n "$milestone" ]; then
      gh api -X PATCH "repos/$repo/milestones/${milestone%% *}" -f state=closed >/dev/null
      echo "factory: closed milestone $minor"
    fi
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
    sed -n '2,11p' "$0" >&2
    exit 64
    ;;
esac
