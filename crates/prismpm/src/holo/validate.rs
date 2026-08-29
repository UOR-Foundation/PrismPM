//! In-memory Holo model validation and referential integrity checking.

use std::collections::BTreeSet;
use crate::error::PrismError;
use super::dto::HoloDocument;

/// Validate structural and referential integrity of a Holo model.
pub fn validate(doc: &HoloDocument) -> Result<(), PrismError> {
    if doc.schema != "prismpm/holo/1" {
        return Err(PrismError::new("PP3005", format!("unsupported Holo schema: {}", doc.schema)));
    }

    if doc.semantic_id.len() != 64 || !doc.semantic_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PrismError::new("PP3002", "semantic_id must be a 64-character hex string"));
    }
    if doc.compiler_semantics_id.len() != 64 || !doc.compiler_semantics_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PrismError::new("PP3002", "compiler_semantics_id must be a 64-character hex string"));
    }
    if doc.emitter_semantics_id.len() != 64 || !doc.emitter_semantics_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PrismError::new("PP3002", "emitter_semantics_id must be a 64-character hex string"));
    }

    let mut component_indices = BTreeSet::new();
    let mut component_ids = BTreeSet::new();
    for (expected_idx, comp) in doc.components.iter().enumerate() {
        if comp.index != expected_idx as u64 {
            return Err(PrismError::new(
                "PP3004",
                format!("component index mismatch: expected {expected_idx}, found {}", comp.index),
            ));
        }
        if !component_ids.insert(&comp.id) {
            return Err(PrismError::new("PP3003", format!("duplicate component id: {}", comp.id)));
        }
        component_indices.insert(comp.index);
    }

    let mut edge_ids = BTreeSet::new();
    for (expected_idx, edge) in doc.edges.iter().enumerate() {
        if edge.index != expected_idx as u64 {
            return Err(PrismError::new(
                "PP3004",
                format!("edge index mismatch: expected {expected_idx}, found {}", edge.index),
            ));
        }
        if !edge_ids.insert(&edge.id) {
            return Err(PrismError::new("PP3003", format!("duplicate edge id: {}", edge.id)));
        }
        if !component_indices.contains(&edge.from_index) {
            return Err(PrismError::new(
                "PP2004",
                format!("edge {} source index {} is dangling", edge.id, edge.from_index),
            ));
        }
        if !component_indices.contains(&edge.to_index) {
            return Err(PrismError::new(
                "PP2004",
                format!("edge {} target index {} is dangling", edge.id, edge.to_index),
            ));
        }
    }

    let mut risk_indices = BTreeSet::new();
    let mut risk_ids = BTreeSet::new();
    for (expected_idx, risk) in doc.risks.iter().enumerate() {
        if risk.index != expected_idx as u64 {
            return Err(PrismError::new(
                "PP3004",
                format!("risk index mismatch: expected {expected_idx}, found {}", risk.index),
            ));
        }
        if !risk_ids.insert(&risk.id) {
            return Err(PrismError::new("PP3003", format!("duplicate risk id: {}", risk.id)));
        }
        if !doc.components.is_empty() && !component_indices.contains(&risk.asset_index) {
            return Err(PrismError::new(
                "PP2005",
                format!("risk {} references undefined asset index {}", risk.id, risk.asset_index),
            ));
        }
        risk_indices.insert(risk.index);
    }

    let mut control_ids = BTreeSet::new();
    for (expected_idx, ctrl) in doc.controls.iter().enumerate() {
        if ctrl.index != expected_idx as u64 {
            return Err(PrismError::new(
                "PP3004",
                format!("control index mismatch: expected {expected_idx}, found {}", ctrl.index),
            ));
        }
        if !control_ids.insert(&ctrl.id) {
            return Err(PrismError::new("PP3003", format!("duplicate control id: {}", ctrl.id)));
        }
        if !doc.risks.is_empty() && !risk_indices.contains(&ctrl.risk_index) {
            return Err(PrismError::new(
                "PP2006",
                format!("control {} references undefined risk index {}", ctrl.id, ctrl.risk_index),
            ));
        }
    }

    let mut qual_ids = BTreeSet::new();
    for (expected_idx, qual) in doc.quality_requirements.iter().enumerate() {
        if qual.index != expected_idx as u64 {
            return Err(PrismError::new(
                "PP3004",
                format!("quality requirement index mismatch: expected {expected_idx}, found {}", qual.index),
            ));
        }
        if !qual_ids.insert(&qual.id) {
            return Err(PrismError::new("PP3003", format!("duplicate quality requirement id: {}", qual.id)));
        }
    }

    Ok(())
}
