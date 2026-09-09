//! Read-only UOR template-contract and reviewable update operations.

use crate::contracts::CanonicalDocument;
use crate::error::PrismError;
use crate::holo::canonical::{decode_value, encode_value};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

const REQUIRED: [&str; 8] = [
    ".devcontainer/devcontainer.json",
    ".github/workflows/bootstrap.yml",
    "AGENTS.md",
    "CONFORMANCE.md",
    "VERIFICATION.md",
    "prismpm.lock",
    "template-contract.json",
    "template.lock",
];

const UNIVERSAL_POLICY: [&str; 7] = [
    ".devcontainer/devcontainer.json",
    ".github/workflows/bootstrap.yml",
    "AGENTS.md",
    "VERIFICATION.md",
    "prismpm.lock",
    "template-contract.json",
    "template.lock",
];

const PROJECT_CONTENT: [&str; 1] = ["CONFORMANCE.md"];

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn read_canonical(root: &Path, name: &str) -> Result<(Vec<u8>, Value), PrismError> {
    let bytes = std::fs::read(root.join(name))
        .map_err(|_| PrismError::new("PP1101", format!("{name} is required")))?;
    let value =
        decode_value(&bytes, name).map_err(|error| PrismError::new("PP1101", error.message))?;
    if encode_value(&value).map_err(|error| PrismError::new("PP1101", error.message))? != bytes {
        return Err(PrismError::new(
            "PP1101",
            format!("{name} is not canonical JSON"),
        ));
    }
    Ok((bytes, value))
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn path_array<'a>(value: &'a Value, field: &str) -> Result<Vec<&'a str>, PrismError> {
    let values = value[field]
        .as_array()
        .ok_or_else(|| PrismError::new("PP1101", format!("{field} is not an array")))?;
    let paths = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|path| {
                    !path.is_empty()
                        && !path.starts_with('/')
                        && !path.contains('\\')
                        && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
                })
                .ok_or_else(|| {
                    PrismError::new("PP1101", format!("{field} contains a non-confined path"))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if paths
        .windows(2)
        .any(|pair| pair[0].as_bytes() >= pair[1].as_bytes())
    {
        return Err(PrismError::new(
            "PP1101",
            format!("{field} is duplicate or noncanonical"),
        ));
    }
    Ok(paths)
}

fn validate_contract(value: &Value) -> Result<(), PrismError> {
    let exact_keys = [
        "project_content_paths",
        "required_paths",
        "schema",
        "universal_policy_paths",
        "version",
    ];
    let object = value
        .as_object()
        .ok_or_else(|| PrismError::new("PP1101", "template contract is not an object"))?;
    if object.keys().map(String::as_str).ne(exact_keys) {
        return Err(PrismError::new(
            "PP1101",
            "template contract has unknown, missing, or noncanonical fields",
        ));
    }
    let required = path_array(value, "required_paths")?;
    let universal = path_array(value, "universal_policy_paths")?;
    let project = path_array(value, "project_content_paths")?;
    let mut union = universal.clone();
    union.extend(project.iter().copied());
    union.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    if value["schema"] != "uor/template-contract/1"
        || value["version"] != "1.0.0"
        || required != REQUIRED
        || universal != UNIVERSAL_POLICY
        || project != PROJECT_CONTENT
        || union != required
        || union.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(PrismError::new(
            "PP1101",
            "template contract does not match uor/template-contract/1",
        ));
    }
    Ok(())
}

fn validate_lock(
    root: &Path,
    value: &Value,
    contract: &Value,
    contract_digest: &str,
    sdk_image: &str,
) -> Result<(), PrismError> {
    let exact_keys = [
        "contract_digest",
        "policy_files",
        "policy_tree_sha256",
        "schema",
        "sdk_image",
        "template_repository",
        "template_revision",
    ];
    let object = value
        .as_object()
        .ok_or_else(|| PrismError::new("PP1101", "template lock is not an object"))?;
    let policy_files = value["policy_files"].as_array();
    if object.keys().map(String::as_str).ne(exact_keys)
        || value["schema"] != "uor/template-lock/1"
        || value["contract_digest"] != contract_digest
        || value["sdk_image"] != sdk_image
        || value["template_repository"] != "https://github.com/UOR-Foundation/template"
        || !value["template_revision"].as_str().is_some_and(|revision| {
            revision.len() == 40
                && revision
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        })
        || policy_files.is_none()
    {
        return Err(PrismError::new(
            "PP1101",
            "template lock is stale, floating, or malformed",
        ));
    }
    let policy_files = policy_files.expect("checked");
    let expected_paths = path_array(contract, "universal_policy_paths")?
        .into_iter()
        .filter(|path| *path != "template.lock")
        .collect::<Vec<_>>();
    if policy_files.len() != expected_paths.len() {
        return Err(PrismError::new(
            "PP1101",
            "template policy file closure is incomplete",
        ));
    }
    for (row, expected_path) in policy_files.iter().zip(expected_paths) {
        let Some(row) = row.as_object() else {
            return Err(PrismError::new(
                "PP1101",
                "template policy row is malformed",
            ));
        };
        if row.keys().map(String::as_str).ne(["path", "sha256"])
            || row["path"] != expected_path
            || row["sha256"]
                .as_str()
                .is_none_or(|digest| !valid_digest(digest))
        {
            return Err(PrismError::new(
                "PP1101",
                "template policy row is malformed",
            ));
        }
        let bytes = std::fs::read(root.join(expected_path)).map_err(|_| {
            PrismError::new(
                "PP1101",
                format!("template policy path is absent: {expected_path}"),
            )
        })?;
        if row["sha256"] != sha(&bytes) {
            return Err(PrismError::new(
                "PP1101",
                format!("template policy path changed: {expected_path}"),
            ));
        }
    }
    let policy_bytes = encode_value(&Value::Array(policy_files.clone()))?;
    if value["policy_tree_sha256"] != sha(&policy_bytes) {
        return Err(PrismError::new(
            "PP1101",
            "template policy tree digest does not match its file closure",
        ));
    }
    Ok(())
}

fn verify_native_regeneration(root: &Path) -> Result<(), PrismError> {
    let before = std::fs::read(root.join("CONFORMANCE.md"))
        .map_err(|_| PrismError::new("PP1101", "CONFORMANCE.md is required"))?;
    let staging = tempfile::tempdir()
        .map_err(|error| PrismError::new("PP1101", format!("template staging: {error}")))?;
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry
            .map_err(|error| PrismError::new("PP1101", format!("template staging: {error}")))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|_| PrismError::new("PP1101", "template staging path escaped"))?;
        if relative.components().any(|component| {
            matches!(
                component.as_os_str().to_str(),
                Some(".git" | ".prism" | "node_modules" | "target")
            )
        }) {
            continue;
        }
        if entry.file_type().is_symlink() {
            return Err(PrismError::new(
                "PP1101",
                "template source contains a symlink",
            ));
        }
        let destination = staging.path().join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&destination)
                .map_err(|error| PrismError::new("PP1101", format!("template staging: {error}")))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|error| {
                    PrismError::new("PP1101", format!("template staging: {error}"))
                })?;
            }
            std::fs::copy(entry.path(), &destination)
                .map_err(|error| PrismError::new("PP1101", format!("template staging: {error}")))?;
        }
    }
    let just = crate::sdk::executable("just")?;
    crate::verification::run_process_limited(
        "template-native-regeneration",
        &just,
        &["model-write".to_owned()],
        staging.path(),
        &BTreeMap::new(),
        &[(staging.path(), "$PROJECT")],
        "PP1101",
        "120s",
        16 * 1024 * 1024,
    )?;
    let after = std::fs::read(staging.path().join("CONFORMANCE.md")).map_err(|_| {
        PrismError::new(
            "PP1101",
            "native regeneration did not produce CONFORMANCE.md",
        )
    })?;
    if before != after {
        return Err(PrismError::new(
            "PP1101",
            "project-owned CONFORMANCE.md is stale after native regeneration",
        ));
    }
    Ok(())
}

/// Verify the universal template contract without changing the repository.
pub fn check(root: &Path) -> Result<Value, PrismError> {
    for path in REQUIRED {
        if !root.join(path).is_file() {
            return Err(PrismError::new(
                "PP1101",
                format!("template-required path is absent: {path}"),
            ));
        }
    }
    let (contract_bytes, contract) = read_canonical(root, "template-contract.json")?;
    validate_contract(&contract)?;
    let sdk_bytes = std::fs::read(root.join("prismpm.lock"))
        .map_err(|_| PrismError::new("PP5401", "prismpm.lock is required"))?;
    let sdk = CanonicalDocument::parse("prismpm/sdk-lock/1", &sdk_bytes)?;
    let (_, lock) = read_canonical(root, "template.lock")?;
    validate_lock(
        root,
        &lock,
        &contract,
        &sha(&contract_bytes),
        sdk.value()["sdk_image"].as_str().unwrap_or_default(),
    )?;
    verify_native_regeneration(root)?;
    for (path, forbidden) in [
        (".devcontainer/devcontainer.json", "\"features\""),
        (".github/workflows/bootstrap.yml", "@v"),
        (".github/workflows/bootstrap.yml", "@main"),
        (".github/workflows/bootstrap.yml", "@master"),
    ] {
        let text = std::fs::read_to_string(root.join(path))
            .map_err(|error| PrismError::new("PP1101", error.to_string()))?;
        if text.contains(forbidden) {
            return Err(PrismError::new(
                "PP1101",
                format!("{path} contains floating or copied bootstrap input {forbidden}"),
            ));
        }
    }
    Ok(json!({
        "contract_digest":sha(&contract_bytes),
        "schema":"uor/template-check-result/1",
        "sdk_image":sdk.value()["sdk_image"],
        "status":"passed",
        "template_revision":lock["template_revision"]
    }))
}

/// Produce a deterministic reviewable lock patch; never mutate the project.
pub fn update(root: &Path, sdk_image: &str, template_revision: &str) -> Result<Value, PrismError> {
    let (contract_bytes, contract) = read_canonical(root, "template-contract.json")?;
    validate_contract(&contract)?;
    if !sdk_image
        .rsplit_once("@sha256:")
        .is_some_and(|(name, digest)| {
            !name.is_empty()
                && digest.len() == 64
                && digest
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        })
        || template_revision.len() != 40
        || !template_revision
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err(PrismError::new(
            "PP1101",
            "template update identity is not immutable",
        ));
    }
    let proposed = json!({
        "contract_digest":sha(&contract_bytes),"schema":"uor/template-lock/1",
        "policy_files":path_array(&contract,"universal_policy_paths")?.into_iter()
            .filter(|path| *path != "template.lock")
            .map(|path| std::fs::read(root.join(path))
                .map(|bytes| json!({"path":path,"sha256":sha(&bytes)}))
                .map_err(|_| PrismError::new("PP1101",format!("template policy path is absent: {path}"))))
            .collect::<Result<Vec<_>,_>>()?,
        "policy_tree_sha256":"",
        "sdk_image":sdk_image,"template_repository":"https://github.com/UOR-Foundation/template",
        "template_revision":template_revision
    });
    let mut proposed = proposed;
    proposed["policy_tree_sha256"] = Value::String(sha(&encode_value(&proposed["policy_files"])?));
    let bytes = encode_value(&proposed)?;
    let current = std::fs::read(root.join("template.lock")).unwrap_or_default();
    let current_text = String::from_utf8_lossy(&current);
    let proposed_text = String::from_utf8_lossy(&bytes);
    Ok(json!({
        "changed":current != bytes,
        "patch":format!("--- a/template.lock\n+++ b/template.lock\n@@ -1 +1 @@\n-{current_text}\n\\ No newline at end of file\n+{proposed_text}\n\\ No newline at end of file\n"),
        "proposed_lock":proposed,
        "schema":"uor/template-update-result/1"
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn update_rejects_mutable_sdk_names() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("template-contract.json"), b"{\"required_paths\":[\".devcontainer/devcontainer.json\",\".github/workflows/bootstrap.yml\",\"AGENTS.md\",\"CONFORMANCE.md\",\"prismpm.lock\",\"template-contract.json\",\"template.lock\",\"VERIFICATION.md\"],\"schema\":\"uor/template-contract/1\",\"version\":\"1.0.0\"}").unwrap();
        assert!(super::update(root.path(), "ghcr.io/example/sdk:latest", &"a".repeat(40)).is_err());
    }
}
