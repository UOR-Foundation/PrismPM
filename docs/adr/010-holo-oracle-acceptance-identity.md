# ADR-010: Nonexistent `prismpm/holo-oracle-acceptance/1` Identity

## Status
Proposed

## Context
Three documents cite `prismpm/holo-oracle-acceptance/1` as the verification identity for the independent-oracle acceptance task:

- README.md:102 (Independent Hologram Oracle row, "Verification" column).
- RELEASE-STATUS.md:170 (release acceptance step 1).
- VERIFICATION.md:1704 (release acceptance closure step 1).

That identity does not exist anywhere in the repository:

- No schema file under `schemas/` uses it; `schemas/oracle-validation-attestation.schema.json:2` is the oracle-related schema.
- No `CanonicalDocument` parse, fixture, or conformance case references it; it is absent from `model/contracts.toml` (48 registered contracts), which contains `prismpm/oracle-validation-attestation/1` (model/contracts.toml:119).
- The execution identity the code actually enforces is `prismpm/hologram-oracle/2` (tests/hologram_interop.rs:143,248; verification.rs:712; SPEC.md:1283), with explicit rejection of `hologram-oracle/1` and unknown editions (hologram_interop.rs:375-379).
- The portable-browser execution identity is `prismpm/portable-browser-oracle/1` (hologram_interop.rs:153,258,352; crates/prismpm/src/embedded/hologram-oracle.browser.mjs:264).
- The acceptance-attestation identity is `prismpm/oracle-validation-attestation/1` (contracts.rs:222,754; oci/verification_closure.rs:505; CONTRACTS.md:23).

Notably, README.md:102's own Description column correctly names `prismpm/hologram-oracle/2`; only the Verification column names the phantom identity.

## Decision
**Resolve the dissonance by correcting the citations to identities that actually exist, not by inventing the phantom schema:**

1. Replace `prismpm/holo-oracle-acceptance/1` in README.md:102, RELEASE-STATUS.md:170, and VERIFICATION.md:1704 with `prismpm/oracle-validation-attestation/1` (the acceptance attestation), paired with `prismpm/hologram-oracle/2` (the execution report schema) and `prismpm/portable-browser-oracle/1` (portable-browser execution) where those are the artifacts actually produced.
2. Do not add a new "holo-oracle-acceptance" schema. Per AGENTS.md, a public identity may only be referenced once it is registered in `model/*.toml` and covered by conformance fixtures.

## Consequences
- **Positive**: every cited verification identity maps to an enforced, fixture-covered schema; readers can audit the cited artifact.
- **Negative**: none for behavior; docs-only correction reflects existing enforcement.
- **Action**: apply the three doc edits; add a conformance register check that all identity citations in README/RELEASE-STATUS/VERIFICATION resolve to `model/contracts.toml`.

## References
- README.md:102, RELEASE-STATUS.md:170, VERIFICATION.md:1704
- `model/contracts.toml:119`, `schemas/oracle-validation-attestation.schema.json:2`
- `tests/hologram_interop.rs:143,153,248,258,352,375-379`
- `crates/prismpm/src/embedded/hologram-oracle.browser.mjs:264`