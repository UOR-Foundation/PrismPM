//! Closed arbitrary-byte package projection. No Holo, View or deployment profile.
use super::application::{bytes, exact_fields, member_name, record, record_with_unevaluated_fields, string, unsigned, Definitions};
use super::model_document::{ApplicationAcceptanceVector, ModelDocument, ModelLibrary, ProjectionProvenance};
use crate::error::PrismError;
use lexlean::SemanticSnapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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

fn invalid(message: &str) -> PrismError { PrismError::new("PP4004", message) }

impl BinaryProgram {
    // Reuse metadata constraints only, without presenting this package as library evidence.
    pub(crate) fn metadata(&self) -> ModelLibrary {
        ModelLibrary {
            profile: "prismpm/native-library/1".into(), name: self.name.clone(),
            cargo_name: self.cargo_name.clone(), cargo_version: self.cargo_version.clone(),
            cargo_description: self.cargo_description.clone(), cargo_repository: self.cargo_repository.clone(),
            cargo_homepage: self.cargo_homepage.clone(), export_roots: self.export_roots.clone(),
            acceptance_roots: vec![self.entry_root.clone()],
        }
    }
}

/// Validate exact profile, package metadata, bounds, roots and arbitrary-byte vectors.
pub fn validate(program: &BinaryProgram) -> Result<(), PrismError> {
    if program.profile != "prismpm/binary-program/1" || program.cli.profile != "prismpm/raw-file-cli/1" {
        return Err(invalid("unsupported binary-program or mandatory raw-file-cli profile"));
    }
    super::library::validate(&program.metadata())?;
    // Generated suffixes must retain the generator's 64-byte Cargo name bound.
    if program.cargo_name.len() > 54 || program.request_maximum == 0 || program.response_maximum == 0
        || program.request_maximum > 16_777_216 || program.response_maximum > 16_777_216
        || !(1..=32_767).contains(&program.memory_pages) {
        return Err(invalid("binary-program package name or resource bound is invalid"));
    }
    let memory = u64::from(program.memory_pages) * 65_536;
    if u64::from(program.request_maximum) + u64::from(program.response_maximum) + 65_536 > memory {
        return Err(invalid("binary-program transport bounds exceed linear memory"));
    }
    if program.acceptance_vectors.is_empty() || program.acceptance_vectors.len() > 4096 {
        return Err(invalid("binary-program acceptance vectors must be nonempty and bounded"));
    }
    let mut total = 0usize;
    for vector in &program.acceptance_vectors {
        if vector.request.len() > program.request_maximum as usize || vector.response.len() > program.response_maximum as usize {
            return Err(invalid("binary-program acceptance vector exceeds its byte bounds"));
        }
        total = total.checked_add(vector.request.len()).and_then(|n| n.checked_add(vector.response.len()))
            .ok_or_else(|| invalid("binary-program vector budget overflow"))?;
        if total > 1_048_576 { return Err(invalid("binary-program vector corpus exceeds byte budget")); }
    }
    Ok(())
}

/// Resolve every export and the exact entry signature in the typed source graph.
pub fn validate_roots(program: &BinaryProgram, snapshot: &SemanticSnapshot) -> Result<(), PrismError> {
    validate(program)?;
    let mut declarations = BTreeMap::new();
    for module in snapshot.modules() { for declaration in module.declarations() {
        if declarations.insert(format!("{}.{}", module.lean_module(), declaration.lean_name()), declaration).is_some() {
            return Err(invalid("binary-program snapshot repeats a generated declaration"));
        }
    }}
    for root in &program.export_roots {
        let declaration = declarations.get(root).ok_or_else(|| PrismError::new("PP2001", format!("binary-program export {root} is not defined")))?;
        if declaration.kind() != "definition" { return Err(PrismError::new("PP2001", format!("binary-program export {root} is not executable"))); }
        if root == &program.entry_root {
            let ir = declaration.linked_ir();
            let params = ir["parameters"].as_array();
            if !params.is_some_and(|p| p.len() == 1 && p[0]["type"] == serde_json::json!({"kind":"bytes"}))
                || ir["result"] != serde_json::json!({"kind":"bytes"})
                || ir.get("type_parameters").is_some_and(|p| !p.as_array().is_some_and(Vec::is_empty)) {
                return Err(PrismError::new("PP2001", "binary-program entry must be monomorphic Bytes to Bytes"));
            }
        }
    }
    Ok(())
}

/// Project exactly one nominal Foundation.Binary.V1.Model.BinaryProgram value.
pub fn project_program(snapshot: &SemanticSnapshot) -> Result<Option<ModelDocument>, PrismError> {
    let mut definitions = Definitions::new();
    for module in snapshot.modules() { for declaration in module.declarations() {
        if declaration.kind() == "definition" {
            definitions.insert((module.name().to_owned(), declaration.logical_id().to_owned()), declaration.linked_ir());
        }
    }}
    let candidates = definitions.iter().filter(|((owner,_), d)| {
        member_name(d) == Some("BinaryProgram")
            && d.pointer("/result/member/module").and_then(Value::as_str).unwrap_or(owner) == "Foundation.Binary.V1.Model"
    }).collect::<Vec<_>>();
    if candidates.is_empty() { return Ok(None); }
    if candidates.len() != 1 { return Err(invalid("binary-program snapshot requires exactly one nominal descriptor")); }
    let ((module,_), declaration) = candidates[0];
    if !declaration["parameters"].as_array().is_some_and(Vec::is_empty)
        || declaration.get("type_parameters").is_some_and(|p| !p.as_array().is_some_and(Vec::is_empty)) {
        return Err(invalid("binary-program descriptor must be closed and monomorphic"));
    }
    let fields = record_with_unevaluated_fields(&definitions,module,&declaration["body"],&["exportRoots"])?;
    exact_fields(&fields,&["profile","name","cargoName","cargoVersion","cargoDescription","cargoRepository","cargoHomepage","exportRoots","entryRoot","requestMaximum","responseMaximum","memoryPages","acceptanceVectors","cli"])?;
    let (owner, cli) = fields["cli"];
    let cli = record(&definitions,owner,cli)?;
    exact_fields(&cli,&["profile"])?;
    let (owner, mut vectors) = fields["acceptanceVectors"];
    let mut acceptance_vectors = Vec::new();
    loop {
        match vectors["kind"].as_str() {
            Some("nil") => break,
            Some("cons") => {
                if acceptance_vectors.len() == 4096 { return Err(invalid("binary-program vector count exceeds bound")); }
                let fields = record(&definitions,owner,&vectors["head"])?;
                exact_fields(&fields,&["request","response"])?;
                acceptance_vectors.push(ApplicationAcceptanceVector { request: bytes(fields["request"].1,"request")?, response: bytes(fields["response"].1,"response")? });
                vectors = &vectors["tail"];
            }
            _ => return Err(invalid("binary-program acceptanceVectors is not a closed list")),
        }
    }
    let bound = |name| u32::try_from(unsigned(&fields,name,"uint32")?).map_err(|_| invalid("binary-program bound exceeds uint32"));
    let program = BinaryProgram {
        profile:string(&fields,"profile")?, name:string(&fields,"name")?, cargo_name:string(&fields,"cargoName")?,
        cargo_version:string(&fields,"cargoVersion")?,cargo_description:string(&fields,"cargoDescription")?,
        cargo_repository:string(&fields,"cargoRepository")?,cargo_homepage:string(&fields,"cargoHomepage")?,
        export_roots:super::library::roots::project(&definitions,&fields,"exportRoots")?,entry_root:string(&fields,"entryRoot")?,
        request_maximum:bound("requestMaximum")?,response_maximum:bound("responseMaximum")?,memory_pages:bound("memoryPages")?,
        acceptance_vectors,cli:RawFileCli{profile:string(&cli,"profile")?},
    };
    validate_roots(&program,snapshot)?;
    let document = ModelDocument {
        schema:"prismpm/model-document/5".into(),
        provenance:ProjectionProvenance { source_id:snapshot.source_id().to_string(),semantic_id:snapshot.semantic_id().to_string(),
            compiler_semantics_id:snapshot.compiler_semantics_id().to_string(),snapshot_id:snapshot.snapshot_id().to_string(),
            emitter_semantics_id:super::projector::compute_emitter_semantics_id(),facet_packages:Vec::new() },
        standards_profile:Vec::new(),architecture:Default::default(),security:Default::default(),quality:Default::default(),
        application:None,library:None,program:Some(program),
    };
    super::validate::validate(&document)?;
    Ok(Some(document))
}
