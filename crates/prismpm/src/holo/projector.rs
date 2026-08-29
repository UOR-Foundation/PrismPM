//! Semantic snapshot to Holo projector.

use std::collections::BTreeMap;
use sha2::{Digest, Sha256};
use crate::error::PrismError;
use super::dto::{
    ComponentRecord, ControlRecord, EdgeRecord, HoloDocument, QualityRequirementRecord,
    RiskRecord,
};

/// Project a LexLean semantic snapshot into a HoloDocument.
pub fn project_snapshot(snapshot_json: &[u8]) -> Result<HoloDocument, PrismError> {
    let parsed: serde_json::Value = serde_json::from_slice(snapshot_json)
        .map_err(|e| PrismError::new("PP3001", format!("malformed snapshot JSON: {e}")))?;

    let spec = parsed.get("spec").and_then(|v| v.as_str()).unwrap_or("");
    if spec != "lexlean/semantic-snapshot/1" {
        return Err(PrismError::new("PP3005", "unsupported semantic snapshot schema"));
    }

    let source_id = parsed
        .get("source_id")
        .and_then(|v| v.as_str())
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000")
        .to_owned();
    let semantic_id = parsed
        .get("semantic_id")
        .and_then(|v| v.as_str())
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000")
        .to_owned();
    let compiler_semantics_id = parsed
        .get("compiler_semantics_id")
        .and_then(|v| v.as_str())
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000")
        .to_owned();

    let mut raw_components = BTreeMap::new();
    let mut raw_edges = BTreeMap::new();
    let mut raw_risks = BTreeMap::new();
    let mut raw_controls = BTreeMap::new();
    let mut raw_qual = BTreeMap::new();

    if let Some(modules) = parsed.get("modules").and_then(|v| v.as_array()) {
        for module in modules {
            let mod_name = module.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(decls) = module.get("declarations").and_then(|v| v.as_array()) {
                for decl in decls {
                    let id = decl.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let qualified = format!("{mod_name}.{id}");
                    if mod_name.contains("Arch") || id.starts_with("comp_") || id.starts_with("component_") {
                        raw_components.insert(qualified.clone(), "service".to_owned());
                    } else if id.starts_with("edge_") || id.starts_with("flow_") {
                        raw_edges.insert(qualified.clone(), ("data_flow".to_owned(), 0u64, 0u64));
                    } else if mod_name.contains("Sec") || id.starts_with("risk_") {
                        raw_risks.insert(qualified.clone(), (0u64, "unauthorized_access".to_owned()));
                    } else if id.starts_with("ctrl_") || id.starts_with("control_") {
                        raw_controls.insert(qualified.clone(), (0u64, "mutual_tls".to_owned()));
                    } else if mod_name.contains("Qual") || id.starts_with("qual_") {
                        raw_qual.insert(qualified.clone(), ("maintainability".to_owned(), "modularity".to_owned()));
                    } else {
                        raw_components.insert(qualified.clone(), "general_component".to_owned());
                    }
                }
            }
        }
    }

    let mut component_indices = BTreeMap::new();
    let mut components = Vec::new();
    for (idx, (id, kind)) in raw_components.into_iter().enumerate() {
        let index = idx as u64;
        component_indices.insert(id.clone(), index);
        components.push(ComponentRecord {
            id,
            index,
            kind,
        });
    }

    let mut edges = Vec::new();
    for (idx, (id, (kind, from_idx, to_idx))) in raw_edges.into_iter().enumerate() {
        let max_comp = components.len().saturating_sub(1) as u64;
        edges.push(EdgeRecord {
            id,
            index: idx as u64,
            from_index: from_idx.min(max_comp),
            to_index: to_idx.min(max_comp),
            kind,
        });
    }

    let mut risks = Vec::new();
    for (idx, (id, (asset_idx, threat))) in raw_risks.into_iter().enumerate() {
        let max_comp = components.len().saturating_sub(1) as u64;
        risks.push(RiskRecord {
            id,
            index: idx as u64,
            asset_index: asset_idx.min(max_comp),
            threat,
        });
    }

    let mut controls = Vec::new();
    for (idx, (id, (risk_idx, objective))) in raw_controls.into_iter().enumerate() {
        let max_risk = risks.len().saturating_sub(1) as u64;
        controls.push(ControlRecord {
            id,
            index: idx as u64,
            risk_index: risk_idx.min(max_risk),
            objective,
        });
    }

    let mut quality_requirements = Vec::new();
    for (idx, (id, (characteristic, measure))) in raw_qual.into_iter().enumerate() {
        quality_requirements.push(QualityRequirementRecord {
            id,
            index: idx as u64,
            characteristic,
            measure,
        });
    }

    let emitter_semantics_id = compute_emitter_semantics_id();

    let doc = HoloDocument {
        schema: "prismpm/holo/1".to_owned(),
        semantic_id,
        compiler_semantics_id,
        emitter_semantics_id,
        standards_profile: "ISO-42010-2022/ISO-27034-1-2011/ISO-27005-2022/ISO-25010-2023".to_owned(),
        components,
        edges,
        risks,
        controls,
        quality_requirements,
    };

    super::validate::validate(&doc)?;
    Ok(doc)
}

/// Compute emitter-semantics ID from emitter inputs.
pub fn compute_emitter_semantics_id() -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"prismpm/holo/1-emitter-v1.0.0");
    format!("{:x}", hasher.finalize())
}
