#!/usr/bin/env bash
# Host prerequisite only; this does not construct or qualify an SDK.
set -euo pipefail
umask 077
unset NODE_PATH NODE_OPTIONS
[[ $# == 3 ]] || { echo "usage: $0 SOURCE_REVISION TARGET_DIRECTORY EVIDENCE_DIRECTORY" >&2; exit 64; }
revision=$1
target=$2
evidence=$3
[[ $revision =~ ^[0-9a-f]{40}$ && $target == /* && $evidence == /* ]] || exit 64
mkdir -p "$evidence"
rm -f "$evidence/semantic-source-path.txt" "$evidence/node-oracles-path.txt" "$evidence/SHA256SUMS"
exec > "$evidence/bootstrap.log" 2>&1
finish() {
  local status=$?
  if test "$status" != 0; then
    rm -f "$evidence/semantic-source-path.txt" "$evidence/node-oracles-path.txt" "$evidence/SHA256SUMS"
  fi
  printf '%s\n' "$status" > "$evidence/bootstrap-exit-status.txt"
}
trap finish EXIT
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$root"
printf '%s\n' "$revision" > "$evidence/selected-source.txt"
git rev-parse HEAD > "$evidence/source-revision.txt"
git rev-parse 'HEAD^{tree}' > "$evidence/source-tree.txt"
test "$(cat "$evidence/source-revision.txt")" = "$revision"
source_status=$(git status --porcelain)
test -z "$source_status"
cp Cargo.lock "$evidence/Cargo.lock"
cp rust-toolchain.toml "$evidence/rust-toolchain.toml"
python3 - <<'PYTHON'
import pathlib, tomllib
assert tomllib.loads(pathlib.Path("rust-toolchain.toml").read_text())["toolchain"]["channel"] == "1.97.1", "source Rust toolchain must match the pinned bootstrap"
PYTHON
node_root=$target-node-oracles
node_evidence=$evidence/node-prerequisite
bash scripts/source-oracles-bootstrap.sh "$revision" "$node_root" "$node_evidence"
node_path=$(cat "$node_evidence/node-oracles-path.txt")
test "$node_path" = "$node_root/node_modules"
# Child readiness scopes only Node dependencies; root readiness waits for Rust.
# Follow the existing M0 host acquisition path. Only acquisition uses the
# network; Cargo validates registry bytes against the unchanged source lock.
rustup toolchain install 1.97.1 --profile minimal --no-self-update
rustup run 1.97.1 rustc --version > "$evidence/rustc-version.txt"
rustup run 1.97.1 cargo --version > "$evidence/cargo-version.txt"
rustup which --toolchain 1.97.1 rustc > "$evidence/rustc-path.txt"
rustup which --toolchain 1.97.1 cargo > "$evidence/cargo-path.txt"
CARGO_NET_OFFLINE=false rustup run 1.97.1 cargo fetch --locked
CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 \
  rustup run 1.97.1 cargo build --locked --offline --jobs 2 \
  --package repo-conformance --bin prismpm-semantic-source --target-dir "$target"
helper="$target/debug/prismpm-semantic-source"
test -x "$helper"
helper_output=$(printf '[]' | "$helper" runtime-audit)
test "$helper_output" = '[]'
source_status=$(git status --porcelain)
test -z "$source_status"
test "$(git rev-parse HEAD)" = "$revision"
test "$(git rev-parse 'HEAD^{tree}')" = "$(cat "$evidence/source-tree.txt")"
cmp Cargo.lock "$evidence/Cargo.lock"
cmp rust-toolchain.toml "$evidence/rust-toolchain.toml"
cmp sdk/oracles/package.json "$node_evidence/node-oracles-package.json"
cmp sdk/oracles/package-lock.json "$node_evidence/node-oracles-package-lock.json"
cmp "$node_root/package.json" "$node_evidence/node-oracles-package.json"
cmp "$node_root/package-lock.json" "$node_evidence/node-oracles-package-lock.json"
cmp tools.lock "$node_evidence/node-oracles-tools.lock"
sha256sum Cargo.lock rust-toolchain.toml "$(cat "$evidence/rustc-path.txt")" "$(cat "$evidence/cargo-path.txt")" "$helper" \
  sdk/oracles/package.json sdk/oracles/package-lock.json "$node_root/package.json" "$node_root/package-lock.json" \
  "$(cat "$node_evidence/node-path.txt")" "$(cat "$node_evidence/npm-path.txt")" > "$evidence/SHA256SUMS"
# Publish both prerequisites only after every source, tool and executable probe succeeds.
printf '%s\n' "$helper" > "$evidence/semantic-source-path.txt"
printf '%s\n' "$node_root/node_modules" > "$evidence/node-oracles-path.txt"
