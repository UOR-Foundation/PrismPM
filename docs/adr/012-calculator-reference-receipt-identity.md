# ADR-012: Calculator Reference Receipt Identity Drift

## Status
Proposed

## Context
The calculator-reference closure emits one receipt, but three different schema names are used for it across the code base and documentation:

- The closure manifest schema is `prismpm/calculator-reference-closure/1` (crates/prismpm/src/acceptance.rs:594, it rejects any other value, e.g. `calculator-reference-closure/2` at tests/calculator_reference_closure.rs:119; documented at acceptance.rs:547,583 and tests/calculator_reference_closure.rs:4,23).
- The receipt the validator actually produces and validates is `prismpm/calculator-reference-receipt/1` (acceptance.rs:666; asserted at tests/calculator_reference_closure.rs:49).
- The release-status-closure Step 5 wrapper, however, records the receipt under `prismpm/calculator-reference-closure-receipt/1` (tests/release_status_closure.rs:51-52), a third name.
- The documentation follows the wrapper: RELEASE-STATUS.md:191 and VERIFICATION.md:1451 both name the step-5 receipt `prismpm/calculator-reference-closure-receipt/1`.

One artifact is therefore identified three ways: `prismpm/calculator-reference-receipt/1` (implemented), `prismpm/calculator-reference-closure-receipt/1` (wrapper + docs), and `prismpm/calculator-reference-closure/1` (the manifest, a different artifact).

## Decision
**Canonicalize the receipt identity as `prismpm/calculator-reference-receipt/1` (the identity the validator emits and the isolation test asserts):**

1. Update `Step5DownstreamClosureReceipt.schema` in tests/release_status_closure.rs:51 to `prismpm/calculator-reference-receipt/1`.
2. Update RELEASE-STATUS.md:191 and VERIFICATION.md:1451 to the same name.
3. Keep the manifest schema `prismpm/calculator-reference-closure/1` distinct from the receipt schema; document the manifest-vs-receipt naming rule (manifest = `<scope>-closure/N`, receipt = `<scope>-receipt/N`) so the `-closure-receipt/` conflation cannot recur.

## Consequences
- **Positive**: receipt identity matches across validator, isolation test, wrapper, and docs.
- **Negative**: RELEASE-STATUS.md:191/VERIFICATION.md:1451 text changes; no wire-format change (the emitted receipt name is unchanged).
- **Action**: apply the three edits; add a conformance case asserting the wrapper's step-5 receipt schema equals the closure validator's emitted schema.

## References
- `crates/prismpm/src/acceptance.rs:594,666`
- `tests/calculator_reference_closure.rs:49,119`
- `tests/release_status_closure.rs:51-52`
- RELEASE-STATUS.md:191, VERIFICATION.md:1451