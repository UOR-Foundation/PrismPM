//! Closed source-owned file session contract; no domain model is interpreted here.
use prismpm::holo::{model_document::ModelDocument, workbench};
use serde_json::{json, Value};
fn fixture() -> Value {
    serde_json::from_slice(include_bytes!(
        "../../../tests/data/file-session-model-document.json"
    ))
    .unwrap()
}
fn valid(value: Value) -> bool {
    serde_json::from_value::<ModelDocument>(value)
        .is_ok_and(|d| prismpm::holo::validate::validate(&d).is_ok())
}
#[test]
fn file_session_closed_schema_and_semantic_contract_agree() {
    let value = fixture();
    let schema: Value = serde_json::from_slice(include_bytes!(
        "../../../schemas/model-document-v7.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(validator.is_valid(&value));
    let checked: ModelDocument = serde_json::from_value(value.clone()).unwrap();
    prismpm::holo::validate::validate(&checked).unwrap();
    for key in value["application"]["session"].as_object().unwrap().keys() {
        let mut bad = value.clone();
        bad["application"]["session"]
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(!validator.is_valid(&bad), "missing {key}");
        assert!(!valid(bad));
    }
    for pointer in ["/application/session", "/application/session/file_inputs/0"] {
        let mut bad = value.clone();
        bad.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("ambient_storage".into(), json!(true));
        assert!(!validator.is_valid(&bad));
        assert!(!valid(bad));
    }
    let doc: ModelDocument = serde_json::from_value(value).unwrap();
    let encoded = prismpm::holo::canonical::encode_canonical(&doc).unwrap();
    assert_eq!(
        prismpm::holo::canonical::decode_canonical(&encoded).unwrap(),
        doc
    );
}
#[test]
fn file_session_old_profiles_refuse_new_metadata_and_new_profile_requires_it() {
    for profile in [
        "prismpm/text-application/1",
        "prismpm/text-application/2",
        "prismpm/text-application/3",
    ] {
        let mut bad = fixture();
        bad["application"]["profile"] = json!(profile);
        assert!(!valid(bad));
    }
    for key in ["session", "workbench"] {
        let mut bad = fixture();
        bad["application"].as_object_mut().unwrap().remove(key);
        assert!(!valid(bad));
    }
    let mut bad = fixture();
    bad["application"]["session"] = Value::Null;
    assert!(!valid(bad));
}
#[test]
fn file_session_state_and_file_bindings_fail_closed() {
    let cases = [
        ("/application/session/state_field", json!("absent")),
        ("/application/session/restore_operation", json!("absent")),
        ("/application/session/maximum_state_bytes", json!(0)),
        ("/application/session/maximum_state_bytes", json!(8193)),
        ("/application/session/file_name", json!("../private")),
        ("/application/session/file_inputs/0/field", json!("state")),
        ("/application/session/file_inputs/0/accept", json!("*/*")),
        (
            "/application/workbench/operations/0/fields/0/required",
            json!(true),
        ),
        (
            "/application/workbench/operations/0/fields/0/initial_value",
            json!("different"),
        ),
    ];
    for (path, value) in cases {
        let mut bad = fixture();
        *bad.pointer_mut(path).unwrap() = value;
        assert!(!valid(bad), "{path}");
    }
    let mut bad = fixture();
    let file = bad["application"]["session"]["file_inputs"][0].clone();
    bad["application"]["session"]["file_inputs"]
        .as_array_mut()
        .unwrap()
        .push(file);
    assert!(!valid(bad));
}
#[test]
fn file_session_only_bounded_success_can_replace_state() {
    let doc: ModelDocument = serde_json::from_value(fixture()).unwrap();
    let app = doc.application.unwrap().workbench().unwrap();
    let mut answer = json!({"version":1,"status":"ok","operation":"echo","message":"A checked response.","rows":0,"columns":0,"cells":[],"details":[],"request":"","response":"a\u{001f}é\r\n\0","error":""});
    assert!(workbench::result(&app, "echo", &serde_json::to_vec(&answer).unwrap()).is_ok());
    answer["response"] = json!("é".repeat(1025));
    assert!(workbench::result(&app, "echo", &serde_json::to_vec(&answer).unwrap()).is_err());
    answer["response"] = json!("small");
    answer["operation"] = json!("restore");
    assert!(workbench::result(&app, "echo", &serde_json::to_vec(&answer).unwrap()).is_err());
}

#[test]
fn file_session_continuations_are_bounded_declared_drafts_without_state() {
    let doc: ModelDocument = serde_json::from_value(fixture()).unwrap();
    let app = doc.application.unwrap().workbench().unwrap();
    let mut answer = json!({"version":1,"status":"ok","operation":"echo","message":"Choose a next step.","rows":0,"columns":0,"cells":[],"details":[],"request":"","response":"accepted","error":""});
    for request in ["ui-echo-v1\u{1f}0:0:", "ui-echo-v1\u{1f}0:7:é\r\n\0\u{1f}:"] {
        answer["request"] = json!(request);
        assert!(
            workbench::result(&app, "echo", &serde_json::to_vec(&answer).unwrap()).is_ok(),
            "{request:?}"
        );
    }
    for request in [
        "missing\u{1f}0:0:",
        "restore\u{1f}0:",
        "ui-echo-v1\u{1f}1:x0:",
        "ui-echo-v1\u{1f}00:0:",
        "ui-echo-v1\u{1f}0:1:é",
        "ui-echo-v1\u{1f}0:",
        "ui-echo-v1\u{1f}0:0:0:",
        "ui-echo-v1\u{1f}0:99999999999:x",
    ] {
        answer["request"] = json!(request);
        assert!(
            workbench::result(&app, "echo", &serde_json::to_vec(&answer).unwrap()).is_err(),
            "{request:?}"
        );
    }
    answer["request"] = json!(format!("ui-echo-v1\u{1f}0:4096:{}", "a".repeat(4096)));
    assert!(workbench::result(&app, "echo", &serde_json::to_vec(&answer).unwrap()).is_err());
}
