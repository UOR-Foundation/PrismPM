//! Browser-local diagnostic names, distinct from CLI PP codes.

use crate::ModelError;
use serde::Deserialize;

/// Closed diagnostics owned by the private browser journal adapter.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserDiagnostics {
    /// Registry version.
    pub spec: String,
    /// Exact owning SDK source path.
    pub source: String,
    /// Owning JavaScript error class.
    pub error_class: String,
    /// Complete owning conformance gate.
    pub capability: String,
    /// Canonically ordered diagnostic names and meanings.
    pub error: Vec<BrowserDiagnostic>,
}

/// One journal-origin browser failure; propagated host errors stay separate.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserDiagnostic {
    /// Exact code carried by the owning error class.
    pub code: String,
    /// Normative failure meaning, without payload or secret material.
    pub statement: String,
}

impl BrowserDiagnostics {
    /// Reject omitted, duplicate, substituted or malformed diagnostic rows.
    pub fn check(&self) -> Result<(), ModelError> {
        const CODES: [&str; 18] = [
            "author-mismatch",
            "event-id-mismatch",
            "generated-execution-failed",
            "invalid-generated-module",
            "invalid-generated-output",
            "invalid-hash",
            "invalid-input",
            "invalid-length",
            "journal-busy",
            "model-rejected",
            "object-corrupt",
            "object-missing",
            "replay-required",
            "request-limit",
            "signature-invalid",
            "storage-outcome-unknown",
            "storage-rejected",
            "storage-result-mismatch",
        ];
        if self.spec != "prismpm/browser-diagnostics/1"
            || self.source != "sdk/browser/journal.mjs"
            || self.error_class != "JournalAdapterError"
            || self.capability != "DK-12"
            || self.error.iter().map(|row| row.code.as_str()).ne(CODES)
            || self.error.iter().any(|row| {
                row.statement.trim().is_empty() || row.statement.contains(['\n', '\r', '|'])
            })
        {
            return Err(ModelError::Inconsistent(
                "invalid browser-local diagnostic register".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BrowserDiagnostics;

    #[test]
    fn browser_diagnostics_reject_incomplete_or_substituted_registrations() {
        let source = include_str!("../../../model/browser-diagnostics.toml");
        let valid: BrowserDiagnostics = toml::from_str(source).unwrap();
        valid.check().unwrap();
        for mutation in 0..7 {
            let mut changed = valid.clone();
            match mutation {
                0 => {
                    changed.error.pop();
                }
                1 => changed.error.push(changed.error[0].clone()),
                2 => changed.error[0].code = "unregistered".to_owned(),
                3 => changed.error[0].statement.clear(),
                4 => changed.source = "sdk/browser/identity.mjs".to_owned(),
                5 => changed.capability = "DK-07".to_owned(),
                _ => changed.error.retain(|row| row.code != "journal-busy"),
            }
            assert!(changed.check().is_err());
        }
        assert!(toml::from_str::<BrowserDiagnostics>(
            &source.replace("spec =", "undeclared = true\nspec =")
        )
        .is_err());
    }
}
