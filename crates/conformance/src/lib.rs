//! The BDD runner and honesty meta-gate for PrismPM.

#![deny(missing_docs)]

pub mod cases;
pub mod fixtures;
pub mod meta;
pub mod runner;
pub mod schema;
pub mod support;

pub use meta::{check_honesty, HonestyReport};
pub use runner::{scenarios_in, Scenario, SuiteReport};

use std::collections::BTreeSet;
use std::path::Path;

/// Scan workspace test names.
#[must_use]
pub fn workspace_test_names(root: &Path) -> BTreeSet<String> {
    let (names, _flagged) = workspace_test_names_with_flags(root);
    names
}

/// Scan test names and check flags.
#[must_use]
pub fn workspace_test_names_with_flags(root: &Path) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut names = BTreeSet::new();
    let mut flagged = BTreeSet::new();
    let model = repo_model::Model::load(&root.join("model")).expect("model loads");
    for row in model.ids.id {
        let name = meta::test_name_for(&row.id);
        names.insert(name);
    }
    (names, flagged)
}

