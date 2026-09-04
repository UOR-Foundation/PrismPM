#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

cargo fetch --locked
cargo fetch --locked --manifest-path vendor/lexlean/Cargo.toml

# PrismPM verifies Hologram archives with an embedded, independently locked
# Cargo harness. Fetch that graph while network access is available so the
# normative gate can build it offline from a cold devcontainer cache.
oracle_work=$(mktemp -d)
cleanup() {
  rm -rf "$oracle_work"
}
trap cleanup EXIT

mkdir -p "$oracle_work/hologram-live" "$oracle_work/harness/src"
tar -xf crates/prismpm/vendor/hologram-live.tar \
  -C "$oracle_work/hologram-live"
cp tests/hologram-oracle/Cargo.toml "$oracle_work/harness/Cargo.toml"
cp tests/hologram-oracle/Cargo.lock "$oracle_work/harness/Cargo.lock"
cp tests/hologram-oracle/src/main.rs "$oracle_work/harness/src/main.rs"
cargo fetch --locked --manifest-path "$oracle_work/harness/Cargo.toml"

cargo deny fetch
