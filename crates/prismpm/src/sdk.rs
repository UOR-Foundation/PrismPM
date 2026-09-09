//! SDK inventory validation and locked consumer-input installation.

use crate::contracts::CanonicalDocument;
use crate::error::PrismError;
use crate::holo::canonical::encode_value;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;
use std::path::PathBuf;
use std::path::{Component, Path};

const STDLIB_SOURCES: &[u8] = include_bytes!("../sdk/stdlib-sources.tar");
const RELEASED_INVENTORY: &str = "/opt/prismpm/share/inventory.json";

pub(crate) fn inventory_path() -> Option<PathBuf> {
    let released = PathBuf::from(RELEASED_INVENTORY);
    if released.is_file() {
        return Some(released);
    }
    std::env::var_os("PRISMPM_SDK_INVENTORY").map(PathBuf::from)
}

fn executable_inventory() -> Result<serde_json::Value, PrismError> {
    let path =
        std::env::var_os("PATH").ok_or_else(|| PrismError::new("PP5401", "SDK PATH is absent"))?;
    let mut seen = BTreeSet::new();
    let mut commands = Vec::new();
    for directory in std::env::split_paths(&path) {
        let mut entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| PrismError::new("PP5401", error.to_string()))?,
            Err(_) => continue,
        };
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let command = entry.file_name().into_string().map_err(|_| {
                PrismError::new("PP5401", "SDK PATH contains a non-UTF-8 command name")
            })?;
            if seen.contains(&command) {
                continue;
            }
            let resolved = match entry.path().canonicalize() {
                Ok(path) => path,
                Err(_) => continue,
            };
            let metadata = match std::fs::metadata(&resolved) {
                Ok(metadata) if metadata.is_file() => metadata,
                _ => continue,
            };
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o111 == 0 {
                    continue;
                }
            }
            let bytes = std::fs::read(&resolved)
                .map_err(|error| PrismError::new("PP5401", error.to_string()))?;
            commands.push(json!({
                "command": command,
                "executable": resolved.to_string_lossy(),
                "sha256": format!("{:x}", Sha256::digest(bytes))
            }));
            seen.insert(command);
        }
    }
    commands.sort_by(|left, right| left["command"].as_str().cmp(&right["command"].as_str()));
    Ok(json!({"commands":commands,"schema":"prismpm/sdk-inventory/1"}))
}

/// Verify that the current executable search path is exactly the signed SDK inventory.
///
/// Source-bootstrap environments have no inventory file and are verified by the
/// repository's independently pinned native gate. A released image discovers its
/// fixed inventory even if a caller removes or overrides the convenience variable.
pub fn verify_environment() -> Result<Option<serde_json::Value>, PrismError> {
    let Some(path) = inventory_path() else {
        return Ok(None);
    };
    let bytes = std::fs::read(&path)
        .map_err(|error| PrismError::new("PP5401", format!("SDK inventory: {error}")))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| PrismError::new("PP5401", format!("SDK inventory: {error}")))?;
    let mut canonical = encode_value(&value)?;
    canonical.push(b'\n');
    if canonical != bytes {
        return Err(PrismError::new("PP5401", "SDK inventory is not canonical"));
    }
    let observed = executable_inventory()?;
    if observed["commands"] != value["commands"]
        || value["schema"] != "prismpm/sdk-inventory/1"
        || value["artifacts"].as_array().is_none_or(|rows| {
            rows.is_empty()
                || rows.windows(2).any(|pair| {
                    pair[0]["id"].as_str().is_none_or(|left| {
                        pair[1]["id"]
                            .as_str()
                            .is_none_or(|right| left.as_bytes() >= right.as_bytes())
                    })
                })
                || rows.iter().any(|row| {
                    row.as_object().is_none_or(|object| {
                        object
                            .keys()
                            .map(String::as_str)
                            .ne(["digest", "id", "kind", "version"])
                    }) || row["digest"].as_str().is_none_or(|digest| {
                        digest.len() != 71
                            || !digest.starts_with("sha256:")
                            || !digest[7..]
                                .bytes()
                                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    }) || row["id"].as_str().is_none_or(str::is_empty)
                        || row["kind"].as_str().is_none_or(str::is_empty)
                        || row["version"].as_str().is_none_or(str::is_empty)
                })
        })
    {
        return Err(PrismError::new(
            "PP5401",
            "SDK inventory or PATH contains missing, changed, shadowed, undeclared, or noncanonical content",
        ));
    }
    Ok(Some(json!({
        "inventory_sha256":format!("{:x}",Sha256::digest(&bytes)),
        "schema":"prismpm/sdk-verification/1",
        "verified":true
    })))
}

/// Resolve one command from the already verified SDK PATH.
///
/// Released images always carry the fixed inventory file; source bootstrap
/// environments are governed by the independently pinned repository gate.
pub fn executable(command: &str) -> Result<PathBuf, PrismError> {
    if command.is_empty()
        || command.len() > 128
        || !command
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(PrismError::new("PP5401", "SDK command name is malformed"));
    }
    let inventory_path = inventory_path();
    let inventory = verify_environment()?;
    let candidate = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|directory| directory.join(command))
        .find(|path| path.is_file())
        .ok_or_else(|| PrismError::new("PP5401", format!("SDK command {command} is absent")))?;
    let resolved = candidate
        .canonicalize()
        .map_err(|error| PrismError::new("PP5401", format!("SDK command {command}: {error}")))?;
    if inventory.is_some() {
        let bytes = std::fs::read(
            inventory_path.ok_or_else(|| PrismError::new("PP5401", "SDK inventory is absent"))?,
        )
        .map_err(|error| PrismError::new("PP5401", error.to_string()))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| PrismError::new("PP5401", error.to_string()))?;
        let expected = value["commands"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|row| row["command"] == command)
            .ok_or_else(|| {
                PrismError::new("PP5401", format!("SDK command {command} is undeclared"))
            })?;
        let observed_sha = format!(
            "{:x}",
            Sha256::digest(
                std::fs::read(&resolved)
                    .map_err(|error| PrismError::new("PP5401", error.to_string()))?
            )
        );
        if expected["executable"] != resolved.to_string_lossy().as_ref()
            || expected["sha256"] != observed_sha
        {
            return Err(PrismError::new(
                "PP5401",
                format!("SDK command {command} disagrees with its signed inventory"),
            ));
        }
    }
    // Preserve the selected command path for multi-call binaries and shims
    // (notably rustup's cargo/rustc links). Executing the canonical target
    // changes argv[0] semantics and can invoke the wrong applet. The target
    // digest and canonical identity were verified above.
    Ok(candidate)
}

fn tree(root: &Path) -> Result<BTreeMap<String, String>, PrismError> {
    let mut rows = BTreeMap::new();
    if !root.exists() {
        return Ok(rows);
    }
    for entry in walkdir::WalkDir::new(root) {
        let entry = entry.map_err(|error| PrismError::new("PP5401", error.to_string()))?;
        if entry.file_type().is_symlink() {
            return Err(PrismError::new(
                "PP5401",
                "SDK source tree contains a symlink",
            ));
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|_| PrismError::new("PP5401", "SDK source path escaped"))?
            .to_string_lossy()
            .replace('\\', "/");
        rows.insert(
            relative,
            format!(
                "{:x}",
                Sha256::digest(
                    std::fs::read(entry.path())
                        .map_err(|error| PrismError::new("PP5401", error.to_string()))?
                )
            ),
        );
    }
    Ok(rows)
}

fn extract(destination: &Path) -> Result<(), PrismError> {
    let mut archive = tar::Archive::new(Cursor::new(STDLIB_SOURCES));
    for entry in archive
        .entries()
        .map_err(|error| PrismError::new("PP5401", format!("SDK archive: {error}")))?
    {
        let mut entry =
            entry.map_err(|error| PrismError::new("PP5401", format!("SDK archive: {error}")))?;
        let path = entry
            .path()
            .map_err(|error| PrismError::new("PP5401", format!("SDK archive path: {error}")))?;
        if path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
            || !(entry.header().entry_type().is_file() || entry.header().entry_type().is_dir())
        {
            return Err(PrismError::new("PP5401", "SDK archive entry is unsafe"));
        }
        entry
            .unpack_in(destination)
            .map_err(|error| PrismError::new("PP5401", format!("SDK archive: {error}")))?;
    }
    Ok(())
}

/// Install exact embedded model sources under ignored, content-checked SDK state.
pub fn install_project_inputs(root: &Path) -> Result<serde_json::Value, PrismError> {
    let parent = root.join(".prism/sdk");
    std::fs::create_dir_all(&parent)
        .map_err(|error| PrismError::new("PP5401", error.to_string()))?;
    let expected_temp = tempfile::Builder::new()
        .prefix("stdlib-expected-")
        .tempdir_in(&parent)
        .map_err(|error| PrismError::new("PP5401", error.to_string()))?;
    extract(expected_temp.path())?;
    let expected = tree(expected_temp.path())?;
    let destination = parent.join("inputs");
    if destination.exists() {
        if tree(&destination)? != expected {
            return Err(PrismError::new(
                "PP5401",
                "installed SDK standard-library sources are stale or modified",
            ));
        }
    } else {
        std::fs::rename(expected_temp.path(), &destination)
            .map_err(|error| PrismError::new("PP5401", format!("SDK publish: {error}")))?;
    }
    let files = expected
        .into_iter()
        .map(|(path, sha256)| json!({"path":path,"sha256":sha256}))
        .collect::<Vec<_>>();
    let manifest = json!({
        "archive_sha256":format!("{:x}",Sha256::digest(STDLIB_SOURCES)),
        "files":files,
        "schema":"prismpm/sdk-inputs/1"
    });
    let bytes = encode_value(&manifest)?;
    let path = parent.join("stdlib-manifest.json");
    if path.exists() {
        if std::fs::read(&path).map_err(|error| PrismError::new("PP5401", error.to_string()))?
            != bytes
        {
            return Err(PrismError::new("PP5401", "SDK input manifest changed"));
        }
    } else {
        std::fs::write(&path, bytes)
            .map_err(|error| PrismError::new("PP5401", error.to_string()))?;
    }
    Ok(manifest)
}

/// Acquire project package-manager inputs during the explicit online fetch phase.
///
/// Subsequent Prism commands never perform this operation implicitly. Both
/// package managers are invoked without a shell and only when their committed
/// lock and project manifest are present.
pub fn fetch_project_dependencies(root: &Path) -> Result<Vec<serde_json::Value>, PrismError> {
    let mut records = Vec::new();
    if root.join("Cargo.toml").is_file() && root.join("Cargo.lock").is_file() {
        let cargo = executable("cargo")?;
        let record = crate::verification::run_process(
            "cargo-fetch",
            &cargo,
            &["fetch".to_owned(), "--locked".to_owned()],
            root,
            &BTreeMap::new(),
            &[(root, "$PROJECT")],
            "PP5402",
        )?;
        records.push(
            serde_json::to_value(record)
                .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
        );
    }
    if root.join("package.json").is_file() && root.join("package-lock.json").is_file() {
        let npm = executable("npm")?;
        let record = crate::verification::run_process(
            "npm-ci",
            &npm,
            &[
                "ci".to_owned(),
                "--ignore-scripts".to_owned(),
                "--no-audit".to_owned(),
                "--no-fund".to_owned(),
            ],
            root,
            &BTreeMap::new(),
            &[(root, "$PROJECT")],
            "PP5402",
        )?;
        records.push(
            serde_json::to_value(record)
                .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
        );
    }
    Ok(records)
}

/// Verify and inspect the committed SDK lock without changing it.
pub fn inspect_lock(root: &Path) -> Result<serde_json::Value, PrismError> {
    let bytes = std::fs::read(root.join("prismpm.lock"))
        .map_err(|_| PrismError::new("PP5401", "prismpm.lock is absent"))?;
    Ok(CanonicalDocument::parse("prismpm/sdk-lock/1", &bytes)?
        .value()
        .clone())
}

/// Produce an RFC 6902-style review object for an explicit SDK-lock update.
/// The command is intentionally read-only; applying the patch remains a normal
/// reviewed source change.
pub fn propose_lock_update(
    root: &Path,
    sdk_image: &str,
    standards_lock: &str,
) -> Result<serde_json::Value, PrismError> {
    crate::oci::validate_reference(sdk_image, true)?;
    let standards = standards_lock
        .strip_prefix("sha256:")
        .filter(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
        .ok_or_else(|| PrismError::new("PP5401", "standards-lock digest is malformed"))?;
    let current = inspect_lock(root)?;
    let mut proposed = current.clone();
    proposed["sdk_image"] = serde_json::Value::String(sdk_image.to_owned());
    proposed["standards_lock"] = serde_json::Value::String(format!("sha256:{standards}"));
    let proposed = CanonicalDocument::from_value("prismpm/sdk-lock/1", proposed)?;
    let changes = ["sdk_image", "standards_lock"]
        .into_iter()
        .filter(|field| current[*field] != proposed.value()[*field])
        .map(|field| {
            json!({
                "from":current[field],
                "op":"replace",
                "path":format!("/{field}"),
                "to":proposed.value()[field]
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "changes":changes,
        "compatibility_review":"required",
        "generated_output_diff":"required",
        "proposed_lock":proposed.value(),
        "schema":"prismpm/sdk-lock-update/1",
        "security_review":"required"
    }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn installation_is_idempotent_and_detects_mutation() {
        let root = tempfile::tempdir().unwrap();
        let first = super::install_project_inputs(root.path()).unwrap();
        let second = super::install_project_inputs(root.path()).unwrap();
        assert_eq!(first, second);
        std::fs::write(
            root.path()
                .join(".prism/sdk/inputs/stdlib/Production/Core.lex.tex"),
            b"changed",
        )
        .unwrap();
        assert!(super::install_project_inputs(root.path()).is_err());
    }
}
