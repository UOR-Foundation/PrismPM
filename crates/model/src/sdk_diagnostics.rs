//! Codes of the Prism SDK modules (#440), distinct from CLI PP codes.

use crate::ModelError;
use serde::Deserialize;

/// Open register of the codes the SDK modules raise; each module adds its rows.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SdkDiagnostics {
    /// Registry version.
    pub spec: String,
    /// Canonically ordered codes and meanings.
    pub error: Vec<SdkDiagnostic>,
}

/// One code an SDK module raises.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SdkDiagnostic {
    /// Exact code, `<module>.<name>`, carried by the error.
    pub code: String,
    /// Owning conformance id.
    pub capability: String,
    /// Normative failure meaning.
    pub statement: String,
}

impl SdkDiagnostics {
    /// Reject duplicate, unordered or malformed rows and an owner `ids` does not hold.
    pub fn check(&self, is_registered: impl Fn(&str) -> bool) -> Result<(), ModelError> {
        if self.spec != "prismpm/sdk-diagnostics/1"
            || !self
                .error
                .windows(2)
                .all(|rows| rows[0].code < rows[1].code)
        {
            return Err(ModelError::Inconsistent(
                "SDK diagnostics must have the exact spec and sorted, unique codes".to_string(),
            ));
        }
        for row in &self.error {
            let well_formed = row.code.split_once('.').is_some_and(|(module, name)| {
                let word = |text: &str| {
                    !text.is_empty() && text.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
                };
                word(module) && word(name)
            });
            if !well_formed || row.statement.trim().is_empty() || !is_registered(&row.capability) {
                return Err(ModelError::Inconsistent(format!(
                    "SDK diagnostic {} needs a <module>.<name> code, a statement and a registered capability",
                    row.code
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn register(codes: &[&str]) -> SdkDiagnostics {
        SdkDiagnostics {
            spec: "prismpm/sdk-diagnostics/1".to_string(),
            error: codes
                .iter()
                .map(|code| SdkDiagnostic {
                    code: (*code).to_string(),
                    capability: "DK-39".to_string(),
                    statement: "A meaning.".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn check_rejects_disorder_malformed_codes_and_unknown_owners() {
        let known = |id: &str| id == "DK-39";
        assert!(register(&["networks.exists", "networks.store"])
            .check(known)
            .is_ok());
        for codes in [
            ["networks.store", "networks.exists"],
            ["networks.exists", "networks.exists"],
        ] {
            assert!(register(&codes).check(known).is_err());
        }
        for code in [
            "PP1001",
            "networks",
            "networks.",
            ".exists",
            "Networks.exists",
        ] {
            assert!(register(&[code]).check(known).is_err(), "{code}");
        }
        assert!(register(&["networks.exists"]).check(|_| false).is_err());
        let mut wrong = register(&["networks.exists"]);
        wrong.spec = "prismpm/sdk-diagnostics/2".to_string();
        assert!(wrong.check(known).is_err());
    }
}
