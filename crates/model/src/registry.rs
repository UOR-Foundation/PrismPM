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
    /// Modeled scope.
    pub scope: String,
    /// Provenance citation.
    pub provenance: String,
    /// Interpretation.
    pub interpretation: String,
    /// Responsible facet package.
    pub facet_package: String,
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
    /// List of input file paths.
    pub inputs: Vec<String>,
}
