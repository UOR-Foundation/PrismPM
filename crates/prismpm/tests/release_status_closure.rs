//! Tests for PrismPM v0.3.0 Release Status Closure steps 1-6.

use prismpm::acceptance::{
    validate_release_status_closure, ReleaseStatusClosure, Step1DependencyClosureReceipt,
    Step2ReproducibilityReceipt, Step3DualPlatformGatesReceipt, Step4FunctionalCoreAndCargoReceipt,
    Step5DownstreamClosureReceipt, Step6EcosystemReleaseReceipt,
};

fn sample_digest(c: char) -> String {
    format!("sha256:{}", c.to_string().repeat(64))
}

fn sample_commit() -> String {
    "2772411ed127411ed127411ed127411ed127411e".to_owned()
}

fn valid_release_status_closure_fixture() -> ReleaseStatusClosure {
    ReleaseStatusClosure {
        schema: "prismpm/release-status-closure/1".to_owned(),
        version: "0.3.0".to_owned(),
        status: "accepted".to_owned(),
        step1_dependency_closure: Step1DependencyClosureReceipt {
            schema: "prismpm/dependency-closure-receipt/1".to_owned(),
            status: "accepted".to_owned(),
            lean4_prod_commit: sample_commit(),
            lexlean_version: "0.3.0".to_owned(),
            holo_oracle_receipt_digest: sample_digest('1'),
        },
        step2_reproducibility: Step2ReproducibilityReceipt {
            schema: "prismpm/reproducibility-receipt/1".to_owned(),
            status: "passed".to_owned(),
            golden_file_count: 324,
            calculator_regression_count: 7,
            integrity_verified: true,
        },
        step3_dual_platform_gates: Step3DualPlatformGatesReceipt {
            schema: "prismpm/gate-closure-receipt/1".to_owned(),
            status: "passed".to_owned(),
            twice_without_cleanup: true,
            sdk_index_digest: sample_digest('2'),
            platforms: vec!["linux/amd64".to_owned(), "linux/arm64".to_owned()],
        },
        step4_functional_core_and_cargo: Step4FunctionalCoreAndCargoReceipt {
            schema: "prismpm/functional-core-receipt/1".to_owned(),
            status: "accepted".to_owned(),
            foundry_bound: true,
            functional_core_verified: true,
            crates_io_bootstrap_verified: true,
        },
        step5_downstream_closure: Step5DownstreamClosureReceipt {
            schema: "prismpm/calculator-reference-closure-receipt/1".to_owned(),
            status: "passed".to_owned(),
            template_contract_digest: sample_digest('3'),
            calculator_reference_closure_digest: sample_digest('4'),
            targets_verified: vec![
                "compose".to_owned(),
                "kubernetes".to_owned(),
                "pages".to_owned(),
            ],
        },
        step6_ecosystem_manifest: Step6EcosystemReleaseReceipt {
            schema: "prismpm/ecosystem-release-receipt/2".to_owned(),
            status: "accepted".to_owned(),
            manifest_digest: sample_digest('5'),
            falsification_classes_verified: 14,
        },
    }
}

#[test]
fn valid_release_status_closure_passes_and_produces_receipt() {
    let closure = valid_release_status_closure_fixture();
    let receipt = validate_release_status_closure(&closure, 2_000)
        .expect("complete release status closure must pass");

    assert_eq!(
        receipt["schema"],
        "prismpm/release-status-closure-receipt/1"
    );
    assert_eq!(receipt["version"], "0.3.0");
    assert_eq!(receipt["status"], "accepted");
    assert_eq!(receipt["result"], "verified");
    assert_eq!(receipt["completed_steps"], 6);
    assert_eq!(receipt["all_steps_closed"], true);
    assert_eq!(receipt["verified_at_unix"], 2_000);
}

#[test]
fn rejects_invalid_schema_or_version() {
    let mut bad_schema = valid_release_status_closure_fixture();
    bad_schema.schema = "prismpm/release-status-closure/2".to_owned();
    let error = validate_release_status_closure(&bad_schema, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut bad_version = valid_release_status_closure_fixture();
    bad_version.version = "0.4.0".to_owned();
    let error = validate_release_status_closure(&bad_version, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_unaccepted_or_development_status() {
    let mut dev = valid_release_status_closure_fixture();
    dev.status = "development".to_owned();
    let error = validate_release_status_closure(&dev, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut smoke = valid_release_status_closure_fixture();
    smoke.status = "smoke-checked".to_owned();
    let error = validate_release_status_closure(&smoke, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_step1_defects() {
    let mut bad_step1_status = valid_release_status_closure_fixture();
    bad_step1_status.step1_dependency_closure.status = "pending".to_owned();
    let error = validate_release_status_closure(&bad_step1_status, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut bad_commit = valid_release_status_closure_fixture();
    bad_commit.step1_dependency_closure.lean4_prod_commit = "short-hash".to_owned();
    let error = validate_release_status_closure(&bad_commit, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut bad_lexlean = valid_release_status_closure_fixture();
    bad_lexlean.step1_dependency_closure.lexlean_version = "0.2.9".to_owned();
    let error = validate_release_status_closure(&bad_lexlean, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_step2_defects() {
    let mut low_goldens = valid_release_status_closure_fixture();
    low_goldens.step2_reproducibility.golden_file_count = 323;
    let error = validate_release_status_closure(&low_goldens, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut zero_regressions = valid_release_status_closure_fixture();
    zero_regressions
        .step2_reproducibility
        .calculator_regression_count = 0;
    let error = validate_release_status_closure(&zero_regressions, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut unverified_integrity = valid_release_status_closure_fixture();
    unverified_integrity
        .step2_reproducibility
        .integrity_verified = false;
    let error = validate_release_status_closure(&unverified_integrity, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_step3_defects() {
    let mut single_run = valid_release_status_closure_fixture();
    single_run.step3_dual_platform_gates.twice_without_cleanup = false;
    let error = validate_release_status_closure(&single_run, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
    assert!(error.to_string().contains("smoke checks are prohibited"));

    let mut missing_arm64 = valid_release_status_closure_fixture();
    missing_arm64.step3_dual_platform_gates.platforms = vec!["linux/amd64".to_owned()];
    let error = validate_release_status_closure(&missing_arm64, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_step4_defects() {
    let mut unbound_foundry = valid_release_status_closure_fixture();
    unbound_foundry
        .step4_functional_core_and_cargo
        .foundry_bound = false;
    let error = validate_release_status_closure(&unbound_foundry, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut unverified_core = valid_release_status_closure_fixture();
    unverified_core
        .step4_functional_core_and_cargo
        .functional_core_verified = false;
    let error = validate_release_status_closure(&unverified_core, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut unverified_crates_io = valid_release_status_closure_fixture();
    unverified_crates_io
        .step4_functional_core_and_cargo
        .crates_io_bootstrap_verified = false;
    let error = validate_release_status_closure(&unverified_crates_io, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_step5_defects() {
    let mut missing_pages = valid_release_status_closure_fixture();
    missing_pages.step5_downstream_closure.targets_verified =
        vec!["compose".to_owned(), "kubernetes".to_owned()];
    let error = validate_release_status_closure(&missing_pages, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_step6_defects() {
    let mut low_falsification = valid_release_status_closure_fixture();
    low_falsification
        .step6_ecosystem_manifest
        .falsification_classes_verified = 13;
    let error = validate_release_status_closure(&low_falsification, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}
