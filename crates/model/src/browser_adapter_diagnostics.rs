//! Closed Command and Query namespaces, separate from Journal's v1 register.

use crate::{BrowserDiagnostic, ModelError};
use serde::Deserialize;

/// Model-owned browser command and admitted-query diagnostic registers.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserAdapterDiagnostics {
    /// Exact registry version; never substitutes the Journal v1 contract.
    pub spec: String,
    /// Exactly the Command and Query namespaces in canonical order.
    pub adapter: Vec<BrowserAdapterDiagnostic>,
}

/// One owning host error class and its generated rejection detail table.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserAdapterDiagnostic {
    /// Owning JavaScript class.
    pub error_class: String,
    /// Exact SDK module path.
    pub source: String,
    /// Complete owning gate.
    pub capability: String,
    /// Exact authoritative LexLean source, not an authentication assertion.
    pub model_source: String,
    /// Host code carrying one generated rejection byte as its detail.
    pub rejection_error: String,
    /// Complete sorted host codes; inherited namespaces are not relabeled.
    pub error: Vec<BrowserDiagnostic>,
    /// Exact declaration-order mapping of generated rejection bytes.
    pub rejection: Vec<BrowserModelRejection>,
}

/// One generated rejection, distinct from a host or propagated failure.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowserModelRejection {
    /// Closed one-byte detail value.
    pub byte: u8,
    /// Exact modeled constructor name.
    pub name: String,
}

impl BrowserAdapterDiagnostics {
    /// Reject omitted, duplicate, relabeled, reordered, or malformed entries.
    pub fn check(&self) -> Result<(), ModelError> {
        const COMMAND_CODES: [&str; 13] = [
            "adapter-busy",
            "adapter-closed",
            "command-rejected",
            "commit-outcome-unknown",
            "generated-execution-failed",
            "identity-missing",
            "invalid-effect-result",
            "invalid-generated-module",
            "invalid-generated-output",
            "invalid-input",
            "refresh-required",
            "request-limit",
            "storage-outcome-unknown",
        ];
        const QUERY_CODES: [&str; 12] = [
            "adapter-busy",
            "adapter-closed",
            "generated-execution-failed",
            "identity-missing",
            "invalid-effect-result",
            "invalid-generated-module",
            "invalid-generated-output",
            "invalid-input",
            "query-context-changed",
            "query-rejected",
            "request-limit",
            "storage-outcome-unknown",
        ];
        const COMMAND_REJECTIONS: [&str; 12] = [
            "BadEncoding",
            "InvalidCommand",
            "InvalidHead",
            "WorkspaceMismatch",
            "EventLimit",
            "WrongPhase",
            "CorrelationMismatch",
            "StaleHead",
            "EffectMismatch",
            "InvalidResult",
            "UnknownOperation",
            "InvalidPending",
        ];
        const QUERY_REJECTIONS: [&str; 13] = [
            "BadEncoding",
            "InvalidHead",
            "InvalidState",
            "InvalidContext",
            "WrongWorkspace",
            "NotAdmitted",
            "InvalidCursor",
            "StaleCursor",
            "CursorSession",
            "CursorPrincipal",
            "CursorTable",
            "CursorRange",
            "UnknownTable",
        ];
        let invalid =
            || ModelError::Inconsistent("invalid browser adapter diagnostic register".to_owned());
        if self.spec != "prismpm/browser-adapter-diagnostics/1" || self.adapter.len() != 2 {
            return Err(invalid());
        }
        for (index, adapter) in self.adapter.iter().enumerate() {
            let (kind, file, capability, code, codes, rejections): (_, _, _, _, &[&str], &[&str]) =
                if index == 0 {
                    (
                        "Command",
                        "commands",
                        "DK-13",
                        "command-rejected",
                        &COMMAND_CODES,
                        &COMMAND_REJECTIONS,
                    )
                } else {
                    (
                        "Query",
                        "queries",
                        "DK-14",
                        "query-rejected",
                        &QUERY_CODES,
                        &QUERY_REJECTIONS,
                    )
                };
            if adapter.error_class != format!("{kind}AdapterError")
                || adapter.source != format!("sdk/browser/{file}.mjs")
                || adapter.capability != capability
                || adapter.model_source
                    != format!("stdlib/src/Foundation/Browser/V1/Workspace{kind}.lex.tex")
                || adapter.rejection_error != code
                || adapter
                    .error
                    .iter()
                    .map(|row| row.code.as_str())
                    .ne(codes.iter().copied())
                || adapter.error.iter().any(|row| {
                    row.statement.trim().is_empty() || row.statement.contains(['\n', '\r', '|'])
                })
                || adapter.rejection.len() != rejections.len()
                || adapter.rejection.iter().zip(rejections).enumerate().any(
                    |(index, (row, name))| usize::from(row.byte) != index + 1 || row.name != *name,
                )
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BrowserAdapterDiagnostics;

    #[test]
    fn browser_adapter_namespaces_and_model_rejections_are_closed() {
        let source = include_str!("../../../model/browser-adapter-diagnostics.toml");
        let valid: BrowserAdapterDiagnostics = toml::from_str(source).unwrap();
        valid.check().unwrap();
        for index in 0..2 {
            for mutation in 0..10 {
                let mut changed = valid.clone();
                let adapter = &mut changed.adapter[index];
                match mutation {
                    0 => {
                        adapter.error.pop();
                    }
                    1 => adapter.error.push(adapter.error[0].clone()),
                    2 => adapter.error[0].code = "journal-busy".to_owned(),
                    3 => adapter.error[0].statement.clear(),
                    4 => adapter.error_class = "JournalAdapterError".to_owned(),
                    5 => adapter.source = "sdk/browser/journal.mjs".to_owned(),
                    6 => adapter.capability = "DK-12".to_owned(),
                    7 => {
                        adapter.rejection.pop();
                    }
                    8 => adapter.rejection[0].byte = 0,
                    _ => adapter.rejection[0].name = "Authenticated".to_owned(),
                }
                assert!(
                    changed.check().is_err(),
                    "accepted adapter {index} mutation {mutation}"
                );
            }
        }
        let mut missing = valid.clone();
        missing.adapter.pop();
        assert!(missing.check().is_err());
        let mut reordered = valid;
        reordered.adapter.swap(0, 1);
        assert!(reordered.check().is_err());
        assert!(toml::from_str::<BrowserAdapterDiagnostics>(
            &source.replace("spec =", "unknown = true\nspec =")
        )
        .is_err());
    }
}
