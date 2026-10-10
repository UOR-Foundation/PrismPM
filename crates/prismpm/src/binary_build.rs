//! Source-bound native, Core-Wasm and mandatory raw CLI generation for BinaryProgram.
use crate::error::PrismError;
use crate::holo::canonical::content_id;
use crate::holo::model_document::ModelDocument;
use crate::library_build::{regular_bytes, sha256, validate_export_identity, write, relative};
use crate::verification::{executable, run_process, ProcessRecord};
use prod_codegen::{generate_cargo_package, generate_core_wasm_package, CargoPackageSpec, CoreWasmSpec};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;
use std::path::Path;
const LEAN4_PROD_ARCHIVE: &[u8] = include_bytes!("../vendor/lean4-prod/lean.tar");
pub(crate) type BinaryArtifact = (String, Vec<u8>);
pub(crate) struct GeneratedBinary { pub(crate) artifacts: Vec<BinaryArtifact>, pub(crate) processes: Vec<ProcessRecord> }

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
    let program = model
        .program
        .as_ref()
        .ok_or_else(|| PrismError::new("PP9001", "binary generation requires a program profile"))?;
    let work = tempfile::Builder::new()
        .prefix("prismpm-binary-program-")
        .tempdir()
        .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
    let workspace = work.path();
    let lean_package = workspace.join("lean4-prod");
    std::fs::create_dir(&lean_package)
        .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
    tar::Archive::new(Cursor::new(LEAN4_PROD_ARCHIVE))
        .unpack(&lean_package)
        .map_err(|error| PrismError::new("PP5008", format!("pinned lean4-prod: {error}")))?;

    let manifest: Value = serde_json::from_slice(lex_manifest_bytes)
        .map_err(|error| PrismError::new("PP4004", format!("LexLean manifest: {error}")))?;
    let outputs = manifest["outputs"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP4004", "LexLean output manifest is absent"))?;
    let mut modules = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for output in outputs {
        if output["kind"] != "lean" {
            continue;
        }
        let path = output["path"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP4004", "generated Lean path is absent"))?;
        relative(path)?;
        if !paths.insert(path) {
            return Err(PrismError::new(
                "PP4004",
                "generated Lean path is duplicated",
            ));
        }
        let local = Path::new(path)
            .strip_prefix("modules")
            .map_err(|_| PrismError::new("PP4004", "generated Lean is outside modules"))?;
        if local.extension().and_then(|extension| extension.to_str()) != Some("lean") {
            return Err(PrismError::new(
                "PP4004",
                "generated Lean extension differs",
            ));
        }
        let bytes = regular_bytes(lex_root, path)?;
        if output["sha256"].as_str() != Some(sha256(&bytes).as_str())
            || output["byte_length"].as_u64() != Some(bytes.len() as u64)
        {
            return Err(PrismError::new(
                "PP4001",
                "generated Lean differs from its manifest",
            ));
        }
        write(&workspace.join(local), &bytes)?;
        modules.insert(local.with_extension("").to_string_lossy().replace('/', "."));
    }
    if modules.is_empty() {
        return Err(PrismError::new(
            "PP4004",
            "library has no generated Lean modules",
        ));
    }
    write(
        &workspace.join("lakefile.toml"),
        format!(
            "name = \"prismpm_binary_program\"\nversion = \"0.1.0\"\n\n[[lean_lib]]\nname = \"PrismGenerated\"\nroots = [{}]\n",
            modules.iter().map(|module| serde_json::to_string(module).expect("module string")).collect::<Vec<_>>().join(", ")
        ).as_bytes(),
    )?;
    write(
        &workspace.join("lean-toolchain"),
        b"leanprover/lean4:v4.32.1\n",
    )?;
    let lake = executable("lake")?;
    let replacements = [(workspace, "$BINARY_WORK"), (repository_root, "$PROJECT")];
    let no_env = BTreeMap::new();
    let mut processes = Vec::new();
    for (tool, directory, args) in [
        (
            "lake-build-generated",
            workspace,
            vec!["build".to_owned(), "PrismGenerated".to_owned()],
        ),
        (
            "lean4-prod-build",
            lean_package.as_path(),
            vec!["build".to_owned(), "prod-export".to_owned()],
        ),
    ] {
        processes.push(run_process(
            tool,
            &lake,
            &args,
            directory,
            &no_env,
            &replacements,
            "PP5004",
        )?);
    }
    let export = workspace.join("export");
    let mut arguments = vec!["exe".to_owned(), "prod-export".to_owned()];
    for module in &modules {
        arguments.extend(["--module".to_owned(), module.clone()]);
    }
    for root in &program.export_roots {
        arguments.extend(["--root".to_owned(), root.clone()]);
    }
    arguments.extend([
        "--ir-module".to_owned(),
        program.cargo_name.replace('-', "_"),
        "--out".to_owned(),
        export.to_string_lossy().into_owned(),
    ]);
    let export_env = BTreeMap::from([(
        "LEAN_PATH".to_owned(),
        workspace
            .join(".lake/build/lib/lean")
            .to_string_lossy()
            .into_owned(),
    )]);
    processes.push(run_process(
        "prod-export",
        &lake,
        &arguments,
        &lean_package,
        &export_env,
        &replacements,
        "PP5004",
    )?);
    let mut artifacts = Vec::new();
    for name in ["coverage.json", "kernel.ir", "roots.json"] {
        let bytes = std::fs::read(export.join(name))
            .map_err(|error| PrismError::new("PP5004", format!("{name}: {error}")))?;
        artifacts.push((format!("binary/{name}"), bytes));
    }
    let kernel = artifacts
        .iter()
        .find(|(path, _)| path == "binary/kernel.ir")
        .expect("kernel recorded")
        .1.clone();
    let text = std::str::from_utf8(&kernel)
        .map_err(|error| PrismError::new("PP5004", error.to_string()))?;
    let module = crate::verification::parse_kernel(text)?;
    let document = |path: &str| -> Result<Value, PrismError> {
        serde_json::from_slice(
            &artifacts
                .iter()
                .find(|(name, _)| name == path)
                .expect("export recorded")
                .1,
        )
        .map_err(|error| PrismError::new("PP5004", error.to_string()))
    };
    validate_export_identity(
        &document("binary/coverage.json")?,
        &document("binary/roots.json")?,
        &module,
        &program.export_roots,
        &[],
    )?;
    let entry = program.entry_root.rsplit('.').next().ok_or_else(|| PrismError::new("PP2001", "binary entry is absent"))?;
    let entry_definition = module.find_def(entry).ok_or_else(|| PrismError::new("PP5004", "binary entry is absent from IR"))?;
    if entry_definition.params.len() != 1 || entry_definition.params[0].1 != prod_ir::Type::Bytes || entry_definition.ret != prod_ir::Type::Bytes {
        return Err(PrismError::new("PP5004", "binary entry IR signature must be Bytes to Bytes"));
    }
    let package = generate_cargo_package(
        &module,
        &CargoPackageSpec {
            name: program.cargo_name.clone(),
            version: program.cargo_version.clone(),
            description: program.cargo_description.clone(),
            repository: program.cargo_repository.clone(),
            homepage: program.cargo_homepage.clone(),
            readme: format!("# {}\n\n{}\n", program.name, program.cargo_description),
            license_mit: include_str!("../LICENSE-MIT").to_owned(),
            license_apache: include_str!("../LICENSE-APACHE").to_owned(),
            input_sha256: sha256(&kernel),
            dependencies: Vec::new(),
        },
    )
    .map_err(|error| PrismError::new("PP4102", error.to_string()))?;
    let package_root = workspace.join("package");
    for file in package.files {
        relative(&file.path)?;
        write(&package_root.join(&file.path), &file.bytes)?;
        artifacts.push((format!("binary/package/{}", file.path), file.bytes));
    }
    let cargo_home = workspace.join("cargo-home");
    std::fs::create_dir(&cargo_home)
        .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
    let cargo_env = BTreeMap::from([
        (
            "CARGO_HOME".to_owned(),
            cargo_home.to_string_lossy().into_owned(),
        ),
        ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
    ]);
    processes.push(run_process(
        "binary-program-package",
        &executable("cargo")?,
        &["package", "--locked", "--offline", "--allow-dirty"].map(str::to_owned),
        &package_root,
        &cargo_env,
        &replacements,
        "PP4102",
    )?);
    let archive_name = format!("{}-{}.crate", program.cargo_name, program.cargo_version);
    let crate_bytes = std::fs::read(package_root.join("target/package").join(&archive_name))
        .map_err(|error| PrismError::new("PP4102", format!("binary program archive: {error}")))?;
    let core_archive = crate_bytes.clone();
    artifacts.push((format!("binary/{archive_name}"), crate_bytes));
    // The CLI consumes the exact generated core archive through an isolated offline registry.
    let consumer = workspace.join("consumer");
    std::fs::create_dir(&consumer).map_err(|error| PrismError::new("PP4002",error.to_string()))?;
    let consumer_home = crate::library_verification::registry_cargo_home(&consumer, &program.metadata(), &core_archive)?;
    let cargo_env = BTreeMap::from([
        ("CARGO_HOME".to_owned(), consumer_home.to_string_lossy().into_owned()),
        ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
        ("RUSTFLAGS".to_owned(), format!("--remap-path-prefix={}=$BINARY_WORK", workspace.display())),
    ]);
    let cli = crate::binary_cli::generate(program)?;
    let cli_root = workspace.join("cli");
    for file in cli.files {
        relative(&file.path)?; write(&cli_root.join(&file.path), &file.bytes)?;
        artifacts.push((format!("binary/cli/{}",file.path), file.bytes));
    }
    for (tool,args) in [
        ("binary-cli-lock", vec!["generate-lockfile","--offline"]),
        ("binary-cli-build", vec!["build","--locked","--offline","--release"]),
        ("binary-cli-package", vec!["package","--locked","--offline","--allow-dirty"]),
    ] {
        processes.push(run_process(tool,&executable("cargo")?,&args.into_iter().map(str::to_owned).collect::<Vec<_>>(),&cli_root,&cargo_env,&replacements,"PP4102")?);
    }
    artifacts.push(("binary/cli/Cargo.lock".into(),std::fs::read(cli_root.join("Cargo.lock")).map_err(|error| PrismError::new("PP4102",error.to_string()))?));
    let cli_archive = format!("{}-cli-{}.crate",program.cargo_name,program.cargo_version);
    artifacts.push((format!("binary/{cli_archive}"), std::fs::read(cli_root.join("target/package").join(cli_archive)).map_err(|error| PrismError::new("PP4102",error.to_string()))?));
    // A generated executable is platform-specific. Record reproducible source and exact lock,
    // and build/run the adapter independently during acceptance instead of publishing a host ELF.
    let core = generate_core_wasm_package(&module,&CoreWasmSpec {
        crate_name:format!("{}-core-wasm",program.cargo_name),entry:entry.to_owned(),export_name:"binary_run".into(),
        input_allocation_cap:program.request_maximum,output_allocation_cap:program.response_maximum,
        maximum_pages:program.memory_pages,input_ir_sha256:sha256(&kernel),
    }).map_err(|error| PrismError::new("PP5101",error.to_string()))?;
    let core_root = workspace.join("core-wasm");
    for file in core.files {
        relative(&file.path)?; write(&core_root.join(&file.path), &file.bytes)?;
        artifacts.push((format!("binary/core-wasm/{}",file.path),file.bytes));
    }
    // Do not override generated target rustflags: they bind exported memory and maximum pages.
    let wasm_env = BTreeMap::from([
        ("CARGO_HOME".to_owned(),consumer_home.to_string_lossy().into_owned()),
        ("CARGO_NET_OFFLINE".to_owned(),"true".to_owned()),
    ]);
    processes.push(run_process("binary-core-wasm-build",&executable("cargo")?,
        &["build","--locked","--offline","--release"].map(str::to_owned),&core_root,&wasm_env,&replacements,"PP5101")?);
    let wasm = std::fs::read(core_root.join("target/wasm32-unknown-unknown/release").join(format!("{}_core_wasm.wasm",program.cargo_name.replace('-',"_"))))
        .map_err(|error| PrismError::new("PP5101",error.to_string()))?;
    artifacts.push(("binary/core.wasm".into(),wasm));
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
    artifacts.push(("binary/acceptance-runner.mjs".into(),include_bytes!("binary_verification/runner.mjs").to_vec()));
    artifacts.push(("binary/memory-inspector.mjs".into(),include_bytes!("binary_verification/memory-inspector.mjs").to_vec()));
    artifacts.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(GeneratedBinary {
        artifacts,
        processes,
    })
}
