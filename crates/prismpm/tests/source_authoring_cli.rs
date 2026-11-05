//! Public command through the actual built CLI, without a project,
//! subprocess parser, typed module normalization or compiler execution.
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

const REQUEST: &str = "prismpm/source-authoring-request/1";
const RESULT: &str = "prismpm/source-authoring-result/1";
fn call(cwd: &Path, arguments: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_prismpm"))
        .args(arguments)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
fn request(source: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"schema":REQUEST,"source":source})).unwrap()
}
fn data(value: &Value) -> String {
    format!("prefix é🙂\r\n\\begin{{semanticmodule}}\n\\semanticdata{{{}}}\n\\end{{semanticmodule}}\r\nsuffix\n",serde_json::to_string(value).unwrap())
}
fn module() -> Value {
    json!({"spec":"lexlean/semantic-module/2","declarations":[{"kind":"definition","name":"truth","parameters":[],"result":{"kind":"bool"},"body":{"kind":"bool","value":true}}]})
}
fn success(output: Output, operation: &str) -> (Value, Vec<u8>) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.iter().filter(|b| **b == b'\n').count(), 1);
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], RESULT);
    assert_eq!(value["operation"], operation);
    (value, output.stdout)
}
fn refusal(output: Output, code: &str, exit: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(exit),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "prismpm/error-result/1");
    assert_eq!(value["diagnostic"]["code"], code);
    assert!(!value.as_object().unwrap().contains_key("value"));
    assert!(!value.as_object().unwrap().contains_key("source"));
    value
}
#[test]
fn full_json_field_presence_and_float_survive_without_project_or_lock_loading() {
    let cwd = tempfile::tempdir().unwrap();
    std::fs::write(cwd.path().join("prismpm.toml"), "this is invalid").unwrap();
    std::fs::write(cwd.path().join("prismpm.lock"), "not a lock").unwrap();
    let value = json!({"absent_container":{},"empty":[],"null":null,"parameters":null,"type_parameters":[],"false":false,"zero":0,"float":1.25,"integer_edge":u64::MAX,"extra":{"unicode":"é🙂"}});
    let source = data(&value);
    let before = std::fs::read_dir(cwd.path()).unwrap().count();
    let (read, bytes) = success(
        call(cwd.path(), &["--json", "source", "read"], &request(&source)),
        "read",
    );
    assert_eq!(read["value"], value);
    assert!(!read["value"]["absent_container"]
        .as_object()
        .unwrap()
        .contains_key("parameters"));
    let (_, again) = success(
        call(cwd.path(), &["--json", "source", "read"], &request(&source)),
        "read",
    );
    assert_eq!(bytes, again);
    assert_eq!(std::fs::read_dir(cwd.path()).unwrap().count(), before);
    assert_eq!(
        std::fs::read_to_string(cwd.path().join("prismpm.lock")).unwrap(),
        "not a lock"
    );
}
#[test]
fn data_and_notation_roundtrip_through_the_actual_pinned_sdk() {
    let cwd = tempfile::tempdir().unwrap();
    // Exact pinned authoring fixtures: semantic SDK sample, tooling surface
    // module() and core surface sample(). These are authoring values, not
    // claims that the external authority rows or kernel terms were verified.
    let tooling = json!({"authority":{"id":"A","lean_githash":"g","lean_version":"4.32.1"},"declarations":[{"constructors":[{"fields":[],"name":"false"}],"kind":"type","name":"Bool","parameters":[]},{"kind":"call","name":"f"}],"imports":[],"spec":"lexlean/tooling-module/1"});
    let core: lexlean::ir::core::CoreModule = serde_json::from_value(json!({
        "spec":"lexlean/core-module/2","imports":["Init"],"proof_nodes":[3],
        "nodes":[{"k":"s","l":{"a":{"a":"u","k":"p"},"k":"s"}},{"k":"c","n":"Nat","u":[]},{"f":1,"k":"a","x":1},{"i":0,"k":"b"},{"b":"e","k":"l","n":"n","t":1,"v":3},{"b":"e","k":"p","n":"","t":2,"v":2}],
        "declarations":[{"class":false,"generated":false,"kind":"definition","levels":["u"],"name":"T","policy":{"kind":"none"},"reducibility":{"height":0,"kind":"regular"},"transparency":"semireducible","type":5,"value":4}]
    })).unwrap();
    for (environment, control, value, kind) in [
        (
            "semanticmodule",
            "semanticdata",
            module(),
            lexlean::sdk::ModuleKind::Semantic,
        ),
        (
            "toolingmodule",
            "toolingdata",
            tooling,
            lexlean::sdk::ModuleKind::Tooling,
        ),
        (
            "coremodule",
            "coredata",
            serde_json::to_value(core).unwrap(),
            lexlean::sdk::ModuleKind::Core,
        ),
    ] {
        let source=format!("prefix é🙂\r\n\\begin{{{environment}}}\n\\{control}{{{}}}\n\\end{{{environment}}}\r\nsuffix\n",serde_json::to_string(&value).unwrap());
        let (original_read, _) = success(
            call(cwd.path(), &["--json", "source", "read"], &request(&source)),
            "read",
        );
        assert_eq!(original_read["value"], value);
        assert_eq!(lexlean::sdk::module_kind(&source), Some(kind));
        let (converted, _) = success(
            call(
                cwd.path(),
                &["source", "to-notation", "--json"],
                &request(&source),
            ),
            "to-notation",
        );
        let notation = converted["source"].as_str().unwrap();
        assert_eq!(notation, lexlean::sdk::to_notation(&source).unwrap());
        assert_eq!(lexlean::sdk::read_module(notation).unwrap(), value);
        assert_eq!(lexlean::sdk::module_kind(notation), Some(kind));
        assert!(notation.starts_with("prefix é🙂\r\n"));
        assert!(notation.ends_with("\r\nsuffix\n"));
        let (notation_read, _) = success(
            call(
                cwd.path(),
                &["--json", "source", "read"],
                &request(notation),
            ),
            "read",
        );
        assert_eq!(notation_read["value"], value);
        for original in [source.as_str(), notation] {
            let mut changed = value.clone();
            changed["declarations"][0]["name"] = json!("renamed");
            let input =
                serde_json::to_vec(&json!({"schema":REQUEST,"source":original,"value":changed}))
                    .unwrap();
            let (rewritten, _) = success(
                call(cwd.path(), &["--json", "source", "write"], &input),
                "write",
            );
            let rewritten = rewritten["source"].as_str().unwrap();
            assert_eq!(
                rewritten,
                lexlean::sdk::write_module(original, &changed).unwrap()
            );
            let (read, _) = success(
                call(
                    cwd.path(),
                    &["--json", "source", "read"],
                    &request(rewritten),
                ),
                "read",
            );
            assert_eq!(read["value"], changed);
            assert_eq!(
                lexlean::sdk::is_notation(original),
                lexlean::sdk::is_notation(rewritten)
            );
        }
    }
    let value = module();
    let mut legacy = value;
    legacy["spec"] = json!("lexlean/semantic-module/1");
    refusal(
        call(
            cwd.path(),
            &["--json", "source", "to-notation"],
            &request(&data(&legacy)),
        ),
        "PP5802",
        1,
    );
}
#[test]
fn required_value_distinguishes_sdk_refusal_from_omission_and_read_preserves_null() {
    let cwd = tempfile::tempdir().unwrap();
    let source = data(&module());
    // Null is present in the public request. The pinned writer's restricted
    // canonical JSON refuses it; the protocol must not call it a missing field.
    refusal(
        call(
            cwd.path(),
            &["--json", "source", "write"],
            &serde_json::to_vec(&json!({"schema":REQUEST,"source":source,"value":null})).unwrap(),
        ),
        "PP5802",
        1,
    );
    refusal(
        call(
            cwd.path(),
            &["--json", "source", "write"],
            &request(&source),
        ),
        "PP5801",
        2,
    );
    // The official reader accepts raw data-form null even though the writer
    // cannot emit it. Preserve that complete parsed Value in the result.
    let (read, _) = success(
        call(
            cwd.path(),
            &["--json", "source", "read"],
            &request(&data(&Value::Null)),
        ),
        "read",
    );
    assert!(read.as_object().unwrap().contains_key("value"));
    assert!(read["value"].is_null());
}

#[test]
fn closed_protocol_rejects_bad_utf8_trailing_unknown_duplicate_and_wrong_shape() {
    let cwd = tempfile::tempdir().unwrap();
    let source = data(&module());
    for bad in [
        json!({"schema":REQUEST,"source":source,"unknown":true}),
        json!({"schema":"wrong","source":source}),
        json!({"schema":REQUEST,"source":null}),
        json!({"schema":REQUEST}),
        json!({"schema":REQUEST,"source":source,"value":null}),
    ] {
        refusal(
            call(
                cwd.path(),
                &["--json", "source", "read"],
                &serde_json::to_vec(&bad).unwrap(),
            ),
            "PP5801",
            2,
        );
    }
    for bytes in [
        b"\xff".as_slice(),
        b"{} {}",
        br#"{"schema":"prismpm/source-authoring-request/1","source":"","source":""}"#,
    ] {
        refusal(
            call(cwd.path(), &["--json", "source", "read"], bytes),
            "PP5801",
            2,
        );
    }
    let no_json = call(cwd.path(), &["source", "read"], &request(&source));
    assert_eq!(no_json.status.code(), Some(2));
    assert!(no_json.stdout.is_empty());
    assert!(String::from_utf8_lossy(&no_json.stderr).contains("PP5801"));
}
#[test]
fn byte_budgets_include_the_envelope_and_lf_and_do_not_clip_success() {
    let cwd = tempfile::tempdir().unwrap();
    let input = request(&data(&module()));
    let (_, bytes) = success(
        call(cwd.path(), &["--json", "source", "read"], &input),
        "read",
    );
    let exact = bytes.len().to_string();
    let short = (bytes.len() - 1).to_string();
    let input_size = input.len().to_string();
    let input_short = (input.len() - 1).to_string();
    let (_, same) = success(
        call(
            cwd.path(),
            &[
                "--json",
                "source",
                "--max-input-bytes",
                &input_size,
                "--max-output-bytes",
                &exact,
                "read",
            ],
            &input,
        ),
        "read",
    );
    assert_eq!(same, bytes);
    refusal(
        call(
            cwd.path(),
            &["--json", "source", "read", "--max-output-bytes", &short],
            &input,
        ),
        "PP5803",
        14,
    );
    refusal(
        call(
            cwd.path(),
            &[
                "--json",
                "source",
                "read",
                "--max-input-bytes",
                &input_short,
            ],
            &input,
        ),
        "PP5803",
        14,
    );
    for flag in ["--max-input-bytes", "--max-output-bytes"] {
        for bound in ["0", "18446744073709551615"] {
            refusal(
                call(
                    cwd.path(),
                    &["--json", "source", "read", flag, bound],
                    &input,
                ),
                "PP5801",
                2,
            );
        }
    }
}
#[test]
fn sdk_offsets_and_bounded_errors_are_preserved_without_semantic_success() {
    let cwd = tempfile::tempdir().unwrap();
    let source="é🙂\n\\begin{semanticmodule}\ndefinition{name: , parameters: [], result: Bool, body: true}\n\\end{semanticmodule}\n";
    let upstream = lexlean::sdk::read_module(source).unwrap_err();
    let offset = upstream.offset.unwrap();
    let error = refusal(
        call(cwd.path(), &["--json", "source", "read"], &request(source)),
        "PP5802",
        1,
    );
    assert_eq!(error["diagnostic"]["primary"]["byte_start"], offset);
    assert_eq!(error["diagnostic"]["primary"]["path"], "stdin-source");
    let bad = json!({"schema":REQUEST,"source":"no module",("x".repeat(5000)):true});
    let error = refusal(
        call(
            cwd.path(),
            &["--json", "source", "read"],
            &serde_json::to_vec(&bad).unwrap(),
        ),
        "PP5801",
        2,
    );
    assert!(error["diagnostic"]["message"].as_str().unwrap().len() < 2100);
}
