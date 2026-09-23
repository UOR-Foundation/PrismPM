//! Complete owning declaration gate; runtime and browser acceptance are separate.

#[path = "../../../tests/support/browser_application.rs"]
mod browser_application;

#[test]
fn browser_application_source_projection_is_closed_and_build_is_unavailable() {
    browser_application::verify(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
}
