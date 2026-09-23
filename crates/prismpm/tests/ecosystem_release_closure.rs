//! Cross-repository acceptance, falsification completeness, and Task 12 ecosystem release closure tests.

use prismpm::acceptance::{validate_ecosystem_release_closure, REQUIRED_FALSIFICATION_CLASSES};
use serde_json::{json, Value};

fn digest(c: char) -> String {
    format!("sha256:{}", c.to_string().repeat(64))
}

fn artifact(name: &str, c: char) -> Value {
    json!({
        "digest": digest(c),
        "media_type": "application/gzip",
        "name": name,
        "public_url": format!("https://example.invalid/artifacts/{name}.tar.gz"),
        "size": 1024
    })
}

fn valid_ecosystem_manifest_fixture() -> Value {
    let repo_names = [
        "LexLean",
        "PrismPM",
        "calculator-example",
        "lean4-prod",
        "template",
    ];

    let mut artifacts = vec![artifact("ecosystem-release", 'd')];
    for (i, name) in repo_names.iter().enumerate() {
        let ch = char::from_digit((i as u32) + 1, 10).unwrap();
        artifacts.push(artifact(&format!("{name}-source"), ch));
    }

    let repositories = repo_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let ch = char::from_digit((i as u32) + 1, 10).unwrap();
            json!({
                "commit": "0123456789abcdef0123456789abcdef01234567",
                "name": name,
                "source_archive": artifact(&format!("{name}-source"), ch),
                "tag": "v0.3.0",
                "url": format!("https://github.com/example/{name}")
            })
        })
        .collect::<Vec<_>>();

    let mut evidence = [
        "acceptance",
        "backup-restore",
        "conformance",
        "deployment",
        "drift",
        "migration",
        "provenance",
        "recovery",
        "rollback",
        "sbom",
        "signature",
        "slo",
        "verification",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, kind)| {
        json!({
            "ci_run": null,
            "digest": digest(char::from_digit((index % 8 + 1) as u32, 10).unwrap()),
            "kind": kind,
            "path": format!("evidence/{index:02}-{kind}.json"),
            "subject_digest": digest('a')
        })
    })
    .collect::<Vec<_>>();

    for defect_class in REQUIRED_FALSIFICATION_CLASSES {
        evidence.push(json!({
            "ci_run": null,
            "digest": digest('f'),
            "kind": "falsification",
            "path": format!("evidence/falsification/{defect_class}.json"),
            "subject_digest": digest('a')
        }));
    }

    artifacts.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    evidence.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));

    let sdk_digest = digest('b');
    let release_a = digest('a');
    let release_b = digest('c');

    json!({
        "actions": [{
            "path": ".github/actions/prismpm/action.yml",
            "repository": "UOR-Foundation/PrismPM",
            "revision": "0123456789abcdef0123456789abcdef01234567",
            "sdk_digest": sdk_digest
        }],
        "artifacts": artifacts,
        "calculator": {
            "application_baseline_digest": digest('e'),
            "coverage_digest": digest('f'),
            "pages": {
                "assets": (0..6).map(|index| artifact(&format!("asset-{index}"), '1')).collect::<Vec<_>>(),
                "commit": "0123456789abcdef0123456789abcdef01234567",
                "url": "https://example.invalid/calculator/"
            },
            "system_releases": [{
                "deployment_evidence": digest('2'),
                "label": "A",
                "product_digest": release_a,
                "reference": format!("ghcr.io/example/calculator-a@{release_a}"),
                "referrers": [digest('1'), digest('2')]
            }, {
                "deployment_evidence": digest('3'),
                "label": "B",
                "product_digest": release_b,
                "reference": format!("ghcr.io/example/calculator-b@{release_b}"),
                "referrers": [digest('3'), digest('4')]
            }]
        },
        "evidence": evidence,
        "packages": [{
            "checksum": "1111111111111111111111111111111111111111111111111111111111111111",
            "name": "prism-calculator",
            "registry": "https://crates.io/crates/prism-calculator",
            "version": "0.1.0"
        }, {
            "checksum": "2222222222222222222222222222222222222222222222222222222222222222",
            "name": "prism-stdlib",
            "registry": "https://crates.io/crates/prism-stdlib",
            "version": "0.2.0"
        }, {
            "checksum": "3333333333333333333333333333333333333333333333333333333333333333",
            "name": "prismpm",
            "registry": "https://crates.io/crates/prismpm",
            "version": "0.3.0"
        }],
        "repositories": repositories,
        "schema": "prismpm/ecosystem-release/2",
        "sdk": {
            "index_digest": sdk_digest,
            "inventory_digest": digest('6'),
            "native_archives": [artifact("aarch64", '7'), artifact("x86_64", '8')],
            "platform_manifests": [
                {"architecture": "amd64", "digest": digest('7'), "os": "linux"},
                {"architecture": "arm64", "digest": digest('8'), "os": "linux"}
            ],
            "reference": format!("ghcr.io/example/prismpm-sdk@{sdk_digest}"),
            "version": "0.3.0"
        },
        "standards_lock_digest": digest('9'),
        "status": "accepted",
        "template": {
            "contract_digest": digest('4'),
            "instantiation_evidence": digest('5'),
            "repository_commit": "0123456789abcdef0123456789abcdef01234567"
        },
        "version": "2"
    })
}

#[test]
fn valid_ecosystem_release_closure_passes() {
    let manifest = valid_ecosystem_manifest_fixture();
    let receipt = validate_ecosystem_release_closure(&manifest, 2_000)
        .expect("ecosystem release manifest must pass validation");

    assert_eq!(receipt["schema"], "prismpm/ecosystem-release-receipt/2");
    assert_eq!(receipt["result"], "verified");
    assert_eq!(receipt["status"], "passed");
    assert_eq!(receipt["repository_count"], 5);
    assert_eq!(receipt["package_count"], 3);
    assert_eq!(receipt["falsification_classes_verified"], 14);
    assert_eq!(receipt["verified_at_unix"], 2_000);
}

#[test]
fn rejects_unaccepted_status_or_invalid_schema() {
    let mut bad_status = valid_ecosystem_manifest_fixture();
    bad_status["status"] = json!("draft");
    let error = validate_ecosystem_release_closure(&bad_status, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut bad_schema = valid_ecosystem_manifest_fixture();
    bad_schema["schema"] = json!("prismpm/ecosystem-release/1");
    let error = validate_ecosystem_release_closure(&bad_schema, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_invalid_repository_commit() {
    let mut bad_commit = valid_ecosystem_manifest_fixture();
    bad_commit["repositories"][0]["commit"] = json!("not-a-40-hex-commit");
    let error = validate_ecosystem_release_closure(&bad_commit, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}

#[test]
fn rejects_missing_source_archive_in_artifacts() {
    let mut missing_archive = valid_ecosystem_manifest_fixture();
    // Remove all artifacts except the first
    missing_archive["artifacts"]
        .as_array_mut()
        .unwrap()
        .truncate(1);
    let error = validate_ecosystem_release_closure(&missing_archive, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
    assert!(error.to_string().contains("is absent from artifacts"));
}

#[test]
fn rejects_zero_size_artifact() {
    let mut zero_size = valid_ecosystem_manifest_fixture();
    zero_size["artifacts"][0]["size"] = json!(0);
    let error = validate_ecosystem_release_closure(&zero_size, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
    assert!(error.to_string().contains("has zero size"));
}

#[test]
fn rejects_identical_calculator_releases_a_and_b() {
    let mut identical_ab = valid_ecosystem_manifest_fixture();
    let release_a_digest =
        identical_ab["calculator"]["system_releases"][0]["product_digest"].clone();
    identical_ab["calculator"]["system_releases"][1]["product_digest"] = release_a_digest.clone();
    identical_ab["calculator"]["system_releases"][1]["reference"] = json!(format!(
        "ghcr.io/example/calculator-b@{}",
        release_a_digest.as_str().unwrap()
    ));
    let error = validate_ecosystem_release_closure(&identical_ab, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
    assert!(error
        .to_string()
        .contains("must have distinct product digests"));
}

#[test]
fn rejects_calculator_release_reusing_application_baseline() {
    let mut reuse_baseline = valid_ecosystem_manifest_fixture();
    let baseline = reuse_baseline["calculator"]["application_baseline_digest"].clone();
    reuse_baseline["calculator"]["system_releases"][0]["product_digest"] = baseline.clone();
    reuse_baseline["calculator"]["system_releases"][0]["reference"] = json!(format!(
        "ghcr.io/example/calculator-a@{}",
        baseline.as_str().unwrap()
    ));
    let error = validate_ecosystem_release_closure(&reuse_baseline, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
    assert!(error
        .to_string()
        .contains("must not reuse the application baseline digest"));
}

#[test]
fn rejects_missing_falsification_defect_class() {
    let mut missing_falsification = valid_ecosystem_manifest_fixture();
    // Remove the falsification entry for 'failed-restore'
    missing_falsification["evidence"]
        .as_array_mut()
        .unwrap()
        .retain(|ev| {
            !ev["path"]
                .as_str()
                .unwrap_or_default()
                .contains("failed-restore")
        });
    let error = validate_ecosystem_release_closure(&missing_falsification, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
    assert!(error.to_string().contains("failed-restore"));
}

#[test]
fn rejects_noncanonical_platform_or_native_archive_ordering() {
    let mut reordered_platforms = valid_ecosystem_manifest_fixture();
    reordered_platforms["sdk"]["platform_manifests"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let error = validate_ecosystem_release_closure(&reordered_platforms, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");

    let mut reordered_archives = valid_ecosystem_manifest_fixture();
    reordered_archives["sdk"]["native_archives"]
        .as_array_mut()
        .unwrap()
        .reverse();
    let error = validate_ecosystem_release_closure(&reordered_archives, 2_000).unwrap_err();
    assert_eq!(error.code, "PP6004");
}
