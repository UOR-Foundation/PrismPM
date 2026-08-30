//! Owned DTOs for the canonical `prismpm/holo/1` interchange format.

use serde::{Deserialize, Serialize};

/// Top-level Holo document. Its three domain sections contain only canonical
/// arrays; verification evidence is deliberately kept outside this format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoloDocument {
    /// Schema discriminator.
    pub schema: String,
    /// Compiler and projection provenance.
    pub provenance: ProjectionProvenance,
    /// Exact, fixed standards profile identifiers.
    pub standards_profile: Vec<String>,
    /// Architecture records.
    pub architecture: ArchitectureModel,
    /// Security and risk records.
    pub security: SecurityModel,
    /// Product-quality records.
    pub quality: QualityModel,
}

/// Provenance that determines projection bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionProvenance {
    /// LexLean source identity.
    pub source_id: String,
    /// LexLean linked-semantic identity.
    pub semantic_id: String,
    /// LexLean compiler-semantics identity.
    pub compiler_semantics_id: String,
    /// SHA-256 identity of the semantic snapshot bytes.
    pub snapshot_id: String,
    /// Prism Holo emitter-semantics identity.
    pub emitter_semantics_id: String,
    /// Exact facet package closure.
    pub facet_packages: Vec<FacetPackage>,
}

/// One exact facet package used by the projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacetPackage {
    /// Package identifier.
    pub package: String,
    /// Semantic version.
    pub version: String,
    /// Canonical package tree digest.
    pub content_id: String,
}

/// Architecture section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureModel {
    /// Fixed component-kind catalog.
    pub component_kinds: Vec<CatalogRecord>,
    /// Fixed edge-kind catalog.
    pub edge_kinds: Vec<CatalogRecord>,
    /// Fixed model-kind catalog.
    pub model_kinds: Vec<CatalogRecord>,
    /// Components.
    pub components: Vec<ComponentRecord>,
    /// Edges.
    pub edges: Vec<EdgeRecord>,
    /// Stakeholders.
    pub stakeholders: Vec<IndexedRecord>,
    /// Concerns.
    pub concerns: Vec<IndexedRecord>,
    /// Viewpoints.
    pub viewpoints: Vec<ViewpointRecord>,
    /// Views.
    pub views: Vec<ViewRecord>,
}

/// Security and risk section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SecurityModel {
    /// Likelihood catalog.
    pub likelihoods: Vec<CatalogRecord>,
    /// Impact catalog.
    pub impacts: Vec<CatalogRecord>,
    /// Assets.
    pub assets: Vec<AssetRecord>,
    /// Threats.
    pub threats: Vec<IndexedRecord>,
    /// Risks.
    pub risks: Vec<RiskRecord>,
    /// Application-security controls.
    pub controls: Vec<ControlRecord>,
    /// Security activities.
    pub activities: Vec<ActivityRecord>,
    /// Verification measurements.
    pub measurements: Vec<MeasurementRecord>,
}

/// Product-quality section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct QualityModel {
    /// Product-quality characteristics.
    pub characteristics: Vec<IndexedRecord>,
    /// Product-quality subcharacteristics.
    pub subcharacteristics: Vec<SubcharacteristicRecord>,
    /// Quality requirements.
    pub requirements: Vec<RequirementRecord>,
    /// Measures.
    pub measures: Vec<MeasureRecord>,
}

/// A fixed catalog value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogRecord {
    /// Qualified package-entry identifier.
    pub id: String,
    /// Canonical zero-based index.
    pub index: u64,
}

/// An entity carrying only identity and canonical index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexedRecord {
    /// Qualified `<module>::<component>` identifier.
    pub id: String,
    /// Canonical zero-based index.
    pub index: u64,
}

/// Architectural component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Qualified component-kind catalog identifier.
    pub kind_id: String,
}

/// Architectural edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Source component index.
    pub from_index: u64,
    /// Target component index.
    pub to_index: u64,
    /// Qualified edge-kind catalog identifier.
    pub kind_id: String,
}

/// Architecture viewpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewpointRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Stakeholder index.
    pub stakeholder_index: u64,
    /// Concern index.
    pub concern_index: u64,
    /// Model-kind index.
    pub model_kind_index: u64,
}

/// Architecture view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Viewpoint index.
    pub viewpoint_index: u64,
    /// Model-kind index.
    pub model_kind_index: u64,
}

/// Security asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Owning component index.
    pub component_index: u64,
}

/// Risk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Asset index.
    pub asset_index: u64,
    /// Threat index.
    pub threat_index: u64,
    /// Qualified likelihood catalog ID.
    pub likelihood_id: String,
    /// Qualified impact catalog ID.
    pub impact_id: String,
}

/// Application-security control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Risk index.
    pub risk_index: u64,
}

/// Security activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActivityRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Control index.
    pub control_index: u64,
}

/// Verification measurement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Control index.
    pub control_index: u64,
    /// Fixed observed Boolean for the reference model.
    pub passed: bool,
}

/// Product-quality subcharacteristic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubcharacteristicRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Characteristic index.
    pub characteristic_index: u64,
}

/// Quality requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Subcharacteristic index.
    pub subcharacteristic_index: u64,
}

/// Quality measure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasureRecord {
    /// Qualified entity identifier.
    pub id: String,
    /// Canonical index.
    pub index: u64,
    /// Requirement index.
    pub requirement_index: u64,
    /// Natural-number threshold.
    pub threshold: u64,
}
