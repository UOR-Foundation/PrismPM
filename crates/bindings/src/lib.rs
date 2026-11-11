//! The bindings writers of `prism generate` (issue #289).
//!
//! Every writer is the LexLean model `bindings/src/Foundation/Bindings/V1`,
//! exported by `cargo xtask bindings-package` as the certified packages under
//! `bindings/generated`. This crate only names the model's values: it builds
//! the exported packages' input from a [`BindingModel`] and reads their
//! output back. It holds no writer logic.

/// A runtime the bindings model writes for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Runtime {
    /// The JavaScript module and its TypeScript declarations.
    JavaScript,
    /// A Rust client crate.
    Rust,
    /// A Python package.
    Python,
}

/// One parameter of a world function: its name and WIT scalar type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingParameter {
    /// The name.
    pub name: String,
    /// The WIT scalar type, such as `u32`.
    pub scalar: String,
}

/// One import or export of the WIT world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingFunction {
    /// The name.
    pub name: String,
    /// The parameters, in order.
    pub parameters: Vec<BindingParameter>,
    /// The WIT scalar result, or none.
    pub result: Option<String>,
    /// The WIT documentation lines.
    pub docs: Vec<String>,
}

/// The WIT world the application runs in, with its digests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingWorld {
    /// The world contract, such as `hologram:guest/core-wasm@1`.
    pub contract: String,
    /// The WIT package.
    pub package: String,
    /// The name.
    pub name: String,
    /// The file stem or path.
    pub file: String,
    /// The BLAKE3 digest, in hex.
    pub blake3: String,
    /// The SHA-256 digest, in hex.
    pub sha256: String,
    /// The exported memory name.
    pub memory: String,
    /// The imported functions.
    pub imports: Vec<BindingFunction>,
    /// The exported functions.
    pub exports: Vec<BindingFunction>,
}

/// One operation of an operations interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingOperation {
    /// The name.
    pub name: String,
    /// The display label.
    pub label: String,
    /// The operation code sent on the wire.
    pub discriminant: u64,
}

/// One error code of an operations interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingErrorCase {
    /// The error code.
    pub code: String,
    /// The fixed message, if the code has one.
    pub message: Option<String>,
}

/// A typed operations interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingOperations {
    /// The interface version.
    pub version: String,
    /// The dispatch method name.
    pub method: String,
    /// The operation type name.
    pub operation_type: String,
    /// The error type name.
    pub error_type: String,
    /// The operations, in discriminant order.
    pub operations: Vec<BindingOperation>,
    /// The error codes.
    pub errors: Vec<BindingErrorCase>,
}

/// How the application takes its requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingInterface {
    /// Raw request bytes in, response bytes out.
    RequestBytes,
    /// Typed operations.
    Operations(BindingOperations),
}

/// The application the bindings call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingApplication {
    /// The name.
    pub name: String,
    /// The type name of the generated client.
    pub type_name: String,
    /// The file stem or path.
    pub file: String,
    /// The description, if the model has one.
    pub description: Option<String>,
    /// The model document schema.
    pub schema: String,
    /// The BLAKE3 digest, in hex.
    pub blake3: String,
    /// The SHA-256 digest, in hex.
    pub sha256: String,
    /// How the application takes requests.
    pub interface: BindingInterface,
}

/// The code model a writer reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingModel {
    /// The generator named in each file header.
    pub generator: String,
    /// The WIT world.
    pub world: BindingWorld,
    /// The application.
    pub application: BindingApplication,
}

/// One written file: its path in the output directory and its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BindingFile {
    /// The path in the output directory.
    pub path: String,
    /// The file text.
    pub text: String,
}

/// What a writer returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingOutput {
    /// The files to write.
    Files(Vec<BindingFile>),
    /// The model cannot be written for this runtime: a code and a message.
    Refused(String, String),
}

/// The exported writer ran out of its checked arithmetic range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overflow;

impl std::fmt::Display for Overflow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("the bindings writer overflowed")
    }
}

impl std::error::Error for Overflow {}

/// The writers recurse to a depth bounded by their input, so each runs on a
/// thread whose stack fits any model this crate is given.
const STACK: usize = 256 << 20;

macro_rules! writer {
    ($package:ident) => {{
        use $package::{
            Adt0, Adt1, Adt10, Adt2, Adt3, Adt4, Adt5, Adt6, Adt7, Adt8, Adt9, List, Str,
        };
        fn text(value: &str) -> Str {
            Str::lit(value)
        }
        fn list<T: Clone, U>(items: &[U], each: impl Fn(&U) -> T) -> List<T> {
            items
                .iter()
                .rev()
                .fold(List::nil(), |tail, item| List::cons(each(item), tail))
        }
        fn function(value: &BindingFunction) -> Adt2 {
            Adt2::C0(
                text(&value.name),
                list(&value.parameters, |parameter| {
                    Adt3::C0(text(&parameter.name), text(&parameter.scalar))
                }),
                value.result.as_deref().map(text),
                list(&value.docs, |line| text(line)),
            )
        }
        move |model: BindingModel| -> Result<BindingOutput, Overflow> {
            let world = &model.world;
            let application = &model.application;
            let interface = match &application.interface {
                BindingInterface::RequestBytes => Adt5::C0,
                BindingInterface::Operations(operations) => Adt5::C1(Adt6::C0(
                    text(&operations.version),
                    text(&operations.method),
                    text(&operations.operation_type),
                    text(&operations.error_type),
                    list(&operations.operations, |operation| {
                        Adt7::C0(
                            text(&operation.name),
                            text(&operation.label),
                            operation.discriminant,
                        )
                    }),
                    list(&operations.errors, |error| {
                        Adt8::C0(text(&error.code), error.message.as_deref().map(text))
                    }),
                )),
            };
            let input = Adt0::C0(
                text(&model.generator),
                Adt1::C0(
                    text(&world.contract),
                    text(&world.package),
                    text(&world.name),
                    text(&world.file),
                    text(&world.blake3),
                    text(&world.sha256),
                    text(&world.memory),
                    list(&world.imports, function),
                    list(&world.exports, function),
                ),
                Adt4::C0(
                    text(&application.name),
                    text(&application.type_name),
                    text(&application.file),
                    application.description.as_deref().map(text),
                    text(&application.schema),
                    text(&application.blake3),
                    text(&application.sha256),
                    interface,
                ),
            );
            match $package::run(input).map_err(|_| Overflow)? {
                Adt9::C1(code, message) => Ok(BindingOutput::Refused(
                    code.text().to_owned(),
                    message.text().to_owned(),
                )),
                Adt9::C0(files) => {
                    let mut written = Vec::new();
                    let mut rest = files;
                    while let Some((Adt10::C0(path, body), tail)) = rest.uncons() {
                        written.push(BindingFile {
                            path: path.text().to_owned(),
                            text: body.text().to_owned(),
                        });
                        rest = tail;
                    }
                    Ok(BindingOutput::Files(written))
                }
            }
        }
    }};
}

/// Write the bindings of `model` for `runtime` with the generated writer.
pub fn write(runtime: Runtime, model: &BindingModel) -> Result<BindingOutput, Overflow> {
    let model = model.clone();
    let worker = std::thread::Builder::new()
        .name("prism-bindings".into())
        .stack_size(STACK)
        .spawn(move || match runtime {
            Runtime::JavaScript => writer!(prism_bindings_js)(model),
            Runtime::Rust => writer!(prism_bindings_rust)(model),
            Runtime::Python => writer!(prism_bindings_python)(model),
        });
    match worker.map(std::thread::JoinHandle::join) {
        Ok(Ok(output)) => output,
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(error) => panic!("cannot start the bindings writer thread: {error}"),
    }
}
