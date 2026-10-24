//! HO-26 and HO-27: the verification of a server application and the exercise of
//! every declared effect; CT-12: the LexLean build root of the controller.

use super::server_serving::{require, run};
use std::path::Path;

/// HO-26: `prismpm verify` of the example server, and its refusals.
pub(super) fn verify_verify(root: &Path) {
    let library = run(root, "HO-26", &["--lib", "server_host::verify_tests"]);
    require(
        "HO-26",
        &library,
        &[
            "a_replayed_vector_with_a_wrong_effect_result_is_refused",
            "a_served_exchange_whose_response_differs_is_refused",
            "a_workspace_without_the_declared_binding_or_its_lock_is_refused",
            "host_packages_equal_to_their_regeneration_are_accepted",
            "host_packages_that_differ_from_their_regeneration_are_refused",
            "staging_lays_the_consumers_files_beside_the_build_and_nothing_else",
        ],
    );
    let contract = run(root, "HO-26", &["--test", "server_verification"]);
    require(
        "HO-26",
        &contract,
        &["verify_of_the_example_server_runs_the_application_acceptance"],
    );
}

/// HO-27: every declared effect is exercised, as observed.
pub(super) fn verify_exercise(root: &Path) {
    let library = run(root, "HO-27", &["--lib", "server_host::exercise_tests"]);
    require(
        "HO-27",
        &library,
        &[
            "a_core_effect_no_vector_reaches_is_refused_with_pp2021",
            "a_frame_effect_no_served_exchange_reaches_is_refused_with_pp2021",
            "the_record_names_every_effect_with_what_reached_it",
        ],
    );
}

/// CT-12: the controller reads the build root from `lexlean.toml`.
pub(super) fn verify_build_root(root: &Path) {
    let library = run(root, "CT-12", &["--lib", "controller::model_worker"]);
    require(
        "CT-12",
        &library,
        &["the_build_root_is_read_from_lexlean_toml"],
    );
    let contract = run(root, "CT-12", &["--test", "controller_build_root"]);
    require(
        "CT-12",
        &contract,
        &[
            "a_project_with_another_build_root_builds_through_it",
            "a_lexlean_toml_without_a_build_root_is_refused",
        ],
    );
}
