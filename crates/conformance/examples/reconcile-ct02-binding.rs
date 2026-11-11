//! One-fixture source preparation, not installed SDK behavior or acceptance evidence.
//! Run only inside the existing M0 exact-package admission and clean-source custody.
use camino::Utf8PathBuf;
use lexlean::{CheckRequest as LexCheckRequest, Engine, LockRequest};
use lexlean::project::Selection;
use prismpm::holo::canonical::{content_id, encode_canonical};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};
use std::os::unix::fs::MetadataExt;

type Result<T> = std::result::Result<T, String>;
const BASE: &str = "354038b14c26913b475c8a2cf0294051cb4fd632";
const COMPILER: &str = "bc71b33f9a4ce7f5582c337b5752a7ba9eb49706";
const FIXTURE: &str = "tests/fixtures/controller/ct-02-prism-code-model/project";
const SYSTEM: &str = "src/System.lex.tex";
const APPLICATION: &str = "src/Specification.lex.tex";
fn require(ok: bool, message: &str) -> Result<()> {
    if ok { Ok(()) } else { Err(message.into()) }
}
fn read(path: &Path) -> Result<Vec<u8>> { fs::read(path).map_err(|e| format!("{}: {e}", path.display())) }
fn text(bytes: &[u8]) -> Result<&str> { std::str::from_utf8(bytes).map_err(|e| e.to_string()) }
fn regular(root: &Path, path: &Path) -> Result<()> {
    require(path.starts_with(root), "path escapes fixture")?;
    let mut cursor = root.to_path_buf();
    for component in path.strip_prefix(root).map_err(|e| e.to_string())?.components() {
        cursor.push(component);
        let m = fs::symlink_metadata(&cursor).map_err(|e| e.to_string())?;
        require(!m.file_type().is_symlink(), "aliased fixture input")?;
    }
    let m = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    require(m.is_file() && m.nlink() == 1, "nonregular or shared fixture input")
}
fn capture(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for name in ["src", "language"] {
        for entry in walkdir::WalkDir::new(root.join(name)).follow_links(false) {
            let entry = entry.map_err(|e| e.to_string())?;
            require(!entry.file_type().is_symlink(), "aliased input tree")?;
            if entry.file_type().is_dir() { continue; }
            regular(root, entry.path())?;
            files.insert(entry.path().to_path_buf(), read(entry.path())?);
        }
    }
    for name in ["lexlean.toml", "prismpm.toml", "lexlean.lock"] {
        let path = root.join(name);
        regular(root, &path)?;
        files.insert(path.clone(), read(&path)?);
    }
    Ok(files)
}
fn identities(files: &BTreeMap<PathBuf, Vec<u8>>, root: &Path) -> Value {
    Value::Object(files.iter().map(|(p, b)|
        (p.strip_prefix(root).unwrap().to_string_lossy().into_owned(), json!(content_id(b))))
        .collect())
}
fn field(record: &Value, name: &str) -> Result<usize> {
    require(record["kind"] == "record", "expected typed record")?;
    let fields = record["fields"].as_array().ok_or("record fields are not an array")?;
    let matches: Vec<_> = fields.iter().enumerate().filter(|(_, v)| v["field"] == name).collect();
    require(matches.len() == 1, "missing or duplicate binding field")?;
    Ok(matches[0].0)
}
fn record_type(record: &Value, name: &str) -> Result<()> {
    require(record["kind"] == "record" && record["type"] == json!({
        "module": "Production.BrowserSystem", "name": name
    }), "unexpected binding record type")
}
// Read-only census rejects other digest fields; mutation uses only two closed paths.
fn count_fields(v: &Value) -> usize {
    usize::from(v.get("field").is_some_and(|f| f == "applicationModelDigest")) + match v {
        Value::Array(a) => a.iter().map(count_fields).sum(),
        Value::Object(o) => o.values().map(count_fields).sum(),
        _ => 0,
    }
}
fn slots(ast: &Value) -> Result<[String; 2]> {
    require(count_fields(ast) == 2, "expected exactly two digest field visits")?;
    let declarations = ast["declarations"].as_array().ok_or("missing declarations")?;
    let mut paths = Vec::new();
    for (name, ty) in [("systemModel", "SystemModel"), ("systemManifest", "SystemManifest")] {
        let matches: Vec<_> = declarations.iter().enumerate().filter(|(_, d)| d["name"] == name).collect();
        require(matches.len() == 1, "missing or ambiguous binding declaration")?;
        let (i, d) = matches[0];
        require(d["kind"] == "definition" && d["parameters"] == json!([])
            && d["result"] == json!({"kind":"named", "member":{
                "module":"Production.BrowserSystem", "name":ty}, "arguments":[]}),
            "unexpected binding declaration type")?;
        let mut path = format!("/declarations/{i}/body");
        record_type(&d["body"], ty)?;
        if name == "systemModel" {
            let j = field(&d["body"], "applicationProfile")?;
            path.push_str(&format!("/fields/{j}/value"));
            record_type(ast.pointer(&path).ok_or("missing application profile")?, "ApplicationProfile")?;
        }
        let record = ast.pointer(&path).ok_or("missing record")?;
        let j = field(record, "applicationModelDigest")?;
        let literal = &record["fields"][j]["value"];
        require(literal["kind"] == "string" && literal["value"].is_string(), "nonliteral digest")?;
        let old = literal["value"].as_str().ok_or("nonliteral digest")?;
        require(old.strip_prefix("sha256:").is_some_and(|hex| hex.len() == 64
            && hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))),
            "digest must be sha256: followed by 64 lowercase hex digits")?;
        path.push_str(&format!("/fields/{j}/value/value"));
        paths.push(path);
    }
    let paths: [String; 2] = paths.try_into().map_err(|_| "binding arity")?;
    require(ast.pointer(&paths[0]) == ast.pointer(&paths[1]), "model/manifest digest disagreement")?;
    Ok(paths)
}
fn projection(config: &Utf8PathBuf) -> Result<(Vec<u8>, Value)> {
    let engine = Engine::load(config).map_err(|e| e.to_string())?;
    let full = engine.snapshot(LexCheckRequest { selection: Selection::Entrypoints }).map_err(|e| e.to_string())?;
    let mut roots = std::collections::BTreeSet::new();
    for module in full.modules() {
        if module.declarations().iter().any(|d| d.kind() == "definition"
            && d.linked_ir().pointer("/result/member/name").and_then(Value::as_str)
                .is_some_and(|n| n.ends_with("Application"))
            && d.linked_ir().pointer("/body/kind").and_then(Value::as_str) == Some("record")) {
            roots.insert(Utf8PathBuf::from(module.source().path()));
        }
    }
    require(roots == [Utf8PathBuf::from(APPLICATION)].into(), "wrong or multiple application roots")?;
    let selected = engine.snapshot(LexCheckRequest { selection: Selection::Files(roots) }).map_err(|e| e.to_string())?;
    let model = prismpm::holo::application::project_application_with_facets(&selected, &full)
        .map_err(|e| e.to_string())?.ok_or("no application projection")?;
    let bytes = encode_canonical(&model).map_err(|e| e.to_string())?;
    Ok((bytes, json!({"full_snapshot_sha256":content_id(&full.canonical_bytes()),
        "selected_snapshot_sha256":content_id(&selected.canonical_bytes()),
        "selection":APPLICATION, "facets":"all configured entrypoints"})))
}
fn prepare(root: &Path, output: &Path, before: BTreeMap<PathBuf, Vec<u8>>,
    custody: BTreeMap<PathBuf, Vec<u8>>, admission: Value) -> Result<()> {
    require(capture(root)? == before, "inputs changed after stack planning")?;
    let observe_custody = || -> Result<()> {
        for (path, bytes) in &custody { require(read(path)? == *bytes, "compiler custody input changed")?; }
        Ok(())
    };
    observe_custody()?;
    let source_path = root.join(SYSTEM);
    let lock_path = root.join("lexlean.lock");
    let source = text(before.get(&source_path).ok_or("System input missing")?)?;
    let config = Utf8PathBuf::from_path_buf(root.join("lexlean.toml")).map_err(|_| "non-UTF-8 path")?;
    let mut ast = lexlean::sdk::read_module(source).map_err(|e| e.to_string())?;
    let original_ast = ast.clone();
    let paths = slots(&ast)?;
    let (model, projection_before) = projection(&config)?;
    let digest = format!("sha256:{}", content_id(&model));
    let changed_fields = paths.iter().filter(|path| ast.pointer(path).and_then(Value::as_str)
        != Some(digest.as_str())).count();
    require(matches!(changed_fields, 0 | 2), "partial binding update is forbidden")?;
    for path in &paths { *ast.pointer_mut(path).ok_or("binding disappeared")? = json!(digest); }
    let mut restored_ast = ast.clone();
    for path in &paths { *restored_ast.pointer_mut(path).unwrap() = original_ast.pointer(path).unwrap().clone(); }
    require(restored_ast == original_ast, "non-target AST changed")?;
    let rewritten = lexlean::sdk::write_module(source, &ast).map_err(|e| e.to_string())?;
    require(changed_fields != 0 || rewritten.as_bytes() == source.as_bytes(),
        "zero changed fields must be a byte-identical no-op")?;
    require(lexlean::sdk::read_module(&rewritten).map_err(|e| e.to_string())? == ast, "SDK write changed AST")?;
    require(capture(root)? == before, "inputs changed during projection")?;
    observe_custody()?;
    // All mutations and verification are caught so both source and lock are restored on failure.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
        fs::write(&source_path, &rewritten).map_err(|e| e.to_string())?;
        Engine::load(&config).map_err(|e| e.to_string())?.lock(LockRequest {
            check_only: false, allow_network: false,
        }).map_err(|e| e.to_string())?;
        let (after_model, projection_after) = projection(&config)?;
        require(after_model == model, "binding changed true application model")?;
        let check = prismpm::Controller::load(root).map_err(|e| e.to_string())?
            .check(prismpm::controller::CheckRequest { config_path: None }).map_err(|e| e.to_string())?;
        observe_custody()?;
        let after = capture(root)?;
        let mut stable = after.clone();
        for path in [&source_path, &lock_path] { stable.insert(path.clone(), before[path].clone()); }
        require(stable == before, "non-target inputs changed")?;
        require(after[&source_path] == rewritten.as_bytes(), "source changed after relock")?;
        let parsed = lexlean::sdk::read_module(&rewritten).map_err(|e| e.to_string())?;
        require(slots(&parsed)? == paths && parsed == ast, "binding AST changed")?;
        require(lexlean::sdk::write_module(&rewritten, &parsed).map_err(|e| e.to_string())? == rewritten,
            "second structural write is not byte-identical")?;
        fs::write(output.join("application-model.canonical.json"), &model).map_err(|e| e.to_string())?;
        let receipt = json!({"scope":"construction-only source-authoring; not acceptance or proof",
            "release_base":BASE, "compiler_revision":COMPILER,
            "admission":admission, "custody_identities":custody.iter().map(|(p,b)|
                json!({"path":p,"sha256":content_id(b)})).collect::<Vec<_>>(),
            "compiler_custody":"owning M0 exact package admission and Cargo selection",
            "before":identities(&before,root), "after":identities(&after,root),
            "projection_before":projection_before, "projection_after":projection_after,
            "application_model_digest":digest, "field_visits":2, "changed_fields":changed_fields,
            "controller_check":check, "second_write_byte_identical":true});
        fs::write(output.join("receipt.json"), serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        Ok(())
    })).unwrap_or_else(|_| Err("preparation panicked; restoring original source and lock".into()));
    if let Err(original) = result {
        let mut failures = Vec::new();
        for path in [&source_path, &lock_path] {
            if let Err(e) = fs::write(path, &before[path]) { failures.push(format!("{}: {e}",path.display())); }
        }
        return Err(format!("{original}; rollback failures: {failures:?}"));
    }
    Ok(())
}
fn run() -> Result<()> {
    // Arguments: repo root, owning m0-reconcile.json, fresh receipt directory.
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    require(args.len() == 3, "expected REPO_ROOT M0_RECONCILE_RECEIPT NEW_RECEIPT_DIR")?;
    let repo = fs::canonicalize(&args[0]).map_err(|e| e.to_string())?;
    let root = repo.join(FIXTURE);
    require(fs::canonicalize(&root).map_err(|e| e.to_string())? == root, "aliased fixture root")?;
    let admission_path = PathBuf::from(&args[1]);
    let admission_bytes = read(&admission_path)?;
    let admission: Value = serde_json::from_slice(&admission_bytes).map_err(|e| e.to_string())?;
    require(admission["revision"] == COMPILER && admission["version"] == "0.4.0"
        && admission["source_tree"] == "339f2df51b278e6dd740fdde67b3401d3be356c8", "wrong compiler admission")?;
    let mut custody = BTreeMap::from([(admission_path, admission_bytes)]);
    for name in ["Cargo.lock", "Cargo.toml", "vendor/lexlean/.cargo_vcs_info.json",
        "vendor/lexlean/MANIFEST.sha256", "vendor/lexlean/lexlean-0.4.0.crate"] {
        let path = repo.join(name);
        regular(&repo, &path)?;
        custody.insert(path.clone(), read(&path)?);
    }
    require(admission["crate_sha256"] == content_id(&custody[&repo.join("vendor/lexlean/lexlean-0.4.0.crate")])
        && admission["manifest_sha256"] == content_id(&custody[&repo.join("vendor/lexlean/MANIFEST.sha256")]),
        "compiler package differs from owning admission")?;
    let compiler_vcs = &custody[&repo.join("vendor/lexlean/.cargo_vcs_info.json")];
    let vcs: Value = serde_json::from_slice(compiler_vcs).map_err(|e| e.to_string())?;
    require(vcs["git"]["sha1"] == COMPILER && vcs["git"].get("dirty").is_none_or(|v| v == false), "compiler provenance mismatch")?;
    let before = capture(&root)?;
    let config_path = root.join("lexlean.toml");
    let config: toml::Value = toml::from_str(text(&before[&config_path])?).map_err(|e| e.to_string())?;
    let expected = ["Architecture", "Design", "Quality", "Security", "System", "Workbench"]
        .map(|n| toml::Value::String(format!("src/{n}.lex.tex"))).to_vec();
    require(config.get("name").and_then(toml::Value::as_str) == Some("prism-code")
        && config.get("entrypoints").and_then(toml::Value::as_array) == Some(&expected)
        && config.get("source_roots").and_then(toml::Value::as_array) == Some(&vec![toml::Value::String("src".into())])
        && config.get("lockfile").and_then(toml::Value::as_str) == Some("lexlean.lock"), "wrong CT02 project configuration")?;
    let config_utf8 = Utf8PathBuf::from_path_buf(config_path).map_err(|_| "non-UTF-8 config")?;
    let project = lexlean::project::Project::load(&config_utf8).map_err(|e| e.to_string())?;
    let stack = project.config.limits.compile_stack_plan().stack_bytes;
    require(project.config_bytes == before[config_utf8.as_std_path()], "configuration changed during planning")?;
    let output = PathBuf::from(&args[2]);
    let output_parent = fs::canonicalize(output.parent().ok_or("receipt directory needs a parent")?)
        .map_err(|e| e.to_string())?;
    require(!output_parent.starts_with(&repo), "receipt directory must be outside repository inputs")?;
    fs::create_dir(&output).map_err(|e| e.to_string())?;
    std::thread::Builder::new().name("ct02-source-preparation".into()).stack_size(stack)
        .spawn(move || prepare(&root, &output, before, custody, admission)).map_err(|e| e.to_string())?
        .join().map_err(|_| "preparation worker terminated unexpectedly".to_string())?
}
fn main() {
    if let Err(error) = run() { eprintln!("CT02 source preparation: {error}"); std::process::exit(1); }
}
