//! Closed Text/4 model, action and nested-report validation.
use prismpm::holo::model_document::{Application, ModelDocument};
use prismpm::holo::research_workbench::{self, ComparisonStatus};
fn fixture() -> ModelDocument {
    serde_json::from_slice(include_bytes!(
        "../../../tests/data/research-workbench-model-document.json"
    ))
    .unwrap()
}
#[test]
fn research_workbench_closed_profile_and_action_authorities() {
    let doc = fixture();
    prismpm::holo::validate::validate(&doc).unwrap();
    let Some(Application::Research(app)) = &doc.application else {
        panic!("Text/4 selected");
    };
    let value = serde_json::to_value(&doc).unwrap();
    prismpm::contracts::CanonicalDocument::from_value("prismpm/model-document/7", value.clone())
        .unwrap();
    for path in ["research", "case_exchange", "runtime_binding"] {
        let mut v = value.clone();
        v["application"]["view"]
            .as_object_mut()
            .unwrap()
            .remove(path);
        assert!(serde_json::from_value::<ModelDocument>(v).is_err());
    }
    let mut bad = app.clone();
    bad.view.case_exchange.file_name = "../case.json".to_owned();
    assert!(research_workbench::validate(&bad).is_err());
    let mut bad = app.clone();
    bad.view.operations[0].trace.timeout_milliseconds = 0;
    assert!(research_workbench::validate(&bad).is_err());
    let mut bad = app.clone();
    bad.view.operations[0].trace.request_prefix = bad.view.operations[0].request_prefix.clone();
    assert!(research_workbench::validate(&bad).is_err());
    let mut bad = app.clone();
    bad.view.case_exchange.file_read_error_code = "missing".to_owned();
    assert!(research_workbench::validate(&bad).is_err());
    for field in ["request", "response", "deadline"] {
        let mut bad = app.clone();
        bad.core_memory_pages = 256;
        if field == "request" {
            bad.guest_allocation_maximum = 2_097_153;
            bad.acceptance_vectors.last_mut().unwrap().request = vec![120; 2_097_153];
        }
        assert!(research_workbench::validate(&bad).is_ok());
        match field {
            "request" => bad.request_maximum = 2_097_153,
            "response" => bad.response_maximum = 4_194_305,
            _ => bad.view.operations[0].trace.timeout_milliseconds = 180_001,
        };
        assert!(
            research_workbench::validate(&bad).is_err(),
            "Text/4 must reject first-over-{field} authority"
        );
    }
}
#[test]
fn research_workbench_strict_nested_report_and_exact_example_context() {
    let doc = fixture();
    let Some(Application::Research(app)) = &doc.application else {
        panic!();
    };
    let example = &app.view.operations[0].examples[0];
    let answer =
        research_workbench::result(app, "echo", "compute", example.expected_response.as_bytes())
            .unwrap();
    assert_eq!(answer.comparison.status, ComparisonStatus::None);
    let frame = research_workbench::example_request(&app.view.operations[0], example).unwrap();
    assert!(frame.contains("\"model_id\":\"\""));
    assert_eq!(app.acceptance_vectors[0].request, frame.as_bytes());
    let duplicate = example.expected_response.replace(
        "\"selection\":{\"row\":0",
        "\"selection\":{\"row\":0,\"row\":1",
    );
    assert!(research_workbench::result(app, "echo", "compute", duplicate.as_bytes()).is_err());
    let mut value: serde_json::Value = serde_json::from_str(&example.expected_response).unwrap();
    value["panels"] = serde_json::json!([{"id":"route","title":"Route","rows":2,"columns":2,"cells":["one"],"request":"real command","response":"one"}]);
    assert!(research_workbench::result(
        app,
        "echo",
        "compute",
        &serde_json::to_vec(&value).unwrap()
    )
    .is_err());
    assert!(
        research_workbench::result(app, "echo", "trace", example.expected_response.as_bytes())
            .is_err()
    );
}

#[test]
fn research_workbench_source_projection_uses_real_lexlean_snapshot() {
    use lexlean::{CheckRequest, Engine, LockRequest, Selection};
    let r = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_owned();
    let work = tempfile::tempdir().unwrap();
    let source = r.join("tests/fixtures/holo/ho-18-research-workbench/project");
    for entry in walkdir::WalkDir::new(&source)
        .into_iter()
        .filter_entry(|entry| {
            !matches!(
                entry.file_name().to_str(),
                Some(".prism" | ".lexlean" | ".lake")
            )
        })
    {
        let entry = entry.unwrap();
        let target = work
            .path()
            .join(entry.path().strip_prefix(&source).unwrap());
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(target).unwrap();
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
    for name in [
        "Foundation/Bytes.lex.tex",
        "Foundation/Utf8.lex.tex",
        "Foundation/View/Text/V1/Model.lex.tex",
        "Foundation/View/Text/V3/Model.lex.tex",
        "Foundation/View/Text/V4/Model.lex.tex",
    ] {
        let target = work.path().join("src").join(name);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(r.join("stdlib/src").join(name), target).unwrap();
    }
    let path = camino::Utf8PathBuf::from_path_buf(work.path().join("lexlean.toml")).unwrap();
    let engine = Engine::load(&path).unwrap();
    engine
        .lock(LockRequest {
            check_only: false,
            allow_network: false,
        })
        .unwrap();
    let snapshot = Engine::load(&path)
        .unwrap()
        .snapshot(CheckRequest {
            selection: Selection::Entrypoints,
        })
        .unwrap();
    let projected = prismpm::holo::application::project_application(&snapshot)
        .unwrap()
        .unwrap();
    prismpm::holo::validate::validate(&projected).unwrap();
    assert_eq!(projected.schema, "prismpm/model-document/7");
    assert_eq!(
        serde_json::to_value(projected.application.unwrap()).unwrap(),
        serde_json::to_value(fixture().application.unwrap()).unwrap()
    );
}
