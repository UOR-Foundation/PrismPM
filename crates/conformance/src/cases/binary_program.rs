//! Real source-to-native/Core-Wasm/raw-CLI package acceptance and counterexamples.
use prismpm::controller::{BuildRequest, CheckRequest, ProductBuildRequest, VerifyRequest};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

fn fixture(root:&Path)->tempfile::TempDir {
    let work=tempfile::tempdir().unwrap();
    crate::fixtures::copy_dir_recursive(&root.join("tests/fixtures/binary/binary-program/project"),work.path()).unwrap();
    work
}
fn tree(root:&Path)->BTreeMap<String,Vec<u8>> {
    walkdir::WalkDir::new(root).into_iter().map(Result::unwrap).filter(|entry|entry.file_type().is_file())
        .map(|entry|(entry.path().strip_prefix(root).unwrap().to_string_lossy().into_owned(),std::fs::read(entry.path()).unwrap())).collect()
}
fn mutate(project:&Path, change:impl FnOnce(&mut Value)) {
    let path=project.join("src/Probe.lex.tex"); let source=std::fs::read_to_string(&path).unwrap();
    let line=source.lines().find(|line|line.starts_with("\\semanticdata{")).unwrap();
    let mut module:Value=serde_json::from_str(line.strip_prefix("\\semanticdata{").unwrap().strip_suffix('}').unwrap()).unwrap();
    change(&mut module);
    std::fs::write(path,source.replace(line,&format!("\\semanticdata{{{}}}",serde_json::to_string(&module).unwrap()))).unwrap();
}
fn field<'a>(module:&'a mut Value,name:&str)->&'a mut Value {
    &mut module["declarations"].as_array_mut().unwrap().iter_mut().find(|d|d["name"]=="probeProgram").unwrap()["body"]["fields"]
        .as_array_mut().unwrap().iter_mut().find(|f|f["field"]==name).unwrap()["value"]
}
pub(super) fn verify(root:&Path) {
    let work=fixture(root);let controller=prismpm::Controller::load(work.path()).unwrap();
    let before=tree(work.path());let checked=controller.check(CheckRequest{config_path:None}).unwrap();
    assert_eq!(tree(work.path()),before,"binary check stays read-only");assert_eq!(checked.entity_count,1);
    let verified=controller.verify(VerifyRequest{config_path:None}).unwrap();
    let build=work.path().join(".prism/build").join(&verified.build_id);
    let model=super::json(&build.join("model.prism.json"));
    assert_eq!(model["schema"],"prismpm/model-document/5");assert!(model.get("application").is_none()&&model.get("library").is_none());
    let evidence=super::json(&work.path().join(&verified.verified_root).join("binary-acceptance.json"));
    assert_eq!(evidence["scope"],"binary-package-only");assert_eq!(evidence["program"],model["program"]);
    assert_eq!(evidence["regeneration"],"byte-identical");
    assert_eq!(evidence["io_coverage"],json!({"platform":"linux","output_write":"passed"}));
    let vectors=&model["program"]["acceptance_vectors"];
    assert_eq!(vectors.as_array().unwrap().len(),4);
    assert_eq!(vectors[0],json!({"request":[],"response":[]}));
    assert_eq!(vectors[2]["request"],json!((0..=255).collect::<Vec<u16>>()));assert_eq!(vectors[2]["request"],vectors[2]["response"]);
    let malformed = vectors[3]["request"].as_array().unwrap().iter()
        .map(|byte| u8::try_from(byte.as_u64().unwrap()).unwrap()).collect::<Vec<_>>();
    assert!(std::str::from_utf8(&malformed).is_err());
    for (row,mode) in evidence["executions"].as_array().unwrap().iter().zip(["std","no_std","core-wasm","cli-stdio","cli-file","cli-mixed"]) {
        assert_eq!(row,&json!({"mode":mode,"status":"passed","vector_count":4}));
    }
    assert_eq!(evidence["executions"].as_array().unwrap().len(),6);
    let files=tree(&build);assert!(files.keys().all(|path|!path.ends_with(".holo")&&!path.starts_with("application/")));
    assert!(files.contains_key("binary/core.wasm"));assert!(files.contains_key("binary/cli/src/main.rs"));
    let second=fixture(root);let rebuilt=prismpm::Controller::load(second.path()).unwrap().build(BuildRequest{config_path:None}).unwrap();
    assert_eq!(rebuilt.build_id,verified.build_id);assert_eq!(files,tree(&second.path().join(".prism/build").join(&rebuilt.build_id)));
    let error=controller.product_build(ProductBuildRequest{config_path:None,locked:true,release:None,reference:"ghcr.io/uor-foundation/prismpm-binary-probe:0.1.0".into()}).unwrap_err();
    assert_eq!(error.code,"PP6101");assert_eq!(error.message,"binary-package acceptance is not product-release or deployment acceptance");
    let retained=tempfile::tempdir().unwrap();
    crate::fixtures::copy_dir_recursive(&build,&retained.path().join(".prism/build").join(&verified.build_id)).unwrap();
    crate::fixtures::copy_dir_recursive(&work.path().join(&verified.verified_root),&retained.path().join(&verified.verified_root)).unwrap();
    let error=prismpm::oci::assemble(retained.path(),&rebuilt,&verified,"ghcr.io/uor-foundation/prismpm-binary-probe:0.1.0",&[]).unwrap_err();
    assert_eq!(error.message,"binary-package evidence cannot authorize a product release");
    for mutation in 0..4 {
        let invalid=fixture(root);mutate(invalid.path(),|module|match mutation {
            0=>field(module,"entryRoot")["value"]=json!("LibraryProbe.Probe.absent"),
            1=>field(module,"requestMaximum")["value"]=json!("255"),
            2=>field(module,"cli")["fields"][0]["value"]["value"]=json!("optional"),
            _=>module["declarations"][0]["parameters"]=json!([]),
        });
        assert!(prismpm::Controller::load(invalid.path()).unwrap().check(CheckRequest{config_path:None}).is_err());
        assert!(!invalid.path().join(".prism/verified").exists());
    }
    // Real generated changed-byte, truncation and newline cores must fail exact vectors.
    for body in [
        json!({"kind":"bytes","hex":"ff"}),
        json!({"kind":"bytes","hex":""}),
        json!({"kind":"primitive","operation":"append","result":{"kind":"bytes"},"arguments":[{"kind":"var","name":"value"},{"kind":"bytes","hex":"0a"}]}),
    ] {
        let invalid=fixture(root);mutate(invalid.path(),|module|module["declarations"][0]["body"]=body);
        let error=prismpm::Controller::load(invalid.path()).unwrap().verify(VerifyRequest{config_path:None}).unwrap_err();
        assert_eq!(error.code,"PP5006");assert!(!invalid.path().join(".prism/verified").exists());
    }
    // A previously accepted artifact cannot be substituted under an unchanged build identity.
    let wasm_path=build.join("binary/core.wasm");let wasm=std::fs::read(&wasm_path).unwrap();
    let mut changed=wasm.clone();
    let position=changed.windows(b"binary_run".len()).position(|window|window==b"binary_run").expect("generated binary ABI name");
    changed[position]=b'x';std::fs::write(&wasm_path,changed).unwrap();
    assert!(controller.verify(VerifyRequest{config_path:None}).is_err(),"changed real Wasm ABI cannot retain acceptance");
    std::fs::write(&wasm_path,&wasm).unwrap();
    let omitted=build.join("binary/cli/src/main.rs");let cli=std::fs::read(&omitted).unwrap();std::fs::remove_file(&omitted).unwrap();
    assert!(controller.verify(VerifyRequest{config_path:None}).is_err(),"omitted generated adapter cannot retain acceptance");
    std::fs::write(omitted,cli).unwrap();
    let restored=controller.verify(VerifyRequest{config_path:None}).unwrap();assert_eq!(restored.build_id,verified.build_id);
}
