set shell := ["bash", "-c"]

default:
    @just --list

# Run full non-mutating verification and acceptance gates
vv:
    cargo metadata --locked
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo run --package xtask -- vv

# Validate claim registers and spec links
validate:
    cargo run --package xtask -- validate

# Check all positive and negative fixture suites
check-fixtures:
    cargo run --package xtask -- check-fixtures

# Rewrite fixture expected outputs
write-fixtures:
    cargo run --package xtask -- check-fixtures --write

# Verify reproducibility across two distinct build roots
check-reproducibility:
    cargo run --package xtask -- check-reproducibility

# Verify examples and standard library
verify-examples:
    cargo run --package xtask -- verify-examples

# Verify golden outputs
check-golden:
    cargo run --package xtask -- check-golden

# Regenerate model artifacts and documentation
codegen:
    cargo run --package xtask -- validate-model --write

# Check release readiness
release-check:
    cargo run --package xtask -- release-check

# Stage release artifacts
release-artifacts:
    cargo run --package xtask -- release-artifacts
