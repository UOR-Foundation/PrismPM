//! SDK inventory validation and locked consumer-input installation.

use crate::contracts::CanonicalDocument;
use crate::error::PrismError;
use crate::holo::canonical::encode_value;
pub use crate::supply_chain::{
    validate_sdk_security_disposition, GraphBinding, LauncherBinding, LockBinding,
    PlatformInventoryBinding, RuntimeBinding, SdkAdvisoryPolicy, SdkSecurityDisposition,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;
use std::path::PathBuf;
use std::path::{Component, Path};

const STDLIB_SOURCES: &[u8] = include_bytes!("../sdk/stdlib-sources.tar");
const RELEASED_INVENTORY: &str = "/opt/prismpm/share/inventory.json";
const SDK_INVENTORY_MAX_BYTES: usize = 8 * 1024 * 1024;
const SDK_INDEX_MAX_BYTES: usize = 1024 * 1024;
// Two exact inventory documents can double when JSON-string escaped, and
// their artifact rows also appear separately. Allow the bounded index and
// envelope too; the per-document limits are still enforced independently.
const SDK_CAPTURE_MAX_BYTES: usize = 64 * 1024 * 1024;

fn select_inventory_path(
    released: &Path,
    installed_marker: &Path,
    configured: Option<PathBuf>,
) -> Option<PathBuf> {
    // The fixed installed SDK wins even when its inventory is missing or
    // malformed: deleting it must not turn a released image into bootstrap.
    if installed_marker.exists() || released.exists() {
        return Some(released.to_owned());
    }
    configured
}

pub(crate) fn inventory_path() -> Option<PathBuf> {
    select_inventory_path(
        Path::new(RELEASED_INVENTORY),
        Path::new("/etc/profile.d/prismpm-sdk.sh"),
        std::env::var_os("PRISMPM_SDK_INVENTORY").map(PathBuf::from),
    )
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

/// Parse either supported exact-major SDK lock; never coerce the legacy format.
pub fn parse_lock(bytes: &[u8]) -> Result<CanonicalDocument, PrismError> {
    if bytes.len() > SDK_CAPTURE_MAX_BYTES {
        return Err(PrismError::new(
            "PP7601",
            "SDK lock exceeds its 67108864 byte limit",
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| PrismError::new("PP5401", format!("SDK lock JSON: {error}")))?;
    let schema = match value["schema"].as_str() {
        Some("prismpm/sdk-lock/1") => "prismpm/sdk-lock/1",
        Some("prismpm/sdk-lock/2") => "prismpm/sdk-lock/2",
        _ => return Err(PrismError::new("PP5401", "unsupported SDK lock schema")),
    };
    CanonicalDocument::parse(schema, bytes)
}

pub(crate) fn validate_platform_lock(value: &serde_json::Value) -> Result<(), PrismError> {
    let fail = || {
        PrismError::new(
            "PP5401",
            "SDK platform inventory or index binding is invalid",
        )
    };
    let bytes = value["sdk_index"].as_str().ok_or_else(fail)?.as_bytes();
    if bytes.len() > SDK_INDEX_MAX_BYTES {
        return Err(fail());
    }
    let index_digest = format!("sha256:{:x}", Sha256::digest(bytes));
    if value["sdk_image"]
        .as_str()
        .and_then(|image| image.rsplit_once('@'))
        .map(|(_, digest)| digest)
        != Some(index_digest.as_str())
    {
        return Err(fail());
    }
    let index: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| fail())?;
    let manifests = index["manifests"].as_array().ok_or_else(fail)?;
    let platforms = value["platforms"].as_array().ok_or_else(fail)?;
    if index["schemaVersion"] != 2
        || index["mediaType"] != "application/vnd.oci.image.index.v1+json"
        || manifests.len() != 2
        || platforms.len() != 2
    {
        return Err(fail());
    }
    let mut expected_identities = None;
    let mut digests = BTreeSet::new();
    for (row, architecture) in platforms.iter().zip(["amd64", "arm64"]) {
        if row["platform"] != format!("linux/{architecture}")
            || !digests.insert(row["manifest_digest"].as_str().ok_or_else(fail)?)
        {
            return Err(fail());
        }
        let matching = manifests
            .iter()
            .filter(|manifest| {
                manifest["platform"]["os"] == "linux"
                    && manifest["platform"]["architecture"] == architecture
            })
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(fail());
        }
        let manifest = matching[0];
        if manifest["digest"] != row["manifest_digest"]
            || manifest["mediaType"] != "application/vnd.oci.image.manifest.v1+json"
            || manifest["size"].as_u64().is_none_or(|size| size == 0)
            || !(manifest["platform"]["variant"].is_null()
                || (architecture == "arm64" && manifest["platform"]["variant"] == "v8"))
        {
            return Err(fail());
        }
        let inventory = row["inventory"].as_array().ok_or_else(fail)?;
        let document = row["inventory_document"].as_str().ok_or_else(fail)?;
        if document.len() > SDK_INVENTORY_MAX_BYTES
            || row["inventory_digest"]
                != format!("sha256:{:x}", Sha256::digest(document.as_bytes()))
        {
            return Err(fail());
        }
        let decoded: serde_json::Value = serde_json::from_str(document).map_err(|_| fail())?;
        let canonical = encode_value(&decoded)?;
        if document.as_bytes() != canonical
            && document.as_bytes().strip_suffix(b"\n") != Some(canonical.as_slice())
        {
            return Err(fail());
        }
        if decoded.as_object().is_none_or(|object| {
            object
                .keys()
                .map(String::as_str)
                .ne(["artifacts", "commands", "schema"])
        }) || decoded["schema"] != "prismpm/sdk-inventory/1"
            || decoded["artifacts"] != row["inventory"]
        {
            return Err(fail());
        }
        let commands = decoded["commands"].as_array().ok_or_else(fail)?;
        let mut previous_command = "";
        if commands.is_empty() {
            return Err(fail());
        }
        for command in commands {
            let name = command["command"].as_str().ok_or_else(fail)?;
            let hash = command["sha256"].as_str().ok_or_else(fail)?;
            if command.as_object().is_none_or(|object| {
                object
                    .keys()
                    .map(String::as_str)
                    .ne(["command", "executable", "sha256"])
            }) || name <= previous_command
                || command["executable"]
                    .as_str()
                    .is_none_or(|path| !path.starts_with('/'))
                || hash.len() != 64
                || !hash
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(fail());
            }
            previous_command = name;
        }
        let identities = inventory
            .iter()
            .map(|item| {
                Some((
                    item["id"].as_str()?.to_owned(),
                    item["kind"].as_str()?.to_owned(),
                    item["version"].as_str()?.to_owned(),
                ))
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(fail)?;
        if identities.iter().any(|row| row.0 == "sdk-manifest")
            || identities.windows(2).any(|pair| pair[0].0 >= pair[1].0)
            || expected_identities
                .as_ref()
                .is_some_and(|expected| expected != &identities)
        {
            return Err(fail());
        }
        expected_identities = Some(identities);
    }
    Ok(())
}

/// Compare the actual running SDK inventory with its exact native lock entry.
/// The caller supplies the detected platform, never a lock-selected fallback.
pub fn validate_running_inventory(
    lock: &CanonicalDocument,
    platform: &str,
    inventory_bytes: &[u8],
) -> Result<(), PrismError> {
    let inventory: serde_json::Value = serde_json::from_slice(inventory_bytes)
        .map_err(|_| PrismError::new("PP5401", "SDK inventory is malformed"))?;
    if inventory["schema"] != "prismpm/sdk-inventory/1" || !inventory["artifacts"].is_array() {
        return Err(PrismError::new("PP5401", "SDK inventory is absent"));
    }
    let value = lock.value();
    if lock.schema() == "prismpm/sdk-lock/1" {
        let rows = value["inventory"]
            .as_array()
            .expect("validated SDK lock inventory");
        let artifacts = rows
            .iter()
            .filter(|row| row["id"] != "sdk-manifest")
            .cloned()
            .collect::<Vec<_>>();
        let digest = value["sdk_image"]
            .as_str()
            .and_then(|image| image.rsplit_once('@'))
            .map(|(_, digest)| digest)
            .expect("validated SDK reference");
        if serde_json::Value::Array(artifacts) != inventory["artifacts"]
            || !rows.iter().any(|row| {
                row["id"] == "sdk-manifest" && row["kind"] == "image" && row["digest"] == digest
            })
        {
            return Err(PrismError::new(
                "PP5401",
                "legacy SDK inventory disagrees with the running SDK",
            ));
        }
    } else {
        let row = value["platforms"]
            .as_array()
            .expect("validated SDK platforms")
            .iter()
            .find(|row| row["platform"] == platform)
            .ok_or_else(|| PrismError::new("PP5401", "running SDK platform is not locked"))?;
        if row["inventory"] != inventory["artifacts"]
            || row["inventory_digest"] != format!("sha256:{:x}", Sha256::digest(inventory_bytes))
        {
            return Err(PrismError::new(
                "PP5401",
                "SDK platform inventory disagrees with the running SDK",
            ));
        }
    }
    Ok(())
}

/// Load a lock for present execution, distinct from parsing a historical OCI
/// artifact. Both lock versions must select the actual native SDK inventory.
/// Only the repository's source bootstrap (no installed/configured SDK) lacks
/// that inventory; its independently pinned devcontainer gates remain required.
fn lock_is_present(root: &Path) -> Result<bool, PrismError> {
    match std::fs::symlink_metadata(root.join("prismpm.lock")) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => Err(PrismError::new(
            "PP5401",
            "prismpm.lock must be a regular file, not a symlink or other node",
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(PrismError::new(
            "PP5401",
            format!("cannot inspect prismpm.lock: {error}"),
        )),
    }
}

pub(crate) fn check_existing_lock(root: &Path) -> Result<(), PrismError> {
    if lock_is_present(root)? {
        execution_lock(root)?;
    }
    Ok(())
}

pub(crate) fn execution_lock(root: &Path) -> Result<CanonicalDocument, PrismError> {
    if !lock_is_present(root)? {
        return Err(PrismError::new("PP5401", "prismpm.lock is absent"));
    }
    let bytes = std::fs::read(root.join("prismpm.lock"))
        .map_err(|_| PrismError::new("PP5401", "prismpm.lock is absent"))?;
    let lock = parse_lock(&bytes)?;
    if let Some(path) = inventory_path() {
        let architecture = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            value => value,
        };
        let inventory = std::fs::read(path)
            .map_err(|_| PrismError::new("PP5401", "running SDK inventory is absent"))?;
        validate_running_inventory(
            &lock,
            &format!("{}/{architecture}", std::env::consts::OS),
            &inventory,
        )?;
    }
    Ok(lock)
}

/// Verify and inspect the committed SDK lock without changing it.
pub fn inspect_lock(root: &Path) -> Result<serde_json::Value, PrismError> {
    let lock = execution_lock(root)?;
    Ok(lock.value().clone())
}

/// Produce an RFC 6902-style review object for an explicit SDK-lock update.
/// Project files are read-only; applying the patch remains a normal reviewed
/// source change. Platform updates pull into the Docker image cache and use
/// temporary, never-started containers to capture immutable target inventories.
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
    if current["schema"] == "prismpm/sdk-lock/2" {
        let proposed = capture_platform_update(sdk_image, standards_lock)?;
        return platform_update_proposal(&current, proposed.value());
    }
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

fn capture_platform_update(
    sdk_image: &str,
    standards_lock: &str,
) -> Result<CanonicalDocument, PrismError> {
    let directory = tempfile::tempdir()
        .map_err(|error| PrismError::new("PP5401", format!("SDK update staging: {error}")))?;
    let helper = directory.path().join("platform-lock.mjs");
    std::fs::write(&helper, include_bytes!("../sdk/platform-lock.mjs"))
        .map_err(|error| PrismError::new("PP5401", format!("SDK update helper: {error}")))?;
    let node = executable("node")?;
    let docker = executable("docker")?;
    let mut environment = BTreeMap::new();
    for name in [
        "DOCKER_CONFIG",
        "DOCKER_HOST",
        "DOCKER_CONTEXT",
        "DOCKER_TLS_VERIFY",
        "DOCKER_CERT_PATH",
    ] {
        if let Some(value) = std::env::var_os(name) {
            environment.insert(name.to_owned(), value.to_string_lossy().into_owned());
        }
    }
    let result = crate::verification::run_process_limited(
        "sdk-platform-update",
        &node,
        &[
            helper.to_string_lossy().into_owned(),
            "capture".to_owned(),
            sdk_image.to_owned(),
            standards_lock.to_owned(),
            docker.to_string_lossy().into_owned(),
        ],
        directory.path(),
        &environment,
        &[],
        "PP5401",
        "600s",
        SDK_CAPTURE_MAX_BYTES,
    )?;
    let proposed = parse_lock(result.stdout.as_bytes())?;
    if proposed.schema() != "prismpm/sdk-lock/2"
        || proposed.value()["sdk_image"] != sdk_image
        || proposed.value()["standards_lock"] != standards_lock
    {
        return Err(PrismError::new(
            "PP5401",
            "captured SDK lock differs from the requested immutable update",
        ));
    }
    Ok(proposed)
}

fn platform_update_proposal(
    current: &serde_json::Value,
    proposed: &serde_json::Value,
) -> Result<serde_json::Value, PrismError> {
    CanonicalDocument::from_value("prismpm/sdk-lock/2", current.clone())?;
    CanonicalDocument::from_value("prismpm/sdk-lock/2", proposed.clone())?;
    let changes = ["platforms", "sdk_image", "sdk_index", "standards_lock"]
        .into_iter()
        .filter(|field| current[*field] != proposed[*field])
        .map(|field| json!({"from":current[field], "op":"replace", "path":format!("/{field}"), "to":proposed[field]}))
        .collect::<Vec<_>>();
    let proposal = CanonicalDocument::from_value(
        "prismpm/sdk-lock-update/2",
        json!({
            "changes":changes, "compatibility_review":"required", "generated_output_diff":"required",
            "proposed_lock":proposed, "schema":"prismpm/sdk-lock-update/2", "security_review":"required"
        }),
    )?;
    Ok(proposal.value().clone())
}

// Isolate environment selection in a subprocess: tests must not mutate the
// process-wide SDK identity while other parallel tests are running. In an SDK,
// the real fixed inventory wins; source-devcontainer tests use explicit,
// clearly synthetic inventory bytes. Neither path bypasses runtime validation.
#[cfg(test)]
pub(crate) fn execution_binding_regression(
    test_name: &str,
    invoke: impl Fn(&Path) -> Result<(), PrismError>,
) {
    if let Some(root) = std::env::var_os("PRISMPM_EXECUTION_BINDING_TEST_ROOT") {
        let result = invoke(Path::new(&root));
        if std::env::var("PRISMPM_EXECUTION_BINDING_TEST_BAD").unwrap() == "true" {
            assert_eq!(result.unwrap_err().code, "PP5401");
        } else if let Err(error) = result {
            assert_ne!(
                error.code, "PP5401",
                "matching native inventory was rejected"
            );
        }
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let actual = inventory_path().map(|path| std::fs::read(path).unwrap()).unwrap_or_else(|| {
        serde_json::to_vec(&json!({"schema":"prismpm/sdk-inventory/1", "commands":[{"command":"test","executable":"/synthetic/test","sha256":"c".repeat(64)}],
            "artifacts":[{"id":"synthetic-native-tool", "kind":"binary", "version":"test",
                "digest":format!("sha256:{}", "c".repeat(64))}]})).unwrap()
    });
    let inventory: serde_json::Value = serde_json::from_slice(&actual).unwrap();
    let inventory_path = root.path().join("inventory.json");
    std::fs::write(&inventory_path, &actual).unwrap();
    let standards = b"{\"authorities\":[]}";
    std::fs::write(root.path().join("standards.lock"), standards).unwrap();
    let standards_digest = format!("sha256:{:x}", Sha256::digest(standards));
    for schema in ["prismpm/sdk-lock/1", "prismpm/sdk-lock/2"] {
        let mut lock = if schema.ends_with("/1") {
            let image_digest = format!("sha256:{}", "a".repeat(64));
            let mut rows = inventory["artifacts"].as_array().unwrap().clone();
            rows.push(json!({"id":"sdk-manifest", "kind":"image", "version":"0.3.0", "digest":image_digest}));
            rows.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
            json!({"schema":schema,"sdk_version":"0.3.0", "sdk_image":format!("example.invalid/test-sdk@{image_digest}"),
                "standards_lock":standards_digest, "inventory":rows})
        } else {
            let mut manifests = Vec::new();
            let mut platforms = Vec::new();
            for (architecture, digit) in [("amd64", "a"), ("arm64", "b")] {
                let manifest = format!("sha256:{}", digit.repeat(64));
                manifests.push(json!({"digest":manifest,"mediaType":"application/vnd.oci.image.manifest.v1+json",
                    "size":100,"platform":{"os":"linux","architecture":architecture}}));
                platforms.push(json!({"platform":format!("linux/{architecture}"),"manifest_digest":manifest,
                    "inventory_digest":format!("sha256:{:x}",Sha256::digest(&actual)),"inventory_document":String::from_utf8(actual.clone()).unwrap(),"inventory":inventory["artifacts"]}));
            }
            let index = serde_json::to_string(&json!({"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":manifests})).unwrap();
            json!({"schema":schema,"sdk_version":"0.3.0", "sdk_image":format!("example.invalid/test-sdk@sha256:{:x}",Sha256::digest(index.as_bytes())),
                "sdk_index":index,"standards_lock":standards_digest,"platforms":platforms})
        };
        for bad in [false, true] {
            if bad {
                let replacement = json!(format!("sha256:{}", "f".repeat(64)));
                if schema.ends_with("/1") {
                    lock["inventory"]
                        .as_array_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|row| row["id"] != "sdk-manifest")
                        .unwrap()["digest"] = replacement;
                } else {
                    for row in lock["platforms"].as_array_mut().unwrap() {
                        row["inventory"][0]["digest"] = replacement.clone();
                        let mut document: serde_json::Value =
                            serde_json::from_str(row["inventory_document"].as_str().unwrap())
                                .unwrap();
                        document["artifacts"] = row["inventory"].clone();
                        let bytes = serde_json::to_vec(&document).unwrap();
                        row["inventory_digest"] =
                            json!(format!("sha256:{:x}", Sha256::digest(&bytes)));
                        row["inventory_document"] = json!(String::from_utf8(bytes).unwrap());
                    }
                }
            }
            let document = CanonicalDocument::from_value(schema, lock.clone()).unwrap();
            std::fs::write(root.path().join("prismpm.lock"), document.bytes()).unwrap();
            let result = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", test_name, "--nocapture"])
                .env("PRISMPM_EXECUTION_BINDING_TEST_ROOT", root.path())
                .env("PRISMPM_EXECUTION_BINDING_TEST_BAD", bad.to_string())
                .env("PRISMPM_SDK_INVENTORY", &inventory_path)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{schema}, bad={bad}: {}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};

    #[cfg(unix)]
    #[test]
    fn execution_boundary_rejects_dangling_external_and_nonfile_locks() {
        use std::os::unix::fs::symlink;
        let project = tempfile::tempdir().unwrap();
        let external = tempfile::tempdir().unwrap();
        let lock = project.path().join("prismpm.lock");
        crate::Controller::load(project.path()).unwrap();
        for target in [
            external.path().join("missing"),
            external.path().join("outside"),
        ] {
            if target.file_name().unwrap() == "outside" {
                std::fs::write(&target, b"{}").unwrap();
            }
            symlink(target, &lock).unwrap();
            let error = crate::Controller::load(project.path()).unwrap_err();
            assert_eq!(error.code, "PP5401");
            assert!(error.message.contains("regular file"));
            let error = super::inspect_lock(project.path()).unwrap_err();
            assert_eq!(error.code, "PP5401");
            assert!(error.message.contains("regular file"));
            std::fs::remove_file(&lock).unwrap();
        }
        std::fs::create_dir(&lock).unwrap();
        assert_eq!(
            crate::Controller::load(project.path()).unwrap_err().code,
            "PP5401"
        );
        assert_eq!(
            super::inspect_lock(project.path()).unwrap_err().code,
            "PP5401"
        );
        // ENOTDIR is not an absent lock: only true NotFound permits bootstrap.
        assert_eq!(
            super::check_existing_lock(&external.path().join("outside"))
                .unwrap_err()
                .code,
            "PP5401"
        );
    }

    #[test]
    fn capture_output_bound_covers_both_escaped_documents_and_repeated_rows() {
        // Quotes/backslashes attain JSON string's worst doubling factor.
        // Artifact rows are contained in each bounded original document;
        // budget them once more, plus the escaped index and 1 MiB envelope.
        let document = "\\".repeat(super::SDK_INVENTORY_MAX_BYTES);
        let escaped = serde_json::to_string(&document).unwrap().len();
        let index = serde_json::to_string(&"\\".repeat(super::SDK_INDEX_MAX_BYTES))
            .unwrap()
            .len();
        let maximum = 2 * (escaped + document.len()) + index + 1024 * 1024;
        assert!(maximum < super::SDK_CAPTURE_MAX_BYTES);
        // A proposal repeats at most the old and new lock fields plus one
        // complete proposed lock. Three bounded lock budgets cover that shape.
        assert!(3 * maximum < 192 * 1024 * 1024);
    }

    #[test]
    fn execution_boundary_binds_both_lock_versions() {
        super::execution_binding_regression(
            "sdk::tests::execution_boundary_binds_both_lock_versions",
            |root| super::inspect_lock(root).map(|_| ()),
        );
    }

    #[test]
    fn installed_inventory_cannot_be_disabled_or_overridden_by_environment() {
        let root = tempfile::tempdir().unwrap();
        let installed = root.path().join("sdk");
        let inventory = installed.join("share/inventory.json");
        let override_path = root.path().join("different-inventory.json");
        assert_eq!(
            super::select_inventory_path(&inventory, &installed, None),
            None
        );
        assert_eq!(
            super::select_inventory_path(&inventory, &installed, Some(override_path.clone())),
            Some(override_path.clone())
        );
        std::fs::create_dir(&installed).unwrap();
        for configured in [None, Some(override_path)] {
            assert_eq!(
                super::select_inventory_path(&inventory, &installed, configured),
                Some(inventory.clone())
            );
            assert!(
                std::fs::read(&inventory).is_err(),
                "missing installed inventory must fail closed"
            );
        }
    }

    // Synthetic schema/proposal fixtures, not captured SDK provenance.
    fn platform_fixture(generation: &str) -> Value {
        let digest = |value: &str| format!("sha256:{:x}", Sha256::digest(value.as_bytes()));
        let mut manifests = Vec::new();
        let mut platforms = Vec::new();
        for architecture in ["amd64", "arm64"] {
            let manifest_digest = digest(&format!("{generation}/{architecture}/manifest"));
            let inventory = json!([{"id":"prismpm","kind":"binary","version":"0.3.0",
                "digest":digest(&format!("{generation}/{architecture}/binary"))}]);
            let document = serde_json::to_string(&json!({"schema":"prismpm/sdk-inventory/1", "artifacts":inventory,
                "commands":[{"command":"prismpm","executable":"/usr/local/bin/prismpm","sha256":"a".repeat(64)}]})).unwrap();
            manifests.push(json!({"mediaType":"application/vnd.oci.image.manifest.v1+json",
                "digest":manifest_digest,"size":100,"platform":{"os":"linux","architecture":architecture}}));
            platforms.push(
                json!({"platform":format!("linux/{architecture}"),"manifest_digest":manifest_digest,
                "inventory_digest":digest(&document),"inventory_document":document,"inventory":inventory}),
            );
        }
        let index = serde_json::to_string(&json!({"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":manifests})).unwrap();
        json!({"schema":"prismpm/sdk-lock/2","sdk_version":"0.3.0",
            "sdk_image":format!("example.invalid/test-sdk@{}",digest(&index)),"sdk_index":index,
            "standards_lock":digest(generation),"platforms":platforms})
    }

    #[test]
    fn platform_update_proposal_describes_every_changed_field_without_adopting_it() {
        let current = platform_fixture("old");
        let proposed = platform_fixture("new");
        let before = current.clone();
        let proposal = super::platform_update_proposal(&current, &proposed).unwrap();
        assert_eq!(proposal["schema"], "prismpm/sdk-lock-update/2");
        assert_eq!(proposal["proposed_lock"], proposed);
        let mut applied = current.clone();
        for change in proposal["changes"].as_array().unwrap() {
            let field = change["path"].as_str().unwrap().strip_prefix('/').unwrap();
            assert_eq!(change["from"], applied[field]);
            applied[field] = change["to"].clone();
        }
        assert_eq!(proposal["changes"].as_array().unwrap().len(), 4);
        assert_eq!(applied, proposed);
        assert_eq!(current, before);
        for review in [
            "compatibility_review",
            "generated_output_diff",
            "security_review",
        ] {
            assert_eq!(proposal[review], "required");
        }
        let unchanged = super::platform_update_proposal(&current, &current).unwrap();
        assert_eq!(unchanged["changes"], json!([]));
    }

    #[test]
    fn platform_update_rejects_partial_platforms_and_inconsistent_or_unreviewed_changes() {
        let current = platform_fixture("old");
        let proposed = platform_fixture("new");
        let mut missing = proposed.clone();
        missing["platforms"].as_array_mut().unwrap().pop();
        assert!(super::platform_update_proposal(&current, &missing).is_err());
        let valid = super::platform_update_proposal(&current, &proposed).unwrap();
        for mutation in 0..6 {
            let mut invalid = valid.clone();
            match mutation {
                0 => invalid["changes"][0]["to"] = current["platforms"].clone(),
                1 => invalid["changes"].as_array_mut().unwrap().reverse(),
                2 => invalid["security_review"] = json!("passed"),
                3 => invalid["changes"][0]["from"] = invalid["changes"][0]["to"].clone(),
                4 => invalid["changes"][0]["path"] = json!("/inventory"),
                _ => invalid["proposed_lock"]["sdk_index"] = json!("tampered index"),
            }
            assert!(
                super::CanonicalDocument::from_value("prismpm/sdk-lock-update/2", invalid).is_err(),
                "mutation {mutation}"
            );
        }
    }

    #[test]
    fn legacy_update_remains_a_read_only_v1_proposal() {
        let root = tempfile::tempdir().unwrap();
        let current = json!({"schema":"prismpm/sdk-lock/1","sdk_version":"0.3.0",
            "sdk_image":format!("example.invalid/test-sdk@sha256:{}","a".repeat(64)),
            "standards_lock":format!("sha256:{}","b".repeat(64)),
            "inventory":[{"id":"prismpm","kind":"binary","version":"0.3.0","digest":format!("sha256:{}","c".repeat(64))}]});
        let bytes = serde_json::to_vec(&current).unwrap();
        std::fs::write(root.path().join("prismpm.lock"), &bytes).unwrap();
        let result = super::propose_lock_update(
            root.path(),
            &format!("example.invalid/test-sdk@sha256:{}", "d".repeat(64)),
            &format!("sha256:{}", "e".repeat(64)),
        )
        .unwrap();
        assert_eq!(result["schema"], "prismpm/sdk-lock-update/1");
        assert_eq!(result["changes"].as_array().unwrap().len(), 2);
        super::CanonicalDocument::from_value("prismpm/sdk-lock-update/1", result).unwrap();
        assert_eq!(
            std::fs::read(root.path().join("prismpm.lock")).unwrap(),
            bytes
        );
        assert!(super::propose_lock_update(
            root.path(),
            "example.invalid/test-sdk:mutable",
            &format!("sha256:{}", "e".repeat(64))
        )
        .is_err());
        assert!(super::propose_lock_update(
            root.path(),
            &format!("example.invalid/test-sdk@sha256:{}", "d".repeat(64)),
            "malformed"
        )
        .is_err());
    }

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
