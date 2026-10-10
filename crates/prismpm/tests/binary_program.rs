//! Binary package contract regression tests; no application or deployment scope.
use prismpm::holo::canonical::{decode_canonical, encode_value};
use serde_json::{json, Value};
fn document() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!("../../../tests/data/text-model-document.json")).unwrap();
    value.as_object_mut().unwrap().remove("application");
    value["schema"] = json!("prismpm/model-document/5");
    value["program"] = json!({
        "profile":"prismpm/binary-program/1", "name":"Binary probe",
        "cargo_name":"prism-binary-probe", "cargo_version":"0.1.0",
        "cargo_description":"Arbitrary binary package acceptance",
        "cargo_repository":"https://github.com/UOR-Foundation/PrismPM",
        "cargo_homepage":"https://github.com/UOR-Foundation/PrismPM",
        "export_roots":["BinaryProbe.Probe.identity"], "entry_root":"BinaryProbe.Probe.identity",
        "request_maximum":256,"response_maximum":256,"memory_pages":32,
        "acceptance_vectors":[{"request":[],"response":[]},{"request":[0,255,192,128],"response":[0,255,192,128]}],
        "cli":{"profile":"prismpm/raw-file-cli/1"}
    });
    value
}
#[test]
fn binary_program_roundtrips_arbitrary_bytes_without_application_scope() {
    let value = document();
    let bytes = encode_value(&value).unwrap();
    let model = decode_canonical(&bytes).expect("binary profile must be supported");
    assert!(model.application.is_none() && model.library.is_none());
    assert_eq!(prismpm::holo::canonical::encode_canonical(&model).unwrap(), bytes);
    prismpm::contracts::CanonicalDocument::from_value("prismpm/model-document/5",value).unwrap();
}
#[test]
fn binary_program_rejects_missing_cli_bad_bounds_roots_and_profile_confusion() {
    for (pointer, replacement) in [
        ("/schema",json!("prismpm/model-document/3")), ("/program",Value::Null),
        ("/program/cli",Value::Null), ("/program/cli/profile",json!("optional")),
        ("/program/request_maximum",json!(0)), ("/program/response_maximum",json!(0)),
        ("/program/memory_pages",json!(0)), ("/program/memory_pages",json!(32768)),
        ("/program/entry_root",json!("BinaryProbe.Probe.absent")),
        ("/program/export_roots",json!(["BinaryProbe.Probe.identity","BinaryProbe.Probe.identity"])),
        ("/program/acceptance_vectors",json!([])),
        ("/program/acceptance_vectors/1/request",json!(vec![0;257])),
        ("/program/acceptance_vectors/1/response",json!(vec![0;257])),
    ] { let mut value=document(); *value.pointer_mut(pointer).unwrap()=replacement;
        assert!(decode_canonical(&encode_value(&value).unwrap()).is_err(),"{pointer}"); }
    for pointer in ["", "/program", "/program/cli", "/program/acceptance_vectors/0"] {
        let mut value=document(); value.pointer_mut(pointer).unwrap().as_object_mut().unwrap().insert("unknown".into(),json!(true));
        assert!(decode_canonical(&encode_value(&value).unwrap()).is_err(),"{pointer}");
    }
}

fn acceptance()->Value {
    let program=document()["program"].clone();
    json!({"schema":"prismpm/binary-acceptance/1","profile":"prismpm/binary-program/1","scope":"binary-package-only", "model_id":"a".repeat(64),"build_id":"b".repeat(64),"lexlean_attestation_id":"c".repeat(64),"program":program,
        "executions":["std","no_std","core-wasm","cli-stdio","cli-file","cli-mixed"].iter().map(|mode|json!({"mode":mode,"status":"passed","vector_count":2})).collect::<Vec<_>>(),
        "io_coverage":{"platform":"linux","output_write":"passed"},"regeneration":"byte-identical","status":"passed","unclaimed":["application","browser","holo","production-release","deployment"]})
}
#[test]
fn binary_acceptance_rejects_vector_transport_omission_and_scope_substitution() {
    use prismpm::contracts::CanonicalDocument;
    let valid=acceptance();CanonicalDocument::from_value("prismpm/binary-acceptance/1",valid.clone()).unwrap();
    for mutation in 0..6 {
        let mut changed=valid.clone();match mutation {
            0=>{changed["program"]["acceptance_vectors"].as_array_mut().unwrap().pop();}
            1=>{changed["executions"].as_array_mut().unwrap().pop();}
            2=>changed["executions"][0]["vector_count"]=json!(1),
            3=>changed["executions"][0]["mode"]=json!("core-wasm"),
            4=>changed["scope"]=json!("production-release"),
            _=>changed["regeneration"]=json!("not-checked"),
        }
        assert!(CanonicalDocument::from_value("prismpm/binary-acceptance/1",changed).is_err());
    }
}

#[test]
fn binary_acceptance_explicitly_scopes_empty_response_io_coverage() {
    use prismpm::contracts::CanonicalDocument;
    let mut value=acceptance();
    for vector in value["program"]["acceptance_vectors"].as_array_mut().unwrap() { vector["response"]=json!([]); }
    assert!(CanonicalDocument::from_value("prismpm/binary-acceptance/1",value.clone()).is_err());
    value["io_coverage"]["output_write"]=json!("not-exercised-empty-responses");
    CanonicalDocument::from_value("prismpm/binary-acceptance/1",value.clone()).unwrap();
    value.as_object_mut().unwrap().remove("io_coverage");
    assert!(CanonicalDocument::from_value("prismpm/binary-acceptance/1",value).is_err());
}
