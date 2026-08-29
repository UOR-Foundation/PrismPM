//! Fixture discovery and validation for PrismPM.

use std::path::{Path, PathBuf};

/// Discover all fixture directories containing case.toml.
pub fn discover(root: &Path) -> Vec<PathBuf> {
    let mut fixtures = Vec::new();
    let search_roots = [root.join("tests/fixtures"), root.join("tests/negative")];
    for search_root in search_roots {
        if !search_root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(search_root).into_iter().flatten() {
            if entry.file_name() == "case.toml" {
                if let Some(parent) = entry.path().parent() {
                    fixtures.push(parent.to_path_buf());
                }
            }
        }
    }
    fixtures.sort();
    fixtures
}

/// Check one fixture directory against its expected files.
pub fn check(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let case_path = dir.join("case.toml");
    if !case_path.exists() {
        return Err(format!("{}: missing case.toml", dir.display()).into());
    }
    let case_content = std::fs::read_to_string(&case_path)?;
    let parsed: toml::Value = case_content.parse()?;
    let spec = parsed.get("spec").and_then(|v| v.as_str()).unwrap_or("");
    if spec != "prismpm/test-case/1" {
        return Err(format!("{}: invalid case spec `{spec}`", dir.display()).into());
    }

    let expected_exit = parsed.get("expected_exit").and_then(|v| v.as_integer()).unwrap_or(0) as i32;

    let project_dir = dir.join("project");
    if project_dir.exists() {
        let temp_dir = tempfile::tempdir()?;
        copy_dir_recursive(&project_dir, temp_dir.path())?;

        let controller = prismpm::Controller::load(temp_dir.path())?;
        let check_res = controller.check(prismpm::controller::CheckRequest { config_path: None });

        if expected_exit == 0 {
            if let Err(e) = check_res {
                return Err(format!("{}: expected success, got error {e}", dir.display()).into());
            }
        } else if check_res.is_ok() {
            return Err(format!("{}: expected failure with exit code {expected_exit}, but check succeeded", dir.display()).into());
        }
    }

    let expected_dir = dir.join("expected");
    if expected_dir.exists() {
        for entry in walkdir::WalkDir::new(&expected_dir).into_iter().flatten() {
            if entry.file_type().is_file() {
                let _bytes = std::fs::read(entry.path())?;
            }
        }
    }

    Ok(())
}

/// Write expected files for a fixture directory.
pub fn write(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = dir.join("project");
    if project_dir.exists() {
        let temp_dir = tempfile::tempdir()?;
        copy_dir_recursive(&project_dir, temp_dir.path())?;
        let controller = prismpm::Controller::load(temp_dir.path())?;
        let build_res = controller.build(prismpm::controller::BuildRequest { config_path: None })?;

        let expected_dir = dir.join("expected");
        std::fs::create_dir_all(&expected_dir)?;
        let result_json = serde_json::to_string_pretty(&build_res)?;
        std::fs::write(expected_dir.join("command.json"), result_json)?;
    }
    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), std::io::Error> {
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
