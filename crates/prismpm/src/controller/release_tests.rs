//! OC-02: genuine named-release verification, not synthetic verification receipts.

use super::*;
use serde_json::Value;
use std::collections::BTreeMap;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned()
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.unwrap();
        let path = destination.join(entry.path().strip_prefix(source).unwrap());
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(path).unwrap();
        } else {
            assert!(entry.file_type().is_file());
            std::fs::copy(entry.path(), path).unwrap();
        }
    }
}

fn calculator_project() -> tempfile::TempDir {
    let temporary = tempfile::tempdir().unwrap();
    let example = repository().join("examples/Calculator");
    copy_tree(&example.join("src"), &temporary.path().join("src"));
    for path in [
        "lexlean.toml",
        "prismpm.toml",
        "lakefile.toml",
        "lake-manifest.json",
        "lean-toolchain",
    ] {
        std::fs::copy(example.join(path), temporary.path().join(path)).unwrap();
    }
    relock(temporary.path());
    temporary
}

fn relock(root: &Path) {
    Engine::load(&utf8(root.join("lexlean.toml")).unwrap())
        .unwrap()
        .lock(lexlean::LockRequest {
            check_only: false,
            allow_network: false,
        })
        .unwrap();
}

fn semantic_data(source: &str) -> Value {
    let (_, payload) = source.split_once("\\semanticdata{").unwrap();
    serde_json::Deserializer::from_str(payload)
        .into_iter::<Value>()
        .next()
        .unwrap()
        .unwrap()
}

// Render test data through the actual stdlib structure declarations. The
// fixture contains only field values; LexLean still typechecks the generated
// semantic module and Lean proves both exact SystemReleaseReady statements.
fn term(value: &Value, ty: &Value, module: &str, types: &BTreeMap<String, Value>) -> Value {
    match ty["kind"].as_str().unwrap() {
        "string" | "bool" => json!({"kind":ty["kind"], "value":value}),
        "nat" => json!({"kind":"nat", "value":value.as_str().unwrap().parse::<u64>().unwrap().to_string()}),
        "uint8" | "uint16" | "uint32" | "uint64" => {
            json!({"kind":"integer", "representation":ty["kind"], "value":value.to_string()})
        }
        "list" => value.as_array().unwrap().iter().rev().fold(
            json!({"kind":"nil", "element":qualified_type(&ty["element"], module)}),
            |tail, head| json!({"kind":"cons", "head":term(head, &ty["element"], module, types), "tail":tail}),
        ),
        "option" => json!({
            "kind":"constructor",
            "constructor":{"name":if value.is_null() {"Option.none"} else {"Option.some"}},
            "type_arguments":[qualified_type(&ty["value"], module)],
            "arguments":if value.is_null() {vec![]} else {vec![term(value, &ty["value"], module, types)]}
        }),
        "named" => {
            let name = ty["member"]["name"].as_str().unwrap();
            let module = ty["member"]["module"].as_str().unwrap_or(module);
            let declaration = &types[&format!("{module}.{name}")];
            let fields = declaration["fields"].as_array().unwrap();
            assert_eq!(fields.len(), value.as_object().unwrap().len());
            json!({"kind":"record", "type":{"module":module,"name":name}, "fields":fields.iter().map(|field| {
                let name = field["name"].as_str().unwrap();
                json!({"field":name,"value":term(&value[name], &field["type"], module, types)})
            }).collect::<Vec<_>>()})
        }
        kind => panic!("unhandled fixture type {kind}"),
    }
}

fn qualified_type(ty: &Value, module: &str) -> Value {
    let mut ty = ty.clone();
    match ty["kind"].as_str().unwrap() {
        "named" if ty["member"]["module"].is_null() => ty["member"]["module"] = json!(module),
        "list" => ty["element"] = qualified_type(&ty["element"], module),
        "option" => ty["value"] = qualified_type(&ty["value"], module),
        _ => {}
    }
    ty
}

fn named(name: &str) -> Value {
    json!({"kind":"named", "member":{"module":"Production.System","name":name},"arguments":[]})
}

fn named_releases() -> tempfile::TempDir {
    let temporary = calculator_project();
    let root = temporary.path();
    // Calculator production scenario from calculator-example@5051dc6adb18e9ca2b6ae4ca7090e638329831ef,
    // with its complete certificate regenerated against the current stdlib.
    // B differs only in release identity; no application dependencies are removed.
    let mut fixture: Value =
        serde_json::from_str(include_str!("../system/release-fixture.json")).unwrap();
    copy_tree(
        &repository().join("stdlib/src/Production"),
        &root.join("src/Production"),
    );
    let mut types = BTreeMap::new();
    for module in [
        "Core",
        "Interface",
        "Runtime",
        "Operations",
        "Validation",
        "System",
    ] {
        let source =
            std::fs::read_to_string(root.join(format!("src/Production/{module}.lex.tex"))).unwrap();
        for declaration in semantic_data(&source)["declarations"].as_array().unwrap() {
            if declaration["kind"] == "structure" {
                types.insert(
                    format!(
                        "Production.{module}.{}",
                        declaration["name"].as_str().unwrap()
                    ),
                    declaration.clone(),
                );
            }
        }
    }
    let mut declarations = Vec::new();
    for release in ["A", "B"] {
        fixture["model"]["product"]["version"] = json!(release);
        for (name, field, ty) in [
            ("systemModel", "model", "SystemModel"),
            ("systemManifest", "manifest", "SystemManifest"),
        ] {
            declarations.push(json!({"kind":"definition","name":format!("{name}{release}"),"parameters":[],"result":named(ty),"body":term(&fixture[field], &named(ty), "Production.System", &types)}));
        }
        declarations.push(json!({
            "kind":"theorem", "name":format!("release{release}SystemReleaseReady"),"parameters":[],"axioms":[],"proof":{"kind":"decide"},
            "statement":{"kind":"eq","left":{"kind":"call","function":{"module":"Production.SystemValidation","name":"validateManifest"},"arguments":[
                {"kind":"call","function":{"name":format!("systemModel{release}")},"arguments":[]},
                {"kind":"call","function":{"name":format!("systemManifest{release}")},"arguments":[]}
            ]},"right":{"kind":"bool","value":true}}
        }));
    }
    std::fs::write(root.join("src/Release.lex.tex"), format!(
        "\\begin{{lexlean}}{{Release}}\n\\useglossary{{lexlean.std.bool@1.1.0}}\n\\useglossary{{lexlean.std.nat@1.1.0}}\n\\importmodule{{Calculator}}\n\\importmodule{{Production.Core}}\n\\importmodule{{Production.Interface}}\n\\importmodule{{Production.Runtime}}\n\\importmodule{{Production.Operations}}\n\\importmodule{{Production.Validation}}\n\\importmodule{{Production.System}}\n\\importmodule{{Production.SystemValidation}}\n\\title{{Boolean}}\n\\begin{{semanticmodule}}\n\\semanticdata{{{}}}\n\\end{{semanticmodule}}\n\\end{{lexlean}}\n",
        serde_json::to_string(&json!({"spec":"lexlean/semantic-module/1","declarations":declarations})).unwrap()
    )).unwrap();
    let config = std::fs::read_to_string(root.join("lexlean.toml"))
        .unwrap()
        .replace(
            "entrypoints = [\"src/Calculator.lex.tex\"]",
            "entrypoints = [\"src/Release.lex.tex\"]",
        );
    std::fs::write(root.join("lexlean.toml"), config).unwrap();
    relock(root);
    // Application identity includes the final project configuration. Bind the
    // exact selected application snapshot only after installing that config.
    let snapshot = Engine::load(&utf8(root.join("lexlean.toml")).unwrap())
        .unwrap()
        .snapshot(LexCheckRequest {
            selection: Selection::Files([Utf8PathBuf::from("src/Calculator.lex.tex")].into()),
        })
        .unwrap();
    let application = crate::holo::application::project_application(&snapshot)
        .unwrap()
        .unwrap();
    let digest = format!(
        "sha256:{}",
        content_id(&encode_canonical(&application).unwrap())
    );
    let source = std::fs::read_to_string(root.join("src/Release.lex.tex")).unwrap();
    let source = source.replace(
        fixture["model"]["applicationProfile"]["applicationModelDigest"]
            .as_str()
            .unwrap(),
        &digest,
    );
    std::fs::write(root.join("src/Release.lex.tex"), source).unwrap();
    relock(root);
    temporary
}

fn assert_receipt(root: &Path, receipt: &VerifyResult, build: &BuildResult) {
    assert_eq!(receipt.build_id, build.build_id);
    let verified = root.join(&receipt.verified_root);
    let bytes = std::fs::read(verified.join("manifest.json")).unwrap();
    assert_eq!(content_id(&bytes), receipt.attestation_id);
    let manifest: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(encode_value(&manifest).unwrap(), bytes);
    assert_eq!(manifest["build_id"], build.build_id);
    assert_eq!(
        manifest["model_sha256"],
        content_id(&std::fs::read(root.join(&build.model_path)).unwrap())
    );
    let acceptance_bytes = std::fs::read(verified.join("application-acceptance.json")).unwrap();
    assert_eq!(manifest["acceptance_sha256"], content_id(&acceptance_bytes));
    let acceptance: Value = serde_json::from_slice(&acceptance_bytes).unwrap();
    assert_eq!(acceptance["build_id"], build.build_id);
    assert_eq!(acceptance["source_id"], build.source_id);
    assert_eq!(acceptance["status"], "verified");
    let lexlean_bytes = std::fs::read(verified.join("lexlean-attestation.json")).unwrap();
    assert_eq!(
        manifest["lexlean_attestation_sha256"],
        content_id(&lexlean_bytes)
    );
    let lexlean: Value = serde_json::from_slice(&lexlean_bytes).unwrap();
    assert_eq!(lexlean["source_id"], build.source_id);
    assert_eq!(lexlean["semantic_id"], build.semantic_id);
    assert_eq!(lexlean["status"], "verified");
}

#[test]
fn explicit_release_requires_a_system_graph() {
    let temporary = calculator_project();
    let controller = Controller::load(temporary.path()).unwrap();
    assert!(controller.prepare(None).unwrap().system.is_none());
    for release in ["A", "Missing", "../A", ""] {
        let error = controller
            .verify_release(VerifyRequest { config_path: None }, Some(release))
            .unwrap_err();
        assert_eq!(error.code, "PP2101");
    }
    assert!(!temporary.path().join(".prism/verified").exists());
    assert!(!temporary.path().join(".prism/build").exists());
}

#[test]
fn alternate_release_verification_is_bound_to_the_selected_build() {
    let temporary = named_releases();
    let root = temporary.path();
    let controller = Controller::load(root).unwrap();
    let a = controller
        .build_release(BuildRequest { config_path: None }, Some("A"))
        .unwrap();
    let b = controller
        .build(BuildRequest { config_path: None })
        .unwrap();
    assert_ne!(a.build_id, b.build_id);
    assert_eq!(a.source_id, b.source_id);
    assert_eq!(a.semantic_id, b.semantic_id);
    for (build, version) in [(&a, "A"), (&b, "B")] {
        let system: Value = serde_json::from_slice(
            &std::fs::read(
                root.join(".prism/build")
                    .join(&build.build_id)
                    .join("system.prism.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(system["product"]["version"], version);
    }
    let verified_a = controller
        .verify_release(VerifyRequest { config_path: None }, Some("A"))
        .unwrap();
    assert_receipt(root, &verified_a, &a);
    require_verified_build(&verified_a, &a).unwrap();
    assert_eq!(
        require_verified_build(&verified_a, &b).unwrap_err().code,
        "PP6101"
    );
    let verified_b = controller
        .verify(VerifyRequest { config_path: None })
        .unwrap();
    assert_receipt(root, &verified_b, &b);
    require_verified_build(&verified_b, &b).unwrap();
    assert_eq!(
        require_verified_build(&verified_b, &a).unwrap_err().code,
        "PP6101"
    );
    for release in ["Missing", "../A", ""] {
        assert_eq!(
            controller
                .verify_release(VerifyRequest { config_path: None }, Some(release))
                .unwrap_err()
                .code,
            "PP2101"
        );
    }
}
