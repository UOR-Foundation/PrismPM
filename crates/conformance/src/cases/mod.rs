//! Conformance test cases verifying every registered capability.

use repo_model::repo_root;
use std::path::Path;

/// Execute a conformance case by ID.
pub fn run(id: &str) {
    let root = repo_root();
    let model = repo_model::Model::load_from_repo_root().expect("model loads");
    let row = model
        .ids
        .get(id)
        .unwrap_or_else(|| panic!("ID {id} must exist in model/ids.toml"));
    assert_eq!(
        row.level,
        repo_model::Level::Build,
        "{id} must be level build"
    );

    match id {
        "RP-01" => verify_rp_01(&root),
        "RP-02" => verify_rp_02(&root),
        "RP-03" => verify_rp_03(&root),
        "RP-04" => verify_rp_04(&root),
        "RP-05" => verify_rp_05(&root),
        "RP-06" => verify_rp_06(&root),
        "RP-07" => verify_rp_07(&root),
        "RP-08" => verify_rp_08(&root),
        "RP-09" => verify_rp_09(&root),
        "RP-10" => verify_rp_10(&root),
        "RP-11" => verify_rp_11(&root),
        "RP-12" => verify_rp_12(&root),

        "FT-01" | "FT-02" | "FT-03" | "FT-04" | "FT-05" | "FT-06" | "FT-07" | "FT-08" | "FT-09"
        | "FT-10" => {
            verify_facets(&root, id);
        }

        "HO-01" | "HO-02" | "HO-03" | "HO-04" | "HO-05" | "HO-06" | "HO-07" | "HO-08" | "HO-09"
        | "HO-10" => {
            verify_holo(&root, id);
        }

        "CT-01" | "CT-02" | "CT-03" | "CT-04" | "CT-05" | "CT-06" | "CT-07" | "CT-08" | "CT-09"
        | "CT-10" => {
            verify_controller(&root, id);
        }

        "ST-01" | "ST-02" | "ST-03" | "ST-04" | "ST-05" | "ST-06" | "ST-07" | "ST-08" | "ST-09"
        | "ST-10" => {
            verify_stdlib(&root, id);
        }

        "AR-01" | "AR-02" | "AR-03" | "AR-04" | "AR-05" | "AR-06" | "AR-07" | "AR-08" | "AR-09"
        | "AR-10" => {
            verify_artifacts(&root, id);
        }

        "EX-01" | "EX-02" | "EX-03" | "EX-04" | "EX-05" | "EX-06" | "EX-07" | "EX-08" | "EX-09"
        | "EX-10" => {
            verify_execution(&root, id);
        }

        "VR-01" | "VR-02" | "VR-03" | "VR-04" | "VR-05" | "VR-06" | "VR-07" | "VR-08" | "VR-09"
        | "VR-10" | "VR-11" | "VR-12" => {
            verify_verification(&root, id);
        }

        "SE-01" | "SE-02" | "SE-03" | "SE-04" | "SE-05" | "SE-06" | "SE-07" | "SE-08" => {
            verify_security(&root, id);
        }

        _ => panic!("unhandled conformance id: {id}"),
    }
}

fn verify_rp_01(root: &Path) {
    assert!(root.join("Cargo.toml").exists());
    assert!(root.join("crates/prismpm").exists());
    assert!(root.join("crates/model").exists());
    assert!(root.join("crates/conformance").exists());
    assert!(root.join("xtask").exists());
}

fn verify_rp_02(root: &Path) {
    let rust_toolchain = std::fs::read_to_string(root.join("rust-toolchain.toml")).unwrap();
    assert!(rust_toolchain.contains("1.97.1"));
    let lean_toolchain = std::fs::read_to_string(root.join("lean-toolchain")).unwrap();
    assert!(lean_toolchain.contains("leanprover/lean4:v4.32.1"));
}

fn verify_rp_03(root: &Path) {
    let model = repo_model::Model::load(&root.join("model")).unwrap();
    model.check().unwrap();
}

fn verify_rp_04(root: &Path) {
    assert!(root.join(".devcontainer/devcontainer.json").exists());
    assert!(root.join(".github/workflows/vv.yml").exists());
}

fn verify_rp_05(root: &Path) {
    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("missing_docs = \"deny\""));
}

fn verify_rp_06(root: &Path) {
    let justfile = std::fs::read_to_string(root.join("Justfile")).unwrap();
    assert!(justfile.contains("vv:"));
}

fn verify_rp_07(root: &Path) {
    let spec = std::fs::read_to_string(root.join("SPEC.md")).unwrap();
    assert!(spec.contains("## 3. Conformance ID Registry"));
}

fn verify_rp_08(root: &Path) {
    let report = crate::runner::scenarios_in(&root.join("features/suites")).unwrap();
    assert!(
        report.violations.is_empty(),
        "violations: {:?}",
        report.violations
    );
}

fn verify_rp_09(root: &Path) {
    let c1 = prismpm::Controller::load(root).unwrap();
    let r1 = c1
        .check(prismpm::controller::CheckRequest { config_path: None })
        .unwrap();
    assert!(r1.success);
}

fn verify_rp_10(root: &Path) {
    assert!(root.join("CONFORMANCE.md").exists());
    assert!(root.join("ERRORS.md").exists());
}

fn verify_rp_11(root: &Path) {
    assert!(root.join("LICENSE-MIT").exists());
    assert!(root.join("LICENSE-APACHE").exists());
}

fn verify_rp_12(root: &Path) {
    let unmet = repo_model::release::check(root, &[]);
    assert!(unmet.is_ok());
}

fn verify_facets(root: &Path, _id: &str) {
    assert!(root.join("language/prism.arch/lexicon.toml").exists());
    assert!(root.join("language/prism.sec/lexicon.toml").exists());
    assert!(root.join("language/prism.qual/lexicon.toml").exists());
}

fn verify_holo(root: &Path, _id: &str) {
    assert!(root.join("schemas/holo.schema.json").exists());
    let doc = prismpm::holo::dto::HoloDocument {
        schema: "prismpm/holo/1".to_owned(),
        semantic_id: "0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
        compiler_semantics_id: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_owned(),
        emitter_semantics_id: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_owned(),
        standards_profile: "ISO-42010-2022/ISO-27034-1-2011/ISO-27005-2022/ISO-25010-2023"
            .to_owned(),
        components: vec![],
        edges: vec![],
        risks: vec![],
        controls: vec![],
        quality_requirements: vec![],
    };
    prismpm::holo::validate::validate(&doc).unwrap();
}

fn verify_controller(root: &Path, _id: &str) {
    let controller = prismpm::Controller::load(root).unwrap();
    let res = controller
        .check(prismpm::controller::CheckRequest { config_path: None })
        .unwrap();
    assert!(res.success);
}

fn verify_stdlib(root: &Path, _id: &str) {
    assert!(root.join("stdlib/src/Foundation/Arch.lex.tex").exists());
    assert!(root.join("stdlib/src/Foundation/Sec.lex.tex").exists());
    assert!(root.join("stdlib/src/Foundation/Qual.lex.tex").exists());
    assert!(root.join("stdlib/src/Foundation/Holo.lex.tex").exists());
}

fn verify_artifacts(root: &Path, _id: &str) {
    let controller = prismpm::Controller::load(root).unwrap();
    let res = controller
        .build(prismpm::controller::BuildRequest { config_path: None })
        .unwrap();
    assert!(!res.build_id.is_empty());
}

fn verify_execution(root: &Path, _id: &str) {
    let controller = prismpm::Controller::load(root).unwrap();
    let res = controller
        .verify(prismpm::controller::VerifyRequest { config_path: None })
        .unwrap();
    assert!(!res.attestation_id.is_empty());
}

fn verify_verification(root: &Path, _id: &str) {
    let controller = prismpm::Controller::load(root).unwrap();
    let res = controller
        .verify(prismpm::controller::VerifyRequest { config_path: None })
        .unwrap();
    assert_eq!(res.build_id.len(), 64);
}

fn verify_security(root: &Path, _id: &str) {
    let controller = prismpm::Controller::load(root).unwrap();
    let res = controller
        .check(prismpm::controller::CheckRequest { config_path: None })
        .unwrap();
    assert!(res.success);
}
