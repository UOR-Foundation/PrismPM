#!/usr/bin/env bash
# The factory-model obligation: the factory checks itself.
#  1. The certified bundle replays without the source, Lean or the network.
#  2. When the pinned Lean is present, the model is verified through the kernel
#     and re-exported, and the export must equal the committed bundle.
#  3. The host's tests pass and the committed workflow equals the projection.
# It runs in a directory that holds only this obligation's declared inputs.
set -euo pipefail
lexlean_target=${CARGO_TARGET_DIR:-$PWD/target}/lexlean
cargo build --quiet --release --locked --manifest-path vendor/lexlean/Cargo.toml --bin lexlean --target-dir "$lexlean_target"
lexlean=$lexlean_target/release/lexlean
"$lexlean" production verify-artifact prism-factory/certified
if command -v lean >/dev/null 2>&1; then
  (cd prism-factory/model && "$lexlean" lock --check && "$lexlean" verify --all)
  result=$(cd prism-factory/model && "$lexlean" production build --request ../production-request.json | sed -n 's/.*; result \([0-9a-f]\{64\}\);.*/\1/p')
  diff -r "prism-factory/model/.lexlean/production/$result" prism-factory/certified
else
  echo "check.sh: no Lean toolchain here; the committed bundle was replayed, not re-exported" >&2
fi
cargo test --quiet --release --locked --offline --manifest-path prism-factory/host/Cargo.toml
cargo build --quiet --release --locked --offline --manifest-path prism-factory/host/Cargo.toml
prism-factory/host/target/release/prism-factory workflow --check .github/workflows/factory.yml
