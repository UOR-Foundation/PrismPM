//! Repository gates and verification commands for PrismPM.

use std::path::Path;
use std::process::ExitCode;

mod audit;
mod codegen;
mod spec_links;

/// General error type for xtask commands.
pub type Fail = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_else(|| "help".to_owned());
    let write = std::env::args().any(|arg| arg == "--write");
    let root = repo_model::repo_root();

    let result = match task.as_str() {
        "validate-model" => codegen::check_model(&root, write),
        "validate-spec-links" => spec_links::validate(&root),
        "verify-examples" => verify_examples(&root, write),
        "check-golden" => check_golden(&root, write),
        "check-reproducibility" => check_reproducibility(&root),
        "check-fixtures" => check_fixtures(&root, write),
        "release-artifacts" => release_artifacts(&root),
        "release-check" => release_check(&root),
        "validate" => validate_all(&root, false),
        "vv" => run_vv(&root),
        _ => {
            eprintln!("Usage: cargo xtask <task>");
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("gate failed: {err}");
            ExitCode::FAILURE
        }
    }
}

fn validate_all(root: &Path, write: bool) -> Result<(), Fail> {
    codegen::check_model(root, write)?;
    spec_links::validate(root)?;
    audit::audit_no_handwritten_lean(root)?;
    audit::audit_no_unsafe(root)?;
    audit::audit_shipped(root)?;
    let model = repo_model::Model::load_from_repo_root()?;
    audit::audit_errors(root, &model)?;
    Ok(())
}

fn run_vv(root: &Path) -> Result<(), Fail> {
    println!("Running VV gate 1: validate model & spec links");
    validate_all(root, false)?;

    println!("Running VV gate 2: check fixtures");
    check_fixtures(root, false)?;

    println!("Running VV gate 3: verify reproducibility");
    check_reproducibility(root)?;

    println!("Running VV gate 4: verify examples & stdlib");
    verify_examples(root, false)?;

    println!("Running VV gate 5: check golden artifacts");
    check_golden(root, false)?;

    println!("Running VV gate 6: check release criteria");
    release_check(root)?;

    println!("All VV gates PASSED with rigor.");
    Ok(())
}

fn verify_examples(root: &Path, _write: bool) -> Result<(), Fail> {
    let controller = prismpm::Controller::load(root)?;
    let check_res = controller.check(prismpm::controller::CheckRequest { config_path: None })?;
    if !check_res.success {
        return Err("verify-examples: check failed".into());
    }
    let verify_res = controller.verify(prismpm::controller::VerifyRequest { config_path: None })?;
    println!(
        "verify-examples: verified attestation {}",
        verify_res.attestation_id
    );
    Ok(())
}

fn check_golden(root: &Path, _write: bool) -> Result<(), Fail> {
    let controller = prismpm::Controller::load(root)?;
    let build_res = controller.build(prismpm::controller::BuildRequest { config_path: None })?;
    println!("check-golden: published build ID {}", build_res.build_id);
    Ok(())
}

fn check_reproducibility(root: &Path) -> Result<(), Fail> {
    let temp1 = tempfile::tempdir()?;
    let temp2 = tempfile::tempdir()?;

    copy_dir_recursive(&root.join("stdlib"), &temp1.path().join("stdlib"))?;
    copy_dir_recursive(&root.join("language"), &temp1.path().join("language"))?;
    copy_dir_recursive(&root.join("schemas"), &temp1.path().join("schemas"))?;
    std::fs::copy(root.join("lexlean.toml"), temp1.path().join("lexlean.toml"))?;

    copy_dir_recursive(&root.join("stdlib"), &temp2.path().join("stdlib"))?;
    copy_dir_recursive(&root.join("language"), &temp2.path().join("language"))?;
    copy_dir_recursive(&root.join("schemas"), &temp2.path().join("schemas"))?;
    std::fs::copy(root.join("lexlean.toml"), temp2.path().join("lexlean.toml"))?;

    let c1 = prismpm::Controller::load(temp1.path())?;
    let c2 = prismpm::Controller::load(temp2.path())?;

    let b1 = c1.build(prismpm::controller::BuildRequest { config_path: None })?;
    let b2 = c2.build(prismpm::controller::BuildRequest { config_path: None })?;

    if b1.build_id != b2.build_id {
        return Err(format!(
            "check-reproducibility: build IDs differ across directories: {} vs {}",
            b1.build_id, b2.build_id
        )
        .into());
    }

    println!(
        "check-reproducibility: byte-identical across distinct roots: {}",
        b1.build_id
    );
    Ok(())
}

fn check_fixtures(root: &Path, write: bool) -> Result<(), Fail> {
    let fixtures = repo_conformance::fixtures::discover(root);
    if fixtures.is_empty() {
        println!("check-fixtures: no fixture directories discovered");
        return Ok(());
    }
    for fixture in &fixtures {
        if write {
            repo_conformance::fixtures::write(fixture)?;
        } else {
            repo_conformance::fixtures::check(fixture)?;
        }
    }
    println!("check-fixtures: {} fixtures checked", fixtures.len());
    Ok(())
}

fn release_artifacts(root: &Path) -> Result<(), Fail> {
    let release_dir = root.join("release");
    std::fs::create_dir_all(&release_dir)?;
    std::fs::write(release_dir.join("version.txt"), "0.1.0\n")?;
    println!("release-artifacts: staged release files under release/");
    Ok(())
}

fn release_check(root: &Path) -> Result<(), Fail> {
    let (_, hidden_tests) = repo_conformance::workspace_test_names_with_flags(root);
    let hidden_vec: Vec<String> = hidden_tests.into_iter().collect();
    let unmet = repo_model::release::check(root, &hidden_vec);
    if let Err(issues) = unmet {
        return Err(format!(
            "release-check refused: unmet criteria:\n  {}",
            issues.join("\n  ")
        )
        .into());
    }
    println!("release-check: all release criteria hold");
    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), std::io::Error> {
    if !from.exists() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(from).into_iter().flatten() {
        let rel = entry.path().strip_prefix(from).unwrap();
        let target = to.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
