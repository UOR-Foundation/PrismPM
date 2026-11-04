//! Source-bound native, Core-Wasm and mandatory raw CLI generation for BinaryProgram.
use super::{
    relative, with_generated_core, write, CoreParts, NativeCoreProfile, NativeCoreRequest,
};
use crate::error::PrismError;
use crate::holo::canonical::content_id;
use crate::holo::model_document::ModelDocument;
use crate::verification::{executable, run_process, ProcessRecord};
use prod_codegen::{generate_core_wasm_package, CoreWasmSpec};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
pub(crate) type BinaryArtifact = (String, Vec<u8>);
pub(crate) struct GeneratedBinary {
    pub(crate) artifacts: Vec<BinaryArtifact>,
    pub(crate) processes: Vec<ProcessRecord>,
}

impl crate::exporter::ProductEvidence for GeneratedBinary {
    fn role(&self) -> &'static str {
        "binary"
    }
    fn artifacts(&self) -> &[(String, Vec<u8>)] {
        &self.artifacts
    }
    fn processes(&self) -> &[ProcessRecord] {
        &self.processes
    }
}

/// Generate unaccepted native artifacts; verification owns acceptance evidence.
pub(crate) fn export_arguments(
    modules: &BTreeSet<String>,
    program: &crate::holo::binary_program::BinaryProgram,
    export: &Path,
) -> Vec<String> {
    super::export_arguments(modules, &program.metadata(), export)
}

/// Generate unaccepted native artifacts; verification owns acceptance evidence.
pub(crate) fn generate(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
) -> Result<Vec<BinaryArtifact>, PrismError> {
    generate_recorded(
        repository_root,
        model,
        model_bytes,
        lex_root,
        lex_manifest_bytes,
    )
    .map(|generated| generated.artifacts)
}

pub(crate) fn generate_recorded(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
) -> Result<GeneratedBinary, PrismError> {
    generate_in(
        repository_root,
        model,
        model_bytes,
        lex_root,
        lex_manifest_bytes,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_owned(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
    owner: &mut crate::exporter::VerifyExporterOwner,
    phase: crate::exporter::Phase,
) -> Result<GeneratedBinary, PrismError> {
    owner.generate(
        repository_root,
        phase,
        model_bytes,
        lex_manifest_bytes,
        |package, acquisition| {
            generate_in(
                repository_root,
                model,
                model_bytes,
                lex_root,
                lex_manifest_bytes,
                Some((package, acquisition)),
            )
        },
    )
}

fn generate_in(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
    owned: Option<(&Path, &crate::exporter::GenerationContext<'_>)>,
) -> Result<GeneratedBinary, PrismError> {
    let program = model
        .program
        .as_ref()
        .ok_or_else(|| PrismError::new("PP9001", "binary generation requires a program profile"))?;
    let library = program.metadata();
    let entry = program
        .entry_root
        .rsplit('.')
        .next()
        .ok_or_else(|| PrismError::new("PP2001", "binary entry is absent"))?;
    with_generated_core(
        NativeCoreRequest {
            profile: NativeCoreProfile::Binary,
            repository_root,
            library: &library,
            lex_root,
            lex_manifest_bytes,
            owned,
        },
        |module| {
            let entry_definition = module
                .find_def(entry)
                .ok_or_else(|| PrismError::new("PP5004", "binary entry is absent from IR"))?;
            if entry_definition.params.len() != 1
                || entry_definition.params[0].1 != prod_ir::Type::Bytes
                || entry_definition.ret != prod_ir::Type::Bytes
            {
                return Err(PrismError::new(
                    "PP5004",
                    "binary entry IR signature must be Bytes to Bytes",
                ));
            }
            Ok(())
        },
        |core| {
            let CoreParts {
                workspace,
                replacements,
                module,
                kernel_sha256,
                archive_name,
                mut artifacts,
                mut processes,
            } = core;
            // The CLI consumes the exact generated core archive through an isolated offline registry.
            let core_archive = &artifacts
                .iter()
                .find(|(path, _)| path == &format!("binary/{archive_name}"))
                .expect("core archive recorded")
                .1;
            let consumer = workspace.join("consumer");
            std::fs::create_dir(&consumer)
                .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
            let consumer_home = crate::library_verification::registry_cargo_home(
                &consumer,
                &program.metadata(),
                core_archive,
            )?;
            let cargo_env = BTreeMap::from([
                (
                    "CARGO_HOME".to_owned(),
                    consumer_home.to_string_lossy().into_owned(),
                ),
                ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
                (
                    "RUSTFLAGS".to_owned(),
                    format!("--remap-path-prefix={}=$BINARY_WORK", workspace.display()),
                ),
            ]);
            let cli = crate::binary_cli::generate(program)?;
            let cli_root = workspace.join("cli");
            for file in cli.files {
                relative(&file.path)?;
                write(&cli_root.join(&file.path), &file.bytes)?;
                artifacts.push((format!("binary/cli/{}", file.path), file.bytes));
            }
            for (tool, args) in [
                ("binary-cli-lock", vec!["generate-lockfile", "--offline"]),
                (
                    "binary-cli-build",
                    vec!["build", "--locked", "--offline", "--release"],
                ),
                (
                    "binary-cli-package",
                    vec!["package", "--locked", "--offline", "--allow-dirty"],
                ),
            ] {
                processes.push(run_process(
                    tool,
                    &executable("cargo")?,
                    &args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                    &cli_root,
                    &cargo_env,
                    replacements,
                    "PP4102",
                )?);
            }
            artifacts.push((
                "binary/cli/Cargo.lock".into(),
                std::fs::read(cli_root.join("Cargo.lock"))
                    .map_err(|error| PrismError::new("PP4102", error.to_string()))?,
            ));
            let cli_archive = format!("{}-cli-{}.crate", program.cargo_name, program.cargo_version);
            artifacts.push((
                format!("binary/{cli_archive}"),
                std::fs::read(cli_root.join("target/package").join(cli_archive))
                    .map_err(|error| PrismError::new("PP4102", error.to_string()))?,
            ));
            // A generated executable is platform-specific. Record reproducible source and exact lock,
            // and build/run the adapter independently during acceptance instead of publishing a host ELF.
            let core = generate_core_wasm_package(
                &module,
                &CoreWasmSpec {
                    crate_name: format!("{}-core-wasm", program.cargo_name),
                    entry: entry.to_owned(),
                    export_name: "binary_run".into(),
                    input_allocation_cap: program.request_maximum,
                    output_allocation_cap: program.response_maximum,
                    maximum_pages: program.memory_pages,
                    input_ir_sha256: kernel_sha256,
                },
            )
            .map_err(|error| PrismError::new("PP5101", error.to_string()))?;
            let core_root = workspace.join("core-wasm");
            for file in core.files {
                relative(&file.path)?;
                write(&core_root.join(&file.path), &file.bytes)?;
                artifacts.push((format!("binary/core-wasm/{}", file.path), file.bytes));
            }
            // Do not override generated target rustflags: they bind exported memory and maximum pages.
            let wasm_env = BTreeMap::from([
                (
                    "CARGO_HOME".to_owned(),
                    consumer_home.to_string_lossy().into_owned(),
                ),
                ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
            ]);
            processes.push(run_process(
                "binary-core-wasm-build",
                &executable("cargo")?,
                &["build", "--locked", "--offline", "--release"].map(str::to_owned),
                &core_root,
                &wasm_env,
                replacements,
                "PP5101",
            )?);
            let wasm = std::fs::read(
                core_root
                    .join("target/wasm32-unknown-unknown/release")
                    .join(format!(
                        "{}_core_wasm.wasm",
                        program.cargo_name.replace('-', "_")
                    )),
            )
            .map_err(|error| PrismError::new("PP5101", error.to_string()))?;
            artifacts.push(("binary/core.wasm".into(), wasm));
            artifacts.push((
                "binary/model-binding.json".to_owned(),
                crate::contracts::CanonicalDocument::from_value(
                    "prismpm/binary-build-binding/1",
                    json!({
                        "program": program,
                        "model_id": content_id(model_bytes), "profile": program.profile,
                        "schema": "prismpm/binary-build-binding/1", "scope": "binary-package-only"
                    }),
                )?
                .bytes()
                .to_vec(),
            ));
            artifacts.push((
                "binary/acceptance-runner.mjs".into(),
                include_bytes!("../binary_verification/runner.mjs").to_vec(),
            ));
            artifacts.push((
                "binary/memory-inspector.mjs".into(),
                include_bytes!("../binary_verification/memory-inspector.mjs").to_vec(),
            ));
            artifacts.sort_by(|left, right| left.0.cmp(&right.0));
            Ok(GeneratedBinary {
                artifacts,
                processes,
            })
        },
    )
}
