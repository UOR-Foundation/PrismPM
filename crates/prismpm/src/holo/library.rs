//! Explicit native-library projection; no implicit facet or application fallback.

use super::application::{
    exact_fields, member_name, record_with_unevaluated_fields, string, Definitions,
};
use super::model_document::{ModelDocument, ModelLibrary, ProjectionProvenance};
use crate::error::PrismError;
use lexlean::SemanticSnapshot;
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) mod roots;

fn invalid(message: &str) -> PrismError {
    PrismError::new("PP4004", message)
}

pub(crate) use crate::package_contracts::qualified_root as qualified;
pub use crate::package_contracts::validate_library as validate;

/// Resolve every root against the exact typed source snapshot, before any build.
pub fn validate_roots(
    library: &ModelLibrary,
    snapshot: &SemanticSnapshot,
) -> Result<(), PrismError> {
    validate(library)?;
    let mut declarations = BTreeMap::new();
    for module in snapshot.modules() {
        for declaration in module.declarations() {
            let name = format!("{}.{}", module.lean_module(), declaration.lean_name());
            if declarations.insert(name, declaration).is_some() {
                return Err(invalid(
                    "native-library snapshot has duplicate generated declarations",
                ));
            }
        }
    }
    for root in &library.export_roots {
        let declaration = declarations.get(root).ok_or_else(|| {
            PrismError::new(
                "PP2001",
                format!("native-library export {root} is not defined"),
            )
        })?;
        if declaration.kind() != "definition" {
            return Err(PrismError::new(
                "PP2001",
                format!("native-library export {root} is not executable"),
            ));
        }
        if library.acceptance_roots.binary_search(root).is_ok() {
            let value = declaration.linked_ir();
            if !value
                .get("parameters")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
                || value.get("result") != Some(&serde_json::json!({"kind":"bool"}))
            {
                return Err(PrismError::new("PP2001", format!("native-library acceptance root {root} must have type Bool with no parameters")));
            }
        }
    }
    Ok(())
}

/// Project exactly one explicit closed NativeLibrary value, if present.
pub fn project_library(snapshot: &SemanticSnapshot) -> Result<Option<ModelDocument>, PrismError> {
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
        .filter(|((owner, _), declaration)| {
            member_name(declaration) == Some("NativeLibrary")
                && declaration
                    .pointer("/result/member/module")
                    .and_then(Value::as_str)
                    .unwrap_or(owner)
                    == "Foundation.Library.V1.Model"
                && declaration
                    .get("parameters")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
                && declaration.pointer("/body/kind").and_then(Value::as_str) == Some("record")
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(None);
    }
    if candidates.len() != 1 {
        return Err(invalid(
            "a native-library snapshot requires exactly one closed NativeLibrary value",
        ));
    }
    let ((module, _), declaration) = candidates[0];
    let fields = record_with_unevaluated_fields(
        &definitions,
        module,
        &declaration["body"],
        &["exportRoots", "acceptanceRoots"],
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
            "acceptanceRoots",
        ],
    )?;
    let library = ModelLibrary {
        profile: string(&fields, "profile")?,
        name: string(&fields, "name")?,
        cargo_name: string(&fields, "cargoName")?,
        cargo_version: string(&fields, "cargoVersion")?,
        cargo_description: string(&fields, "cargoDescription")?,
        cargo_repository: string(&fields, "cargoRepository")?,
        cargo_homepage: string(&fields, "cargoHomepage")?,
        export_roots: roots::project(&definitions, &fields, "exportRoots")?,
        acceptance_roots: roots::project(&definitions, &fields, "acceptanceRoots")?,
    };
    validate_roots(&library, snapshot)?;
    let document = ModelDocument {
        schema: "prismpm/model-document/3".to_owned(),
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
        library: Some(library),
        program: None,
    };
    super::validate::validate(&document)?;
    Ok(Some(document))
}
