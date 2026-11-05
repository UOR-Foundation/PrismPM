//! Binary package contract regression tests; no application or deployment scope.
use prismpm::holo::canonical::{decode_canonical, encode_value};
use serde_json::{json, Value};
fn document() -> Value {
    let mut value: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/data/text-model-document.json"
    ))
    .unwrap();
    value.as_object_mut().unwrap().remove("application");
    value["schema"] = json!("prismpm/model-document/7");
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
    assert_eq!(
        prismpm::holo::canonical::encode_canonical(&model).unwrap(),
        bytes
    );
    prismpm::contracts::CanonicalDocument::from_value("prismpm/model-document/7", value).unwrap();
}
#[test]
fn binary_raw_file_v1_transport_rejects_bounds_above_sixteen_mebibytes() {
    use prismpm::contracts::CanonicalDocument;
    for field in ["request_maximum", "response_maximum"] {
        let mut value = document();
        value["program"][field] = json!(16_777_217);
        // Memory is sufficient, so refusal measures the existing transport profile's cap.
        value["program"]["memory_pages"] = json!(2048);
        assert!(CanonicalDocument::from_value("prismpm/model-document/7", value.clone()).is_err());
        assert!(decode_canonical(&encode_value(&value).unwrap()).is_err());
        let program = value["program"].clone();
        let binding = json!({
            "schema": "prismpm/binary-build-binding/1",
            "profile": "prismpm/binary-program/1",
            "scope": "binary-package-only",
            "model_id": "a".repeat(64),
            "program": program
        });
        assert!(CanonicalDocument::from_value("prismpm/binary-build-binding/1", binding).is_err());
        let mut accepted = acceptance();
        accepted["program"] = program;
        assert!(CanonicalDocument::from_value("prismpm/binary-acceptance/1", accepted).is_err());
    }
}

#[test]
fn binary_document_version_is_distinct_from_both_existing_text_contracts() {
    use prismpm::contracts::CanonicalDocument;
    let binary = document();
    CanonicalDocument::from_value("prismpm/model-document/7", binary.clone()).unwrap();
    for schema in ["prismpm/model-document/5", "prismpm/model-document/6"] {
        let mut mislabeled = binary.clone();
        mislabeled["schema"] = json!(schema);
        assert!(CanonicalDocument::from_value(schema, mislabeled.clone()).is_err());
        assert!(decode_canonical(&encode_value(&mislabeled).unwrap()).is_err());
    }

    let mut text_v2: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/data/text-model-document.json"
    ))
    .unwrap();
    text_v2["schema"] = json!("prismpm/model-document/5");
    text_v2["application"]["profile"] = json!("prismpm/text-application/2");
    text_v2["application"]["core_memory_pages"] = json!(64);
    text_v2["application"]["core_stack_bytes"] = json!(131072);
    let text_v3: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/data/workbench-model-document.json"
    ))
    .unwrap();
    for (schema, text) in [
        ("prismpm/model-document/5", text_v2),
        ("prismpm/model-document/6", text_v3),
    ] {
        CanonicalDocument::from_value(schema, text.clone()).unwrap();
        let mut mislabeled = text.clone();
        mislabeled["schema"] = json!("prismpm/model-document/7");
        assert!(CanonicalDocument::from_value("prismpm/model-document/7", mislabeled).is_err());
    }
}

#[test]
fn binary_program_rejects_missing_cli_bad_bounds_roots_and_profile_confusion() {
    for (pointer, replacement) in [
        ("/schema", json!("prismpm/model-document/3")),
        ("/program", Value::Null),
        ("/program/cli", Value::Null),
        ("/program/cli/profile", json!("optional")),
        ("/program/request_maximum", json!(0)),
        ("/program/response_maximum", json!(0)),
        ("/program/memory_pages", json!(0)),
        ("/program/memory_pages", json!(32768)),
        ("/program/entry_root", json!("BinaryProbe.Probe.absent")),
        (
            "/program/export_roots",
            json!(["BinaryProbe.Probe.identity", "BinaryProbe.Probe.identity"]),
        ),
        ("/program/acceptance_vectors", json!([])),
        ("/program/acceptance_vectors/1/request", json!(vec![0; 257])),
        (
            "/program/acceptance_vectors/1/response",
            json!(vec![0; 257]),
        ),
    ] {
        let mut value = document();
        *value.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            decode_canonical(&encode_value(&value).unwrap()).is_err(),
            "{pointer}"
        );
    }
    for pointer in [
        "",
        "/program",
        "/program/cli",
        "/program/acceptance_vectors/0",
    ] {
        let mut value = document();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unknown".into(), json!(true));
        assert!(
            decode_canonical(&encode_value(&value).unwrap()).is_err(),
            "{pointer}"
        );
    }
}

fn acceptance() -> Value {
    let program = document()["program"].clone();
    let executions = [
        "std",
        "no_std",
        "core-wasm",
        "cli-stdio",
        "cli-file",
        "cli-mixed",
    ]
    .iter()
    .map(|mode| json!({"mode": mode, "status": "passed", "vector_count": 2}))
    .collect::<Vec<_>>();
    json!({"schema":"prismpm/binary-acceptance/1","profile":"prismpm/binary-program/1","scope":"binary-package-only", "model_id":"a".repeat(64),"build_id":"b".repeat(64),"lexlean_attestation_id":"c".repeat(64),"program":program,
        "executions": executions,
        "io_coverage":{"platform":"linux","output_write":"passed"},"regeneration":"byte-identical","status":"passed","unclaimed":["application","browser","holo","production-release","deployment"]})
}
#[test]
fn binary_acceptance_rejects_vector_transport_omission_and_scope_substitution() {
    use prismpm::contracts::CanonicalDocument;
    let valid = acceptance();
    CanonicalDocument::from_value("prismpm/binary-acceptance/1", valid.clone()).unwrap();
    for mutation in 0..6 {
        let mut changed = valid.clone();
        match mutation {
            0 => {
                changed["program"]["acceptance_vectors"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            1 => {
                changed["executions"].as_array_mut().unwrap().pop();
            }
            2 => changed["executions"][0]["vector_count"] = json!(1),
            3 => changed["executions"][0]["mode"] = json!("core-wasm"),
            4 => changed["scope"] = json!("production-release"),
            _ => changed["regeneration"] = json!("not-checked"),
        }
        assert!(CanonicalDocument::from_value("prismpm/binary-acceptance/1", changed).is_err());
    }
}

#[test]
fn binary_acceptance_explicitly_scopes_empty_response_io_coverage() {
    use prismpm::contracts::CanonicalDocument;
    let mut value = acceptance();
    for vector in value["program"]["acceptance_vectors"]
        .as_array_mut()
        .unwrap()
    {
        vector["response"] = json!([]);
    }
    assert!(CanonicalDocument::from_value("prismpm/binary-acceptance/1", value.clone()).is_err());
    value["io_coverage"]["output_write"] = json!("not-exercised-empty-responses");
    CanonicalDocument::from_value("prismpm/binary-acceptance/1", value.clone()).unwrap();
    value.as_object_mut().unwrap().remove("io_coverage");
    assert!(CanonicalDocument::from_value("prismpm/binary-acceptance/1", value).is_err());
}

fn source_fixture() -> tempfile::TempDir {
    fn copy(source: &std::path::Path, destination: &std::path::Path) {
        std::fs::create_dir_all(destination).unwrap();
        for entry in std::fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            // Local qualification may have populated the repository fixture;
            // every front-end probe starts from its source and lock alone.
            if matches!(
                entry.file_name().to_str(),
                Some(".prism" | ".lexlean" | ".lake")
            ) {
                continue;
            }
            let target = destination.join(entry.file_name());
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                copy(&entry.path(), &target);
            } else {
                assert!(kind.is_file());
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    let project = tempfile::tempdir().unwrap();
    copy(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/binary/binary-program/project"),
        project.path(),
    );
    project
}

fn source_tree(
    root: &std::path::Path,
) -> std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .map(Result::unwrap)
        .map(|entry| {
            let kind = entry.file_type();
            assert!(kind.is_file() || kind.is_dir());
            (
                entry.path().strip_prefix(root).unwrap().to_owned(),
                kind.is_file().then(|| std::fs::read(entry.path()).unwrap()),
            )
        })
        .collect()
}

fn use_source_language_1_2(project: &std::path::Path) {
    // Generic definitions belong to language 1.2. Prepare an actual temporary
    // source/lock pair rather than expecting a language-1.1 syntax error.
    let config_path = project.join("lexlean.toml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    assert!(config.contains("language = \"1.1\""));
    std::fs::write(
        &config_path,
        config.replace("language = \"1.1\"", "language = \"1.2\""),
    )
    .unwrap();
    for entry in walkdir::WalkDir::new(project.join("src")) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            let source = std::fs::read_to_string(entry.path())
                .unwrap()
                .replace("@1.1.0", "@1.2.0");
            let mut module = lexlean::sdk::read_module(&source).unwrap();
            module["spec"] = json!("lexlean/semantic-module/2");
            let source = lexlean::sdk::write_module(&source, &module).unwrap();
            std::fs::write(entry.path(), lexlean::sdk::to_notation(&source).unwrap()).unwrap();
        }
    }
    let config = camino::Utf8PathBuf::from_path_buf(config_path).unwrap();
    lexlean::Engine::load(&config)
        .unwrap()
        .lock(lexlean::LockRequest {
            check_only: false,
            allow_network: false,
        })
        .expect("lock the real language-1.2 builtin closure offline");
    // Lock preparation owns this directory; subsequent source checks must
    // create neither this lock directory nor any build/verification output.
    std::fs::remove_dir_all(project.join(".lexlean")).unwrap();
}

fn source_snapshot(project: &std::path::Path) -> lexlean::SemanticSnapshot {
    let config = camino::Utf8PathBuf::from_path_buf(project.join("lexlean.toml")).unwrap();
    lexlean::Engine::load(&config)
        .unwrap()
        .snapshot(lexlean::CheckRequest {
            selection: lexlean::Selection::Entrypoints,
        })
        .expect("the mutated fixture must remain valid typed LexLean source")
}

fn source_declaration<'a>(module: &'a mut Value, name: &str) -> &'a mut Value {
    module["declarations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|declaration| declaration["name"] == name)
        .unwrap()
}

fn source_program_field<'a>(module: &'a mut Value, name: &str) -> &'a mut Value {
    &mut source_declaration(module, "probeProgram")["body"]["fields"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|field| field["field"] == name)
        .unwrap()["value"]
}

#[test]
fn binary_source_check_is_read_only_and_binds_the_selected_typed_snapshot() {
    use prismpm::controller::CheckRequest;
    for language_1_2 in [false, true] {
        let project = source_fixture();
        if language_1_2 {
            use_source_language_1_2(project.path());
        }
        let before = source_tree(project.path());
        let checked = prismpm::Controller::load(project.path())
            .unwrap()
            .check(CheckRequest { config_path: None })
            .expect("the real binary source fixture must pass Controller::check");
        let snapshot = source_snapshot(project.path());
        let model = prismpm::holo::binary_program::project_program(&snapshot)
            .unwrap()
            .unwrap();
        assert_eq!(checked.entity_count, 1);
        assert_eq!(checked.semantic_id, snapshot.semantic_id().to_string());
        assert_eq!(checked.snapshot_id, snapshot.snapshot_id().to_string());
        assert_eq!(model.schema, "prismpm/model-document/7");
        assert_eq!(model.provenance.source_id, snapshot.source_id().to_string());
        assert_eq!(
            checked.model_id,
            prismpm::holo::canonical::content_id(
                &prismpm::holo::canonical::encode_canonical(&model).unwrap()
            )
        );
        assert_eq!(source_tree(project.path()), before);
        for output in [".prism", ".lexlean", ".lake"] {
            assert!(!project.path().join(output).exists(), "{output}");
        }
    }
}

#[test]
fn binary_source_check_rejects_unexported_missing_and_nonmonomorphic_entries_without_writes() {
    use prismpm::controller::CheckRequest;
    for mutation in [
        "missing entry",
        "entry outside exports",
        "wrong input type",
        "wrong result type",
        "extra entry parameter",
        "generic entry",
        "parameterized descriptor",
        "generic descriptor",
    ] {
        let project = source_fixture();
        if matches!(mutation, "generic entry" | "generic descriptor") {
            use_source_language_1_2(project.path());
        }
        let path = project.path().join("src/Probe.lex.tex");
        let original = std::fs::read_to_string(&path).unwrap();
        let mut module: Value = lexlean::sdk::read_module(&original).unwrap();
        match mutation {
            "missing entry" => {
                source_program_field(&mut module, "entryRoot")["value"] =
                    json!("LibraryProbe.Probe.absent");
                source_program_field(&mut module, "exportRoots")["head"]["value"] =
                    json!("LibraryProbe.Probe.absent");
            }
            "entry outside exports" => {
                let mut other = source_declaration(&mut module, "identity").clone();
                other["name"] = json!("other");
                module["declarations"].as_array_mut().unwrap().push(other);
                source_program_field(&mut module, "exportRoots")["head"]["value"] =
                    json!("LibraryProbe.Probe.other");
            }
            "wrong input type" => {
                let entry = source_declaration(&mut module, "identity");
                entry["parameters"][0]["type"] = json!({"kind":"nat"});
                entry["body"] = json!({"kind":"bytes","hex":""});
            }
            "wrong result type" => {
                let entry = source_declaration(&mut module, "identity");
                entry["result"] = json!({"kind":"nat"});
                entry["body"] = json!({"kind":"nat","value":"0"});
            }
            "extra entry parameter" => {
                source_declaration(&mut module, "identity")["parameters"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"name":"extra","type":{"kind":"nat"}}));
            }
            "generic entry" => {
                source_declaration(&mut module, "identity")["type_parameters"] = json!(["T"]);
            }
            "parameterized descriptor" => {
                source_declaration(&mut module, "probeProgram")["parameters"] =
                    json!([{"name":"extra","type":{"kind":"nat"}}]);
            }
            "generic descriptor" => {
                source_declaration(&mut module, "probeProgram")["type_parameters"] = json!(["T"]);
            }
            _ => unreachable!(),
        }
        std::fs::write(
            &path,
            lexlean::sdk::write_module(&original, &module).unwrap(),
        )
        .unwrap();
        let before = source_tree(project.path());
        // Establish that the negative reaches Prism's source-contract checks,
        // rather than passing because the source failed to parse or typecheck.
        let _snapshot = source_snapshot(project.path());
        let error = prismpm::Controller::load(project.path())
            .unwrap()
            .check(CheckRequest { config_path: None })
            .expect_err(mutation);
        let (code, message) = match mutation {
            "missing entry" => (
                "PP2001",
                "binary-program export LibraryProbe.Probe.absent is not defined",
            ),
            "entry outside exports" => ("PP4004", "acceptance roots must be declared exports"),
            "parameterized descriptor" | "generic descriptor" => (
                "PP4004",
                "binary-program descriptor must be closed and monomorphic",
            ),
            _ => (
                "PP2001",
                "binary-program entry must be monomorphic Bytes to Bytes",
            ),
        };
        assert_eq!(error.code.as_str(), code, "{mutation}: {error:?}");
        assert!(error.message.contains(message), "{mutation}: {error:?}");
        assert_eq!(source_tree(project.path()), before, "{mutation}");
        for output in [".prism", ".lexlean", ".lake"] {
            assert!(
                !project.path().join(output).exists(),
                "{mutation}: {output}"
            );
        }
    }
}

#[test]
fn binary_source_check_rejects_a_genuine_native_library_descriptor_without_writes() {
    use prismpm::controller::CheckRequest;
    let project = source_fixture();
    use_source_language_1_2(project.path());
    let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/library/native-library/project/src");
    let library_model = project
        .path()
        .join("src/Foundation/Library/V1/Model.lex.tex");
    std::fs::create_dir_all(library_model.parent().unwrap()).unwrap();
    std::fs::copy(
        library.join("Foundation/Library/V1/Model.lex.tex"),
        library_model,
    )
    .unwrap();
    let path = project.path().join("src/Probe.lex.tex");
    let original = std::fs::read_to_string(&path).unwrap().replace(
        "\\importmodule{Foundation.Binary.V1.Model}",
        "\\importmodule{Foundation.Binary.V1.Model}\n\\importmodule{Foundation.Library.V1.Model}",
    );
    let mut module = lexlean::sdk::read_module(&original).unwrap();
    // Keep the genuine native-library fixture's identity and acceptance roots
    // intact, giving only the binary fixture's independent entry a fresh name.
    source_declaration(&mut module, "identity")["name"] = json!("binaryIdentity");
    source_program_field(&mut module, "entryRoot")["value"] =
        json!("LibraryProbe.Probe.binaryIdentity");
    source_program_field(&mut module, "exportRoots")["head"]["value"] =
        json!("LibraryProbe.Probe.binaryIdentity");
    let mut library_module =
        lexlean::sdk::read_module(&std::fs::read_to_string(library.join("Probe.lex.tex")).unwrap())
            .unwrap();
    module["declarations"]
        .as_array_mut()
        .unwrap()
        .append(library_module["declarations"].as_array_mut().unwrap());
    std::fs::write(
        &path,
        lexlean::sdk::write_module(&original, &module).unwrap(),
    )
    .unwrap();
    let before = source_tree(project.path());
    let snapshot = source_snapshot(project.path());
    assert!(prismpm::holo::binary_program::project_program(&snapshot)
        .unwrap()
        .is_some());
    assert!(prismpm::holo::library::project_library(&snapshot)
        .unwrap()
        .is_some());
    let error = prismpm::Controller::load(project.path())
        .unwrap()
        .check(CheckRequest { config_path: None })
        .unwrap_err();
    assert_eq!(error.code.as_str(), "PP4004");
    assert_eq!(
        error.message,
        "a binary program cannot also declare a native library, application or system release"
    );
    assert_eq!(source_tree(project.path()), before);
    for output in [".prism", ".lexlean", ".lake"] {
        assert!(!project.path().join(output).exists(), "{output}");
    }
}
