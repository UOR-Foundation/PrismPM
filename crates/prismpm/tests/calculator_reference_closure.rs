//! Integration tests for Calculator reference system closure (Task 11 / Issue #17).
//!
//! Validates:
//! 1. Complete Calculator reference system closure against production policy (`prismpm/calculator-reference-closure/1`).
//! 2. Enforcement of locked SDK digest consumption (no vendored PrismPM fallback).
//! 3. Distinct immutable product digests for Release A and Release B.
//! 4. Full target coverage across Compose, Kubernetes, and Pages.
//! 5. Binding of capability coverage matrix and production acceptance evidence.
//! 6. Rejection of tampered, malformed, or missing dependencies.

use prismpm::acceptance::{validate_calculator_reference_closure, CalculatorReferenceClosure};

fn sha256_hex(c: char) -> String {
    c.to_string().repeat(64)
}

fn sha256_digest(c: char) -> String {
    format!("sha256:{}", sha256_hex(c))
}

fn valid_closure(sdk_digest: &str) -> CalculatorReferenceClosure {
    CalculatorReferenceClosure {
        schema: "prismpm/calculator-reference-closure/1".to_owned(),
        repository: "UOR-Foundation/calculator-example".to_owned(),
        sdk_digest: sdk_digest.to_owned(),
        release_a_digest: sha256_digest('a'),
        release_b_digest: sha256_digest('b'),
        coverage_digest: sha256_digest('c'),
        acceptance_digest: sha256_digest('d'),
        targets: vec![
            "compose".to_owned(),
            "kubernetes".to_owned(),
            "pages".to_owned(),
        ],
        features_covered: 161,
        diagnostics_covered: 84,
        verified_at_unix: 1_789_000_000,
    }
}

#[test]
fn valid_calculator_reference_closure_passes_and_produces_receipt() {
    let expected_sdk = sha256_digest('1');
    let closure = valid_closure(&expected_sdk);

    let receipt = validate_calculator_reference_closure(&closure, &expected_sdk)
        .expect("valid calculator reference closure must pass");

    assert_eq!(receipt["schema"], "prismpm/calculator-reference-receipt/1");
    assert_eq!(receipt["status"], "verified");
    assert_eq!(receipt["repository"], "UOR-Foundation/calculator-example");
    assert_eq!(receipt["sdk_digest"], expected_sdk);
    assert_eq!(receipt["features_covered"], 161);
    assert_eq!(receipt["diagnostics_covered"], 84);
    assert_eq!(receipt["release_a_digest"], sha256_digest('a'));
    assert_eq!(receipt["release_b_digest"], sha256_digest('b'));
    let targets = receipt["targets"].as_array().expect("targets array");
    assert_eq!(targets.len(), 3);
}

#[test]
fn calculator_reference_closure_rejects_wrong_sdk_digest() {
    let expected_sdk = sha256_digest('1');
    let mut closure = valid_closure(&expected_sdk);
    closure.sdk_digest = sha256_digest('2');

    let err = validate_calculator_reference_closure(&closure, &expected_sdk)
        .expect_err("mismatched SDK digest must fail closed");
    assert_eq!(err.code, "PP6002");
    assert!(err
        .message
        .contains("does not match the expected locked SDK"));
}

#[test]
fn calculator_reference_closure_rejects_identical_release_a_and_b_digests() {
    let expected_sdk = sha256_digest('1');
    let mut closure = valid_closure(&expected_sdk);
    closure.release_b_digest = closure.release_a_digest.clone();

    let err = validate_calculator_reference_closure(&closure, &expected_sdk)
        .expect_err("identical release digests must fail closed");
    assert_eq!(err.code, "PP6002");
    assert!(err.message.contains("distinct immutable product digests"));
}

#[test]
fn calculator_reference_closure_rejects_missing_required_targets() {
    let expected_sdk = sha256_digest('1');

    for target in ["compose", "kubernetes", "pages"] {
        let mut closure = valid_closure(&expected_sdk);
        closure.targets.retain(|t| t != target);
        let err = validate_calculator_reference_closure(&closure, &expected_sdk)
            .expect_err("omitted required target must fail closed");
        assert_eq!(err.code, "PP6002");
        assert!(err
            .message
            .contains(&format!("must cover required target {target}")));
    }
}

#[test]
fn calculator_reference_closure_rejects_wrong_repository_or_schema() {
    let expected_sdk = sha256_digest('1');

    // Wrong repo
    let mut bad_repo = valid_closure(&expected_sdk);
    bad_repo.repository = "other-org/other-repo".to_owned();
    let err_repo = validate_calculator_reference_closure(&bad_repo, &expected_sdk)
        .expect_err("wrong repository must fail closed");
    assert_eq!(err_repo.code, "PP6002");
    assert!(err_repo
        .message
        .contains("repository differs from expected"));

    // Wrong schema
    let mut bad_schema = valid_closure(&expected_sdk);
    bad_schema.schema = "prismpm/calculator-reference-closure/2".to_owned();
    let err_schema = validate_calculator_reference_closure(&bad_schema, &expected_sdk)
        .expect_err("wrong schema must fail closed");
    assert_eq!(err_schema.code, "PP6002");
    assert!(err_schema.message.contains("schema is unsupported"));
}

#[test]
fn calculator_reference_closure_rejects_malformed_digests() {
    let expected_sdk = sha256_digest('1');

    let mut bad_digest = valid_closure(&expected_sdk);
    bad_digest.coverage_digest = "not-a-digest".to_owned();
    let err = validate_calculator_reference_closure(&bad_digest, &expected_sdk)
        .expect_err("malformed digest must fail closed");
    assert_eq!(err.code, "PP6002");
    assert!(err.message.contains("coverage_digest is malformed"));
}

#[test]
fn calculator_reference_closure_rejects_zero_coverage_counts() {
    let expected_sdk = sha256_digest('1');

    let mut zero_features = valid_closure(&expected_sdk);
    zero_features.features_covered = 0;
    let err_feat = validate_calculator_reference_closure(&zero_features, &expected_sdk)
        .expect_err("zero features must fail closed");
    assert_eq!(err_feat.code, "PP6002");

    let mut zero_diagnostics = valid_closure(&expected_sdk);
    zero_diagnostics.diagnostics_covered = 0;
    let err_diag = validate_calculator_reference_closure(&zero_diagnostics, &expected_sdk)
        .expect_err("zero diagnostics must fail closed");
    assert_eq!(err_diag.code, "PP6002");
}
