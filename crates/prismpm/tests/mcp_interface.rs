//! MCP 2026-07-28 as a system-model interface kind: the kind and its single
//! protocol binding are accepted together and refused apart.

use prismpm::contracts::CanonicalDocument;
use prismpm::system::{validate, validation_certificate};
use serde_json::{json, Value};
use std::path::Path;

fn model_field_name(record: Option<&str>, name: &str) -> String {
    match name {
        "authentication" => return "auth".to_owned(),
        "defaultValue" => return "default".to_owned(),
        "fromComponent" if record == Some("Flow") => return "from".to_owned(),
        "interfaceId" if record == Some("Flow") => return "interface".to_owned(),
        "publiclyAccessible" => return "public".to_owned(),
        "toComponent" if record == Some("Flow") => return "to".to_owned(),
        "valueType" => return "type".to_owned(),
        _ => {}
    }
    let mut out = String::with_capacity(name.len() + 4);
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            out.push('_');
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(character);
        }
    }
    out
}

fn normalize(value: &Value, record: Option<&str>) -> Value {
    match value {
        Value::Array(rows) => rows.iter().map(|row| normalize(row, record)).collect(),
        Value::Object(fields) => fields
            .iter()
            .map(|(name, value)| {
                (
                    model_field_name(record, name),
                    normalize(value, (name == "flows").then_some("Flow")),
                )
            })
            .collect(),
        _ => value.clone(),
    }
}

fn fixture_model() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/system/release-fixture.json");
    let fixture: Value =
        serde_json::from_slice(&std::fs::read(path).expect("read release fixture"))
            .expect("parse release fixture");
    let mut model = normalize(&fixture["model"], None);
    model["schema"] = json!("prismpm/system-model/1");
    model
}

#[allow(clippy::result_large_err)]
fn checked(model: Value) -> Result<(), prismpm::error::PrismError> {
    let model = CanonicalDocument::from_value("prismpm/system-model/1", model)?
        .value()
        .clone();
    let mut certificate = validation_certificate(&model)?;
    certificate.as_object_mut().unwrap().remove("schema");
    validate(&model, &certificate)
}

fn declare(model: &mut Value, kind: &str, protocol: &str) {
    model["interfaces"][1]["kind"] = json!(kind);
    model["interfaces"][1]["protocol"] = json!(protocol);
}

#[test]
fn an_mcp_interface_is_accepted_with_its_protocol_binding() {
    checked(fixture_model()).expect("the unmodified fixture validates");
    let mut model = fixture_model();
    declare(&mut model, "mcp", "mcp-http-2026-07-28");
    checked(model).expect("an mcp interface on the mcp-http-2026-07-28 protocol validates");
}

#[test]
fn the_mcp_kind_and_protocol_are_refused_apart() {
    let mut kind_alone = fixture_model();
    declare(&mut kind_alone, "mcp", "https-json");
    assert_eq!(checked(kind_alone).unwrap_err().code, "PP2101");

    let mut protocol_alone = fixture_model();
    declare(&mut protocol_alone, "openapi", "mcp-http-2026-07-28");
    assert_eq!(checked(protocol_alone).unwrap_err().code, "PP2101");

    let mut unknown_kind = fixture_model();
    declare(&mut unknown_kind, "mcp-2025", "mcp-http-2026-07-28");
    assert_eq!(checked(unknown_kind).unwrap_err().code, "PP1101");
}
