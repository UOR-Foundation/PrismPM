//! Build provenance of a `.holo` (issue #268, design
//! `docs/proposals/268-build-provenance.md`).
//!
//! `prismpm build` writes one in-toto Statement v1 with a SLSA provenance v1
//! predicate into the attestation section of the archive it builds. Subjects
//! are the four content blobs, `resolvedDependencies` is derived from
//! `prismpm.lock`, and the builder is the SDK image the lock pins. The bytes
//! equal what `sdk/npm/prism-code/src/provenance.mjs` writes for the same
//! archive and lock.
//!
//! Min mode, the default, holds nothing that varies between machines or runs.
//! Max mode adds the arguments, platform, invocation id and timestamps and is
//! opt-in because those leak.
//!
//! Nothing here signs. The bundle carries one signature with an empty `sig`,
//! which no verifier accepts, so a built holo stays P0 until `attest` (#272)
//! adds a signed statement.

use crate::attest::clock::iso8601;
use crate::error::PrismError;
use crate::holo::archive::{attach_attestations, parse_application, read_attestations};
use crate::holo::attestation::{BUNDLE_MEDIA_TYPE, DSSE_PAYLOAD_TYPE, STATEMENT_TYPE};
use crate::holo::canonical::encode_value;
use crate::input_lock::{SDK_LOCK_V2, SDK_LOCK_V3};
use crate::inspect::verdict::PROVENANCE_PREDICATE;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// The build type of a holo built by `prismpm build`.
pub const HOLO_BUILD_TYPE: &str = "https://uor.foundation/prism/holo-build/v1";

/// The statement could not be made, so nothing is described.
const REFUSED: &str = "PP3018";

const LOCK_URI: &str = "urn:prismpm:sdk-lock";
const STANDARDS_LOCK_URI: &str = "urn:prismpm:standards-lock";
const TOOLCHAIN_URI: &str = "urn:prismpm:toolchain:";
const GENERATOR_URI: &str = "urn:prismpm:generator:";
const IDENTITIES_ARTIFACT: &str = "application/holo-identities.json";

/// What a build records about itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BuildProvenance {
    /// No statement. The default of the library API and of the verified
    /// product release, whose archive the pinned Hologram oracle checks.
    #[default]
    Omit,
    /// `prismpm build`: the deterministic statement.
    Min,
    /// `prismpm build --provenance=max`, with the command line after `prismpm`.
    Max {
        /// The arguments as the process received them.
        arguments: Vec<String>,
    },
}

/// The members only max mode records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaxDetails {
    /// The command line after `prismpm`.
    pub arguments: Vec<String>,
    /// The OCI platform of the builder, such as `linux/amd64`.
    pub platform: String,
    /// Distinguishes this run from any other.
    pub invocation_id: String,
    /// UTC RFC 3339, whole seconds.
    pub started_on: String,
    /// UTC RFC 3339, whole seconds.
    pub finished_on: String,
}

fn refused(message: impl Into<String>) -> PrismError {
    PrismError::new(REFUSED, message.into())
}

fn dual_digest(bytes: &[u8]) -> Value {
    json!({
        "blake3": blake3::hash(bytes).to_hex().to_string(),
        "sha256": format!("{:x}", Sha256::digest(bytes)),
    })
}

fn is_hex64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_of<'a>(value: &'a Value, member: &str) -> Result<&'a str, PrismError> {
    value.as_str().filter(|text| is_hex64(text)).ok_or_else(|| {
        refused(format!(
            "the lock's {member} is not 64 lowercase hex characters"
        ))
    })
}

fn parse_lock(lock: &[u8]) -> Result<(Value, &'static str), PrismError> {
    let document: Value = serde_json::from_slice(lock)
        .map_err(|error| refused(format!("prismpm.lock is not UTF-8 JSON: {error}")))?;
    let media_type = match document.get("schema").and_then(Value::as_str) {
        Some(SDK_LOCK_V2) => "application/vnd.prismpm.sdk-lock.v2+json",
        Some(SDK_LOCK_V3) => "application/vnd.prismpm.sdk-lock.v3+json",
        schema => {
            return Err(refused(format!(
                "prismpm.lock schema is {}, expected {SDK_LOCK_V2} or {SDK_LOCK_V3}",
                schema.unwrap_or("absent")
            )))
        }
    };
    Ok((document, media_type))
}

/// The resolved dependency rows a lock gives, sorted by `uri` then `name`.
fn resolved_dependencies(
    lock: &[u8],
    document: &Value,
    media_type: &str,
) -> Result<Vec<Value>, PrismError> {
    let standards = document["standards_lock"]
        .as_str()
        .and_then(|value| value.strip_prefix("sha256:"))
        .filter(|hex| is_hex64(hex))
        .ok_or_else(|| refused("the lock's standards_lock is not a sha256: digest"))?;
    let image = document["sdk_image"].as_str().unwrap_or_default();
    let image_digest = image
        .rsplit_once("@sha256:")
        .map(|(_, hex)| hex)
        .filter(|hex| is_hex64(hex))
        .ok_or_else(|| {
            refused("the lock's sdk_image is not an image reference pinned by @sha256:<digest>")
        })?;
    let mut rows = vec![
        json!({"name": "sdk-lock", "uri": LOCK_URI, "digest": dual_digest(lock), "mediaType": media_type}),
        json!({"name": "standards-lock", "uri": STANDARDS_LOCK_URI, "digest": {"sha256": standards}}),
        json!({"name": "sdk-image", "uri": image, "digest": {"sha256": image_digest}}),
    ];
    if document["schema"] == SDK_LOCK_V3 {
        let inputs = document
            .get("inputs")
            .filter(|inputs| inputs.is_object())
            .ok_or_else(|| refused(format!("a {SDK_LOCK_V3} lock has no inputs")))?;
        if let Some(world) = inputs.get("world").filter(|world| world.is_object()) {
            let contract = world["contract"]
                .as_str()
                .ok_or_else(|| refused("the lock's inputs.world names no contract"))?;
            rows.push(json!({
                "name": "world",
                "uri": contract,
                "digest": {
                    "blake3": hex_of(&world["blake3"], "inputs.world.blake3")?,
                    "sha256": hex_of(&world["sha256"], "inputs.world.sha256")?,
                },
            }));
        }
        if let Some(toolchain) = inputs.get("toolchain").and_then(Value::as_object) {
            for (tool, version) in toolchain {
                let version = version
                    .as_str()
                    .filter(|text| !text.is_empty())
                    .ok_or_else(|| {
                        refused(format!("inputs.toolchain.{tool} is not a version string"))
                    })?;
                let name = tool.replace('_', "-");
                rows.push(json!({"name": name, "uri": format!("{TOOLCHAIN_URI}{name}:{version}")}));
            }
        }
        for kind in ["application", "library"] {
            let member = format!("{kind}_generator_sha256");
            rows.push(json!({
                "name": format!("{kind}-generator"),
                "uri": format!("{GENERATOR_URI}{kind}"),
                "digest": {"sha256": hex_of(&inputs["generation"][&member], &format!("inputs.generation.{member}"))?},
            }));
        }
    }
    rows.sort_by(|left, right| {
        (left["uri"].as_str(), left["name"].as_str())
            .cmp(&(right["uri"].as_str(), right["name"].as_str()))
    });
    Ok(rows)
}

/// The provenance statement of an archive. Min mode when `max` is `None`.
///
/// # Errors
///
/// `PP3018` when the lock is not `/2` or `/3`, pins no image by digest, or does
/// not describe this archive; the archive's own diagnostic when it does not
/// validate.
pub fn statement(holo: &[u8], lock: &[u8], max: Option<&MaxDetails>) -> Result<Value, PrismError> {
    let parsed = parse_application(holo)?;
    let directory: Value = serde_json::from_slice(&parsed.directory)
        .map_err(|error| refused(format!("the application directory: {error}")))?;
    let extension: Value = serde_json::from_slice(&parsed.prism_extension)
        .map_err(|error| refused(format!("the model extension: {error}")))?;
    let cargo_name = extension["cargo_name"]
        .as_str()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| refused("the model extension names no cargo_name"))?;
    let capabilities = directory["requires_kappa"]
        .as_str()
        .ok_or_else(|| refused("the application directory names no capability set"))?;
    let roles = [
        (
            format!("core-wasm/{}_core_wasm.wasm", cargo_name.replace('-', "_")),
            parsed.identities.guest_content_kappa.as_str(),
        ),
        (
            "view/hologram/view.holoview".to_owned(),
            parsed.identities.view_content_kappa.as_str(),
        ),
        (
            "model.prism.json".to_owned(),
            parsed.identities.model_content_kappa.as_str(),
        ),
        (
            "application/capability-request.bin".to_owned(),
            capabilities,
        ),
    ];
    let mut subject = Vec::with_capacity(roles.len());
    for (name, label) in &roles {
        let content = parsed
            .blobs
            .get(*label)
            .ok_or_else(|| refused(format!("{label} is not a content blob of this archive")))?;
        subject.push(json!({"name": name, "digest": dual_digest(content)}));
    }
    let model = subject[2]["digest"].clone();

    let (document, media_type) = parse_lock(lock)?;
    let mut external = json!({"model": model});
    let mut builder = json!({"id": document["sdk_image"]});
    if document["schema"] == SDK_LOCK_V3 {
        if document["inputs"]["model"] != model {
            return Err(refused(
                "the lock's inputs.model is not the model document of this archive; the lock does not describe this build",
            ));
        }
        external["spec"] = document["inputs"]["spec"].clone();
        if let Some(prismpm) = document["inputs"]["toolchain"]["prismpm"].as_str() {
            builder["version"] = json!({"prismpm": prismpm});
        }
    }
    let dependencies = resolved_dependencies(lock, &document, media_type)?;
    let mut internal = json!({});
    let mut run_details = json!({"builder": builder});
    if let Some(max) = max {
        external["arguments"] = json!(max.arguments);
        internal = json!({"platform": max.platform});
        run_details["metadata"] = json!({
            "finishedOn": max.finished_on,
            "invocationId": max.invocation_id,
            "startedOn": max.started_on,
        });
    }
    Ok(json!({
        "_type": STATEMENT_TYPE,
        "subject": subject,
        "predicateType": PROVENANCE_PREDICATE,
        "predicate": {
            "buildDefinition": {
                "buildType": HOLO_BUILD_TYPE,
                "externalParameters": external,
                "internalParameters": internal,
                "resolvedDependencies": dependencies,
            },
            "runDetails": run_details,
        },
    }))
}

/// One unsigned bundle line for a statement, as canonical JSON bytes. The
/// envelope payload is the canonical statement; the signature is empty.
///
/// # Errors
///
/// When the value cannot be serialized canonically.
pub fn bundle(statement: &Value) -> Result<Vec<u8>, PrismError> {
    encode_value(&json!({
        "dsseEnvelope": {
            "payload": STANDARD.encode(encode_value(statement)?),
            "payloadType": DSSE_PAYLOAD_TYPE,
            "signatures": [{"sig": ""}],
        },
        "mediaType": BUNDLE_MEDIA_TYPE,
        "verificationMaterial": {"publicKey": {}},
    }))
}

/// A new archive that carries its provenance statement. Refuses an archive that
/// already has one.
///
/// # Errors
///
/// `PP3018` as [`statement`] says, or when a statement is already present.
pub fn attach(holo: &[u8], lock: &[u8], max: Option<&MaxDetails>) -> Result<Vec<u8>, PrismError> {
    if read_attestations(holo)?
        .iter()
        .any(|attestation| attestation.predicate_type == PROVENANCE_PREDICATE)
    {
        return Err(refused(
            "the archive already carries a provenance statement",
        ));
    }
    let line = bundle(&statement(holo, lock, max)?)?;
    attach_attestations(holo, &[line])
}

fn unix_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
        })
}

/// UTC RFC 3339 without fractions, as the predicate schema requires.
fn whole_seconds(millis: i64) -> String {
    iso8601(millis - millis.rem_euclid(1000)).replace(".000Z", "Z")
}

fn oci_platform() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let architecture = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}/{architecture}")
}

/// A UUID v4 URN. The field only has to distinguish runs, so the standard
/// library's per-process random hasher keys are enough.
fn invocation_id() -> String {
    let mut seed = Sha256::new();
    for salt in [0_u8, 1] {
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_u8(salt);
        seed.update(hasher.finish().to_le_bytes());
    }
    seed.update(unix_millis().to_le_bytes());
    seed.update(std::process::id().to_le_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&seed.finalize()[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!(
        "urn:uuid:{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

/// A build's plan to stamp its archive, made before any compilation so that a
/// project with nothing to derive a statement from fails fast.
pub(crate) struct Stamp {
    lock: Vec<u8>,
    max: Option<(Vec<String>, i64)>,
}

impl Stamp {
    /// `None` for [`BuildProvenance::Omit`].
    pub(crate) fn begin(
        root: &Path,
        provenance: &BuildProvenance,
    ) -> Result<Option<Self>, PrismError> {
        let max = match provenance {
            BuildProvenance::Omit => return Ok(None),
            BuildProvenance::Min => None,
            BuildProvenance::Max { arguments } => Some((arguments.clone(), unix_millis())),
        };
        let (_, lock) = crate::sdk::committed_lock(root)?.ok_or_else(|| {
            refused(
                "prismpm build derives provenance from prismpm.lock, and it is absent; bind the SDK with prismpm update --toolchain <digest-ref>, or pass --no-provenance (unsafe: the holo cannot reach P1)",
            )
        })?;
        parse_lock(&lock).map_err(|error| {
            refused(format!(
                "{}; run prismpm lock, or pass --no-provenance (unsafe: the holo cannot reach P1)",
                error.message
            ))
        })?;
        Ok(Some(Self { lock, max }))
    }

    /// Replace the `.holo` among a build's artifacts with the stamped archive
    /// and record the identities of what is now on disk. The footer and the
    /// archive kappa move; rows 0 to 7 keep their bytes.
    pub(crate) fn apply(&self, artifacts: &mut [(String, Vec<u8>)]) -> Result<(), PrismError> {
        let max = self.max.as_ref().map(|(arguments, started)| MaxDetails {
            arguments: arguments.clone(),
            platform: oci_platform(),
            invocation_id: invocation_id(),
            started_on: whole_seconds(*started),
            finished_on: whole_seconds(unix_millis()),
        });
        let (_, holo) = artifacts
            .iter_mut()
            .find(|(path, _)| path.ends_with(".holo"))
            .ok_or_else(|| refused("the application build produced no .holo to describe"))?;
        *holo = attach(holo, &self.lock, max.as_ref())?;
        let identities = serde_json::to_value(parse_application(holo)?.identities)
            .map_err(|error| PrismError::new("PP9001", error.to_string()))?;
        let identities = encode_value(&identities)?;
        let (_, recorded) = artifacts
            .iter_mut()
            .find(|(path, _)| path == IDENTITIES_ARTIFACT)
            .ok_or_else(|| refused(format!("the build produced no {IDENTITIES_ARTIFACT}")))?;
        *recorded = identities;
        Ok(())
    }
}

#[cfg(test)]
#[path = "provenance_tests.rs"]
mod tests;
