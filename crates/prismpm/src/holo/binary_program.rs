//! Closed arbitrary-byte package projection. No Holo, View or deployment profile.
use super::application::{
    bytes, exact_fields, member_name, record, record_with_unevaluated_fields, string, unsigned,
    Definitions,
};
use super::model_document::{ApplicationAcceptanceVector, ModelDocument, ProjectionProvenance};
use crate::error::PrismError;
use lexlean::SemanticSnapshot;
use serde_json::Value;
use std::collections::BTreeMap;

pub use crate::package_contracts::{
    validate_binary_program as validate, BinaryProgram, RawFileCli,
};

fn invalid(message: &str) -> PrismError {
    PrismError::new("PP4004", message)
}

/// Resolve every export and the exact entry signature in the typed source graph.
pub fn validate_roots(
    program: &BinaryProgram,
    snapshot: &SemanticSnapshot,
) -> Result<(), PrismError> {
    validate(program)?;
    let mut declarations = BTreeMap::new();
    for module in snapshot.modules() {
        for declaration in module.declarations() {
            if declarations
                .insert(
                    format!("{}.{}", module.lean_module(), declaration.lean_name()),
                    declaration,
                )
                .is_some()
            {
                return Err(invalid(
                    "binary-program snapshot repeats a generated declaration",
                ));
            }
        }
    }
    for root in &program.export_roots {
        let declaration = declarations.get(root).ok_or_else(|| {
            PrismError::new(
                "PP2001",
                format!("binary-program export {root} is not defined"),
            )
        })?;
        if declaration.kind() != "definition" {
            return Err(PrismError::new(
                "PP2001",
                format!("binary-program export {root} is not executable"),
            ));
        }
        if root == &program.entry_root {
            let ir = declaration.linked_ir();
            let params = ir["parameters"].as_array();
            if !params.is_some_and(|p| {
                p.len() == 1 && p[0]["type"] == serde_json::json!({"kind":"bytes"})
            }) || ir["result"] != serde_json::json!({"kind":"bytes"})
                || ir
                    .get("type_parameters")
                    .is_some_and(|p| !p.as_array().is_some_and(Vec::is_empty))
            {
                return Err(PrismError::new(
                    "PP2001",
                    "binary-program entry must be monomorphic Bytes to Bytes",
                ));
            }
        }
    }
    Ok(())
}

/// Project exactly one nominal Foundation.Binary.V1.Model.BinaryProgram value.
pub fn project_program(snapshot: &SemanticSnapshot) -> Result<Option<ModelDocument>, PrismError> {
    let mut definitions = Definitions::new();
    for module in snapshot.modules() {
        for declaration in module.declarations() {
            if declaration.kind() == "definition" {
                definitions.insert(
                    (
                        module.name().to_owned(),
                        declaration.logical_id().to_owned(),
                    ),
                    declaration.linked_ir(),
                );
            }
        }
    }
    let candidates = definitions
        .iter()
        .filter(|((owner, _), d)| {
            member_name(d) == Some("BinaryProgram")
                && d.pointer("/result/member/module")
                    .and_then(Value::as_str)
                    .unwrap_or(owner)
                    == "Foundation.Binary.V1.Model"
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() != 1 {
        return Err(invalid(
            "binary-program snapshot requires exactly one nominal descriptor",
        ));
    }
    let ((module, _), declaration) = candidates[0];
    if !declaration["parameters"]
        .as_array()
        .is_some_and(Vec::is_empty)
        || declaration
            .get("type_parameters")
            .is_some_and(|p| !p.as_array().is_some_and(Vec::is_empty))
    {
        return Err(invalid(
            "binary-program descriptor must be closed and monomorphic",
        ));
    }
    let fields = record_with_unevaluated_fields(
        &definitions,
        module,
        &declaration["body"],
        &["exportRoots"],
    )?;
    exact_fields(
        &fields,
        &[
            "profile",
            "name",
            "cargoName",
            "cargoVersion",
            "cargoDescription",
            "cargoRepository",
            "cargoHomepage",
            "exportRoots",
            "entryRoot",
            "requestMaximum",
            "responseMaximum",
            "memoryPages",
            "acceptanceVectors",
            "cli",
        ],
    )?;
    let (owner, cli) = fields["cli"];
    let cli = record(&definitions, owner, cli)?;
    exact_fields(&cli, &["profile"])?;
    let (owner, mut vectors) = fields["acceptanceVectors"];
    let mut acceptance_vectors = Vec::new();
    loop {
        match vectors["kind"].as_str() {
            Some("nil") => break,
            Some("cons") => {
                if acceptance_vectors.len() == 4096 {
                    return Err(invalid("binary-program vector count exceeds bound"));
                }
                let fields = record(&definitions, owner, &vectors["head"])?;
                exact_fields(&fields, &["request", "response"])?;
                acceptance_vectors.push(ApplicationAcceptanceVector {
                    request: bytes(fields["request"].1, "request")?,
                    response: bytes(fields["response"].1, "response")?,
                });
                vectors = &vectors["tail"];
            }
            _ => {
                return Err(invalid(
                    "binary-program acceptanceVectors is not a closed list",
                ))
            }
        }
    }
    let bound = |name| {
        u32::try_from(unsigned(&fields, name, "uint32")?)
            .map_err(|_| invalid("binary-program bound exceeds uint32"))
    };
    let program = BinaryProgram {
        profile: string(&fields, "profile")?,
        name: string(&fields, "name")?,
        cargo_name: string(&fields, "cargoName")?,
        cargo_version: string(&fields, "cargoVersion")?,
        cargo_description: string(&fields, "cargoDescription")?,
        cargo_repository: string(&fields, "cargoRepository")?,
        cargo_homepage: string(&fields, "cargoHomepage")?,
        export_roots: super::library::roots::project(&definitions, &fields, "exportRoots")?,
        entry_root: string(&fields, "entryRoot")?,
        request_maximum: bound("requestMaximum")?,
        response_maximum: bound("responseMaximum")?,
        memory_pages: bound("memoryPages")?,
        acceptance_vectors,
        cli: RawFileCli {
            profile: string(&cli, "profile")?,
        },
    };
    validate_roots(&program, snapshot)?;
    let document = ModelDocument {
        schema: "prismpm/model-document/7".into(),
        provenance: ProjectionProvenance {
            source_id: snapshot.source_id().to_string(),
            semantic_id: snapshot.semantic_id().to_string(),
            compiler_semantics_id: snapshot.compiler_semantics_id().to_string(),
            snapshot_id: snapshot.snapshot_id().to_string(),
            emitter_semantics_id: super::projector::compute_emitter_semantics_id(),
            facet_packages: Vec::new(),
        },
        standards_profile: Vec::new(),
        architecture: Default::default(),
        security: Default::default(),
        quality: Default::default(),
        application: None,
        library: None,
        program: Some(program),
    };
    super::validate::validate(&document)?;
    Ok(Some(document))
}
