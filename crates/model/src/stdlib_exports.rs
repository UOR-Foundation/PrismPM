//! The frozen 0.1.x application API preserved by the standard-library package.

use crate::ModelError;
use serde::Deserialize;
use std::collections::BTreeSet;

/// Extra named package exports, distinct from allocation-free validator roots.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdlibExports {
    /// Closed package-export register schema.
    pub spec: String,
    /// Generated Lean import module containing every named entry point.
    pub lean_module: String,
    /// Name of the single verified LCNF module.
    pub ir_module: String,
    /// Canonically ordered, explicitly preserved legacy entry points.
    pub export: Vec<StdlibExport>,
}

/// One source declaration and its original public Rust function type.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdlibExport {
    /// Fully qualified generated Lean declaration, never handwritten Lean.
    pub lean_name: String,
    /// Original public Rust symbol.
    pub rust_name: String,
    /// Consumer-facing function type; this describes an ABI, not semantics.
    pub rust_signature: String,
}

impl StdlibExports {
    /// Reject missing, duplicated, substituted, or incompatible legacy exports.
    pub fn check(&self) -> Result<(), ModelError> {
        let invalid =
            || ModelError::Inconsistent("invalid standard-library package exports".to_owned());
        if self.spec != "prismpm/stdlib-exports/1"
            || self.lean_module != "PrismPM.Runtime"
            || self.ir_module != "PrismPM"
            || self.export.len() != 20
            || self
                .export
                .windows(2)
                .any(|rows| rows[0].lean_name >= rows[1].lean_name)
        {
            return Err(invalid());
        }
        let mut names = BTreeSet::new();
        for row in &self.export {
            // The compatibility policy is frozen for the 0.2.0 package. It
            // cannot be waived by deleting or renaming a register row.
            let (module, signature) = match row.rust_name.as_str() {
                "appendBytes" => ("Bytes", "fn(Vec<u8>, Vec<u8>) -> Vec<u8>"),
                "byteAt" => ("Bytes", "fn(Vec<u8>, u64) -> Option<u8>"),
                "byteLength" => ("Bytes", "fn(Vec<u8>) -> u64"),
                "compareBytes" => ("Bytes", "fn(Vec<u8>, Vec<u8>) -> core::cmp::Ordering"),
                "sliceBytes" => ("Bytes", "fn(Vec<u8>, u64, u64) -> Option<Vec<u8>>"),
                "formatInt64" => ("Codec", "fn(i64) -> String"),
                "parseInt64" => ("Codec", "fn(String) -> Option<i64>"),
                "portableTrue" => ("Core", "fn() -> bool"),
                "applicationSecurityEdition"
                | "architectureEdition"
                | "controlEdition"
                | "qualityEdition"
                | "riskEdition" => ("Holo.StandardsProfile", "fn(StandardsProfile) -> u64"),
                "checkedAddInt64"
                | "checkedDivideInt64"
                | "checkedMultiplyInt64"
                | "checkedSubtractInt64" => ("Integer", "fn(i64, i64) -> Option<i64>"),
                "checkedNegateInt64" => ("Integer", "fn(i64) -> Option<i64>"),
                "decode" => ("Utf8", "fn(Vec<u8>) -> Option<String>"),
                "encode" => ("Utf8", "fn(String) -> Vec<u8>"),
                _ => return Err(invalid()),
            };
            if row.lean_name != format!("PrismPM.Foundation.{module}.{}", row.rust_name)
                || row.rust_signature != signature
                || !names.insert(&row.rust_name)
            {
                return Err(invalid());
            }
        }
        Ok(())
    }

    /// Canonical complete native export request; validator accounting is unchanged.
    pub fn union_with_runtime(&self, runtime_roots: &[String]) -> Result<Vec<String>, ModelError> {
        self.check()?;
        Ok(runtime_roots
            .iter()
            .cloned()
            .chain(self.export.iter().map(|row| row.lean_name.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("../../../model/stdlib-exports.toml");

    #[test]
    fn stdlib_exports_preserve_every_original_entry_point() {
        let exports: StdlibExports = toml::from_str(SOURCE).unwrap();
        exports.check().unwrap();
        for index in 0..exports.export.len() {
            let mut missing = exports.clone();
            missing.export.remove(index);
            assert!(
                missing.check().is_err(),
                "accepted omitted legacy export {index}"
            );
        }
        for mutation in 0..5 {
            let mut changed = exports.clone();
            match mutation {
                0 => changed.export[0].rust_signature = "fn(&[u8], &[u8]) -> Vec<u8>".to_owned(),
                1 => {
                    changed.export[0].lean_name = "PrismPM.Foundation.Core.portableTrue".to_owned()
                }
                2 => changed.export[0].rust_name = "replacement".to_owned(),
                3 => changed.export[1] = changed.export[0].clone(),
                4 => changed.export.swap(0, 1),
                _ => unreachable!(),
            }
            assert!(changed.check().is_err(), "accepted mutation {mutation}");
        }
    }

    #[test]
    fn stdlib_exports_are_closed_and_separate_from_validator_roots() {
        let exports: StdlibExports = toml::from_str(SOURCE).unwrap();
        let runtime = vec!["PrismPM.Foundation.Holo.validateComponentIndexes".to_owned()];
        let union = exports.union_with_runtime(&runtime).unwrap();
        assert_eq!(union.len(), 21);
        assert_eq!(runtime.len(), 1);
        assert!(union.contains(&runtime[0]));
        let mut document: toml::Value = toml::from_str(SOURCE).unwrap();
        document
            .as_table_mut()
            .unwrap()
            .insert("unexpected".to_owned(), true.into());
        assert!(document.try_into::<StdlibExports>().is_err());
        let mut document: toml::Value = toml::from_str(SOURCE).unwrap();
        document["export"][0]
            .as_table_mut()
            .unwrap()
            .insert("unchecked".to_owned(), true.into());
        assert!(document.try_into::<StdlibExports>().is_err());
    }
}
