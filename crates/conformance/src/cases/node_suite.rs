//! Complete owning Node execution. File wrappers are not registered tests.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const REPORTER: &[u8] = include_bytes!("../../../../scripts/owning-node-reporter.mjs");
const FILE_PREFIX: &str = "# prismpm-owning-file ";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileCompletion {
    file: String,
    success: bool,
    tests: u64,
    passed: u64,
    failed: u64,
    cancelled: u64,
    skipped: u64,
    todo: u64,
    #[serde(rename = "topLevel")]
    top_level: u64,
    suites: u64,
}

fn selected_files(root: &Path, files: &[&str]) -> Vec<PathBuf> {
    assert!(root.is_absolute(), "absolute owning test root");
    assert!(
        !files.is_empty() && files.len() <= 64,
        "bounded selected file set"
    );
    assert_eq!(
        files.iter().collect::<BTreeSet<_>>().len(),
        files.len(),
        "duplicate selected test file"
    );
    files
        .iter()
        .map(|file| {
            assert!(
                file.split('/').all(|part| !part.is_empty()
                    && part != "."
                    && part != ".."
                    && part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"_.-".contains(&byte))),
                "closed relative selected test file"
            );
            let selected = root.join(file);
            let mut current = PathBuf::new();
            for part in selected.components() {
                assert!(
                    matches!(part, Component::RootDir | Component::Normal(_)),
                    "closed selected test path"
                );
                current.push(part.as_os_str());
                let metadata = std::fs::symlink_metadata(&current)
                    .expect("every selected test file and parent must exist");
                assert!(
                    !metadata.file_type().is_symlink(),
                    "selected test path alias"
                );
                assert!(
                    if current == selected {
                        metadata.is_file()
                    } else {
                        metadata.is_dir()
                    },
                    "selected regular test file and directory parents"
                );
            }
            selected
        })
        .collect()
}

fn file_completions(stdout: &str, files: &[PathBuf], total: usize) {
    let rows = stdout
        .lines()
        .filter_map(|line| line.strip_prefix(FILE_PREFIX))
        .map(|line| {
            let row =
                serde_json::from_str::<FileCompletion>(line).expect("closed file completion JSON");
            assert_eq!(
                serde_json::to_string(&row).unwrap(),
                line,
                "exact file completion JSON"
            );
            row
        })
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        files.len(),
        "complete selected test file summaries"
    );
    let mut observed = Vec::new();
    let mut tests = 0usize;
    for row in &rows {
        observed.push(PathBuf::from(&row.file));
        assert!(row.success, "successful selected test file");
        let count = |count: u64| {
            assert!(count <= 9_007_199_254_740_991, "safe file test count");
            usize::try_from(count).expect("platform file test count")
        };
        for value in [
            row.tests,
            row.passed,
            row.failed,
            row.cancelled,
            row.skipped,
            row.todo,
            row.top_level,
            row.suites,
        ] {
            count(value);
        }
        assert!(
            row.tests > 0 && row.top_level > 0 && row.top_level <= row.tests,
            "nonempty registered tests in every selected file"
        );
        assert_eq!(row.passed, row.tests, "complete selected file pass set");
        for outcome in [row.failed, row.cancelled, row.skipped, row.todo] {
            assert_eq!(outcome, 0, "unsuccessful selected file completion");
        }
        tests = tests
            .checked_add(count(row.tests))
            .expect("bounded total file test count");
        assert!(tests <= 9_007_199_254_740_991, "safe total file test count");
    }
    observed.sort();
    let mut expected = files.to_vec();
    expected.sort();
    assert_eq!(observed, expected, "exact selected file completions");
    assert_eq!(
        tests, total,
        "file completions match complete TAP test count"
    );
}

pub(super) fn verify(root: &Path, id: &str, files: &[&str], minimum_tests: usize, timeout: &str) {
    // Validate all selected paths before dispatch. The installed SDK separately
    // binds immutable source bytes; this preflight is not a filesystem race lock.
    let selected = selected_files(root, files);
    let encoded = REPORTER
        .iter()
        .map(|byte| format!("%{byte:02X}"))
        .collect::<String>();
    let reporter = format!("--test-reporter=data:text/javascript,{encoded}");
    let output = Command::new("node")
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("NODE_TEST_CONTEXT")
        .args(["--test", &reporter, "--test-timeout", timeout])
        .args(files)
        .current_dir(root)
        .output()
        .expect("execute complete owning Node suite in the devcontainer");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 TAP output");
    assert!(
        output.status.success(),
        "{id}: {stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let count = |name: &str| {
        let prefix = format!("# {name} ");
        let values = stdout
            .lines()
            .filter_map(|line| line.strip_prefix(&prefix))
            .map(|value| value.parse::<usize>().expect("numeric TAP summary"))
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 1, "{id}: missing or duplicate {name} summary");
        values[0]
    };
    assert!(
        count("tests") >= minimum_tests,
        "{id}: incomplete test suite"
    );
    assert_eq!(count("tests"), count("pass"), "{id}: incomplete pass set");
    for outcome in ["fail", "cancelled", "skipped", "todo"] {
        assert_eq!(
            count(outcome),
            0,
            "{id}: {outcome} tests cannot satisfy acceptance"
        );
    }
    file_completions(&stdout, &selected, count("tests"));
}

#[cfg(test)]
mod tests {
    use super::{file_completions, selected_files, verify, FileCompletion, FILE_PREFIX};
    use std::path::PathBuf;

    #[test]
    fn selected_paths_reject_aliases_nonregular_and_omitted_files_before_execution() {
        for defect in [
            "missing",
            "file-alias",
            "parent-alias",
            "root-alias",
            "directory",
            "fifo",
        ] {
            let temporary = tempfile::tempdir().unwrap();
            let root = temporary.path().join("root");
            std::fs::create_dir_all(root.join("cases")).unwrap();
            let marker = root.join("executed");
            std::fs::write(root.join("first.mjs"), format!(
                "import{{writeFileSync}}from'node:fs';import{{test}}from'node:test';writeFileSync({},'ran');test('first',()=>{{}});",
                serde_json::to_string(&marker).unwrap())).unwrap();
            let selected = root.join("cases/second.mjs");
            std::fs::write(
                &selected,
                "import{test}from'node:test';test('second',()=>{});",
            )
            .unwrap();
            let mut selected_root = root.clone();
            match defect {
                "missing" => std::fs::remove_file(&selected).unwrap(),
                "file-alias" => {
                    std::fs::remove_file(&selected).unwrap();
                    std::os::unix::fs::symlink("../first.mjs", &selected).unwrap();
                }
                "parent-alias" => {
                    std::fs::rename(root.join("cases"), root.join("actual")).unwrap();
                    std::os::unix::fs::symlink("actual", root.join("cases")).unwrap();
                }
                "root-alias" => {
                    selected_root = temporary.path().join("alias");
                    std::os::unix::fs::symlink("root", &selected_root).unwrap();
                }
                "directory" => {
                    std::fs::remove_file(&selected).unwrap();
                    std::fs::create_dir(&selected).unwrap();
                }
                "fifo" => {
                    std::fs::remove_file(&selected).unwrap();
                    assert!(std::process::Command::new("mkfifo")
                        .arg(&selected)
                        .status()
                        .unwrap()
                        .success());
                }
                _ => unreachable!(),
            }
            assert!(
                std::panic::catch_unwind(|| {
                    verify(
                        &selected_root,
                        "test",
                        &["first.mjs", "cases/second.mjs"],
                        1,
                        "5000",
                    );
                })
                .is_err(),
                "{defect}"
            );
            assert!(
                !marker.exists(),
                "{defect} dispatched code before complete preflight"
            );
        }
    }

    #[test]
    fn selected_file_inventory_is_closed_and_unique() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("first.mjs"), "").unwrap();
        for files in [
            vec![],
            vec!["first.mjs", "first.mjs"],
            vec!["/first.mjs"],
            vec!["../first.mjs"],
            vec!["./first.mjs"],
            vec!["a//b.mjs"],
            vec!["*.mjs"],
            vec!["first?.mjs"],
            vec!["[first].mjs"],
            vec!["first\\file.mjs"],
            vec!["first.mjs"; 65],
        ] {
            assert!(std::panic::catch_unwind(|| selected_files(root.path(), &files)).is_err());
        }
    }

    #[test]
    fn file_rows_are_closed_unique_nonempty_and_reconciled() {
        let row = FileCompletion {
            file: "/exact.mjs".to_owned(),
            success: true,
            tests: 3,
            passed: 3,
            failed: 0,
            cancelled: 0,
            skipped: 0,
            todo: 0,
            top_level: 3,
            suites: 0,
        };
        let raw = serde_json::to_string(&row).unwrap();
        let encoded = format!("{FILE_PREFIX}{raw}\n");
        let files = [PathBuf::from("/exact.mjs")];
        file_completions(&encoded, &files, 3);
        for changed in [
            String::new(),
            encoded.repeat(2),
            encoded.replace("/exact.mjs", "/other.mjs"),
            encoded.replace("\"success\":true", "\"success\":false"),
            encoded.replace("\"tests\":3", "\"tests\":0"),
            encoded.replace("\"tests\":3", "\"tests\":\"3\""),
            encoded.replace("\"tests\":3", "\"tests\":3,\"tests\":3"),
            encoded.replace("\"passed\":3", "\"passed\":2"),
            encoded.replace("\"failed\":0", "\"failed\":1"),
            encoded.replace("\"cancelled\":0", "\"cancelled\":1"),
            encoded.replace("\"skipped\":0", "\"skipped\":1"),
            encoded.replace("\"todo\":0", "\"todo\":1"),
            encoded.replace("\"topLevel\":3", "\"topLevel\":0"),
            encoded.replace("\"topLevel\":3", "\"topLevel\":4"),
            encoded.replace("\"suites\":0", "\"suites\":-1"),
            encoded.replace("\"suites\":0", "\"suites\":9007199254740992"),
            encoded.replace("\"suites\":0", "\"suites\":0,\"extra\":true"),
        ] {
            assert!(std::panic::catch_unwind(|| file_completions(&changed, &files, 3)).is_err());
        }
        assert!(std::panic::catch_unwind(|| file_completions(&encoded, &files, 4)).is_err());
    }

    #[test]
    fn invented_stdout_completion_does_not_become_a_registered_test() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("test.mjs"),
            "console.log('# prismpm-owning-file '+JSON.stringify({file:import.meta.filename,tests:1,passed:1}));").unwrap();
        assert!(
            std::panic::catch_unwind(|| verify(root.path(), "test", &["test.mjs"], 1, "5000"))
                .is_err()
        );
    }
}
