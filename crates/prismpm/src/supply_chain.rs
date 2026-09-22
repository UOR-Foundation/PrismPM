//! Release-graph SBOM, provenance-policy, vulnerability, and secret evidence.

use crate::error::PrismError;
use crate::holo::canonical::encode_value;
use crate::oci;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use x509_parser::prelude::parse_x509_certificate;

const SPDX: &str = "application/spdx+json;version=3.0.1";
const POLICY: &str = "application/vnd.prismpm.supply-chain.v1+json";
const OSV_DATABASE_ID: &str = "OSV-CRATES-DB-G1788555739396732";
const OSV_DATABASE_SHA256: &str =
    "03f56153d83125941b4b6990be1fb767b968dc97e73459f933464c832362882c";
const OSV_DATABASE_EXPIRES_UNIX: u64 = 1_789_171_200;
const IMAGE_PLATFORMS: [(&str, &str); 2] = [("linux/amd64", "amd64"), ("linux/arm64", "arm64")];
const INTOTO_STATEMENT_V1: &str = "https://in-toto.io/Statement/v1";
const GITHUB_ACTIONS_ISSUER: &str = "https://token.actions.githubusercontent.com";
const SIGSTORE_TRUSTED_ROOT: &[u8] =
    include_bytes!("../standards/trust/sigstore-trusted-root-cosign-3.1.3.json");
const SLSA_PROVENANCE_V1: &str = "https://slsa.dev/provenance/v1";
/// Canonical SLSA build type for PrismPM builds.
pub const PRISM_BUILD_TYPE: &str = "https://uor.foundation/prismpm/build/v1";
const GENERATED_ARTIFACT_LICENSE: &str = "MIT OR Apache-2.0";
const SIGSTORE_BUNDLE_V03: &str = "application/vnd.dev.sigstore.bundle.v0.3+json";
const FULCIO_ISSUER: &str = "1.3.6.1.4.1.57264.1.8";
const FULCIO_RUNNER_ENVIRONMENT: &str = "1.3.6.1.4.1.57264.1.11";
const FULCIO_DEPLOYMENT_ENVIRONMENT: &str = "1.3.6.1.4.1.57264.1.23";
const COSIGN_AMD64_SHA256: &str =
    "4629c757b7618056f8ddd7e2625ae9fdd94c0372a65049520bc7d9df9efc7f71";
const COSIGN_ARM64_SHA256: &str =
    "c5d324e091826b0d7a78eb16fef316450b4eb9aaec045611c08ba06f5e73220a";

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn creation_info() -> Value {
    json!({
        "comment":"Reproducible-build epoch; wall-clock observations are excluded from deterministic artifacts",
        "created":"1970-01-01T00:00:00Z",
        "createdBy":["urn:spdx:prismpm:tool"],
        "specVersion":"3.0.1",
        "type":"CreationInfo"
    })
}

fn verified_sha256(digest: &str) -> Value {
    json!([{
        "algorithm":"sha256",
        "hashValue":digest.trim_start_matches("sha256:"),
        "type":"Hash"
    }])
}

fn license_id(expression: &str) -> String {
    format!("urn:spdx:prismpm:license:{:x}", Sha256::digest(expression))
}

fn push_relationship(graph: &mut Vec<Value>, from: &str, relationship_type: &str, to: &[String]) {
    let identity = format!("{from}:{relationship_type}:{}", to.join(","));
    graph.push(json!({
        "creationInfo":creation_info(),
        "from":from,
        "relationshipType":relationship_type,
        "spdxId":format!("urn:spdx:prismpm:relationship:{:x}",Sha256::digest(identity)),
        "to":to,
        "type":"Relationship"
    }));
}

fn digest_hex(value: &str) -> Option<&str> {
    value.strip_prefix("sha256:").filter(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn bare_digest_hex(value: &str) -> Option<&str> {
    (value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    .then_some(value)
}

fn secret_marker(bytes: &[u8]) -> Option<&'static str> {
    let text = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    [
        ("-----begin private key", "private-key"),
        ("-----begin openssh private key", "ssh-private-key"),
        ("authorization: bearer ", "bearer-token"),
        ("\"authorization\":\"bearer ", "bearer-token"),
        ("github_pat_", "github-token"),
        ("ghp_", "github-token"),
        ("client_secret=", "client-secret"),
        ("password=", "password"),
        ("aws_secret_access_key=", "cloud-credential"),
    ]
    .into_iter()
    .find_map(|(needle, name)| text.contains(needle).then_some(name))
}

fn sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase().replace(['-', '_'], "");
    [
        "authorization",
        "token",
        "accesstoken",
        "refreshtoken",
        "idtoken",
        "credential",
        "credentials",
        "clientsecret",
        "secret",
        "password",
        "operand",
        "operands",
        "history",
        "label",
        "labels",
        "clientlabel",
        "secretvalue",
    ]
    .contains(&key.as_str())
}

fn redact_value(value: &Value) -> Value {
    match value {
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        if sensitive_key(key) {
                            Value::String("[REDACTED]".to_owned())
                        } else {
                            redact_value(value)
                        },
                    )
                })
                .collect(),
        ),
        Value::Array(array) => Value::Array(array.iter().map(redact_value).collect()),
        _ => value.clone(),
    }
}

/// Canonical summary of build-time supply-chain evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupplyChainResult {
    /// Result contract.
    pub schema: String,
    /// Subject release digest.
    pub release_digest: String,
    /// SPDX referrer manifest digest.
    pub sbom_referrer: String,
    /// Policy referrer manifest digest.
    pub policy_referrer: String,
    /// Number of graph artifacts checked for secret material.
    pub scanned_artifacts: u64,
    /// Number of dependencies checked against the locked advisory input.
    pub scanned_dependencies: u64,
    /// Whether external CI identity and signature policy permits promotion.
    pub promotion_eligible: bool,
    /// Vulnerability fact boundary; this is never silently upgraded to current.
    pub vulnerability_scope: String,
}

#[derive(Debug, Clone)]
struct InventoryComponent {
    id: String,
    name: String,
    version: String,
    kind: String,
    digest: String,
    license: String,
    source: String,
    external_reference: String,
}

type CargoInventory = (Vec<InventoryComponent>, Vec<(String, String)>, Vec<String>);

fn source_tree_digest(root: &Path) -> Result<String, PrismError> {
    let mut rows = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry =
            entry.map_err(|error| PrismError::new("PP7801", format!("package source: {error}")))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|_| PrismError::new("PP7801", "package source path escaped"))?;
        if relative.components().any(|component| {
            matches!(
                component.as_os_str().to_str(),
                Some(".git" | ".prism" | "target")
            )
        }) {
            continue;
        }
        if entry.file_type().is_symlink() {
            return Err(PrismError::new(
                "PP7801",
                "package source contains a symlink",
            ));
        }
        if entry.file_type().is_file() {
            let bytes = std::fs::read(entry.path()).map_err(|error| {
                PrismError::new("PP7801", format!("package source checksum: {error}"))
            })?;
            rows.push((relative.to_string_lossy().replace('\\', "/"), sha(&bytes)));
        }
    }
    rows.sort();
    let value = serde_json::to_value(rows)
        .map_err(|error| PrismError::new("PP7801", format!("package source checksum: {error}")))?;
    Ok(sha(&encode_value(&value)?))
}

#[derive(Debug)]
pub(crate) struct VulnerabilityScan {
    database_edition: String,
    database_source: Value,
    result: Value,
    result_digest: String,
    package_count: u64,
    inventory: Vec<InventoryComponent>,
    dependency_relationships: Vec<(String, String)>,
    unresolved_licenses: Vec<String>,
    source_secret_scan: Vec<Value>,
    image_scans: Vec<Value>,
    image_package_count: u64,
    image_finding_count: u64,
    image_rejected_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ImageInput {
    reference: String,
    digest: String,
    kind: String,
}

fn source_secret_scan(root: &Path) -> Result<Vec<Value>, PrismError> {
    let project_bytes = std::fs::read(root.join("lexlean.toml"))
        .map_err(|_| PrismError::new("PP7801", "lexlean.toml is absent for source scan"))?;
    let project_text = std::str::from_utf8(&project_bytes)
        .map_err(|error| PrismError::new("PP7801", format!("lexlean.toml: {error}")))?;
    let project: toml::Value = toml::from_str(project_text)
        .map_err(|error| PrismError::new("PP7801", format!("lexlean.toml: {error}")))?;
    let mut paths = [
        "Cargo.lock",
        "lexlean.toml",
        "prismpm.lock",
        "standards.lock",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    for entrypoint in project
        .get("entrypoints")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        let path = entrypoint
            .as_str()
            .ok_or_else(|| PrismError::new("PP7801", "LexLean entrypoint is malformed"))?;
        let relative = Path::new(path);
        if relative.is_absolute()
            || relative.components().any(|component| {
                !matches!(
                    component,
                    std::path::Component::Normal(_) | std::path::Component::CurDir
                )
            })
        {
            return Err(PrismError::new(
                "PP7801",
                "LexLean entrypoint escapes the project",
            ));
        }
        paths.push(path.to_owned());
    }
    paths.sort();
    paths.dedup();
    let mut facts = Vec::new();
    for path in paths {
        let full = root.join(&path);
        if std::fs::symlink_metadata(&full)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(PrismError::new("PP7801", "source scan input is a symlink"));
        }
        let bytes = std::fs::read(&full)
            .map_err(|error| PrismError::new("PP7801", format!("source scan {path}: {error}")))?;
        if let Some(kind) = secret_marker(&bytes) {
            return Err(PrismError::new(
                "PP7801",
                format!("source or lock {path} contains probable {kind}"),
            ));
        }
        facts.push(json!({
            "byte_count":bytes.len(),
            "digest":sha(&bytes),
            "finding_count":0,
            "subject":path
        }));
    }
    Ok(facts)
}

fn cargo_lock_checksums(root: &Path) -> Result<BTreeMap<(String, String), String>, PrismError> {
    let bytes = std::fs::read(root.join("Cargo.lock"))
        .map_err(|_| PrismError::new("PP7801", "production scan requires Cargo.lock"))?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| PrismError::new("PP7801", format!("Cargo.lock: {error}")))?;
    let lock: toml::Value = toml::from_str(text)
        .map_err(|error| PrismError::new("PP7801", format!("Cargo.lock: {error}")))?;
    let mut checksums = BTreeMap::new();
    for package in lock
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(name) = package.get("name").and_then(toml::Value::as_str) else {
            continue;
        };
        let Some(version) = package.get("version").and_then(toml::Value::as_str) else {
            continue;
        };
        if let Some(checksum) = package.get("checksum").and_then(toml::Value::as_str) {
            checksums.insert((name.to_owned(), version.to_owned()), checksum.to_owned());
        }
    }
    Ok(checksums)
}

fn cargo_inventory(root: &Path) -> Result<CargoInventory, PrismError> {
    let checksums = cargo_lock_checksums(root)?;
    let cargo = crate::sdk::executable("cargo")?;
    let record = crate::verification::run_process_limited(
        "cargo-metadata",
        &cargo,
        &[
            "metadata".to_owned(),
            "--locked".to_owned(),
            "--offline".to_owned(),
            "--format-version".to_owned(),
            "1".to_owned(),
        ],
        root,
        &BTreeMap::new(),
        &[(root, "$PROJECT")],
        "PP7801",
        "60s",
        32 * 1024 * 1024,
    )?;
    let metadata: Value = serde_json::from_str(&record.stdout)
        .map_err(|error| PrismError::new("PP7801", format!("cargo metadata: {error}")))?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7801", "cargo metadata packages are absent"))?;
    let mut package_ids = BTreeMap::new();
    let mut inventory = Vec::new();
    let mut unresolved = Vec::new();
    for package in packages {
        let package_id = package["id"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP7801", "cargo package id is absent"))?;
        let name = package["name"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP7801", "cargo package name is absent"))?;
        let version = package["version"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP7801", "cargo package version is absent"))?;
        let source = package["source"].as_str().unwrap_or("workspace").to_owned();
        let identity = format!("{name}@{version}@{source}");
        let id = format!("urn:spdx:prismpm:cargo:{:x}", Sha256::digest(identity));
        package_ids.insert(package_id.to_owned(), id.clone());
        let license = package["license"]
            .as_str()
            .unwrap_or("NOASSERTION")
            .to_owned();
        if license == "NOASSERTION" {
            unresolved.push(format!("cargo:{name}@{version}"));
        }
        let manifest_path = package["manifest_path"].as_str().unwrap_or_default();
        let digest = checksums
            .get(&(name.to_owned(), version.to_owned()))
            .map(|checksum| format!("sha256:{checksum}"))
            .map(Ok)
            .unwrap_or_else(|| {
                let parent = Path::new(manifest_path).parent().ok_or_else(|| {
                    PrismError::new("PP7801", format!("source absent for {name}@{version}"))
                })?;
                source_tree_digest(parent)
            })?;
        inventory.push(InventoryComponent {
            id,
            name: name.to_owned(),
            version: version.to_owned(),
            kind: "cargo-package".to_owned(),
            digest,
            license,
            source,
            external_reference: format!("pkg:cargo/{name}@{version}"),
        });
    }
    let mut relationships = Vec::new();
    for node in metadata
        .pointer("/resolve/nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(from) = node["id"].as_str().and_then(|id| package_ids.get(id)) else {
            continue;
        };
        for dependency in node["dependencies"].as_array().into_iter().flatten() {
            if let Some(to) = dependency.as_str().and_then(|id| package_ids.get(id)) {
                relationships.push((from.clone(), to.clone()));
            }
        }
    }
    inventory.sort_by(|left, right| left.id.cmp(&right.id));
    relationships.sort();
    relationships.dedup();
    unresolved.sort();
    unresolved.dedup();
    Ok((inventory, relationships, unresolved))
}

fn locked_inventory(
    root: &Path,
    mut inventory: Vec<InventoryComponent>,
    unresolved: &mut Vec<String>,
) -> Result<Vec<InventoryComponent>, PrismError> {
    let sdk_lock = crate::sdk::execution_lock(root)?;
    let sdk = sdk_lock.value();
    let cargo_licenses = inventory
        .iter()
        .map(|item| {
            (
                (item.name.clone(), item.version.clone()),
                item.license.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut sdk_inventory = Vec::new();
    if sdk_lock.schema() == "prismpm/sdk-lock/1" {
        for item in sdk["inventory"].as_array().into_iter().flatten() {
            sdk_inventory.push((
                String::new(),
                sdk["sdk_image"].as_str().unwrap_or_default().to_owned(),
                item.clone(),
            ));
        }
    } else {
        let reference = sdk["sdk_image"].as_str().expect("validated SDK image");
        let (repository, digest) = reference.rsplit_once('@').expect("validated SDK reference");
        sdk_inventory.push((
            String::new(),
            reference.to_owned(),
            json!({"id":"sdk-manifest", "kind":"image", "version":"0.3.0", "digest":digest}),
        ));
        for platform in sdk["platforms"]
            .as_array()
            .expect("validated SDK platforms")
        {
            let name = platform["platform"]
                .as_str()
                .expect("validated SDK platform");
            let source = format!(
                "{repository}@{}",
                platform["manifest_digest"]
                    .as_str()
                    .expect("validated SDK manifest")
            );
            for item in platform["inventory"]
                .as_array()
                .expect("validated SDK inventory")
            {
                sdk_inventory.push((format!("/{name}"), source.clone(), item.clone()));
            }
        }
    }
    for (platform, source, item) in sdk_inventory {
        let name = item["id"].as_str().unwrap_or_default();
        let version = item["version"].as_str().unwrap_or_default();
        let digest = item["digest"].as_str().unwrap_or_default();
        if name.is_empty() || version.is_empty() || digest_hex(digest).is_none() {
            return Err(PrismError::new(
                "PP5401",
                "SDK inventory entry is malformed",
            ));
        }
        let license = cargo_licenses
            .get(&(name.to_owned(), version.to_owned()))
            .cloned()
            .unwrap_or_else(|| "NOASSERTION".to_owned());
        if license == "NOASSERTION" {
            unresolved.push(format!("sdk:{name}@{version}"));
        }
        let kind = item["kind"].as_str().unwrap_or("dependency");
        let identity = format!("{kind}:{name}@{version}{platform}");
        inventory.push(InventoryComponent {
            id: format!("urn:spdx:prismpm:sdk:{:x}", Sha256::digest(identity)),
            name: name.to_owned(),
            version: version.to_owned(),
            kind: format!("sdk-{kind}"),
            digest: digest.to_owned(),
            license,
            source,
            external_reference: format!("urn:prismpm:sdk:{name}:{version}{platform}"),
        });
    }
    let standards_bytes = std::fs::read(root.join("standards.lock"))
        .map_err(|_| PrismError::new("PP1101", "standards.lock is absent"))?;
    let standards: Value = serde_json::from_slice(&standards_bytes)
        .map_err(|error| PrismError::new("PP1101", format!("standards.lock: {error}")))?;
    for authority in standards["authorities"].as_array().into_iter().flatten() {
        let Some(digest) = authority.pointer("/source/sha256").and_then(Value::as_str) else {
            continue;
        };
        let id = authority["id"].as_str().unwrap_or_default();
        let edition = authority["edition"].as_str().unwrap_or_default();
        let license = authority
            .pointer("/source/license")
            .and_then(Value::as_str)
            .unwrap_or("NOASSERTION");
        if license == "NOASSERTION" {
            unresolved.push(format!("authority:{id}@{edition}"));
        }
        inventory.push(InventoryComponent {
            id: format!("urn:spdx:prismpm:authority:{:x}", Sha256::digest(id)),
            name: id.to_owned(),
            version: edition.to_owned(),
            kind: "validation-authority".to_owned(),
            digest: format!("sha256:{digest}"),
            license: license.to_owned(),
            source: authority
                .pointer("/source/url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            external_reference: authority["canonical_id"].as_str().unwrap_or(id).to_owned(),
        });
    }
    inventory.sort_by(|left, right| left.id.cmp(&right.id));
    inventory.dedup_by(|left, right| left.id == right.id && left.digest == right.digest);
    unresolved.sort();
    unresolved.dedup();
    Ok(inventory)
}

fn osv_databases(root: &Path, lock: &Value) -> Result<(Vec<(String, PathBuf)>, Value), PrismError> {
    let mut files = Vec::new();
    let mut facts = Vec::new();
    for database in lock["authorities"].as_array().into_iter().flatten() {
        let Some(canonical_id) = database["canonical_id"].as_str() else {
            continue;
        };
        let Some(ecosystem) = canonical_id
            .strip_prefix("gs://osv-vulnerabilities/")
            .and_then(|value| value.strip_suffix("/all.zip"))
        else {
            continue;
        };
        let id = database["id"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP7801", "OSV database ID is absent"))?;
        let edition = database["edition"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP7801", "OSV database edition is absent"))?;
        let digest = database
            .pointer("/source/sha256")
            .and_then(Value::as_str)
            .and_then(bare_digest_hex)
            .ok_or_else(|| PrismError::new("PP7801", "OSV database digest is absent"))?;
        let acquired = root.join(".prism/cache/authorities/sha256").join(digest);
        let bytes = std::fs::read(&acquired).map_err(|_| {
            PrismError::new(
                "PP7801",
                format!("verified {ecosystem} OSV database cache is absent"),
            )
        })?;
        if sha(&bytes) != format!("sha256:{digest}") {
            return Err(PrismError::new(
                "PP7801",
                format!("cached {ecosystem} OSV database digest changed"),
            ));
        }
        files.push((ecosystem.to_owned(), acquired));
        facts.push(json!({
            "digest":format!("sha256:{digest}"),
            "ecosystem":ecosystem,
            "edition":edition,
            "id":id,
            "source":database["source"]
        }));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    facts.sort_by(|left, right| left["ecosystem"].as_str().cmp(&right["ecosystem"].as_str()));
    if files.is_empty()
        || !files.iter().any(|(ecosystem, _)| ecosystem == "crates.io")
        || files.windows(2).any(|rows| rows[0].0 == rows[1].0)
    {
        return Err(PrismError::new(
            "PP7801",
            "locked OSV database set is absent, duplicated, or omits crates.io",
        ));
    }
    Ok((files, Value::Array(facts)))
}

fn stage_osv_databases(home: &Path, databases: &[(String, PathBuf)]) -> Result<(), PrismError> {
    for (ecosystem, source) in databases {
        let directory = home.join(".cache/osv-scanner").join(ecosystem);
        std::fs::create_dir_all(&directory)
            .map_err(|error| PrismError::new("PP7801", format!("OSV scan cache: {error}")))?;
        let destination = directory.join("all.zip");
        std::fs::hard_link(source, &destination)
            .or_else(|_| std::fs::copy(source, &destination).map(|_| ()))
            .map_err(|error| PrismError::new("PP7801", format!("OSV scan cache: {error}")))?;
    }
    Ok(())
}

fn required_image_inputs(root: &Path, systems: &[Value]) -> Result<Vec<ImageInput>, PrismError> {
    let sdk_bytes = std::fs::read(root.join("prismpm.lock"))
        .map_err(|_| PrismError::new("PP5401", "prismpm.lock is absent"))?;
    let sdk = crate::sdk::parse_lock(&sdk_bytes)?;
    let sdk_reference = sdk.value()["sdk_image"]
        .as_str()
        .expect("schema-validated SDK image");
    let sdk_digest = oci::validate_reference(sdk_reference, true)?;
    let mut images = BTreeSet::from([ImageInput {
        reference: sdk_reference.to_owned(),
        digest: sdk_digest.to_owned(),
        kind: "sdk-image".to_owned(),
    }]);
    for system in systems {
        for artifact in system["artifacts"].as_array().into_iter().flatten() {
            if !matches!(
                artifact["media_type"].as_str(),
                Some(
                    oci::OCI_INDEX
                        | oci::OCI_MANIFEST
                        | oci::DOCKER_MANIFEST
                        | oci::DOCKER_MANIFEST_LIST
                )
            ) {
                continue;
            }
            let path = artifact["path"]
                .as_str()
                .expect("schema-validated artifact path");
            let digest = artifact["digest"]
                .as_str()
                .expect("schema-validated artifact digest");
            let reference = format!("{path}@{digest}");
            if oci::validate_reference(&reference, true)? != digest {
                return Err(PrismError::new(
                    "PP7801",
                    "runtime image reference does not select its modeled digest",
                ));
            }
            images.insert(ImageInput {
                reference,
                digest: digest.to_owned(),
                kind: artifact["role"]
                    .as_str()
                    .unwrap_or("component-image")
                    .to_owned(),
            });
        }
    }
    Ok(images.into_iter().collect())
}

fn image_scan_path(root: &Path, image: &ImageInput, architecture: &str) -> PathBuf {
    root.join(".prism/cache/advisory-scans/sha256")
        .join(image.digest.trim_start_matches("sha256:"))
        .join(format!("linux-{architecture}.json"))
}

fn selected_platform_descriptor(index: &Value, platform: &str) -> Option<Value> {
    let (os, architecture) = platform.split_once('/')?;
    index["manifests"]
        .as_array()?
        .iter()
        .find(|descriptor| {
            descriptor.pointer("/platform/os").and_then(Value::as_str) == Some(os)
                && descriptor
                    .pointer("/platform/architecture")
                    .and_then(Value::as_str)
                    == Some(architecture)
                && descriptor
                    .pointer("/platform/variant")
                    .and_then(Value::as_str)
                    .is_none_or(|variant| variant.is_empty())
        })
        .cloned()
}

fn rating_is_rejected(rating: &str) -> bool {
    matches!(
        rating
            .rsplit_once(':')
            .map_or(rating, |(_, value)| value)
            .to_ascii_lowercase()
            .as_str(),
        "high" | "critical"
    )
}

fn finding_rows(result: &Value) -> (Vec<Value>, u64, u64) {
    let mut findings = Vec::new();
    let mut package_count = 0_u64;
    for row in result["results"].as_array().into_iter().flatten() {
        for package in row["packages"].as_array().into_iter().flatten() {
            package_count += 1;
            let name = package
                .pointer("/package/name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let version = package
                .pointer("/package/version")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let ecosystem = package
                .pointer("/package/ecosystem")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let grouped = package["groups"]
                .as_array()
                .is_some_and(|groups| !groups.is_empty());
            for vulnerability in package["vulnerabilities"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|_| !grouped)
            {
                let mut ratings = vulnerability["severity"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|severity| {
                        Some(format!(
                            "{}:{}",
                            severity["type"].as_str()?,
                            severity["score"].as_str()?
                        ))
                    })
                    .collect::<Vec<_>>();
                ratings.extend(
                    vulnerability["affected"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|affected| {
                            affected
                                .pointer("/ecosystem_specific/urgency")
                                .and_then(Value::as_str)
                                .map(|urgency| format!("vendor:{urgency}"))
                        }),
                );
                if let Some(severity) = vulnerability
                    .pointer("/database_specific/severity")
                    .and_then(Value::as_str)
                    .filter(|severity| !severity.is_empty())
                {
                    ratings.push(format!("vendor:{severity}"));
                }
                ratings.sort();
                ratings.dedup();
                let rejected = ratings.iter().any(|rating| rating_is_rejected(rating));
                findings.push(json!({
                    "ecosystem":ecosystem,
                    "id":vulnerability["id"],
                    "package":name,
                    "ratings":ratings,
                    "rejected":rejected,
                    "version":version
                }));
            }
            // OSV-Scanner's container-image result schema groups aliases and
            // exposes the computed maximum rating instead of embedding the
            // complete vulnerability records used by source scans. Treating
            // only `vulnerabilities` as findings would silently report every
            // scanned image as clean. Retain every group, including unrated
            // groups, and apply policy to the scanner's maximum rating.
            for group in package["groups"].as_array().into_iter().flatten() {
                let ids = group["ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let aliases = group["aliases"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let mut ratings = group["max_severity"]
                    .as_str()
                    .filter(|severity| !severity.is_empty())
                    .map(|severity| vec![format!("scanner:{severity}")])
                    .unwrap_or_default();
                ratings.sort();
                ratings.dedup();
                let rejected = ratings.iter().any(|rating| rating_is_rejected(rating));
                findings.push(json!({
                    "aliases":aliases,
                    "ecosystem":ecosystem,
                    "id":ids.first().cloned().unwrap_or_default(),
                    "ids":ids,
                    "package":name,
                    "ratings":ratings,
                    "rejected":rejected,
                    "version":version
                }));
            }
        }
    }
    findings.sort_by(|left, right| {
        (
            left["ecosystem"].as_str(),
            left["package"].as_str(),
            left["version"].as_str(),
            left["id"].as_str(),
        )
            .cmp(&(
                right["ecosystem"].as_str(),
                right["package"].as_str(),
                right["version"].as_str(),
                right["id"].as_str(),
            ))
    });
    findings.dedup();
    let rejected = findings
        .iter()
        .filter(|finding| finding["rejected"] == true)
        .count() as u64;
    (findings, package_count, rejected)
}

fn verify_image_scan(
    bytes: &[u8],
    image: &ImageInput,
    platform: &str,
    database_set_digest: &str,
) -> Result<Value, PrismError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| PrismError::new("PP7801", format!("image scan: {error}")))?;
    let result_bytes = encode_value(&value["result"])?;
    let findings = value["findings"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7801", "cached image scan findings are absent"))?;
    let rejected = findings
        .iter()
        .filter(|finding| finding["rejected"] == true)
        .count() as u64;
    let index_manifest = value["index_manifest"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7801", "cached image index is absent"))?;
    if sha(index_manifest.as_bytes()) != image.digest {
        return Err(PrismError::new(
            "PP7801",
            "cached image index bytes disagree with the modeled root digest",
        ));
    }
    let index: Value = serde_json::from_str(index_manifest)
        .map_err(|error| PrismError::new("PP7801", format!("cached image index: {error}")))?;
    if !matches!(
        index["mediaType"].as_str(),
        Some(oci::OCI_INDEX | oci::DOCKER_MANIFEST_LIST)
    ) {
        return Err(PrismError::new(
            "PP7801",
            "modeled multi-platform image is not an OCI or Docker manifest index",
        ));
    }
    let selected = selected_platform_descriptor(&index, platform).ok_or_else(|| {
        PrismError::new(
            "PP7801",
            "cached image index omits the required platform descriptor",
        )
    })?;
    if value["platform_descriptor"] != selected
        || selected["digest"]
            .as_str()
            .is_none_or(|digest| digest_hex(digest).is_none())
    {
        return Err(PrismError::new(
            "PP7801",
            "cached image platform descriptor disagrees with the root index",
        ));
    }
    if value["schema"] != "prismpm/image-advisory-scan/1"
        || value["subject"] != image.digest
        || value["reference"] != image.reference
        || value["subject_kind"] != image.kind
        || value["platform"] != platform
        || value["database_set_digest"] != database_set_digest
        || value["result_digest"] != sha(&result_bytes)
        || value["finding_count"].as_u64() != Some(findings.len() as u64)
        || value["rejected_count"].as_u64() != Some(rejected)
        || value["scanner_digest"]
            .as_str()
            .is_none_or(|digest| digest_hex(digest).is_none())
    {
        return Err(PrismError::new(
            "PP7801",
            "cached image advisory scan is stale, malformed, or bound to another subject",
        ));
    }
    Ok(value)
}

/// Acquire and scan every modeled component and SDK image during the explicit online phase.
pub(crate) fn fetch_image_advisories(
    root: &Path,
    systems: &[Value],
) -> Result<Vec<Value>, PrismError> {
    let lock: Value = serde_json::from_slice(
        &std::fs::read(root.join("standards.lock"))
            .map_err(|_| PrismError::new("PP1101", "standards.lock is absent"))?,
    )
    .map_err(|error| PrismError::new("PP1101", format!("standards.lock: {error}")))?;
    let (databases, database_facts) = osv_databases(root, &lock)?;
    for required in ["Debian", "Go", "Ubuntu", "npm"] {
        if !databases.iter().any(|(ecosystem, _)| ecosystem == required) {
            return Err(PrismError::new(
                "PP7801",
                format!("locked OSV database set omits required {required} coverage"),
            ));
        }
    }
    let database_set_digest = sha(&encode_value(&database_facts)?);
    let images = required_image_inputs(root, systems)?;
    let docker = crate::sdk::executable("docker")?;
    let scanner = crate::sdk::executable("osv-scanner")?;
    let scan_home = tempfile::tempdir()
        .map_err(|error| PrismError::new("PP7801", format!("image scan home: {error}")))?;
    stage_osv_databases(scan_home.path(), &databases)?;
    let mut scan_env = BTreeMap::from([
        (
            "HOME".to_owned(),
            scan_home.path().to_string_lossy().into_owned(),
        ),
        (
            "XDG_CACHE_HOME".to_owned(),
            scan_home
                .path()
                .join(".cache")
                .to_string_lossy()
                .into_owned(),
        ),
    ]);
    if let Some(value) = std::env::var_os("DOCKER_CONFIG") {
        scan_env.insert(
            "DOCKER_CONFIG".to_owned(),
            value.to_string_lossy().into_owned(),
        );
    }
    let mut records = Vec::new();
    for image in images {
        let index_record = crate::verification::run_process_limited(
            "docker-image-index",
            &docker,
            &[
                "buildx".to_owned(),
                "imagetools".to_owned(),
                "inspect".to_owned(),
                "--raw".to_owned(),
                image.reference.clone(),
            ],
            root,
            &BTreeMap::new(),
            &[(root, "$PROJECT")],
            "PP7801",
            "120s",
            16 * 1024 * 1024,
        )?;
        if sha(index_record.stdout.as_bytes()) != image.digest {
            return Err(PrismError::new(
                "PP7801",
                "remote image index bytes disagree with the modeled root digest",
            ));
        }
        let index: Value = serde_json::from_str(&index_record.stdout)
            .map_err(|error| PrismError::new("PP7801", format!("image index: {error}")))?;
        if !matches!(
            index["mediaType"].as_str(),
            Some(oci::OCI_INDEX | oci::DOCKER_MANIFEST_LIST)
        ) {
            return Err(PrismError::new(
                "PP7801",
                "required image is not a multi-platform manifest index",
            ));
        }
        let repository = image
            .reference
            .rsplit_once('@')
            .map(|(repository, _)| repository)
            .ok_or_else(|| PrismError::new("PP7801", "image reference omits its root digest"))?;
        for (platform, architecture) in IMAGE_PLATFORMS {
            let platform_descriptor =
                selected_platform_descriptor(&index, platform).ok_or_else(|| {
                    PrismError::new(
                        "PP7801",
                        format!("image index omits required {platform} descriptor"),
                    )
                })?;
            let platform_digest = platform_descriptor["digest"]
                .as_str()
                .filter(|digest| digest_hex(digest).is_some())
                .ok_or_else(|| PrismError::new("PP7801", "platform digest is malformed"))?;
            let platform_reference = format!("{repository}@{platform_digest}");
            let destination = image_scan_path(root, &image, architecture);
            if destination.is_file() {
                let bytes = std::fs::read(&destination)
                    .map_err(|error| PrismError::new("PP7801", format!("image scan: {error}")))?;
                records.push(verify_image_scan(
                    &bytes,
                    &image,
                    platform,
                    &database_set_digest,
                )?);
                continue;
            }
            std::fs::create_dir_all(destination.parent().expect("scan cache parent"))
                .map_err(|error| PrismError::new("PP7801", format!("image scan cache: {error}")))?;
            crate::verification::run_process_limited(
                "docker-pull-advisory-subject",
                &docker,
                &[
                    "image".to_owned(),
                    "pull".to_owned(),
                    "--platform".to_owned(),
                    platform.to_owned(),
                    platform_reference.clone(),
                ],
                root,
                &BTreeMap::new(),
                &[(root, "$PROJECT")],
                "PP7801",
                "900s",
                4 * 1024 * 1024,
            )?;
            let inspected = crate::verification::run_process_limited(
                "docker-inspect-advisory-subject",
                &docker,
                &[
                    "image".to_owned(),
                    "inspect".to_owned(),
                    platform_reference.clone(),
                ],
                root,
                &BTreeMap::new(),
                &[(root, "$PROJECT")],
                "PP7801",
                "60s",
                4 * 1024 * 1024,
            )?;
            let inspected: Value = serde_json::from_str(&inspected.stdout)
                .map_err(|error| PrismError::new("PP7801", format!("image inspect: {error}")))?;
            let inspected = inspected
                .as_array()
                .and_then(|rows| rows.first())
                .ok_or_else(|| {
                    PrismError::new("PP7801", "image inspect did not return one subject")
                })?;
            if inspected["Architecture"] != architecture
                || inspected["RepoDigests"].as_array().is_none_or(|digests| {
                    !digests.iter().any(|digest| digest == &platform_reference)
                })
            {
                return Err(PrismError::new(
                    "PP7801",
                    "pulled image platform or root digest disagrees with the lock",
                ));
            }
            let archive = tempfile::Builder::new()
                .suffix(".tar")
                .tempfile_in(destination.parent().expect("scan cache parent"))
                .map_err(|error| PrismError::new("PP7801", format!("image archive: {error}")))?;
            let archive_path = archive.path().to_path_buf();
            crate::verification::run_process_limited(
                "docker-save-advisory-subject",
                &docker,
                &[
                    "image".to_owned(),
                    "save".to_owned(),
                    "--output".to_owned(),
                    archive_path.to_string_lossy().into_owned(),
                    platform_reference,
                ],
                root,
                &BTreeMap::new(),
                &[(root, "$PROJECT"), (&archive_path, "$IMAGE_ARCHIVE")],
                "PP7801",
                "900s",
                4 * 1024 * 1024,
            )?;
            let scan = crate::verification::run_process_limited_allowed(
                "osv-image-scan",
                &scanner,
                &[
                    "scan".to_owned(),
                    "image".to_owned(),
                    "--archive".to_owned(),
                    archive_path.to_string_lossy().into_owned(),
                    "--offline".to_owned(),
                    "--offline-vulnerabilities".to_owned(),
                    "--format".to_owned(),
                    "json".to_owned(),
                    "--all-packages".to_owned(),
                    "--verbosity".to_owned(),
                    "error".to_owned(),
                ],
                root,
                &scan_env,
                &[
                    (root, "$PROJECT"),
                    (&archive_path, "$IMAGE_ARCHIVE"),
                    (scan_home.path(), "$SCAN_HOME"),
                ],
                "PP7801",
                "900s",
                64 * 1024 * 1024,
                &[0, 1],
            )?;
            let result: Value = serde_json::from_str(&scan.stdout)
                .map_err(|error| PrismError::new("PP7801", format!("OSV image result: {error}")))?;
            let (findings, package_count, rejected_count) = finding_rows(&result);
            if package_count == 0 {
                return Err(PrismError::new(
                    "PP7801",
                    "OSV image scan found no packages and cannot establish coverage",
                ));
            }
            let result_digest = sha(&encode_value(&result)?);
            let value = json!({
                "database_set":database_facts,
                "database_set_digest":database_set_digest,
                "finding_count":findings.len(),
                "findings":findings,
                "index_manifest":index_record.stdout.clone(),
                "package_count":package_count,
                "platform":platform,
                "platform_descriptor":platform_descriptor,
                "policy":"reject-vendor-high-or-critical; retain all rated and unrated findings",
                "reference":image.reference,
                "rejected_count":rejected_count,
                "result":result,
                "result_digest":result_digest,
                "scanner_digest":format!("sha256:{}",scan.executable_sha256),
                "schema":"prismpm/image-advisory-scan/1",
                "subject":image.digest,
                "subject_kind":image.kind
            });
            let bytes = encode_value(&value)?;
            let mut staging = tempfile::Builder::new()
                .prefix("scan-")
                .tempfile_in(destination.parent().expect("scan cache parent"))
                .map_err(|error| PrismError::new("PP7801", format!("image scan cache: {error}")))?;
            use std::io::Write as _;
            staging
                .write_all(&bytes)
                .and_then(|()| staging.as_file().sync_all())
                .map_err(|error| PrismError::new("PP7801", format!("image scan cache: {error}")))?;
            staging
                .persist(&destination)
                .map_err(|error| PrismError::new("PP7801", format!("image scan cache: {error}")))?;
            records.push(verify_image_scan(
                &bytes,
                &image,
                platform,
                &database_set_digest,
            )?);
        }
    }
    records.sort_by(|left, right| {
        (left["reference"].as_str(), left["platform"].as_str())
            .cmp(&(right["reference"].as_str(), right["platform"].as_str()))
    });
    Ok(records)
}

pub(crate) fn scan_vulnerabilities(
    root: &Path,
    systems: &[Value],
) -> Result<VulnerabilityScan, PrismError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrismError::new("PP7801", format!("system clock: {error}")))?
        .as_secs();
    if now >= OSV_DATABASE_EXPIRES_UNIX {
        return Err(PrismError::new(
            "PP7801",
            "locked OSV database is stale for the seven-day production policy",
        ));
    }
    let lock: Value = serde_json::from_slice(
        &std::fs::read(root.join("standards.lock"))
            .map_err(|_| PrismError::new("PP1101", "standards.lock is absent"))?,
    )
    .map_err(|error| PrismError::new("PP1101", format!("standards.lock: {error}")))?;
    let database = lock["authorities"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|row| row["id"] == OSV_DATABASE_ID)
        .ok_or_else(|| PrismError::new("PP7801", "locked OSV database is absent"))?;
    if database["source"]["sha256"] != OSV_DATABASE_SHA256 {
        return Err(PrismError::new(
            "PP7801",
            "locked OSV database identity changed",
        ));
    }
    let (databases, database_facts) = osv_databases(root, &lock)?;
    let database_set_digest = sha(&encode_value(&database_facts)?);
    let (cargo, dependency_relationships, mut unresolved_licenses) = cargo_inventory(root)?;
    let inventory = locked_inventory(root, cargo, &mut unresolved_licenses)?;
    let source_secret_scan = source_secret_scan(root)?;
    let package_count = inventory
        .iter()
        .filter(|item| item.kind == "cargo-package")
        .count() as u64;
    let lockfile = root.join("Cargo.lock");
    let home = tempfile::tempdir()
        .map_err(|error| PrismError::new("PP7801", format!("OSV scan home: {error}")))?;
    stage_osv_databases(home.path(), &databases)?;
    let scanner = crate::sdk::executable("osv-scanner")?;
    let environment = BTreeMap::from([
        (
            "HOME".to_owned(),
            home.path().to_string_lossy().into_owned(),
        ),
        (
            "XDG_CACHE_HOME".to_owned(),
            home.path().join(".cache").to_string_lossy().into_owned(),
        ),
    ]);
    let record = crate::verification::run_process_limited(
        "osv-scanner",
        &scanner,
        &[
            "scan".to_owned(),
            "source".to_owned(),
            "--lockfile".to_owned(),
            lockfile.to_string_lossy().into_owned(),
            "--offline".to_owned(),
            "--offline-vulnerabilities".to_owned(),
            "--format".to_owned(),
            "json".to_owned(),
            "--all-packages".to_owned(),
            "--verbosity".to_owned(),
            "error".to_owned(),
        ],
        root,
        &environment,
        &[(root, "$PROJECT"), (home.path(), "$SCAN_HOME")],
        "PP7801",
        "120s",
        16 * 1024 * 1024,
    )?;
    let mut result: Value = serde_json::from_str(&record.stdout)
        .map_err(|error| PrismError::new("PP7801", format!("OSV result: {error}")))?;
    for row in result["results"].as_array_mut().into_iter().flatten() {
        if row["source"]["path"].is_string() {
            row["source"]["path"] = Value::String("Cargo.lock".to_owned());
        }
    }
    let vulnerability_count = result["results"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|row| row["packages"].as_array().into_iter().flatten())
        .flat_map(|row| row["vulnerabilities"].as_array().into_iter().flatten())
        .count();
    if vulnerability_count != 0 {
        return Err(PrismError::new(
            "PP7801",
            format!("OSV policy rejected {vulnerability_count} known vulnerabilities"),
        ));
    }
    let result_bytes = encode_value(&result)?;
    let mut image_scans = Vec::new();
    let mut image_package_count = 0_u64;
    let mut image_finding_count = 0_u64;
    let mut image_rejected_count = 0_u64;
    for image in required_image_inputs(root, systems)? {
        for (platform, architecture) in IMAGE_PLATFORMS {
            let path = image_scan_path(root, &image, architecture);
            let bytes = std::fs::read(&path).map_err(|_| {
                PrismError::new(
                    "PP7801",
                    format!(
                        "locked advisory scan is absent for {} {platform}",
                        image.reference
                    ),
                )
            })?;
            let scan = verify_image_scan(&bytes, &image, platform, &database_set_digest)?;
            let rejected = scan["rejected_count"].as_u64().unwrap_or(u64::MAX);
            image_package_count = image_package_count
                .saturating_add(scan["package_count"].as_u64().unwrap_or_default());
            image_finding_count = image_finding_count
                .saturating_add(scan["finding_count"].as_u64().unwrap_or_default());
            image_rejected_count = image_rejected_count.saturating_add(rejected);
            image_scans.push(json!({
                "database_set_digest":scan["database_set_digest"],
                "finding_count":scan["finding_count"],
                "findings":scan["findings"],
                "package_count":scan["package_count"],
                "platform":scan["platform"],
                "policy":scan["policy"],
                "reference":scan["reference"],
                "rejected_count":scan["rejected_count"],
                "result_digest":scan["result_digest"],
                "scanner_digest":scan["scanner_digest"],
                "subject":scan["subject"],
                "subject_kind":scan["subject_kind"]
            }));
        }
    }
    if image_rejected_count != 0 {
        return Err(PrismError::new(
            "PP7801",
            format!(
                "image advisory policy rejected {image_rejected_count} high or critical findings"
            ),
        ));
    }
    image_scans.sort_by(|left, right| {
        (left["reference"].as_str(), left["platform"].as_str())
            .cmp(&(right["reference"].as_str(), right["platform"].as_str()))
    });
    Ok(VulnerabilityScan {
        database_edition: format!(
            "{} databases through {}",
            database_facts.as_array().map_or(0, Vec::len),
            database["edition"].as_str().unwrap_or_default()
        ),
        database_source: database_facts,
        result,
        result_digest: sha(&result_bytes),
        package_count,
        inventory,
        dependency_relationships,
        unresolved_licenses,
        source_secret_scan,
        image_scans,
        image_package_count,
        image_finding_count,
        image_rejected_count,
    })
}

/// Inputs objectively known by the build platform for SLSA provenance.
#[derive(Debug, Clone)]
pub struct ProvenanceInputs {
    /// Stable artifact name placed in the in-toto subject.
    pub subject_name: String,
    /// Exact `sha256:` OCI subject digest.
    pub subject_digest: String,
    /// URI identifying the builder implementation.
    pub builder_id: String,
    /// Build-platform invocation identifier.
    pub invocation_id: String,
    /// Canonical source-repository URI.
    pub source_uri: String,
    /// Exact source revision supplied to the builder.
    pub source_revision: String,
    /// Public, canonical build parameters; secrets must not appear here.
    pub external_parameters: Value,
    /// Resolved dependency URI and `sha256:` digest pairs.
    pub dependencies: Vec<(String, String)>,
}

/// Build a canonical in-toto Statement v1 with a SLSA provenance v1 predicate.
pub fn provenance_statement(inputs: &ProvenanceInputs) -> Result<Vec<u8>, PrismError> {
    let subject = digest_hex(&inputs.subject_digest)
        .ok_or_else(|| PrismError::new("PP7401", "provenance subject digest is malformed"))?;
    if inputs.subject_name.is_empty()
        || inputs.builder_id.is_empty()
        || inputs.source_uri.is_empty()
        || inputs.source_revision.is_empty()
        || !inputs.external_parameters.is_object()
        || inputs.dependencies.is_empty()
    {
        return Err(PrismError::new(
            "PP7401",
            "provenance omits subject, builder, source, parameters, or dependencies",
        ));
    }
    let mut dependencies = inputs
        .dependencies
        .iter()
        .map(|(uri, digest)| {
            let digest = digest_hex(digest).ok_or_else(|| {
                PrismError::new("PP7401", "provenance dependency digest is malformed")
            })?;
            Ok(json!({"digest":{"sha256":digest},"uri":uri}))
        })
        .collect::<Result<Vec<_>, PrismError>>()?;
    dependencies.push(json!({
        "digest":{"gitCommit":inputs.source_revision},
        "uri":inputs.source_uri
    }));
    dependencies.sort_by_key(|left| left.to_string());
    dependencies.dedup();
    encode_value(&json!({
        "_type":INTOTO_STATEMENT_V1,
        "predicate":{
            "buildDefinition":{
                "buildType":PRISM_BUILD_TYPE,
                "externalParameters":inputs.external_parameters,
                "internalParameters":{},
                "resolvedDependencies":dependencies
            },
            "runDetails":{
                "builder":{"id":inputs.builder_id},
                "metadata":{"invocationId":inputs.invocation_id}
            }
        },
        "predicateType":SLSA_PROVENANCE_V1,
        "subject":[{"digest":{"sha256":subject},"name":inputs.subject_name}]
    }))
}

/// Closed production promotion policy evaluated after external signature verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotionPolicy {
    /// Policy contract identifier.
    pub schema: String,
    /// Exact release subject name.
    pub subject_name: String,
    /// Exact release `sha256:` digest.
    pub release_digest: String,
    /// Trusted SLSA builder URI.
    pub builder_id: String,
    /// Trusted source repository URI.
    pub source_uri: String,
    /// Exact accepted source revision.
    pub source_revision: String,
    /// Exact external build parameters.
    pub external_parameters: Value,
    /// Required resolved dependency URI and `sha256:` digest pairs.
    pub required_dependencies: Vec<(String, String)>,
    /// Trusted OIDC issuer.
    pub issuer: String,
    /// Trusted certificate identity subject.
    pub identity_subject: String,
    /// Trusted repository claim.
    pub repository: String,
    /// Trusted workflow claim.
    pub workflow: String,
    /// Trusted Git ref claim.
    pub git_ref: String,
    /// Trusted deployment environment claim.
    pub environment: String,
    /// Whether the verifier must prove transparency-log inclusion.
    pub require_transparency_log: bool,
    /// Whether local/unhosted provenance must be rejected.
    pub require_hosted_build: bool,
    /// Exact digest of the canonical Sigstore trusted-root JSON.
    pub trusted_root_digest: String,
}

fn github_environment(name: &str) -> Result<(), PrismError> {
    if name.is_empty()
        || name.len() > 255
        || name.chars().any(char::is_control)
        || name.trim() != name
    {
        return Err(PrismError::new(
            "PP7401",
            "production promotion environment is empty or malformed",
        ));
    }
    Ok(())
}

fn github_identity(name: &str) -> Result<String, PrismError> {
    std::env::var(name).map_err(|_| {
        PrismError::new(
            "PP7401",
            format!("prepare-promotion requires the GitHub Actions {name} identity fact"),
        )
    })
}

fn exact_sha256(value: &str) -> bool {
    digest_hex(value).is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn existing_promotion_materialization(
    destination: &Path,
    policy_bytes: &[u8],
    trusted_root_bytes: &[u8],
) -> Result<bool, PrismError> {
    let Ok(metadata) = std::fs::symlink_metadata(destination) else {
        return Ok(false);
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(PrismError::new(
            "PP7401",
            "promotion materialization destination is not a real directory",
        ));
    }
    let mut names = std::fs::read_dir(destination)
        .map_err(|error| PrismError::new("PP7401", format!("promotion materialization: {error}")))?
        .map(|entry| {
            entry.map(|entry| entry.file_name()).map_err(|error| {
                PrismError::new("PP7401", format!("promotion materialization: {error}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    let expected_names = vec![
        std::ffi::OsString::from("policy.json"),
        std::ffi::OsString::from("trusted-root.json"),
    ];
    if names != expected_names {
        return Err(PrismError::new(
            "PP7401",
            "existing promotion materialization has unexpected content",
        ));
    }
    for (name, expected) in [
        ("policy.json", policy_bytes),
        ("trusted-root.json", trusted_root_bytes),
    ] {
        let path = destination.join(name);
        let metadata = std::fs::symlink_metadata(&path).map_err(|error| {
            PrismError::new("PP7401", format!("promotion materialization: {error}"))
        })?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || std::fs::read(&path).map_err(|error| {
                PrismError::new("PP7401", format!("promotion materialization: {error}"))
            })? != expected
        {
            return Err(PrismError::new(
                "PP7401",
                "existing promotion materialization differs from authoritative evidence",
            ));
        }
    }
    Ok(true)
}

/// Derive and atomically materialize the closed production policy from the
/// verified release provenance and ambient GitHub Actions identity. Consumers
/// do not author or vendor either trust material or release policy.
pub fn prepare_promotion(
    root: &Path,
    release_digest: &str,
    environment: &str,
) -> Result<PromotionPolicy, PrismError> {
    if !exact_sha256(release_digest) {
        return Err(PrismError::new("PP7401", "release digest is malformed"));
    }
    github_environment(environment)?;
    if github_identity("GITHUB_ACTIONS")?.as_str() != "true"
        || github_identity("RUNNER_ENVIRONMENT")?.as_str() != "github-hosted"
    {
        return Err(PrismError::new(
            "PP7401",
            "production promotion policy may be prepared only by a GitHub-hosted Actions job",
        ));
    }
    let repository = github_identity("GITHUB_REPOSITORY")?;
    let workflow = github_identity("GITHUB_WORKFLOW")?;
    let workflow_ref = github_identity("GITHUB_WORKFLOW_REF")?;
    let git_ref = github_identity("GITHUB_REF")?;
    let source_revision = github_identity("GITHUB_SHA")?;
    if repository.split('/').count() != 2
        || repository.split('/').any(str::is_empty)
        || repository.chars().any(char::is_whitespace)
        || workflow.is_empty()
        || workflow.chars().any(char::is_control)
        || !matches!(source_revision.len(), 40 | 64)
        || !source_revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || !(git_ref.starts_with("refs/heads/") || git_ref.starts_with("refs/tags/"))
    {
        return Err(PrismError::new(
            "PP7401",
            "GitHub repository, workflow, ref, or source revision identity is malformed",
        ));
    }
    let expected_workflow_prefix = format!("{repository}/.github/workflows/");
    let expected_workflow_suffix = format!("@{git_ref}");
    if !workflow_ref.starts_with(&expected_workflow_prefix)
        || !workflow_ref.ends_with(&expected_workflow_suffix)
        || workflow_ref
            [expected_workflow_prefix.len()..workflow_ref.len() - expected_workflow_suffix.len()]
            .is_empty()
    {
        return Err(PrismError::new(
            "PP7401",
            "GITHUB_WORKFLOW_REF is not the current repository workflow at GITHUB_REF",
        ));
    }

    let statement = oci::singleton_referrer_evidence(root, release_digest, oci::INTOTO)?;
    if statement["_type"] != INTOTO_STATEMENT_V1
        || statement["predicateType"] != SLSA_PROVENANCE_V1
        || statement.pointer("/predicate/buildDefinition/buildType")
            != Some(&json!(PRISM_BUILD_TYPE))
    {
        return Err(PrismError::new(
            "PP7401",
            "release provenance is not the supported in-toto/SLSA build statement",
        ));
    }
    let subjects = statement["subject"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7401", "release provenance subject is absent"))?;
    if subjects.len() != 1
        || subjects[0]
            .pointer("/digest/sha256")
            .and_then(Value::as_str)
            != digest_hex(release_digest)
    {
        return Err(PrismError::new(
            "PP7401",
            "release provenance is not bound to the one requested subject",
        ));
    }
    let subject_name = subjects[0]["name"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PrismError::new("PP7401", "release provenance subject name is absent"))?
        .to_owned();
    let builder_id = statement
        .pointer("/predicate/runDetails/builder/id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PrismError::new("PP7401", "release provenance builder is absent"))?
        .to_owned();
    let external_parameters = statement
        .pointer("/predicate/buildDefinition/externalParameters")
        .filter(|value| value.is_object())
        .ok_or_else(|| PrismError::new("PP7401", "release provenance parameters are absent"))?
        .clone();
    let dependencies = statement
        .pointer("/predicate/buildDefinition/resolvedDependencies")
        .and_then(Value::as_array)
        .filter(|values| !values.is_empty())
        .ok_or_else(|| PrismError::new("PP7401", "release provenance dependencies are absent"))?;
    let mut source = None;
    let mut required_dependencies = Vec::new();
    for dependency in dependencies {
        let uri = dependency["uri"]
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| PrismError::new("PP7401", "provenance dependency URI is absent"))?;
        let digest = dependency["digest"]
            .as_object()
            .ok_or_else(|| PrismError::new("PP7401", "provenance dependency digest is absent"))?;
        if digest.len() != 1 {
            return Err(PrismError::new(
                "PP7401",
                "provenance dependencies must have exactly one supported digest",
            ));
        }
        if let Some(revision) = digest.get("gitCommit").and_then(Value::as_str) {
            if source
                .replace((uri.to_owned(), revision.to_owned()))
                .is_some()
            {
                return Err(PrismError::new(
                    "PP7401",
                    "release provenance has more than one source dependency",
                ));
            }
        } else if let Some(hex) = digest.get("sha256").and_then(Value::as_str) {
            let value = format!("sha256:{hex}");
            if !exact_sha256(&value) {
                return Err(PrismError::new(
                    "PP7401",
                    "provenance dependency sha256 is malformed",
                ));
            }
            required_dependencies.push((uri.to_owned(), value));
        } else {
            return Err(PrismError::new(
                "PP7401",
                "release provenance uses an unsupported dependency digest",
            ));
        }
    }
    let (source_uri, provenance_revision) = source.ok_or_else(|| {
        PrismError::new(
            "PP7401",
            "release provenance has no exact source dependency",
        )
    })?;
    let expected_source_uri = format!("https://github.com/{repository}");
    if source_uri != expected_source_uri || provenance_revision != source_revision {
        return Err(PrismError::new(
            "PP7401",
            "release provenance source does not match the current GitHub repository and revision",
        ));
    }
    required_dependencies.sort();
    if required_dependencies.is_empty()
        || required_dependencies
            .windows(2)
            .any(|rows| rows[0] == rows[1])
    {
        return Err(PrismError::new(
            "PP7401",
            "release provenance dependency closure is empty or ambiguous",
        ));
    }

    let trusted_root: Value = serde_json::from_slice(SIGSTORE_TRUSTED_ROOT).map_err(|error| {
        PrismError::new("PP7401", format!("embedded Sigstore trusted root: {error}"))
    })?;
    let trusted_root_bytes = encode_value(&trusted_root)?;
    if trusted_root_bytes != SIGSTORE_TRUSTED_ROOT {
        return Err(PrismError::new(
            "PP7401",
            "embedded Sigstore trusted root is not canonical JSON",
        ));
    }
    let policy = PromotionPolicy {
        schema: "prismpm/promotion-policy/1".to_owned(),
        subject_name,
        release_digest: release_digest.to_owned(),
        builder_id,
        source_uri,
        source_revision,
        external_parameters,
        required_dependencies,
        issuer: GITHUB_ACTIONS_ISSUER.to_owned(),
        identity_subject: format!("https://github.com/{workflow_ref}"),
        repository,
        workflow,
        git_ref,
        environment: environment.to_owned(),
        require_transparency_log: true,
        require_hosted_build: true,
        trusted_root_digest: sha(&trusted_root_bytes),
    };
    validate_provenance_policy(&statement, None, &policy).or_else(|error| {
        // The policy must close over provenance before signature material exists.
        if error.message == "unsigned development evidence cannot be promoted" {
            Ok(json!({}))
        } else {
            Err(error)
        }
    })?;
    let policy_bytes = encode_value(
        &serde_json::to_value(&policy)
            .map_err(|error| PrismError::new("PP9001", format!("promotion policy: {error}")))?,
    )?;
    let promotion_root = root.join(".prism/promotion");
    if let Ok(metadata) = std::fs::symlink_metadata(&promotion_root) {
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(PrismError::new(
                "PP7401",
                "promotion materialization root is not a real directory",
            ));
        }
    } else {
        std::fs::create_dir_all(&promotion_root).map_err(|error| {
            PrismError::new("PP7401", format!("promotion materialization: {error}"))
        })?;
    }
    let destination =
        promotion_root.join(digest_hex(release_digest).expect("validated release digest"));
    if !existing_promotion_materialization(&destination, &policy_bytes, &trusted_root_bytes)? {
        let staging = tempfile::Builder::new()
            .prefix(".promotion-")
            .tempdir_in(&promotion_root)
            .map_err(|error| {
                PrismError::new("PP7401", format!("promotion materialization: {error}"))
            })?;
        std::fs::write(staging.path().join("policy.json"), &policy_bytes)
            .and_then(|()| {
                std::fs::write(
                    staging.path().join("trusted-root.json"),
                    &trusted_root_bytes,
                )
            })
            .map_err(|error| {
                PrismError::new("PP7401", format!("promotion materialization: {error}"))
            })?;
        let staged = staging.keep();
        if let Err(error) = std::fs::rename(&staged, &destination) {
            let _ = std::fs::remove_dir_all(&staged);
            if !existing_promotion_materialization(
                &destination,
                &policy_bytes,
                &trusted_root_bytes,
            )? {
                return Err(PrismError::new(
                    "PP7401",
                    format!("publish promotion materialization: {error}"),
                ));
            }
        }
    }
    Ok(policy)
}

/// Identity facts returned by an external Cosign certificate and Rekor verifier.
///
/// Constructing this value is not signature verification. Callers must populate it
/// only from successful cryptographic verification of the exact release subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedSignature {
    /// Cryptographically verified artifact subject digest.
    pub subject_digest: String,
    /// Certificate OIDC issuer.
    pub issuer: String,
    /// Certificate subject/SAN identity.
    pub identity_subject: String,
    /// Signed repository workflow claim.
    pub repository: String,
    /// Signed workflow-path claim.
    pub workflow: String,
    /// Signed source-ref claim.
    pub git_ref: String,
    /// Signed protected-environment claim.
    pub environment: String,
    /// True only after transparency inclusion and consistency verification.
    pub transparency_verified: bool,
    /// True only for the accepted hosted build execution.
    pub hosted_build: bool,
}

/// Canonical outcome from exact-subject Cosign verification and OCI attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignatureResult {
    /// Result contract identifier.
    pub schema: String,
    /// Exact signed release digest.
    pub release_digest: String,
    /// Digest of the stored Sigstore bundle document.
    pub bundle_digest: String,
    /// Same-subject OCI signature-referrer digest.
    pub signature_referrer: String,
    /// True only after Cosign and certificate policy both accepted the bundle.
    pub verified: bool,
}

/// Canonical outcome from signing one exact deployment-evidence document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSignatureResult {
    /// Result contract identifier.
    pub schema: String,
    /// Exact release to which the evidence is attached.
    pub release_digest: String,
    /// Number of exact deployment-evidence referrers covered.
    pub evidence_count: usize,
    /// Digest of the canonical deployment-evidence closure that was signed.
    pub evidence_closure_digest: String,
    /// Digest of the stored Sigstore bundle document.
    pub bundle_digest: String,
    /// OCI manifest digest of the verified evidence-signature referrer.
    pub signature_referrer: String,
    /// True only after Cosign and the closed identity policy accepted the bundle.
    pub verified: bool,
}

/// Canonical outcome from one signed, immediate promotion transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotionResult {
    /// Result contract identifier.
    pub schema: String,
    /// Exact release digest preserved by the transition.
    pub release_digest: String,
    /// Previous release status.
    pub from: String,
    /// Newly accepted release status.
    pub to: String,
    /// Same-subject OCI promotion-referrer digest.
    pub promotion_referrer: String,
    /// Digest of the signed canonical promotion statement.
    pub statement_digest: String,
    /// True only after release and transition signatures passed policy.
    pub verified: bool,
}

/// Load a canonical closed promotion policy document.
pub fn load_promotion_policy(path: &Path) -> Result<PromotionPolicy, PrismError> {
    let (value, canonical) = read_json_document(path, "promotion policy")?;
    let observed = std::fs::read(path)
        .map_err(|error| PrismError::new("PP7401", format!("promotion policy: {error}")))?;
    if observed != canonical {
        return Err(PrismError::new(
            "PP7401",
            "promotion policy must be canonical JSON",
        ));
    }
    serde_json::from_value(value)
        .map_err(|error| PrismError::new("PP7401", format!("promotion policy: {error}")))
}

fn read_json_document(path: &Path, role: &str) -> Result<(Value, Vec<u8>), PrismError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| PrismError::new("PP7401", format!("{role}: {error}")))?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > 16_777_216
    {
        return Err(PrismError::new(
            "PP7401",
            format!("{role} must be a regular JSON file no larger than 16 MiB"),
        ));
    }
    let bytes = std::fs::read(path)
        .map_err(|error| PrismError::new("PP7401", format!("{role}: {error}")))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| PrismError::new("PP7401", format!("{role}: {error}")))?;
    let canonical = encode_value(&value)?;
    Ok((value, canonical))
}

fn der_utf8(value: &[u8]) -> Option<String> {
    if value.first() != Some(&0x0c) {
        return None;
    }
    let first = *value.get(1)?;
    let (length, offset) = if first & 0x80 == 0 {
        (usize::from(first), 2)
    } else {
        let count = usize::from(first & 0x7f);
        if count == 0 || count > std::mem::size_of::<usize>() || value.len() < 2 + count {
            return None;
        }
        let mut length = 0_usize;
        for byte in &value[2..2 + count] {
            length = length.checked_mul(256)?.checked_add(usize::from(*byte))?;
        }
        (length, 2 + count)
    };
    let bytes = value.get(offset..offset.checked_add(length)?)?;
    (offset + length == value.len())
        .then(|| std::str::from_utf8(bytes).ok().map(str::to_owned))
        .flatten()
}

fn bundle_certificate_claims(bundle: &Value) -> Result<BTreeMap<String, String>, PrismError> {
    if bundle["mediaType"] != SIGSTORE_BUNDLE_V03
        || bundle
            .pointer("/verificationMaterial/tlogEntries")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
    {
        return Err(PrismError::new(
            "PP7401",
            "Cosign requires a standard v0.3 bundle with transparency-log evidence",
        ));
    }
    let encoded = bundle
        .pointer("/verificationMaterial/certificate/rawBytes")
        .and_then(Value::as_str)
        .ok_or_else(|| PrismError::new("PP7401", "Sigstore bundle certificate is absent"))?;
    let der = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|_| PrismError::new("PP7401", "Sigstore bundle certificate is malformed"))?;
    let (remaining, certificate) = parse_x509_certificate(&der)
        .map_err(|_| PrismError::new("PP7401", "Sigstore bundle X.509 certificate is malformed"))?;
    if !remaining.is_empty() {
        return Err(PrismError::new(
            "PP7401",
            "Sigstore bundle certificate has trailing bytes",
        ));
    }
    let mut claims = BTreeMap::new();
    for extension in certificate.extensions() {
        let oid = extension.oid.to_id_string();
        if [
            FULCIO_ISSUER,
            FULCIO_RUNNER_ENVIRONMENT,
            FULCIO_DEPLOYMENT_ENVIRONMENT,
        ]
        .contains(&oid.as_str())
        {
            let value = der_utf8(extension.value).ok_or_else(|| {
                PrismError::new(
                    "PP7401",
                    format!("Fulcio certificate extension {oid} is malformed"),
                )
            })?;
            if claims.insert(oid, value).is_some() {
                return Err(PrismError::new(
                    "PP7401",
                    "Fulcio certificate contains a duplicate policy extension",
                ));
            }
        }
    }
    Ok(claims)
}

fn checked_cosign() -> Result<PathBuf, PrismError> {
    let executable = crate::sdk::executable("cosign").map_err(|error| {
        PrismError::new(
            "PP7401",
            format!("Cosign 3.1.3 is unavailable: {}", error.message),
        )
    })?;
    let resolved = executable
        .canonicalize()
        .map_err(|error| PrismError::new("PP7401", format!("resolve Cosign 3.1.3: {error}")))?;
    let digest = format!(
        "{:x}",
        Sha256::digest(std::fs::read(&resolved).map_err(|error| {
            PrismError::new("PP7401", format!("read Cosign 3.1.3: {error}"))
        })?)
    );
    if !matches!(digest.as_str(), COSIGN_AMD64_SHA256 | COSIGN_ARM64_SHA256) {
        return Err(PrismError::new(
            "PP7401",
            "SDK Cosign executable is not the pinned 3.1.3 release",
        ));
    }
    Ok(executable)
}

fn verification_arguments(
    bundle: &Path,
    trusted_root: &Path,
    subject: &Path,
    policy: &PromotionPolicy,
) -> Vec<String> {
    vec![
        "verify-blob".to_owned(),
        "--bundle".to_owned(),
        bundle.display().to_string(),
        "--trusted-root".to_owned(),
        trusted_root.display().to_string(),
        "--certificate-identity".to_owned(),
        policy.identity_subject.clone(),
        "--certificate-oidc-issuer".to_owned(),
        policy.issuer.clone(),
        "--certificate-github-workflow-repository".to_owned(),
        policy.repository.clone(),
        "--certificate-github-workflow-name".to_owned(),
        policy.workflow.clone(),
        "--certificate-github-workflow-ref".to_owned(),
        policy.git_ref.clone(),
        "--certificate-github-workflow-sha".to_owned(),
        policy.source_revision.clone(),
        subject.display().to_string(),
    ]
}

#[allow(clippy::too_many_arguments)]
fn verify_blob(
    root: &Path,
    release_digest: &str,
    signed_blob_digest: &str,
    subject: &[u8],
    bundle: &Value,
    bundle_bytes: &[u8],
    trusted_root_bytes: &[u8],
    policy: &PromotionPolicy,
) -> Result<(VerifiedSignature, Value), PrismError> {
    if policy.schema != "prismpm/promotion-policy/1"
        || policy.release_digest != release_digest
        || digest_hex(release_digest).is_none()
        || sha(subject) != signed_blob_digest
        || sha(trusted_root_bytes) != policy.trusted_root_digest
        || policy.issuer.is_empty()
        || policy.identity_subject.is_empty()
        || policy.repository.is_empty()
        || policy.workflow.is_empty()
        || policy.git_ref.is_empty()
        || policy.environment.is_empty()
        || !policy.require_transparency_log
        || !policy.require_hosted_build
    {
        return Err(PrismError::new(
            "PP7401",
            "production signature policy is incomplete or bound to another subject/trusted root",
        ));
    }
    let claims = bundle_certificate_claims(bundle)?;
    if claims.get(FULCIO_ISSUER) != Some(&policy.issuer)
        || claims.get(FULCIO_DEPLOYMENT_ENVIRONMENT) != Some(&policy.environment)
        || claims.get(FULCIO_RUNNER_ENVIRONMENT).map(String::as_str) != Some("github-hosted")
    {
        return Err(PrismError::new(
            "PP7401",
            "Fulcio issuer, deployment environment, or hosted runner claim is not trusted",
        ));
    }
    let output = root.join(".prism");
    std::fs::create_dir_all(&output)
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let staging = tempfile::Builder::new()
        .prefix("cosign-verify-")
        .tempdir_in(&output)
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let subject_path = staging.path().join("subject");
    let bundle_path = staging.path().join("bundle.json");
    let trusted_root_path = staging.path().join("trusted-root.json");
    std::fs::write(&subject_path, subject)
        .and_then(|()| std::fs::write(&bundle_path, bundle_bytes))
        .and_then(|()| std::fs::write(&trusted_root_path, trusted_root_bytes))
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let executable = checked_cosign()?;
    let arguments = verification_arguments(&bundle_path, &trusted_root_path, &subject_path, policy);
    let record = crate::verification::run_process_limited(
        "cosign-3.1.3-verify-blob",
        &executable,
        &arguments,
        root,
        &BTreeMap::new(),
        &[(root, "$PROJECT"), (staging.path(), "$SIGNATURE")],
        "PP7401",
        "300",
        1_048_576,
    )?;
    let verified = VerifiedSignature {
        subject_digest: release_digest.to_owned(),
        issuer: policy.issuer.clone(),
        identity_subject: policy.identity_subject.clone(),
        repository: policy.repository.clone(),
        workflow: policy.workflow.clone(),
        git_ref: policy.git_ref.clone(),
        environment: policy.environment.clone(),
        transparency_verified: true,
        hosted_build: true,
    };
    let evidence = json!({
        "bundle":bundle,
        "bundle_digest":sha(bundle_bytes),
        "claims":verified,
        "cosign":{"argv":record.argv,"executable_sha256":record.executable_sha256,"version":"3.1.3"},
        "schema":"prismpm/cosign-verification/1",
        "signed_blob_digest":signed_blob_digest,
        "subject":release_digest,
        "transparency_log_verified":true,
        "trusted_root_digest":sha(trusted_root_bytes),
        "verified":true
    });
    Ok((verified, evidence))
}

fn sign_and_verify_blob(
    root: &Path,
    release_digest: &str,
    signed_blob_digest: &str,
    subject: &[u8],
    trusted_root_path: &Path,
    policy: &PromotionPolicy,
    role: &str,
) -> Result<(String, Value), PrismError> {
    if sha(subject) != signed_blob_digest {
        return Err(PrismError::new(
            "PP7401",
            "signed evidence bytes disagree with the requested digest",
        ));
    }
    let (_trusted_root, trusted_root_bytes) =
        read_json_document(trusted_root_path, "Sigstore trusted root")?;
    if sha(&trusted_root_bytes) != policy.trusted_root_digest {
        return Err(PrismError::new(
            "PP7401",
            "Sigstore trusted root does not match the promotion policy",
        ));
    }
    let token = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN").map_err(|_| {
        PrismError::new(
            "PP7401",
            format!("{role} signing requires GitHub Actions id-token: write credentials"),
        )
    })?;
    let token_url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").map_err(|_| {
        PrismError::new(
            "PP7401",
            format!("{role} signing requires GitHub Actions id-token: write credentials"),
        )
    })?;
    if token.is_empty() || token_url.is_empty() {
        return Err(PrismError::new(
            "PP7401",
            format!("{role} signing received empty GitHub Actions OIDC credentials"),
        ));
    }
    let output = root.join(".prism");
    std::fs::create_dir_all(&output)
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let staging = tempfile::Builder::new()
        .prefix("cosign-evidence-")
        .tempdir_in(&output)
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let subject_path = staging.path().join("subject.json");
    let bundle_path = staging.path().join("signature.sigstore.json");
    let trusted_root_staged = staging.path().join("trusted-root.json");
    std::fs::write(&subject_path, subject)
        .and_then(|()| std::fs::write(&trusted_root_staged, &trusted_root_bytes))
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let executable = checked_cosign()?;
    let signing_arguments = vec![
        "sign-blob".to_owned(),
        "--yes".to_owned(),
        "--oidc-provider".to_owned(),
        "github-actions".to_owned(),
        "--bundle".to_owned(),
        bundle_path.display().to_string(),
        "--trusted-root".to_owned(),
        trusted_root_staged.display().to_string(),
        subject_path.display().to_string(),
    ];
    let signing_env = BTreeMap::from([
        ("ACTIONS_ID_TOKEN_REQUEST_TOKEN".to_owned(), token),
        ("ACTIONS_ID_TOKEN_REQUEST_URL".to_owned(), token_url),
    ]);
    crate::verification::run_process_limited(
        &format!("cosign-3.1.3-sign-{role}"),
        &executable,
        &signing_arguments,
        root,
        &signing_env,
        &[(root, "$PROJECT"), (staging.path(), "$SIGNATURE")],
        "PP7401",
        "300",
        1_048_576,
    )?;
    let (bundle, bundle_bytes) = read_json_document(&bundle_path, "Sigstore bundle")?;
    let (claims, evidence) = verify_blob(
        root,
        release_digest,
        signed_blob_digest,
        subject,
        &bundle,
        &bundle_bytes,
        &trusted_root_bytes,
        policy,
    )?;
    let provenance = oci::singleton_referrer_evidence(root, release_digest, oci::INTOTO)?;
    validate_provenance_policy(&provenance, Some(&claims), policy)?;
    Ok((sha(&bundle_bytes), evidence))
}

fn replay_signature_document(
    root: &Path,
    release_digest: &str,
    signed_bytes: &[u8],
    document: &Value,
    trusted_root_bytes: &[u8],
    policy: &PromotionPolicy,
) -> Result<VerifiedSignature, PrismError> {
    let signed_blob_digest = sha(signed_bytes);
    if document["schema"] != "prismpm/cosign-verification/1"
        || document["subject"].as_str() != Some(release_digest)
        || document["signed_blob_digest"].as_str() != Some(signed_blob_digest.as_str())
    {
        return Err(PrismError::new(
            "PP7401",
            "stored signature is malformed or bound to different bytes",
        ));
    }
    let bundle = document["bundle"].clone();
    let bundle_bytes = encode_value(&bundle)?;
    if document["bundle_digest"].as_str() != Some(sha(&bundle_bytes).as_str())
        || document["trusted_root_digest"].as_str() != Some(sha(trusted_root_bytes).as_str())
    {
        return Err(PrismError::new(
            "PP7401",
            "stored signature bundle or trusted-root digest changed",
        ));
    }
    let (claims, _) = verify_blob(
        root,
        release_digest,
        &signed_blob_digest,
        signed_bytes,
        &bundle,
        &bundle_bytes,
        trusted_root_bytes,
        policy,
    )?;
    let expected_claims = serde_json::to_value(&claims)
        .map_err(|error| PrismError::new("PP9001", format!("signature claims: {error}")))?;
    if document["claims"] != expected_claims
        || document["verified"] != true
        || document["transparency_log_verified"] != true
    {
        return Err(PrismError::new(
            "PP7401",
            "stored signature claims disagree with cryptographic verification",
        ));
    }
    Ok(claims)
}

/// Independently replay every signature and promotion decision for one local
/// release using only the policy carried by its OCI graph and the SDK's
/// embedded trusted root. No serialized `verified` field is a trust input.
pub fn verify_release_trust(root: &Path, release_digest: &str) -> Result<Value, PrismError> {
    let store = oci::Store::open(root)?;
    verify_release_trust_in_store(root, &store, release_digest)
}

pub(crate) fn verify_release_transfer_in_store(
    root: &Path,
    store: &oci::Store,
    release_digest: &str,
) -> Result<(), PrismError> {
    let status = oci::verified_store_promotion_status(store, release_digest)?;
    let policy_count =
        oci::verified_referrer_evidence_rows(store, release_digest, oci::PRISM_PROMOTION_POLICY)?
            .len();
    let signature_count =
        oci::verified_referrer_evidence_rows(store, release_digest, oci::COSIGN_SIGNATURE)?.len();
    let promotion_count =
        oci::verified_referrer_evidence_rows(store, release_digest, oci::PRISM_PROMOTION)?.len();
    let evidence_signature_count =
        oci::verified_referrer_evidence_rows(store, release_digest, oci::PRISM_EVIDENCE_SIGNATURE)?
            .len();
    if status == "development"
        && policy_count == 0
        && signature_count == 0
        && promotion_count == 0
        && evidence_signature_count == 0
    {
        return Ok(());
    }
    verify_release_trust_in_store(root, store, release_digest).map(|_| ())
}

/// Verify that a local release graph is safe to consume in development.
///
/// A development graph is accepted only while it is complete, unsigned, and
/// carries no partial trust material. Any graph with trust attachments must
/// instead pass the same cryptographic replay required for transfer.
pub(crate) fn verify_release_for_development(
    root: &Path,
    release_digest: &str,
) -> Result<(), PrismError> {
    let store = oci::Store::open(root)?;
    verify_release_transfer_in_store(root, &store, release_digest)
}

pub(crate) fn verify_release_trust_in_store(
    root: &Path,
    store: &oci::Store,
    release_digest: &str,
) -> Result<Value, PrismError> {
    verify_release_trust_in_store_mode(root, store, release_digest, true)
}

fn verify_release_trust_for_evidence_signing(
    root: &Path,
    release_digest: &str,
) -> Result<Value, PrismError> {
    let store = oci::Store::open(root)?;
    verify_release_trust_in_store_mode(root, &store, release_digest, false)
}

fn verify_release_trust_in_store_mode(
    root: &Path,
    store: &oci::Store,
    release_digest: &str,
    require_current_accepted_evidence: bool,
) -> Result<Value, PrismError> {
    let policy_value = oci::verified_singleton_referrer_evidence(
        store,
        release_digest,
        oci::PRISM_PROMOTION_POLICY,
    )?;
    let policy_bytes = encode_value(&policy_value)?;
    let policy: PromotionPolicy = serde_json::from_value(policy_value)
        .map_err(|error| PrismError::new("PP7401", format!("promotion policy: {error}")))?;
    if policy.schema != "prismpm/promotion-policy/1"
        || policy.release_digest != release_digest
        || sha(SIGSTORE_TRUSTED_ROOT) != policy.trusted_root_digest
    {
        return Err(PrismError::new(
            "PP7401",
            "attached promotion policy is incomplete, untrusted, or bound to another release",
        ));
    }
    let provenance = oci::verified_singleton_referrer_evidence(store, release_digest, oci::INTOTO)?;
    let root_manifest = oci::verified_root_manifest(store, release_digest)?;
    let release_signature =
        oci::verified_singleton_referrer_evidence(store, release_digest, oci::COSIGN_SIGNATURE)?;
    let root_claims = replay_signature_document(
        root,
        release_digest,
        &root_manifest,
        &release_signature,
        SIGSTORE_TRUSTED_ROOT,
        &policy,
    )?;
    validate_provenance_policy(&provenance, Some(&root_claims), &policy)?;

    let promotion_rows =
        oci::verified_referrer_evidence_rows(store, release_digest, oci::PRISM_PROMOTION)?;
    for promotion in &promotion_rows {
        let statement_bytes = encode_value(&promotion["statement"])?;
        if promotion["statement"]["policy_digest"].as_str() != Some(sha(&policy_bytes).as_str())
            || promotion["statement_digest"].as_str() != Some(sha(&statement_bytes).as_str())
        {
            return Err(PrismError::new(
                "PP7401",
                "promotion statement does not bind the attached exact policy",
            ));
        }
        let claims = replay_signature_document(
            root,
            release_digest,
            &statement_bytes,
            &promotion["signature"],
            SIGSTORE_TRUSTED_ROOT,
            &policy,
        )?;
        let expected_policy_result =
            validate_provenance_policy(&provenance, Some(&claims), &policy)?;
        if promotion["statement"]["policy_result"] != expected_policy_result {
            return Err(PrismError::new(
                "PP7401",
                "promotion statement's policy result cannot be reproduced",
            ));
        }
    }

    let evidence_signatures =
        oci::verified_referrer_evidence_rows(store, release_digest, oci::PRISM_EVIDENCE_SIGNATURE)?;
    for evidence_signature in &evidence_signatures {
        let closure_bytes = encode_value(&evidence_signature["evidence_closure"])?;
        if evidence_signature["evidence_closure_digest"].as_str()
            != Some(sha(&closure_bytes).as_str())
        {
            return Err(PrismError::new(
                "PP7401",
                "deployment-evidence closure digest cannot be reproduced",
            ));
        }
        replay_signature_document(
            root,
            release_digest,
            &closure_bytes,
            &evidence_signature["signature"],
            SIGSTORE_TRUSTED_ROOT,
            &policy,
        )?;
    }

    let status = oci::verified_store_promotion_status(store, release_digest)?;
    if status == "accepted" && require_current_accepted_evidence {
        let acceptance = oci::verified_singleton_referrer_evidence(
            store,
            release_digest,
            oci::PRISM_PRODUCTION_ACCEPTANCE,
        )?;
        if acceptance["release_digest"].as_str() != Some(release_digest)
            || acceptance["status"] != "accepted"
        {
            return Err(PrismError::new(
                "PP7401",
                "accepted release lacks exact production acceptance evidence",
            ));
        }
        crate::acceptance::verify_closure(&acceptance)?;
        let current_closure = oci::verified_deployment_evidence_closure(store, release_digest)?;
        let current_closure_digest = sha(&current_closure);
        let current_matches = evidence_signatures
            .iter()
            .filter(|document| {
                document["evidence_closure_digest"].as_str()
                    == Some(current_closure_digest.as_str())
                    && encode_value(&document["evidence_closure"])
                        .is_ok_and(|bytes| bytes == current_closure)
            })
            .count();
        if current_matches != 1 {
            return Err(PrismError::new(
                "PP7401",
                "accepted release does not have exactly one replayed signature over its complete current deployment evidence",
            ));
        }
    }

    Ok(json!({
        "deployment_evidence_signatures":evidence_signatures.len(),
        "policy_digest":sha(&policy_bytes),
        "promotion_signatures":promotion_rows.len(),
        "release_digest":release_digest,
        "release_signatures":1,
        "schema":"prismpm/signature-closure-result/1",
        "status":status,
        "verified":true
    }))
}

/// Sign the exact local root manifest with GitHub Actions workload identity,
/// immediately verify the resulting Sigstore bundle, and attach only the
/// verified evidence to the release.
pub fn sign_release(
    root: &Path,
    release_digest: &str,
    trusted_root_path: &Path,
    policy: &PromotionPolicy,
) -> Result<SignatureResult, PrismError> {
    let subject = oci::root_manifest(root, release_digest)?;
    if sha(&subject) != release_digest {
        return Err(PrismError::new(
            "PP7401",
            "release root bytes disagree with the signing subject",
        ));
    }
    let (_trusted_root, trusted_root_bytes) =
        read_json_document(trusted_root_path, "Sigstore trusted root")?;
    if sha(&trusted_root_bytes) != policy.trusted_root_digest {
        return Err(PrismError::new(
            "PP7401",
            "Sigstore trusted root does not match the promotion policy",
        ));
    }
    let token = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN").map_err(|_| {
        PrismError::new(
            "PP7401",
            "release signing requires GitHub Actions id-token: write credentials",
        )
    })?;
    let token_url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").map_err(|_| {
        PrismError::new(
            "PP7401",
            "release signing requires GitHub Actions id-token: write credentials",
        )
    })?;
    if token.is_empty() || token_url.is_empty() {
        return Err(PrismError::new(
            "PP7401",
            "release signing received empty GitHub Actions OIDC credentials",
        ));
    }

    let output = root.join(".prism");
    std::fs::create_dir_all(&output)
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let staging = tempfile::Builder::new()
        .prefix("cosign-release-")
        .tempdir_in(&output)
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let subject_path = staging.path().join("release-manifest.json");
    let bundle_path = staging.path().join("release.sigstore.json");
    let trusted_root_staged = staging.path().join("trusted-root.json");
    std::fs::write(&subject_path, &subject)
        .and_then(|()| std::fs::write(&trusted_root_staged, &trusted_root_bytes))
        .map_err(|error| PrismError::new("PP7401", format!("signature staging: {error}")))?;
    let executable = checked_cosign()?;
    let signing_arguments = vec![
        "sign-blob".to_owned(),
        "--yes".to_owned(),
        "--oidc-provider".to_owned(),
        "github-actions".to_owned(),
        "--bundle".to_owned(),
        bundle_path.display().to_string(),
        "--trusted-root".to_owned(),
        trusted_root_staged.display().to_string(),
        subject_path.display().to_string(),
    ];
    let signing_env = BTreeMap::from([
        ("ACTIONS_ID_TOKEN_REQUEST_TOKEN".to_owned(), token),
        ("ACTIONS_ID_TOKEN_REQUEST_URL".to_owned(), token_url),
    ]);
    crate::verification::run_process_limited(
        "cosign-3.1.3-sign-release",
        &executable,
        &signing_arguments,
        root,
        &signing_env,
        &[(root, "$PROJECT"), (staging.path(), "$SIGNATURE")],
        "PP7401",
        "300",
        1_048_576,
    )?;
    let (bundle, bundle_bytes) = read_json_document(&bundle_path, "release Sigstore bundle")?;
    let (_, evidence) = verify_blob(
        root,
        release_digest,
        release_digest,
        &subject,
        &bundle,
        &bundle_bytes,
        &trusted_root_bytes,
        policy,
    )?;
    let policy_bytes = encode_value(
        &serde_json::to_value(policy)
            .map_err(|error| PrismError::new("PP9001", format!("promotion policy: {error}")))?,
    )?;
    oci::attach_promotion_policy(root, release_digest, &policy_bytes)?;
    let signature_referrer =
        oci::attach_verified_signature(root, release_digest, &encode_value(&evidence)?)?;
    Ok(SignatureResult {
        schema: "prismpm/signature-result/1".to_owned(),
        release_digest: release_digest.to_owned(),
        bundle_digest: sha(&bundle_bytes),
        signature_referrer,
        verified: true,
    })
}

/// Sign one exact attached deployment-evidence document with the ambient
/// GitHub Actions workload identity, independently verify it, and attach the
/// verification bundle to the unchanged product-release subject.
pub fn sign_deployment_evidence(
    root: &Path,
    release_digest: &str,
    trusted_root_path: &Path,
    policy: &PromotionPolicy,
) -> Result<EvidenceSignatureResult, PrismError> {
    let trust = verify_release_trust_for_evidence_signing(root, release_digest)?;
    if !matches!(trust["status"].as_str(), Some("candidate" | "accepted")) {
        return Err(PrismError::new(
            "PP7401",
            "deployment evidence may be signed only for a verified candidate or accepted release",
        ));
    }
    let evidence_bytes = oci::deployment_evidence_closure(root, release_digest)?;
    let evidence_closure: Value = serde_json::from_slice(&evidence_bytes).map_err(|error| {
        PrismError::new("PP7401", format!("deployment evidence closure: {error}"))
    })?;
    let evidence_count = evidence_closure["evidence"]
        .as_array()
        .map(Vec::len)
        .unwrap_or(0);
    let signed_blob_digest = sha(&evidence_bytes);
    let (bundle_digest, signature) = sign_and_verify_blob(
        root,
        release_digest,
        &signed_blob_digest,
        &evidence_bytes,
        trusted_root_path,
        policy,
        "deployment-evidence",
    )?;
    let envelope = encode_value(&json!({
        "evidence_closure":evidence_closure,
        "evidence_closure_digest":signed_blob_digest,
        "schema":"prismpm/evidence-signature/1",
        "signature":signature,
        "subject":release_digest,
        "verified":true
    }))?;
    let signature_referrer =
        oci::attach_verified_evidence_signature(root, release_digest, &envelope)?;
    Ok(EvidenceSignatureResult {
        schema: "prismpm/evidence-signature-result/1".to_owned(),
        release_digest: release_digest.to_owned(),
        evidence_count,
        evidence_closure_digest: signed_blob_digest,
        bundle_digest,
        signature_referrer,
        verified: true,
    })
}

/// Cryptographically verify a Cosign bundle for the exact release and attach it.
pub fn verify_release_signature(
    root: &Path,
    release_digest: &str,
    bundle_path: &Path,
    trusted_root_path: &Path,
    policy: &PromotionPolicy,
) -> Result<SignatureResult, PrismError> {
    let subject = oci::root_manifest(root, release_digest)?;
    let (bundle, bundle_bytes) = read_json_document(bundle_path, "Sigstore bundle")?;
    let (_trusted_root, trusted_root_bytes) =
        read_json_document(trusted_root_path, "Sigstore trusted root")?;
    let (_, evidence) = verify_blob(
        root,
        release_digest,
        release_digest,
        &subject,
        &bundle,
        &bundle_bytes,
        &trusted_root_bytes,
        policy,
    )?;
    let policy_bytes = encode_value(
        &serde_json::to_value(policy)
            .map_err(|error| PrismError::new("PP9001", format!("promotion policy: {error}")))?,
    )?;
    oci::attach_promotion_policy(root, release_digest, &policy_bytes)?;
    let evidence_bytes = encode_value(&evidence)?;
    let signature_referrer = oci::attach_verified_signature(root, release_digest, &evidence_bytes)?;
    Ok(SignatureResult {
        schema: "prismpm/signature-result/1".to_owned(),
        release_digest: release_digest.to_owned(),
        bundle_digest: sha(&bundle_bytes),
        signature_referrer,
        verified: true,
    })
}

/// Validate provenance and externally verified signing claims against promotion policy.
pub fn validate_provenance_policy(
    statement: &Value,
    signature: Option<&VerifiedSignature>,
    policy: &PromotionPolicy,
) -> Result<Value, PrismError> {
    if statement["_type"] != INTOTO_STATEMENT_V1
        || statement["predicateType"] != SLSA_PROVENANCE_V1
        || statement.pointer("/predicate/buildDefinition/buildType")
            != Some(&json!(PRISM_BUILD_TYPE))
        || !statement
            .pointer("/predicate/buildDefinition/externalParameters")
            .is_some_and(Value::is_object)
    {
        return Err(PrismError::new(
            "PP7401",
            "provenance envelope or predicate changed",
        ));
    }
    let subjects = statement["subject"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7401", "provenance subjects are absent"))?;
    if subjects.len() != 1
        || subjects[0]["name"].as_str() != Some(policy.subject_name.as_str())
        || subjects[0]
            .pointer("/digest/sha256")
            .and_then(Value::as_str)
            != digest_hex(&policy.release_digest)
    {
        return Err(PrismError::new(
            "PP7401",
            "provenance subject is wrong or ambiguous",
        ));
    }
    if statement
        .pointer("/predicate/runDetails/builder/id")
        .and_then(Value::as_str)
        != Some(policy.builder_id.as_str())
    {
        return Err(PrismError::new(
            "PP7401",
            "provenance builder is not trusted",
        ));
    }
    if statement.pointer("/predicate/buildDefinition/externalParameters")
        != Some(&policy.external_parameters)
    {
        return Err(PrismError::new(
            "PP7401",
            "provenance external parameters do not match policy",
        ));
    }
    let dependencies = statement
        .pointer("/predicate/buildDefinition/resolvedDependencies")
        .and_then(Value::as_array)
        .ok_or_else(|| PrismError::new("PP7401", "resolved dependencies are absent"))?;
    let source_matches = dependencies
        .iter()
        .filter(|dependency| {
            dependency["uri"].as_str() == Some(policy.source_uri.as_str())
                && dependency
                    .pointer("/digest/gitCommit")
                    .and_then(Value::as_str)
                    == Some(policy.source_revision.as_str())
                && dependency["digest"]
                    .as_object()
                    .is_some_and(|digest| digest.len() == 1)
        })
        .count();
    if dependencies.is_empty() || source_matches != 1 {
        return Err(PrismError::new(
            "PP7401",
            "provenance omits the exact source revision or resolved dependencies",
        ));
    }
    let mut observed_dependencies = Vec::new();
    for dependency in dependencies {
        if dependency.pointer("/digest/gitCommit").is_some() {
            continue;
        }
        let uri = dependency["uri"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP7401", "provenance dependency URI is absent"))?;
        let digest_object = dependency["digest"]
            .as_object()
            .ok_or_else(|| PrismError::new("PP7401", "provenance dependency digest is absent"))?;
        if digest_object.len() != 1 {
            return Err(PrismError::new(
                "PP7401",
                "provenance dependency digest is ambiguous",
            ));
        }
        let hex = digest_object
            .get("sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PrismError::new("PP7401", "provenance dependency digest is unsupported")
            })?;
        let digest = format!("sha256:{hex}");
        if !exact_sha256(&digest) {
            return Err(PrismError::new(
                "PP7401",
                "provenance dependency digest is malformed",
            ));
        }
        observed_dependencies.push((uri.to_owned(), digest));
    }
    observed_dependencies.sort();
    if observed_dependencies != policy.required_dependencies {
        return Err(PrismError::new(
            "PP7401",
            "provenance resolved dependencies do not exactly match policy",
        ));
    }
    let signature = signature.ok_or_else(|| {
        PrismError::new("PP7401", "unsigned development evidence cannot be promoted")
    })?;
    let identity_matches = signature.subject_digest == policy.release_digest
        && signature.issuer == policy.issuer
        && signature.identity_subject == policy.identity_subject
        && signature.repository == policy.repository
        && signature.workflow == policy.workflow
        && signature.git_ref == policy.git_ref
        && signature.environment == policy.environment
        && (!policy.require_transparency_log || signature.transparency_verified)
        && (!policy.require_hosted_build || signature.hosted_build);
    if !identity_matches {
        return Err(PrismError::new(
            "PP7401",
            "signature issuer, identity, repository, workflow, ref, environment, transparency, or build level is not trusted",
        ));
    }
    Ok(json!({
        "build_level":if signature.hosted_build {"hosted-build-platform"} else {"local-development"},
        "promotion_eligible":true,
        "schema":"prismpm/provenance-policy-result/1",
        "signature_verification":"external-cosign-verification-accepted",
        "subject":policy.release_digest
    }))
}

/// Reverify the release signature, sign one immediate transition with the
/// ambient GitHub Actions workload identity, and attach the accepted evidence.
pub fn promote_release(
    root: &Path,
    release_digest: &str,
    to: &str,
    trusted_root_path: &Path,
    policy: &PromotionPolicy,
) -> Result<PromotionResult, PrismError> {
    let from = oci::verified_promotion_status(root, release_digest)?;
    let replay = verify_release_trust(root, release_digest)?;
    if replay["status"].as_str() != Some(from.as_str()) {
        return Err(PrismError::new(
            "PP7401",
            "stored promotion status disagrees with cryptographic replay",
        ));
    }
    if !matches!(
        (from.as_str(), to),
        ("development", "candidate") | ("candidate", "accepted")
    ) {
        return Err(PrismError::new(
            "PP7401",
            "promotion must be the next development-to-candidate or candidate-to-accepted transition",
        ));
    }
    if to == "accepted" {
        let acceptance = oci::singleton_referrer_evidence(
            root,
            release_digest,
            oci::PRISM_PRODUCTION_ACCEPTANCE,
        )?;
        if acceptance["release_digest"].as_str() != Some(release_digest)
            || acceptance["status"] != "accepted"
        {
            return Err(PrismError::new(
                "PP7401",
                "accepted promotion requires exact-release production acceptance evidence",
            ));
        }
        oci::require_complete_signed_deployment_evidence(root, release_digest)?;
    }
    let release_signature =
        oci::singleton_referrer_evidence(root, release_digest, oci::COSIGN_SIGNATURE)?;
    if release_signature["schema"] != "prismpm/cosign-verification/1"
        || release_signature["subject"] != release_digest
        || release_signature["signed_blob_digest"] != release_digest
        || release_signature["verified"] != true
    {
        return Err(PrismError::new(
            "PP7401",
            "release signature evidence is absent, malformed, or bound to another subject",
        ));
    }
    let release_bundle = release_signature["bundle"].clone();
    let release_bundle_bytes = encode_value(&release_bundle)?;
    if release_signature["bundle_digest"] != sha(&release_bundle_bytes) {
        return Err(PrismError::new(
            "PP7401",
            "stored release signature bundle digest changed",
        ));
    }
    let (_trusted_root, trusted_root_bytes) =
        read_json_document(trusted_root_path, "Sigstore trusted root")?;
    let root_manifest = oci::root_manifest(root, release_digest)?;
    let (release_claims, _) = verify_blob(
        root,
        release_digest,
        release_digest,
        &root_manifest,
        &release_bundle,
        &release_bundle_bytes,
        &trusted_root_bytes,
        policy,
    )?;
    let provenance = oci::singleton_referrer_evidence(root, release_digest, oci::INTOTO)?;
    let policy_result = validate_provenance_policy(&provenance, Some(&release_claims), policy)?;
    let policy_bytes = encode_value(
        &serde_json::to_value(policy)
            .map_err(|error| PrismError::new("PP9001", format!("promotion policy: {error}")))?,
    )?;
    let statement = json!({
        "from":from,
        "policy_digest":sha(&policy_bytes),
        "policy_result":policy_result,
        "schema":"prismpm/promotion-statement/1",
        "subject":release_digest,
        "to":to
    });
    let statement_bytes = encode_value(&statement)?;
    let statement_digest = sha(&statement_bytes);

    let output = root.join(".prism");
    std::fs::create_dir_all(&output)
        .map_err(|error| PrismError::new("PP7401", format!("promotion staging: {error}")))?;
    let staging = tempfile::Builder::new()
        .prefix("cosign-promote-")
        .tempdir_in(&output)
        .map_err(|error| PrismError::new("PP7401", format!("promotion staging: {error}")))?;
    let statement_path = staging.path().join("promotion.json");
    let bundle_path = staging.path().join("promotion.sigstore.json");
    let trusted_root_staged = staging.path().join("trusted-root.json");
    std::fs::write(&statement_path, &statement_bytes)
        .and_then(|()| std::fs::write(&trusted_root_staged, &trusted_root_bytes))
        .map_err(|error| PrismError::new("PP7401", format!("promotion staging: {error}")))?;
    let token = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN").map_err(|_| {
        PrismError::new(
            "PP7401",
            "promotion signing requires GitHub Actions id-token: write credentials",
        )
    })?;
    let token_url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL").map_err(|_| {
        PrismError::new(
            "PP7401",
            "promotion signing requires GitHub Actions id-token: write credentials",
        )
    })?;
    if token.is_empty() || token_url.is_empty() {
        return Err(PrismError::new(
            "PP7401",
            "promotion signing received empty GitHub Actions OIDC credentials",
        ));
    }
    let executable = checked_cosign()?;
    let signing_arguments = vec![
        "sign-blob".to_owned(),
        "--yes".to_owned(),
        "--oidc-provider".to_owned(),
        "github-actions".to_owned(),
        "--bundle".to_owned(),
        bundle_path.display().to_string(),
        "--trusted-root".to_owned(),
        trusted_root_staged.display().to_string(),
        statement_path.display().to_string(),
    ];
    let signing_env = BTreeMap::from([
        ("ACTIONS_ID_TOKEN_REQUEST_TOKEN".to_owned(), token),
        ("ACTIONS_ID_TOKEN_REQUEST_URL".to_owned(), token_url),
    ]);
    crate::verification::run_process_limited(
        "cosign-3.1.3-sign-blob",
        &executable,
        &signing_arguments,
        root,
        &signing_env,
        &[(root, "$PROJECT"), (staging.path(), "$SIGNATURE")],
        "PP7401",
        "300",
        1_048_576,
    )?;
    let (promotion_bundle, promotion_bundle_bytes) =
        read_json_document(&bundle_path, "promotion Sigstore bundle")?;
    let (_, promotion_signature) = verify_blob(
        root,
        release_digest,
        &statement_digest,
        &statement_bytes,
        &promotion_bundle,
        &promotion_bundle_bytes,
        &trusted_root_bytes,
        policy,
    )?;
    let evidence = json!({
        "from":from,
        "schema":"prismpm/promotion/1",
        "signature":promotion_signature,
        "statement":statement,
        "statement_digest":statement_digest,
        "subject":release_digest,
        "to":to,
        "verified":true
    });
    let promotion_referrer =
        oci::attach_verified_promotion(root, release_digest, &encode_value(&evidence)?)?;
    Ok(PromotionResult {
        schema: "prismpm/promotion-result/1".to_owned(),
        release_digest: release_digest.to_owned(),
        from,
        to: to.to_owned(),
        promotion_referrer,
        statement_digest,
        verified: true,
    })
}

/// One immutable vulnerability/advisory scan result supplied by a pinned scanner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdvisoryScanFact {
    /// Exact `sha256:` digest of the scanned component, SDK, or dependency set.
    pub subject_digest: String,
    /// Closed caller-defined subject class, such as `component-image` or `sdk-image`.
    pub subject_kind: String,
    /// Locked advisory database identity.
    pub database_id: String,
    /// Exact digest of the advisory database bytes.
    pub database_digest: String,
    /// Unix time at which the external scanner ran.
    pub scanned_at_unix: u64,
    /// Unix time after which the locked database cannot satisfy this policy.
    pub database_expires_unix: u64,
    /// Count of findings rejected by policy; it must be zero for acceptance.
    pub rejected_findings: u64,
    /// Digest of the canonical, redacted scanner result.
    pub result_digest: String,
}

/// Closed advisory coverage and freshness policy.
#[derive(Debug, Clone)]
pub struct AdvisoryPolicy {
    /// Exact subject digests that must each have one scan fact.
    pub required_subjects: Vec<String>,
    /// Required advisory database identity.
    pub database_id: String,
    /// Required advisory database digest.
    pub database_digest: String,
    /// Maximum permitted age of the scan execution.
    pub max_age_seconds: u64,
}

/// Validate complete component/SDK scan coverage without upgrading offline facts.
pub fn validate_advisory_coverage(
    facts: &[AdvisoryScanFact],
    policy: &AdvisoryPolicy,
    now_unix: u64,
) -> Result<Value, PrismError> {
    let required = policy
        .required_subjects
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if required.len() != policy.required_subjects.len()
        || required.iter().any(|digest| digest_hex(digest).is_none())
    {
        return Err(PrismError::new(
            "PP7801",
            "advisory policy subjects are duplicated or malformed",
        ));
    }
    let mut observed = BTreeSet::new();
    for fact in facts {
        if digest_hex(&fact.subject_digest).is_none()
            || digest_hex(&fact.database_digest).is_none()
            || digest_hex(&fact.result_digest).is_none()
            || fact.subject_kind.is_empty()
            || fact.database_id != policy.database_id
            || fact.database_digest != policy.database_digest
            || fact.scanned_at_unix > now_unix
            || now_unix.saturating_sub(fact.scanned_at_unix) > policy.max_age_seconds
            || now_unix >= fact.database_expires_unix
            || fact.rejected_findings != 0
            || !observed.insert(fact.subject_digest.as_str())
        {
            return Err(PrismError::new(
                "PP7801",
                "advisory scan is malformed, stale, duplicated, wrong-database, or rejected",
            ));
        }
    }
    if observed != required {
        return Err(PrismError::new(
            "PP7801",
            "advisory scans do not cover every required component and SDK subject",
        ));
    }
    Ok(json!({
        "database_digest":policy.database_digest,
        "database_id":policy.database_id,
        "result":"passed",
        "scan_count":facts.len(),
        "schema":"prismpm/advisory-policy-result/1",
        "subjects":policy.required_subjects
    }))
}

fn component_element(component: &InventoryComponent) -> Value {
    json!({
        "creationInfo":creation_info(),
        "externalIdentifier":[{"externalIdentifierType":"other","identifier":component.external_reference,"type":"ExternalIdentifier"}],
        "name":component.name,
        "software_downloadLocation":component.source,
        "software_packageVersion":component.version,
        "spdxId":component.id,
        "summary":component.kind,
        "type":"software_Package",
        "verifiedUsing":verified_sha256(&component.digest)
    })
}

/// Validate the complete SPDX 3.0.1 graph closure against OCI descriptors and external artifacts.
pub fn validate_sbom_closure(
    spdx: &Value,
    layers: &[oci::Descriptor],
    external_artifacts: &[Value],
) -> Result<(), PrismError> {
    let elements = spdx["@graph"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7801", "SPDX graph is absent"))?;
    let relationships = elements
        .iter()
        .filter(|element| element["type"] == "Relationship")
        .collect::<Vec<_>>();
    let ids = elements
        .iter()
        .filter_map(|element| element["spdxId"].as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != elements.len() {
        return Err(PrismError::new(
            "PP7801",
            "SPDX element identities are duplicated",
        ));
    }
    for relationship in &relationships {
        let from = relationship["from"].as_str().unwrap_or_default();
        let targets = relationship["to"].as_array().into_iter().flatten();
        if !ids.contains(from)
            || targets.into_iter().any(|target| {
                !target.as_str().is_some_and(|id| {
                    ids.contains(id) || id == "expandedlicensing_NoAssertionLicense"
                })
            })
        {
            return Err(PrismError::new("PP7801", "SPDX relationship is dangling"));
        }
    }
    let artifact_digests = elements
        .iter()
        .filter(|element| {
            matches!(
                element["summary"].as_str(),
                Some("release-artifact" | "external-runtime-artifact")
            )
        })
        .filter_map(|element| {
            element
                .pointer("/verifiedUsing/0/hashValue")
                .and_then(Value::as_str)
        })
        .map(|digest| format!("sha256:{digest}"))
        .collect::<BTreeSet<_>>();
    let mut layer_digests = layers
        .iter()
        .map(|descriptor| descriptor.digest.clone())
        .collect::<BTreeSet<_>>();
    layer_digests.extend(
        external_artifacts
            .iter()
            .filter_map(|artifact| artifact["digest"].as_str().map(str::to_owned)),
    );
    if artifact_digests != layer_digests {
        return Err(PrismError::new(
            "PP7801",
            "SPDX release-artifact closure disagrees with OCI layer closure",
        ));
    }
    Ok(())
}

/// Generate graph inventory and fail closed on known secret-value forms.
pub(crate) fn attach_build_evidence(
    root: &Path,
    release_digest: &str,
    vulnerability: &VulnerabilityScan,
) -> Result<SupplyChainResult, PrismError> {
    let layers = oci::layers(root, release_digest)?;
    let store = oci::Store::open(root)?;
    let external_artifacts = oci::release_external_artifacts(&store, release_digest)?;
    let mut graph = vec![
        json!({
            "creationInfo":creation_info(),
            "name":"PrismPM 0.3.0",
            "spdxId":"urn:spdx:prismpm:tool",
            "type":"Tool"
        }),
        json!({
            "creationInfo":creation_info(),
            "name":"PrismPM product release",
            "software_downloadLocation":format!("urn:oci:{release_digest}"),
            "software_packageVersion":release_digest.trim_start_matches("sha256:"),
            "spdxId":"urn:spdx:prismpm:release",
            "summary":"product-release",
            "type":"software_Package",
            "verifiedUsing":verified_sha256(release_digest)
        }),
    ];
    let mut artifact_ids = Vec::new();
    let mut secret_scan = Vec::new();
    for (index, descriptor) in layers.iter().enumerate() {
        let bytes = oci::read_descriptor(root, descriptor)?;
        if let Some(kind) = secret_marker(&bytes) {
            return Err(PrismError::new(
                "PP7801",
                format!("release artifact contains probable {kind}"),
            ));
        }
        let id = format!("urn:spdx:prismpm:artifact:{index}");
        let name = descriptor
            .annotations
            .as_ref()
            .and_then(|rows| rows.get("org.opencontainers.image.title"))
            .cloned()
            .unwrap_or_else(|| descriptor.digest.clone());
        graph.push(json!({
            "creationInfo":creation_info(),
            "name":name,
            "spdxId":id,
            "summary":"release-artifact",
            "type":"software_File",
            "verifiedUsing":verified_sha256(&descriptor.digest)
        }));
        push_relationship(
            &mut graph,
            "urn:spdx:prismpm:release",
            "contains",
            std::slice::from_ref(&id),
        );
        artifact_ids.push(id);
        secret_scan.push(json!({
            "byte_count":bytes.len(),
            "digest":descriptor.digest,
            "finding_count":0,
            "media_type":descriptor.media_type,
            "subject":name
        }));
    }
    let mut external_ids = Vec::new();
    for artifact in &external_artifacts {
        let reference = artifact["reference"]
            .as_str()
            .expect("schema-validated external reference");
        let digest = artifact["digest"]
            .as_str()
            .expect("schema-validated external digest");
        let license = artifact["license_expression"]
            .as_str()
            .expect("schema-validated external license expression");
        let id = format!(
            "urn:spdx:prismpm:external-runtime:{:x}",
            Sha256::digest(reference)
        );
        graph.push(json!({
            "creationInfo":creation_info(),
            "externalIdentifier":[{"externalIdentifierType":"other","identifier":reference,"type":"ExternalIdentifier"}],
            "name":reference.split('@').next().unwrap_or(reference),
            "software_downloadLocation":reference,
            "software_packageVersion":digest.trim_start_matches("sha256:"),
            "spdxId":id,
            "summary":"external-runtime-artifact",
            "type":"software_Package",
            "verifiedUsing":verified_sha256(digest)
        }));
        push_relationship(
            &mut graph,
            "urn:spdx:prismpm:release",
            "dependsOn",
            std::slice::from_ref(&id),
        );
        external_ids.push((id, license.to_owned()));
    }
    let mut license_ids = BTreeMap::from([(
        GENERATED_ARTIFACT_LICENSE.to_owned(),
        license_id(GENERATED_ARTIFACT_LICENSE),
    )]);
    for component in &vulnerability.inventory {
        graph.push(component_element(component));
        push_relationship(
            &mut graph,
            "urn:spdx:prismpm:release",
            "dependsOn",
            std::slice::from_ref(&component.id),
        );
        if component.license != "NOASSERTION" {
            let id = license_id(&component.license);
            license_ids.insert(component.license.clone(), id);
        }
    }
    for (_, expression) in &external_ids {
        license_ids.insert(expression.clone(), license_id(expression));
    }
    for (from, to) in &vulnerability.dependency_relationships {
        push_relationship(&mut graph, from, "dependsOn", std::slice::from_ref(to));
    }
    for (expression, id) in &license_ids {
        graph.push(json!({
            "creationInfo":creation_info(),
            "simplelicensing_licenseExpression":expression,
            "spdxId":id,
            "type":"simplelicensing_LicenseExpression"
        }));
    }
    let mut licensed_artifacts = vec![(
        "urn:spdx:prismpm:release".to_owned(),
        GENERATED_ARTIFACT_LICENSE.to_owned(),
    )];
    licensed_artifacts.extend(
        artifact_ids
            .iter()
            .cloned()
            .map(|id| (id, GENERATED_ARTIFACT_LICENSE.to_owned())),
    );
    licensed_artifacts.extend(
        vulnerability
            .inventory
            .iter()
            .map(|item| (item.id.clone(), item.license.clone())),
    );
    licensed_artifacts.extend(external_ids);
    for (artifact, license) in licensed_artifacts {
        let target = if license == "NOASSERTION" {
            "expandedlicensing_NoAssertionLicense".to_owned()
        } else {
            license_ids[&license].clone()
        };
        push_relationship(
            &mut graph,
            &artifact,
            "hasDeclaredLicense",
            std::slice::from_ref(&target),
        );
        push_relationship(
            &mut graph,
            &artifact,
            "hasConcludedLicense",
            std::slice::from_ref(&target),
        );
    }
    let mut element_ids = graph
        .iter()
        .filter_map(|element| element["spdxId"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    element_ids.sort();
    graph.push(json!({
        "creationInfo":creation_info(),
        "element":element_ids,
        "profileConformance":["core","software","simpleLicensing"],
        "rootElement":["urn:spdx:prismpm:release"],
        "software_sbomType":["build","design","runtime"],
        "spdxId":"urn:spdx:prismpm:sbom",
        "type":"software_Sbom"
    }));
    graph.push(json!({
        "creationInfo":creation_info(),
        "element":["urn:spdx:prismpm:sbom"],
        "name":"PrismPM release SPDX 3.0.1 document",
        "profileConformance":["core","software","simpleLicensing"],
        "rootElement":["urn:spdx:prismpm:sbom"],
        "spdxId":"urn:spdx:prismpm:document",
        "type":"SpdxDocument"
    }));
    graph.sort_by(|left, right| left["spdxId"].as_str().cmp(&right["spdxId"].as_str()));
    let spdx_value = json!({
        "@context":"https://spdx.org/rdf/3.0.1/spdx-context.jsonld",
        "@graph":graph
    });
    validate_sbom_closure(&spdx_value, &layers, &external_artifacts)?;
    let spdx = encode_value(&spdx_value)?;
    let sbom_referrer = oci::attach_referrer(root, release_digest, SPDX, &spdx)?;
    let license_complete = vulnerability.unresolved_licenses.is_empty();
    // Local assembly cannot manufacture CI OIDC or transparency evidence.
    let promotion_eligible = false;
    let vulnerability_result = redact_value(&vulnerability.result);
    let policy = encode_value(&json!({
        "checks":[
            {"id":"oci-spdx-closure","result":"passed"},
            {"id":"secret-exclusion","result":"passed"},
            {"id":"license-inventory","result":if license_complete {"passed"} else {"incomplete"}},
            {"id":"vulnerability-scan","result":"passed"},
            {"id":"signed-promotion","result":"external-verification-required"}
        ],
        "license_inventory":{
            "component_count":vulnerability.inventory.len() + external_artifacts.len(),
            "complete":license_complete,
            "unresolved":vulnerability.unresolved_licenses
        },
        "promotion":{
            "eligible":promotion_eligible,
            "reason":"local assembly is unsigned development evidence; CI Cosign/OIDC verification is required"
        },
        "release_digest":release_digest,
        "schema":"prismpm/supply-chain-evidence/1",
        "secret_scan":{
            "finding_count":0,
            "redaction_policy":["authorization","tokens","credentials","operands","history","labels","secret-values"],
            "results":{"release_artifacts":secret_scan,"source_and_locks":vulnerability.source_secret_scan},
            "scanner":"prismpm/secret-pattern-profile/1"
        },
        "spdx":{
            "closure_matches_oci":true,
            "digest":sha(&spdx),
            "profile":"SPDX-3.0.1-software-system"
        },
        "vulnerability_input":{
            "edition":vulnerability.database_edition,
            "expires_unix":OSV_DATABASE_EXPIRES_UNIX,
            "freshness_policy_days":7,
            "source":vulnerability.database_source,
            "status":"within-production-policy"
        },
        "vulnerability_result":{
            "dependency_classes":["cargo-package","component-image","sdk-image"],
            "digest":vulnerability.result_digest,
            "image_finding_count":vulnerability.image_finding_count,
            "image_rejected_count":vulnerability.image_rejected_count,
            "image_scans":vulnerability.image_scans,
            "package_count":vulnerability.package_count + vulnerability.image_package_count,
            "result":vulnerability_result,
            "cargo_vulnerability_count":0,
            "image_vulnerability_finding_count":vulnerability.image_finding_count
        }
    }))?;
    if secret_marker(&policy).is_some() || secret_marker(&spdx).is_some() {
        return Err(PrismError::new(
            "PP7801",
            "generated supply-chain evidence contains a secret",
        ));
    }
    let policy_referrer = oci::attach_referrer(root, release_digest, POLICY, &policy)?;
    Ok(SupplyChainResult {
        schema: "prismpm/supply-chain-result/1".to_owned(),
        release_digest: release_digest.to_owned(),
        sbom_referrer,
        policy_referrer,
        scanned_artifacts: (layers.len() + external_artifacts.len()) as u64,
        scanned_dependencies: vulnerability.package_count + vulnerability.image_package_count,
        promotion_eligible,
        vulnerability_scope: format!(
            "{} through unix {}; zero Cargo findings in {} packages; {} retained and {} rejected image findings across {} image-package observations",
            vulnerability.database_edition,
            OSV_DATABASE_EXPIRES_UNIX,
            vulnerability.package_count,
            vulnerability.image_finding_count,
            vulnerability.image_rejected_count,
            vulnerability.image_package_count
        ),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn execution_boundary_rejects_wrong_native_inventory() {
        crate::sdk::execution_binding_regression(
            "supply_chain::tests::execution_boundary_rejects_wrong_native_inventory",
            |root| super::locked_inventory(root, Vec::new(), &mut Vec::new()).map(|_| ()),
        );
    }

    use super::{
        bare_digest_hex, bundle_certificate_claims, finding_rows, provenance_statement,
        redact_value, secret_marker, validate_advisory_coverage, validate_provenance_policy,
        validate_sbom_closure, verification_arguments, AdvisoryPolicy, AdvisoryScanFact,
        PromotionPolicy, ProvenanceInputs, VerifiedSignature,
    };
    use crate::oci::Descriptor;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    fn digest(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    #[test]
    fn authority_source_digest_accepts_only_canonical_bare_sha256() {
        let canonical = "a".repeat(64);
        assert_eq!(bare_digest_hex(&canonical), Some(canonical.as_str()));
        for rejected in [
            format!("sha256:{canonical}"),
            "A".repeat(64),
            "a".repeat(63),
            "a".repeat(65),
            format!("{}g", "a".repeat(63)),
        ] {
            assert_eq!(bare_digest_hex(&rejected), None);
        }
    }

    #[test]
    fn planted_secret_forms_are_rejected_without_echoing_values() {
        assert_eq!(
            secret_marker(b"Authorization: Bearer planted"),
            Some("bearer-token")
        );
        assert_eq!(secret_marker(b"not a secret reference"), None);
    }

    #[test]
    fn structured_evidence_redacts_sensitive_fields_recursively() {
        let redacted = redact_value(&json!({
            "authorization":"Bearer planted",
            "nested":{"history":[1,2],"safe":"visible"},
            "token_reference":"vault://token"
        }));
        assert_eq!(redacted["authorization"], "[REDACTED]");
        assert_eq!(
            redacted.pointer("/nested/history"),
            Some(&json!("[REDACTED]"))
        );
        assert_eq!(redacted.pointer("/nested/safe"), Some(&json!("visible")));
        assert_eq!(redacted["token_reference"], "vault://token");
    }

    fn provenance() -> (Value, PromotionPolicy, VerifiedSignature) {
        let release = digest('a');
        let bytes = provenance_statement(&ProvenanceInputs {
            subject_name: "example".to_owned(),
            subject_digest: release.clone(),
            builder_id: "https://github.com/actions/runner".to_owned(),
            invocation_id: "run-1".to_owned(),
            source_uri: "git+https://github.com/UOR-Foundation/example".to_owned(),
            source_revision: "0123456789abcdef".to_owned(),
            external_parameters: json!({"target":"release"}),
            dependencies: vec![("pkg:cargo/example@1.0.0".to_owned(), digest('b'))],
        })
        .unwrap();
        let statement = serde_json::from_slice(&bytes).unwrap();
        let policy = PromotionPolicy {
            schema: "prismpm/promotion-policy/1".to_owned(),
            subject_name: "example".to_owned(),
            release_digest: release.clone(),
            builder_id: "https://github.com/actions/runner".to_owned(),
            source_uri: "git+https://github.com/UOR-Foundation/example".to_owned(),
            source_revision: "0123456789abcdef".to_owned(),
            external_parameters: json!({"target":"release"}),
            required_dependencies: vec![("pkg:cargo/example@1.0.0".to_owned(), digest('b'))],
            issuer: "https://token.actions.githubusercontent.com".to_owned(),
            identity_subject: "repo:UOR-Foundation/example:environment:release".to_owned(),
            repository: "UOR-Foundation/example".to_owned(),
            workflow: ".github/workflows/release.yml".to_owned(),
            git_ref: "refs/tags/v1.0.0".to_owned(),
            environment: "release".to_owned(),
            require_transparency_log: true,
            require_hosted_build: true,
            trusted_root_digest: digest('f'),
        };
        let signature = VerifiedSignature {
            subject_digest: release,
            issuer: policy.issuer.clone(),
            identity_subject: policy.identity_subject.clone(),
            repository: policy.repository.clone(),
            workflow: policy.workflow.clone(),
            git_ref: policy.git_ref.clone(),
            environment: policy.environment.clone(),
            transparency_verified: true,
            hosted_build: true,
        };
        (statement, policy, signature)
    }

    #[test]
    fn provenance_binds_subject_source_builder_dependencies_and_identity() {
        let (statement, policy, signature) = provenance();
        let result = validate_provenance_policy(&statement, Some(&signature), &policy).unwrap();
        assert_eq!(result["promotion_eligible"], true);
        assert_eq!(result["build_level"], "hosted-build-platform");
    }

    #[test]
    fn promotion_rejects_unsigned_wrong_subject_builder_and_identity() {
        let (statement, policy, signature) = provenance();
        assert!(validate_provenance_policy(&statement, None, &policy).is_err());
        let mut wrong_subject = statement.clone();
        wrong_subject["subject"][0]["digest"]["sha256"] = json!("0".repeat(64));
        assert!(validate_provenance_policy(&wrong_subject, Some(&signature), &policy).is_err());
        let mut wrong_builder = statement.clone();
        wrong_builder["predicate"]["runDetails"]["builder"]["id"] = json!("untrusted");
        assert!(validate_provenance_policy(&wrong_builder, Some(&signature), &policy).is_err());
        let mut wrong_identity = signature;
        wrong_identity.transparency_verified = false;
        assert!(validate_provenance_policy(&statement, Some(&wrong_identity), &policy).is_err());
    }

    #[test]
    fn cosign_argv_closes_identity_claims_without_insecure_bypasses() {
        let (_, policy, _) = provenance();
        let arguments = verification_arguments(
            std::path::Path::new("bundle.json"),
            std::path::Path::new("trusted-root.json"),
            std::path::Path::new("subject"),
            &policy,
        );
        assert_eq!(arguments.first().map(String::as_str), Some("verify-blob"));
        for required in [
            "--bundle",
            "--trusted-root",
            "--certificate-identity",
            "--certificate-oidc-issuer",
            "--certificate-github-workflow-repository",
            "--certificate-github-workflow-name",
            "--certificate-github-workflow-ref",
            "--certificate-github-workflow-sha",
        ] {
            assert!(arguments.iter().any(|argument| argument == required));
        }
        assert!(!arguments.iter().any(|argument| {
            matches!(
                argument.as_str(),
                "--insecure-ignore-tlog" | "--insecure-ignore-sct" | "--check-claims=false"
            )
        }));
    }

    #[test]
    fn nonempty_or_legacy_bundle_is_never_verification() {
        for bundle in [
            json!({"bundle":"nonempty"}),
            json!({
                "mediaType":"application/vnd.dev.sigstore.bundle.v0.3+json",
                "verificationMaterial":{"tlogEntries":[]}
            }),
            json!({
                "mediaType":"application/vnd.dev.sigstore.bundle.v0.2+json",
                "verificationMaterial":{"certificate":{"rawBytes":"AA=="},"tlogEntries":[{}]}
            }),
        ] {
            let error = bundle_certificate_claims(&bundle).unwrap_err();
            assert_eq!(error.code, "PP7401");
        }
    }

    #[test]
    fn spdx_closure_rejects_missing_and_dangling_edges() {
        let descriptor = Descriptor {
            media_type: "application/octet-stream".to_owned(),
            digest: digest('c'),
            size: 1,
            artifact_type: None,
            annotations: Some(BTreeMap::new()),
        };
        let valid = json!({
            "@graph":[
                {"spdxId":"release","summary":"product-release"},
                {"spdxId":"artifact","summary":"release-artifact","verifiedUsing":[{"hashValue":"c".repeat(64)}]},
                {"spdxId":"relationship","type":"Relationship","from":"release","to":["artifact"]}
            ]
        });
        validate_sbom_closure(&valid, std::slice::from_ref(&descriptor), &[]).unwrap();
        let mut missing = valid.clone();
        missing["@graph"].as_array_mut().unwrap().remove(1);
        assert!(validate_sbom_closure(&missing, std::slice::from_ref(&descriptor), &[]).is_err());
        let mut dangling = valid;
        dangling["@graph"][2]["to"] = json!(["absent"]);
        assert!(validate_sbom_closure(&dangling, &[descriptor], &[]).is_err());
    }

    #[test]
    fn advisory_policy_requires_fresh_exact_complete_scan_coverage() {
        let subjects = vec![digest('d'), digest('e')];
        let policy = AdvisoryPolicy {
            required_subjects: subjects.clone(),
            database_id: "osv-locked".to_owned(),
            database_digest: digest('f'),
            max_age_seconds: 600,
        };
        let facts = subjects
            .iter()
            .map(|subject| AdvisoryScanFact {
                subject_digest: subject.clone(),
                subject_kind: "component-image".to_owned(),
                database_id: policy.database_id.clone(),
                database_digest: policy.database_digest.clone(),
                scanned_at_unix: 900,
                database_expires_unix: 2_000,
                rejected_findings: 0,
                result_digest: digest('1'),
            })
            .collect::<Vec<_>>();
        assert!(validate_advisory_coverage(&facts, &policy, 1_000).is_ok());
        assert!(validate_advisory_coverage(&facts[..1], &policy, 1_000).is_err());
        assert!(validate_advisory_coverage(&facts, &policy, 2_000).is_err());
        let mut wrong_database = facts;
        wrong_database[0].database_id = "floating-current".to_owned();
        assert!(validate_advisory_coverage(&wrong_database, &policy, 1_000).is_err());
    }

    #[test]
    fn image_scan_groups_are_retained_and_high_ratings_fail_closed() {
        let result = json!({
            "results":[{
                "packages":[{
                    "package":{"ecosystem":"Ubuntu","name":"libc6","version":"1"},
                    "groups":[
                        {"aliases":["CVE-1"],"ids":["USN-1"],"max_severity":"HIGH"},
                        {"aliases":["CVE-2"],"ids":["GO-2"],"max_severity":""}
                    ]
                }]
            }]
        });
        let (findings, package_count, rejected_count) = finding_rows(&result);
        assert_eq!(package_count, 1);
        assert_eq!(findings.len(), 2);
        assert_eq!(rejected_count, 1);
        assert!(findings.iter().any(|finding| finding["id"] == "GO-2"
            && finding["ratings"].as_array().is_some_and(Vec::is_empty)));
    }
}
