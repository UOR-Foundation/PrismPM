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
# Release packaging also verifies the independently locked upstream compiler.
# Review these pins whenever an independently locked verification graph changes.
sha256sum --check --strict <<'CHECKSUMS'
ec19fdecf241b6e502726c1740bdc962dffc7fc1eaf456468ba0836b8fb2272a  tests/hologram-oracle/Cargo.toml
16e2a6e4ce47c12b3510813b605897a377981b8314d031775a4ec99e89b5c277  tests/hologram-oracle/Cargo.lock
5be0cb25a1bc7ebff4a709c6dd4c724ae787c6d97e4e9b0a9070ca2cafa71f89  tests/holo-codec-oracle/Cargo.toml
8ab0b99efa9ab6f44d9368d026aefadcefae4640cc92c814947609b41aaf0df4  tests/holo-codec-oracle/Cargo.lock
1e0a368fc436907ea62dab3cf44d73a4aad133584ae6dd9799d741aa82fe1f68  tests/browser-workspace/Cargo.toml
5552bacea5fe3a48d3a0f9531729d655c7b10b859985939e69c666ab64138a16  tests/browser-workspace/Cargo.lock
a8ddd90c616555741a750f91d820b80827438612523624c5afdc7e0301a76493  tests/browser-envelope/driver/Cargo.toml
0d23592c7ee5fef496eb212410c59a4c925378fc949a9f953df71ba17b23c47e  tests/browser-envelope/driver/Cargo.lock
996a76f015df5c4166e450f67e5016b7ac39817acdd57d3c678aee9c99212681  tests/browser-journal/driver/Cargo.toml
c5d06660f26c67ddb0a6f17e2bb99c0c4117ce21fb96b23bfe87b59bec3e0cb3  tests/browser-journal/driver/Cargo.lock
caf5c34ef2b21d58c1aa12acf81cb13ace1adaffb3c69a641f54f490ed61cf66  crates/prismpm/vendor/hologram-live.tar
feb39fe80f840e1e564b6e07a213ca8f6eace18f85416d5386c699addec3ba87  tests/browser-command/driver/Cargo.toml
d96acc78f13923988e2a47c2c38d6663e814fd6c486fd4fcc8006ce360e0292f  tests/browser-command/driver/Cargo.lock
f0a86585e2c142af57db2b060e7626accc670ccbcbc42e120f8261429cec5f31  tests/browser-query/driver/Cargo.toml
7883e01534eb5d20a4a48bf21ca6448c60626a5d8967e76b2cc48fb125959f84  tests/browser-query/driver/Cargo.lock
9a900b56d97f32040f6a6eb386226062822ec299f9e8d3832d880eb67e7980fa  tests/browser-view/driver/Cargo.toml
db207c9b98bff0b7c75c757305ed5ba1d7dfd490b2e82007aebcf1da95d7f22a  tests/browser-view/driver/Cargo.lock
34114e4a9080e9fee708d7ced51ddb05e03fe82c633e7847cf596d431ad42348  tests/browser-effects/driver/Cargo.toml
b63355d07d3c2952db5748c197de3f346401038c697eea2e3992abcf02a4b758  tests/browser-effects/driver/Cargo.lock
cd971655426523e79de558c58706629b633f18f55b977f9441279879139b96b8  tests/browser-presentation/driver/Cargo.toml
f6355ccb80b69a9d5859a03f2a54c9897c20ab8cf9c1ec4ab4ec4841a9cf2a3f  tests/browser-presentation/driver/Cargo.lock
b0b774f35dc26abaf653abcd7e7b6b89d93d55d92131a71af83758f361d7f10a  tests/browser-p256/driver/Cargo.toml
d528d36a8169af8db4bea14e553359cb20d06d5654b5d82cf67b978b0650a210  tests/browser-p256/driver/Cargo.lock
5f597a966ad257bde250c2b71914bb943cc16ffbbb8cf16513d4d9c103fb3d06  tests/browser-session/driver/Cargo.toml
1bc381d37d894ce5706413a0e3ae85fb5e98d9124b90f019c18c21664267f891  tests/browser-session/driver/Cargo.lock
367929deb65c746365b59a6055c76f8e0ff97f334d08ca5382bc163474346213  tests/browser-custody/driver/Cargo.toml
2da38a5ecc3859423805e22be644d1071ae2a51734a1692ba2dc12083df876bf  tests/browser-custody/driver/Cargo.lock
10c2a425a0028ebe478c5a2e4f4435623634ed6529cf7193a2185aed9852206f  tests/browser-operation-journal/driver/Cargo.toml
f46f306db55bc7b8937f74a229d860f00e480238ddee63951871247ff98259ce  tests/browser-operation-journal/driver/Cargo.lock
d833db76924bdf573f4d0cd47a24b585bb454879be0ff668670629688277039f  tests/publication-admission/driver/Cargo.toml
8733132f1980bf72e1b7cf43cc54b385ddd626683531a21b22922442a83593f5  tests/publication-admission/driver/Cargo.lock
fc49c5659bded9db17612c76d47370fd715e4fa900f2b0aa8617e77a0fb13581  tests/browser-budget/driver/Cargo.toml
d4d57c5c7b105950aa2293f86a259284de6d332d18490123b759ac9fe04f777e  tests/browser-budget/driver/Cargo.lock
63e9f1793c48cddffd4c8378fe230ea1972a128ded41a2dda5814e841fefc2f4  vendor/lean4-prod/rust/Cargo.toml
5e16d324b08b942c476099b6c10235f4ddd58bca28aaf69514b8a88c9252c88f  vendor/lean4-prod/rust/Cargo.lock
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
  tests/browser-presentation/driver/Cargo.toml \
  tests/browser-p256/driver/Cargo.toml \
  tests/browser-session/driver/Cargo.toml \
  tests/browser-custody/driver/Cargo.toml \
  tests/browser-operation-journal/driver/Cargo.toml \
  tests/publication-admission/driver/Cargo.toml \
  tests/browser-budget/driver/Cargo.toml \
  vendor/lean4-prod/rust/Cargo.toml; do
  cargo fetch --locked --manifest-path "$manifest"
  cargo metadata --locked --offline --format-version 1 --manifest-path "$manifest" >/dev/null
done
