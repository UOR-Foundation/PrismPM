//! DK-41 owns the complete source-built CLI and transport regressions.
//! Installed-image acceptance additionally belongs to the normal SDK runtime gate.

use std::path::Path;
use std::process::Command;

pub(super) fn verify(root: &Path) {
    // Respect Cargo's standard configured cache; isolate it when no caller
    // configuration exists. This changes no source, lock or test selection.
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| root.join("target/conformance-source-authoring"));
    for (selection, expected) in [
        (vec!["--lib", "source_authoring::tests"], 5),
        (vec!["--test", "source_authoring_cli"], 6),
    ] {
        let output = Command::new("cargo")
            .args(["test", "--locked", "--offline", "-p", "prismpm"])
            .args(selection)
            .args(["--", "--test-threads=1"])
            .env("CARGO_TARGET_DIR", &target)
            .current_dir(root)
            .output()
            .expect("DK-41 executes its complete Rust owners");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "DK-41: {stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let summaries = stdout
            .lines()
            .filter(|line| line.starts_with("test result: "))
            .collect::<Vec<_>>();
        assert_eq!(
            summaries.len(),
            1,
            "DK-41 requires one complete summary: {stdout}"
        );
        assert!(
            summaries[0].starts_with(&format!(
                "test result: ok. {expected} passed; 0 failed; 0 ignored; 0 measured; "
            )),
            "DK-41 requires its complete exact inventory: {stdout}"
        );
    }
    super::node_suite::verify_exact(
        root,
        "DK-41",
        &[("scripts/source-authoring-sdk-check.test.mjs", 6)],
        "120000",
    );
}
