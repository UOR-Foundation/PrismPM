//! Repository audits for PrismPM.

use crate::Fail;
use repo_model::Model;
use std::collections::BTreeSet;
use std::path::Path;

/// Audit that no handwritten .lean or lakefile.lean exists in source paths.
pub fn audit_no_handwritten_lean(root: &Path) -> Result<(), Fail> {
    for entry in walkdir::WalkDir::new(root).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(path).to_string_lossy();
        if rel.starts_with(".prism")
            || rel.starts_with(".lexlean")
            || rel.starts_with("target")
            || rel.contains("expected")
        {
            continue;
        }
        if rel.ends_with(".lean") {
            return Err(
                format!("no-handwritten-lean audit failed: found {}", path.display()).into(),
            );
        }
        if rel.ends_with("lakefile.lean") {
            return Err(
                format!("no-handwritten-lean audit failed: found {}", path.display()).into(),
            );
        }
    }
    Ok(())
}

/// Audit that unsafe code is forbidden across workspace Rust crates.
pub fn audit_no_unsafe(root: &Path) -> Result<(), Fail> {
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let text = std::fs::read_to_string(path)?;
            if text.contains("unsafe ") && !path.to_string_lossy().contains("tests") {
                return Err(
                    format!("unsafe audit failed: found unsafe in {}", path.display()).into(),
                );
            }
        }
    }
    Ok(())
}

/// Audit that only crates/prismpm is publishable and repo crates are private.
pub fn audit_shipped(root: &Path) -> Result<(), Fail> {
    let internal_crates = ["crates/model", "crates/conformance", "xtask"];
    for cr in internal_crates {
        let manifest_path = root.join(cr).join("Cargo.toml");
        let content = std::fs::read_to_string(&manifest_path)?;
        if !content.contains("publish = false") {
            return Err(format!(
                "{}: internal crate must have publish = false",
                manifest_path.display()
            )
            .into());
        }
    }
    Ok(())
}

/// Audit that every diagnostic code used in crates/ is registered in model/errors.toml.
pub fn audit_errors(root: &Path, model: &Model) -> Result<(), Fail> {
    let registered: BTreeSet<&str> = model.errors.error.iter().map(|e| e.code.as_str()).collect();
    for entry in walkdir::WalkDir::new(root.join("crates"))
        .into_iter()
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let text = std::fs::read_to_string(path)?;
            for word in text.split(|c: char| !c.is_alphanumeric()) {
                if word.starts_with("PP")
                    && word.len() == 6
                    && word.chars().skip(2).all(|c| c.is_ascii_digit())
                    && !registered.contains(word)
                {
                    return Err(format!(
                        "{}: references unregistered error code `{word}`",
                        path.display()
                    )
                    .into());
                }
            }
        }
    }
    Ok(())
}
