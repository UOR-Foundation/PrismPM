# ADR-002: Crates.io Bootstrap Strategy for First-Party Crates

## Status
Proposed

## Context
RELEASE-STATUS.md:32-41 documents that five first-party crates (lexlean, prod-ir, prod-codegen, prism-stdlib, prismpm) are not registered on crates.io. The current OIDC-only workflows cannot perform initial bootstrap unaided because:
- crates.io requires an owner-controlled first upload for new crate names
- Trusted publishing (OIDC) only works after initial publication
- The owner requires Foundry publication and live verification before those uploads

This creates a circular dependency: Foundry needs published crates, but crates need Foundry verification first.

SPEC.md:497-510 requires "registry-only fresh consumers" for release acceptance.

## Decision
**Adopt a three-stage bootstrap protocol:**

### Stage 1: Owner-Manual Bootstrap (One-time)
- Owner manually publishes each crate to crates.io using `cargo publish` with explicit credentials
- Version must match exactly: lexlean=0.3.0, prod-ir=0.1.0, prod-codegen=0.1.0, prism-stdlib=0.2.0, prismpm=0.3.0
- Configure trusted publishing (OIDC) for each crate to `uor-foundation/PrismPM` repository
- Record exact crate checksums in `model/dependencies.toml` and `prismpm.lock`

### Stage 2: Automated Verification
- CI pipeline verifies published crates match local builds byte-for-byte
- `cargo xtask package-api` gate runs against public registry (not local)
- Fresh consumer test: `cargo add prism-stdlib@=0.2.0` in clean project compiles and passes tests

### Stage 3: Ecosystem Release Gate
- Only after Stage 2 passes, `prismpm/ecosystem-release/2` manifest can be finalized
- Calculator-example repository updates to use published crate versions
- Pages deployment uses only published artifacts

## Consequences
- **Positive**: Breaks circular dependency; maintains reproducibility; enables true "registry-only fresh consumers"
- **Negative**: Requires manual owner action; introduces human step in otherwise automated pipeline
- **Risk**: If owner is unavailable, release blocks; mitigate by documenting procedure in RUNBOOK.md
- **Action**: Create `scripts/crates-io-bootstrap.sh` with exact commands; add to release checklist

## References
- RELEASE-STATUS.md § "Modeled codec and independent oracles"
- SPEC.md §9 (registry-only fresh consumers)
- VERIFICATION.md § "Gate 15 — Package/public API"