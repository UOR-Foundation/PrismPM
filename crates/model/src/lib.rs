//! Typed registries parsed from model/*.toml.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod codegen;
pub mod registry;
pub mod release;

pub use registry::{
    Authorities, AuthorityRow, Claim, EmitterInputs, ErrorRow, Errors, ExecutionCorpus,
    ExecutionOracle, ExhaustiveCorpus, IdRow, Ids, Ledger, Level, PropertyCorpus, RuntimeRoots,
    StandardRow, Standards,
};

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
    /// model/execution-corpus.toml
    pub execution_corpus: ExecutionCorpus,
    /// model/runtime-roots.toml
    pub runtime_roots: RuntimeRoots,
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
            execution_corpus: read(dir, "execution-corpus.toml")?,
            runtime_roots: read(dir, "runtime-roots.toml")?,
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
        self.check_execution_corpus()?;
        Ok(())
    }

    fn check_execution_corpus(&self) -> Result<(), ModelError> {
        let corpus = &self.execution_corpus;
        let bad = |message: &str| ModelError::Inconsistent(message.to_owned());
        let width = corpus
            .exhaustive
            .value_max
            .checked_add(1)
            .ok_or_else(|| bad("execution corpus value range overflows"))?;
        let mut list_count = 0_u64;
        let mut width_power = 1_u64;
        for length in 0..=corpus.exhaustive.max_length {
            list_count = list_count
                .checked_add(width_power)
                .ok_or_else(|| bad("execution corpus size overflows"))?;
            if length != corpus.exhaustive.max_length {
                width_power = width_power
                    .checked_mul(width)
                    .ok_or_else(|| bad("execution corpus size overflows"))?;
            }
        }
        let exhaustive_cases = list_count;
        let property_cases = u64::try_from(corpus.property.case_count)
            .map_err(|_| bad("execution property count overflows"))?;
        let expected_cases = exhaustive_cases
            .checked_add(property_cases)
            .ok_or_else(|| bad("execution corpus size overflows"))?;
        let mut functions = corpus
            .oracle
            .iter()
            .map(|row| row.function.as_str())
            .collect::<Vec<_>>();
        let original = functions.clone();
        functions.sort_unstable();
        functions.dedup();
        let theorem_count = corpus
            .oracle
            .iter()
            .map(|row| row.theorem.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let runtime_functions = corpus
            .oracle
            .iter()
            .filter(|row| row.runtime_root)
            .map(|row| row.function.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let runtime_roots = self
            .runtime_roots
            .roots
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        if corpus.spec != "prismpm/execution-corpus/1"
            || corpus.strategy != "exhaustive-v1+lcg-v1"
            || corpus.seed.len() != 16
            || !corpus.seed.bytes().all(|byte| byte.is_ascii_hexdigit())
            || corpus.value_domain != "u64"
            || corpus.exhaustive.case_count != exhaustive_cases
            || corpus.property.shrink_result != "not-applicable-passed"
            || corpus.case_count != expected_cases
            || corpus.oracle.is_empty()
            || functions != original
            || theorem_count != corpus.oracle.len()
            || self.runtime_roots.spec != "prismpm/runtime-roots/1"
            || self.runtime_roots.lean_module != "PrismPM.Foundation.Holo"
            || self.runtime_roots.ir_module != "PrismPM"
            || self
                .runtime_roots
                .roots
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || runtime_roots.len() != self.runtime_roots.roots.len()
            || runtime_functions != runtime_roots
            || corpus.oracle.iter().any(|row| {
                !row.function.starts_with("PrismPM.Foundation.Holo.")
                    || !row.theorem.starts_with("PrismPM.Foundation.Holo.")
            })
        {
            return Err(bad(
                "execution corpus is not canonical or internally consistent",
            ));
        }
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
                    return Err(bad(format!(
                        "{}: authority realized_by unknown ID {id}",
                        a.id
                    )));
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
