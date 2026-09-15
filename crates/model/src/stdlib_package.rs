//! Authoritative metadata for the generated standard-library Cargo package.

use crate::ModelError;
use serde::Deserialize;

/// Closed development-package metadata, independent of generated Cargo output.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdlibPackage {
    /// Register schema identifier.
    pub spec: String,
    /// Stable standard-library crate name.
    pub name: String,
    /// Exact currently supported release version, 0.2.0 (SPEC §5).
    pub version: String,
    /// Published Cargo description.
    pub description: String,
    /// Authoritative source repository.
    pub repository: String,
    /// Published project homepage.
    pub homepage: String,
}

impl StdlibPackage {
    /// Validate the closed package identity and metadata contract.
    pub fn check(&self) -> Result<(), ModelError> {
        if self.spec != "prismpm/stdlib-package/1"
            || self.name != "prism-stdlib"
            || self.version != "0.2.0"
            || self.description.trim().is_empty()
            || self.repository != "https://github.com/UOR-Foundation/PrismPM"
            || self.homepage != self.repository
        {
            return Err(ModelError::Inconsistent(
                "invalid standard-library package metadata".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("../../../model/stdlib-package.toml");

    #[test]
    fn stdlib_package_metadata_is_closed_and_validated() {
        let metadata: StdlibPackage = toml::from_str(SOURCE).unwrap();
        metadata.check().unwrap();
        assert!(toml::from_str::<StdlibPackage>(&format!("{SOURCE}\nunknown = true")).is_err());
        for (field, value) in [
            ("spec", "other/1"),
            ("name", "other-crate"),
            ("version", "0.2"),
            ("version", "0.02.0"),
            ("version", "0.1.0"),
            ("version", "0.2.1"),
            ("version", "1.0.0"),
            ("description", " "),
            ("repository", "http://github.com/UOR-Foundation/PrismPM"),
            ("homepage", "https://github.com/UOR-Foundation/template"),
        ] {
            let mut document: toml::Value = toml::from_str(SOURCE).unwrap();
            document[field] = value.into();
            let invalid: StdlibPackage = document.try_into().unwrap();
            assert!(invalid.check().is_err(), "accepted invalid {field}");
        }
    }
}
