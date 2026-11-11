//! The generated writers reproduce the `prism-code generate` golden files of
//! the Calculator fixture byte for byte, and refuse what they cannot write.

use prism_bindings::{
    write, BindingApplication, BindingErrorCase, BindingFunction, BindingInterface, BindingModel,
    BindingOperation, BindingOperations, BindingOutput, BindingParameter, BindingWorld, Runtime,
};
use std::path::PathBuf;

fn function(name: &str, parameters: &[(&str, &str)], result: &str, doc: &str) -> BindingFunction {
    BindingFunction {
        name: name.into(),
        parameters: parameters
            .iter()
            .map(|(name, scalar)| BindingParameter {
                name: (*name).into(),
                scalar: (*scalar).into(),
            })
            .collect(),
        result: Some(result.into()),
        docs: vec![doc.into()],
    }
}

/// The code model of `sdk/npm/prism-code/fixtures/Calculator.holo`.
fn calculator() -> BindingModel {
    let operations = [
        ("add", "Add"),
        ("subtract", "Subtract"),
        ("multiply", "Multiply"),
        ("divide", "Divide"),
    ];
    let errors = [
        ("overflow", Some("Arithmetic overflow.")),
        ("division-by-zero", Some("Division by zero.")),
        ("malformed-request", Some("Enter a signed 64-bit integer.")),
        ("unsupported-version", None),
        ("unknown-operation", None),
    ];
    BindingModel {
        generator: "prism-code generate".into(),
        world: BindingWorld {
            contract: "hologram:guest/core-wasm@1".into(),
            package: "hologram:guest".into(),
            name: "core-wasm".into(),
            file: "worlds/core-wasm.wit".into(),
            blake3: "0774a8ecbfa945a427712ee7d7ad406195b2d07492ceae15cca0067276c9ced8".into(),
            sha256: "a566c87a359cadf98f652747f1b3574aa307ff5f652a190c6d99c350db9cf369".into(),
            memory: "memory".into(),
            imports: Vec::new(),
            exports: vec![
                function(
                    "holo_alloc",
                    &[("length", "u32")],
                    "u32",
                    "Reserve `length` bytes of guest memory for a request and return their offset.",
                ),
                function(
                    "holo_run",
                    &[("offset", "u32"), ("length", "u32")],
                    "u64",
                    "Run the request at `offset` with `length` bytes. Returns `offset << 32 | length` of the response.",
                ),
            ],
        },
        application: BindingApplication {
            name: "Calculator".into(),
            type_name: "Calculator".into(),
            file: "calculator".into(),
            description: Some("A Lean-verified, Prism-generated checked i64 calculator.".into()),
            schema: "prismpm/model-document/1".into(),
            blake3: "e76544838a98c46341ed8807f72897ec68de5a8a9bf28721738a76761c9df90e".into(),
            sha256: "074921890f5c372c89f4ed794d26dd550231d5e9bf99bf9474cc1a78923292c6".into(),
            interface: BindingInterface::Operations(BindingOperations {
                version: "1".into(),
                method: "calculate".into(),
                operation_type: "Operation".into(),
                error_type: "CalculatorError".into(),
                operations: operations
                    .iter()
                    .zip(0..)
                    .map(|((name, label), discriminant)| BindingOperation {
                        name: (*name).into(),
                        label: (*label).into(),
                        discriminant,
                    })
                    .collect(),
                errors: errors
                    .iter()
                    .map(|(code, message)| BindingErrorCase {
                        code: (*code).into(),
                        message: message.map(Into::into),
                    })
                    .collect(),
            }),
        },
    }
}

fn golden(runtime: Runtime, directory: &str) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../sdk/npm/prism-code/test/fixtures/generate")
        .join(directory);
    let BindingOutput::Files(files) = write(runtime, &calculator()).expect("no overflow") else {
        panic!("the Calculator model is refused for {runtime:?}");
    };
    let mut written: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();
    written.sort_unstable();
    let mut committed: Vec<String> = std::fs::read_dir(&root)
        .expect("golden directory")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    committed.sort_unstable();
    assert_eq!(
        written, committed,
        "{directory} lists exactly the written files"
    );
    for file in &files {
        let expected = std::fs::read_to_string(root.join(&file.path))
            .expect("golden file")
            .replace("\r\n", "\n");
        assert!(
            file.text == expected,
            "{} differs from {directory}",
            file.path
        );
    }
}

#[test]
fn javascript_equals_the_golden_files() {
    golden(Runtime::JavaScript, "calculator-js");
}

#[test]
fn rust_equals_the_golden_files() {
    golden(Runtime::Rust, "calculator-rust");
}

#[test]
fn python_equals_the_golden_files() {
    golden(Runtime::Python, "calculator-python");
}

#[test]
fn every_writer_refuses_an_unknown_world() {
    let mut model = calculator();
    model.world.contract = "hologram:guest/other@1".into();
    for runtime in [Runtime::JavaScript, Runtime::Rust, Runtime::Python] {
        let output = write(runtime, &model).expect("no overflow");
        assert!(
            matches!(output, BindingOutput::Refused(..)),
            "{runtime:?} wrote bindings for an unknown world: {output:?}"
        );
    }
}
