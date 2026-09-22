//! Integration tests for PrismPM v0.3.0 release acceptance closure.

use prismpm::acceptance::{
    validate_release_status_closure, validate_v0_3_0_release_acceptance, ReleaseStatusClosure,
    Step1DependencyClosureReceipt, Step2ReproducibilityReceipt, Step3DualPlatformGatesReceipt,
    Step4FunctionalCoreAndCargoReceipt, Step5DownstreamClosureReceipt,
    Step6EcosystemReleaseReceipt,
};

fn valid_closure() -> ReleaseStatusClosure {
    ReleaseStatusClosure {
        schema: "prismpm/release-status-closure/1".to_string(),
        version: "0.3.0".to_string(),
        status: "accepted".to_string(),
        step1_dependency_closure: Step1DependencyClosureReceipt {
            schema: "prismpm/dependency-closure-receipt/1".to_string(),
            status: "accepted".to_string(),
            lean4_prod_commit: "4b825dc642cb6eb9a060e54bf8d69288fbee4904".to_string(),
            lexlean_version: "0.3.0".to_string(),
            holo_oracle_receipt_digest:
                "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                    .to_string(),
        },
        step2_reproducibility: Step2ReproducibilityReceipt {
            schema: "prismpm/reproducibility-receipt/1".to_string(),
            status: "accepted".to_string(),
            golden_file_count: 324,
            calculator_regression_count: 42,
            integrity_verified: true,
        },
        step3_dual_platform_gates: Step3DualPlatformGatesReceipt {
            schema: "prismpm/dual-platform-receipt/1".to_string(),
            status: "accepted".to_string(),
            twice_without_cleanup: true,
            sdk_index_digest:
                "sha256:2222222222222222222222222222222222222222222222222222222222222222"
                    .to_string(),
            platforms: vec!["linux/amd64".to_string(), "linux/arm64".to_string()],
        },
        step4_functional_core_and_cargo: Step4FunctionalCoreAndCargoReceipt {
            schema: "prismpm/functional-core-receipt/1".to_string(),
            status: "accepted".to_string(),
            foundry_bound: true,
            functional_core_verified: true,
            crates_io_bootstrap_verified: true,
        },
        step5_downstream_closure: Step5DownstreamClosureReceipt {
            schema: "prismpm/downstream-closure-receipt/1".to_string(),
            status: "accepted".to_string(),
            template_contract_digest:
                "sha256:3333333333333333333333333333333333333333333333333333333333333333"
                    .to_string(),
            calculator_reference_closure_digest:
                "sha256:4444444444444444444444444444444444444444444444444444444444444444"
                    .to_string(),
            targets_verified: vec![
                "compose".to_string(),
                "kubernetes".to_string(),
                "pages".to_string(),
            ],
        },
        step6_ecosystem_manifest: Step6EcosystemReleaseReceipt {
            schema: "prismpm/ecosystem-release-receipt/2".to_string(),
            status: "accepted".to_string(),
            manifest_digest:
                "sha256:5555555555555555555555555555555555555555555555555555555555555555"
                    .to_string(),
            falsification_classes_verified: 14,
        },
    }
}

#[test]
fn test_valid_v0_3_0_release_acceptance() {
    let closure = valid_closure();
    let result = validate_v0_3_0_release_acceptance(&closure, 1758470400).unwrap();
    assert_eq!(result["status"], "released");
    assert_eq!(result["result"], "accepted");
    assert_eq!(result["release_version"], "0.3.0");
    assert_eq!(result["schema"], "prismpm/production-release-acceptance/1");
    assert_eq!(
        result["acceptance_receipt"]["schema"],
        "prismpm/release-status-closure-receipt/1"
    );
    assert_eq!(result["acceptance_receipt"]["all_steps_closed"], true);
    assert_eq!(result["acceptance_receipt"]["completed_steps"], 6);
}

#[test]
fn test_invalid_schema_or_version() {
    let mut closure = valid_closure();
    closure.schema = "prismpm/release-status-closure/0".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.version = "0.4.0".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.status = "under verification".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");
}

#[test]
fn test_step1_defects_rejected() {
    let mut closure = valid_closure();
    closure.step1_dependency_closure.status = "pending".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.step1_dependency_closure.lean4_prod_commit = "invalid_commit".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.step1_dependency_closure.lexlean_version = "0.2.0".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.step1_dependency_closure.holo_oracle_receipt_digest = "not-sha256".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");
}

#[test]
fn test_step2_defects_rejected() {
    let mut closure = valid_closure();
    closure.step2_reproducibility.golden_file_count = 323;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.step2_reproducibility.calculator_regression_count = 0;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.step2_reproducibility.integrity_verified = false;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");
}

#[test]
fn test_step3_defects_rejected() {
    let mut closure = valid_closure();
    closure.step3_dual_platform_gates.twice_without_cleanup = false;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.step3_dual_platform_gates.platforms = vec!["linux/amd64".to_string()];
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");
}

#[test]
fn test_step4_defects_rejected() {
    let mut closure = valid_closure();
    closure.step4_functional_core_and_cargo.foundry_bound = false;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure
        .step4_functional_core_and_cargo
        .functional_core_verified = false;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure
        .step4_functional_core_and_cargo
        .crates_io_bootstrap_verified = false;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");
}

#[test]
fn test_step5_defects_rejected() {
    let mut closure = valid_closure();
    closure.step5_downstream_closure.targets_verified =
        vec!["compose".to_string(), "kubernetes".to_string()];
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");
}

#[test]
fn test_step6_defects_rejected() {
    let mut closure = valid_closure();
    closure
        .step6_ecosystem_manifest
        .falsification_classes_verified = 13;
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");

    let mut closure = valid_closure();
    closure.step6_ecosystem_manifest.manifest_digest = "sha256:bad".to_string();
    let err = validate_release_status_closure(&closure, 1758470400).unwrap_err();
    assert_eq!(err.code, "PP6004");
}
