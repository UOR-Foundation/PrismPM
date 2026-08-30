//! Structural, ordering, profile, and cross-reference validation for Holo.

use super::dto::HoloDocument;
use crate::error::PrismError;
use std::collections::BTreeSet;

const PROFILE: [&str; 5] = [
    "ISO-25010-2023",
    "ISO-27005-2022",
    "ISO-27034-1-2011",
    "ISO-27034-5-2017",
    "ISO-42010-2022",
];

fn digest(value: &str, field: &str) -> Result<(), PrismError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(PrismError::new(
            "PP3002",
            format!("{field} is not a lowercase SHA-256 digest"),
        ))
    }
}

fn identifier(value: &str, field: &str) -> Result<(), PrismError> {
    if value.is_empty()
        || !value.is_ascii()
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'@' | b'-' | b'_')
        })
    {
        return Err(PrismError::new(
            "PP3002",
            format!("{field} is not a qualified ASCII identifier"),
        ));
    }
    Ok(())
}

fn ordered<'a>(
    kind: &str,
    rows: impl IntoIterator<Item = (&'a str, u64)>,
) -> Result<BTreeSet<String>, PrismError> {
    let rows: Vec<_> = rows.into_iter().collect();
    let mut ids = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for (expected, (id, index)) in rows.iter().enumerate() {
        identifier(id, kind)?;
        if !ids.insert((*id).to_owned()) {
            return Err(PrismError::new(
                "PP3003",
                format!("duplicate {kind} ID {id}"),
            ));
        }
        if previous.is_some_and(|prior| prior.as_bytes() >= id.as_bytes()) {
            return Err(PrismError::new(
                "PP3002",
                format!("{kind} IDs are not in canonical ASCII order"),
            ));
        }
        if *index != expected as u64 {
            return Err(PrismError::new(
                "PP3004",
                format!("{kind} {id} has index {index}, expected {expected}"),
            ));
        }
        previous = Some(id);
    }
    Ok(ids)
}

fn contains_index<T>(rows: &[T], index: u64) -> bool {
    usize::try_from(index).is_ok_and(|value| value < rows.len())
}

/// Validate a Holo DTO before encoding or after strict decoding.
pub fn validate(doc: &HoloDocument) -> Result<(), PrismError> {
    if doc.schema != "prismpm/holo/1" {
        return Err(PrismError::new("PP3005", "unsupported Holo schema"));
    }
    digest(&doc.provenance.source_id, "source_id")?;
    digest(&doc.provenance.semantic_id, "semantic_id")?;
    digest(
        &doc.provenance.compiler_semantics_id,
        "compiler_semantics_id",
    )?;
    digest(&doc.provenance.snapshot_id, "snapshot_id")?;
    digest(&doc.provenance.emitter_semantics_id, "emitter_semantics_id")?;
    if doc
        .standards_profile
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        != PROFILE
    {
        return Err(PrismError::new("PP2001", "standards profile is not exact"));
    }
    let packages: Vec<_> = doc
        .provenance
        .facet_packages
        .iter()
        .map(|row| row.package.as_str())
        .collect();
    if packages != ["prism.arch", "prism.qual", "prism.sec"] {
        return Err(PrismError::new("PP2001", "facet package list is not exact"));
    }
    for package in &doc.provenance.facet_packages {
        identifier(&package.package, "facet package")?;
        if package.version != "1.0.0" {
            return Err(PrismError::new("PP2001", "facet version is not exact"));
        }
        digest(&package.content_id, "facet content_id")?;
    }

    let component_kinds = ordered(
        "component kind",
        doc.architecture
            .component_kinds
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    let edge_kinds = ordered(
        "edge kind",
        doc.architecture
            .edge_kinds
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    ordered(
        "model kind",
        doc.architecture
            .model_kinds
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    ordered(
        "component",
        doc.architecture
            .components
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.architecture.components {
        if !component_kinds.contains(&row.kind_id) {
            return Err(PrismError::new("PP2001", "component kind is unresolved"));
        }
    }
    ordered(
        "edge",
        doc.architecture
            .edges
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.architecture.edges {
        if !contains_index(&doc.architecture.components, row.from_index)
            || !contains_index(&doc.architecture.components, row.to_index)
        {
            return Err(PrismError::new("PP2004", "edge endpoint is dangling"));
        }
        if !edge_kinds.contains(&row.kind_id) {
            return Err(PrismError::new("PP2001", "edge kind is unresolved"));
        }
    }
    ordered(
        "stakeholder",
        doc.architecture
            .stakeholders
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    ordered(
        "concern",
        doc.architecture
            .concerns
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    ordered(
        "viewpoint",
        doc.architecture
            .viewpoints
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.architecture.viewpoints {
        if !contains_index(&doc.architecture.stakeholders, row.stakeholder_index)
            || !contains_index(&doc.architecture.concerns, row.concern_index)
            || !contains_index(&doc.architecture.model_kinds, row.model_kind_index)
        {
            return Err(PrismError::new("PP2001", "viewpoint link is unresolved"));
        }
    }
    ordered(
        "view",
        doc.architecture
            .views
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.architecture.views {
        if !contains_index(&doc.architecture.viewpoints, row.viewpoint_index)
            || !contains_index(&doc.architecture.model_kinds, row.model_kind_index)
        {
            return Err(PrismError::new("PP2001", "view link is unresolved"));
        }
    }

    let likelihoods = ordered(
        "likelihood",
        doc.security
            .likelihoods
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    let impacts = ordered(
        "impact",
        doc.security
            .impacts
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    ordered(
        "asset",
        doc.security
            .assets
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.security.assets {
        if !contains_index(&doc.architecture.components, row.component_index) {
            return Err(PrismError::new("PP2005", "asset component is unresolved"));
        }
    }
    ordered(
        "threat",
        doc.security
            .threats
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    ordered(
        "risk",
        doc.security
            .risks
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.security.risks {
        if !contains_index(&doc.security.assets, row.asset_index)
            || !contains_index(&doc.security.threats, row.threat_index)
            || !likelihoods.contains(&row.likelihood_id)
            || !impacts.contains(&row.impact_id)
        {
            return Err(PrismError::new("PP2005", "risk link is unresolved"));
        }
    }
    ordered(
        "control",
        doc.security
            .controls
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.security.controls {
        if !contains_index(&doc.security.risks, row.risk_index) {
            return Err(PrismError::new("PP2006", "control risk is unresolved"));
        }
    }
    ordered(
        "activity",
        doc.security
            .activities
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.security.activities {
        if !contains_index(&doc.security.controls, row.control_index) {
            return Err(PrismError::new("PP2006", "activity control is unresolved"));
        }
    }
    ordered(
        "measurement",
        doc.security
            .measurements
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.security.measurements {
        if !contains_index(&doc.security.controls, row.control_index) {
            return Err(PrismError::new(
                "PP2006",
                "measurement control is unresolved",
            ));
        }
    }

    ordered(
        "quality characteristic",
        doc.quality
            .characteristics
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    ordered(
        "quality subcharacteristic",
        doc.quality
            .subcharacteristics
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.quality.subcharacteristics {
        if !contains_index(&doc.quality.characteristics, row.characteristic_index) {
            return Err(PrismError::new(
                "PP2007",
                "quality characteristic is unresolved",
            ));
        }
    }
    ordered(
        "quality requirement",
        doc.quality
            .requirements
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.quality.requirements {
        if !contains_index(&doc.quality.subcharacteristics, row.subcharacteristic_index) {
            return Err(PrismError::new("PP2007", "subcharacteristic is unresolved"));
        }
    }
    ordered(
        "quality measure",
        doc.quality
            .measures
            .iter()
            .map(|row| (row.id.as_str(), row.index)),
    )?;
    for row in &doc.quality.measures {
        if !contains_index(&doc.quality.requirements, row.requirement_index) {
            return Err(PrismError::new(
                "PP2007",
                "measure requirement is unresolved",
            ));
        }
    }
    Ok(())
}
