//! Closed containing-system profile for an existing generated browser application.

use super::{CanonicalDocument, PrismError, Projection};
use crate::holo::canonical::{content_id, encode_value};
use crate::holo::model_document::ModelDocument;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const SCHEMA: &str = "prismpm/system-model/2";
pub(super) const MODULE: &str = "Production.BrowserSystem";
pub(super) const CONTROLS: [&str; 4] = [
    "application-acceptance",
    "artifact-closure",
    "source-proof",
    "spdx",
];
pub(crate) const BINDING: &str = "projections/browser-system-release.json";

fn invalid(message: &str) -> PrismError {
    PrismError::new("PP2101", message)
}

/// The same exact scalar/list bindings checked by the modeled manifest predicate.
pub(super) fn certificate(system: &Value) -> Value {
    json!({
        "application_model_digest":system["application_profile"]["application_model_digest"],
        "artifact_id":system["artifacts"][0]["id"],
        "component_id":system["components"][0]["id"],
        "controls":system["controls"],
        "product_id":system["product"]["id"],
        "product_version":system["product"]["version"],
        "target_id":system["targets"][0]["id"]
    })
}

pub(super) fn validate(system: &Value, manifest: &Value) -> Result<(), PrismError> {
    // Closed schema owns cardinalities and profile constants. The authored
    // manifest is never a caller-supplied validity flag.
    let ids = [
        &system["product"]["id"],
        &system["components"][0]["id"],
        &system["artifacts"][0]["id"],
        &system["targets"][0]["id"],
    ];
    if ids
        .iter()
        .filter_map(|id| id.as_str())
        .collect::<BTreeSet<_>>()
        .len()
        != 4
        || system["components"][0]["artifact"] != system["artifacts"][0]["id"]
        || &certificate(system) != manifest
    {
        return Err(invalid(
            "browser system identity, reference, or manifest binding differs",
        ));
    }
    let target = &system["targets"][0];
    if target["adapter_digest"] != crate::deployment::digest("github-pages-browser")?
        || target["api_version"] != "github-pages-artifact@v4"
    {
        return Err(PrismError::new(
            "PP7101",
            "browser system target adapter binding differs",
        ));
    }
    Ok(())
}

pub(crate) fn validate_application(
    system: &Value,
    model: &ModelDocument,
) -> Result<(), PrismError> {
    let application = model
        .application
        .as_ref()
        .ok_or_else(|| invalid("browser system requires a selected generated application"))?;
    if !matches!(
        application,
        crate::holo::model_document::Application::Legacy(_)
            | crate::holo::model_document::Application::Text(_)
    ) || !application.capabilities_empty()
        || system["components"][0]["version"] != application.cargo_version().as_str()
        || system["application_profile"]["application_model_digest"]
            != format!(
                "sha256:{}",
                content_id(&encode_value(
                    &serde_json::to_value(model)
                        .map_err(|error| PrismError::new("PP9001", error.to_string()))?
                )?)
            )
    {
        return Err(invalid(
            "browser system selected application or resource requirements differ",
        ));
    }
    Ok(())
}

fn binding<T: AsRef<[u8]>>(
    system: &CanonicalDocument,
    files: &BTreeMap<String, T>,
) -> Result<CanonicalDocument, PrismError> {
    let model: ModelDocument = serde_json::from_slice(
        files
            .get("model.prism.json")
            .ok_or_else(|| invalid("browser system selected model is absent"))?
            .as_ref(),
    )
    .map_err(|_| invalid("browser system selected model is malformed"))?;
    validate_application(system.value(), &model)?;
    // Reuse the closed supported application browser profile. No arbitrary
    // prefix-owned bundle or substitute generated UI is accepted.
    crate::oci::browser_export::browser_files(files)?;
    let application = model.application.as_ref().expect("validated application");
    if !files.contains_key(&format!("{}.holo", application.name())) {
        return Err(invalid(
            "browser system generated Holo application is absent",
        ));
    }
    let rows = files
        .iter()
        .filter(|(path, _)| {
            path.as_str() != "system.prism.json"
                && path.as_str() != "manifest.json"
                && !path.starts_with("projections/")
        })
        .map(|(path, bytes)| {
            json!({
                "digest":format!("sha256:{}", content_id(bytes.as_ref())),
                "path":path,
                "size":bytes.as_ref().len()
            })
        })
        .collect::<Vec<_>>();
    CanonicalDocument::from_value(
        "prismpm/browser-system-release/1",
        json!({
            "application_model_digest":system.value()["application_profile"]["application_model_digest"],
            "artifact":system.value()["artifacts"][0]["id"],
            "component":system.value()["components"][0]["id"],
            "controls":CONTROLS,
            "files":rows,
            "schema":"prismpm/browser-system-release/1",
            "system_digest":system.digest(),
            "target":system.value()["targets"][0]["id"]
        }),
    )
}

pub(super) fn projections(
    system: &CanonicalDocument,
    artifacts: &[(String, Vec<u8>)],
) -> Result<Vec<Projection>, PrismError> {
    let files = artifacts
        .iter()
        .map(|(path, bytes)| (path.clone(), bytes.as_slice()))
        .collect::<BTreeMap<_, _>>();
    if files.len() != artifacts.len() {
        return Err(invalid(
            "browser system generated artifact paths are duplicated",
        ));
    }
    let binding = binding(system, &files)?;
    Ok(vec![
        Projection {
            path: BINDING.to_owned(),
            media_type: "application/vnd.prismpm.browser-system-release.v1+json".to_owned(),
            bytes: binding.bytes().to_vec(),
        },
        Projection {
            path: "projections/spdx.json".to_owned(),
            media_type: "application/spdx+json".to_owned(),
            bytes: encode_value(&super::spdx(system.value()))?,
        },
    ])
}

/// Recompute the complete deterministic output, independently of a source tree.
pub(crate) fn replay(
    system: &CanonicalDocument,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<(), PrismError> {
    let expected = [
        (BINDING, binding(system, files)?.bytes().to_vec()),
        (
            "projections/spdx.json",
            encode_value(&super::spdx(system.value()))?,
        ),
    ]
    .into_iter()
    .collect::<BTreeMap<_, _>>();
    let actual = files
        .iter()
        .filter(|(path, _)| path.starts_with("projections/"))
        .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
        .collect::<BTreeMap<_, _>>();
    let expected = expected
        .iter()
        .map(|(path, bytes)| (*path, bytes.as_slice()))
        .collect::<BTreeMap<_, _>>();
    if actual != expected {
        return Err(invalid(
            "browser system generated projection or artifact closure differs",
        ));
    }
    Ok(())
}
