//! Diagnostic specimens passed unchanged through the actual project loader.

use crate::config::ProjectConfig;
use crate::error::PrismError;
use serde::Deserialize;
use serde_json::{json, Value};

const CONTROL: &str = "spec = \"prismpm/project/1\"\nproject = \"Diagnostic\"\nlexlean_project = \"lexlean.toml\"\nbuild_root = \".prism\"\n[limits]\nmax_holo_bytes = 1\nmax_entities = 1\nmax_diagnostics = 1\n";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Specimen {
    configuration: String,
}

pub(super) fn specimens(code: &str) -> (Value, Value) {
    let malformed = match code {
        "PP1001" => format!("unknown = true\n{CONTROL}"),
        "PP1002" => CONTROL.replace("project = \"Diagnostic\"\n", ""),
        "PP1003" => CONTROL.replace("max_entities = 1", "max_entities = 0"),
        _ => unreachable!("closed configuration diagnostic register"),
    };
    (
        json!({"configuration": CONTROL}),
        json!({"configuration": malformed}),
    )
}

pub(super) fn validate(value: &Value) -> Result<(), PrismError> {
    let specimen: Specimen = serde_json::from_value(value.clone()).map_err(|error| {
        PrismError::new("PP9001", format!("configuration probe shape: {error}"))
    })?;
    let io_error = |error| PrismError::new("PP9001", format!("configuration probe setup: {error}"));
    let project = tempfile::tempdir().map_err(io_error)?;
    let path = project.path().join("prismpm.toml");
    std::fs::write(&path, &specimen.configuration).map_err(io_error)?;
    let result = ProjectConfig::load(project.path(), None).map(|_| ());
    if std::fs::read(&path).map_err(io_error)? != specimen.configuration.as_bytes()
        || std::fs::read_dir(project.path()).map_err(io_error)?.count() != 1
    {
        return Err(PrismError::new(
            "PP9001",
            "configuration loading modified its input",
        ));
    }
    result
}
