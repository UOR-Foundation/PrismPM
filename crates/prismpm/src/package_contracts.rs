//! Pure package contract DTOs and validation, independent of source and execution.
//!
//! Original Holo module paths re-export these types and functions. Source graph
//! projection, declaration checks, build custody, and artifact execution stay
//! with their image owners.

use crate::error::PrismError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Closed metadata for model-owned, typed native exports and finite acceptance roots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelLibrary {
    /// Exact native-library profile identifier.
    pub profile: String,
    /// Display name; not a filesystem path.
    pub name: String,
    /// Generated Cargo package name.
    pub cargo_name: String,
    /// Exact stable generated package version.
    pub cargo_version: String,
    /// Generated package description.
    pub cargo_description: String,
    /// HTTPS source repository.
    pub cargo_repository: String,
    /// HTTPS project homepage.
    pub cargo_homepage: String,
    /// Sorted fully qualified executable definitions in the bound semantic graph.
    pub export_roots: Vec<String>,
    /// Sorted exported zero-argument Boolean definitions that must all return true.
    pub acceptance_roots: Vec<String>,
}

/// One canonical byte-level application acceptance vector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplicationAcceptanceVector {
    /// Canonical or deliberately malformed request bytes.
    pub request: Vec<u8>,
    /// Exact modeled response bytes.
    pub response: Vec<u8>,
}

/// Mandatory raw file/stdin/stdout adapter. No optional transports or flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFileCli {
    /// Exact immutable transport contract.
    pub profile: String,
}

/// Source-owned bounded Bytes-to-Bytes package, independent of application profiles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryProgram {
    /// Exact immutable package profile.
    pub profile: String,
    /// Human-facing name.
    pub name: String,
    /// Exact generated Cargo package name.
    pub cargo_name: String,
    /// Exact stable version.
    pub cargo_version: String,
    /// Package description.
    pub cargo_description: String,
    /// HTTPS source repository.
    pub cargo_repository: String,
    /// HTTPS homepage.
    pub cargo_homepage: String,
    /// Strictly ordered executable source roots.
    pub export_roots: Vec<String>,
    /// Exported monomorphic Bytes-to-Bytes entry.
    pub entry_root: String,
    /// Maximum input byte count, before core invocation.
    pub request_maximum: u32,
    /// Maximum output byte count, before publication.
    pub response_maximum: u32,
    /// Core-Wasm maximum linear-memory pages.
    pub memory_pages: u32,
    /// Complete finite source-owned exact-byte acceptance corpus.
    pub acceptance_vectors: Vec<ApplicationAcceptanceVector>,
    /// Required bounded file and standard-stream transport.
    pub cli: RawFileCli,
}

fn invalid(message: &str) -> PrismError {
    PrismError::new("PP4004", message)
}

impl BinaryProgram {
    // Reuse metadata constraints only, without presenting this package as library evidence.
    pub(crate) fn metadata(&self) -> ModelLibrary {
        ModelLibrary {
            profile: "prismpm/native-library/1".into(),
            name: self.name.clone(),
            cargo_name: self.cargo_name.clone(),
            cargo_version: self.cargo_version.clone(),
            cargo_description: self.cargo_description.clone(),
            cargo_repository: self.cargo_repository.clone(),
            cargo_homepage: self.cargo_homepage.clone(),
            export_roots: self.export_roots.clone(),
            acceptance_roots: vec![self.entry_root.clone()],
        }
    }
}

/// Validate exact profile, package metadata, bounds, roots and arbitrary-byte vectors.
pub fn validate_binary_program(program: &BinaryProgram) -> Result<(), PrismError> {
    if program.profile != "prismpm/binary-program/1"
        || program.cli.profile != "prismpm/raw-file-cli/1"
    {
        return Err(invalid(
            "unsupported binary-program or mandatory raw-file-cli profile",
        ));
    }
    validate_library(&program.metadata())?;
    // Generated suffixes must retain the generator's 64-byte Cargo name bound.
    if program.cargo_name.len() > 54
        || program.request_maximum == 0
        || program.response_maximum == 0
        || program.request_maximum > 16_777_216
        || program.response_maximum > 16_777_216
        || !(1..=32_767).contains(&program.memory_pages)
    {
        return Err(invalid(
            "binary-program package name or resource bound is invalid",
        ));
    }
    let memory = u64::from(program.memory_pages) * 65_536;
    if u64::from(program.request_maximum) + u64::from(program.response_maximum) + 65_536 > memory {
        return Err(invalid(
            "binary-program transport bounds exceed linear memory",
        ));
    }
    if program.acceptance_vectors.is_empty() || program.acceptance_vectors.len() > 4096 {
        return Err(invalid(
            "binary-program acceptance vectors must be nonempty and bounded",
        ));
    }
    let mut total = 0usize;
    for vector in &program.acceptance_vectors {
        if vector.request.len() > program.request_maximum as usize
            || vector.response.len() > program.response_maximum as usize
        {
            return Err(invalid(
                "binary-program acceptance vector exceeds its byte bounds",
            ));
        }
        total = total
            .checked_add(vector.request.len())
            .and_then(|n| n.checked_add(vector.response.len()))
            .ok_or_else(|| invalid("binary-program vector budget overflow"))?;
        if total > 1_048_576 {
            return Err(invalid("binary-program vector corpus exceeds byte budget"));
        }
    }
    Ok(())
}

pub(crate) fn qualified_root(root: &str) -> bool {
    root.len() <= 1024
        && root.contains('.')
        && root.split('.').all(|part| {
            !part.is_empty()
                && (part.as_bytes()[0].is_ascii_alphabetic() || part.starts_with('_'))
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
}

/// Validate the closed native-only metadata independently of source loading.
pub fn validate_library(library: &ModelLibrary) -> Result<(), PrismError> {
    if library.profile != "prismpm/native-library/1" {
        return Err(invalid("unsupported native-library profile"));
    }
    for (value, maximum) in [(&library.name, 128), (&library.cargo_description, 1024)] {
        if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
            return Err(invalid("native-library display metadata is invalid"));
        }
    }
    let name = &library.cargo_name;
    if name.is_empty()
        || name.len() > 64
        || !name.as_bytes()[0].is_ascii_lowercase()
        || !name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return Err(invalid("native-library Cargo name is invalid"));
    }
    let version = semver::Version::parse(&library.cargo_version)
        .map_err(|_| invalid("native-library Cargo version is invalid"))?;
    if !version.pre.is_empty()
        || !version.build.is_empty()
        || version.to_string() != library.cargo_version
    {
        return Err(invalid(
            "native-library Cargo version must be exact and stable",
        ));
    }
    let uri = jsonschema::options()
        .should_validate_formats(true)
        .build(&serde_json::json!({"type":"string", "format":"uri", "pattern":"^https://"}))
        .map_err(|error| PrismError::new("PP9001", error.to_string()))?;
    for value in [&library.cargo_repository, &library.cargo_homepage] {
        if value.len() > 2048
            || !value.is_ascii()
            || value
                .bytes()
                .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
            || !uri.is_valid(&Value::String(value.clone()))
        {
            return Err(invalid(
                "native-library Cargo URL must be a bounded HTTPS URI",
            ));
        }
    }
    for roots in [&library.export_roots, &library.acceptance_roots] {
        if roots.is_empty()
            || roots.len() > 1024
            || roots.iter().any(|root| !qualified_root(root))
            || roots.windows(2).any(|rows| rows[0] >= rows[1])
        {
            return Err(invalid(
                "native-library roots must be nonempty, bounded, qualified and strictly ordered",
            ));
        }
    }
    if library
        .acceptance_roots
        .iter()
        .any(|root| library.export_roots.binary_search(root).is_err())
    {
        return Err(invalid(
            "native-library acceptance roots must be declared exports",
        ));
    }
    Ok(())
}
