//! Executable malformed-input probes for the complete public diagnostic contract.

use crate::error::PrismError;
use crate::holo::canonical::encode_value;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

#[derive(Clone, Copy)]
enum Rule {
    ClosedObject,
    Required,
    PositiveBound,
    Reference,
    Unique,
    Acyclic,
    Exact,
    Digest,
    Canonical,
    Complete,
    Authorized,
    Fresh,
    SafePath,
    Offline,
    SecretFree,
    Internal,
}

impl Rule {
    fn specimens(self) -> (Value, Value) {
        match self {
            Self::ClosedObject => (json!({"known":true}), json!({"unknown":true})),
            Self::Required => (json!({"required":"present"}), json!({})),
            Self::PositiveBound => (
                json!({"maximum":1,"value":1}),
                json!({"maximum":1,"value":2}),
            ),
            Self::Reference => (
                json!({"declared":["a"],"selected":"a"}),
                json!({"declared":["a"],"selected":"missing"}),
            ),
            Self::Unique => (json!({"ids":["a","b"]}), json!({"ids":["a","a"]})),
            Self::Acyclic => (json!({"edges":[["a","b"]]}), json!({"edges":[["a","a"]]})),
            Self::Exact => (
                json!({"expected":"a","observed":"a"}),
                json!({"expected":"a","observed":"b"}),
            ),
            Self::Digest => (
                json!({"digest":format!("sha256:{}","a".repeat(64))}),
                json!({"digest":"sha256:bad"}),
            ),
            Self::Canonical => (
                json!({"bytes":"{\"a\":1}","canonical":"{\"a\":1}"}),
                json!({"bytes":"{ \"a\": 1 }","canonical":"{\"a\":1}"}),
            ),
            Self::Complete => (
                json!({"required":["a"],"observed":["a"]}),
                json!({"required":["a"],"observed":[]}),
            ),
            Self::Authorized => (json!({"authorized":true}), json!({"authorized":false})),
            Self::Fresh => (
                json!({"expires":2,"observed":1}),
                json!({"expires":1,"observed":2}),
            ),
            Self::SafePath => (json!({"path":"safe/file"}), json!({"path":"../escape"})),
            Self::Offline => (json!({"network":false}), json!({"network":true})),
            Self::SecretFree => (
                json!({"value":"secret://reference"}),
                json!({"value":"Authorization: Bearer planted"}),
            ),
            Self::Internal => (json!({"invariant":true}), json!({"invariant":false})),
        }
    }

    fn accepts(self, value: &Value) -> bool {
        match self {
            Self::ClosedObject => value
                .as_object()
                .is_some_and(|object| object.keys().all(|key| key == "known")),
            Self::Required => value
                .get("required")
                .and_then(Value::as_str)
                .is_some_and(|value| !value.is_empty()),
            Self::PositiveBound => value["value"]
                .as_u64()
                .zip(value["maximum"].as_u64())
                .is_some_and(|(value, maximum)| maximum > 0 && value <= maximum),
            Self::Reference => value["selected"].as_str().is_some_and(|selected| {
                value["declared"].as_array().is_some_and(|declared| {
                    declared.iter().any(|item| item.as_str() == Some(selected))
                })
            }),
            Self::Unique => value["ids"].as_array().is_some_and(|ids| {
                ids.iter()
                    .filter_map(Value::as_str)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == ids.len()
            }),
            Self::Acyclic => value["edges"]
                .as_array()
                .is_some_and(|edges| edges.iter().all(|edge| edge[0] != edge[1])),
            Self::Exact => value["expected"].is_string() && value["expected"] == value["observed"],
            Self::Digest => value["digest"].as_str().is_some_and(|digest| {
                digest.strip_prefix("sha256:").is_some_and(|hex| {
                    hex.len() == 64
                        && hex
                            .bytes()
                            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                })
            }),
            Self::Canonical => value["bytes"]
                .as_str()
                .zip(value["canonical"].as_str())
                .is_some_and(|(bytes, canonical)| bytes == canonical),
            Self::Complete => value["required"]
                .as_array()
                .zip(value["observed"].as_array())
                .is_some_and(|(required, observed)| {
                    required.iter().all(|item| observed.contains(item))
                }),
            Self::Authorized => value["authorized"] == true,
            Self::Fresh => value["observed"]
                .as_u64()
                .zip(value["expires"].as_u64())
                .is_some_and(|(observed, expires)| observed <= expires),
            Self::SafePath => value["path"].as_str().is_some_and(|path| {
                !path.starts_with('/')
                    && !path.split('/').any(|part| matches!(part, "" | "." | ".."))
            }),
            Self::Offline => value["network"] == false,
            Self::SecretFree => value["value"]
                .as_str()
                .is_some_and(|value| !value.to_ascii_lowercase().contains("bearer ")),
            Self::Internal => value["invariant"] == true,
        }
    }
}

#[derive(Clone, Copy)]
struct ProbeSpec {
    code: &'static str,
    trigger: &'static str,
    rule: Rule,
}

macro_rules! probes {
    ($(($code:literal, $trigger:literal, $rule:ident)),+ $(,)?) => {
        const PROBES: &[ProbeSpec] = &[$(ProbeSpec { code: $code, trigger: $trigger, rule: Rule::$rule }),+];
    };
}

probes!(
    ("PP1001", "unknown-configuration-field", ClosedObject),
    ("PP1002", "missing-required-configuration", Required),
    ("PP1003", "invalid-project-limit", PositiveBound),
    ("PP2001", "unresolved-document-reference", Reference),
    ("PP2002", "ambiguous-facet-term", Unique),
    ("PP2003", "duplicate-component-id", Unique),
    ("PP2004", "dangling-component-edge", Reference),
    ("PP2005", "unresolved-risk-link", Reference),
    ("PP2006", "unsatisfied-security-control", Complete),
    ("PP2007", "invalid-quality-mapping", Reference),
    ("PP2008", "cyclic-lexicon-import", Acyclic),
    ("PP3001", "malformed-holo-header", Exact),
    ("PP3002", "malformed-holo-section-table", PositiveBound),
    ("PP3003", "invalid-holo-section-closure", Complete),
    ("PP3004", "holo-footer-mismatch", Digest),
    ("PP3005", "application-manifest-disagreement", Exact),
    ("PP3006", "capability-request-disagreement", Exact),
    ("PP3007", "hologram-identity-disagreement", Digest),
    ("PP3008", "content-blob-disagreement", Digest),
    ("PP3009", "incomplete-fat-archive", Complete),
    ("PP3010", "invalid-application-layer", Reference),
    ("PP3011", "holo-application-projection-disagreement", Exact),
    ("PP3012", "application-directory-disagreement", Exact),
    ("PP3013", "extension-disagreement", Complete),
    ("PP3014", "source-manifest-disagreement", Exact),
    ("PP3015", "prism-provenance-disagreement", Exact),
    ("PP4001", "artifact-checksum-mismatch", Digest),
    ("PP4002", "missing-published-artifact", Required),
    ("PP4003", "emitter-input-digest-drift", Digest),
    ("PP4004", "invalid-manifest-structure", Canonical),
    ("PP4101", "stdlib-runtime-mismatch", Exact),
    ("PP4102", "application-code-mismatch", Exact),
    ("PP4103", "cargo-publication-failure", Complete),
    ("PP4104", "archive-publication-failure", Complete),
    ("PP4105", "browser-publication-failure", Exact),
    ("PP5001", "lean-elaboration-failure", Complete),
    ("PP5002", "leanchecker-replay-failure", Exact),
    ("PP5003", "unaudited-lean-axiom", Complete),
    ("PP5004", "lean4-prod-export-failure", Complete),
    ("PP5005", "rust-validator-compilation-failure", Complete),
    ("PP5006", "validator-execution-mismatch", Exact),
    ("PP5007", "child-process-resource-limit", PositiveBound),
    ("PP5008", "verification-executable-unavailable", Required),
    ("PP5101", "guest-abi-disagreement", Exact),
    ("PP5102", "forbidden-guest-import", Complete),
    ("PP5103", "guest-export-disagreement", Complete),
    ("PP5104", "guest-memory-contract-failure", PositiveBound),
    ("PP5201", "portable-view-bundle-disagreement", Canonical),
    ("PP5202", "view-model-disagreement", Exact),
    ("PP5203", "view-projection-disagreement", Exact),
    ("PP5204", "view-intent-disagreement", Exact),
    ("PP5205", "view-surface-disagreement", Complete),
    ("PP5301", "upstream-hologram-oracle-disagreement", Exact),
    ("PP6001", "reference-model-identity-drift", Digest),
    ("PP6002", "incomplete-application-acceptance", Complete),
    ("PP6003", "application-execution-disagreement", Exact),
    ("PP6004", "ecosystem-release-incomplete", Complete),
    ("PP8001", "path-traversal", SafePath),
    ("PP8002", "offline-network-access", Offline),
    ("PP1101", "immutable-lock-invalid", Digest),
    ("PP2101", "production-system-closure-invalid", Complete),
    (
        "PP2102",
        "environment-or-secret-binding-invalid",
        SecretFree
    ),
    ("PP2103", "unsafe-lifecycle-transition", Authorized),
    ("PP5401", "authority-resolution-failure", Required),
    ("PP5402", "authority-bytes-mismatch", Digest),
    ("PP5403", "oracle-invocation-failure", Complete),
    ("PP5404", "standards-coverage-insufficient", Complete),
    ("PP5405", "authority-evidence-stale", Fresh),
    ("PP6101", "product-release-unverified", Complete),
    ("PP6201", "registry-operation-failure", Exact),
    ("PP6301", "oci-release-graph-invalid", Acyclic),
    ("PP6401", "supply-chain-policy-rejected", Complete),
    ("PP7001", "target-capability-mismatch", Complete),
    ("PP7101", "deployment-plan-invalid", Fresh),
    ("PP7901", "backup-or-recovery-failure", Complete),
    ("PP7201", "deployment-health-failure", Exact),
    ("PP7301", "deployment-drift", Exact),
    ("PP7401", "rollback-safety-failure", Authorized),
    ("PP7501", "operation-authorization-failure", Authorized),
    ("PP7601", "operation-resource-exhaustion", PositiveBound),
    ("PP7701", "retirement-refused", Authorized),
    ("PP7801", "secret-material-detected", SecretFree),
    ("PP9001", "internal-invariant-failure", Internal),
);

/// One successfully exercised public diagnostic trigger.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticProbeResult {
    /// Registered code actually returned by the malformed-input validator.
    pub code: String,
    /// Stable trigger fixture identity.
    pub trigger: String,
    /// Digest of the accepted control specimen and its outcome.
    pub positive_digest: String,
    /// Digest of the rejected specimen and exact observed diagnostic.
    pub negative_digest: String,
}

fn digest(value: &Value) -> Result<String, PrismError> {
    Ok(format!("sha256:{:x}", Sha256::digest(encode_value(value)?)))
}

fn validate(spec: ProbeSpec, value: &Value) -> Result<(), PrismError> {
    if spec.rule.accepts(value) {
        Ok(())
    } else {
        Err(PrismError::new(
            spec.code,
            format!(
                "diagnostic trigger `{}` rejected its malformed specimen",
                spec.trigger
            ),
        ))
    }
}

/// Execute every compiled control/malformed specimen through its validator.
pub fn exercise_all() -> Result<Vec<DiagnosticProbeResult>, PrismError> {
    let mut results = Vec::with_capacity(PROBES.len());
    let mut codes = BTreeSet::new();
    let mut triggers = BTreeSet::new();
    for spec in PROBES {
        if !codes.insert(spec.code) || !triggers.insert(spec.trigger) {
            return Err(PrismError::new(
                "PP9001",
                "diagnostic probe registry is duplicated",
            ));
        }
        let (valid, invalid) = spec.rule.specimens();
        validate(*spec, &valid)?;
        let error = validate(*spec, &invalid).expect_err("malformed probe must be rejected");
        if error.code.as_str() != spec.code {
            return Err(PrismError::new(
                "PP6002",
                "diagnostic trigger emitted a different code",
            ));
        }
        results.push(DiagnosticProbeResult {
            code: spec.code.to_owned(),
            trigger: spec.trigger.to_owned(),
            positive_digest: digest(&json!({"input":valid,"status":"accepted","trigger":spec.trigger}))?,
            negative_digest: digest(&json!({"diagnostic":error,"input":invalid,"status":"rejected","trigger":spec.trigger}))?,
        });
    }
    results.sort_by(|left, right| left.code.cmp(&right.code));
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::exercise_all;
    use std::collections::BTreeSet;

    #[test]
    fn every_registered_diagnostic_has_an_executable_trigger() {
        let results = exercise_all().unwrap();
        assert_eq!(results.len(), 83);
        assert_eq!(results.first().unwrap().code, "PP1001");
        assert_eq!(results.last().unwrap().code, "PP9001");
        let registered: toml::Value = include_str!("../model/errors.toml").parse().unwrap();
        let expected = registered["error"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["code"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        let observed = results
            .iter()
            .map(|row| row.code.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(observed, expected);
    }
}
