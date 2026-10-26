//! HO-23, HO-24 and HO-25: the serving effect of a server application, its
//! RFC 9112 framing over a loopback socket, and the core built through the
//! production API.

use std::path::Path;
use std::process::Command;

fn run(root: &Path, id: &str, arguments: &[&str]) -> String {
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["test", "--locked", "--offline", "-p", "prismpm"])
        .args(arguments)
        .args(["--", "--nocapture", "--test-threads=1"])
        // The outer conformance Cargo owner may retain its target lock; use a
        // disjoint target for the owning suites.
        .env(
            "CARGO_TARGET_DIR",
            root.join("target/conformance-server-serving"),
        )
        .output()
        .expect("the owning suites execute");
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 owning stdout");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 owning stderr");
    assert!(
        output.status.success(),
        "{id} {arguments:?}: {stdout}\n{stderr}"
    );
    assert!(
        stdout.contains("test result: ok.") && !stdout.contains("running 0 tests"),
        "{id} requires nonempty executed suites: {arguments:?}"
    );
    stdout
}

fn require(id: &str, output: &str, names: &[&str]) {
    for name in names {
        assert!(
            output
                .lines()
                .any(|line| line.ends_with(&format!("{name} ... ok"))),
            "{id} requires passing {name}"
        );
    }
}

/// HO-23: the serving declaration and its closed validation.
pub(super) fn verify_declaration(root: &Path) {
    let contract = run(root, "HO-23", &["--test", "server_serving_contract"]);
    require(
        "HO-23",
        &contract,
        &[
            "serving_declaration_roundtrips_and_requires_the_closed_record",
            "serving_rejects_every_invalid_declaration",
            "serving_refusals_agree_across_lean_registry_and_generated_adapter",
            "a_served_application_projects_from_source",
        ],
    );
    let library = run(root, "HO-23", &["--lib", "server_application::serving"]);
    require(
        "HO-23",
        &library,
        &["a_frame_effect_is_never_an_effect_the_core_may_request"],
    );
}

/// HO-24: the generated listener and its framing, over a real socket.
pub(super) fn verify_framing(root: &Path) {
    let library = run(root, "HO-24", &["--lib", "server_host::serving_tests"]);
    require(
        "HO-24",
        &library,
        &[
            "the_listener_binds_loopback_only_and_refuses_any_other_address",
            "a_malformed_request_line_is_a_400_before_the_core_is_called",
            "an_oversized_header_section_is_a_431",
            "too_many_header_fields_are_a_431",
            "an_oversized_body_is_a_413_by_content_length_and_by_chunks",
            "a_forbidden_origin_is_a_403_before_the_core_is_called",
            "accept_and_content_type_are_required_from_the_model",
            "fields_reach_the_core_in_received_order_without_merging",
            "content_length_and_chunked_bodies_are_delimited_by_rfc_9112",
            "host_provided_frame_sections_come_from_declared_effects",
            "the_core_response_is_serialized_as_returned",
            "every_served_vector_is_answered_over_a_real_socket",
        ],
    );
}

/// HO-25: the core through `production`, and the example server.
pub(super) fn verify_build(root: &Path) {
    let contract = run(root, "HO-25", &["--test", "server_serving_build"]);
    require(
        "HO-25",
        &contract,
        &[
            "the_example_server_serves_its_vectors_end_to_end",
            "a_core_that_is_not_a_production_root_is_refused",
            "the_example_holds_the_foundation_modules_it_imports_byte_for_byte",
        ],
    );
}
