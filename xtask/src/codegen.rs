//! The model validation and code generation gate.

use crate::Fail;
use repo_model::{codegen, Model};
use std::path::{Path, PathBuf};

/// Validate that generated documents equal the model registers.
pub fn check_model(root: &Path, write: bool) -> Result<(), Fail> {
    let model = Model::load(&root.join("model"))?;
    model.check()?;

    let conformance = codegen::render_conformance(&model);
    let errors = codegen::render_errors(&model);
    let spec = std::fs::read_to_string(root.join("SPEC.md"))?;
    let (spec_body, _) = spec
        .split_once("## Appendix A. Conformance ID Registry")
        .ok_or("SPEC.md has no conformance registry appendix")?;
    let spec = format!("{}{}", spec_body, codegen::render_spec_appendix(&model));
    let conformance_path: PathBuf = root.join(codegen::CONFORMANCE_PATH);
    let errors_path: PathBuf = root.join(codegen::ERRORS_PATH);
    let spec_path: PathBuf = root.join("SPEC.md");

    if write {
        std::fs::write(&conformance_path, &conformance)?;
        std::fs::write(&errors_path, &errors)?;
        std::fs::write(&spec_path, &spec)?;
        println!(
            "wrote {}, {}, and {}",
            conformance_path.display(),
            errors_path.display(),
            spec_path.display()
        );
        return Ok(());
    }

    for (path, generated) in [(&conformance_path, &conformance), (&errors_path, &errors)] {
        let committed = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}\nrun `just model-write`", path.display()))?;
        if committed != **generated {
            return Err(format!(
                "{} is stale: it disagrees with model/*.toml (R1). Run `just model-write`.",
                path.display()
            )
            .into());
        }
    }
    let committed_spec = std::fs::read_to_string(&spec_path)?;
    if committed_spec != spec {
        return Err(format!(
            "{} has a stale conformance appendix; run `just codegen`.",
            spec_path.display()
        )
        .into());
    }

    let tests = repo_conformance::workspace_test_names(root);
    let report = repo_conformance::check_honesty(root, &tests)?;
    if !report.is_clean() {
        return Err(format!(
            "the honesty meta-gate failed inside validate-model:\n\n{}",
            report.violations.join("\n\n")
        )
        .into());
    }

    crate::audit::audit_no_handwritten_lean(root)?;
    crate::audit::audit_no_unsafe(root)?;

    println!(
        "validate-model: documents current, {} ids, {} codes, meta-gate and audits clean (R1)",
        model.ids.id.len(),
        model.errors.error.len()
    );
    Ok(())
}
