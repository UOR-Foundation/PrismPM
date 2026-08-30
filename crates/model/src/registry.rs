//! Typed shape of model/*.toml registers for PrismPM.

use crate::ModelError;
use serde::Deserialize;

/// Honesty levels for claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// Fact reproduced from an authority.
    SomeTrue,
    /// Constructed and validated against its oracle.
    Build,
    /// Measured and reported, never asserted.
    Open,
}

impl Level {
    /// String token representation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SomeTrue => "some-true",
            Self::Build => "build",
            Self::Open => "open",
        }
    }
}

/// model/ledger.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ledger {
    /// Schema spec identifier.
    pub spec: String,
    /// List of claims.
    pub claim: Vec<Claim>,
}

/// Single claim in the ledger.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    /// Claim ID.
    pub id: String,
    /// Honesty level.
    pub level: Level,
    /// Statement of claim.
    pub statement: String,
    /// Scenario feature file for build claims.
    #[serde(default)]
    pub feature: Option<String>,
    /// Authority cited for some-true claims.
    #[serde(default)]
    pub authority: Option<String>,
    /// Sample size for statistics.
    #[serde(default)]
    pub sample_size: Option<u64>,
    /// Seed for random generators.
    #[serde(default)]
    pub seed: Option<u64>,
}

impl Ledger {
    /// Validate structural invariants of the ledger.
    pub fn check(&self) -> Result<(), ModelError> {
        for c in &self.claim {
            match c.level {
                Level::SomeTrue => {
                    if c.authority.is_none() {
                        return Err(ModelError::Inconsistent(format!(
                            "{}: some-true claim must name an authority",
                            c.id
                        )));
                    }
                }
                Level::Build => {
                    if c.authority.is_some() {
                        return Err(ModelError::Inconsistent(format!(
                            "{}: build claim cannot cite an authority",
                            c.id
                        )));
                    }
                }
                Level::Open => {}
            }
        }
        Ok(())
    }
}

/// model/ids.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ids {
    /// Schema spec identifier.
    pub spec: String,
    /// List of capability IDs.
    pub id: Vec<IdRow>,
}

/// Single capability row.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdRow {
    /// Capability ID.
    pub id: String,
    /// Gherkin suite name.
    pub suite: String,
    /// Honesty level.
    pub level: Level,
    /// Capability statement.
    pub statement: String,
}

impl Ids {
    /// Find row by ID.
    pub fn get(&self, id: &str) -> Option<&IdRow> {
        self.id.iter().find(|r| r.id == id)
    }
}

/// model/authorities.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authorities {
    /// Schema spec identifier.
    pub spec: String,
    /// List of authorities.
    pub authority: Vec<AuthorityRow>,
}

/// Single authority entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityRow {
    /// Authority ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Canonical citation.
    pub citation: String,
    /// Checksum string or 'none'.
    pub checksum: String,
    /// Stated reason if checksum is 'none'.
    pub checksum_reason: String,
    /// Authoritative statement.
    pub statement: String,
    /// Realized capability IDs.
    pub realized_by: Vec<String>,
}

/// model/errors.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Errors {
    /// Schema spec identifier.
    pub spec: String,
    /// List of error entries.
    pub error: Vec<ErrorRow>,
}

/// Single error entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorRow {
    /// Error code (e.g. PP1001).
    pub code: String,
    /// Error title.
    pub title: String,
    /// Error class.
    pub class: String,
    /// Process exit code.
    pub exit: i32,
    /// Explanatory statement.
    pub statement: String,
}

impl Errors {
    /// Validate error codes and uniqueness.
    pub fn check(&self) -> Result<(), ModelError> {
        let mut seen = Vec::new();
        for e in &self.error {
            if seen.contains(&e.code.as_str()) {
                return Err(ModelError::Inconsistent(format!(
                    "{}: duplicate error code",
                    e.code
                )));
            }
            seen.push(&e.code);
            if !e.code.starts_with("PP") || e.code.len() != 6 {
                return Err(ModelError::Inconsistent(format!(
                    "{}: invalid error code shape",
                    e.code
                )));
            }
        }
        Ok(())
    }
}

/// model/standards.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Standards {
    /// Schema spec identifier.
    pub spec: String,
    /// List of standard entries.
    pub standard: Vec<StandardRow>,
}

/// Single standard row.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StandardRow {
    /// Standard identifier.
    pub id: String,
    /// Full standard name.
    pub name: String,
    /// Standard edition.
    pub edition: String,
    /// Exact public catalog reference.
    pub reference: String,
    /// Evidence basis (`public-catalog` or `authorized-clause-review`).
    pub basis: String,
    /// Modeled scope.
    pub scope: String,
    /// Provenance citation.
    pub provenance: String,
    /// Interpretation.
    pub interpretation: String,
    /// Responsible facet package.
    pub facet_package: String,
    /// Exact facet entry IDs implementing this interpretation.
    pub facet_entries: Vec<String>,
    /// Coverage state.
    pub coverage_state: String,
    /// Included in release scope.
    pub release_scope: bool,
}

impl Standards {
    /// Validate standards list invariants.
    pub fn check(&self) -> Result<(), ModelError> {
        for s in &self.standard {
            if s.edition.trim().is_empty() {
                return Err(ModelError::Inconsistent(format!(
                    "{}: missing edition",
                    s.id
                )));
            }
            if s.provenance.trim().is_empty() {
                return Err(ModelError::Inconsistent(format!(
                    "{}: missing provenance",
                    s.id
                )));
            }
            if !matches!(
                s.basis.as_str(),
                "public-catalog" | "authorized-clause-review"
            ) {
                return Err(ModelError::Inconsistent(format!(
                    "{}: unsupported standards evidence basis",
                    s.id
                )));
            }
            if !s.reference.starts_with("https://www.iso.org/standard/") {
                return Err(ModelError::Inconsistent(format!(
                    "{}: reference is not an exact ISO catalog URL",
                    s.id
                )));
            }
            if s.basis == "public-catalog"
                && (s.scope.to_ascii_lowercase().contains("clause")
                    || s.interpretation
                        .to_ascii_lowercase()
                        .contains("conforms to"))
            {
                return Err(ModelError::Inconsistent(format!(
                    "{}: catalog metadata cannot support clause-level conformance",
                    s.id
                )));
            }
            let claim_text = format!("{} {}", s.scope, s.interpretation).to_ascii_lowercase();
            if claim_text.contains("iso certified") || claim_text.contains("iso certification") {
                return Err(ModelError::Inconsistent(format!(
                    "{}: unsupported certification claim",
                    s.id
                )));
            }
            if !matches!(s.coverage_state.as_str(), "implemented" | "known") {
                return Err(ModelError::Inconsistent(format!(
                    "{}: unsupported coverage state",
                    s.id
                )));
            }
            if s.release_scope && s.coverage_state != "implemented" {
                return Err(ModelError::Inconsistent(format!(
                    "{}: release-scope row is not implemented",
                    s.id
                )));
            }
        }
        Ok(())
    }
}

/// model/emitter-inputs.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmitterInputs {
    /// Schema spec identifier.
    pub spec: String,
    /// Canonical framed tree digest of the exact input list.
    pub digest: String,
    /// List of input file paths.
    pub inputs: Vec<String>,
}

/// model/execution-corpus.toml
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCorpus {
    /// Schema spec identifier.
    pub spec: String,
    /// Fixed finite execution strategy identifier.
    pub strategy: String,
    /// Canonical hexadecimal deterministic seed.
    pub seed: String,
    /// Total exhaustive plus deterministic-property case count.
    pub case_count: u64,
    /// Runtime scalar domain.
    pub value_domain: String,
    /// Exhaustive finite-list bounds.
    pub exhaustive: ExhaustiveCorpus,
    /// Deterministic property-test bounds.
    pub property: PropertyCorpus,
    /// Runtime functions and their LexLean-authored oracle theorems.
    pub oracle: Vec<ExecutionOracle>,
}

/// Exhaustive finite-list strategy parameters.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExhaustiveCorpus {
    /// Inclusive maximum list length.
    pub max_length: usize,
    /// Inclusive maximum generated value.
    pub value_max: u64,
    /// Exclusive bound supplied to the all-below validator.
    pub all_below_bound: u64,
    /// Number of exhaustive generated input cases.
    pub case_count: u64,
}

/// Deterministic property-test strategy parameters.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyCorpus {
    /// Number of deterministic generated cases.
    pub case_count: usize,
    /// Inclusive maximum generated list length.
    pub max_length: usize,
    /// Exclusive bound supplied to the flattened validator.
    pub all_below_bound: u64,
    /// Modulus used to generate ordinary values.
    pub generated_value_modulus: u64,
    /// Shrink outcome; passing fixed corpora have no counterexample to shrink.
    pub shrink_result: String,
}

/// One runtime function bound to its formal oracle theorem.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionOracle {
    /// Fully qualified executable function.
    pub function: String,
    /// Fully qualified LexLean-generated theorem stating its expected result.
    pub theorem: String,
    /// Whether the function must occur in the exact named-export root set.
    pub runtime_root: bool,
}

/// Exact generated Lean definitions selected for named LCNF export.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRoots {
    /// Schema spec identifier.
    pub spec: String,
    /// Generated Lean module containing the roots.
    pub lean_module: String,
    /// Logical LCNF module name.
    pub ir_module: String,
    /// Sorted, unique, fully qualified generated Lean definition names.
    pub roots: Vec<String>,
}
