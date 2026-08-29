//! Holo DTO data structures conforming to prismpm/holo/1.

use serde::{Deserialize, Serialize};

/// Top-level Holo document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoloDocument {
    /// Schema spec tag (prismpm/holo/1).
    pub schema: String,
    /// Semantic identifier.
    pub semantic_id: String,
    /// Compiler semantics identifier.
    pub compiler_semantics_id: String,
    /// Emitter semantics identifier.
    pub emitter_semantics_id: String,
    /// Standards profile.
    pub standards_profile: String,
    /// Architectural components.
    pub components: Vec<ComponentRecord>,
    /// Architectural edges.
    pub edges: Vec<EdgeRecord>,
    /// Security risks.
    pub risks: Vec<RiskRecord>,
    /// Security controls.
    pub controls: Vec<ControlRecord>,
    /// Product quality requirements.
    pub quality_requirements: Vec<QualityRequirementRecord>,
}

/// Architectural component record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRecord {
    /// Qualified string ID.
    pub id: String,
    /// Normalized numeric runtime index.
    pub index: u64,
    /// Component kind.
    pub kind: String,
}

/// Architectural edge record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeRecord {
    /// Qualified string ID.
    pub id: String,
    /// Normalized index.
    pub index: u64,
    /// Source component index.
    pub from_index: u64,
    /// Target component index.
    pub to_index: u64,
    /// Edge kind.
    pub kind: String,
}

/// Security risk record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskRecord {
    /// Qualified string ID.
    pub id: String,
    /// Normalized index.
    pub index: u64,
    /// Target asset index.
    pub asset_index: u64,
    /// Threat identifier.
    pub threat: String,
}

/// Security control record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlRecord {
    /// Qualified string ID.
    pub id: String,
    /// Normalized index.
    pub index: u64,
    /// Satisfied risk index.
    pub risk_index: u64,
    /// Control objective.
    pub objective: String,
}

/// Quality requirement record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityRequirementRecord {
    /// Qualified string ID.
    pub id: String,
    /// Normalized index.
    pub index: u64,
    /// ISO 25010 characteristic.
    pub characteristic: String,
    /// Quality measure.
    pub measure: String,
}

