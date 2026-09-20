//! Complete private compiler prerequisite gate; no runtime publication bypass.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

pub(super) fn verify(root: &Path) {
    let output = Command::new("cargo")
        .current_dir(root)
        .args([
            "test",
            "--locked",
            "--offline",
            "-p",
            "prismpm",
            "--lib",
            "browser_build::tests::",
            "--",
            "--nocapture",
            "--test-threads=1",
        ])
        .output()
        .expect("the private compiler owning gate executes");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    let names = stdout
        .lines()
        .filter_map(|line| {
            line.strip_prefix("test ")
                .and_then(|line| line.strip_suffix(" ... ok"))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        names,
        BTreeSet::from([
            "browser_build::tests::browser_compiler_actual_source_native_wasm_and_complete_replay",
        "browser_build::tests::browser_compiler_plan_preserves_roles_roots_and_exact_budgets",
        "browser_build::tests::browser_compiler_toolchain_rejects_ambient_selection_and_spoofed_cargo",
        ])
    );
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.starts_with("test result:"))
            .count(),
        1
    );
    assert!(
        stdout.contains("test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured;"),
        "{stdout}"
    );
}
