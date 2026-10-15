//! Stable source-built compiler provenance, not a serialized cache authority.
use crate::PrismError;
use serde_json::Value;
use sha2::{Digest, Sha256};
fn failure() -> PrismError {
    PrismError::new("PP5008", "controller compiler capsule provenance differs")
}
pub(super) fn validate(value: &Value, executable: &str) -> Result<(), PrismError> {
    let fields = [
        "archive_sha256",
        "compiler_revision",
        "construction",
        "executable",
        "executable_sha256",
        "lean_sha256",
        "mode",
        "platform",
        "schema",
        "toolchain",
    ];
    if value
        .as_object()
        .is_none_or(|object| object.keys().map(String::as_str).ne(fields))
        || value["schema"] != "prismpm/exporter-acquisition/1"
        || value["mode"] != "controller-capsule"
        || value["archive_sha256"] != format!("{:x}", Sha256::digest(super::ARCHIVE))
        || value["executable_sha256"] != executable
        || !matches!(
            value["platform"].as_str(),
            Some("linux/amd64" | "linux/arm64")
        )
    {
        return Err(failure());
    }
    for name in ["executable_sha256", "lean_sha256"] {
        let hash = value[name].as_str().ok_or_else(failure)?;
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(failure());
        }
    }
    let (revision, toolchain) = super::compiler_identity()?;
    if value["compiler_revision"] != revision || value["toolchain"] != toolchain {
        return Err(failure());
    }
    let target = &value["executable"];
    if target.as_object().is_none_or(|object| {
        object
            .keys()
            .map(String::as_str)
            .ne(["byte_length", "mode", "sha256"])
    }) || target["sha256"] != executable
        || target["byte_length"]
            .as_u64()
            .is_none_or(|n| n == 0 || n > 256 * 1024 * 1024)
        || target["mode"]
            .as_u64()
            .is_none_or(|m| m > 0o777 || m & 0o111 == 0)
    {
        return Err(failure());
    }
    let rows = serde_json::json!([value["construction"]]);
    let parsed = crate::release_verification::process_records(&rows, false)?;
    if parsed.len() != 1
        || value["construction"]["tool"] != "lean4-prod-build"
        || value["construction"]["argv"] != serde_json::json!(["build", "prod-export"])
        || value["construction"]["exit_code"] != 0
        || value["construction"].get("exporter").is_some()
    {
        return Err(failure());
    }
    Ok(())
}
