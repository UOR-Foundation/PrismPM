//! Source-bound native acceptance of the closed browser Holo wire profile.

use prod_codegen::{generate_cargo_package, CargoPackageSpec};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn verify(root: &Path) {
    let verified = super::verified_root(root);
    let kernel = std::fs::read(verified.join("kernel.ir")).unwrap();
    let manifest = super::verification_manifest(root);
    assert_eq!(
        manifest["artifacts"]["kernel_ir"]["sha256"],
        super::sha256(&kernel)
    );
    assert_eq!(
        manifest["artifacts"]["kernel_ir"]["byte_length"],
        kernel.len()
    );
    let text = std::str::from_utf8(&kernel).unwrap();
    let (remaining, module) = prod_ir::parser::parse_module(text).unwrap();
    assert!(remaining.trim().is_empty());
    let metadata: repo_model::StdlibPackage =
        toml::from_str(&super::read(root, "model/stdlib-package.toml")).unwrap();
    metadata.check().unwrap();
    let spec = CargoPackageSpec {
        name: metadata.name,
        version: metadata.version,
        description: metadata.description,
        repository: metadata.repository,
        homepage: metadata.homepage,
        readme: super::read(root, "stdlib/README.md"),
        license_mit: super::read(root, "stdlib/LICENSE-MIT"),
        license_apache: super::read(root, "stdlib/LICENSE-APACHE"),
        input_sha256: super::sha256(&kernel),
        dependencies: Vec::new(),
    };
    let generated =
        generate_cargo_package(&module, &spec).expect("current verified LCNF regenerates");
    let directory = root.join("stdlib/generated/package");
    let expected = generated
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    for entry in walkdir::WalkDir::new(&directory) {
        let entry = entry.unwrap();
        assert!(
            !entry.file_type().is_symlink(),
            "no generated-package aliases"
        );
        if entry.file_type().is_file() {
            observed.insert(
                entry
                    .path()
                    .strip_prefix(&directory)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        } else {
            assert!(
                entry.file_type().is_dir(),
                "no generated-package special files"
            );
        }
    }
    assert_eq!(expected, observed.iter().map(String::as_str).collect());
    for file in generated.files {
        assert_eq!(
            std::fs::read(directory.join(&file.path)).unwrap(),
            file.bytes,
            "current source-derived package {}",
            file.path
        );
    }
    super::verify_node_suite(
        root,
        "HO-13",
        &["tests/holo-browser-codec/check.mjs"],
        3,
        "1200000",
    );
}
