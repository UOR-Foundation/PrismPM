//! Deterministic operational checks and redaction for observed evidence.

use crate::error::PrismError;
use crate::holo::canonical::encode_value;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn names(model: &Value, field: &str) -> Result<BTreeSet<String>, PrismError> {
    model[field]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7401", format!("system {field} is absent")))?
        .iter()
        .map(|row| {
            row["id"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| PrismError::new("PP7401", format!("{field} ID is absent")))
        })
        .collect()
}

/// Replace modeled sensitive fields recursively before a value enters evidence.
pub fn redact(model: &Value, observed: &mut Value) -> Result<(), PrismError> {
    let fields = model["observability"]["redacted_fields"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7401", "redacted field model is absent"))?
        .iter()
        .filter_map(Value::as_str)
        .map(|value| value.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    fn visit(value: &mut Value, fields: &BTreeSet<String>) {
        match value {
            Value::Array(rows) => rows.iter_mut().for_each(|row| visit(row, fields)),
            Value::Object(rows) => {
                for (name, value) in rows {
                    if fields.contains(&name.to_ascii_lowercase()) {
                        *value = Value::String("[REDACTED]".to_owned());
                    } else {
                        visit(value, fields);
                    }
                }
            }
            _ => {}
        }
    }
    visit(observed, &fields);
    Ok(())
}

/// Check that the operations model closes over probes, signals, SLIs, SLOs, and alerts.
pub fn model_evidence(model: &Value, release_digest: &str) -> Result<Vec<Value>, PrismError> {
    let slis = names(model, "slis")?;
    let slos = names(model, "slos")?;
    let alerts = names(model, "alerts")?;
    if slis.is_empty() || slos.is_empty() || alerts.is_empty() {
        return Err(PrismError::new(
            "PP7401",
            "production system requires modeled SLI, SLO, and alert rows",
        ));
    }
    for component in model["components"].as_array().into_iter().flatten() {
        if component["health"].as_str().unwrap_or_default().is_empty() {
            return Err(PrismError::new(
                "PP7401",
                "component health boundary is absent",
            ));
        }
    }
    let observation = encode_value(&json!({
        "alerts":alerts,"release_digest":release_digest,"slis":slis,"slos":slos,
        "telemetry":model["observability"]
    }))?;
    Ok(vec![
        json!({"evidence_digest":sha(&observation),"id":"operations-model","kind":"contract","status":"passed"}),
    ])
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn nested_sensitive_values_are_redacted() {
        let model = json!({"observability":{"redacted_fields":["authorization","token"]}});
        let mut value = json!({"nested":{"token":"planted"},"safe":"visible"});
        super::redact(&model, &mut value).unwrap();
        assert_eq!(value["nested"]["token"], "[REDACTED]");
        assert_eq!(value["safe"], "visible");
    }
}
