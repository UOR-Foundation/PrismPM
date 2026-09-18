//! Closed View namespace, independent of Journal and Command/Query registers.

use crate::{BrowserDiagnostic, BrowserModelRejection, ModelError};
use serde::Deserialize;

/// Model-owned private View host diagnostics and generated rejection details.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserViewDiagnostics {
    /// Exact separate registry version.
    pub spec: String,
    /// Owning JavaScript error class.
    pub error_class: String,
    /// Canonical complete SDK source closure for this namespace.
    pub source: Vec<String>,
    /// Owning complete browser conformance capability.
    pub capability: String,
    /// Authoritative generated View rejection model.
    pub model_source: String,
    /// Host code whose detail is exactly a modeled rejection byte.
    pub rejection_error: String,
    /// Complete sorted host diagnostic codes.
    pub error: Vec<BrowserDiagnostic>,
    /// Complete declaration-order generated rejection table.
    pub rejection: Vec<BrowserModelRejection>,
}

impl BrowserViewDiagnostics {
    /// Reject omitted, duplicated, reordered, substituted or malformed entries.
    pub fn check(&self) -> Result<(), ModelError> {
        const CODES: [&str; 11] = [
            "generated-execution-failed",
            "host-unavailable",
            "invalid-effect-result",
            "invalid-generated-module",
            "invalid-generated-output",
            "invalid-input",
            "invalid-labels",
            "invalid-root",
            "model-rejected",
            "request-limit",
            "view-closed",
        ];
        const REJECTIONS: [&str; 14] = [
            "BadEncoding",
            "InvalidState",
            "InvalidIntent",
            "Busy",
            "ReplayRequired",
            "Closed",
            "CounterExhausted",
            "NoSelection",
            "NoNextPage",
            "WrongPhase",
            "CorrelationMismatch",
            "InvalidOutcome",
            "InvalidPage",
            "UnknownOperation",
        ];
        if self.spec != "prismpm/browser-view-diagnostics/1"
            || self.error_class != "ViewHostError"
            || self.source
                != [
                    "sdk/browser/view-dom.mjs",
                    "sdk/browser/view-error.mjs",
                    "sdk/browser/view-host.mjs",
                ]
            || self.capability != "DK-16"
            || self.model_source != "stdlib/src/Foundation/View/Workspace/V1/Interaction.lex.tex"
            || self.rejection_error != "model-rejected"
            || self.error.iter().map(|row| row.code.as_str()).ne(CODES)
            || self.error.iter().any(|row| {
                row.statement.trim().is_empty() || row.statement.contains(['\n', '\r', '|'])
            })
            || self.rejection.len() != REJECTIONS.len()
            || self
                .rejection
                .iter()
                .zip(REJECTIONS)
                .enumerate()
                .any(|(index, (row, name))| usize::from(row.byte) != index + 1 || row.name != name)
        {
            return Err(ModelError::Inconsistent(
                "invalid browser View diagnostic register".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BrowserViewDiagnostics;

    #[test]
    fn browser_view_namespace_and_model_rejections_are_closed() {
        let source = include_str!("../../../model/browser-view-diagnostics.toml");
        let valid: BrowserViewDiagnostics = toml::from_str(source).unwrap();
        valid.check().unwrap();
        for mutation in 0..13 {
            let mut changed = valid.clone();
            match mutation {
                0 => changed.spec = "prismpm/browser-adapter-diagnostics/1".to_owned(),
                1 => changed.error_class = "JournalAdapterError".to_owned(),
                2 => changed.source.reverse(),
                3 => {
                    changed.source.pop();
                }
                4 => changed.capability = "DK-14".to_owned(),
                5 => changed.model_source = "src/Authority.lex.tex".to_owned(),
                6 => changed.rejection_error = "command-rejected".to_owned(),
                7 => {
                    changed.error.pop();
                }
                8 => changed.error.push(changed.error[0].clone()),
                9 => changed.error[0].statement.clear(),
                10 => changed.rejection[0].byte = 0,
                11 => changed.rejection[0].name = "Authenticated".to_owned(),
                _ => {
                    changed.rejection.pop();
                }
            }
            assert!(changed.check().is_err(), "accepted mutation {mutation}");
        }
        assert!(toml::from_str::<BrowserViewDiagnostics>(
            &source.replace("spec =", "unknown = true\nspec =")
        )
        .is_err());
    }
}
