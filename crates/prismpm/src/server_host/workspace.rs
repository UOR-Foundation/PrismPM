//! The consumer's workspace as the declaration sees it.
//!
//! The binding crate is a member of the consumer's own Cargo workspace, at the
//! path the model declares. The declaration alone cannot say whether that
//! directory holds the declared crate, so this check reads the workspace:
//! the generated packages and the binding are exactly listed members, the
//! binding directory resolves inside the workspace, its manifest names the
//! declared crate and depends on the generated API package, and every other
//! dependency it has is a crates.io dependency or another workspace member.

use super::{api_library_name, api_name, program_name};
use crate::error::PrismError;
use crate::holo::server_application::ServerApplication;
use std::path::{Path, PathBuf};

fn mismatch(message: impl Into<String>) -> PrismError {
    PrismError::new("PP2012", message)
}

fn manifest(path: &Path) -> Result<toml::Table, PrismError> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| mismatch(format!("{}: {error}", path.display())))?;
    text.parse::<toml::Table>()
        .map_err(|error| mismatch(format!("{}: {error}", path.display())))
}

fn inside(root: &Path, path: &Path) -> Result<PathBuf, PrismError> {
    let resolved = path
        .canonicalize()
        .map_err(|error| mismatch(format!("{}: {error}", path.display())))?;
    if resolved.starts_with(root) {
        Ok(resolved)
    } else {
        Err(mismatch(format!(
            "{} resolves outside the workspace",
            path.display()
        )))
    }
}

/// Check the workspace at `workspace` against the declared binding.
///
/// # Errors
/// Returns `PP2012` when the workspace does not hold the declared binding as
/// described above.
pub fn verify(application: &ServerApplication, workspace: &Path) -> Result<(), PrismError> {
    let root = workspace
        .canonicalize()
        .map_err(|error| mismatch(format!("{}: {error}", workspace.display())))?;
    let members = manifest(&root.join("Cargo.toml"))?
        .get("workspace")
        .and_then(|table| table.get("members"))
        .and_then(toml::Value::as_array)
        .map(|members| {
            members
                .iter()
                .filter_map(|member| member.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .ok_or_else(|| mismatch("the workspace manifest lists no members"))?;
    let mut required = vec![api_name(application), program_name(application)];
    required.extend(application.bindings.iter().map(|binding| binding.path.clone()));
    for path in required {
        // Members are listed exactly: a glob could admit a directory nobody declared.
        if !members.contains(&path) {
            return Err(mismatch(format!(
                "the workspace does not list {path} as a member"
            )));
        }
    }
    for binding in &application.bindings {
        let directory = inside(&root, &root.join(&binding.path))?;
        let manifest = manifest(&directory.join("Cargo.toml"))?;
        let package = manifest
            .get("package")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| mismatch("the binding manifest has no package"))?;
        if package.get("name").and_then(toml::Value::as_str) != Some(binding.crate_name.as_str()) {
            return Err(mismatch(format!(
                "the crate at {} is not named {}",
                binding.path, binding.crate_name
            )));
        }
        if let Some(name) = manifest
            .get("lib")
            .and_then(|lib| lib.get("name"))
            .and_then(toml::Value::as_str)
        {
            if name != binding.crate_name.replace('-', "_") {
                return Err(mismatch(
                    "the binding library is not named after its crate",
                ));
            }
        }
        let api = api_name(application);
        let dependencies = manifest
            .get("dependencies")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| mismatch("the binding declares no dependencies"))?;
        if !dependencies.contains_key(&api) {
            return Err(mismatch(format!(
                "the binding does not depend on the generated {api} ({})",
                api_library_name(application)
            )));
        }
        for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
            let Some(table) = manifest.get(section).and_then(toml::Value::as_table) else {
                continue;
            };
            for (name, dependency) in table {
                let Some(dependency) = dependency.as_table() else {
                    continue;
                };
                if dependency.contains_key("git") || dependency.contains_key("registry") {
                    return Err(mismatch(format!(
                        "the binding dependency {name} is not a crates.io dependency"
                    )));
                }
                if let Some(path) = dependency.get("path").and_then(toml::Value::as_str) {
                    inside(&root, &directory.join(path)).map_err(|_| {
                        mismatch(format!(
                            "the binding dependency {name} is a path outside the workspace"
                        ))
                    })?;
                }
            }
        }
    }
    Ok(())
}
