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

#[test]
fn physical_archive_limit_rejects_before_publishing_any_build() {
    let temporary = calculator_project();
    let root = temporary.path();
    let controller = Controller::load(root).unwrap();
    let initial = controller
        .build(BuildRequest { config_path: None })
        .unwrap();
    let directory = root.join(".prism/build").join(&initial.build_id);
    let archive = walkdir::WalkDir::new(&directory)
        .into_iter()
        .map(Result::unwrap)
        .find(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|value| value == "holo")
        })
        .unwrap();
    let archive_bytes = std::fs::read(archive.path()).unwrap();
    let model_bytes = std::fs::read(root.join(&initial.model_path)).unwrap();
    assert!(archive_bytes.len() > model_bytes.len());
    let builds = || {
        std::fs::read_dir(root.join(".prism/build"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<BTreeSet<_>>()
    };
    let initial_builds = builds();
    let config_path = root.join("prismpm.toml");
    let mut config: toml::Value =
        toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    config["limits"]["max_holo_bytes"] = toml::Value::Integer(archive_bytes.len() as i64 - 1);
    std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();
    // The model fits; it is the physical archive boundary that must reject.
    controller
        .check(CheckRequest { config_path: None })
        .unwrap();
    assert_eq!(
        controller
            .build(BuildRequest { config_path: None })
            .unwrap_err()
            .code,
        "PP1003"
    );
    assert_eq!(builds(), initial_builds);
    assert_eq!(std::fs::read(archive.path()).unwrap(), archive_bytes);
    config["limits"]["max_holo_bytes"] = toml::Value::Integer(archive_bytes.len() as i64);
    std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();
    controller
        .build(BuildRequest { config_path: None })
        .unwrap();
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

    let build_root = root.join(&build.manifest_path).parent().unwrap().to_owned();
    let build_manifest_bytes = std::fs::read(root.join(&build.manifest_path)).unwrap();
    let build_manifest: Value = serde_json::from_slice(&build_manifest_bytes).unwrap();
    let build_files = build_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let name = row["path"].as_str().unwrap();
            (
                name.to_owned(),
                std::fs::read(build_root.join(name)).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let verification_files = walkdir::WalkDir::new(&verified)
        .min_depth(1)
        .into_iter()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.file_type().is_file() && !entry.file_type().is_symlink());
            (
                entry
                    .path()
                    .strip_prefix(&verified)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                std::fs::read(entry.path()).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let binding = crate::release_verification::validate(
        &build_manifest_bytes,
        &build_files,
        &verification_files,
    )
    .unwrap();
    assert_eq!(binding.build_id, build.build_id);
    assert_eq!(
        binding.build_digest,
        format!("sha256:{}", content_id(&build_manifest_bytes))
    );
    assert_eq!(
        binding.model_digest,
        format!("sha256:{}", content_id(&build_files["model.prism.json"]))
    );
    assert_eq!(binding.attestation_id, receipt.attestation_id);
    assert_eq!(binding.family, "application");
    crate::release_verification::tests::reject_mutations(
        &build_manifest_bytes,
        &build_files,
        &verification_files,
    );
}

struct BrowserRelease {
    reference: String,
    digest: String,
    model_digest: String,
    build_digest: String,
    files: BTreeMap<String, Vec<u8>>,
    layout: BTreeMap<String, Vec<u8>>,
}

fn browser_release(root: &Path, build: &BuildResult, verified: &VerifyResult) -> BrowserRelease {
    let staging = tempfile::tempdir().unwrap();
    let (store, descriptor) = crate::oci::browser_export_fixture(
        staging.path(),
        "example.invalid/calculator:fixture",
        root,
        build,
        verified,
    );
    let build_root = root.join(&build.manifest_path).parent().unwrap().to_owned();
    let files = [
        "app.css",
        "app.js",
        "index.html",
        "prism_calculator.js",
        "prism_calculator_bg.wasm",
        "provenance.json",
    ]
    .into_iter()
    .map(|name| {
        (
            name.to_owned(),
            std::fs::read(build_root.join("view/browser").join(name)).unwrap(),
        )
    })
    .collect();
    let mut layout = BTreeMap::new();
    let marker = format!(
        "verified/{}.json",
        descriptor.digest.strip_prefix("sha256:").unwrap()
    );
    for entry in walkdir::WalkDir::new(store.root()).min_depth(1) {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        assert!(entry.file_type().is_file() && !entry.file_type().is_symlink());
        let name = entry.path().strip_prefix(store.root()).unwrap();
        if name.starts_with("blobs")
            || name == Path::new("index.json")
            || name == Path::new("oci-layout")
            || name == Path::new(&marker)
        {
            layout.insert(
                name.to_str().unwrap().to_owned(),
                std::fs::read(entry.path()).unwrap(),
            );
        }
    }
    // This is publication-consistency metadata, not a substitute for replaying
    // the complete proof closure. Its removal/forgery is tested below.
    assert!(layout.contains_key(&marker));
    let release = BrowserRelease {
        reference: format!("example.invalid/calculator@{}", descriptor.digest),
        digest: descriptor.digest,
        model_digest: format!(
            "sha256:{}",
            content_id(&std::fs::read(root.join(&build.model_path)).unwrap())
        ),
        build_digest: format!(
            "sha256:{}",
            content_id(&std::fs::read(root.join(&build.manifest_path)).unwrap())
        ),
        files,
        layout,
    };
    drop(store);
    staging.close().unwrap();
    release
}

fn browser_receiver(release: &BrowserRelease) -> tempfile::TempDir {
    let boundary = tempfile::tempdir().unwrap();
    let receiver = boundary.path().join("receiver");
    std::fs::create_dir(&receiver).unwrap();
    for (name, bytes) in &release.layout {
        let destination = receiver.join(".prism/oci").join(name);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, bytes).unwrap();
    }
    for absent in [
        "src",
        "prismpm.toml",
        ".lexlean",
        ".prism/build",
        ".prism/verified",
    ] {
        assert!(!receiver.join(absent).exists());
    }
    boundary
}

fn assert_browser_refused(root: &Path, reference: &str, output: &Path) {
    let before = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<BTreeSet<_>>();
    let error = Controller::load(root)
        .unwrap()
        .export_browser(ExportBrowserRequest {
            reference: reference.to_owned(),
            output: output.to_owned(),
        })
        .unwrap_err();
    assert!(
        matches!(error.code.as_str(), "PP6101" | "PP8001"),
        "{error:?}"
    );
    let after = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<BTreeSet<_>>();
    assert_eq!(after, before, "failure left an output or staging directory");
}

fn assert_source_free_browser_export(release: &BrowserRelease) {
    let boundary = browser_receiver(release);
    let root = boundary.path().join("receiver");
    let result = Controller::load(&root)
        .unwrap()
        .export_browser(ExportBrowserRequest {
            reference: release.reference.clone(),
            output: "browser-output".into(),
        })
        .unwrap();
    let rows = release
        .files
        .iter()
        .map(|(path, bytes)| {
            json!({"path":path,"digest":format!("sha256:{}",content_id(bytes)),"size":bytes.len()})
        })
        .collect::<Vec<_>>();
    assert_eq!(
        result,
        json!({
            "schema":"prismpm/browser-export/1",
            "reference":release.reference,
            "release_digest":release.digest,
            "model_digest":release.model_digest,
            "build_digest":release.build_digest,
            "output":"browser-output",
            "files":rows,
            "tree_digest":format!("sha256:{}",content_id(&encode_value(&json!(rows)).unwrap()))
        })
    );
    let exported = std::fs::read_dir(root.join("browser-output"))
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_file());
            (
                entry.file_name().to_str().unwrap().to_owned(),
                std::fs::read(entry.path()).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(exported, release.files);
    for absent in [
        "src",
        "prismpm.toml",
        ".lexlean",
        ".prism/build",
        ".prism/verified",
    ] {
        assert!(!root.join(absent).exists(), "export rebuilt {absent}");
    }
    for (name, bytes) in &release.layout {
        assert_eq!(
            std::fs::read(root.join(".prism/oci").join(name)).unwrap(),
            *bytes,
            "export modified the copied OCI closure"
        );
    }
    assert_eq!(
        walkdir::WalkDir::new(root.join(".prism/oci"))
            .into_iter()
            .map(Result::unwrap)
            .filter(|entry| entry.file_type().is_file())
            .count(),
        release.layout.len(),
        "export created additional OCI evidence"
    );

    // Existing directory/file/symlink destinations must never be replaced.
    assert_browser_refused(&root, &release.reference, Path::new("browser-output"));
    for (name, bytes) in &release.files {
        assert_eq!(
            std::fs::read(root.join("browser-output").join(name)).unwrap(),
            *bytes
        );
    }
    assert_eq!(
        std::fs::read_dir(root.join("browser-output"))
            .unwrap()
            .count(),
        6
    );
    std::fs::write(root.join("existing-file"), b"preserve").unwrap();
    assert_browser_refused(&root, &release.reference, Path::new("existing-file"));
    assert_eq!(
        std::fs::read(root.join("existing-file")).unwrap(),
        b"preserve"
    );
    for name in [
        "",
        ".",
        "..",
        "../escape",
        "nested/output",
        "./output",
        ".hidden",
        "-output",
        "with space",
        "with\\slash",
        "with:colon",
        "é",
    ] {
        assert_browser_refused(&root, &release.reference, Path::new(name));
    }
    assert_browser_refused(&root, &release.reference, Path::new(&"a".repeat(129)));
    assert_browser_refused(
        &root,
        &release.reference,
        &boundary.path().join("absolute-output"),
    );
    assert!(!boundary.path().join("absolute-output").exists());
    assert!(!boundary.path().join("escape").exists());
    assert_browser_refused(
        &root,
        "example.invalid/calculator:fixture",
        Path::new("tag-output"),
    );

    #[cfg(unix)]
    {
        let outside = boundary.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        std::fs::write(outside.join("sentinel"), b"preserve").unwrap();
        std::os::unix::fs::symlink(&outside, root.join("linked-output")).unwrap();
        assert_browser_refused(&root, &release.reference, Path::new("linked-output"));
        assert_eq!(
            std::fs::read(outside.join("sentinel")).unwrap(),
            b"preserve"
        );
        assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 1);
    }

    for mutation in [
        "browser-bytes",
        "missing-proof",
        "missing-proof-blob",
        "missing-marker",
        "forged-marker",
    ] {
        let damaged = browser_receiver(release);
        let root = damaged.path().join("receiver");
        let layout = root.join(".prism/oci");
        let mut index: Value =
            serde_json::from_slice(&std::fs::read(layout.join("index.json")).unwrap()).unwrap();
        if matches!(mutation, "missing-marker" | "forged-marker") {
            let marker = layout.join("verified").join(format!(
                "{}.json",
                release.digest.strip_prefix("sha256:").unwrap()
            ));
            if mutation == "missing-marker" {
                std::fs::remove_file(marker).unwrap();
            } else {
                let mut value: Value =
                    serde_json::from_slice(&std::fs::read(&marker).unwrap()).unwrap();
                value["graph_digest"] = json!(format!("sha256:{}", "0".repeat(64)));
                std::fs::write(marker, encode_value(&value).unwrap()).unwrap();
            }
        } else if mutation == "missing-proof" {
            index["manifests"]
                .as_array_mut()
                .unwrap()
                .retain(|row| row["artifactType"] != crate::oci::PRISM_VERIFICATION);
            std::fs::write(layout.join("index.json"), encode_value(&index).unwrap()).unwrap();
        } else {
            let digest = if mutation == "browser-bytes" {
                content_id(&release.files["app.js"])
            } else {
                index["manifests"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|row| row["artifactType"] == crate::oci::PRISM_VERIFICATION)
                    .unwrap()["digest"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("sha256:")
                    .unwrap()
                    .to_owned()
            };
            let blob = layout.join("blobs/sha256").join(digest);
            if mutation == "browser-bytes" {
                std::fs::write(blob, b"tampered browser code").unwrap();
            } else {
                std::fs::remove_file(blob).unwrap();
            }
        }
        assert_browser_refused(&root, &release.reference, Path::new("browser-output"));
        assert!(!root.join("browser-output").exists());
    }

    #[cfg(unix)]
    {
        let linked = browser_receiver(release);
        let root = linked.path().join("receiver");
        let outside = linked.path().join("external-blob");
        std::fs::write(&outside, &release.files["app.js"]).unwrap();
        let blob = root
            .join(".prism/oci/blobs/sha256")
            .join(content_id(&release.files["app.js"]));
        std::fs::remove_file(&blob).unwrap();
        std::os::unix::fs::symlink(&outside, &blob).unwrap();
        assert_browser_refused(&root, &release.reference, Path::new("browser-output"));
        assert_eq!(std::fs::read(outside).unwrap(), release.files["app.js"]);
    }
}

#[test]
fn browser_export_without_a_store_is_no_write() {
    let root = tempfile::tempdir().unwrap();
    assert_browser_refused(
        root.path(),
        &format!("example.invalid/calculator@sha256:{}", "1".repeat(64)),
        Path::new("browser-output"),
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
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
fn modeled_system_projections_pass_all_seven_locked_oracles() {
    let temporary = named_releases();
    let root = temporary.path();
    let controller = Controller::load(root).unwrap();
    let build = controller
        .build_release(BuildRequest { config_path: None }, Some("A"))
        .unwrap();
    let manifest_path = root.join(&build.manifest_path);
    let manifest = std::fs::read(&manifest_path).unwrap();
    let results = crate::oci::projection_oracle_fixture(root, &build);
    assert_eq!(
        results
            .iter()
            .map(|result| result["oracle"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "asyncapi-3.1-schema",
            "cloudevents-1.0-json",
            "compose-fee041b3",
            "kubernetes-1.36.4",
            "openapi-3.2-schema",
            "otel-collector-0.136.0",
            "spdx-3.0.1-model",
        ]
    );
    for result in results {
        assert_eq!(result["schema"], "prismpm/validation-result/1");
        assert_eq!(result["valid"], true);
        assert!(result["evidence_path"].as_str().is_some());
    }
    assert_eq!(std::fs::read(manifest_path).unwrap(), manifest);
    assert!(!root.join(".prism/verified").exists());
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
    let releases = [
        browser_release(root, &a, &verified_a),
        browser_release(root, &b, &verified_b),
    ];
    assert_ne!(releases[0].digest, releases[1].digest);
    let deleted_source = root.to_owned();
    drop(controller);
    temporary.close().unwrap();
    assert!(!deleted_source.exists());
    for release in releases {
        assert_source_free_browser_export(&release);
    }
}
