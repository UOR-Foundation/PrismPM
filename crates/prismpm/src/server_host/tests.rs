//! Generation, structure and execution tests of the host adapter.
//!
//! The execution tests compile the generated package against a small core
//! crate that stands in for the generated application crate, run the generated
//! acceptance suite, and drive the real program for every vector and every
//! registered failure. A second, rogue core requests an effect the declaration
//! does not contain: the program must refuse it.

use super::{acceptance, generate, help, library_name, program_name};
use crate::holo::server_application::wire::{
    encode_fields, encode_outcome, Outcome, Value as Wire,
};
use crate::holo::server_application::{validate, HostEffect, HostKind, ServerApplication};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const USAGE: i32 = 64;
const IO: i32 = 74;
const FAULT: i32 = 70;

/// A pure root following the turn protocol: `a` performs nothing, `g` reads
/// one key, `t` reads, asks the time and logs.
const CORE: &str = r#"
fn word(turn: &[u8], at: &mut usize) -> u32 {
    let value = u32::from_be_bytes([turn[*at], turn[*at + 1], turn[*at + 2], turn[*at + 3]]);
    *at += 4;
    value
}

fn done(response: &[u8]) -> Vec<u8> {
    let mut out = vec![0, 0, 0, 0];
    out.extend_from_slice(response);
    out
}

fn perform(tag: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0, 0, 0, 1];
    out.extend_from_slice(&tag.to_be_bytes());
    out.extend_from_slice(payload);
    out
}

fn chunk(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}

pub fn dispatch(turn: Vec<u8>) -> Vec<u8> {
    let mut at = 0;
    let length = word(&turn, &mut at) as usize;
    let request = turn[at..at + length].to_vec();
    at += length;
    let count = word(&turn, &mut at) as usize;
    let mut outcomes: Vec<Vec<u8>> = Vec::new();
    for _ in 0..count {
        let _tag = word(&turn, &mut at);
        let length = word(&turn, &mut at) as usize;
        outcomes.push(turn[at..at + length].to_vec());
        at += length;
    }
    let get = |key: &[u8], limit: u32| {
        let mut payload = Vec::new();
        chunk(&mut payload, key);
        payload.extend_from_slice(&limit.to_be_bytes());
        payload
    };
    match request.first() {
        Some(b'a') => done(b"plain"),
        Some(b'g') => match outcomes.first() {
            None => perform(0, &get(&request[1..], 7)),
            Some(outcome) => match outcome[3] {
                0 => {
                    let mut out = b"found:".to_vec();
                    let value_length = u32::from_be_bytes([outcome[13], outcome[14], outcome[15], outcome[16]]) as usize;
                    out.extend_from_slice(&outcome[17..17 + value_length]);
                    done(&out)
                }
                _ if outcome[7] == 0 => done(b"denied"),
                _ => done(b"unavailable"),
            },
        },
        Some(b't') => match outcomes.len() {
            0 => perform(0, &get(b"t", 1)),
            1 => perform(1, &[]),
            2 => {
                let mut payload = Vec::new();
                chunk(&mut payload, b"t");
                perform(2, &payload)
            }
            _ => {
                let millis = u64::from_be_bytes([
                    outcomes[1][4], outcomes[1][5], outcomes[1][6], outcomes[1][7],
                    outcomes[1][8], outcomes[1][9], outcomes[1][10], outcomes[1][11],
                ]);
                done(format!("time:{millis}").as_bytes())
            }
        },
        _ => done(b"unknown"),
    }
}
"#;

/// What the LexLean `rust-std` package runtime exports, as far as the adapter
/// uses it, around a hand-written step function: `run` takes the turn as
/// `Bytes` and returns the step as a list of numbers or fails with `Overflow`.
/// An empty step stands for `Overflow`, a step of `256` for an element that is
/// not one octet.
const PRELUDE: &str = r#"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow;

#[derive(Clone)]
pub struct Bytes(std::rc::Rc<[u8]>);
impl Bytes {
    pub fn lit(octets: &[u8]) -> Bytes { Bytes(std::rc::Rc::from(octets)) }
    pub fn octets(&self) -> &[u8] { &self.0 }
}

#[derive(Clone)]
pub struct List<T>(std::rc::Rc<Vec<T>>, usize);
impl<T: Clone> List<T> {
    pub fn uncons(&self) -> Option<(T, List<T>)> {
        self.0.get(self.1).map(|head| (head.clone(), List(self.0.clone(), self.1 + 1)))
    }
}

pub fn run(turn: Bytes) -> Result<List<u64>, Overflow> {
    let step = dispatch(turn.octets().to_vec());
    if step.is_empty() {
        return Err(Overflow);
    }
    let mut numbers: Vec<u64> = step.iter().map(|byte| u64::from(*byte)).collect();
    if step[0] == 255 {
        numbers[0] = 256;
    }
    Ok(List(std::rc::Rc::new(numbers), 0))
}
"#;

/// A root that misbehaves by request: `r` requests an effect the declaration
/// does not contain, `o` overflows, `w` returns an element that is no octet.
const ROGUE: &str = r#"
pub fn dispatch(turn: Vec<u8>) -> Vec<u8> {
    match turn.get(4) {
        Some(b'o') => Vec::new(),
        Some(b'w') => vec![255, 0, 0, 0],
        _ => {
            let mut out = vec![0, 0, 0, 1];
            out.extend_from_slice(&9u32.to_be_bytes());
            out
        }
    }
}
"#;

fn field(name: &str, kind: &str) -> Value {
    json!({"name": name, "kind": kind})
}

fn effects() -> Value {
    json!([
        {
            "name": "store.get",
            "request": [field("key", "bytes"), field("limit", "u32")],
            "result": [field("found", "flag"), field("version", "u64"), field("value", "text")],
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
        },
        {
            "name": "log.write",
            "request": [field("line", "text")],
            "result": [],
            "failures": ["full"],
            "request_maximum": 64,
            "result_maximum": 8
        }
    ])
}

fn declared() -> Vec<HostEffect> {
    serde_json::from_value(effects()).expect("effects deserialize")
}

fn request(name: &str, values: &[Wire]) -> Vec<u8> {
    let effects = declared();
    let effect = effects
        .iter()
        .find(|candidate| candidate.name == name)
        .unwrap();
    encode_fields(&effect.request, values).expect("request encodes")
}

fn outcome(name: &str, outcome: &Outcome) -> Vec<u8> {
    let effects = declared();
    let effect = effects
        .iter()
        .find(|candidate| candidate.name == name)
        .unwrap();
    encode_outcome(effect, outcome).expect("outcome encodes")
}

fn step(effect: &str, request_bytes: Vec<u8>, outcome_bytes: Vec<u8>) -> Value {
    json!({"effect": effect, "request": request_bytes, "outcome": outcome_bytes})
}

fn key_request(key: &[u8], limit: u32) -> Vec<u8> {
    request("store.get", &[Wire::Bytes(key.to_vec()), Wire::U32(limit)])
}

fn found(version: u64, value: &str) -> Vec<u8> {
    outcome(
        "store.get",
        &Outcome::Completed(vec![
            Wire::Flag(true),
            Wire::U64(version),
            Wire::Text(value.to_owned()),
        ]),
    )
}

fn model_value() -> Value {
    json!({
        "profile": "prismpm/server-application/1",
        "name": "Echo Host",
        "cargo_name": "echo-core",
        "cargo_version": "0.1.0",
        "cargo_description": "Echo host",
        "cargo_repository": "https://example.invalid/echo",
        "cargo_homepage": "https://example.invalid/echo",
        "acceptance_vectors": [
            {"request": b"a".to_vec(), "response": b"plain".to_vec()},
            {"request": b"gk".to_vec(), "response": b"found:v".to_vec()},
            {"request": b"gx".to_vec(), "response": b"denied".to_vec()},
            {"request": b"gy".to_vec(), "response": b"unavailable".to_vec()},
            {"request": b"t".to_vec(), "response": b"time:42".to_vec()}
        ],
        "core": {"root": "Echo.Host.run", "package": "echo_core", "version": "0.1.0"},
        "bindings": [{"crate_name": "echo-binding", "version": "0.1.0", "host": "echo_binding::Native"}],
        "request_maximum": 16,
        "response_maximum": 64,
        "host": {
            "protocol": "prismpm/host-effects/1",
            "description": "Replays a scripted host for one exchange.\n",
            "usage_code": USAGE,
            "io_code": IO,
            "fault_code": FAULT,
            "effect_budget": 4,
            "effects": effects(),
            "transcripts": [
                {"vector": 1, "steps": [step("store.get", key_request(b"k", 7), found(3, "v"))]},
                {"vector": 2, "steps": [step(
                    "store.get",
                    key_request(b"x", 7),
                    outcome("store.get", &Outcome::Failed(0)),
                )]},
                {"vector": 3, "steps": [step(
                    "store.get",
                    key_request(b"y", 7),
                    outcome("store.get", &Outcome::Failed(1)),
                )]},
                {"vector": 4, "steps": [
                    step(
                        "store.get",
                        key_request(b"t", 1),
                        outcome("store.get", &Outcome::Completed(vec![
                            Wire::Flag(false), Wire::U64(0), Wire::Text(String::new()),
                        ])),
                    ),
                    step(
                        "clock.now",
                        request("clock.now", &[]),
                        outcome("clock.now", &Outcome::Completed(vec![Wire::U64(42)])),
                    ),
                    step(
                        "log.write",
                        request("log.write", &[Wire::Text("t".to_owned())]),
                        outcome("log.write", &Outcome::Completed(vec![])),
                    )
                ]}
            ]
        }
    })
}

pub(super) fn model() -> ServerApplication {
    serde_json::from_value(model_value()).expect("fixture deserializes")
}

fn file<'a>(package: &'a prod_codegen::GeneratedPackage, path: &str) -> &'a str {
    let file = package
        .files
        .iter()
        .find(|file| file.path == path)
        .unwrap_or_else(|| panic!("generated package lacks {path}"));
    std::str::from_utf8(&file.bytes).unwrap()
}

struct Built {
    _directory: tempfile::TempDir,
    program: PathBuf,
    package: PathBuf,
    target: PathBuf,
}

fn cargo(arguments: &[&str], directory: &Path, target: &Path) -> Output {
    Command::new("cargo")
        .args(arguments)
        .current_dir(directory)
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("cargo runs")
}

fn build(application: &ServerApplication, core_source: &str) -> Built {
    let directory = tempfile::tempdir().unwrap();
    let core = directory.path().join("core");
    std::fs::create_dir_all(core.join("src")).unwrap();
    std::fs::write(
        core.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2021\"\n",
            application.core.package, application.core.version
        ),
    )
    .unwrap();
    std::fs::write(core.join("src/lib.rs"), format!("{PRELUDE}{core_source}")).unwrap();
    let package = directory.path().join("host");
    let generated = generate(application).unwrap();
    for item in &generated.files {
        let path = package.join(&item.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = if item.path == "Cargo.toml" {
            // The generated dependency is a locked registry version; the stand-in
            // core is a path dependency.
            let manifest = String::from_utf8(item.bytes.clone()).unwrap();
            let (head, _) = manifest.split_once("[dependencies]").unwrap();
            let (_, tail) = manifest.split_once("[profile.release]").unwrap();
            format!(
                "{head}[dependencies]\n{} = {{ path = \"../core\" }}\n\n[profile.release]{tail}",
                application.core.package
            )
            .into_bytes()
        } else {
            item.bytes.clone()
        };
        std::fs::write(path, bytes).unwrap();
    }
    let target = directory.path().join("target");
    let output = cargo(&["build", "--offline"], &package, &target);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Built {
        program: target.join("debug").join(program_name(application)),
        package,
        target,
        _directory: directory,
    }
}

fn built() -> &'static Built {
    static BUILT: OnceLock<Built> = OnceLock::new();
    BUILT.get_or_init(|| build(&model(), CORE))
}

fn rogue() -> &'static Built {
    static BUILT: OnceLock<Built> = OnceLock::new();
    BUILT.get_or_init(|| build(&model(), ROGUE))
}

fn run(program: &Path, arguments: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("program starts");
    let _ = child.stdin.take().unwrap().write_all(stdin);
    child.wait_with_output().unwrap()
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("exit code")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn script_file(directory: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = directory.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn the_fixture_is_a_valid_declaration() {
    validate(&model()).unwrap();
}

#[test]
fn generated_package_has_the_closed_file_set() {
    let package = generate(&model()).unwrap();
    let mut paths = package
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    paths.sort_unstable();
    assert_eq!(
        paths,
        [
            "Cargo.toml",
            "src/acceptance.rs",
            "src/exchange.rs",
            "src/failure.rs",
            "src/lib.rs",
            "src/main.rs",
            "src/protocol.rs",
            "src/script.rs",
            "src/surface.rs",
            "src/wire.rs",
            "tests/acceptance.rs",
        ]
    );
    let manifest = file(&package, "Cargo.toml");
    assert!(manifest.contains("name = \"echo-core-host\""));
    assert!(manifest.contains("echo_core = { version = \"=0.1.0\" }"));
    assert_eq!(library_name(&model()), "echo_core_host");
    assert!(file(&package, "src/exchange.rs").contains("echo_core::run(echo_core::Bytes::lit(&input))"));
    assert!(file(&package, "src/main.rs").contains("use echo_core_host::"));
    assert!(!file(&package, "src/main.rs").contains('@'));
}

#[test]
fn the_request_sum_and_host_trait_contain_exactly_the_declared_effects() {
    let package = generate(&model()).unwrap();
    let protocol = file(&package, "src/protocol.rs");
    let (_, sum) = protocol.split_once("pub enum Request {").unwrap();
    let (sum, _) = sum.split_once('}').unwrap();
    let variants = sum
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "StoreGet(store_get::Request),",
            "ClockNow(clock_now::Request),",
            "LogWrite(log_write::Request),"
        ]
    );
    let (_, host) = protocol.split_once("pub trait Host {").unwrap();
    let (host, _) = host.split_once("\n}\n").unwrap();
    assert_eq!(host.matches("fn ").count(), 3);
    for method in ["fn store_get(", "fn clock_now(", "fn log_write("] {
        assert!(host.contains(method), "{method}");
    }
    assert!(protocol.contains("pub const EFFECTS: &[&str] = &[\n    \"store.get\",\n    \"clock.now\",\n    \"log.write\",\n];"));
}

#[test]
fn an_application_without_effects_renders_the_plain_frame_function() {
    let mut value = model_value();
    value["host"]["effect_budget"] = json!(0);
    value["host"]["effects"] = json!([]);
    value["host"]["transcripts"] = json!([]);
    value["bindings"] = json!([]);
    let plain: ServerApplication = serde_json::from_value(value).unwrap();
    validate(&plain).unwrap();
    let package = generate(&plain).unwrap();
    let protocol = file(&package, "src/protocol.rs");
    assert!(protocol.contains("pub enum Request {\n}"));
    assert!(protocol.contains("pub const EFFECTS: &[&str] = &[\n];"));
    assert!(file(&package, "src/surface.rs").contains("pub const EFFECT_BUDGET: usize = 0;"));
}

#[test]
fn help_is_derived_only_from_the_declaration() {
    let text = help(&model());
    assert!(text.starts_with("Echo Host 0.1.0\n\nReplays a scripted host for one exchange.\n"));
    assert!(text.contains("echo-core-host exchange SCRIPT [FILE]"));
    assert!(text.contains("at most 16 of them"));
    assert!(text.contains("  0  store.get  request at most 64 bytes, outcome at most 64 bytes"));
    assert!(text.contains(" 64  usage"));
    assert!(text.contains(" 70  fault"));
    assert!(text.contains(" 74  io"));
}

#[test]
fn the_generated_suite_and_program_tests_pass_against_the_core() {
    let built = built();
    let output = cargo(&["test", "--offline"], &built.package, &built.target);
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(output.status.success(), "{text}\n{}", stderr(&output));
    for name in [
        "every_vector_replays_its_transcript_to_its_response",
        "every_declared_effect_is_performed_by_some_vector",
        "undeclared_effects_cannot_be_decoded_or_scripted",
        "a_root_cannot_step_outside_the_protocol",
        "the_effect_budget_bounds_every_exchange",
        "the_scripted_host_refuses_a_script_that_does_not_match",
    ] {
        assert!(text.contains(&format!("{name} ... ok")), "{name}\n{text}");
    }
}

#[test]
fn the_program_accepts_every_vector_over_a_file_and_standard_input() {
    let built = built();
    let work = tempfile::tempdir().unwrap();
    let record = acceptance::execute(&model(), &built.program, work.path()).unwrap();
    assert_eq!(record["schema"], acceptance::SCHEMA);
    assert_eq!(record["status"], "verified");
    assert_eq!(record["vectors"], 5);
    assert_eq!(record["steps"], 6);
    assert_eq!(
        record["effects"],
        json!([
            {"name": "store.get", "steps": 4},
            {"name": "clock.now", "steps": 1},
            {"name": "log.write", "steps": 1}
        ])
    );
    assert_eq!(record["program_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn acceptance_refuses_a_response_that_differs_from_the_model() {
    let built = built();
    let work = tempfile::tempdir().unwrap();
    let mut value = model_value();
    value["acceptance_vectors"][0]["response"] = json!(b"other".to_vec());
    let changed: ServerApplication = serde_json::from_value(value).unwrap();
    assert_eq!(
        acceptance::execute(&changed, &built.program, work.path())
            .unwrap_err()
            .code,
        "PP6003"
    );
}

#[test]
fn the_program_prints_help_and_version_from_the_declaration() {
    let built = built();
    let output = run(&built.program, &["--help"], b"");
    assert_eq!(code(&output), 0);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), help(&model()));
    let output = run(&built.program, &["--version"], b"");
    assert_eq!(code(&output), 0);
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "echo-core-host 0.1.0\n"
    );
}

#[test]
fn invalid_command_lines_exit_with_the_usage_code() {
    let built = built();
    for (arguments, failure) in [
        (vec![], "MissingCommand"),
        (vec!["--bogus"], "UnknownFlag"),
        (vec!["bogus"], "UnknownCommand"),
        (vec!["exchange"], "MissingOperand"),
        (vec!["exchange", "a", "b", "c"], "ExtraArgument"),
    ] {
        let output = run(&built.program, &arguments, b"");
        assert_eq!(code(&output), USAGE, "{arguments:?}");
        assert!(
            stderr(&output).contains(failure),
            "{arguments:?}: {}",
            stderr(&output)
        );
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn unreadable_missing_and_non_regular_inputs_exit_with_the_io_code() {
    let built = built();
    let work = tempfile::tempdir().unwrap();
    let script = script_file(work.path(), "empty.script", b"");
    let script = script.to_str().unwrap();
    let missing = work.path().join("missing");
    for (arguments, failure) in [
        (vec!["exchange", missing.to_str().unwrap()], "InputMissing"),
        (
            vec!["exchange", script, missing.to_str().unwrap()],
            "InputMissing",
        ),
        (vec!["exchange", work.path().to_str().unwrap()], "InputType"),
        (
            vec!["exchange", script, work.path().to_str().unwrap()],
            "InputType",
        ),
    ] {
        let output = run(&built.program, &arguments, b"a");
        assert_eq!(code(&output), IO, "{arguments:?}");
        assert!(
            stderr(&output).contains(failure),
            "{arguments:?}: {}",
            stderr(&output)
        );
    }
    #[cfg(unix)]
    {
        let link = work.path().join("link");
        std::os::unix::fs::symlink(script, &link).unwrap();
        let output = run(&built.program, &["exchange", link.to_str().unwrap()], b"a");
        assert_eq!(code(&output), IO);
        assert!(stderr(&output).contains("InputType"), "{}", stderr(&output));
    }
}

#[test]
fn oversized_requests_and_scripts_are_refused_before_dispatch() {
    let built = built();
    let work = tempfile::tempdir().unwrap();
    let script = script_file(work.path(), "empty.script", b"");
    let output = run(
        &built.program,
        &["exchange", script.to_str().unwrap()],
        &[b'a'; 17],
    );
    assert_eq!(code(&output), FAULT);
    assert!(stderr(&output).contains("InputLimit"));
    let long = script_file(
        work.path(),
        "long.script",
        &vec![0u8; 4 * (12 + 64 + 64) + 1],
    );
    let output = run(&built.program, &["exchange", long.to_str().unwrap()], b"a");
    assert_eq!(code(&output), FAULT);
    assert!(stderr(&output).contains("ScriptMalformed"));
}

#[test]
fn script_failures_exit_with_the_fault_code_and_publish_nothing() {
    let built = built();
    let work = tempfile::tempdir().unwrap();
    let application = model();
    let complete = acceptance::script_bytes(&application, 1).unwrap();
    let nothing = script_file(work.path(), "nothing.script", b"");
    let mut flipped = complete.clone();
    let at = 4 + 4 + 4; // the key byte of the encoded request, past tag, length and key length
    flipped[at] ^= 0xff;
    let mismatched = script_file(work.path(), "mismatched.script", &flipped);
    let mut extra = complete.clone();
    extra.extend_from_slice(&complete);
    let unused = script_file(work.path(), "unused.script", &extra);
    let truncated = script_file(
        work.path(),
        "truncated.script",
        &complete[..complete.len() - 1],
    );
    let complete = script_file(work.path(), "complete.script", &complete);
    for (script, request, failure) in [
        (&nothing, &b"gk"[..], "ScriptExhausted"),
        (&mismatched, &b"gk"[..], "ScriptMismatch"),
        (&unused, &b"gk"[..], "ScriptUnused"),
        (&truncated, &b"gk"[..], "ScriptMalformed"),
        (&complete, &b"a"[..], "ScriptUnused"),
    ] {
        let output = run(
            &built.program,
            &["exchange", script.to_str().unwrap()],
            request,
        );
        assert_eq!(code(&output), FAULT, "{failure}");
        assert!(
            stderr(&output).contains(failure),
            "{failure}: {}",
            stderr(&output)
        );
        assert!(output.stdout.is_empty(), "{failure}");
    }
    let output = run(
        &built.program,
        &["exchange", complete.to_str().unwrap()],
        b"gk",
    );
    assert_eq!(code(&output), 0);
    assert_eq!(output.stdout, b"found:v");
}

#[test]
fn a_core_that_requests_an_undeclared_effect_is_refused_by_the_program() {
    let rogue = rogue();
    let work = tempfile::tempdir().unwrap();
    let script = script_file(work.path(), "empty.script", b"");
    for (request, failure) in [
        (&b"r"[..], "EffectUndeclared"),
        (&b"o"[..], "ComputeFailure"),
        (&b"w"[..], "StepMalformed"),
    ] {
        let output = run(
            &rogue.program,
            &["exchange", script.to_str().unwrap()],
            request,
        );
        assert_eq!(code(&output), FAULT, "{failure}");
        assert!(stderr(&output).contains(failure), "{}", stderr(&output));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn declared_kinds_are_the_only_wire_kinds() {
    // The fixture exercises every value kind the declaration admits.
    let effects = declared();
    let kinds = effects
        .iter()
        .flat_map(|effect| effect.request.iter().chain(&effect.result))
        .map(|field| field.kind)
        .collect::<Vec<_>>();
    for kind in [
        HostKind::U32,
        HostKind::U64,
        HostKind::Flag,
        HostKind::Bytes,
        HostKind::Text,
    ] {
        assert!(kinds.contains(&kind), "{kind:?}");
    }
}
