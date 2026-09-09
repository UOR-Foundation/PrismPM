//! OCI 1.1 product-release assembly and confined local content storage.

use crate::contracts::CanonicalDocument;
use crate::error::PrismError;
use crate::holo::canonical::{content_id, encode_value};
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};

/// OCI image-manifest media type adopted by Prism release graphs.
pub const OCI_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
/// OCI image-index media type adopted by Prism release graphs.
pub const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";
/// Docker Distribution schema-2 manifest media type accepted only for an
/// explicitly declared external runtime dependency.
pub const DOCKER_MANIFEST: &str = "application/vnd.docker.distribution.manifest.v2+json";
/// Docker Distribution schema-2 manifest-list media type accepted only for an
/// explicitly declared external runtime dependency.
pub const DOCKER_MANIFEST_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
/// Prism-owned root release configuration media type.
pub const PRISM_RELEASE: &str = "application/vnd.prismpm.product.release.v1+json";
/// Prism validation-attestation media type.
pub const PRISM_VALIDATION: &str = "application/vnd.prismpm.validation.v1+json";
/// in-toto Statement media type.
pub const INTOTO: &str = "application/vnd.in-toto+json";
/// OCI empty configuration media type used by artifact manifests.
pub const OCI_EMPTY: &str = "application/vnd.oci.empty.v1+json";
/// SPDX 3.0.1 JSON-LD media type used by release SBOM referrers.
pub const SPDX: &str = "application/spdx+json;version=3.0.1";
/// Prism supply-chain policy-result media type.
pub const PRISM_SUPPLY_CHAIN: &str = "application/vnd.prismpm.supply-chain.v1+json";
/// Digest-bound, executed production acceptance and public-capability coverage.
pub const PRISM_PRODUCTION_ACCEPTANCE: &str =
    "application/vnd.prismpm.production.acceptance.v1+json";
/// Verified Sigstore signature over one exact deployment-evidence document.
pub const PRISM_EVIDENCE_SIGNATURE: &str = "application/vnd.prismpm.evidence.signature.v1+json";
/// Canonical deployment evidence emitted by target lifecycle operations.
pub const PRISM_DEPLOYMENT_EVIDENCE: &str = "application/vnd.prismpm.deployment.evidence.v1+json";
/// Closed, exact-release identity and provenance policy used to replay every
/// Sigstore verification from a clean OCI pull.
pub const PRISM_PROMOTION_POLICY: &str = "application/vnd.prismpm.promotion-policy.v1+json";
/// Prism promotion-attestation media type reserved for signed status changes.
pub const PRISM_PROMOTION: &str = "application/vnd.prismpm.promotion.v1+json";
/// Sigstore/Cosign OCI 1.1 signature artifact type accepted as an external referrer.
pub const COSIGN_SIGNATURE: &str = "application/vnd.dev.cosign.artifact.sig.v1+json";
/// Sigstore/Cosign simple-signing payload media type.
pub const COSIGN_SIMPLE_SIGNING: &str = "application/vnd.dev.cosign.simplesigning.v1+json";

const VERIFIED_SCHEMA: &str = "prismpm/verified-oci-root/1";
const CORE_REFERRERS: [&str; 2] = [INTOTO, PRISM_VALIDATION];
const RELEASE_REFERRERS: [&str; 4] = [INTOTO, PRISM_VALIDATION, SPDX, PRISM_SUPPLY_CHAIN];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerifiedRoot {
    schema: String,
    policy: String,
    root: Descriptor,
    graph_digest: String,
    referrers: Vec<Descriptor>,
}

/// Closed OCI descriptor subset used at every graph edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    /// Descriptor media type.
    #[serde(rename = "mediaType")]
    pub media_type: String,
    /// Content digest.
    pub digest: String,
    /// Content byte length.
    pub size: u64,
    /// Artifact type supplied on manifest descriptors.
    #[serde(rename = "artifactType", skip_serializing_if = "Option::is_none")]
    pub artifact_type: Option<String>,
    /// Standard and Prism annotations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
}

/// Canonical result of a verified product build.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildResult {
    /// Result contract.
    pub schema: String,
    /// Mutable discovery name supplied only at publication time.
    pub reference: String,
    /// Canonical product-release configuration digest.
    pub product_digest: String,
    /// Immutable OCI root-manifest digest.
    pub release_digest: String,
    /// Canonical Prism model digest.
    pub model_digest: String,
    /// Atomic Prism build-manifest digest.
    pub build_digest: String,
    /// Confined local result path.
    pub evidence_path: String,
}

/// Canonical result of a registry transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferResult {
    /// Result contract.
    pub schema: String,
    /// Registry-qualified immutable reference.
    pub reference: String,
    /// Independently verified release digest.
    pub release_digest: String,
    /// True only after closure and remote descriptor checks pass.
    pub verified: bool,
}

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn digest_hex(digest: &str) -> Result<&str, PrismError> {
    let value = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| PrismError::new("PP6101", "OCI digest algorithm is not sha256"))?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI descriptor digest is malformed",
        ));
    }
    Ok(value)
}

fn relative(value: &str) -> Result<&str, PrismError> {
    if value.is_empty()
        || value.contains('\\')
        || Path::new(value)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI artifact path is not confined",
        ));
    }
    Ok(value)
}

fn validate_media_type(value: &str) -> Result<(), PrismError> {
    if value.is_empty()
        || value.len() > 256
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace() || !byte.is_ascii())
    {
        return Err(PrismError::new("PP6101", "OCI media type is malformed"));
    }
    let essence = value.split(';').next().unwrap_or_default();
    let Some((kind, subtype)) = essence.split_once('/') else {
        return Err(PrismError::new("PP6101", "OCI media type is malformed"));
    };
    let token = |part: &str| {
        !part.is_empty()
            && part.bytes().all(|byte| {
                byte.is_ascii_alphanumeric()
                    || matches!(
                        byte,
                        b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'.' | b'+' | b'-'
                    )
            })
    };
    if !token(kind) || !token(subtype) {
        return Err(PrismError::new("PP6101", "OCI media type is malformed"));
    }
    if essence.starts_with("application/vnd.prismpm.")
        && !matches!(
            essence,
            PRISM_RELEASE
                | PRISM_VALIDATION
                | PRISM_SUPPLY_CHAIN
                | PRISM_PRODUCTION_ACCEPTANCE
                | PRISM_EVIDENCE_SIGNATURE
                | PRISM_PROMOTION_POLICY
                | PRISM_PROMOTION
                | PRISM_DEPLOYMENT_EVIDENCE
                | "application/vnd.prismpm.failed-rollout.v1+json"
                | "application/vnd.prismpm.recovery.v1+json"
        )
    {
        return Err(PrismError::new(
            "PP6101",
            format!("unregistered Prism OCI media type: {essence}"),
        ));
    }
    Ok(())
}

fn validate_descriptor(descriptor: &Descriptor) -> Result<(), PrismError> {
    validate_media_type(&descriptor.media_type)?;
    digest_hex(&descriptor.digest)?;
    if descriptor.size > 10_737_418_240 {
        return Err(PrismError::new(
            "PP6101",
            "OCI descriptor exceeds the fixed limit",
        ));
    }
    if let Some(artifact_type) = descriptor.artifact_type.as_deref() {
        validate_media_type(artifact_type)?;
    }
    if let Some(annotations) = &descriptor.annotations {
        if annotations.len() > 4096
            || annotations.iter().any(|(key, value)| {
                key.is_empty()
                    || key.len() > 256
                    || value.len() > 4096
                    || key.bytes().any(|byte| byte.is_ascii_control())
                    || value.bytes().any(|byte| byte.is_ascii_control())
            })
        {
            return Err(PrismError::new(
                "PP6101",
                "OCI descriptor annotations exceed the supported boundary",
            ));
        }
    }
    Ok(())
}

fn prism_output(project: &Path) -> Result<PathBuf, PrismError> {
    let output = project.join(".prism");
    match std::fs::symlink_metadata(&output) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            return Err(PrismError::new(
                "PP8001",
                "Prism output root is not a confined directory",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(&output)
                .map_err(|error| PrismError::new("PP6101", format!("OCI output root: {error}")))?;
        }
        Err(error) => {
            return Err(PrismError::new(
                "PP6101",
                format!("OCI output root: {error}"),
            ));
        }
    }
    Ok(output)
}

/// One OCI image-layout content store below the project-owned output root.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open or create a confined OCI image layout.
    pub fn open(project: &Path) -> Result<Self, PrismError> {
        Self::open_layout(prism_output(project)?.join("oci"))
    }

    pub(crate) fn open_layout(root: PathBuf) -> Result<Self, PrismError> {
        if std::fs::symlink_metadata(&root)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(PrismError::new("PP8001", "OCI store is a symlink"));
        }
        std::fs::create_dir_all(root.join("blobs/sha256"))
            .map_err(|error| PrismError::new("PP6101", format!("OCI store: {error}")))?;
        let layout = root.join("oci-layout");
        let expected = b"{\"imageLayoutVersion\":\"1.0.0\"}";
        if layout.exists() {
            if std::fs::read(&layout)
                .map_err(|error| PrismError::new("PP6101", error.to_string()))?
                != expected
            {
                return Err(PrismError::new("PP6101", "OCI layout version changed"));
            }
        } else {
            write_new(&layout, expected)?;
        }
        Ok(Self { root })
    }

    /// Absolute layout root for standards-compatible clients.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn blob_path(&self, digest: &str) -> Result<PathBuf, PrismError> {
        Ok(self.root.join("blobs/sha256").join(digest_hex(digest)?))
    }

    /// Atomically insert immutable bytes and return their descriptor.
    pub fn put(&self, media_type: &str, bytes: &[u8]) -> Result<Descriptor, PrismError> {
        validate_media_type(media_type)?;
        if bytes.len() as u64 > 10_737_418_240 {
            return Err(PrismError::new(
                "PP6101",
                "OCI blob exceeds the fixed limit",
            ));
        }
        let digest = sha(bytes);
        let path = self.blob_path(&digest)?;
        if path.exists() {
            if std::fs::read(&path).map_err(|error| PrismError::new("PP6101", error.to_string()))?
                != bytes
            {
                return Err(PrismError::new("PP6101", "OCI content-address collision"));
            }
        } else {
            let parent = path
                .parent()
                .ok_or_else(|| PrismError::new("PP9001", "blob has no parent"))?;
            let mut temp = tempfile::Builder::new()
                .prefix("blob-")
                .tempfile_in(parent)
                .map_err(|error| PrismError::new("PP6101", format!("OCI staging: {error}")))?;
            temp.write_all(bytes)
                .and_then(|()| temp.as_file().sync_all())
                .map_err(|error| PrismError::new("PP6101", format!("OCI blob: {error}")))?;
            match temp.persist_noclobber(&path) {
                Ok(_) => {}
                Err(error) if path.exists() => {
                    if std::fs::read(&path)
                        .map_err(|read| PrismError::new("PP6101", read.to_string()))?
                        != bytes
                    {
                        return Err(PrismError::new(
                            "PP6101",
                            format!("OCI publication race: {}", error.error),
                        ));
                    }
                }
                Err(error) => {
                    return Err(PrismError::new(
                        "PP6101",
                        format!("OCI publish: {}", error.error),
                    ));
                }
            }
        }
        Ok(Descriptor {
            media_type: media_type.to_owned(),
            digest,
            size: bytes.len() as u64,
            artifact_type: None,
            annotations: None,
        })
    }

    /// Read and independently verify a descriptor.
    pub fn read(&self, descriptor: &Descriptor) -> Result<Vec<u8>, PrismError> {
        validate_descriptor(descriptor)?;
        let path = self.blob_path(&descriptor.digest)?;
        let metadata = std::fs::symlink_metadata(&path).map_err(|_| {
            PrismError::new("PP6101", format!("missing OCI blob {}", descriptor.digest))
        })?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() != descriptor.size
        {
            return Err(PrismError::new(
                "PP6101",
                "OCI descriptor size or file type changed",
            ));
        }
        let bytes =
            std::fs::read(path).map_err(|error| PrismError::new("PP6101", error.to_string()))?;
        if sha(&bytes) != descriptor.digest {
            return Err(PrismError::new(
                "PP6101",
                "OCI blob digest verification failed",
            ));
        }
        Ok(bytes)
    }

    fn descriptor(&self, digest: &str, media_type: &str) -> Result<Descriptor, PrismError> {
        let path = self.blob_path(digest)?;
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| PrismError::new("PP6101", format!("unknown OCI digest {digest}")))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(PrismError::new(
                "PP6101",
                format!("OCI digest is not an immutable regular file: {digest}"),
            ));
        }
        Ok(Descriptor {
            media_type: media_type.to_owned(),
            digest: digest.to_owned(),
            size: metadata.len(),
            artifact_type: None,
            annotations: None,
        })
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), PrismError> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| PrismError::new("PP6101", format!("{}: {error}", path.display())))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| PrismError::new("PP6101", format!("{}: {error}", path.display())))
}

fn file_media(path: &str) -> &'static str {
    if path.ends_with(".holo") {
        "application/vnd.hologram.archive.v1"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".crate") {
        "application/vnd.rust.crate"
    } else if path.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}

fn file_descriptor(
    store: &Store,
    path: &str,
    role: &str,
    bytes: &[u8],
) -> Result<Descriptor, PrismError> {
    relative(path)?;
    let mut descriptor = store.put(file_media(path), bytes)?;
    descriptor.annotations = Some(BTreeMap::from([
        ("org.opencontainers.image.title".to_owned(), path.to_owned()),
        ("org.prismpm.role".to_owned(), role.to_owned()),
    ]));
    Ok(descriptor)
}

fn oci_manifest(
    store: &Store,
    artifact_type: &str,
    config: Descriptor,
    layers: Vec<Descriptor>,
    subject: Option<Descriptor>,
) -> Result<Descriptor, PrismError> {
    validate_media_type(artifact_type)?;
    validate_descriptor(&config)?;
    for layer in &layers {
        validate_descriptor(layer)?;
    }
    if let Some(subject) = &subject {
        validate_descriptor(subject)?;
    }
    let mut value = json!({
        "artifactType": artifact_type,
        "config": config,
        "layers": layers,
        "mediaType": OCI_MANIFEST,
        "schemaVersion": 2
    });
    if let Some(subject) = subject {
        value.as_object_mut().expect("manifest object").insert(
            "subject".to_owned(),
            serde_json::to_value(subject).expect("descriptor"),
        );
    }
    let bytes = encode_value(&value)?;
    let mut descriptor = store.put(OCI_MANIFEST, &bytes)?;
    descriptor.artifact_type = Some(artifact_type.to_owned());
    Ok(descriptor)
}

fn source_revision(model: &Value) -> Result<String, PrismError> {
    if let Some(revision) = std::env::var_os("PRISMPM_SOURCE_REVISION") {
        let revision = revision
            .into_string()
            .map_err(|_| PrismError::new("PP7401", "PRISMPM_SOURCE_REVISION is not UTF-8"))?;
        if !matches!(revision.len(), 40 | 64)
            || !revision
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(PrismError::new(
                "PP7401",
                "PRISMPM_SOURCE_REVISION is not a lowercase Git object ID",
            ));
        }
        return Ok(revision);
    }
    model
        .pointer("/provenance/source_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| PrismError::new("PP7401", "model source identity is absent"))
}

/// Assemble a verified build into a deterministic OCI product graph.
pub fn assemble(
    root: &Path,
    build_id: &str,
    reference: &str,
    validations: &[Value],
) -> Result<ProductBuildResult, PrismError> {
    validate_reference(reference, false)?;
    // All fallible production-policy checks run before any root descriptor is
    // published into the local OCI index.
    let prism_root = prism_output(root)?;
    let staging_project = tempfile::Builder::new()
        .prefix("oci-assembly-")
        .tempdir_in(&prism_root)
        .map_err(|error| PrismError::new("PP6101", format!("OCI staging: {error}")))?;
    let store = Store::open(staging_project.path())?;
    let build_root = root.join(".prism/build").join(build_id);
    if !build_root.is_dir() {
        return Err(PrismError::new(
            "PP6101",
            "verified build directory is absent",
        ));
    }
    let manifest_bytes = std::fs::read(build_root.join("manifest.json"))
        .map_err(|error| PrismError::new("PP6101", format!("build manifest: {error}")))?;
    let build_manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| PrismError::new("PP6101", format!("build manifest: {error}")))?;
    let rows = build_manifest["files"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "build manifest has no files"))?;
    let mut layers = Vec::new();
    let mut release_artifacts = Vec::new();
    for row in rows {
        let path = row["path"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP6101", "build file path is absent"))?;
        relative(path)?;
        let bytes = std::fs::read(build_root.join(path))
            .map_err(|error| PrismError::new("PP6101", format!("{path}: {error}")))?;
        let descriptor = file_descriptor(&store, path, "release-artifact", &bytes)?;
        if row["sha256"].as_str() != Some(descriptor.digest.trim_start_matches("sha256:")) {
            return Err(PrismError::new(
                "PP6101",
                format!("build artifact changed: {path}"),
            ));
        }
        release_artifacts.push(json!({
            "annotations": descriptor.annotations,
            "digest": descriptor.digest,
            "media_type": descriptor.media_type,
            "role": "release-artifact",
            "size": descriptor.size
        }));
        layers.push(descriptor);
    }
    let model_bytes = std::fs::read(build_root.join("model.prism.json"))
        .map_err(|error| PrismError::new("PP6101", format!("model: {error}")))?;
    let model: Value = serde_json::from_slice(&model_bytes)
        .map_err(|error| PrismError::new("PP6101", format!("model: {error}")))?;
    let system = std::fs::read(build_root.join("system.prism.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
    let vulnerability = crate::supply_chain::scan_vulnerabilities(
        root,
        &system.iter().cloned().collect::<Vec<_>>(),
    )?;
    let product = system
        .as_ref()
        .and_then(|value| value.pointer("/product/id"))
        .and_then(Value::as_str)
        .or_else(|| model.pointer("/application/name").and_then(Value::as_str))
        .unwrap_or("prism-product");
    let release = system
        .as_ref()
        .and_then(|value| value.pointer("/product/version"))
        .and_then(Value::as_str)
        .unwrap_or("development");
    let mut external_artifacts = system
        .as_ref()
        .and_then(|value| value["artifacts"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|artifact| {
            let media_type = artifact["media_type"].as_str()?;
            if !matches!(
                media_type,
                OCI_INDEX | OCI_MANIFEST | DOCKER_MANIFEST | DOCKER_MANIFEST_LIST
            ) {
                return None;
            }
            let digest = artifact["digest"].as_str()?;
            let path = artifact["path"].as_str()?;
            Some(json!({
                "digest":digest,
                "license_expression":artifact["license_expression"],
                "media_type":media_type,
                "reference":format!("{path}@{digest}"),
                "role":artifact["role"]
            }))
        })
        .collect::<Vec<_>>();
    external_artifacts
        .sort_by(|left, right| left["reference"].as_str().cmp(&right["reference"].as_str()));
    if external_artifacts.windows(2).any(|rows| rows[0] == rows[1]) {
        return Err(PrismError::new(
            "PP6101",
            "system declares a duplicate external OCI artifact",
        ));
    }
    let sdk_lock_bytes = std::fs::read(root.join("prismpm.lock"))
        .map_err(|_| PrismError::new("PP5401", "prismpm.lock is required"))?;
    let sdk_lock = CanonicalDocument::parse("prismpm/sdk-lock/1", &sdk_lock_bytes)?;
    let standards_lock_bytes = std::fs::read(root.join("standards.lock"))
        .map_err(|_| PrismError::new("PP1101", "standards.lock is required"))?;
    let _standards_lock =
        CanonicalDocument::parse("prismpm/standards-lock/1", &standards_lock_bytes)?;
    let sdk_lock_digest = sha(&sdk_lock_bytes);
    let standards_lock_digest = sha(&standards_lock_bytes);
    let sdk_image = sdk_lock.value()["sdk_image"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP5401", "SDK image reference is absent"))?;
    let sdk_image_digest = validate_reference(sdk_image, true)?.to_owned();
    for (path, role, bytes) in [
        ("prismpm.lock", "sdk-lock", sdk_lock_bytes.as_slice()),
        (
            "standards.lock",
            "standards-lock",
            standards_lock_bytes.as_slice(),
        ),
    ] {
        let descriptor = file_descriptor(&store, path, role, bytes)?;
        release_artifacts.push(json!({
            "annotations": descriptor.annotations,
            "digest": descriptor.digest,
            "media_type": descriptor.media_type,
            "role": role,
            "size": descriptor.size
        }));
        layers.push(descriptor);
    }
    layers.sort_by(|left, right| left.annotations.cmp(&right.annotations));
    release_artifacts.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
    let release_value = json!({
        "artifacts": release_artifacts,
        "external_artifacts": external_artifacts,
        "model_digest": sha(&model_bytes),
        "product": product,
        "release": release,
        "schema": "prismpm/product-release/1",
        "sdk_digest": sdk_image_digest,
        "sdk_lock": sdk_lock_digest,
        "standards_lock": standards_lock_digest,
        "status": "development"
    });
    let release_doc = CanonicalDocument::from_value("prismpm/product-release/1", release_value)?;
    let config = store.put(PRISM_RELEASE, release_doc.bytes())?;
    let root_descriptor = oci_manifest(&store, PRISM_RELEASE, config, layers, None)?;

    let empty = store.put(OCI_EMPTY, b"{}")?;
    let semantic_source_id = model
        .pointer("/provenance/source_id")
        .and_then(Value::as_str)
        .ok_or_else(|| PrismError::new("PP7401", "model source identity is absent"))?;
    let source_revision = source_revision(&model)?;
    let source_uri = std::fs::read_to_string(root.join("Cargo.toml"))
        .ok()
        .and_then(|text| text.parse::<toml::Value>().ok())
        .and_then(|manifest| {
            manifest
                .get("package")
                .and_then(|package| package.get("repository"))
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("urn:prismpm:source:{product}"));
    let provenance =
        crate::supply_chain::provenance_statement(&crate::supply_chain::ProvenanceInputs {
            subject_name: product.to_owned(),
            subject_digest: root_descriptor.digest.clone(),
            builder_id: sdk_image.to_owned(),
            invocation_id: build_id.to_owned(),
            source_uri,
            source_revision,
            external_parameters: json!({
                "model_digest":sha(&model_bytes),
                "product_release":release,
                "semantic_source_id":semantic_source_id,
                "standards_lock":standards_lock_digest
            }),
            dependencies: vec![
                (sdk_image.to_owned(), sdk_image_digest),
                ("urn:prismpm:sdk-lock".to_owned(), sdk_lock_digest),
                (
                    "urn:prismpm:standards-lock".to_owned(),
                    standards_lock_digest,
                ),
                (
                    "urn:prismpm:build-manifest".to_owned(),
                    sha(&manifest_bytes),
                ),
            ],
        })?;
    let provenance_blob = store.put(INTOTO, &provenance)?;
    let provenance_manifest = oci_manifest(
        &store,
        INTOTO,
        empty.clone(),
        vec![provenance_blob],
        Some(root_descriptor.clone()),
    )?;
    let validation = encode_value(&json!({
        "build_digest": format!("sha256:{}", content_id(&manifest_bytes)),
        "oracle_results": validations,
        "result": "passed",
        "schema": "prismpm/release-validation/1",
        "subject": root_descriptor.digest
    }))?;
    let validation_blob = store.put(PRISM_VALIDATION, &validation)?;
    let validation_manifest = oci_manifest(
        &store,
        PRISM_VALIDATION,
        empty,
        vec![validation_blob],
        Some(root_descriptor.clone()),
    )?;
    let index = encode_value(&json!({
        "manifests": [
            {"annotations":{"org.opencontainers.image.ref.name":reference},"artifactType":PRISM_RELEASE,"digest":root_descriptor.digest,"mediaType":root_descriptor.media_type,"size":root_descriptor.size},
            provenance_manifest,
            validation_manifest
        ],
        "mediaType": OCI_INDEX,
        "schemaVersion": 2
    }))?;
    atomic_replace(&store.root.join("index.json"), &index)?;
    write_verified_marker(
        &store,
        verified_release(&store, &root_descriptor.digest, &CORE_REFERRERS, "core")?,
    )?;
    crate::supply_chain::attach_build_evidence(
        staging_project.path(),
        &root_descriptor.digest,
        &vulnerability,
    )?;
    write_verified_marker(
        &store,
        verified_release(
            &store,
            &root_descriptor.digest,
            &RELEASE_REFERRERS,
            "release",
        )?,
    )?;

    let evidence_path = format!(
        ".prism/releases/{}/result.json",
        digest_hex(&root_descriptor.digest)?
    );
    let result = ProductBuildResult {
        schema: "prismpm/product-release-result/1".to_owned(),
        reference: reference.to_owned(),
        product_digest: sha(release_doc.bytes()),
        release_digest: root_descriptor.digest.clone(),
        model_digest: sha(&model_bytes),
        build_digest: format!("sha256:{}", content_id(&manifest_bytes)),
        evidence_path: evidence_path.clone(),
    };
    let bytes = encode_value(
        &serde_json::to_value(&result)
            .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
    )?;
    let evidence = root.join(&evidence_path);
    std::fs::create_dir_all(evidence.parent().expect("evidence parent"))
        .map_err(|error| PrismError::new("PP6101", error.to_string()))?;
    if evidence.exists() {
        if std::fs::read(&evidence).map_err(|error| PrismError::new("PP6101", error.to_string()))?
            != bytes
        {
            return Err(PrismError::new(
                "PP6101",
                "release result changed for an existing digest",
            ));
        }
    } else {
        write_new(&evidence, &bytes)?;
    }
    publish_staged_layout(root, &store, &root_descriptor.digest, reference)?;
    Ok(result)
}

fn atomic_replace(path: &Path, bytes: &[u8]) -> Result<(), PrismError> {
    let parent = path
        .parent()
        .ok_or_else(|| PrismError::new("PP9001", "path has no parent"))?;
    let mut temp = tempfile::Builder::new()
        .prefix("index-")
        .tempfile_in(parent)
        .map_err(|error| PrismError::new("PP6101", error.to_string()))?;
    temp.write_all(bytes)
        .and_then(|()| temp.as_file().sync_all())
        .map_err(|error| PrismError::new("PP6101", error.to_string()))?;
    temp.persist(path)
        .map_err(|error| PrismError::new("PP6101", error.to_string()))?;
    File::open(parent)
        .and_then(|file| file.sync_all())
        .map_err(|error| PrismError::new("PP6101", error.to_string()))
}

fn manifest_descriptors(value: &Value) -> impl Iterator<Item = &Value> {
    value
        .get("config")
        .into_iter()
        .chain(value.get("subject"))
        .chain(
            value
                .get("layers")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        )
}

fn manifest(store: &Store, descriptor: &Descriptor) -> Result<Value, PrismError> {
    if descriptor.media_type != OCI_MANIFEST {
        return Err(PrismError::new(
            "PP6101",
            "OCI manifest descriptor has the wrong media type",
        ));
    }
    let bytes = store.read(descriptor)?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| PrismError::new("PP6101", format!("OCI manifest: {error}")))?;
    if encode_value(&value)? != bytes
        || value["schemaVersion"] != 2
        || value["mediaType"] != OCI_MANIFEST
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI manifest is noncanonical or unsupported",
        ));
    }
    let object = value
        .as_object()
        .ok_or_else(|| PrismError::new("PP6101", "OCI manifest is not an object"))?;
    let allowed = [
        "annotations",
        "artifactType",
        "config",
        "layers",
        "mediaType",
        "schemaVersion",
        "subject",
    ];
    if object.keys().any(|key| !allowed.contains(&key.as_str()))
        || !value["layers"].is_array()
        || !value["config"].is_object()
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI manifest fields are unsupported",
        ));
    }
    let artifact_type = value["artifactType"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP6101", "OCI artifact type is absent"))?;
    validate_media_type(artifact_type)?;
    if descriptor
        .artifact_type
        .as_deref()
        .is_some_and(|declared| declared != artifact_type)
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI descriptor artifact type disagrees with its manifest",
        ));
    }
    let mut edges = BTreeSet::new();
    for child in manifest_descriptors(&value) {
        let child: Descriptor = serde_json::from_value(child.clone())
            .map_err(|error| PrismError::new("PP6101", format!("OCI descriptor: {error}")))?;
        validate_descriptor(&child)?;
        if !edges.insert(child.digest.clone()) {
            return Err(PrismError::new(
                "PP6101",
                "OCI manifest contains a duplicate descriptor edge",
            ));
        }
    }
    Ok(value)
}

fn visit_graph(
    store: &Store,
    descriptor: Descriptor,
    visiting: &mut BTreeSet<String>,
    complete: &mut BTreeMap<String, (String, u64)>,
    manifests: &mut u64,
) -> Result<(), PrismError> {
    validate_descriptor(&descriptor)?;
    if let Some((media_type, size)) = complete.get(&descriptor.digest) {
        if media_type != &descriptor.media_type || *size != descriptor.size {
            return Err(PrismError::new(
                "PP6101",
                "OCI digest is referenced with inconsistent descriptor metadata",
            ));
        }
        return Ok(());
    }
    if !visiting.insert(descriptor.digest.clone()) {
        return Err(PrismError::new("PP6101", "OCI digest cycle detected"));
    }
    let bytes = store.read(&descriptor)?;
    if descriptor.media_type == OCI_MANIFEST {
        *manifests += 1;
        let value = manifest(store, &descriptor)?;
        for child in manifest_descriptors(&value) {
            let child: Descriptor = serde_json::from_value(child.clone())
                .map_err(|error| PrismError::new("PP6101", format!("OCI descriptor: {error}")))?;
            visit_graph(store, child, visiting, complete, manifests)?;
        }
    } else if bytes.starts_with(b"{") && descriptor.media_type == OCI_INDEX {
        return Err(PrismError::new(
            "PP6101",
            "an OCI index cannot be hidden behind a blob edge",
        ));
    }
    visiting.remove(&descriptor.digest);
    complete.insert(descriptor.digest, (descriptor.media_type, descriptor.size));
    if complete.len() > 65_536 {
        return Err(PrismError::new(
            "PP6101",
            "OCI graph exceeds the descriptor limit",
        ));
    }
    Ok(())
}

/// Verify closure, sizes, media types, duplicate edges, and every digest without execution.
pub fn verify_graph(store: &Store, root_digest: &str) -> Result<Value, PrismError> {
    let root = store.descriptor(root_digest, OCI_MANIFEST)?;
    let mut visiting = BTreeSet::new();
    let mut complete = BTreeMap::new();
    let mut manifests = 0_u64;
    visit_graph(store, root, &mut visiting, &mut complete, &mut manifests)?;
    Ok(json!({
        "descriptor_count": complete.len(),
        "manifest_count": manifests,
        "root_digest": root_digest,
        "schema": "prismpm/inspect-result/1",
        "verified": true
    }))
}

fn index_rows(store: &Store) -> Result<Vec<Descriptor>, PrismError> {
    let bytes = std::fs::read(store.root.join("index.json"))
        .map_err(|error| PrismError::new("PP6101", format!("OCI index: {error}")))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| PrismError::new("PP6101", format!("OCI index: {error}")))?;
    if encode_value(&value)? != bytes
        || value["schemaVersion"] != 2
        || value["mediaType"] != OCI_INDEX
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI index is noncanonical or unsupported",
        ));
    }
    let object = value
        .as_object()
        .ok_or_else(|| PrismError::new("PP6101", "OCI index is not an object"))?;
    let allowed = [
        "annotations",
        "artifactType",
        "manifests",
        "mediaType",
        "schemaVersion",
        "subject",
    ];
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(PrismError::new(
            "PP6101",
            "OCI index fields are unsupported",
        ));
    }
    let rows = value["manifests"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "OCI index has no manifests"))?;
    if rows.len() > 65_536 {
        return Err(PrismError::new(
            "PP6101",
            "OCI index exceeds the descriptor limit",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut references = BTreeMap::new();
    let mut descriptors = Vec::new();
    for row in rows {
        let descriptor: Descriptor = serde_json::from_value(row.clone())
            .map_err(|error| PrismError::new("PP6101", format!("OCI index descriptor: {error}")))?;
        validate_descriptor(&descriptor)?;
        if !seen.insert(descriptor.digest.clone()) {
            return Err(PrismError::new(
                "PP6101",
                "OCI index contains a duplicate descriptor",
            ));
        }
        if descriptor.media_type != OCI_MANIFEST {
            return Err(PrismError::new(
                "PP6101",
                "OCI index contains a non-manifest descriptor",
            ));
        }
        let value = manifest(store, &descriptor)?;
        if descriptor.artifact_type.as_deref() != value["artifactType"].as_str() {
            return Err(PrismError::new(
                "PP6101",
                "OCI index artifact type disagrees with its manifest",
            ));
        }
        if let Some(reference) = descriptor
            .annotations
            .as_ref()
            .and_then(|rows| rows.get("org.opencontainers.image.ref.name"))
        {
            if let Some(previous) = references.insert(reference.clone(), descriptor.digest.clone())
            {
                if previous != descriptor.digest {
                    return Err(PrismError::new(
                        "PP6101",
                        "OCI index contains a mutable reference race",
                    ));
                }
            }
        }
        descriptors.push(descriptor);
    }
    Ok(descriptors)
}

fn indexed_referrers(store: &Store, subject_digest: &str) -> Result<Vec<Descriptor>, PrismError> {
    let mut referrers = Vec::new();
    for descriptor in index_rows(store)? {
        let manifest = manifest(store, &descriptor)?;
        if manifest
            .get("subject")
            .and_then(|subject| subject["digest"].as_str())
            == Some(subject_digest)
        {
            verify_graph(store, &descriptor.digest)?;
            referrers.push(descriptor);
        }
    }
    referrers.sort_by(|left, right| left.digest.cmp(&right.digest));
    Ok(referrers)
}

fn descriptor_same_edge(left: &Descriptor, right: &Descriptor) -> bool {
    left.digest == right.digest
        && left.size == right.size
        && left.media_type == right.media_type
        && left.artifact_type == right.artifact_type
}

fn validate_referrer(
    store: &Store,
    root: &Descriptor,
    descriptor: &Descriptor,
) -> Result<String, PrismError> {
    let value = manifest(store, descriptor)?;
    let subject: Descriptor = serde_json::from_value(value["subject"].clone())
        .map_err(|error| PrismError::new("PP6101", format!("OCI subject: {error}")))?;
    if !descriptor_same_edge(&subject, root) {
        return Err(PrismError::new(
            "PP6101",
            "OCI referrer is bound to a confused subject descriptor",
        ));
    }
    let artifact_type = value["artifactType"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP6101", "OCI referrer artifact type is absent"))?;
    if artifact_type == PRISM_RELEASE || artifact_type == OCI_MANIFEST || artifact_type == OCI_INDEX
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI referrer uses a reserved root media type",
        ));
    }
    let config: Descriptor = serde_json::from_value(value["config"].clone())
        .map_err(|error| PrismError::new("PP6101", format!("OCI config: {error}")))?;
    if config.media_type != OCI_EMPTY || store.read(&config)? != b"{}" {
        return Err(PrismError::new(
            "PP6101",
            "OCI referrer config is not the canonical empty descriptor",
        ));
    }
    let layers = value["layers"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "OCI referrer layers are absent"))?;
    if layers.len() != 1 {
        return Err(PrismError::new(
            "PP6101",
            "OCI referrer must contain exactly one evidence layer",
        ));
    }
    let layer: Descriptor = serde_json::from_value(layers[0].clone())
        .map_err(|error| PrismError::new("PP6101", format!("OCI evidence: {error}")))?;
    if layer.media_type != artifact_type
        && !(artifact_type == COSIGN_SIGNATURE && layer.media_type == COSIGN_SIMPLE_SIGNING)
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI referrer role and evidence media type disagree",
        ));
    }
    let evidence = store.read(&layer)?;
    if artifact_type.contains("json") {
        let document: Value = serde_json::from_slice(&evidence)
            .map_err(|error| PrismError::new("PP6101", format!("OCI evidence JSON: {error}")))?;
        if encode_value(&document)? != evidence {
            return Err(PrismError::new(
                "PP6101",
                "OCI evidence JSON is not canonical",
            ));
        }
        for key in ["subject", "release_digest"] {
            if document
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|claimed| claimed != root.digest)
            {
                return Err(PrismError::new(
                    "PP6101",
                    "OCI evidence claims a different subject digest",
                ));
            }
        }
        if artifact_type == COSIGN_SIGNATURE {
            let bundle_bytes = encode_value(&document["bundle"])?;
            if document["schema"] != "prismpm/cosign-verification/1"
                || document["subject"].as_str() != Some(root.digest.as_str())
                || document["signed_blob_digest"].as_str() != Some(root.digest.as_str())
                || document["verified"] != true
                || document["transparency_log_verified"] != true
                || document["bundle"]["mediaType"]
                    != "application/vnd.dev.sigstore.bundle.v0.3+json"
                || document["bundle"]
                    .pointer("/verificationMaterial/tlogEntries")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
                || document["bundle_digest"].as_str() != Some(sha(&bundle_bytes).as_str())
                || document["trusted_root_digest"]
                    .as_str()
                    .and_then(|digest| digest_hex(digest).ok())
                    .is_none()
            {
                return Err(PrismError::new(
                    "PP6101",
                    "Cosign referrer is not exact-subject verified v0.3 bundle evidence",
                ));
            }
        }
        if artifact_type == PRISM_PROMOTION {
            let from = document["from"].as_str().unwrap_or_default();
            let to = document["to"].as_str().unwrap_or_default();
            let valid_transition = matches!(
                (from, to),
                ("development", "candidate") | ("candidate", "accepted")
            );
            let statement_bytes = encode_value(&document["statement"])?;
            if !valid_transition
                || document["schema"] != "prismpm/promotion/1"
                || document["subject"].as_str() != Some(root.digest.as_str())
                || document["verified"] != true
                || document["statement_digest"].as_str() != Some(sha(&statement_bytes).as_str())
                || document["statement"]["schema"] != "prismpm/promotion-statement/1"
                || document["statement"]["subject"].as_str() != Some(root.digest.as_str())
                || document["statement"]["from"].as_str() != Some(from)
                || document["statement"]["to"].as_str() != Some(to)
                || document["statement"]["policy_result"]["promotion_eligible"] != true
                || document["signature"]["schema"] != "prismpm/cosign-verification/1"
                || document["signature"]["subject"].as_str() != Some(root.digest.as_str())
                || document["signature"]["signed_blob_digest"] != document["statement_digest"]
                || document["signature"]["verified"] != true
                || document["signature"]["transparency_log_verified"] != true
            {
                return Err(PrismError::new(
                    "PP6101",
                    "promotion must be signed and preserve one exact subject",
                ));
            }
        }
        if artifact_type == PRISM_PROMOTION_POLICY {
            let policy_document =
                CanonicalDocument::parse("prismpm/promotion-policy/1", &evidence)?;
            let policy: crate::supply_chain::PromotionPolicy =
                serde_json::from_value(policy_document.value().clone()).map_err(|error| {
                    PrismError::new("PP6101", format!("promotion policy: {error}"))
                })?;
            if policy.schema != "prismpm/promotion-policy/1" || policy.release_digest != root.digest
            {
                return Err(PrismError::new(
                    "PP6101",
                    "promotion policy is malformed or bound to another release",
                ));
            }
        }
        if artifact_type == PRISM_PRODUCTION_ACCEPTANCE {
            let acceptance =
                CanonicalDocument::parse("prismpm/production-acceptance/1", &evidence)?;
            if acceptance.value()["release_digest"].as_str() != Some(root.digest.as_str()) {
                return Err(PrismError::new(
                    "PP6101",
                    "production acceptance evidence claims a different release",
                ));
            }
            crate::acceptance::verify_closure(acceptance.value())?;
        }
        if artifact_type == PRISM_DEPLOYMENT_EVIDENCE {
            let document = CanonicalDocument::parse("prismpm/deployment-evidence/1", &evidence)?;
            let mut core = document.value().clone();
            let claimed = core
                .as_object_mut()
                .and_then(|object| object.remove("evidence_digest"))
                .and_then(|value| value.as_str().map(str::to_owned))
                .ok_or_else(|| PrismError::new("PP6101", "deployment evidence digest is absent"))?;
            if claimed != sha(&encode_value(&core)?) {
                return Err(PrismError::new(
                    "PP6101",
                    "deployment evidence self-digest changed",
                ));
            }
        }
        if artifact_type == PRISM_EVIDENCE_SIGNATURE
            && (CanonicalDocument::parse("prismpm/evidence-signature/1", &evidence).is_err()
                || document["schema"] != "prismpm/evidence-signature/1"
                || document["subject"].as_str() != Some(root.digest.as_str())
                || document["verified"] != true
                || document["evidence_closure"]["release_digest"].as_str()
                    != Some(root.digest.as_str())
                || document["signature"]["schema"] != "prismpm/cosign-verification/1"
                || document["signature"]["subject"].as_str() != Some(root.digest.as_str())
                || document["signature"]["verified"] != true
                || document["signature"]["transparency_log_verified"] != true)
        {
            return Err(PrismError::new(
                "PP6101",
                "deployment-evidence signature is incomplete or bound to another release",
            ));
        }
        if artifact_type == PRISM_EVIDENCE_SIGNATURE {
            let closure_bytes = encode_value(&document["evidence_closure"])?;
            if document["evidence_closure_digest"].as_str() != Some(sha(&closure_bytes).as_str())
                || document["signature"]["signed_blob_digest"].as_str()
                    != Some(sha(&closure_bytes).as_str())
            {
                return Err(PrismError::new(
                    "PP6101",
                    "evidence signature does not bind its canonical closure bytes",
                ));
            }
            let indexed = indexed_referrers(store, &root.digest)?;
            for row in document["evidence_closure"]["evidence"]
                .as_array()
                .expect("schema-validated evidence closure")
            {
                let evidence_referrer = row["referrer"].as_str().expect("validated referrer");
                let evidence_descriptor = indexed
                    .iter()
                    .find(|descriptor| descriptor.digest == evidence_referrer)
                    .ok_or_else(|| {
                        PrismError::new(
                            "PP6101",
                            "evidence signature references an unindexed referrer",
                        )
                    })?;
                if evidence_descriptor.artifact_type.as_deref() != Some(PRISM_DEPLOYMENT_EVIDENCE) {
                    return Err(PrismError::new(
                        "PP6101",
                        "evidence signature references a non-deployment referrer",
                    ));
                }
                let evidence_document = referrer_evidence(store, evidence_descriptor)?;
                if evidence_document["release_digest"].as_str() != Some(root.digest.as_str())
                    || evidence_document["evidence_digest"] != row["digest"]
                    || evidence_document["operation"] != row["operation"]
                    || evidence_document["target"] != row["target"]
                {
                    return Err(PrismError::new(
                        "PP6101",
                        "signed evidence closure does not match its deployment evidence",
                    ));
                }
            }
            let bundle_bytes = encode_value(&document["signature"]["bundle"])?;
            if document["signature"]["bundle"]["mediaType"]
                != "application/vnd.dev.sigstore.bundle.v0.3+json"
                || document["signature"]["bundle"]
                    .pointer("/verificationMaterial/tlogEntries")
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
                || document["signature"]["bundle_digest"].as_str()
                    != Some(sha(&bundle_bytes).as_str())
            {
                return Err(PrismError::new(
                    "PP6101",
                    "evidence signature has no verified standard Sigstore bundle",
                ));
            }
        }
    }
    Ok(artifact_type.to_owned())
}

fn referrer_evidence(store: &Store, descriptor: &Descriptor) -> Result<Value, PrismError> {
    let value = manifest(store, descriptor)?;
    let layers = value["layers"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "OCI referrer layers are absent"))?;
    if layers.len() != 1 {
        return Err(PrismError::new(
            "PP6101",
            "OCI referrer must contain exactly one evidence layer",
        ));
    }
    let layer: Descriptor = serde_json::from_value(layers[0].clone())
        .map_err(|error| PrismError::new("PP6101", format!("OCI evidence: {error}")))?;
    serde_json::from_slice(&store.read(&layer)?)
        .map_err(|error| PrismError::new("PP6101", format!("OCI evidence JSON: {error}")))
}

fn promotion_status(store: &Store, state: &VerifiedRoot) -> Result<&'static str, PrismError> {
    let mut transitions = Vec::new();
    for descriptor in &state.referrers {
        if descriptor.artifact_type.as_deref() == Some(PRISM_PROMOTION) {
            let evidence = referrer_evidence(store, descriptor)?;
            transitions.push((
                evidence["from"].as_str().unwrap_or_default().to_owned(),
                evidence["to"].as_str().unwrap_or_default().to_owned(),
            ));
        }
    }
    let mut status = "development";
    while !transitions.is_empty() {
        let matches = transitions
            .iter()
            .enumerate()
            .filter(|(_, (from, _))| from == status)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(PrismError::new(
                "PP6101",
                "promotion evidence is branching or out of order",
            ));
        }
        let (_, to) = transitions.remove(matches[0]);
        status = match (status, to.as_str()) {
            ("development", "candidate") => "candidate",
            ("candidate", "accepted") => "accepted",
            _ => {
                return Err(PrismError::new(
                    "PP6101",
                    "promotion evidence contains an invalid transition",
                ));
            }
        };
    }
    Ok(status)
}

fn verified_release(
    store: &Store,
    root_digest: &str,
    required: &[&str],
    policy: &str,
) -> Result<VerifiedRoot, PrismError> {
    let rows = index_rows(store)?;
    let roots = rows
        .iter()
        .filter(|descriptor| descriptor.digest == root_digest)
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err(PrismError::new(
            "PP6101",
            "verified OCI root is absent or duplicated in the index",
        ));
    }
    let root = roots[0].clone();
    if root.artifact_type.as_deref() != Some(PRISM_RELEASE) {
        return Err(PrismError::new(
            "PP6101",
            "OCI root is not a Prism product release",
        ));
    }
    verify_graph(store, root_digest)?;
    let value = manifest(store, &root)?;
    if value.get("subject").is_some() || value["artifactType"] != PRISM_RELEASE {
        return Err(PrismError::new(
            "PP6101",
            "Prism product root cannot itself be a referrer",
        ));
    }
    let config: Descriptor = serde_json::from_value(value["config"].clone())
        .map_err(|error| PrismError::new("PP6101", format!("release config: {error}")))?;
    if config.media_type != PRISM_RELEASE {
        return Err(PrismError::new(
            "PP6101",
            "Prism product root has the wrong configuration media type",
        ));
    }
    let release = CanonicalDocument::parse("prismpm/product-release/1", &store.read(&config)?)?;
    let declared = release.value()["artifacts"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "product release artifacts are absent"))?;
    let layers = value["layers"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "product release layers are absent"))?;
    if declared.len() != layers.len() {
        return Err(PrismError::new(
            "PP6101",
            "product release declaration does not close over its layers",
        ));
    }
    let mut titles = BTreeSet::new();
    let mut digests = BTreeSet::new();
    let mut roles = BTreeMap::new();
    for layer in layers {
        let layer: Descriptor = serde_json::from_value(layer.clone())
            .map_err(|error| PrismError::new("PP6101", format!("release layer: {error}")))?;
        let declaration = declared
            .iter()
            .find(|candidate| candidate["digest"] == layer.digest)
            .ok_or_else(|| {
                PrismError::new("PP6101", "release layer is absent from its declaration")
            })?;
        let title = layer
            .annotations
            .as_ref()
            .and_then(|rows| rows.get("org.opencontainers.image.title"))
            .ok_or_else(|| PrismError::new("PP6101", "release layer title is absent"))?;
        let role = layer
            .annotations
            .as_ref()
            .and_then(|rows| rows.get("org.prismpm.role"))
            .ok_or_else(|| PrismError::new("PP6101", "release layer role is absent"))?;
        if !matches!(
            role.as_str(),
            "release-artifact" | "sdk-lock" | "standards-lock"
        ) || declaration["role"] != role.as_str()
            || declaration["digest"] != layer.digest
            || declaration["size"] != layer.size
            || declaration["media_type"] != layer.media_type
            || declaration["annotations"]
                != serde_json::to_value(&layer.annotations)
                    .map_err(|error| PrismError::new("PP9001", error.to_string()))?
            || !titles.insert(title.clone())
            || !digests.insert(layer.digest.clone())
        {
            return Err(PrismError::new(
                "PP6101",
                "release artifact role, descriptor, or uniqueness check failed",
            ));
        }
        if role != "release-artifact" && roles.insert(role.clone(), layer.clone()).is_some() {
            return Err(PrismError::new(
                "PP6101",
                "release contains a duplicate lock role",
            ));
        }
    }
    if roles.get("sdk-lock").map(|row| row.digest.as_str()) != release.value()["sdk_lock"].as_str()
        || roles.get("standards-lock").map(|row| row.digest.as_str())
            != release.value()["standards_lock"].as_str()
    {
        return Err(PrismError::new(
            "PP6101",
            "release lock layers do not match the declared lock digests",
        ));
    }
    let sdk_lock = CanonicalDocument::parse(
        "prismpm/sdk-lock/1",
        &store.read(roles.get("sdk-lock").expect("checked SDK lock layer"))?,
    )?;
    let standards_lock = CanonicalDocument::parse(
        "prismpm/standards-lock/1",
        &store.read(
            roles
                .get("standards-lock")
                .expect("checked standards lock layer"),
        )?,
    )?;
    let sdk_image_digest = sdk_lock.value()["sdk_image"]
        .as_str()
        .and_then(|value| value.rsplit_once('@'))
        .map(|(_, digest)| digest);
    if sdk_image_digest != release.value()["sdk_digest"].as_str()
        || sdk_lock.value()["standards_lock"] != standards_lock.digest()
    {
        return Err(PrismError::new(
            "PP6101",
            "release lock contents do not match the declared SDK and standards identities",
        ));
    }
    let referrers = indexed_referrers(store, root_digest)?;
    let mut singleton_roles = BTreeSet::new();
    let mut observed_roles = BTreeSet::new();
    for descriptor in &referrers {
        let role = validate_referrer(store, &root, descriptor)?;
        observed_roles.insert(role.clone());
        if matches!(
            role.as_str(),
            INTOTO
                | PRISM_VALIDATION
                | SPDX
                | PRISM_SUPPLY_CHAIN
                | PRISM_PRODUCTION_ACCEPTANCE
                | PRISM_PROMOTION_POLICY
        ) && !singleton_roles.insert(role)
        {
            return Err(PrismError::new(
                "PP6101",
                "OCI release contains a duplicate singleton evidence role",
            ));
        }
    }
    for expected in required {
        if !observed_roles.contains(*expected) {
            return Err(PrismError::new(
                "PP6101",
                format!("OCI release is missing required {expected} evidence"),
            ));
        }
    }
    let mut referrers = referrers;
    referrers.sort_by(|left, right| left.digest.cmp(&right.digest));
    let graph_digest = sha(&encode_value(&json!({"referrers":referrers,"root":root}))?);
    let state = VerifiedRoot {
        schema: VERIFIED_SCHEMA.to_owned(),
        policy: policy.to_owned(),
        root,
        graph_digest,
        referrers,
    };
    promotion_status(store, &state)?;
    Ok(state)
}

fn marker_path(store: &Store, digest: &str) -> Result<PathBuf, PrismError> {
    Ok(store
        .root
        .join("verified")
        .join(format!("{}.json", digest_hex(digest)?)))
}

fn write_verified_marker(store: &Store, state: VerifiedRoot) -> Result<(), PrismError> {
    let path = marker_path(store, &state.root.digest)?;
    std::fs::create_dir_all(path.parent().expect("verified marker parent"))
        .map_err(|error| PrismError::new("PP6101", format!("verified state: {error}")))?;
    atomic_replace(
        &path,
        &encode_value(
            &serde_json::to_value(state)
                .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
        )?,
    )
}

fn require_verified(
    store: &Store,
    root_digest: &str,
    release_policy: bool,
) -> Result<VerifiedRoot, PrismError> {
    let path = marker_path(store, root_digest)?;
    let bytes = std::fs::read(&path)
        .map_err(|_| PrismError::new("PP6101", "OCI root has no verified publication state"))?;
    let stored: VerifiedRoot = serde_json::from_slice(&bytes)
        .map_err(|error| PrismError::new("PP6101", format!("verified state: {error}")))?;
    if encode_value(
        &serde_json::to_value(&stored)
            .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
    )? != bytes
        || stored.schema != VERIFIED_SCHEMA
        || stored.root.digest != root_digest
        || (release_policy && stored.policy != "release")
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI verified publication state is invalid",
        ));
    }
    let required = if stored.policy == "release" {
        &RELEASE_REFERRERS[..]
    } else if stored.policy == "core" {
        &CORE_REFERRERS[..]
    } else {
        return Err(PrismError::new(
            "PP6101",
            "OCI verification policy is unknown",
        ));
    };
    let observed = verified_release(store, root_digest, required, &stored.policy)?;
    if stored != observed {
        return Err(PrismError::new(
            "PP6101",
            "OCI graph changed after verified publication",
        ));
    }
    Ok(observed)
}

fn publish_staged_layout(
    project: &Path,
    staging: &Store,
    root_digest: &str,
    reference: &str,
) -> Result<(), PrismError> {
    let output = prism_output(project)?;
    let lock_path = output.join("oci.publish.lock");
    if std::fs::symlink_metadata(&lock_path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI publication lock is a symlink",
        ));
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|error| PrismError::new("PP6101", format!("OCI publication lock: {error}")))?;
    lock.lock_exclusive()
        .map_err(|error| PrismError::new("PP6101", format!("OCI publication lock: {error}")))?;

    let destination = Store::open(project)?;
    let staged_rows = index_rows(staging)?;
    let mut rows = if destination.root.join("index.json").is_file() {
        index_rows(&destination)?
    } else {
        Vec::new()
    };
    if rows
        .iter()
        .any(|descriptor| descriptor.digest == root_digest)
    {
        let existing = require_verified(&destination, root_digest, true)?;
        let incoming = require_verified(staging, root_digest, true)?;
        let existing_referrers = existing
            .referrers
            .iter()
            .map(|descriptor| descriptor.digest.as_str())
            .collect::<BTreeSet<_>>();
        let incoming_referrers = incoming
            .referrers
            .iter()
            .map(|descriptor| descriptor.digest.as_str())
            .collect::<BTreeSet<_>>();
        if !descriptor_same_edge(&existing.root, &incoming.root)
            || existing_referrers != incoming_referrers
        {
            return Err(PrismError::new(
                "PP6101",
                "OCI publication conflicts with an existing release digest",
            ));
        }
        let existing_reference = existing
            .root
            .annotations
            .as_ref()
            .and_then(|values| values.get("org.opencontainers.image.ref.name"));
        if !reference.contains('@') && existing_reference.map(String::as_str) != Some(reference) {
            return Err(PrismError::new(
                "PP6101",
                "an existing OCI digest cannot be silently retagged",
            ));
        }
        return Ok(());
    }
    for descriptor in &rows {
        if descriptor
            .annotations
            .as_ref()
            .and_then(|values| values.get("org.opencontainers.image.ref.name"))
            .is_some_and(|value| value == reference && descriptor.digest != root_digest)
        {
            return Err(PrismError::new(
                "PP6101",
                "OCI publication lost a mutable tag race",
            ));
        }
    }
    for descriptor in staged_rows {
        if let Some(previous) = rows
            .iter()
            .find(|candidate| candidate.digest == descriptor.digest)
        {
            if !descriptor_same_edge(previous, &descriptor) {
                return Err(PrismError::new(
                    "PP6101",
                    "OCI publication changed metadata for an existing digest",
                ));
            }
        } else {
            rows.push(descriptor);
        }
    }
    rows.sort_by(|left, right| left.digest.cmp(&right.digest));

    for entry in std::fs::read_dir(staging.root.join("blobs/sha256"))
        .map_err(|error| PrismError::new("PP6101", format!("OCI staging blobs: {error}")))?
    {
        let entry = entry.map_err(|error| PrismError::new("PP6101", error.to_string()))?;
        let metadata = std::fs::symlink_metadata(entry.path())
            .map_err(|error| PrismError::new("PP6101", error.to_string()))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(PrismError::new(
                "PP6101",
                "OCI staging contains a non-regular blob",
            ));
        }
        let bytes = std::fs::read(entry.path())
            .map_err(|error| PrismError::new("PP6101", error.to_string()))?;
        let descriptor = destination.put("application/octet-stream", &bytes)?;
        if entry.file_name().to_string_lossy() != digest_hex(&descriptor.digest)? {
            return Err(PrismError::new(
                "PP6101",
                "OCI staging blob name disagrees with its digest",
            ));
        }
    }

    let staged_marker = std::fs::read(marker_path(staging, root_digest)?)
        .map_err(|error| PrismError::new("PP6101", format!("verified state: {error}")))?;
    let destination_marker = marker_path(&destination, root_digest)?;
    std::fs::create_dir_all(destination_marker.parent().expect("verified marker parent"))
        .map_err(|error| PrismError::new("PP6101", error.to_string()))?;
    atomic_replace(&destination_marker, &staged_marker)?;
    let index = encode_value(&json!({
        "manifests": rows,
        "mediaType": OCI_INDEX,
        "schemaVersion": 2
    }))?;
    if let Err(error) = atomic_replace(&destination.root.join("index.json"), &index) {
        let _ = std::fs::remove_file(&destination_marker);
        return Err(error);
    }
    if let Err(error) = require_verified(&destination, root_digest, true) {
        let _ = std::fs::remove_file(&destination_marker);
        return Err(error);
    }
    Ok(())
}

/// Inspect one immutable local release without executing it.
pub fn inspect(root: &Path, reference: &str) -> Result<Value, PrismError> {
    let digest = validate_reference(reference, true)?;
    let store = Store::open(root)?;
    let state = require_verified(&store, digest, true)?;
    let mut result = verify_graph(&store, digest)?;
    let referrers = state.referrers;
    result.as_object_mut().expect("inspection object").insert(
        "referrers".to_owned(),
        serde_json::to_value(referrers)
            .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
    );
    Ok(result)
}

/// Return the canonical root-manifest bytes for an already verified release.
pub fn root_manifest(root: &Path, root_digest: &str) -> Result<Vec<u8>, PrismError> {
    let store = Store::open(root)?;
    verified_root_manifest(&store, root_digest)
}

pub(crate) fn verified_root_manifest(
    store: &Store,
    root_digest: &str,
) -> Result<Vec<u8>, PrismError> {
    require_verified(store, root_digest, true)?;
    let descriptor = store.descriptor(root_digest, OCI_MANIFEST)?;
    let bytes = store.read(&descriptor)?;
    if sha(&bytes) != root_digest {
        return Err(PrismError::new(
            "PP6101",
            "OCI root-manifest bytes disagree with the requested digest",
        ));
    }
    Ok(bytes)
}

/// Read the one verified JSON evidence document for a singleton artifact type.
pub fn singleton_referrer_evidence(
    root: &Path,
    root_digest: &str,
    artifact_type: &str,
) -> Result<Value, PrismError> {
    let store = Store::open(root)?;
    let state = require_verified(&store, root_digest, true)?;
    let matches = state
        .referrers
        .iter()
        .filter(|descriptor| descriptor.artifact_type.as_deref() == Some(artifact_type))
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(PrismError::new(
            "PP6101",
            format!("OCI release requires exactly one {artifact_type} referrer"),
        ));
    }
    referrer_evidence(&store, matches[0])
}

pub(crate) fn verified_referrer_evidence_rows(
    store: &Store,
    root_digest: &str,
    artifact_type: &str,
) -> Result<Vec<Value>, PrismError> {
    let state = require_verified(store, root_digest, true)?;
    state
        .referrers
        .iter()
        .filter(|descriptor| descriptor.artifact_type.as_deref() == Some(artifact_type))
        .map(|descriptor| referrer_evidence(store, descriptor))
        .collect()
}

pub(crate) fn verified_singleton_referrer_evidence(
    store: &Store,
    root_digest: &str,
    artifact_type: &str,
) -> Result<Value, PrismError> {
    let mut matches = verified_referrer_evidence_rows(store, root_digest, artifact_type)?;
    if matches.len() != 1 {
        return Err(PrismError::new(
            "PP6101",
            format!("OCI release requires exactly one {artifact_type} referrer"),
        ));
    }
    Ok(matches.remove(0))
}

/// Return a canonical, sorted closure over every deployment-evidence referrer
/// currently attached to an exact release.
pub fn deployment_evidence_closure(root: &Path, root_digest: &str) -> Result<Vec<u8>, PrismError> {
    let store = Store::open(root)?;
    verified_deployment_evidence_closure(&store, root_digest)
}

pub(crate) fn verified_deployment_evidence_closure(
    store: &Store,
    root_digest: &str,
) -> Result<Vec<u8>, PrismError> {
    let state = require_verified(store, root_digest, true)?;
    let mut evidence_rows = Vec::new();
    for descriptor in state
        .referrers
        .iter()
        .filter(|descriptor| descriptor.artifact_type.as_deref() == Some(PRISM_DEPLOYMENT_EVIDENCE))
    {
        let evidence = referrer_evidence(store, descriptor)?;
        evidence_rows.push(json!({
            "digest":evidence["evidence_digest"],
            "operation":evidence["operation"],
            "referrer":descriptor.digest,
            "target":evidence["target"]
        }));
    }
    evidence_rows.sort_by(|left, right| left["referrer"].as_str().cmp(&right["referrer"].as_str()));
    if evidence_rows.is_empty()
        || evidence_rows
            .windows(2)
            .any(|rows| rows[0]["referrer"] == rows[1]["referrer"])
    {
        return Err(PrismError::new(
            "PP7401",
            "release has no unambiguous deployment evidence to sign",
        ));
    }
    encode_value(&json!({
        "evidence":evidence_rows,
        "release_digest":root_digest,
        "schema":"prismpm/deployment-evidence-closure/1"
    }))
}

/// Require one verified Sigstore signature whose signed closure is exactly the
/// complete set of deployment evidence currently attached to this release.
/// Adding a later deploy, verify, rollback, restore, or destroy observation
/// therefore requires a new closure signature before acceptance.
pub fn require_complete_signed_deployment_evidence(
    root: &Path,
    root_digest: &str,
) -> Result<Value, PrismError> {
    let closure_bytes = deployment_evidence_closure(root, root_digest)?;
    let closure: Value = serde_json::from_slice(&closure_bytes).map_err(|error| {
        PrismError::new("PP7401", format!("deployment evidence closure: {error}"))
    })?;
    let closure_digest = sha(&closure_bytes);
    let store = Store::open(root)?;
    let state = require_verified(&store, root_digest, true)?;
    let matches = state
        .referrers
        .iter()
        .filter(|descriptor| descriptor.artifact_type.as_deref() == Some(PRISM_EVIDENCE_SIGNATURE))
        .filter_map(|descriptor| referrer_evidence(&store, descriptor).ok())
        .filter(|document| {
            document["verified"] == true
                && document["subject"].as_str() == Some(root_digest)
                && document["evidence_closure"] == closure
                && document["evidence_closure_digest"].as_str() == Some(closure_digest.as_str())
                && document["signature"]["signed_blob_digest"].as_str()
                    == Some(closure_digest.as_str())
                && document["signature"]["verified"] == true
                && document["signature"]["transparency_log_verified"] == true
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(PrismError::new(
            "PP7401",
            "accepted promotion requires exactly one verified signature over the complete current deployment-evidence closure",
        ));
    }
    Ok(json!({
        "evidence_closure_digest":closure_digest,
        "evidence_count":closure["evidence"].as_array().map(Vec::len).unwrap_or(0),
        "release_digest":root_digest,
        "schema":"prismpm/evidence-signature-coverage/1",
        "verified":true
    }))
}

/// Return the status obtained from the verified, linear promotion chain.
pub fn verified_promotion_status(root: &Path, root_digest: &str) -> Result<String, PrismError> {
    let store = Store::open(root)?;
    verified_store_promotion_status(&store, root_digest)
}

pub(crate) fn verified_store_promotion_status(
    store: &Store,
    root_digest: &str,
) -> Result<String, PrismError> {
    let state = require_verified(store, root_digest, true)?;
    Ok(promotion_status(store, &state)?.to_owned())
}

/// Read one named release layer after verifying the complete root graph.
pub fn artifact(root: &Path, root_digest: &str, title: &str) -> Result<Vec<u8>, PrismError> {
    let store = Store::open(root)?;
    require_verified(&store, root_digest, true)?;
    let descriptor = store.descriptor(root_digest, OCI_MANIFEST)?;
    let manifest: Value = serde_json::from_slice(&store.read(&descriptor)?)
        .map_err(|error| PrismError::new("PP6101", format!("OCI root: {error}")))?;
    let layers = manifest["layers"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "OCI root has no layers"))?;
    let row = layers
        .iter()
        .find(|row| {
            row.pointer("/annotations/org.opencontainers.image.title")
                == Some(&Value::String(title.to_owned()))
        })
        .ok_or_else(|| PrismError::new("PP6101", format!("release artifact is absent: {title}")))?;
    let descriptor: Descriptor = serde_json::from_value(row.clone())
        .map_err(|error| PrismError::new("PP6101", format!("OCI descriptor: {error}")))?;
    store.read(&descriptor)
}

/// Return the verified root layer descriptors in canonical manifest order.
pub fn layers(root: &Path, root_digest: &str) -> Result<Vec<Descriptor>, PrismError> {
    let store = Store::open(root)?;
    require_verified(&store, root_digest, false)?;
    let descriptor = store.descriptor(root_digest, OCI_MANIFEST)?;
    let manifest: Value = serde_json::from_slice(&store.read(&descriptor)?)
        .map_err(|error| PrismError::new("PP6101", format!("OCI root: {error}")))?;
    manifest["layers"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6101", "OCI root has no layers"))?
        .iter()
        .map(|row| {
            serde_json::from_value(row.clone())
                .map_err(|error| PrismError::new("PP6101", format!("OCI descriptor: {error}")))
        })
        .collect()
}

/// Read a verified blob descriptor from the local OCI store.
pub fn read_descriptor(root: &Path, descriptor: &Descriptor) -> Result<Vec<u8>, PrismError> {
    Store::open(root)?.read(descriptor)
}

/// Attach canonical evidence as a discoverable OCI 1.1 subject referrer.
fn attach_referrer_internal(
    root: &Path,
    subject_digest: &str,
    artifact_type: &str,
    evidence: &[u8],
    cryptographically_verified: bool,
) -> Result<String, PrismError> {
    let store = Store::open(root)?;
    validate_media_type(artifact_type)?;
    let lock_path = prism_output(root)?.join("oci.publish.lock");
    if std::fs::symlink_metadata(&lock_path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI publication lock is a symlink",
        ));
    }
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|error| PrismError::new("PP6101", format!("OCI publication lock: {error}")))?;
    lock.lock_exclusive()
        .map_err(|error| PrismError::new("PP6101", format!("OCI publication lock: {error}")))?;
    let previous = require_verified(&store, subject_digest, false)?;
    if matches!(
        artifact_type,
        PRISM_PROMOTION | PRISM_PROMOTION_POLICY | COSIGN_SIGNATURE | PRISM_EVIDENCE_SIGNATURE
    ) && !cryptographically_verified
    {
        return Err(PrismError::new(
            "PP7401",
            "signature and promotion-policy referrers require verified supply-chain construction",
        ));
    }
    if artifact_type == PRISM_PROMOTION {
        let document: Value = serde_json::from_slice(evidence)
            .map_err(|error| PrismError::new("PP6101", format!("promotion evidence: {error}")))?;
        if document["from"].as_str() != Some(promotion_status(&store, &previous)?) {
            return Err(PrismError::new(
                "PP6101",
                "promotion does not continue the subject's accepted status chain",
            ));
        }
    }
    let mut subject = store.descriptor(subject_digest, OCI_MANIFEST)?;
    subject.artifact_type = Some(PRISM_RELEASE.to_owned());
    let empty = store.put(OCI_EMPTY, b"{}")?;
    let layer = store.put(artifact_type, evidence)?;
    let referrer = oci_manifest(&store, artifact_type, empty, vec![layer], Some(subject))?;
    validate_referrer(&store, &previous.root, &referrer)?;
    if previous
        .referrers
        .iter()
        .any(|descriptor| descriptor.digest == referrer.digest)
    {
        return Ok(referrer.digest);
    }
    if matches!(
        artifact_type,
        INTOTO
            | PRISM_VALIDATION
            | SPDX
            | PRISM_SUPPLY_CHAIN
            | PRISM_PRODUCTION_ACCEPTANCE
            | PRISM_PROMOTION_POLICY
            | COSIGN_SIGNATURE
    ) && previous
        .referrers
        .iter()
        .any(|descriptor| descriptor.artifact_type.as_deref() == Some(artifact_type))
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI release already has evidence for this singleton role",
        ));
    }
    let index_path = store.root.join("index.json");
    let mut index: Value = serde_json::from_slice(
        &std::fs::read(&index_path)
            .map_err(|error| PrismError::new("PP6101", format!("OCI index: {error}")))?,
    )
    .map_err(|error| PrismError::new("PP6101", format!("OCI index: {error}")))?;
    let manifests = index["manifests"]
        .as_array_mut()
        .ok_or_else(|| PrismError::new("PP6101", "OCI index has no manifest list"))?;
    if !manifests.iter().any(|row| row["digest"] == referrer.digest) {
        manifests.push(
            serde_json::to_value(&referrer)
                .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
        );
        manifests.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        atomic_replace(&index_path, &encode_value(&index)?)?;
    }
    let required = if previous.policy == "release" {
        &RELEASE_REFERRERS[..]
    } else {
        &CORE_REFERRERS[..]
    };
    write_verified_marker(
        &store,
        verified_release(&store, subject_digest, required, &previous.policy)?,
    )?;
    Ok(referrer.digest)
}

/// Attach canonical non-signature evidence as a discoverable OCI 1.1 subject referrer.
///
/// Signature and promotion roles are deliberately unavailable through this generic
/// entry point; [`crate::supply_chain`] attaches them only after real Cosign verification.
pub fn attach_referrer(
    root: &Path,
    subject_digest: &str,
    artifact_type: &str,
    evidence: &[u8],
) -> Result<String, PrismError> {
    attach_referrer_internal(root, subject_digest, artifact_type, evidence, false)
}

pub(crate) fn attach_verified_signature(
    root: &Path,
    subject_digest: &str,
    evidence: &[u8],
) -> Result<String, PrismError> {
    attach_referrer_internal(root, subject_digest, COSIGN_SIGNATURE, evidence, true)
}

pub(crate) fn attach_promotion_policy(
    root: &Path,
    subject_digest: &str,
    evidence: &[u8],
) -> Result<String, PrismError> {
    attach_referrer_internal(root, subject_digest, PRISM_PROMOTION_POLICY, evidence, true)
}

pub(crate) fn attach_verified_promotion(
    root: &Path,
    subject_digest: &str,
    evidence: &[u8],
) -> Result<String, PrismError> {
    attach_referrer_internal(root, subject_digest, PRISM_PROMOTION, evidence, true)
}

pub(crate) fn attach_verified_evidence_signature(
    root: &Path,
    subject_digest: &str,
    evidence: &[u8],
) -> Result<String, PrismError> {
    attach_referrer_internal(
        root,
        subject_digest,
        PRISM_EVIDENCE_SIGNATURE,
        evidence,
        true,
    )
}

fn registry_env() -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    if let Some(value) = std::env::var_os("DOCKER_CONFIG") {
        env.insert(
            "DOCKER_CONFIG".to_owned(),
            value.to_string_lossy().into_owned(),
        );
    }
    env
}

fn run_oras(
    root: &Path,
    arguments: Vec<String>,
) -> Result<crate::verification::ProcessRecord, PrismError> {
    let oras = crate::sdk::executable("oras")?;
    crate::verification::run_process(
        "oras",
        &oras,
        &arguments,
        root,
        &registry_env(),
        &[(root, "$PROJECT")],
        "PP6201",
    )
}

pub(crate) fn release_external_artifacts(
    store: &Store,
    root_digest: &str,
) -> Result<Vec<Value>, PrismError> {
    Ok(release_metadata(store, root_digest)?["external_artifacts"]
        .as_array()
        .expect("schema-validated external artifacts")
        .clone())
}

pub(crate) fn release_metadata(store: &Store, root_digest: &str) -> Result<Value, PrismError> {
    let root = store.descriptor(root_digest, OCI_MANIFEST)?;
    let root_manifest = manifest(store, &root)?;
    let config: Descriptor = serde_json::from_value(root_manifest["config"].clone())
        .map_err(|error| PrismError::new("PP6201", format!("release config: {error}")))?;
    if config.media_type != PRISM_RELEASE {
        return Err(PrismError::new(
            "PP6201",
            "OCI root does not contain a Prism product-release configuration",
        ));
    }
    let bytes = store.read(&config)?;
    let release = CanonicalDocument::parse("prismpm/product-release/1", &bytes)?;
    Ok(release.value().clone())
}

fn verify_remote_external_artifacts(
    root: &Path,
    store: &Store,
    root_digest: &str,
) -> Result<(), PrismError> {
    for artifact in release_external_artifacts(store, root_digest)? {
        let reference = artifact["reference"]
            .as_str()
            .expect("schema-validated external reference");
        let expected_digest = artifact["digest"]
            .as_str()
            .expect("schema-validated external digest");
        let expected_media_type = artifact["media_type"]
            .as_str()
            .expect("schema-validated external media type");
        let selected = validate_reference(reference, true)?;
        if selected != expected_digest {
            return Err(PrismError::new(
                "PP6201",
                "external OCI reference selects a different digest",
            ));
        }
        let record = run_oras(
            root,
            vec![
                "manifest".to_owned(),
                "fetch".to_owned(),
                "--descriptor".to_owned(),
                reference.to_owned(),
            ],
        )?;
        let descriptor: Value = serde_json::from_str(record.stdout.trim()).map_err(|error| {
            PrismError::new("PP6201", format!("external OCI descriptor: {error}"))
        })?;
        let observed_digest = descriptor["digest"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP6201", "external OCI descriptor digest is absent"))?;
        let observed_media_type = descriptor["mediaType"].as_str().ok_or_else(|| {
            PrismError::new("PP6201", "external OCI descriptor media type is absent")
        })?;
        let observed_size = descriptor["size"]
            .as_u64()
            .ok_or_else(|| PrismError::new("PP6201", "external OCI descriptor size is absent"))?;
        validate_media_type(observed_media_type)?;
        digest_hex(observed_digest)?;
        if observed_size > 10_737_418_240
            || observed_digest != expected_digest
            || observed_media_type != expected_media_type
        {
            return Err(PrismError::new(
                "PP6201",
                format!("external OCI descriptor changed: {reference}"),
            ));
        }
    }
    Ok(())
}

/// Push an existing verified local graph without rebuilding it.
pub fn push(root: &Path, reference: &str) -> Result<TransferResult, PrismError> {
    let digest = validate_reference(reference, true)?.to_owned();
    let store = Store::open(root)?;
    let local = require_verified(&store, &digest, true)?;
    crate::supply_chain::verify_release_transfer_in_store(root, &store, &digest)?;
    verify_remote_external_artifacts(root, &store, &digest)?;
    run_oras(
        root,
        vec![
            "copy".to_owned(),
            "--recursive".to_owned(),
            "--from-oci-layout".to_owned(),
            format!("{}@{digest}", store.root.display()),
            reference.to_owned(),
        ],
    )?;
    let descriptor = run_oras(
        root,
        vec![
            "manifest".to_owned(),
            "fetch".to_owned(),
            "--descriptor".to_owned(),
            reference.to_owned(),
        ],
    )?;
    let value: Descriptor = serde_json::from_str(descriptor.stdout.trim())
        .map_err(|error| PrismError::new("PP6201", format!("remote descriptor: {error}")))?;
    validate_descriptor(&value)?;
    if value.digest != digest || !descriptor_same_edge(&value, &local.root) {
        return Err(PrismError::new(
            "PP6201",
            "registry returned different release descriptor metadata",
        ));
    }
    let local_referrers = local
        .referrers
        .iter()
        .map(|descriptor| descriptor.digest.clone())
        .collect::<BTreeSet<_>>();
    if !local_referrers.is_empty() {
        let discovered = run_oras(
            root,
            vec![
                "discover".to_owned(),
                "--format".to_owned(),
                "json".to_owned(),
                reference.to_owned(),
            ],
        )?;
        let document: Value = serde_json::from_str(discovered.stdout.trim())
            .map_err(|error| PrismError::new("PP6201", format!("remote referrers: {error}")))?;
        let remote = document
            .get("referrers")
            .or_else(|| document.get("manifests"))
            .and_then(Value::as_array)
            .ok_or_else(|| PrismError::new("PP6201", "registry omitted referrer discovery"))?
            .iter()
            .filter_map(|row| row["digest"].as_str().map(str::to_owned))
            .collect::<BTreeSet<_>>();
        if !local_referrers.is_subset(&remote) {
            return Err(PrismError::new(
                "PP6201",
                "registry round trip omitted one or more required referrers",
            ));
        }
    }
    let output = prism_output(root)?;
    let roundtrip = tempfile::Builder::new()
        .prefix("push-roundtrip-")
        .tempdir_in(&output)
        .map_err(|error| PrismError::new("PP6201", error.to_string()))?;
    let layout = roundtrip.path().join("oci");
    run_oras(
        root,
        vec![
            "copy".to_owned(),
            "--recursive".to_owned(),
            "--to-oci-layout".to_owned(),
            reference.to_owned(),
            format!("{}:roundtrip", layout.display()),
        ],
    )?;
    let remote = Store::open_layout(layout)?;
    let remote_state = verified_release(&remote, &digest, &RELEASE_REFERRERS, "release")?;
    let remote_referrers = remote_state
        .referrers
        .iter()
        .map(|descriptor| descriptor.digest.clone())
        .collect::<BTreeSet<_>>();
    if !descriptor_same_edge(&remote_state.root, &local.root) || remote_referrers != local_referrers
    {
        return Err(PrismError::new(
            "PP6201",
            "registry round trip changed the OCI release closure",
        ));
    }
    Ok(TransferResult {
        schema: "prismpm/push-result/1".to_owned(),
        reference: reference.to_owned(),
        release_digest: digest,
        verified: true,
    })
}

/// Pull an immutable graph into a staging layout, verify it, then publish it.
pub fn pull(root: &Path, reference: &str) -> Result<TransferResult, PrismError> {
    let digest = validate_reference(reference, true)?.to_owned();
    let output = prism_output(root)?;
    let staging = tempfile::Builder::new()
        .prefix("pull-")
        .tempdir_in(&output)
        .map_err(|error| PrismError::new("PP6201", error.to_string()))?;
    let layout = staging.path().join("oci");
    run_oras(
        root,
        vec![
            "copy".to_owned(),
            "--recursive".to_owned(),
            "--to-oci-layout".to_owned(),
            reference.to_owned(),
            format!("{}:pulled", layout.display()),
        ],
    )?;
    let staging_store = Store::open_layout(layout)?;
    let state = verified_release(&staging_store, &digest, &RELEASE_REFERRERS, "release")?;
    verify_remote_external_artifacts(root, &staging_store, &digest)?;
    write_verified_marker(&staging_store, state)?;
    crate::supply_chain::verify_release_transfer_in_store(root, &staging_store, &digest)?;
    publish_staged_layout(root, &staging_store, &digest, reference)?;
    Ok(TransferResult {
        schema: "prismpm/pull-result/1".to_owned(),
        reference: reference.to_owned(),
        release_digest: digest,
        verified: true,
    })
}

fn valid_registry_authority(authority: &str) -> bool {
    let (host, port) = if let Some((host, port)) = authority.rsplit_once(':') {
        (host, Some(port))
    } else {
        (authority, None)
    };
    let valid_label = |label: &str| {
        !label.is_empty()
            && label.len() <= 63
            && label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && label
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
            && label
                .bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_alphanumeric())
    };
    (host == "localhost"
        || host.contains('.') && host.len() <= 253 && host.split('.').all(valid_label))
        && port.is_none_or(|port| {
            !port.is_empty()
                && port.bytes().all(|byte| byte.is_ascii_digit())
                && port.parse::<u16>().ok().is_some_and(|value| value != 0)
        })
}

fn valid_repository_component(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut cursor = 0;
    let consume_alphanumeric = |cursor: &mut usize| {
        let start = *cursor;
        while *cursor < bytes.len()
            && (bytes[*cursor].is_ascii_lowercase() || bytes[*cursor].is_ascii_digit())
        {
            *cursor += 1;
        }
        *cursor > start
    };
    if !consume_alphanumeric(&mut cursor) {
        return false;
    }
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'.' => cursor += 1,
            b'_' if bytes.get(cursor + 1) == Some(&b'_') => cursor += 2,
            b'_' => cursor += 1,
            b'-' => {
                while bytes.get(cursor) == Some(&b'-') {
                    cursor += 1;
                }
            }
            _ => return false,
        }
        if !consume_alphanumeric(&mut cursor) {
            return false;
        }
    }
    true
}

/// Validate a tagged build reference or an immutable operation reference.
pub fn validate_reference(reference: &str, require_digest: bool) -> Result<&str, PrismError> {
    if reference.len() > 1024
        || reference.is_empty()
        || reference
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err(PrismError::new("PP6101", "OCI reference is malformed"));
    }
    if reference.contains("\\")
        || reference.contains("://")
        || reference.contains('?')
        || reference.contains('#')
        || reference.matches('@').count() > 1
    {
        return Err(PrismError::new("PP6101", "OCI reference is malformed"));
    }
    let (name, digest) = if let Some((name, digest)) = reference.rsplit_once('@') {
        digest_hex(digest)?;
        (name, Some(digest))
    } else {
        (reference, None)
    };
    let segments = name.split('/').collect::<Vec<_>>();
    if segments.len() < 2
        || segments
            .iter()
            .any(|segment| segment.is_empty() || matches!(*segment, "." | ".."))
        || !valid_registry_authority(segments[0])
    {
        return Err(PrismError::new("PP6101", "OCI registry path is malformed"));
    }
    let last = segments.last().copied().unwrap_or_default();
    let (repository, tag) = if let Some((repository, tag)) = last.rsplit_once(':') {
        (repository, Some(tag))
    } else {
        (last, None)
    };
    if !valid_repository_component(repository)
        || segments[1..segments.len() - 1]
            .iter()
            .any(|segment| !valid_repository_component(segment))
    {
        return Err(PrismError::new(
            "PP6101",
            "OCI repository path is malformed",
        ));
    }
    if let Some(digest) = digest {
        if tag.is_some() {
            return Err(PrismError::new(
                "PP6101",
                "immutable OCI references cannot also contain a mutable tag",
            ));
        }
        return Ok(digest);
    }
    if require_digest {
        return Err(PrismError::new(
            "PP6101",
            "operation requires an immutable digest reference",
        ));
    }
    let tag = tag.ok_or_else(|| {
        PrismError::new("PP6101", "build reference requires a registry path and tag")
    })?;
    if tag.len() > 128
        || !tag
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        || !tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
    {
        return Err(PrismError::new("PP6101", "OCI tag is malformed"));
    }
    Ok(reference)
}

#[cfg(test)]
mod tests {
    use super::{
        attach_referrer, index_rows, marker_path, oci_manifest, publish_staged_layout,
        require_verified, sha, verified_release, verify_graph, write_verified_marker, Descriptor,
        Store, CORE_REFERRERS, COSIGN_SIGNATURE, INTOTO, OCI_EMPTY, OCI_INDEX, OCI_MANIFEST,
        PRISM_PROMOTION, PRISM_PROMOTION_POLICY, PRISM_RELEASE, PRISM_SUPPLY_CHAIN,
        PRISM_VALIDATION, RELEASE_REFERRERS, SPDX,
    };
    use crate::contracts::CanonicalDocument;
    use crate::holo::canonical::encode_value;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    fn fixture(project: &std::path::Path, reference: &str, payload: &[u8]) -> (Store, Descriptor) {
        let store = Store::open(project).unwrap();
        let mut layer = store.put("application/octet-stream", payload).unwrap();
        layer.annotations = Some(BTreeMap::from([
            ("org.opencontainers.image.title".into(), "app.bin".into()),
            ("org.prismpm.role".into(), "release-artifact".into()),
        ]));
        let standards_lock_bytes = include_bytes!("../standards.lock");
        CanonicalDocument::parse("prismpm/standards-lock/1", standards_lock_bytes).unwrap();
        let sdk_lock_document = CanonicalDocument::from_value(
            "prismpm/sdk-lock/1",
            json!({
                "inventory":[{
                    "digest":format!("sha256:{}", "1".repeat(64)),
                    "id":"prismpm",
                    "kind":"binary",
                    "version":"0.3.0"
                }],
                "schema":"prismpm/sdk-lock/1",
                "sdk_image":format!("example.test/prismpm-sdk@sha256:{}", "2".repeat(64)),
                "sdk_version":"0.3.0",
                "standards_lock":sha(standards_lock_bytes)
            }),
        )
        .unwrap();
        let mut sdk_lock = store
            .put("application/json", sdk_lock_document.bytes())
            .unwrap();
        sdk_lock.annotations = Some(BTreeMap::from([
            (
                "org.opencontainers.image.title".into(),
                "prismpm.lock".into(),
            ),
            ("org.prismpm.role".into(), "sdk-lock".into()),
        ]));
        let mut standards_lock = store.put("application/json", standards_lock_bytes).unwrap();
        standards_lock.annotations = Some(BTreeMap::from([
            (
                "org.opencontainers.image.title".into(),
                "standards.lock".into(),
            ),
            ("org.prismpm.role".into(), "standards-lock".into()),
        ]));
        let mut artifacts = [
            (&layer, "release-artifact"),
            (&sdk_lock, "sdk-lock"),
            (&standards_lock, "standards-lock"),
        ]
        .into_iter()
        .map(|(descriptor, role)| {
            json!({
                "annotations":descriptor.annotations,
                "digest":descriptor.digest,
                "media_type":descriptor.media_type,
                "role":role,
                "size":descriptor.size
            })
        })
        .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        let release = CanonicalDocument::from_value(
            "prismpm/product-release/1",
            json!({
                "artifacts":artifacts,
                "external_artifacts":[],
                "model_digest":format!("sha256:{}", "1".repeat(64)),
                "product":"fixture",
                "release":"1",
                "schema":"prismpm/product-release/1",
                "sdk_digest":format!("sha256:{}", "2".repeat(64)),
                "sdk_lock":sdk_lock.digest,
                "standards_lock":standards_lock.digest,
                "status":"development"
            }),
        )
        .unwrap();
        let config = store.put(PRISM_RELEASE, release.bytes()).unwrap();
        let root = oci_manifest(
            &store,
            PRISM_RELEASE,
            config,
            vec![layer, sdk_lock, standards_lock],
            None,
        )
        .unwrap();
        let empty = store.put(OCI_EMPTY, b"{}").unwrap();
        let mut rows = vec![serde_json::to_value({
            let mut root = root.clone();
            root.annotations = Some(BTreeMap::from([(
                "org.opencontainers.image.ref.name".into(),
                reference.into(),
            )]));
            root
        })
        .unwrap()];
        for (role, evidence) in [
            (
                INTOTO,
                encode_value(&json!({"subject":root.digest})).unwrap(),
            ),
            (
                PRISM_VALIDATION,
                encode_value(&json!({"subject":root.digest})).unwrap(),
            ),
            (
                SPDX,
                encode_value(&json!({"release_digest":root.digest})).unwrap(),
            ),
            (
                PRISM_SUPPLY_CHAIN,
                encode_value(&json!({"release_digest":root.digest})).unwrap(),
            ),
        ] {
            let evidence = store.put(role, &evidence).unwrap();
            let referrer = oci_manifest(
                &store,
                role,
                empty.clone(),
                vec![evidence],
                Some(root.clone()),
            )
            .unwrap();
            rows.push(serde_json::to_value(referrer).unwrap());
        }
        rows.sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        super::atomic_replace(
            &store.root.join("index.json"),
            &encode_value(&json!({
                "manifests":rows,
                "mediaType":OCI_INDEX,
                "schemaVersion":2
            }))
            .unwrap(),
        )
        .unwrap();
        let state = verified_release(&store, &root.digest, &RELEASE_REFERRERS, "release").unwrap();
        write_verified_marker(&store, state).unwrap();
        (store, root)
    }

    #[test]
    fn store_rejects_dangling_graphs_and_mutable_references() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(directory.path()).unwrap();
        let config = store.put("application/json", b"{}").unwrap();
        let missing = super::Descriptor {
            digest: format!("sha256:{}", "0".repeat(64)),
            media_type: "application/octet-stream".into(),
            size: 1,
            artifact_type: None,
            annotations: None,
        };
        let bytes = encode_value(&json!({
            "artifactType":"test","config":config,"layers":[missing],
            "mediaType":OCI_MANIFEST,"schemaVersion":2
        }))
        .unwrap();
        let root = store.put(OCI_MANIFEST, &bytes).unwrap();
        assert!(verify_graph(&store, &root.digest).is_err());
        assert!(super::validate_reference("example.invalid/a:b", true).is_err());
        assert!(super::validate_reference("../escape:tag", false).is_err());
        assert!(super::validate_reference("ghcr.io/uor/../escape:tag", false).is_err());
        assert!(super::validate_reference("ghcr.io/UOR/app:tag", false).is_err());
        assert!(super::validate_reference("ghcr.io/uor/app/../escape:tag", false).is_err());
        assert!(super::validate_reference("ghcr.io/uor/app:bad/tag", false).is_err());
        assert!(super::validate_reference("ghcr.io:port/uor/app:tag", false).is_err());
        assert!(super::validate_reference("ghcr.io/uor/app:tag", false).is_ok());
        assert!(super::validate_reference("localhost:5000/uor/app:RC_1", false).is_ok());
        let immutable = format!("ghcr.io/uor/app@sha256:{}", "a".repeat(64));
        assert_eq!(
            super::validate_reference(&immutable, true).unwrap(),
            format!("sha256:{}", "a".repeat(64))
        );

        #[cfg(unix)]
        {
            let escaped = tempfile::tempdir().unwrap();
            let project = tempfile::tempdir().unwrap();
            std::os::unix::fs::symlink(escaped.path(), project.path().join(".prism")).unwrap();
            assert_eq!(Store::open(project.path()).unwrap_err().code, "PP8001");
        }
    }

    #[test]
    fn verified_release_closes_roles_subjects_and_marker_state() {
        let directory = tempfile::tempdir().unwrap();
        let (store, root) = fixture(directory.path(), "example.invalid/app:v1", b"one");
        let state = require_verified(&store, &root.digest, true).unwrap();
        assert_eq!(state.referrers.len(), 4);
        assert_eq!(index_rows(&store).unwrap().len(), 5);

        crate::supply_chain::verify_release_transfer_in_store(
            directory.path(),
            &store,
            &root.digest,
        )
        .unwrap();

        let trust =
            crate::supply_chain::verify_release_trust(directory.path(), &root.digest).unwrap_err();
        assert_eq!(trust.code, "PP6101");
        assert!(trust.message.contains(PRISM_PROMOTION_POLICY));

        let mut marker: serde_json::Value = serde_json::from_slice(
            &std::fs::read(marker_path(&store, &root.digest).unwrap()).unwrap(),
        )
        .unwrap();
        marker["graph_digest"] = json!(format!("sha256:{}", "0".repeat(64)));
        super::atomic_replace(
            &marker_path(&store, &root.digest).unwrap(),
            &encode_value(&marker).unwrap(),
        )
        .unwrap();
        assert!(require_verified(&store, &root.digest, true).is_err());
    }

    #[test]
    fn referrer_confused_subject_and_duplicate_role_are_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let (store, root) = fixture(directory.path(), "example.invalid/app:v1", b"one");
        let rows = index_rows(&store).unwrap();
        let validation = rows
            .iter()
            .find(|row| row.artifact_type.as_deref() == Some(PRISM_VALIDATION))
            .unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&store.read(validation).unwrap()).unwrap();
        value["subject"]["size"] = json!(root.size + 1);
        let mut confused = store
            .put(OCI_MANIFEST, &encode_value(&value).unwrap())
            .unwrap();
        confused.artifact_type = Some(PRISM_VALIDATION.into());
        let mut index: serde_json::Value =
            serde_json::from_slice(&std::fs::read(store.root.join("index.json")).unwrap()).unwrap();
        index["manifests"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(confused).unwrap());
        index["manifests"]
            .as_array_mut()
            .unwrap()
            .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        super::atomic_replace(
            &store.root.join("index.json"),
            &encode_value(&index).unwrap(),
        )
        .unwrap();
        assert!(verified_release(&store, &root.digest, &CORE_REFERRERS, "core").is_err());

        let duplicate_directory = tempfile::tempdir().unwrap();
        let (duplicate_store, duplicate_root) = fixture(
            duplicate_directory.path(),
            "example.invalid/app:v1",
            b"duplicate",
        );
        let empty = duplicate_store.put(OCI_EMPTY, b"{}").unwrap();
        let evidence = duplicate_store
            .put(
                PRISM_VALIDATION,
                &encode_value(&json!({
                    "detail":"second",
                    "subject":duplicate_root.digest
                }))
                .unwrap(),
            )
            .unwrap();
        let extra = oci_manifest(
            &duplicate_store,
            PRISM_VALIDATION,
            empty,
            vec![evidence],
            Some(duplicate_root.clone()),
        )
        .unwrap();
        let mut index: serde_json::Value = serde_json::from_slice(
            &std::fs::read(duplicate_store.root.join("index.json")).unwrap(),
        )
        .unwrap();
        index["manifests"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(extra).unwrap());
        index["manifests"]
            .as_array_mut()
            .unwrap()
            .sort_by(|left, right| left["digest"].as_str().cmp(&right["digest"].as_str()));
        super::atomic_replace(
            &duplicate_store.root.join("index.json"),
            &encode_value(&index).unwrap(),
        )
        .unwrap();
        assert!(verified_release(
            &duplicate_store,
            &duplicate_root.digest,
            &RELEASE_REFERRERS,
            "release"
        )
        .is_err());
    }

    #[test]
    fn duplicate_edges_and_noncanonical_manifests_fail_closed() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(directory.path()).unwrap();
        let config = store.put("application/json", b"{}").unwrap();
        let layer = store.put("application/octet-stream", b"x").unwrap();
        let duplicate = encode_value(&json!({
            "artifactType":"application/example",
            "config":config,
            "layers":[layer,layer],
            "mediaType":OCI_MANIFEST,
            "schemaVersion":2
        }))
        .unwrap();
        let root = store.put(OCI_MANIFEST, &duplicate).unwrap();
        assert!(verify_graph(&store, &root.digest).is_err());

        let noncanonical = b"{ \"artifactType\":\"application/example\",\"config\":{\"digest\":\"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"mediaType\":\"application/json\",\"size\":0},\"layers\":[],\"mediaType\":\"application/vnd.oci.image.manifest.v1+json\",\"schemaVersion\":2}";
        let root = store.put(OCI_MANIFEST, noncanonical).unwrap();
        assert!(verify_graph(&store, &root.digest).is_err());
    }

    #[test]
    fn staged_publication_is_atomic_and_rejects_tag_races() {
        let destination = tempfile::tempdir().unwrap();
        let staging_one = tempfile::tempdir().unwrap();
        let (one, root_one) = fixture(staging_one.path(), "example.invalid/app:v1", b"one");
        publish_staged_layout(
            destination.path(),
            &one,
            &root_one.digest,
            "example.invalid/app:v1",
        )
        .unwrap();
        let live = Store::open(destination.path()).unwrap();
        require_verified(&live, &root_one.digest, true).unwrap();

        let staging_two = tempfile::tempdir().unwrap();
        let (two, root_two) = fixture(staging_two.path(), "example.invalid/app:v1", b"two");
        assert!(publish_staged_layout(
            destination.path(),
            &two,
            &root_two.digest,
            "example.invalid/app:v1",
        )
        .is_err());
        require_verified(&live, &root_one.digest, true).unwrap();
        assert!(!marker_path(&live, &root_two.digest).unwrap().exists());

        let incomplete = tempfile::tempdir().unwrap();
        let (incomplete_store, incomplete_root) =
            fixture(incomplete.path(), "example.invalid/app:v2", b"incomplete");
        std::fs::remove_file(marker_path(&incomplete_store, &incomplete_root.digest).unwrap())
            .unwrap();
        let empty_destination = tempfile::tempdir().unwrap();
        assert!(publish_staged_layout(
            empty_destination.path(),
            &incomplete_store,
            &incomplete_root.digest,
            "example.invalid/app:v2",
        )
        .is_err());
        assert!(!empty_destination
            .path()
            .join(".prism/oci/index.json")
            .exists());
    }

    #[test]
    fn attachments_preserve_verified_subject_and_promotion_identity() {
        let directory = tempfile::tempdir().unwrap();
        let (store, root) = fixture(directory.path(), "example.invalid/app:v1", b"one");
        let mut deployment_value = json!({
            "checks":[{"evidence_digest":format!("sha256:{}", "1".repeat(64)),"id":"readiness","kind":"readiness","status":"passed"}],
            "observed_state":format!("sha256:{}", "2".repeat(64)),
            "operation":"deploy",
            "plan_digest":format!("sha256:{}", "3".repeat(64)),
            "release_digest":root.digest,
            "schema":"prismpm/deployment-evidence/1",
            "status":"accepted",
            "target":"compose-local"
        });
        let evidence_digest = sha(&encode_value(&deployment_value).unwrap());
        deployment_value
            .as_object_mut()
            .unwrap()
            .insert("evidence_digest".to_owned(), json!(evidence_digest));
        let deployment = encode_value(&deployment_value).unwrap();
        attach_referrer(
            directory.path(),
            &root.digest,
            "application/vnd.prismpm.deployment.evidence.v1+json",
            &deployment,
        )
        .unwrap();
        let closure = super::deployment_evidence_closure(directory.path(), &root.digest).unwrap();
        let closure: Value = serde_json::from_slice(&closure).unwrap();
        assert_eq!(closure["evidence"].as_array().unwrap().len(), 1);
        assert_eq!(
            closure["evidence"][0]["digest"],
            deployment_value["evidence_digest"]
        );
        let unsigned =
            super::require_complete_signed_deployment_evidence(directory.path(), &root.digest)
                .unwrap_err();
        assert_eq!(unsigned.code, "PP7401");
        assert_eq!(
            require_verified(&store, &root.digest, true)
                .unwrap()
                .referrers
                .len(),
            5
        );

        let wrong = encode_value(&json!({
            "release_digest":format!("sha256:{}", "0".repeat(64))
        }))
        .unwrap();
        assert!(attach_referrer(
            directory.path(),
            &root.digest,
            "application/vnd.prismpm.deployment.evidence.v1+json",
            &wrong,
        )
        .is_err());
        assert_eq!(
            require_verified(&store, &root.digest, true)
                .unwrap()
                .referrers
                .len(),
            5
        );

        let fake_signature = encode_value(&json!({
            "bundle":{"mediaType":"application/vnd.dev.sigstore.bundle.v0.3+json"},
            "subject":root.digest
        }))
        .unwrap();
        let error = attach_referrer(
            directory.path(),
            &root.digest,
            COSIGN_SIGNATURE,
            &fake_signature,
        )
        .unwrap_err();
        assert_eq!(error.code, "PP7401");

        let skipped = encode_value(&json!({
            "from":"candidate",
            "signature":{"bundle":"fixture-signature"},
            "subject":root.digest,
            "to":"accepted"
        }))
        .unwrap();
        assert!(
            attach_referrer(directory.path(), &root.digest, PRISM_PROMOTION, &skipped,).is_err()
        );

        let promotion = encode_value(&json!({
            "from":"development",
            "signature":{"bundle":"fixture-signature"},
            "subject":root.digest,
            "to":"candidate"
        }))
        .unwrap();
        let error = attach_referrer(directory.path(), &root.digest, PRISM_PROMOTION, &promotion)
            .unwrap_err();
        assert_eq!(error.code, "PP7401");
        let state = require_verified(&store, &root.digest, true).unwrap();
        assert_eq!(state.root.digest, root.digest);
        assert_eq!(state.referrers.len(), 5);
    }
}
