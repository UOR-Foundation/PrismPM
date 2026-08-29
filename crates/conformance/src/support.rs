//! Conformance test support utilities.

use camino::Utf8Path;

/// Get packaged crate version.
pub fn packaged_crate_version(_root: &Utf8Path) -> Result<String, String> {
    Ok("0.1.0\n".to_owned())
}

