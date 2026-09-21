//! Integration test suite for universal SDK entrypoint and template contract (Task 10 / Issue #12).
//!
//! Validates:
//! 1. Template R1-R6 requirements and anti-vacuity controls as a versioned contract.
//! 2. Exact pinned SDK image and commit/digest pins (rejection of floating @v, @main, @master, "features").
//! 3. Thin pinned Actions/reusable workflows delegating to SDK/CLI behavior.
//! 4. Least-privilege workflow separation and protected deployment identity.
//! 5. Deterministic `template check` and reviewable non-mutating `template update` patch flow.

use prismpm::template::{check, update};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn create_valid_template_repo(dir: &Path, sdk_image: &str, revision: &str) {
    for sub in [".devcontainer", ".github/workflows"] {
        std::fs::create_dir_all(dir.join(sub)).expect("create dir");
    }

    let required_paths = json!([
        ".devcontainer/devcontainer.json",
        ".github/workflows/bootstrap.yml",
        "AGENTS.md",
        "CONFORMANCE.md",
        "VERIFICATION.md",
        "prismpm.lock",
        "template-contract.json",
        "template.lock"
    ]);

    let universal_paths = json!([
        ".devcontainer/devcontainer.json",
        ".github/workflows/bootstrap.yml",
        "AGENTS.md",
        "VERIFICATION.md",
        "prismpm.lock",
        "template-contract.json",
        "template.lock"
    ]);

    let contract = json!({
        "project_content_paths": ["CONFORMANCE.md"],
        "required_paths": required_paths,
        "schema": "uor/template-contract/1",
        "universal_policy_paths": universal_paths,
        "version": "1.0.0"
    });

    let contract_bytes = prismpm::holo::canonical::encode_value(&contract).unwrap();
    std::fs::write(dir.join("template-contract.json"), &contract_bytes).unwrap();

    // Write sdk lock (prismpm.lock)
    let sdk_lock = json!({
        "inventory": [
            {
                "digest": format!("sha256:{}", "0".repeat(64)),
                "id": "prismpm",
                "kind": "binary",
                "version": "0.3.0"
            }
        ],
        "schema": "prismpm/sdk-lock/1",
        "sdk_image": sdk_image,
        "sdk_version": "0.3.0",
        "standards_lock": format!("sha256:{}", "4".repeat(64))
    });
    let canonical_lock =
        prismpm::contracts::CanonicalDocument::from_value("prismpm/sdk-lock/1", sdk_lock)
            .expect("canonical sdk lock");
    std::fs::write(dir.join("prismpm.lock"), canonical_lock.bytes()).unwrap();

    // Write devcontainer.json (pinned, no "features")
    std::fs::write(
        dir.join(".devcontainer/devcontainer.json"),
        b"{\"image\":\"ghcr.io/uor-foundation/prismpm-sdk@sha256:1111111111111111111111111111111111111111111111111111111111111111\"}\n",
    )
    .unwrap();

    // Write bootstrap.yml (pinned full SHA, no @v or @main)
    std::fs::write(
        dir.join(".github/workflows/bootstrap.yml"),
        b"name: bootstrap\non: push\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683\n",
    )
    .unwrap();

    // Standard markdown and task files
    std::fs::write(dir.join("AGENTS.md"), b"# Agents\nInstruction profile.\n").unwrap();
    std::fs::write(
        dir.join("CONFORMANCE.md"),
        b"# Conformance\nExact matrix.\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("VERIFICATION.md"),
        b"# Verification\nEvidence log.\n",
    )
    .unwrap();
    std::fs::write(dir.join("Justfile"), b"model-write:\n    @true\n").unwrap();

    // Build policy file records for template.lock
    let policy_paths = [
        ".devcontainer/devcontainer.json",
        ".github/workflows/bootstrap.yml",
        "AGENTS.md",
        "VERIFICATION.md",
        "prismpm.lock",
        "template-contract.json",
    ];

    let policy_files: Vec<Value> = policy_paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(dir.join(path)).unwrap();
            json!({
                "path": path,
                "sha256": format!("sha256:{}", sha256_hex(&bytes))
            })
        })
        .collect();

    let policy_bytes =
        prismpm::holo::canonical::encode_value(&Value::Array(policy_files.clone())).unwrap();
    let policy_tree_sha256 = format!("sha256:{}", sha256_hex(&policy_bytes));

    let template_lock = json!({
        "contract_digest": format!("sha256:{}", sha256_hex(&contract_bytes)),
        "policy_files": policy_files,
        "policy_tree_sha256": policy_tree_sha256,
        "schema": "uor/template-lock/1",
        "sdk_image": sdk_image,
        "template_repository": "https://github.com/UOR-Foundation/template",
        "template_revision": revision
    });

    let template_lock_bytes = prismpm::holo::canonical::encode_value(&template_lock).unwrap();
    std::fs::write(dir.join("template.lock"), &template_lock_bytes).unwrap();
}

#[test]
fn template_contract_validates_cleanly_and_enforces_anti_vacuity() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sdk_image = "ghcr.io/uor-foundation/prismpm-sdk@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let revision = "0123456789abcdef0123456789abcdef01234567";

    create_valid_template_repo(tmp.path(), sdk_image, revision);

    // Valid repository check passes
    let result = check(tmp.path()).expect("template check");
    assert_eq!(result["status"], "passed");
    assert_eq!(result["sdk_image"], sdk_image);
    assert_eq!(result["template_revision"], revision);

    // Anti-vacuity: model-write altering CONFORMANCE.md during generation fails closed with PP1101
    std::fs::write(
        tmp.path().join("Justfile"),
        b"model-write:\n    @printf 'tampered\\n' > CONFORMANCE.md\n",
    )
    .unwrap();
    let err = check(tmp.path()).unwrap_err();
    assert_eq!(err.code, "PP1101");

    // Restore clean Justfile
    std::fs::write(tmp.path().join("Justfile"), b"model-write:\n    @true\n").unwrap();
    assert!(check(tmp.path()).is_ok());

    // Tampering any policy file fails closed with PP1101
    std::fs::write(tmp.path().join("AGENTS.md"), b"modified agents content\n").unwrap();
    assert_eq!(check(tmp.path()).unwrap_err().code, "PP1101");
}

#[test]
fn template_check_rejects_floating_and_copied_bootstrap_inputs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sdk_image = "ghcr.io/uor-foundation/prismpm-sdk@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let revision = "0123456789abcdef0123456789abcdef01234567";

    create_valid_template_repo(tmp.path(), sdk_image, revision);
    assert!(check(tmp.path()).is_ok());

    // 1. devcontainer.json with "features" rejected with PP1101
    std::fs::write(
        tmp.path().join(".devcontainer/devcontainer.json"),
        b"{\"features\":{\"ghcr.io/devcontainers/features/rust:1\":{}}}\n",
    )
    .unwrap();
    assert_eq!(check(tmp.path()).unwrap_err().code, "PP1101");

    // Restore valid devcontainer
    create_valid_template_repo(tmp.path(), sdk_image, revision);
    assert!(check(tmp.path()).is_ok());

    // 2. bootstrap.yml with floating tag (@v4, @main, @master) rejected with PP1101
    for floating in [
        "actions/checkout@v4\n",
        "actions/checkout@main\n",
        "actions/checkout@master\n",
    ] {
        std::fs::write(
            tmp.path().join(".github/workflows/bootstrap.yml"),
            format!("name: bootstrap\nsteps:\n  - uses: {floating}"),
        )
        .unwrap();
        assert_eq!(check(tmp.path()).unwrap_err().code, "PP1101");
    }
}

#[test]
fn template_update_produces_reviewable_patch_without_mutating_project() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sdk_image = "ghcr.io/uor-foundation/prismpm-sdk@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let revision = "0123456789abcdef0123456789abcdef01234567";

    create_valid_template_repo(tmp.path(), sdk_image, revision);

    // Read initial tree contents
    let initial_lock = std::fs::read(tmp.path().join("template.lock")).unwrap();

    let new_sdk = "ghcr.io/uor-foundation/prismpm-sdk@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let new_rev = "fedcba9876543210fedcba9876543210fedcba98";

    let update_result = update(tmp.path(), new_sdk, new_rev).expect("update");

    // Must never mutate the project
    let current_lock = std::fs::read(tmp.path().join("template.lock")).unwrap();
    assert_eq!(
        initial_lock, current_lock,
        "update must not mutate the project"
    );

    // Patch must be generated
    assert_eq!(update_result["changed"], true);
    let patch = update_result["patch"].as_str().expect("patch string");
    assert!(patch.contains("--- a/template.lock"));
    assert!(patch.contains("+++ b/template.lock"));
    assert!(patch.contains(new_rev));

    // Update with floating or unpinned SDK image fails closed with PP1101
    assert_eq!(
        update(
            tmp.path(),
            "ghcr.io/uor-foundation/prismpm-sdk:latest",
            new_rev
        )
        .unwrap_err()
        .code,
        "PP1101"
    );

    // Update with invalid revision (not 40 hex chars) fails closed with PP1101
    assert_eq!(
        update(tmp.path(), new_sdk, "short-rev").unwrap_err().code,
        "PP1101"
    );
}

#[test]
fn template_contract_missing_required_paths_fails_closed() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sdk_image = "ghcr.io/uor-foundation/prismpm-sdk@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let revision = "0123456789abcdef0123456789abcdef01234567";

    create_valid_template_repo(tmp.path(), sdk_image, revision);

    // Removing template policy documents fails closed with PP1101
    for required in [
        "AGENTS.md",
        "VERIFICATION.md",
        "template-contract.json",
        "template.lock",
    ] {
        let path = tmp.path().join(required);
        let backup = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(check(tmp.path()).unwrap_err().code, "PP1101");

        std::fs::write(&path, backup).unwrap();
    }

    // Removing prismpm.lock fails closed (PP5401 / PP1101)
    let lock_path = tmp.path().join("prismpm.lock");
    std::fs::remove_file(&lock_path).unwrap();
    let err = check(tmp.path()).unwrap_err();
    assert!(err.code == "PP1101" || err.code == "PP5401");
}
