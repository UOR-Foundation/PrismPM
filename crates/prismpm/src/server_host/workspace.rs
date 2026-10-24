//! The consumer's workspace as the declaration sees it.
//!
//! The binding crate is a member of the consumer's own Cargo workspace, at the
//! path the model declares. The declaration alone cannot say whether that
//! directory holds the declared crate, so this check reads the workspace:
//! the production core package, the generated packages and the binding are
//! exactly listed members, the
//! binding directory resolves inside the workspace, its manifest names the
//! declared crate and depends on the generated API package, and every other
//! dependency it has is a crates.io dependency or another workspace member.
//!
//! Verification builds that workspace away from the project: [`stage`] lays the
//! consumer's own files, the root manifest, its committed lock, a pinned
//! toolchain and every member the build does not generate, beside the files of
//! the build, so that what is built is what the project holds and the build
//! produced, and nothing the project's working tree happens to contain.

use super::{api_library_name, api_name, program_name};
use crate::error::PrismError;
use crate::holo::server_application::ServerApplication;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

/// Most bytes of the consumer's own files a verification stages.
pub const STAGED_MAXIMUM: u64 = 64 * 1024 * 1024;
/// The file the build records its compiler result in; it is not a workspace file.
const RESULT: &str = "production-result.json";

fn mismatch(message: impl Into<String>) -> PrismError {
    PrismError::new("PP2013", message)
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
/// Returns `PP2013` when the workspace does not hold the declared binding as
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
    let mut required = vec![
        application.core.package.clone(),
        api_name(application),
        program_name(application),
    ];
    required.extend(
        application
            .bindings
            .iter()
            .map(|binding| binding.path.clone()),
    );
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
                return Err(mismatch("the binding library is not named after its crate"));
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

fn confined(member: &str) -> bool {
    !member.is_empty()
        && !member.contains(['*', '?', '[', '\\'])
        && Path::new(member)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// Copy one regular file or a tree of them; a symlink or a special file is refused.
fn copy(from: &Path, to: &Path, total: &mut u64) -> Result<(), PrismError> {
    let metadata = std::fs::symlink_metadata(from)
        .map_err(|error| mismatch(format!("{}: {error}", from.display())))?;
    if metadata.is_dir() {
        std::fs::create_dir_all(to)
            .map_err(|error| mismatch(format!("{}: {error}", to.display())))?;
        let mut entries = std::fs::read_dir(from)
            .map_err(|error| mismatch(format!("{}: {error}", from.display())))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| mismatch(format!("{}: {error}", from.display())))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            // Cargo's own output is not the consumer's file.
            if entry.file_name() == "target" {
                continue;
            }
            copy(&entry.path(), &to.join(entry.file_name()), total)?;
        }
        return Ok(());
    }
    if !metadata.is_file() {
        return Err(mismatch(format!(
            "{} is neither a regular file nor a directory",
            from.display()
        )));
    }
    *total = total.saturating_add(metadata.len());
    if *total > STAGED_MAXIMUM {
        return Err(mismatch(
            "the consumer's workspace files exceed the staging bound",
        ));
    }
    std::fs::copy(from, to)
        .map(|_| ())
        .map_err(|error| mismatch(format!("{}: {error}", from.display())))
}

/// Lay the consumer's workspace out below `destination` for verification: the
/// project's `Cargo.toml`, its committed `Cargo.lock`, an optional
/// `rust-toolchain.toml`, every member the build does not generate, and the files
/// of the build under `server` (all but the compiler's record), then check the
/// result with [`verify`].
///
/// # Errors
/// Returns `PP2013` when the project holds no workspace manifest or no committed
/// lock, lists a member that is a glob or leaves the project, holds a symlink or a
/// special file in a member, exceeds [`STAGED_MAXIMUM`], or does not hold the
/// declared binding.
pub fn stage(
    application: &ServerApplication,
    project: &Path,
    server: &Path,
    destination: &Path,
) -> Result<(), PrismError> {
    let mut total = 0;
    for file in ["Cargo.toml", "Cargo.lock"] {
        if !project.join(file).is_file() {
            return Err(mismatch(format!(
                "the project holds no {file}: the consumer commits the workspace manifest and its lock"
            )));
        }
    }
    let members = manifest(&project.join("Cargo.toml"))?
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
    let generated = BTreeSet::from([
        application.core.package.clone(),
        api_name(application),
        program_name(application),
    ]);
    std::fs::create_dir_all(destination)
        .map_err(|error| mismatch(format!("{}: {error}", destination.display())))?;
    for file in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml"] {
        if project.join(file).exists() {
            copy(&project.join(file), &destination.join(file), &mut total)?;
        }
    }
    for member in members.iter().filter(|member| !generated.contains(*member)) {
        if !confined(member) {
            return Err(mismatch(format!(
                "the member {member} is not a plain path inside the workspace"
            )));
        }
        copy(&project.join(member), &destination.join(member), &mut total)?;
    }
    for entry in walkdir::WalkDir::new(server).follow_links(false) {
        let entry = entry.map_err(|error| mismatch(error.to_string()))?;
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(server)
            .map_err(|error| mismatch(error.to_string()))?;
        if relative == Path::new(RESULT) {
            continue;
        }
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| mismatch(format!("{}: {error}", parent.display())))?;
        }
        copy(entry.path(), &target, &mut total)?;
    }
    verify(application, destination)
}
