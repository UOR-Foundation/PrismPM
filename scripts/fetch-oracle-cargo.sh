#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"
if [ "$#" -gt 1 ] || { [ "$#" -eq 1 ] && [ "$1" != --check ]; }; then
  printf 'usage: %s [--check]\n' "$0" >&2
  exit 64
fi

# Acquisition is separate from the offline verifier and production Cargo graph.
# Browser compiler fixtures are internal gates, not external standards oracles.
# Review these pins whenever an independently locked verification graph changes.
sha256sum --check --strict <<'CHECKSUMS'
ec19fdecf241b6e502726c1740bdc962dffc7fc1eaf456468ba0836b8fb2272a  tests/hologram-oracle/Cargo.toml
16e2a6e4ce47c12b3510813b605897a377981b8314d031775a4ec99e89b5c277  tests/hologram-oracle/Cargo.lock
5be0cb25a1bc7ebff4a709c6dd4c724ae787c6d97e4e9b0a9070ca2cafa71f89  tests/holo-codec-oracle/Cargo.toml
8ab0b99efa9ab6f44d9368d026aefadcefae4640cc92c814947609b41aaf0df4  tests/holo-codec-oracle/Cargo.lock
1e0a368fc436907ea62dab3cf44d73a4aad133584ae6dd9799d741aa82fe1f68  tests/browser-workspace/Cargo.toml
6000241145996dc2cf1402dc69a5f3cd6b9a196f4b2625fc2a86adb2071f4f8a  tests/browser-workspace/Cargo.lock
a8ddd90c616555741a750f91d820b80827438612523624c5afdc7e0301a76493  tests/browser-envelope/driver/Cargo.toml
84c8c714e0260a7e3cbce39e46094b049ce00813c7a9628c1bfe30eb41febb39  tests/browser-envelope/driver/Cargo.lock
996a76f015df5c4166e450f67e5016b7ac39817acdd57d3c678aee9c99212681  tests/browser-journal/driver/Cargo.toml
153bbe47305311f8789d8f720d1d0e5c1d6dbc6aff347ec59a852378f1625dbc  tests/browser-journal/driver/Cargo.lock
caf5c34ef2b21d58c1aa12acf81cb13ace1adaffb3c69a641f54f490ed61cf66  crates/prismpm/vendor/hologram-live.tar
feb39fe80f840e1e564b6e07a213ca8f6eace18f85416d5386c699addec3ba87  tests/browser-command/driver/Cargo.toml
9eaf71006d86d0e4214ee8dc280940e94fdcd4c0756190f7b11bed32d2454d1b  tests/browser-command/driver/Cargo.lock
f0a86585e2c142af57db2b060e7626accc670ccbcbc42e120f8261429cec5f31  tests/browser-query/driver/Cargo.toml
01165f79f7a0781566ad21192c9a25eedd160b802e584706290443962c6f839f  tests/browser-query/driver/Cargo.lock
9a900b56d97f32040f6a6eb386226062822ec299f9e8d3832d880eb67e7980fa  tests/browser-view/driver/Cargo.toml
acebc6ba5696ee8057f5c7affcba3cdf3503eed6e55cd9aeaef5e7628e49654f  tests/browser-view/driver/Cargo.lock
34114e4a9080e9fee708d7ced51ddb05e03fe82c633e7847cf596d431ad42348  tests/browser-effects/driver/Cargo.toml
3c41b979ab8f83003a1f99db2e2934b4e53c0abe2a3c145bce89a276be2bdce0  tests/browser-effects/driver/Cargo.lock
cd971655426523e79de558c58706629b633f18f55b977f9441279879139b96b8  tests/browser-presentation/driver/Cargo.toml
8d7d1ee0441527cabe690e8cb81fd9682db87f35526ac4c3adcd1f1928985317  tests/browser-presentation/driver/Cargo.lock
CHECKSUMS
cmp tests/hologram-oracle/Cargo.toml crates/prismpm/src/embedded/hologram-oracle.Cargo.toml
cmp tests/hologram-oracle/Cargo.lock crates/prismpm/src/embedded/hologram-oracle.Cargo.lock
cmp tests/hologram-oracle/src/main.rs crates/prismpm/src/embedded/hologram-oracle.main.rs
if [ "${1:-}" = --check ]; then
  exit 0
fi

oracle_work=$(mktemp -d)
trap 'rm -r -- "$oracle_work"' EXIT
mkdir -p "$oracle_work/hologram-live" "$oracle_work/harness/src"
tar -xf crates/prismpm/vendor/hologram-live.tar -C "$oracle_work/hologram-live"
cp tests/hologram-oracle/Cargo.toml "$oracle_work/harness/Cargo.toml"
cp tests/hologram-oracle/Cargo.lock "$oracle_work/harness/Cargo.lock"
cp tests/hologram-oracle/src/main.rs "$oracle_work/harness/src/main.rs"

for manifest in "$oracle_work/harness/Cargo.toml" \
  tests/holo-codec-oracle/Cargo.toml \
  tests/browser-workspace/Cargo.toml \
  tests/browser-envelope/driver/Cargo.toml \
  tests/browser-journal/driver/Cargo.toml \
  tests/browser-command/driver/Cargo.toml \
  tests/browser-query/driver/Cargo.toml \
  tests/browser-view/driver/Cargo.toml \
  tests/browser-effects/driver/Cargo.toml \
  tests/browser-presentation/driver/Cargo.toml; do
  cargo fetch --locked --manifest-path "$manifest"
  cargo metadata --locked --offline --format-version 1 --manifest-path "$manifest" >/dev/null
done
