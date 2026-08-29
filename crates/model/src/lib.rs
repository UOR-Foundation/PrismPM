//! Typed registries parsed from model/*.toml.

#![deny(missing_docs)]

pub mod codegen;
pub mod registry;
pub mod release;

pub use registry::{Authorities, AuthorityRow, Claim, ErrorRow, Errors, IdRow, Ids, Ledger, Level, StandardRow, Standards, EmitterInputs};

use std::path::{Path, PathBuf};

/// All model registers parsed and checked.
#[derive(Debug, Clone)]
pub struct Model {
    /// model/ledger.toml
    pub ledger: Ledger,
    /// model/ids.toml
    pub ids: Ids,
    /// model/authorities.toml
    pub authorities: Authorities,
    /// model/errors.toml
    pub errors: Errors,
    /// model/standards.toml
    pub standards: Standards,
    /// model/emitter-inputs.toml
    pub emitter_inputs: EmitterInputs,
}

/// Model load/check failure.
#[derive(Debug)]
pub enum ModelError {
    /// I/O error reading file.
    Io(PathBuf, std::io::Error),
    /// Parse error in TOML.
    Parse(PathBuf, toml::de::Error),
    /// Inconsistency within or across model files.
    Inconsistent(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(p, e) => write!(f, "reading {}: {e}", p.display()),
            Self::Parse(p, e) => write!(f, "parsing {}: {e}", p.display()),
            Self::Inconsistent(m) => write!(f, "model is inconsistent: {m}"),
        }
    }
}

impl std::error::Error for ModelError {}

impl Model {
    /// Load all model files from a directory.
    pub fn load(dir: &Path) -> Result<Self, ModelError> {
        Ok(Self {
            ledger: read(dir, "ledger.toml")?,
            ids: read(dir, "ids.toml")?,
            authorities: read(dir, "authorities.toml")?,
            errors: read(dir, "errors.toml")?,
            standards: read(dir, "standards.toml")?,
            emitter_inputs: read(dir, "emitter-inputs.toml")?,
        })
    }

    /// Load model from repository root.
    pub fn load_from_repo_root() -> Result<Self, ModelError> {
        Self::load(&repo_root().join("model"))
    }

    /// Cross-check all model invariants.
    pub fn check(&self) -> Result<(), ModelError> {
        self.ledger.check()?;
        self.check_ids()?;
        self.check_authorities()?;
        self.errors.check()?;
        self.standards.check()?;
        Ok(())
    }

    fn check_ids(&self) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);
        let mut seen = Vec::new();
        for row in &self.ids.id {
            if seen.contains(&row.id.as_str()) {
                return Err(bad(format!("{}: registered twice", row.id)));
            }
            seen.push(&row.id);
            if row.statement.trim().is_empty() {
                return Err(bad(format!("{}: empty statement", row.id)));
            }
            if row.suite.trim().is_empty() {
                return Err(bad(format!("{}: empty suite", row.id)));
            }
            if row.level != Level::Build {
                return Err(bad(format!("{}: level must be build", row.id)));
            }
        }
        Ok(())
    }

    fn check_authorities(&self) -> Result<(), ModelError> {
        let bad = |m: String| ModelError::Inconsistent(m);
        for a in &self.authorities.authority {
            if a.citation.trim().is_empty() {
                return Err(bad(format!("{}: authority missing citation", a.id)));
            }
            for id in &a.realized_by {
                if self.ids.get(id).is_none() {
                    return Err(bad(format!("{}: authority realized_by unknown ID {id}", a.id)));
                }
            }
        }
        for c in &self.ledger.claim {
            if c.level == Level::SomeTrue {
                let Some(name) = &c.authority else {
                    return Err(bad(format!("{}: some-true claim missing authority", c.id)));
                };
                if !self.authorities.authority.iter().any(|a| &a.id == name) {
                    return Err(bad(format!("{}: cites unknown authority {name}", c.id)));
                }
            }
        }
        Ok(())
    }
}

fn read<T: serde::de::DeserializeOwned>(dir: &Path, name: &str) -> Result<T, ModelError> {
    let path = dir.join(name);
    let text = std::fs::read_to_string(&path).map_err(|e| ModelError::Io(path.clone(), e))?;
    toml::from_str(&text).map_err(|e| ModelError::Parse(path, e))
}

/// Repository root resolved relative to CARGO_MANIFEST_DIR.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/model is two levels below repository root")
        .to_path_buf()
}

