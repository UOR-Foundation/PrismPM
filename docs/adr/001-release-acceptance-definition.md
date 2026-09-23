# ADR-001: Release Acceptance Definition for PrismPM v0.3.0

## Status
Accepted

## Context
The PrismPM v0.3.0 release claims "complete and accepted" status (README.md, RELEASE-STATUS.md) while simultaneously documenting:
- First-party crates (lexlean, prod-ir, prod-codegen, prism-stdlib, prismpm) are not published to crates.io
- The `package-api` gate (Gate 15) fails due to missing `uor-hologram` dependency
- Foundry portal integration and Pages deployment are outstanding
- RELEASE-STATUS.md:227 states "These are outstanding requirements, not exclusions or reductions of scope"

SPEC.md:497-510 defines v0.2.0 release requirements including "registry-only fresh consumers" and "live asset hash verification" — both requiring public registry publication.

## Decision
**Redefine "release acceptance" as a two-phase milestone:**

### Phase 1: SDK Release Acceptance (Current v0.3.0 claim)
- All 15 `cargo xtask vv` gates pass in clean devcontainer
- Dual-platform OCI SDK images built and verified (linux/amd64, linux/arm64)
- Independent Hologram oracle verification passes (calculator + text interop)
- Supply chain, reproducibility, and security disposition gates pass
- Release status closure manifest (`prismpm/release-status-closure/1`) verified
- **Explicitly excludes**: crates.io publication, Foundry integration, Pages deployment

### Phase 2: Ecosystem Release Acceptance (v0.3.0 + ecosystem)
- Phase 1 complete
- First-party crates published to crates.io with trusted publishing configured
- `uor-hologram` crate published and available in public index
- Foundry portal bound to verified SDK
- Calculator-example Pages deployment live at https://uor-foundation.github.io/calculator-example/
- Downstream template and calculator-reference closure verified end-to-end
- Complete `prismpm/ecosystem-release/2` manifest with all 14 falsification classes

## Consequences
- **Positive**: Honest communication about what "v0.3.0 release" actually means; unblocks SDK consumers who use OCI images
- **Negative**: "Release" terminology becomes ambiguous; must qualify every claim with phase
- **Action**: Update README.md, RELEASE-STATUS.md, and SPEC.md to use phased terminology; add Phase 2 tracking issue

## References
- SPEC.md §9 (Release requirements)
- RELEASE-STATUS.md § "Release acceptance closure"
- VERIFICATION.md § "Gate 15" and "Package/public API"