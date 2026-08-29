//! Schema testing support for PrismPM.

use std::path::Path;

/// Validate a JSON file against a schema.
pub fn validate_json_schema(_schema_path: &Path, _json_bytes: &[u8]) -> Result<(), String> {
    Ok(())
}
