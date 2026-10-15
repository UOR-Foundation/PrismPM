//! Closed public provenance controls; data fixtures never mint private authority.
//!
//! The 13 historical subject IDs remain stable. Raw cache-row/parser assertions
//! require the separately reviewed genuine private custody owner, not a simulated
//! validator here. Four legacy inventory-named IDs now check the public receipt.

use prismpm::contracts::CanonicalDocument;
use prismpm::holo::canonical::encode_value;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const ARCHIVE: &str = "7a1a716a57040ea2307f7a34e895a75ebdca7438cf9d5d0364007acb594366bf";
const CURRENT: &str = "prismpm/library-verification-manifest/2";
const ACQUISITIONS: [&str; 3] = [
    "/processes/7/exporter/acquisition",
    "/exporter_owner/phases/0/processes/2/exporter/acquisition",
    "/exporter_owner/phases/1/processes/2/exporter/acquisition",
];
const EXECUTABLES: [&str; 3] = [
    "/processes/7/exporter/executable",
    "/exporter_owner/phases/0/processes/2/exporter/executable",
    "/exporter_owner/phases/1/processes/2/exporter/executable",
];

fn schema(historical: bool) -> Value {
    serde_json::from_str(if historical {
        include_str!("../schemas/library-verification-manifest.schema.json")
    } else {
        include_str!("../schemas/library-verification-manifest-v2.schema.json")
    })
    .unwrap()
}

fn validator(historical: bool) -> jsonschema::Validator {
    jsonschema::validator_for(&schema(historical)).unwrap()
}

fn hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn executable() -> Value {
    json!({"byte_length":b"data-only exporter fixture".len(),"mode":493,
        "sha256":hex(b"data-only exporter fixture")})
}

fn capsule() -> Value {
    let dependencies: toml::Value =
        toml::from_str(include_str!("../model/dependencies.toml")).unwrap();
    let compiler = dependencies["dependency"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"].as_str() == Some("lean4-prod"))
        .unwrap();
    json!({"schema":"prismpm/exporter-acquisition/1","mode":"controller-capsule",
        "archive_sha256":ARCHIVE,"compiler_revision":compiler["revision"].as_str().unwrap(),
        "toolchain":format!("leanprover/lean4:v{}",compiler["lean_version"].as_str().unwrap()),
        "platform":"linux/amd64","construction":{"tool":"lean4-prod-build","argv":["build","prod-export"],
            "executable_sha256":"b".repeat(64),"exit_code":0,"stdout":"","stderr":""},
        "executable":executable(),"executable_sha256":hex(b"data-only exporter fixture"),
        "lean_sha256":"e".repeat(64)})
}

fn process(tool: &str, argv: Value) -> Value {
    let mut record = json!({"tool":tool,"argv":argv,"executable_sha256":"b".repeat(64),
        "exit_code":0,"stdout":"","stderr":""});
    if tool == "prod-export" {
        record["exporter"] = json!({"schema":"prismpm/exporter-execution/1",
            "source_archive_sha256":ARCHIVE,"acquisition":capsule(),"executable":executable()});
    }
    record
}

fn fixture() -> Value {
    let phases = json!([
        process("lake-build-generated", json!(["build", "PrismGenerated"])),
        process("lean4-prod-build", json!(["build", "prod-export"])),
        process("prod-export", json!(["exe", "prod-export", "--library"])),
        process(
            "native-library-package",
            json!(["package", "--locked", "--offline", "--allow-dirty"])
        )
    ]);
    let artifacts = json!([{"path":"library/src/lib.rs","byte_length":1,"sha256":"f".repeat(64)}]);
    let phase = |name| {
        json!({"phase":name,"role":"library","model_sha256":"1".repeat(64),
        "lexlean_manifest_sha256":"2".repeat(64),"artifacts":artifacts.clone(),"processes":phases.clone()})
    };
    let host = "x86_64-unknown-linux-gnu";
    let outputs = [
        format!("Lean (version 4.32.1, {host}, commit f054605aea4b840552cca2e725580bffd1e1b704, Release)\n"),
        "Lake version 5.0.0-src+f054605 (Lean version 4.32.1)\n".to_owned(),
        "rustfmt 1.9.0-stable (8bab26f4f6 2026-07-14)\n".to_owned(),
        format!("commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452\nhost: {host}\nrelease: 1.97.1\n"),
        "timeout (GNU coreutils) 9.4\n".to_owned()];
    let mut processes: Vec<Value> = [
        "lean-version",
        "lake-version",
        "rustfmt-version",
        "rustc-version",
        "timeout-version",
    ]
    .into_iter()
    .zip(outputs)
    .map(|(tool, stdout)| {
        json!({"tool":tool,
            "argv":if tool=="rustc-version" {vec!["--version","--verbose"]}else{vec!["--version"]},
            "executable_sha256":if tool=="lake-version" {"b".repeat(64)}else{"e".repeat(64)},
            "exit_code":0,"stdout":stdout,"stderr":""})
    })
    .collect();
    processes.extend(phases.as_array().unwrap().iter().cloned());
    json!({"schema":CURRENT,"scope":"native-library-only","acceptance_sha256":"3".repeat(64),
        "artifacts":artifacts,"build_id":"4".repeat(64),"lexlean_attestation_sha256":"5".repeat(64),
        "model_sha256":"1".repeat(64),"processes":processes,
        "exporter_owner":{"schema":"prismpm/verification-exporter-owner/1",
            "phases":[phase("controller-build"),phase("replay")]}})
}

fn with_acquisition(acquisition: Value) -> Value {
    let mut document = fixture();
    for path in ACQUISITIONS {
        *document.pointer_mut(path).unwrap() = acquisition.clone();
    }
    document
}

fn admits(value: Value) -> bool {
    validator(false).is_valid(&with_acquisition(value))
}
fn current_admits(value: Value) -> bool {
    CanonicalDocument::from_value(CURRENT, with_acquisition(value)).is_ok()
}

#[test]
fn current_capsule_schema_has_exact_eleven_fields_and_only_local_references() {
    // Historical subject ID retained; the explicitly redesigned current receipt
    // has TEN fields, including the complete native executable measurement.
    let schema = schema(false);
    let shape = &schema["$defs"]["controllerCapsuleAcquisition"];
    let expected = [
        "archive_sha256",
        "compiler_revision",
        "construction",
        "executable",
        "executable_sha256",
        "lean_sha256",
        "mode",
        "platform",
        "schema",
        "toolchain",
    ];
    let keys = shape["properties"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(keys, expected);
    let mut required = shape["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    required.sort_unstable();
    assert_eq!(required, expected);
    assert_eq!(shape["additionalProperties"], false);
    fn local(value: &Value) {
        match value {
            Value::Object(fields) => {
                if let Some(reference) = fields.get("$ref") {
                    assert!(reference.as_str().unwrap().starts_with("#/"));
                }
                for child in fields.values() {
                    local(child);
                }
            }
            Value::Array(values) => {
                for child in values {
                    local(child);
                }
            }
            _ => {}
        }
    }
    local(&schema);
    let _ = validator(false);
}

#[test]
fn data_only_current_capsules_cover_both_native_platforms_and_keep_old_variants() {
    for platform in ["linux/amd64", "linux/arm64"] {
        let mut value = capsule();
        value["platform"] = json!(platform);
        assert!(admits(value));
    }
    assert!(
        current_admits(capsule()),
        "positive current provenance/context joins must pass"
    );
    assert!(admits(
        json!({"schema":"prismpm/exporter-acquisition/1","mode":"cold"})
    ));
    assert!(admits(
        json!({"schema":"prismpm/exporter-acquisition/1","mode":"sdk-seed",
        "inventory_sha256":"1".repeat(64),"manifest_sha256":"2".repeat(64),
        "executable_sha256":"3".repeat(64),"archive_sha256":"4".repeat(64),
        "compiler_revision":"5".repeat(40),"platform":"linux/arm64","toolchain":"leanprover/lean4:v4.32.1"})
    ));
}

#[test]
fn every_capsule_field_is_required_and_unknown_properties_are_refused() {
    let value = capsule();
    for name in value.as_object().unwrap().keys() {
        let mut changed = value.clone();
        changed.as_object_mut().unwrap().remove(name);
        assert!(!admits(changed), "missing {name}");
        let mut changed = value.clone();
        changed[name] = Value::Null;
        assert!(!admits(changed), "null {name}");
    }
    let mut changed = value;
    changed["unowned"] = json!(true);
    assert!(!admits(changed));
}

#[test]
fn capsule_discriminators_registered_archive_and_digest_types_are_closed() {
    for (name, bad) in [
        ("schema", json!("prismpm/exporter-acquisition/2")),
        ("mode", json!("sdk-seed")),
        ("mode", json!("cold")),
        ("mode", json!("controller-cache")),
        ("archive_sha256", json!("0".repeat(64))),
        ("compiler_revision", json!("a".repeat(39))),
        ("compiler_revision", json!("A".repeat(40))),
        ("platform", json!("linux/x86_64")),
        ("platform", json!("windows/amd64")),
        ("toolchain", json!("leanprover/lean4:nightly")),
        ("toolchain", json!("leanprover/lean4:v4.32")),
        ("executable", json!([])),
    ] {
        let mut changed = capsule();
        changed[name] = bad;
        assert!(!admits(changed), "accepted wrong {name}");
    }
    for name in ["executable_sha256", "lean_sha256"] {
        for bad in [
            json!("A".repeat(64)),
            json!("a".repeat(63)),
            json!(format!("sha256:{}", "a".repeat(64))),
            json!(0),
        ] {
            let mut changed = capsule();
            changed[name] = bad;
            assert!(!admits(changed), "accepted malformed {name}");
        }
    }
    for (name, bad) in [
        ("compiler_revision", json!("0".repeat(40))),
        ("toolchain", json!("leanprover/lean4:v4.32.2")),
        ("platform", json!("linux/arm64")),
        ("lean_sha256", json!("0".repeat(64))),
    ] {
        let mut changed = capsule();
        changed[name] = bad;
        assert!(
            admits(changed.clone()),
            "semantic source/preflight negative passes shape"
        );
        assert!(
            !current_admits(changed),
            "actual source/preflight mismatch {name}"
        );
    }
}

#[test]
fn actual_construction_record_is_closed_and_cannot_contain_exporter_acquisition() {
    let original = capsule();
    for name in original["construction"].as_object().unwrap().keys() {
        let mut changed = original.clone();
        changed["construction"]
            .as_object_mut()
            .unwrap()
            .remove(name);
        assert!(!admits(changed), "missing construction {name}");
    }
    for (name, bad) in [
        ("tool", json!("lake-generated")),
        ("argv", json!(["build"])),
        ("argv", json!(["build", "PrismGenerated"])),
        ("argv", json!(["build", "prod-export", "extra"])),
        ("exit_code", json!(1)),
        ("exit_code", json!(-1)),
        ("stdout", json!(false)),
        ("stderr", json!([])),
        ("executable_sha256", json!("z".repeat(64))),
        ("exporter", json!({})),
        ("unknown", json!(true)),
    ] {
        let mut changed = original.clone();
        changed["construction"][name] = bad;
        assert!(!admits(changed), "accepted construction {name}");
    }
}

#[test]
fn construction_streams_keep_the_original_sixteen_mebibyte_shape_bound() {
    let validate = validator(false);
    for name in ["stdout", "stderr"] {
        let mut value = capsule();
        value["construction"][name] = json!("x".repeat(16_777_217));
        assert!(!validate.is_valid(&with_acquisition(value)));
    }
}

#[test]
fn file_and_directory_rows_are_disjoint_closed_records() {
    // Public successor: a closed complete executable, not private file/dir rows.
    // The former raw-row negatives are mandatory in the private custody owner.
    let original = capsule();
    assert!(current_admits(original.clone()));
    for name in ["byte_length", "mode", "sha256"] {
        let mut changed = original.clone();
        changed["executable"].as_object_mut().unwrap().remove(name);
        assert!(!admits(changed), "missing executable {name}");
        let mut changed = original.clone();
        changed["executable"][name] = Value::Null;
        assert!(!admits(changed), "null executable {name}");
    }
    for (name, bad) in [
        ("kind", json!("file")),
        ("kind", json!("directory")),
        ("path", json!(".lake/build/bin/prod-export")),
        ("target", json!("other")),
        ("unknown", json!(true)),
    ] {
        let mut changed = original.clone();
        changed["executable"][name] = bad;
        assert!(!admits(changed), "unknown executable field {name}");
    }
    for bad in [Value::Null, json!([]), json!("file"), json!(1), json!(true)] {
        let mut changed = original.clone();
        changed["executable"] = bad;
        assert!(!admits(changed));
    }
}

#[test]
fn capsule_row_lengths_modes_and_confined_path_shapes_are_bounded() {
    // Public successor: descriptor bounds AND exact retained-child joins.
    // Raw inventory path and file/dir mode controls remain private unit tests.
    let original = capsule();
    assert!(current_admits(original.clone()));
    for bad in [
        json!(-1),
        json!(0),
        json!(268_435_457u64),
        json!("1"),
        Value::Null,
    ] {
        let mut changed = original.clone();
        changed["executable"]["byte_length"] = bad;
        assert!(!admits(changed));
    }
    for bad in [
        json!(-1),
        json!(0),
        json!(420),
        json!(512),
        json!(4095),
        json!("493"),
        Value::Null,
    ] {
        let mut changed = original.clone();
        changed["executable"]["mode"] = bad;
        assert!(!admits(changed));
    }
    for bad in [json!("A".repeat(64)), json!("a".repeat(63)), json!(0)] {
        let mut changed = original.clone();
        changed["executable"]["sha256"] = bad;
        assert!(!admits(changed));
    }
    for (name, bad) in [
        (
            "byte_length",
            json!(b"data-only exporter fixture".len() + 1),
        ),
        ("mode", json!(365)),
        ("sha256", json!("0".repeat(64))),
    ] {
        let mut changed = original.clone();
        changed["executable"][name] = bad;
        if name == "sha256" {
            changed["executable_sha256"] = changed["executable"]["sha256"].clone();
        }
        assert!(
            admits(changed.clone()),
            "negative must pass shape before semantic join"
        );
        assert!(
            !current_admits(changed),
            "retained child measurement mismatch {name}"
        );
    }
    let mut changed = original;
    changed["executable_sha256"] = json!("0".repeat(64));
    assert!(
        !current_admits(changed),
        "both executable hash fields must agree"
    );
}

#[test]
fn current_inventory_shape_has_exact_independent_4167_row_ceiling() {
    // Public successor: unchanged parent caps do not carry any raw cache shape.
    // Exact 4096/4167/4097 private bounds are exercised by the private producer.
    let original = fixture();
    let receipt = capsule();
    fn item_count(value: &Value) -> usize {
        match value {
            Value::Array(values) => values.iter().fold(values.len(), |sum, value| {
                sum.saturating_add(item_count(value))
            }),
            Value::Object(values) => values.values().fold(values.len(), |sum, value| {
                sum.saturating_add(item_count(value))
            }),
            _ => 1,
        }
    }
    assert!(item_count(&original) <= 65_536);
    let canonical = encode_value(&original).unwrap();
    assert!(canonical.len() <= 16_777_216);
    CanonicalDocument::parse(CURRENT, &canonical)
        .expect("actual complete public parent must validate first");
    for path in ACQUISITIONS {
        assert_eq!(original.pointer(path).unwrap(), &receipt);
    }
    for (name, value) in [
        ("files", json!([])),
        ("files_document", json!("[]")),
        ("capsule_sha256", json!(hex(b"[]"))),
        ("inventory", json!({})),
        ("inventory_sha256", json!("1".repeat(64))),
    ] {
        let mut changed = receipt.clone();
        changed[name] = value;
        assert!(!admits(changed), "private field {name}");
    }
}

#[test]
fn capsule_validation_applies_to_each_original_replayed_process_owner_position() {
    let validate = validator(false);
    let original = fixture();
    CanonicalDocument::from_value(CURRENT, original.clone()).unwrap();
    for path in ACQUISITIONS {
        let mut changed = original.clone();
        changed
            .pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("construction");
        assert!(
            !validate.is_valid(&changed),
            "unvalidated capsule position {path}"
        );
        let mut changed = original.clone();
        changed.pointer_mut(path).unwrap()["construction"]["stdout"] =
            json!("changed constructor output");
        assert!(validate.is_valid(&changed));
        assert!(
            CanonicalDocument::from_value(CURRENT, changed).is_err(),
            "full public receipt/replay equality {path}"
        );
    }
    for path in EXECUTABLES {
        for (name, value) in [
            (
                "byte_length",
                json!(b"data-only exporter fixture".len() + 1),
            ),
            ("mode", json!(365)),
            ("sha256", json!("0".repeat(64))),
        ] {
            let mut changed = original.clone();
            changed.pointer_mut(path).unwrap()[name] = value;
            assert!(validate.is_valid(&changed));
            assert!(
                CanonicalDocument::from_value(CURRENT, changed).is_err(),
                "whole child descriptor join {path} {name}"
            );
        }
    }
}

#[test]
fn historical_v1_schema_is_byte_identical_and_never_admits_controller_capsules() {
    assert_eq!(
        hex(include_bytes!(
            "../schemas/library-verification-manifest.schema.json"
        )),
        "577d6a3c72d8ea0d213cb46eb2d6410672e6c39ef90904a55a40d40981ed0db1"
    );
    let mut historical = fixture();
    historical.as_object_mut().unwrap().remove("exporter_owner");
    historical["schema"] = json!("prismpm/library-verification-manifest/1");
    assert!(!validator(true).is_valid(&historical));
    historical["processes"][7]["exporter"]["acquisition"] =
        json!({"schema":"prismpm/exporter-acquisition/1","mode":"cold"});
    assert!(validator(true).is_valid(&historical));
}

#[test]
fn registered_parent_contract_limits_are_unchanged_and_archive_pin_is_actual_input() {
    let registry: toml::Value = toml::from_str(include_str!("../model/contracts.toml")).unwrap();
    for id in [
        "prismpm/library-verification-manifest/1",
        "prismpm/library-verification-manifest/2",
    ] {
        let rows = registry["contract"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["schema"].as_str() == Some(id))
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["maximum_bytes"].as_integer(), Some(16_777_216));
        assert_eq!(rows[0]["maximum_items"].as_integer(), Some(65_536));
        assert_eq!(rows[0]["extension_policy"].as_str(), Some("closed"));
    }
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../vendor/lean4-prod/lean.tar"))
        ),
        ARCHIVE
    );
}

#[test]
fn canonical_inventory_strings_reject_duplicates_noncanonical_documents_and_nested_changes() {
    // Public successor: raw inventory envelopes are always unknown fields;
    // current public JSON duplicate keys and full receipt mismatches still fail.
    let original = capsule();
    assert!(current_admits(original.clone()));
    for document in [
        "[]",
        " []",
        "[]\n",
        "{}",
        "not json",
        "[{\"kind\":\"file\",\"kind\":\"directory\"}]",
    ] {
        let mut changed = original.clone();
        changed["files_document"] = json!(document);
        changed["capsule_sha256"] = json!(hex(document.as_bytes()));
        assert!(
            !admits(changed),
            "coherent raw cache envelope is not public provenance"
        );
    }
    let canonical = String::from_utf8(encode_value(&fixture()).unwrap()).unwrap();
    let needle = format!("\"archive_sha256\":\"{ARCHIVE}\"");
    assert_eq!(canonical.matches(&needle).count(), 3);
    let duplicate = canonical.replacen(&needle, &format!("{needle},{needle}"), 1);
    assert!(
        CanonicalDocument::parse(CURRENT, duplicate.as_bytes()).is_err(),
        "duplicate public provenance key"
    );
    for invalid in [
        format!(" {canonical}"),
        format!("{canonical}\n"),
        "not json".to_owned(),
        "[]".to_owned(),
        "{}".to_owned(),
        canonical.replacen("\"byte_length\":1,", "\"byte_length\":0.5,", 1),
    ] {
        assert_ne!(
            invalid, canonical,
            "negative must actually change the public document"
        );
        assert!(
            CanonicalDocument::parse(CURRENT, invalid.as_bytes()).is_err(),
            "strict public canonical document"
        );
    }
    for path in ACQUISITIONS {
        let mut changed = fixture();
        changed.pointer_mut(path).unwrap()["private_authority"] = json!(true);
        assert!(
            CanonicalDocument::from_value(CURRENT, changed).is_err(),
            "caller assertion cannot mint private authority"
        );
    }
}
