//! Strict Prism project configuration and confined path resolution.

use crate::error::PrismError;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Positive resource limits for projection and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectLimits {
    /// Maximum canonical model document and physical Holo archive byte length.
    pub max_holo_bytes: u64,
    /// Maximum total Holo entities and catalog rows.
    pub max_entities: u64,
    /// Maximum returned diagnostics.
    pub max_diagnostics: u64,
}

impl ProjectLimits {
    pub(crate) fn check_holo_length(&self, length: usize) -> Result<(), PrismError> {
        if u64::try_from(length).is_ok_and(|length| length <= self.max_holo_bytes) {
            Ok(())
        } else {
            Err(PrismError::new("PP1003", "max_holo_bytes exceeded"))
        }
    }
}

/// Closed prismpm/project/1 configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    /// Schema discriminator.
    pub spec: String,
    /// Human-facing project name.
    pub project: String,
    /// Confined path to the LexLean project file.
    pub lexlean_project: String,
    /// Confined relative output root.
    pub build_root: String,
    /// Positive resource limits.
    pub limits: ProjectLimits,
}

// Retain missing fields and signed TOML integers until their registered
// configuration diagnostics can be selected. Serde still owns closed shapes
// and type checking; error classification never parses Serde's prose.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigInput {
    spec: Option<String>,
    project: Option<String>,
    lexlean_project: Option<String>,
    build_root: Option<String>,
    limits: Option<LimitsInput>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitsInput {
    max_holo_bytes: Option<i64>,
    max_entities: Option<i64>,
    max_diagnostics: Option<i64>,
}

fn required<T>(value: Option<T>, field: &str) -> Result<T, PrismError> {
    value.ok_or_else(|| {
        PrismError::new(
            "PP1002",
            format!("missing required configuration field {field}"),
        )
    })
}

impl ConfigInput {
    fn complete(self) -> Result<ProjectConfig, PrismError> {
        let spec = required(self.spec, "spec")?;
        let project = required(self.project, "project")?;
        let lexlean_project = required(self.lexlean_project, "lexlean_project")?;
        let build_root = required(self.build_root, "build_root")?;
        let limits = required(self.limits, "limits")?;
        let integer = |value, field| {
            u64::try_from(required(value, field)?)
                .map_err(|_| PrismError::new("PP1003", "project limits are outside fixed bounds"))
        };
        Ok(ProjectConfig {
            spec,
            project,
            lexlean_project,
            build_root,
            limits: ProjectLimits {
                max_holo_bytes: integer(limits.max_holo_bytes, "limits.max_holo_bytes")?,
                max_entities: integer(limits.max_entities, "limits.max_entities")?,
                max_diagnostics: integer(limits.max_diagnostics, "limits.max_diagnostics")?,
            },
        })
    }
}

fn relative(path: &str, field: &str) -> Result<PathBuf, PrismError> {
    let value = Path::new(path);
    let valid_component = |component: &str| {
        let body = component.strip_prefix('.').unwrap_or(component);
        !body.is_empty()
            && body
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
            && body
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    };
    if value.as_os_str().is_empty()
        || path.contains('\\')
        || path.split('/').any(str::is_empty)
        || !path.split('/').all(valid_component)
        || value.components().any(|component| {
            matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
    {
        return Err(PrismError::new(
            "PP8001",
            format!("{field} is not a confined relative path"),
        ));
    }
    Ok(value.to_path_buf())
}

fn existing_file(root: &Path, relative: &Path) -> Result<PathBuf, PrismError> {
    let joined = root.join(relative);
    let canonical = joined
        .canonicalize()
        .map_err(|error| PrismError::new("PP1002", format!("{}: {error}", joined.display())))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(PrismError::new(
            "PP8001",
            "configured file escapes the project",
        ));
    }
    let mut cursor = root.to_path_buf();
    for component in relative.components() {
        cursor.push(component.as_os_str());
        if std::fs::symlink_metadata(&cursor)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(PrismError::new(
                "PP8001",
                "configured path contains a symlink",
            ));
        }
    }
    Ok(canonical)
}

impl ProjectConfig {
    pub(crate) fn load(
        root: &Path,
        selected: Option<&Path>,
    ) -> Result<(Self, PathBuf), PrismError> {
        let selected = selected.unwrap_or_else(|| Path::new("prismpm.toml"));
        let relative = relative(
            selected
                .to_str()
                .ok_or_else(|| PrismError::new("PP1001", "configuration path is not UTF-8"))?,
            "configuration path",
        )?;
        let path = existing_file(root, &relative)?;
        let bytes = std::fs::read(&path)
            .map_err(|error| PrismError::new("PP1002", format!("{}: {error}", path.display())))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| PrismError::new("PP1001", "configuration is not UTF-8"))?;
        let input: ConfigInput = toml::from_str(text)
            .map_err(|error| PrismError::new("PP1001", format!("configuration: {error}")))?;
        let config = input.complete()?;
        config.validate()?;
        Ok((config, path))
    }

    fn validate(&self) -> Result<(), PrismError> {
        if self.spec != "prismpm/project/1" || self.project.is_empty() {
            return Err(PrismError::new(
                "PP1002",
                "project schema or name is invalid",
            ));
        }
        let _ = relative(&self.lexlean_project, "lexlean_project")?;
        let build_root = relative(&self.build_root, "build_root")?;
        if build_root.components().count() != 1 {
            return Err(PrismError::new(
                "PP8001",
                "build_root must be one project-root directory name",
            ));
        }
        if self.limits.max_holo_bytes == 0
            || self.limits.max_entities == 0
            || self.limits.max_diagnostics == 0
            || self.limits.max_holo_bytes > 1_073_741_824
            || self.limits.max_entities > 10_000_000
            || self.limits.max_diagnostics > 10_000
        {
            return Err(PrismError::new(
                "PP1003",
                "project limits are outside fixed bounds",
            ));
        }
        Ok(())
    }

    pub(crate) fn lexlean_path(&self, root: &Path) -> Result<PathBuf, PrismError> {
        existing_file(root, &relative(&self.lexlean_project, "lexlean_project")?)
    }

    pub(crate) fn output_root(&self, root: &Path) -> Result<PathBuf, PrismError> {
        let relative = relative(&self.build_root, "build_root")?;
        let mut cursor = root.to_path_buf();
        for component in relative.components() {
            cursor.push(component.as_os_str());
            match std::fs::symlink_metadata(&cursor) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(PrismError::new(
                        "PP8001",
                        "configured output path contains a symlink",
                    ));
                }
                Ok(metadata) if !metadata.is_dir() => {
                    return Err(PrismError::new(
                        "PP8001",
                        "configured output path contains a non-directory",
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
                Err(error) => {
                    return Err(PrismError::new(
                        "PP8001",
                        format!("inspect configured output path: {error}"),
                    ));
                }
            }
        }
        Ok(root.join(relative))
    }
}

#[cfg(test)]
mod limit_tests {
    use super::{ProjectConfig, ProjectLimits};

    const CONTROL: &str = "spec = \"prismpm/project/1\"\nproject = \"Diagnostic\"\nlexlean_project = \"lexlean.toml\"\nbuild_root = \".prism\"\n[limits]\nmax_holo_bytes = 1\nmax_entities = 1\nmax_diagnostics = 1\n";

    fn load(text: &str) -> Result<ProjectConfig, crate::error::PrismError> {
        let project = tempfile::tempdir().unwrap();
        let path = project.path().join("prismpm.toml");
        std::fs::write(&path, text).unwrap();
        let result = ProjectConfig::load(project.path(), None).map(|(config, _)| config);
        assert_eq!(std::fs::read_to_string(path).unwrap(), text);
        assert_eq!(std::fs::read_dir(project.path()).unwrap().count(), 1);
        result
    }

    #[test]
    fn loader_reports_absent_required_fields_at_the_registered_boundary() {
        load(CONTROL).unwrap();
        for line in CONTROL.lines().filter(|line| line.contains(" = ")) {
            let malformed = CONTROL.replace(&format!("{line}\n"), "");
            let actual = load(&malformed).unwrap_err();
            assert_eq!(actual.code.as_str(), "PP1002", "absent {line}: {actual}");
        }
        let actual = load(CONTROL.split("[limits]").next().unwrap()).unwrap_err();
        assert_eq!(actual.code.as_str(), "PP1002");
    }

    #[test]
    fn loader_distinguishes_closed_shape_and_positive_limit_errors() {
        for malformed in [
            format!("unknown = true\n{CONTROL}"),
            format!("{CONTROL}unknown = true\n"),
            format!("{CONTROL}max_entities = 1\n"),
            CONTROL.replace("max_entities = 1", "max_entities = \"one\""),
        ] {
            assert_eq!(load(&malformed).unwrap_err().code.as_str(), "PP1001");
        }
        for (field, maximum) in [
            ("max_holo_bytes", 1_073_741_824_i64),
            ("max_entities", 10_000_000),
            ("max_diagnostics", 10_000),
        ] {
            for accepted in [1, maximum] {
                load(&CONTROL.replace(&format!("{field} = 1"), &format!("{field} = {accepted}")))
                    .unwrap();
            }
            for rejected in [-1, 0, maximum + 1] {
                let malformed =
                    CONTROL.replace(&format!("{field} = 1"), &format!("{field} = {rejected}"));
                let actual = load(&malformed).unwrap_err();
                assert_eq!(actual.code.as_str(), "PP1003", "{field}: {actual}");
            }
        }
    }

    #[test]
    fn holo_length_limit_includes_the_boundary_and_rejects_larger_bytes() {
        let limits = ProjectLimits {
            max_holo_bytes: 128,
            max_entities: 1,
            max_diagnostics: 1,
        };
        for length in [0, 127, 128] {
            limits.check_holo_length(length).unwrap();
        }
        for length in [129, 1048576, usize::MAX] {
            assert_eq!(limits.check_holo_length(length).unwrap_err().code, "PP1003");
        }
    }
}
