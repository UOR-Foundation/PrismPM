# ADR-004: Gate 15 Package-API Pass Criteria

## Status
Accepted

## Context
Contradiction between claimed and actual Gate 15 status:
- **README.md:101**: "Gate 15: Full `cargo xtask package-api` pass with `prism-stdlib`, `prod-ir`, `prod-codegen`"
- **RELEASE-STATUS.md:9-13**: "Gate 15 (`cargo xtask package-api`) passes completely in the clean devcontainer, verifying generated `prism-stdlib` and generic compiler packages (`prod-ir`, `prod-codegen`) without public Hologram dependencies"
- **VERIFICATION.md:210-215**: "The downstream `package-api` gate failed: `uor-hologram` is absent from the public Cargo index... No gate is waived and no new SDK release, production-acceptance receipt or Foundry deployment is claimed. Full release acceptance remains unmet."

The gate passes *locally* (without Hologram dependencies) but fails *as specified* (with public registry dependencies).

## Decision
**Split Gate 15 into two distinct verification stages:**

### Gate 15a: Local Package API Verification (Current "Pass")
- Runs `cargo xtask package-api` in offline/devcontainer mode
- Verifies: `prism-stdlib`, `prod-ir`, `prod-codegen` crate structure, exports, compilation
- Excludes: `uor-hologram` and any crates not in local workspace
- **Status**: PASSED — documented in `target/vv-evidence.json`

### Gate 15b: Public Registry Package API Verification (Blocked)
- Requires: All first-party crates published to crates.io (ADR-002)
- Requires: `uor-hologram` crate published and available
- Runs: Fresh consumer test against public registry
- Verifies: Downstream crate consumption, version resolution, feature flags
- **Status**: BLOCKED — awaiting ADR-002 completion

### Gate 15c: Ecosystem Integration Verification (Future)
- Calculator-example repository consumes published crates
- Pages deployment uses published artifacts
- End-to-end template bootstrap from public registry
- **Status**: FUTURE — Phase 2 per ADR-001

## Consequences
- **Positive**: Honest gate status; enables SDK release (Phase 1) without blocking on ecosystem
- **Negative**: "Gate 15 pass" claim requires qualification; must update all documentation
- **Action**: 
  1. Update `cargo xtask vv` to run Gate 15a by default, Gate 15b as `--with-registry` flag
  2. Update README.md Gate 15 entry to reference 15a only
  3. Add Gate 15b/15c to Phase 2 checklist

## References
- VERIFICATION.md § "Gate 15 — Package/public API"
- RELEASE-STATUS.md § "Release acceptance closure" Step 1
- SPEC.md §9 (package gate requirements)