//! Text/4 source, bounded model, browser Worker and portable native research owner.
use std::path::Path;
use std::process::Command;

fn rust(root: &Path, target: Option<&str>, prefix: &str, expected: usize) {
    let mut command = Command::new("cargo");
    command.args(["test", "--locked", "--offline", "-p", "prismpm"]);
    if let Some(target) = target {
        command.args(["--test", target]);
    } else {
        command.arg("--lib");
    }
    let output = command
        .args([prefix, "--", "--nocapture"])
        .env(
            "CARGO_TARGET_DIR",
            root.join("target/conformance-text-research"),
        )
        .current_dir(root)
        .output()
        .expect("HO-18 executes its actual Rust owners");
    assert!(
        output.status.success(),
        "HO-18 Rust {target:?}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let expected = format!("test result: ok. {expected} passed; 0 failed; 0 ignored;");
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line.starts_with(&expected)),
        "HO-18 requires the exact nonempty Rust owner count"
    );
}

pub(super) fn verify(root: &Path) {
    rust(root, Some("research_workbench"), "research_workbench_", 3);
    rust(
        root,
        Some("research_model_contract"),
        "research_workbench_contract_",
        3,
    );
    rust(root, None, "research_workbench_generator_", 1);
    super::node_suite::verify_exact(
        root,
        "HO-18",
        &[("scripts/text-research.test.mjs", 31)],
        "240000",
    );
    let work = tempfile::tempdir().expect("private research source root");
    let fixture = root.join("tests/fixtures/holo/ho-18-research-workbench-large/project");
    for entry in walkdir::WalkDir::new(&fixture)
        .into_iter()
        .filter_entry(|entry| {
            !matches!(
                entry.file_name().to_str(),
                Some(".prism" | ".lexlean" | ".lake" | "target")
            )
        })
    {
        let entry = entry.expect("regular research source entry");
        assert!(
            !entry.file_type().is_symlink(),
            "HO-18 fixture has no symlinks"
        );
        let destination = work
            .path()
            .join(entry.path().strip_prefix(&fixture).unwrap());
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(destination).unwrap();
        } else {
            std::fs::copy(entry.path(), destination).unwrap();
        }
    }
    let controller = prismpm::Controller::load(work.path())
        .expect("load the complete bounded research source fixture");
    let build = controller
        .build(prismpm::controller::BuildRequest { config_path: None })
        .expect("HO-18 actual native and Wasm source build");
    let artifact = work
        .path()
        .join(&build.model_path)
        .parent()
        .unwrap()
        .to_owned();
    let adapter = Command::new("node")
        .arg(root.join("scripts/research-generated-adapter.mjs"))
        .arg(&artifact)
        .current_dir(root)
        .output()
        .expect("HO-18 executes the actual generated browser adapter");
    assert!(
        adapter.status.success(),
        "HO-18 generated adapter: {}\n{}",
        String::from_utf8_lossy(&adapter.stdout),
        String::from_utf8_lossy(&adapter.stderr)
    );
    let adapter_report: serde_json::Value =
        serde_json::from_slice(&adapter.stdout).expect("one generated adapter result");
    assert_eq!(adapter_report["vectors"], 8);
    assert_eq!(adapter_report["answers"], 6);
    assert_eq!(adapter_report["length_refusals"], 2);
    assert_eq!(adapter_report["exact_multibyte_output_bytes"], 4_194_304);
    let verified = controller
        .verify(prismpm::controller::VerifyRequest { config_path: None })
        .expect("HO-18 actual source, guest and native portable verification");
    assert_eq!(
        build.build_id, verified.build_id,
        "portable acceptance binds the current complete source build"
    );
    // Execute the native HTTP admission negative owner against the same
    // locked source package and warm, invocation-owned native target.
    let native = work.path().join("query-owner");
    let upstream = native.join("hologram-live");
    let harness = native.join("harness");
    std::fs::create_dir_all(&upstream).unwrap();
    std::fs::create_dir_all(harness.join("src")).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&upstream, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let archive = root.join("vendor/hologram-live.tar");
    let extracted = Command::new("tar")
        .args(["--no-same-owner", "--same-permissions", "-xf"])
        .arg(&archive)
        .arg("-C")
        .arg(&upstream)
        .output()
        .expect("HO-18 extracts its exact native source");
    assert!(extracted.status.success());
    let compared = Command::new("python3")
        .args(["-I", "-B"])
        .arg(root.join("scripts/portable-oracle-source.py"))
        .arg(&archive)
        .arg(&upstream)
        .output()
        .expect("HO-18 authenticates the complete native source package");
    assert!(
        compared.status.success(),
        "{}",
        String::from_utf8_lossy(&compared.stderr)
    );
    for file in ["Cargo.toml", "Cargo.lock", "src/main.rs"] {
        std::fs::copy(
            root.join("tests/hologram-oracle").join(file),
            harness.join(file),
        )
        .unwrap();
    }
    let query = Command::new("cargo")
        .args([
            "test",
            "--locked",
            "--offline",
            "research_asset_query_",
            "--",
            "--nocapture",
        ])
        .current_dir(&harness)
        .env(
            "CARGO_TARGET_DIR",
            work.path().join("target/hologram-oracle"),
        )
        .env("CARGO_BUILD_JOBS", "2")
        .env("CARGO_PROFILE_DEV_DEBUG", "0")
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .expect("HO-18 executes native exact-model query admission");
    assert!(
        query.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&query.stdout),
        String::from_utf8_lossy(&query.stderr)
    );
    assert!(
        String::from_utf8_lossy(&query.stdout)
            .lines()
            .any(|line| line.starts_with(
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;"
            )),
        "HO-18 executes its one exact native query owner"
    );
    eprintln!(
        "HO-18 actual research build {} attestation {}",
        build.build_id, verified.attestation_id
    );
}
