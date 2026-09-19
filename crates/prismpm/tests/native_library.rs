//! Native-library projection is distinct from application or release acceptance.

use prismpm::holo::canonical::{decode_canonical, encode_value};
use serde_json::{json, Value};

fn document() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/data/text-model-document.json"
    ))
    .unwrap();
    value.as_object_mut().unwrap().remove("application");
    value["schema"] = json!("prismpm/model-document/3");
    value["library"] = json!({
        "profile": "prismpm/native-library/1",
        "name": "Library probe",
        "cargo_name": "prism-library-probe",
        "cargo_version": "0.1.0",
        "cargo_description": "Finite native-library acceptance fixture",
        "cargo_repository": "https://github.com/UOR-Foundation/PrismPM",
        "cargo_homepage": "https://github.com/UOR-Foundation/PrismPM",
        "export_roots": ["LibraryProbe.Probe.acceptance", "LibraryProbe.Probe.identity"],
        "acceptance_roots": ["LibraryProbe.Probe.acceptance"]
    });
    value
}

#[test]
fn explicit_native_library_roundtrips_without_an_application() {
    let bytes = encode_value(&document()).unwrap();
    let model = decode_canonical(&bytes).expect("explicit native-library profile is supported");
    assert!(model.application.is_none());
    assert_eq!(
        prismpm::holo::canonical::encode_canonical(&model).unwrap(),
        bytes
    );
}

#[test]
fn native_library_preserves_legal_package_names_including_rust_keywords() {
    for name in ["self", "crate", "super", "core", "std", "alloc", "match"] {
        let mut value = document();
        value["library"]["cargo_name"] = json!(name);
        let model = decode_canonical(&encode_value(&value).unwrap()).unwrap();
        assert_eq!(model.library.unwrap().cargo_name, name);
    }
}

#[test]
fn native_library_rejects_wrong_schema_nulls_unknown_fields_and_invalid_metadata() {
    let valid = document();
    for (pointer, replacement) in [
        ("/schema", json!("prismpm/model-document/1")),
        ("/schema", json!("prismpm/model-document/2")),
        ("/library", Value::Null),
        ("/library/profile", json!("prismpm/native-library/2")),
        ("/library/name", json!(" \n")),
        ("/library/cargo_name", json!("../outside")),
        ("/library/cargo_version", json!("0.1.0-preview")),
        (
            "/library/cargo_homepage",
            json!("http://github.com/UOR-Foundation/PrismPM"),
        ),
        ("/library/export_roots", json!([])),
        (
            "/library/export_roots",
            json!([
                "LibraryProbe.Probe.acceptance",
                "LibraryProbe.Probe.acceptance"
            ]),
        ),
        ("/library/acceptance_roots", json!([])),
        (
            "/library/acceptance_roots",
            json!(["LibraryProbe.Probe.missing"]),
        ),
        ("/library/export_roots", json!(["escape();"])),
        ("/standards_profile", json!(["self-certified"])),
    ] {
        let mut bad = valid.clone();
        *bad.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            decode_canonical(&encode_value(&bad).unwrap()).is_err(),
            "{pointer}"
        );
    }
    for pointer in ["", "/library", "/provenance"] {
        let mut bad = valid.clone();
        bad.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_owned(), json!(true));
        assert!(
            decode_canonical(&encode_value(&bad).unwrap()).is_err(),
            "{pointer}"
        );
    }
    let mut bad = valid;
    bad["application"] = Value::Null;
    assert!(decode_canonical(&encode_value(&bad).unwrap()).is_err());
    let mut legacy: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/data/text-model-document.json"
    ))
    .unwrap();
    legacy["library"] = Value::Null;
    assert!(decode_canonical(&encode_value(&legacy).unwrap()).is_err());
}

#[test]
fn native_library_schema_and_semantic_contract_reject_additive_fields() {
    let schema: Value =
        serde_json::from_slice(include_bytes!("../schemas/model-document-v3.schema.json")).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let valid = document();
    assert!(validator.is_valid(&valid));
    prismpm::contracts::CanonicalDocument::from_value("prismpm/model-document/3", valid.clone())
        .unwrap();
    for pointer in ["", "/library", "/provenance"] {
        let mut bad = valid.clone();
        bad.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".to_owned(), json!(true));
        assert!(!validator.is_valid(&bad));
        assert!(
            prismpm::contracts::CanonicalDocument::from_value("prismpm/model-document/3", bad)
                .is_err()
        );
    }
    for field in valid["library"].as_object().unwrap().keys() {
        let mut bad = valid.clone();
        bad["library"].as_object_mut().unwrap().remove(field);
        assert!(!validator.is_valid(&bad), "{field}");
        assert!(
            decode_canonical(&encode_value(&bad).unwrap()).is_err(),
            "{field}"
        );
    }
}

fn binding() -> Value {
    json!({
        "acceptance_roots": ["LibraryProbe.Probe.acceptance"],
        "export_roots": ["LibraryProbe.Probe.acceptance", "LibraryProbe.Probe.identity"],
        "model_id": "a".repeat(64),
        "profile": "prismpm/native-library/1",
        "schema": "prismpm/library-build-binding/1",
        "scope": "native-library-only"
    })
}

fn acceptance() -> Value {
    json!({
        "build_id": "b".repeat(64),
        "executions": [
            {"mode":"std", "roots":["LibraryProbe.Probe.acceptance"], "status":"passed"},
            {"mode":"no_std", "roots":["LibraryProbe.Probe.acceptance"], "status":"passed"}
        ],
        "export_roots": ["LibraryProbe.Probe.acceptance", "LibraryProbe.Probe.identity"],
        "lexlean_attestation_id": "c".repeat(64),
        "model_id": "a".repeat(64),
        "profile": "prismpm/native-library/1",
        "regeneration": "byte-identical",
        "schema": "prismpm/library-acceptance/1",
        "scope": "native-library-only",
        "status": "passed",
        "unclaimed": ["application", "browser", "holo", "production-release", "deployment"]
    })
}

#[test]
fn native_library_evidence_requires_identical_ordered_exported_acceptance_roots() {
    use prismpm::contracts::CanonicalDocument;
    for (id, valid) in [
        ("prismpm/library-build-binding/1", binding()),
        ("prismpm/library-acceptance/1", acceptance()),
    ] {
        CanonicalDocument::from_value(id, valid.clone()).unwrap();
        for roots in [
            json!([]),
            json!([
                "LibraryProbe.Probe.identity",
                "LibraryProbe.Probe.acceptance"
            ]),
            json!([
                "LibraryProbe.Probe.acceptance",
                "LibraryProbe.Probe.acceptance"
            ]),
            json!(["LibraryProbe.Probe.identity"]),
            json!(["not a qualified name"]),
            json!(["LibraryProbe..acceptance"]),
            json!(["LibraryProbe.Probe.42"]),
            json!([format!("LibraryProbe.{}", "a".repeat(1024))]),
        ] {
            let mut changed = valid.clone();
            changed["export_roots"] = roots;
            assert!(CanonicalDocument::from_value(id, changed).is_err());
        }
        for (field, replacement) in [
            ("scope", json!("production-release")),
            ("profile", json!("prismpm/native-library/2")),
            ("extra", json!(true)),
        ] {
            let mut changed = valid.clone();
            changed[field] = replacement;
            assert!(CanonicalDocument::from_value(id, changed).is_err());
        }
    }
    let id = "prismpm/library-build-binding/1";
    for roots in [
        json!([]),
        json!(["LibraryProbe.Probe.missing"]),
        json!([
            "LibraryProbe.Probe.identity",
            "LibraryProbe.Probe.acceptance"
        ]),
    ] {
        let mut changed = binding();
        changed["acceptance_roots"] = roots;
        assert!(CanonicalDocument::from_value(id, changed).is_err());
    }
    let id = "prismpm/library-acceptance/1";
    for (pointer, replacement) in [
        ("/executions/0/roots", json!([])),
        (
            "/executions/1/roots",
            json!(["LibraryProbe.Probe.identity"]),
        ),
        ("/executions/0/mode", json!("no_std")),
        ("/executions/1/status", json!("failed")),
        ("/unclaimed", json!([])),
        ("/regeneration", json!("unchecked")),
    ] {
        let mut changed = acceptance();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            CanonicalDocument::from_value(id, changed).is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn native_library_manifest_rejects_path_escape_duplicate_artifacts_and_failed_processes() {
    use prismpm::contracts::CanonicalDocument;
    let id = "prismpm/library-verification-manifest/1";
    let artifact = |path| json!({"path":path,"byte_length":0,"sha256":"a".repeat(64)});
    let valid = json!({
        "acceptance_sha256":"a".repeat(64),
        "artifacts":[artifact("library/a"),artifact("library/b")],
        "build_id":"b".repeat(64),
        "lexlean_attestation_sha256":"c".repeat(64),
        "model_sha256":"d".repeat(64),
        "processes":[{"tool":"native-library-std-acceptance","argv":[],
            "executable_sha256":"e".repeat(64),"exit_code":0,"stdout":"","stderr":""}],
        "schema":id,
        "scope":"native-library-only"
    });
    CanonicalDocument::from_value(id, valid.clone()).unwrap();
    for path in [
        "library/../escape",
        "library/./a",
        "library//a",
        "library/",
        "/library/a",
        "library\\a",
    ] {
        let mut changed = valid.clone();
        changed["artifacts"] = json!([artifact(path)]);
        assert!(
            CanonicalDocument::from_value(id, changed).is_err(),
            "{path}"
        );
    }
    for rows in [
        json!([]),
        json!([artifact("library/b"), artifact("library/a")]),
        json!([artifact("library/a"), artifact("library/a")]),
    ] {
        let mut changed = valid.clone();
        changed["artifacts"] = rows;
        assert!(CanonicalDocument::from_value(id, changed).is_err());
    }
    for (pointer, replacement) in [
        ("/processes", json!([])),
        ("/processes/0/exit_code", json!(1)),
        ("/processes/0/executable_sha256", json!("unbound")),
        ("/artifacts/0/byte_length", json!(-1)),
        ("/scope", json!("production-release")),
    ] {
        let mut changed = valid.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            CanonicalDocument::from_value(id, changed).is_err(),
            "{pointer}"
        );
    }
}
