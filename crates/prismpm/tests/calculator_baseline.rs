//! Acceptance tests for the sealed Calculator baseline and production bootstrap record (Task 1).

use prismpm::contracts::CanonicalDocument;
use serde_json::Value;

#[test]
fn canonical_calculator_baseline_validates_against_contract_schema() {
    let baseline_bytes = include_bytes!("../../../tests/data/calculator-baseline.json");
    let baseline: Value = serde_json::from_slice(baseline_bytes).expect("valid JSON");

    // Must validate cleanly through the contracts engine
    CanonicalDocument::from_value("prismpm/calculator-baseline/1", baseline)
        .expect("canonical calculator baseline must satisfy contract schema");
}

#[test]
fn calculator_baseline_rejects_tampered_or_unordered_records() {
    let baseline_bytes = include_bytes!("../../../tests/data/calculator-baseline.json");
    let mut baseline: Value = serde_json::from_slice(baseline_bytes).expect("valid JSON");

    // Reversing repositories array must trigger ordering / noncanonical failure
    if let Some(repos) = baseline.get_mut("repositories").and_then(Value::as_array_mut) {
        repos.reverse();
    }
    let err = CanonicalDocument::from_value("prismpm/calculator-baseline/1", baseline)
        .expect_err("unordered repositories must be rejected");
    assert_eq!(err.code, "PP1101");
}
