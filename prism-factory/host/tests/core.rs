//! Differential and behavioral checks of the certified factory core through
//! the host. The kernel proves the routing and graph theorems in the model;
//! these hold the byte-level parts the kernel cannot evaluate (SHA-256 over
//! `Bytes`, the text protocol) against an independent oracle.

use prism_factory_host::{plan, reweigh, workflow};

fn planned(request: &str) -> String {
    String::from_utf8(plan(request.as_bytes()).expect("no overflow")).expect("UTF-8")
}

fn route<'a>(plan: &'a str, name: &str) -> (&'a str, &'a str) {
    let prefix = format!("obligation {name} ");
    let line = plan.lines().find(|line| line.starts_with(&prefix)).expect("obligation in plan");
    let mut rest = line[prefix.len()..].split(' ');
    (rest.next().unwrap(), rest.next().unwrap())
}

const TREE: &str = "tree 100644:aaaa Cargo.lock\ntree 100644:bbbb docs/x.md\n";

#[test]
fn identity_matches_an_independent_sha256() {
    // sha256 of the identity text of `cargo-fetch` over TREE, computed with
    // Python's hashlib from the format in prism-factory/README.md:
    //   closure = sha256("file 100644:aaaa Cargo.lock\n")
    //   identity = sha256("prism-factory/obligation/1\nname cargo-fetch\n...closure <closure>\n")
    let plan = planned(TREE);
    assert_eq!(route(&plan, "cargo-fetch"), ("observe", "34597c03661e0387be65405b80d92b93d011b6f97575208a2f26658d3b38e2f9"));
}

#[test]
fn a_change_outside_the_closure_keeps_the_identity() {
    let before = planned(TREE);
    let after = planned("tree 100644:aaaa Cargo.lock\ntree 100644:cccc docs/x.md\n");
    assert_eq!(route(&before, "cargo-fetch").1, route(&after, "cargo-fetch").1);
    assert_eq!(route(&before, "factory-model").1, route(&after, "factory-model").1);
    // Whole-source obligations read docs/ too, so they change.
    assert_ne!(route(&before, "source-vv-dk-3").1, route(&after, "source-vv-dk-3").1);
}

#[test]
fn a_change_inside_the_closure_reaches_every_dependent() {
    let before = planned(TREE);
    let after = planned("tree 100644:abcd Cargo.lock\ntree 100644:bbbb docs/x.md\n");
    assert_ne!(route(&before, "cargo-fetch").1, route(&after, "cargo-fetch").1);
    assert_eq!(route(&before, "devcontainer-image").1, route(&after, "devcontainer-image").1);
    assert_ne!(route(&before, "gate").1, route(&after, "gate").1);
    assert_ne!(route(&before, "release").1, route(&after, "release").1);
}

#[test]
fn receipts_route_by_kind() {
    let first = planned(TREE);
    let fetch = route(&first, "cargo-fetch").1.to_string();
    let model = route(&first, "factory-model").1.to_string();
    let passed = planned(&format!("{TREE}receipt {fetch} pass\nreceipt {model} pass\n"));
    assert_eq!(route(&passed, "cargo-fetch").0, "proved");
    assert_eq!(route(&passed, "factory-model").0, "proved");
    let failed = planned(&format!("{TREE}receipt {fetch} fail\nreceipt {model} fail\n"));
    // A fetch reaches the network, so a failure is observed again; a hermetic
    // failure on the same inputs is refuted and not re-run.
    assert_eq!(route(&failed, "cargo-fetch").0, "observe");
    assert_eq!(route(&failed, "factory-model").0, "refuted");
    assert!(failed.contains("releasable no"));
}

#[test]
fn only_observed_host_obligations_carry_their_files() {
    let plan = planned(TREE);
    assert!(plan.contains("file cargo-fetch Cargo.lock\n"));
    assert!(!plan.contains("file cargo-fetch docs/x.md"));
    assert!(!plan.lines().any(|line| line.starts_with("file source-vv-")));
}

#[test]
fn the_schedule_follows_learned_priority() {
    let plan = planned(&format!("{TREE}weight factory-model 9\n"));
    let order: Vec<&str> = plan.lines().filter_map(|line| line.strip_prefix("schedule ")).collect();
    assert_eq!(order.first(), Some(&"factory-model"));
}

#[test]
fn reweighing_raises_failures_and_lowers_passes() {
    let out = String::from_utf8(reweigh(b"weight gate 4\noutcome gate fail\noutcome cargo-fetch pass\n").unwrap()).unwrap();
    assert!(out.contains("weight gate 6\n"));
    assert!(out.contains("weight cargo-fetch 3\n"));
    assert!(out.contains("weight release 4\n"));
}

#[test]
fn the_committed_workflow_is_the_projection() {
    assert_eq!(workflow(), include_bytes!("../../../.github/workflows/factory.yml"));
}
