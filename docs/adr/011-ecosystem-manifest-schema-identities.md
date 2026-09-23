# ADR-011: Ecosystem Manifest Schema Identities

## Status
Proposed

## Context
CHANGELOG.md:11 claims the v0.3.0 release introduces:

> "Authoritative production-system model (`prismpm/system-schema/1`, `prismpm/conformance-model/1`), immutable standards/oracle bindings, OCI release graphs, digest-only lifecycle controller, Compose and Kubernetes target projections, operations/supply-chain evidence, and the reproducible multi-platform PrismPM SDK."

Neither `prismpm/system-schema/1` nor `prismpm/conformance-model/1` exists in the repository:
- No schema file, fixture, conformance case, or `model/*.toml` register entry references them.
- The system-model identity actually enforced is `prismpm/system-model/1` (tests/production_system_model.rs:71-72,232; validated via `CanonicalDocument::from_value`).
- The production contract identities actually enforced are `prismpm/product-release/1` and `prismpm/deployment-plan/1` (tests/production_contracts.rs:15-69), with result schemas `prismpm/product-release-result/1`, `prismpm/deployment-plan/1` registered in model/commands.toml (lines 19, 91).
- These are the identities the release acceptance steps rely on (RELEASE-STATUS.md:174-175; VERIFICATION.md:1707), so the CHANGELOG wording describes different names than the artifacts the rest of the release documents.

## Decision
**Correct CHANGELOG.md:11 to the enforced identities:**

1. Name the production-system-model identity `prismpm/system-model/1`, and cite the production contract set as `prismpm/product-release/1` and `prismpm/deployment-plan/1` (with `prismpm/product-release-result/1`).
2. Drop `prismpm/system-schema/1` and `prismpm/conformance-model/1` unless a distinct conformance-model schema is deliberately introduced; if one is intended, register it in `model/contracts.toml` and add conformance fixtures before citing it.

## Consequences
- **Positive**: CHANGELOG names match the schemas tests validate; release-notes readers can locate the artifacts.
- **Negative**: none behavioral; the change is release-notes accuracy.
- **Action**: edit CHANGELOG.md:11; optionally add a conformance check that CHANGELOG schema citations resolve to the contract register.

## References
- CHANGELOG.md:11
- `tests/production_system_model.rs:71-72,232`
- `tests/production_contracts.rs:15-69`
- `model/commands.toml:19,91`, RELEASE-STATUS.md:174-175, VERIFICATION.md:1707