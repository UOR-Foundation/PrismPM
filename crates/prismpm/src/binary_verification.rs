//! Complete source/kernel audit, independent regeneration and real arbitrary-byte execution.
use crate::controller::VerifyResult;
use crate::error::PrismError;
use crate::holo::binary_program::BinaryProgram;
use crate::holo::canonical::{content_id, encode_value};
use crate::library_build::{sha256, write};
use crate::verification::{executable, run_process};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn acceptance_manifest(program: &BinaryProgram, enabled: bool) -> String {
    let features = if enabled {
        ", features = [\"std\"]"
    } else {
        ""
    };
    format!("[package]\nname = \"prism-binary-acceptance\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\nprism_binary_core = {{ package = \"{}\", version = \"={}\", default-features = false{} }}\n",program.cargo_name,program.cargo_version,features)
}
fn acceptance_harness(program: &BinaryProgram) -> Result<String, PrismError> {
    let entry = program
        .entry_root
        .rsplit('.')
        .next()
        .ok_or_else(|| PrismError::new("PP2001", "binary entry is absent"))?;
    let mut source=String::from("trait Computed { fn computed(self)->Vec<u8>; } impl Computed for Vec<u8> {fn computed(self)->Vec<u8>{self}} impl<E:core::fmt::Debug> Computed for Result<Vec<u8>,E>{fn computed(self)->Vec<u8>{self.expect(\"core computation failed\")}} fn main(){ let mut results=Vec::new();\n");
    for vector in &program.acceptance_vectors {
        source.push_str(&format!("let actual=Computed::computed(prism_binary_core::{entry}(vec!{:?})); assert_eq!(actual,vec!{:?}); results.push(actual);\n",vector.request,vector.response));
    }
    source.push_str("print!(\"[\"); for (index,bytes) in results.iter().enumerate(){if index>0{print!(\",\");} print!(\"[\");for (at,byte) in bytes.iter().enumerate(){if at>0{print!(\",\");} print!(\"{}\",byte);}print!(\"]\");}println!(\"]\");}\n");
    Ok(source)
}

pub(crate) fn run(
    mut context: crate::library_verification::LibraryVerification<'_>,
) -> Result<VerifyResult, PrismError> {
    let program = context.model.program.as_ref().ok_or_else(|| {
        PrismError::new("PP9001", "binary verification requires a program profile")
    })?;
    let attestation: Value = serde_json::from_slice(&context.lex_attestation)
        .map_err(|error| PrismError::new("PP4004", error.to_string()))?;
    crate::library_verification::audit_declarations(&attestation, &context.lex_snapshot)?;
    crate::verification::verify_application_build_closure(
        context.build_root,
        &context.build_manifest,
    )?;
    let lex_root = context.build_root.join("lexlean/build");
    let lex_manifest = std::fs::read(lex_root.join("manifest.json"))
        .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
    // The controller build is export A. This independently exports and packages
    // the attested Lean closure as export B; both complete byte sets must agree.
    let replay = crate::binary_build::generate_recorded(
        context.repository_root,
        &context.model,
        &context.model_bytes,
        &lex_root,
        &lex_manifest,
    )?;
    let expected_paths = context.build_manifest["files"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP4004", "binary build manifest files are absent"))?
        .iter()
        .filter_map(|row| row["path"].as_str())
        .filter(|path| path.starts_with("binary/"))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let replay_paths = replay
        .artifacts
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    if expected_paths != replay_paths {
        return Err(PrismError::new(
            "PP5004",
            "binary regeneration artifact closure differs",
        ));
    }
    for (path, bytes) in &replay.artifacts {
        let observed = std::fs::read(context.build_root.join(path))
            .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
        if observed != *bytes {
            return Err(PrismError::new(
                "PP5004",
                format!("binary regeneration differs: {path}"),
            ));
        }
    }
    context.processes.extend(replay.processes);
    let work = tempfile::Builder::new()
        .prefix("prismpm-binary-verify-")
        .tempdir()
        .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
    let archive_name = format!("{}-{}.crate", program.cargo_name, program.cargo_version);
    let archive = &replay
        .artifacts
        .iter()
        .find(|(path, _)| path == &format!("binary/{archive_name}"))
        .ok_or_else(|| PrismError::new("PP4102", "generated library archive is absent"))?
        .1;
    let cargo_home = crate::library_verification::registry_cargo_home(
        work.path(),
        &program.metadata(),
        archive,
    )?;
    let environment = BTreeMap::from([
        (
            "CARGO_HOME".to_owned(),
            cargo_home.to_string_lossy().into_owned(),
        ),
        ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
        (
            "RUSTFLAGS".to_owned(),
            format!(
                "--remap-path-prefix={}=$BINARY_VERIFY",
                work.path().display()
            ),
        ),
    ]);
    let replacements = [
        (work.path(), "$BINARY_VERIFY"),
        (context.repository_root, "$PROJECT"),
        (context.build_root, "$BUILD"),
    ];
    let cargo = executable("cargo")?;
    let mut executions = Vec::new();
    for (mode, enabled) in [("std", true), ("no_std", false)] {
        let consumer = work.path().join(format!("consumer-{mode}"));
        write(
            &consumer.join("Cargo.toml"),
            acceptance_manifest(program, enabled).as_bytes(),
        )?;
        write(
            &consumer.join("src/main.rs"),
            acceptance_harness(program)?.as_bytes(),
        )?;
        context.processes.push(run_process(
            &format!("binary-{mode}-lock"),
            &cargo,
            &["generate-lockfile", "--offline"].map(str::to_owned),
            &consumer,
            &environment,
            &replacements,
            "PP4102",
        )?);
        let record = run_process(
            &format!("binary-{mode}-acceptance"),
            &cargo,
            &["run", "--locked", "--offline", "--quiet"].map(str::to_owned),
            &consumer,
            &environment,
            &replacements,
            "PP5006",
        )?;
        let expected = serde_json::to_string(
            &program
                .acceptance_vectors
                .iter()
                .map(|v| &v.response)
                .collect::<Vec<_>>(),
        )
        .map_err(|error| PrismError::new("PP9001", error.to_string()))?;
        if record.stdout != format!("{expected}\n") || !record.stderr.is_empty() {
            return Err(PrismError::new(
                "PP5006",
                "binary native transcript differs from source-owned exact bytes",
            ));
        }
        executions.push(
            json!({"mode":mode,"vector_count":program.acceptance_vectors.len(),"status":"passed"}),
        );
        context.processes.push(record);
    }
    let cli_root = work.path().join("cli");
    for (path, bytes) in &replay.artifacts {
        if let Some(relative) = path.strip_prefix("binary/cli/") {
            write(&cli_root.join(relative), bytes)?;
        }
    }
    let allocation = run_process(
        "binary-cli-allocation-acceptance",
        &cargo,
        &[
            "test",
            "--locked",
            "--offline",
            "--release",
            "--",
            "--exact",
            "tests::adapter_allocation_maps_real_capacity_overflow",
        ]
        .map(str::to_owned),
        &cli_root,
        &environment,
        &replacements,
        "PP5006",
    )?;
    if !allocation
        .stdout
        .contains("test tests::adapter_allocation_maps_real_capacity_overflow ... ok")
        || !allocation
            .stdout
            .contains("1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out")
    {
        return Err(PrismError::new(
            "PP5006",
            "generated adapter allocation negative did not execute exactly once",
        ));
    }
    context.processes.push(allocation);
    context.processes.push(run_process(
        "binary-cli-acceptance-build",
        &cargo,
        &["build", "--locked", "--offline", "--release"].map(str::to_owned),
        &cli_root,
        &environment,
        &replacements,
        "PP4102",
    )?);
    let runner = work.path().join("runner.mjs");
    write(&runner, include_bytes!("binary_verification/runner.mjs"))?;
    write(
        &work.path().join("memory-inspector.mjs"),
        include_bytes!("binary_verification/memory-inspector.mjs"),
    )?;
    let program_value = serde_json::to_value(program)
        .map_err(|error| PrismError::new("PP9001", error.to_string()))?;
    write(
        &work.path().join("program.json"),
        &encode_value(&program_value)?,
    )?;
    let cli_binary = cli_root
        .join("target/release")
        .join(format!("{}-cli", program.cargo_name));
    let record = run_process(
        "binary-transports-acceptance",
        &executable("node")?,
        &[
            runner.to_string_lossy().into_owned(),
            work.path()
                .join("program.json")
                .to_string_lossy()
                .into_owned(),
            context
                .build_root
                .join("binary/core.wasm")
                .to_string_lossy()
                .into_owned(),
            cli_binary.to_string_lossy().into_owned(),
            work.path()
                .join("transports")
                .to_string_lossy()
                .into_owned(),
        ],
        work.path(),
        &BTreeMap::new(),
        &replacements,
        "PP5006",
    )?;
    let observed: Value = serde_json::from_str(&record.stdout)
        .map_err(|error| PrismError::new("PP5006", error.to_string()))?;
    let expected = ["core-wasm","cli-stdio","cli-file","cli-mixed"].iter().map(|mode|
        json!({"mode":mode,"vector_count":program.acceptance_vectors.len(),"status":"passed"})).collect::<Vec<_>>();
    let io_coverage = json!({"platform":"linux","output_write":if program.acceptance_vectors.iter().any(|vector|!vector.response.is_empty()) {"passed"} else {"not-exercised-empty-responses"}});
    if observed != json!({"executions":expected,"io_coverage":io_coverage})
        || !record.stderr.is_empty()
    {
        return Err(PrismError::new(
            "PP5006",
            "binary transport transcript or scoped I/O coverage differs",
        ));
    }
    executions.extend(expected);
    context.processes.push(record);
    let bindings = replay
        .artifacts
        .iter()
        .map(|(path, bytes)| {
            json!({
                "byte_length": bytes.len(), "path":path, "sha256":sha256(bytes),
            })
        })
        .collect::<Vec<_>>();
    let acceptance = crate::contracts::CanonicalDocument::from_value(
        "prismpm/binary-acceptance/1",
        json!({
            "build_id":context.build.build_id,
            "executions":executions,
            "io_coverage":io_coverage,
            "program":program,
            "lexlean_attestation_id":context.lex_attestation_id,
            "model_id":content_id(&context.model_bytes),
            "profile":program.profile,
            "regeneration":"byte-identical",
            "schema":"prismpm/binary-acceptance/1",
            "scope":"binary-package-only",
            "status":"passed",
            "unclaimed":["application","browser","holo","production-release","deployment"]
        }),
    )?
    .bytes()
    .to_vec();
    let manifest = crate::contracts::CanonicalDocument::from_value(
        "prismpm/binary-verification-manifest/1",
        json!({
            "acceptance_sha256":sha256(&acceptance),
            "artifacts":bindings,
            "build_id":context.build.build_id,
            "lexlean_attestation_sha256":sha256(&context.lex_attestation),
            "model_sha256":sha256(&context.model_bytes),
            "processes":context.processes,
            "schema":"prismpm/binary-verification-manifest/1",
            "scope":"binary-package-only"
        }),
    )?
    .bytes()
    .to_vec();
    let attestation_id = content_id(&manifest);
    let files = vec![
        ("binary-acceptance.json".to_owned(), acceptance),
        (
            "lexlean-attestation.json".to_owned(),
            context.lex_attestation,
        ),
        ("manifest.json".to_owned(), manifest),
    ];
    crate::verification::publish(
        &context.config.output_root(context.repository_root)?,
        &attestation_id,
        &files,
    )?;
    Ok(VerifyResult {
        schema: "prismpm/verify-result/1".to_owned(),
        attestation_id: attestation_id.clone(),
        build_id: context.build.build_id,
        verified_root: format!("{}/verified/{attestation_id}", context.config.build_root),
    })
}
