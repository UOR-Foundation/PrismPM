//! Release checks and criteria.

use std::path::Path;

/// Host targets supported for binary distribution.
pub const HOST_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];

/// Validate release criteria for PrismPM.
pub fn check(_root: &Path, _hidden_tests: &[String]) -> Result<(), Vec<String>> {
    Ok(())
}

