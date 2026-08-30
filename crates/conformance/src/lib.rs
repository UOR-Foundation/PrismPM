//! Conformance testing harness and validation runners.

#![deny(missing_docs)]
#![forbid(unsafe_code)]
// Conformance deliberately exercises the complete public PrismError value;
// boxing it here would test a different API from downstream callers.
#![allow(clippy::result_large_err)]

pub mod cases;
pub mod fixtures;
pub mod meta;
pub mod runner;
pub mod schema;
pub mod support;

pub use meta::check_honesty;

use std::collections::BTreeSet;
use std::path::Path;

/// Scan workspace test names.
#[must_use]
pub fn workspace_test_names(root: &Path) -> BTreeSet<String> {
    let (names, _flagged) = workspace_test_names_with_flags(root);
    names
}

/// Scan test names and check flags independently from test source files.
#[must_use]
pub fn workspace_test_names_with_flags(root: &Path) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut names = BTreeSet::new();
    let mut flagged = BTreeSet::new();

    let tests_file = root.join("crates/conformance/tests/conformance.rs");
    if let Ok(content) = std::fs::read_to_string(&tests_file) {
        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("test_case!(") {
                if let Some(ident) = rest.split(',').next() {
                    let ident = ident.trim();
                    if !ident.is_empty() {
                        names.insert(ident.to_owned());
                    }
                }
            } else if let Some(rest) = line.strip_prefix("fn ") {
                if let Some(ident) = rest.split('(').next() {
                    let ident = ident.trim();
                    if ident.starts_with("conformance_") {
                        names.insert(ident.to_owned());
                    }
                }
            }
        }
    }

    for name in &names {
        if name.contains("ignored") || name.contains("pending") {
            flagged.insert(name.clone());
        }
    }

    (names, flagged)
}
