//! Closed server application contract: schema, validation, source projection
//! and the registered failure vocabulary shared by Lean, the registry and the
//! generated host adapter. Nothing here executes an application.

use lexlean::{CheckRequest, Engine, LockRequest, Selection};
use prismpm::holo::canonical::{decode_canonical, encode_canonical};
use prismpm::holo::model_document::{Application, ModelDocument};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const MODEL: &str = "Foundation.Server.V1.Model";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn field(name: &str, kind: &str) -> Value {
    json!({"name": name, "kind": kind})
}

fn declaration() -> Value {
    json!({
        "profile": "prismpm/server-application/1",
        "name": "Server Probe",
        "cargo_name": "server-probe",
        "cargo_version": "0.1.0",
        "cargo_description": "A modeled host effect conformance fixture.",
        "cargo_repository": "https://example.invalid/probe",
        "cargo_homepage": "https://example.invalid/probe",
        "acceptance_vectors": [
            {"request": [97], "response": [98]},
            {"request": [99], "response": [100]}
        ],
        "core": {"root": "ServerProbe.ServerProbe.dispatch", "package": "server_probe_core", "version": "0.1.0"},
        "bindings": [{"path": "bindings/server-probe-binding", "crate_name": "server-probe-binding", "host": "server_probe_binding::Native"}],
        "request_maximum": 8,
        "response_maximum": 8,
        "host": {
            "protocol": "prismpm/host-effects/1",
            "description": "Probe.\n",
            "usage_code": 64,
            "io_code": 74,
            "fault_code": 70,
            "effect_budget": 2,
            "effects": [
                {
                    "name": "store.get",
                    "request": [field("key", "bytes"), field("limit", "u32")],
                    "result": [field("found", "flag"), field("note", "text")],
                    "failures": ["denied", "unavailable"],
                    "request_maximum": 64,
                    "result_maximum": 64
                },
                {
                    "name": "clock.now",
                    "request": [],
                    "result": [field("millis", "u64")],
                    "failures": [],
                    "request_maximum": 16,
                    "result_maximum": 16
                }
            ],
            "transcripts": [
                {"vector": 0, "steps": [
                    {
                        "effect": "store.get",
                        "request": [0, 0, 0, 1, 107, 0, 0, 0, 7],
                        "outcome": [0, 0, 0, 0, 1, 0, 0, 0, 0]
                    },
                    {
                        "effect": "clock.now",
                        "request": [],
                        "outcome": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 42]
                    }
                ]},
                {"vector": 1, "steps": [{
                    "effect": "store.get",
                    "request": [0, 0, 0, 0, 0, 0, 0, 1],
                    "outcome": [0, 0, 0, 1, 0, 0, 0, 1]
                }]}
            ]
        }
    })
}

fn fixture() -> Value {
    let mut value: Value = serde_json::from_slice(
        &std::fs::read(root().join("tests/data/text-model-document.json")).unwrap(),
    )
    .unwrap();
    value["schema"] = json!("prismpm/model-document/7");
    value["application"] = declaration();
    value
}

fn typed(value: Value) -> ModelDocument {
    serde_json::from_value(value).unwrap()
}

fn schema() -> jsonschema::Validator {
    let schema: Value =
        serde_json::from_slice(include_bytes!("../schemas/model-document-v7.schema.json")).unwrap();
    jsonschema::validator_for(&schema).unwrap()
}

fn rejected(value: Value) -> bool {
    match serde_json::from_value::<ModelDocument>(value) {
        Ok(doc) => prismpm::holo::validate::validate(&doc).is_err(),
        Err(_) => true,
    }
}

#[test]
fn server_contract_roundtrips_and_requires_the_closed_declaration() {
    let value = fixture();
    assert!(schema().is_valid(&value));
    let doc = typed(value.clone());
    prismpm::holo::validate::validate(&doc).unwrap();
    let bytes = encode_canonical(&doc).unwrap();
    assert_eq!(decode_canonical(&bytes).unwrap(), doc);
    assert_eq!(serde_json::to_value(&doc).unwrap(), value);
    let Some(Application::Server(server)) = &doc.application else {
        panic!("the server profile projects a server application");
    };
    assert_eq!(server.host.effects.len(), 2);
    assert_eq!(server.host.transcripts.len(), 2);
    prismpm::contracts::CanonicalDocument::from_value("prismpm/model-document/7", value.clone())
        .unwrap();

    let mut missing = value.clone();
    missing["application"]
        .as_object_mut()
        .unwrap()
        .remove("host");
    assert!(!schema().is_valid(&missing));
    assert!(
        prismpm::contracts::CanonicalDocument::from_value("prismpm/model-document/7", missing)
            .is_err()
    );
    for pointer in [
        "/application/extra",
        "/application/core/extra",
        "/application/bindings/0/extra",
        // The version of a workspace crate is its own manifest's, not the declaration's.
        "/application/bindings/0/version",
        "/application/host/extra",
        "/application/host/effects/0/extra",
        "/application/host/effects/0/request/0/extra",
        "/application/host/transcripts/0/extra",
        "/application/host/transcripts/0/steps/0/extra",
    ] {
        let mut unknown = value.clone();
        let (parent, name) = pointer.rsplit_once('/').unwrap();
        unknown.pointer_mut(parent).unwrap()[name] = json!(1);
        assert!(!schema().is_valid(&unknown), "{pointer}");
        assert!(
            prismpm::contracts::CanonicalDocument::from_value(
                "prismpm/model-document/7",
                unknown
            )
            .is_err(),
            "{pointer}"
        );
    }
    let mut kind = value;
    kind["application"]["host"]["effects"][0]["request"][0]["kind"] = json!("u128");
    assert!(!schema().is_valid(&kind));
}

#[test]
fn server_contract_rejects_every_invalid_declaration() {
    let many_effects = (0..65)
        .map(|index| {
            json!({
                "name": format!("effect.n{index}"),
                "request": [],
                "result": [],
                "failures": [],
                "request_maximum": 4,
                "result_maximum": 4
            })
        })
        .collect::<Vec<_>>();
    let mutations: Vec<(&str, Value)> = vec![
        ("/application/profile", json!("prismpm/server-application/2")),
        ("/application/profile", json!("prismpm/text-application/3")),
        ("/application/host/protocol", json!("prismpm/host-effects/2")),
        ("/application/request_maximum", json!(0)),
        ("/application/request_maximum", json!(64 * 1024 * 1024 + 1)),
        ("/application/response_maximum", json!(0)),
        ("/application/response_maximum", json!(64 * 1024 * 1024 + 1)),
        ("/application/cargo_name", json!("Not A Crate")),
        ("/application/cargo_version", json!("1")),
        ("/application/core/root", json!("dispatch")),
        ("/application/core/package", json!("Not A Crate")),
        ("/application/core/package", json!("server_probe_host")),
        ("/application/core/package", json!("std")),
        ("/application/core/package", json!("fn")),
        ("/application/core/version", json!("1")),
        ("/application/core/version", json!("01.0.0")),
        ("/application/bindings", json!([])),
        (
            "/application/bindings",
            json!([
                {"path": "a", "crate_name": "a-binding", "host": "a_binding::Native"},
                {"path": "b", "crate_name": "b-binding", "host": "b_binding::Native"}
            ]),
        ),
        ("/application/bindings/0/crate_name", json!("Binding")),
        ("/application/bindings/0/crate_name", json!("server-probe-host")),
        ("/application/bindings/0/crate_name", json!("server_probe_core")),
        ("/application/bindings/0/crate_name", json!("server-probe-host-api")),
        ("/application/bindings/0/path", json!("")),
        ("/application/bindings/0/path", json!("/abs/path")),
        ("/application/bindings/0/path", json!("../outside")),
        ("/application/bindings/0/path", json!("bindings/../../outside")),
        ("/application/bindings/0/path", json!("bindings/./b")),
        ("/application/bindings/0/path", json!("bindings//b")),
        ("/application/bindings/0/path", json!("bindings/b/")),
        ("/application/bindings/0/path", json!("bindings\\b")),
        ("/application/bindings/0/path", json!(".hidden/b")),
        ("/application/bindings/0/path", json!("target/debug/b")),
        ("/application/bindings/0/path", json!("server-probe-host")),
        ("/application/bindings/0/path", json!("server-probe-host-api/binding")),
        ("/application/bindings/0/path", json!("a/b/c/d/e/f/g/h/i")),
        ("/application/bindings/0/host", json!("other_binding::Native")),
        ("/application/bindings/0/host", json!("Native")),
        ("/application/bindings/0/host", json!("binding::fn")),
        ("/application/bindings/0/host", json!("binding::")),
        ("/application/bindings/0/host", json!("binding::Native; evil")),
        ("/application/host/description", json!("  \n")),
        ("/application/host/description", json!("bell\u{7}")),
        ("/application/host/usage_code", json!(74)),
        ("/application/host/usage_code", json!(0)),
        ("/application/host/usage_code", json!(126)),
        ("/application/host/io_code", json!(70)),
        ("/application/host/fault_code", json!(64)),
        ("/application/host/fault_code", json!(0)),
        ("/application/host/effect_budget", json!(0)),
        ("/application/host/effect_budget", json!(257)),
        ("/application/host/effects", json!(many_effects)),
        ("/application/host/effects/0/name", json!("Store.Get")),
        ("/application/host/effects/0/name", json!("store..get")),
        ("/application/host/effects/0/name", json!("store.")),
        ("/application/host/effects/0/name", json!("")),
        ("/application/host/effects/0/name", json!("fn")),
        ("/application/host/effects/0/name", json!("failure")),
        ("/application/host/effects/0/name", json!("std")),
        ("/application/host/effects/0/name", json!("store_get")),
        ("/application/host/effects/0/name", json!("clock.now")),
        ("/application/host/effects/0/request/0/name", json!("Key")),
        ("/application/host/effects/0/request/0/name", json!("self")),
        ("/application/host/effects/0/request/0/name", json!("limit")),
        ("/application/host/effects/0/request/0/name", json!("trailing_")),
        ("/application/host/effects/0/request/0/name", json!("double__score")),
        ("/application/host/effects/0/result/0/name", json!("note")),
        ("/application/host/effects/0/failures/0", json!("Denied")),
        ("/application/host/effects/0/failures/0", json!("self")),
        ("/application/host/effects/0/failures/0", json!("unavailable")),
        ("/application/host/effects/0/request_maximum", json!(0)),
        ("/application/host/effects/0/request_maximum", json!(7)),
        ("/application/host/effects/0/request_maximum", json!(16 * 1024 * 1024 + 1)),
        ("/application/host/effects/0/result_maximum", json!(7)),
        ("/application/host/effects/0/result_maximum", json!(16 * 1024 * 1024 + 1)),
        ("/application/host/effects/1/result_maximum", json!(11)),
        (
            "/application/host/effects/1/request_maximum",
            json!(0),
        ),
        ("/application/host/transcripts/0/vector", json!(1)),
        ("/application/host/transcripts/0/vector", json!(2)),
        ("/application/host/transcripts/0/steps", json!([])),
        (
            "/application/host/transcripts/1/steps",
            json!([
                {"effect": "clock.now", "request": [], "outcome": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]},
                {"effect": "clock.now", "request": [], "outcome": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]},
                {"effect": "clock.now", "request": [], "outcome": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]}
            ]),
        ),
        ("/application/host/transcripts/0/steps/0/effect", json!("store.put")),
        ("/application/host/transcripts/0/steps/0/request", json!([0, 0, 0, 1, 107, 0, 0, 0])),
        ("/application/host/transcripts/0/steps/0/request", json!([0, 0, 0, 1, 107, 0, 0, 0, 7, 0])),
        ("/application/host/transcripts/0/steps/0/outcome", json!([0, 0, 0, 2])),
        ("/application/host/transcripts/0/steps/0/outcome", json!([0, 0, 0, 0, 2, 0, 0, 0, 0])),
        ("/application/host/transcripts/0/steps/0/outcome", json!([0, 0, 0, 1, 0, 0, 0, 2])),
        ("/application/host/transcripts/0/steps/0/outcome", json!([0, 0, 0, 0, 1, 0, 0, 0, 1, 0xff])),
        ("/application/acceptance_vectors", json!([])),
        (
            "/application/acceptance_vectors",
            json!([{"request": [97], "response": [98]}, {"request": [97], "response": [99]}]),
        ),
        (
            "/application/acceptance_vectors/0/request",
            json!([1, 2, 3, 4, 5, 6, 7, 8, 9]),
        ),
        (
            "/application/acceptance_vectors/0/response",
            json!([1, 2, 3, 4, 5, 6, 7, 8, 9]),
        ),
    ];
    for (pointer, replacement) in mutations {
        let mut value = fixture();
        *value.pointer_mut(pointer).unwrap() = replacement.clone();
        assert!(rejected(value), "{pointer} = {replacement}");
    }
    // No effect is exercised by a transcript: the declaration is incomplete.
    let mut unexercised = fixture();
    unexercised["application"]["host"]["transcripts"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert!(rejected(unexercised));
    // The turn of the largest budget must fit a 32-bit length.
    let mut unbounded = fixture();
    unbounded["application"]["request_maximum"] = json!(64 * 1024 * 1024);
    unbounded["application"]["host"]["effect_budget"] = json!(256);
    unbounded["application"]["host"]["effects"][0]["result_maximum"] = json!(16 * 1024 * 1024);
    assert!(rejected(unbounded));
    // With no effects declared the budget, effects and transcripts are empty.
    let mut plain = fixture();
    plain["application"]["host"]["effects"] = json!([]);
    plain["application"]["host"]["transcripts"] = json!([]);
    assert!(rejected(plain.clone()));
    plain["application"]["host"]["effect_budget"] = json!(0);
    // A binding crate without an effect to serve is refused.
    assert!(rejected(plain.clone()));
    plain["application"]["bindings"] = json!([]);
    assert!(!rejected(plain));
}

#[test]
fn server_contract_binds_the_profile_and_schema_together() {
    let mut wrong_schema = fixture();
    wrong_schema["schema"] = json!("prismpm/model-document/6");
    assert!(rejected(wrong_schema));
    let mut older = fixture();
    older["schema"] = json!("prismpm/model-document/2");
    assert!(rejected(older));
    // A text document is not accepted under the server schema.
    let mut text: Value = serde_json::from_slice(
        &std::fs::read(root().join("tests/data/text-model-document.json")).unwrap(),
    )
    .unwrap();
    text["schema"] = json!("prismpm/model-document/7");
    assert!(rejected(text));
}

fn lean_failures() -> Vec<String> {
    let source =
        std::fs::read_to_string(root().join("stdlib/src/Foundation/Server/V1/Model.lex.tex"))
            .unwrap();
    let (_, after) = source.split_once("name: ServerFailure,").unwrap();
    let (block, _) = after.split_once("]\n}").unwrap();
    block
        .split("{name: ")
        .skip(1)
        .map(|row| row.split_once(',').unwrap().0.to_owned())
        .collect()
}

fn lean_classes() -> Vec<(String, String)> {
    let source =
        std::fs::read_to_string(root().join("stdlib/src/Foundation/Server/V1/Model.lex.tex"))
            .unwrap();
    source
        .lines()
        .filter_map(|line| {
            let row = line.trim().strip_prefix("| ServerFailure.")?;
            let (name, class) = row.split_once(" => (ctor ServerFailureClass.")?;
            Some((
                name.to_owned(),
                class.trim_end_matches([')', ',']).to_owned(),
            ))
        })
        .collect()
}

#[test]
fn host_failures_agree_across_lean_registry_and_generated_adapter() {
    let registry: toml::Value = toml::from_str(
        &std::fs::read_to_string(root().join("model/host-effects-diagnostics.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        registry["spec"].as_str(),
        Some("prismpm/host-effects-diagnostics/1")
    );
    let rows = registry["failure"].as_array().unwrap();
    let registered = rows
        .iter()
        .map(|row| row["code"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let lean = lean_failures();
    assert_eq!(
        registered, lean,
        "registry order must follow the Lean constructors"
    );
    assert_eq!(lean.len(), lean.iter().collect::<BTreeSet<_>>().len());
    let classes = lean_classes();
    assert_eq!(classes.len(), lean.len());
    for row in rows {
        let code = row["code"].as_str().unwrap();
        let class = classes.iter().find(|(name, _)| name == code).unwrap();
        assert_eq!(
            row["class"].as_str().unwrap(),
            class.1.to_lowercase(),
            "{code}"
        );
        assert!(!row["statement"].as_str().unwrap().is_empty(), "{code}");
    }
    let template = include_str!("../src/server_host/failure.rs.in");
    let (_, enumeration) = template.split_once("pub enum Failure {").unwrap();
    let (enumeration, _) = enumeration.split_once('}').unwrap();
    let program = enumeration
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(program, lean);
    let (_, arms) = template.split_once("pub fn class(self)").unwrap();
    let mut program_classes = Vec::new();
    // Each arm lists the failures that precede its `=> FailureClass::Class`.
    let mut remaining = arms;
    while let Some((names, rest)) = remaining.split_once("=> FailureClass::") {
        let class = rest
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>();
        for name in names.split("Failure::").skip(1) {
            let name = name
                .chars()
                .take_while(char::is_ascii_alphanumeric)
                .collect::<String>();
            program_classes.push((name, class.clone()));
        }
        remaining = rest;
    }
    program_classes.sort();
    let mut modeled = classes.clone();
    modeled.sort();
    assert_eq!(program_classes, modeled);
}

fn list(name: &str, rows: Vec<String>) -> String {
    rows.into_iter()
        .rev()
        .fold(format!("(nil {name})"), |tail, head| {
            format!("(cons {head} {tail})")
        })
}

fn record(name: &str, fields: Vec<(&str, String)>) -> String {
    let fields = fields
        .into_iter()
        .map(|(name, value)| format!("{name} := {value}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("(record {MODEL}::{name} {{{fields}}})")
}

fn text(value: &Value) -> String {
    serde_json::to_string(value.as_str().unwrap()).unwrap()
}

fn texts(values: &Value) -> String {
    list(
        "String",
        values.as_array().unwrap().iter().map(text).collect(),
    )
}

fn octets(value: &Value) -> String {
    let bytes = value
        .as_array()
        .unwrap()
        .iter()
        .map(|byte| format!("{:02x}", byte.as_u64().unwrap()))
        .collect::<String>();
    format!("0x{bytes}")
}

fn fields(values: &Value) -> String {
    list(
        &format!("{MODEL}::HostField"),
        values
            .as_array()
            .unwrap()
            .iter()
            .map(|field| {
                let kind = match field["kind"].as_str().unwrap() {
                    "u32" => "U32",
                    "u64" => "U64",
                    "flag" => "Flag",
                    "bytes" => "Bytes",
                    _ => "Text",
                };
                record(
                    "HostField",
                    vec![
                        ("name", text(&field["name"])),
                        ("kind", format!("(ctor {MODEL}::HostKind.{kind})")),
                    ],
                )
            })
            .collect(),
    )
}

fn number(value: &Value, kind: &str) -> String {
    format!("({} : {kind})", value.as_u64().unwrap())
}

fn notation(application: &Value) -> String {
    let host = &application["host"];
    let effects = list(
        &format!("{MODEL}::HostEffect"),
        host["effects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|effect| {
                record(
                    "HostEffect",
                    vec![
                        ("name", text(&effect["name"])),
                        ("request", fields(&effect["request"])),
                        ("result", fields(&effect["result"])),
                        ("failures", texts(&effect["failures"])),
                        ("requestMaximum", number(&effect["request_maximum"], "UInt32")),
                        ("resultMaximum", number(&effect["result_maximum"], "UInt32")),
                    ],
                )
            })
            .collect(),
    );
    let transcripts = list(
        &format!("{MODEL}::HostTranscript"),
        host["transcripts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|transcript| {
                record(
                    "HostTranscript",
                    vec![
                        ("vector", number(&transcript["vector"], "UInt32")),
                        (
                            "steps",
                            list(
                                &format!("{MODEL}::HostStep"),
                                transcript["steps"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|step| {
                                        record(
                                            "HostStep",
                                            vec![
                                                ("effect", text(&step["effect"])),
                                                ("request", octets(&step["request"])),
                                                ("outcome", octets(&step["outcome"])),
                                            ],
                                        )
                                    })
                                    .collect(),
                            ),
                        ),
                    ],
                )
            })
            .collect(),
    );
    let protocol = record(
        "HostProtocol",
        vec![
            ("protocol", text(&host["protocol"])),
            ("description", text(&host["description"])),
            ("usageCode", number(&host["usage_code"], "UInt8")),
            ("ioCode", number(&host["io_code"], "UInt8")),
            ("faultCode", number(&host["fault_code"], "UInt8")),
            ("effectBudget", number(&host["effect_budget"], "UInt32")),
            ("effects", effects),
            ("transcripts", transcripts),
        ],
    );
    let vectors = list(
        &format!("{MODEL}::AcceptanceVector"),
        application["acceptance_vectors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|vector| {
                record(
                    "AcceptanceVector",
                    vec![
                        ("request", octets(&vector["request"])),
                        ("response", octets(&vector["response"])),
                    ],
                )
            })
            .collect(),
    );
    let core = record(
        "ServerCore",
        vec![
            ("root", text(&application["core"]["root"])),
            ("package", text(&application["core"]["package"])),
            ("version", text(&application["core"]["version"])),
        ],
    );
    let bindings = list(
        &format!("{MODEL}::ServerBinding"),
        application["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|binding| {
                record(
                    "ServerBinding",
                    vec![
                        ("path", text(&binding["path"])),
                        ("crateName", text(&binding["crate_name"])),
                        ("host", text(&binding["host"])),
                    ],
                )
            })
            .collect(),
    );
    record(
        "ServerApplication",
        vec![
            ("profile", text(&application["profile"])),
            ("name", text(&application["name"])),
            ("cargoName", text(&application["cargo_name"])),
            ("cargoVersion", text(&application["cargo_version"])),
            ("cargoDescription", text(&application["cargo_description"])),
            ("cargoRepository", text(&application["cargo_repository"])),
            ("cargoHomepage", text(&application["cargo_homepage"])),
            ("core", core),
            ("acceptanceVectors", vectors),
            ("requestMaximum", number(&application["request_maximum"], "UInt32")),
            ("responseMaximum", number(&application["response_maximum"], "UInt32")),
            ("bindings", bindings),
            ("host", protocol),
        ],
    )
}

fn source(application: &Value, owner: &str) -> String {
    format!(
        "\\begin{{lexlean}}{{ServerProbe}}\n\\useglossary{{lexlean.std.bool@1.2.0}}\n\\useglossary{{lexlean.std.nat@1.2.0}}\n\\useglossary{{prism.arch@1.0.0}}\n\\importmodule{{Foundation.Bytes}}\n\\importmodule{{{owner}}}\n\\title{{Server application model}}\n\\begin{{semanticmodule}}\ndefinition{{\n  name: application,\n  parameters: [],\n  result: {owner}::ServerApplication,\n  body: {}\n}}\n\ndefinition{{\n  name: dispatch,\n  parameters: [value : Bytes],\n  result: Bytes,\n  body: 0x62,\n  axioms: []\n}}\n\\end{{semanticmodule}}\n\\end{{lexlean}}\n",
        notation(application).replace(MODEL, owner)
    )
}

#[allow(clippy::result_large_err)]
fn project(source: &str, owner: &str) -> Result<ModelDocument, prismpm::PrismError> {
    let temp = tempfile::tempdir().unwrap();
    let repo = root();
    let project = repo.join("tests/fixtures/holo/ho-11-text-application/project");
    for name in [
        "lexlean.toml",
        "lexlean.lock",
        "lean-toolchain",
        "lakefile.toml",
        "lake-manifest.json",
    ] {
        let text = std::fs::read_to_string(project.join(name))
            .unwrap()
            .replace("PrismTextRequest", "ServerProbe")
            .replace("TextRequest.lex.tex", "ServerProbe.lex.tex");
        std::fs::write(temp.path().join(name), text).unwrap();
    }
    let bytes = temp.path().join("src/Foundation/Bytes.lex.tex");
    std::fs::create_dir_all(bytes.parent().unwrap()).unwrap();
    std::fs::copy(repo.join("stdlib/src/Foundation/Bytes.lex.tex"), bytes).unwrap();
    let model = std::fs::read_to_string(repo.join("stdlib/src/Foundation/Server/V1/Model.lex.tex"))
        .unwrap()
        .replace(MODEL, owner);
    let path = temp
        .path()
        .join("src")
        .join(owner.replace('.', "/") + ".lex.tex");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, model).unwrap();
    std::fs::write(temp.path().join("src/ServerProbe.lex.tex"), source).unwrap();
    let language = repo.join("language/prism.arch");
    for entry in walkdir::WalkDir::new(&language) {
        let entry = entry.unwrap();
        let destination = temp
            .path()
            .join("language/prism.arch")
            .join(entry.path().strip_prefix(&language).unwrap());
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(destination).unwrap();
        } else {
            std::fs::copy(entry.path(), destination).unwrap();
        }
    }
    let manifest = camino::Utf8PathBuf::from_path_buf(temp.path().join("lexlean.toml")).unwrap();
    Engine::load(&manifest)
        .unwrap()
        .lock(LockRequest {
            check_only: false,
            allow_network: false,
        })
        .unwrap();
    let snapshot = Engine::load(&manifest)
        .unwrap()
        .snapshot(CheckRequest {
            selection: Selection::Entrypoints,
        })
        .unwrap();
    prismpm::holo::application::project_application(&snapshot).map(Option::unwrap)
}

#[test]
fn server_application_projects_only_from_the_exact_source_owned_type() {
    let application = declaration();
    let source = source(&application, MODEL);
    let doc = project(&source, MODEL).unwrap();
    assert_eq!(doc.schema, "prismpm/model-document/7");
    let Some(Application::Server(server)) = &doc.application else {
        panic!("the server declaration projects a server application");
    };
    assert_eq!(serde_json::to_value(server).unwrap(), application);
    assert!(schema().is_valid(&serde_json::to_value(&doc).unwrap()));
    assert_eq!(project(&source, MODEL).unwrap(), doc);

    let wrong_profile = source.replace("prismpm/server-application/1", "prismpm/server-application/2");
    assert_eq!(project(&wrong_profile, MODEL).unwrap_err().code, "PP2012");
    let foreign = source.replace(MODEL, "Foreign.Server.Model");
    assert_eq!(
        project(&foreign, "Foreign.Server.Model").unwrap_err().code,
        "PP2012"
    );
    // A transcript step that names an undeclared effect is refused when the source is projected.
    let undeclared = source.replace("\"clock.now\", request := 0x", "\"clock.tick\", request := 0x");
    assert_eq!(project(&undeclared, MODEL).unwrap_err().code, "PP2012");
    // The entry root must be a definition of the sources.
    let missing = source.replace("name: dispatch,", "name: other,");
    assert_eq!(project(&missing, MODEL).unwrap_err().code, "PP2001");
}
