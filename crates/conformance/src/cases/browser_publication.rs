//! Complete public byte-observation gate, including actual loopback TLS and OCI replay.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

const TESTS: [&str; 12] = [
    "cli::tests::browser_publication_requires_explicit_target_without_trust_bypass_flags",
    "oci::browser_publication::tests::changed_missing_oversized_chunked_and_non_success_responses_fail",
    "oci::browser_publication::tests::descendant_holding_stdout_cannot_extend_the_deadline",
    "oci::browser_publication::tests::every_redirect_is_rejected_without_contacting_its_destination",
    "oci::browser_publication::tests::invalid_local_release_is_refused_before_any_https_request",
    "oci::browser_publication::tests::production_command_has_no_redirect_authentication_or_trust_bypass",
    "oci::browser_publication::tests::receipt_contract_rejects_inconsistent_identities_profiles_and_targets",
    "oci::browser_publication::tests::source_free_release_binds_complete_observation_and_rejects_graph_mutation",
    "oci::browser_publication::tests::stalled_response_is_killed_at_the_independent_deadline",
    "oci::browser_publication::tests::target_grammar_rejects_aliases_credentials_and_cross_origin_spelling",
    "oci::browser_publication::tests::tls_transport_observes_exact_binary_and_empty_responses",
    "oci::browser_publication::tests::untrusted_tls_and_ambient_transport_overrides_are_rejected",
];

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
            "browser_publication",
            "--",
            "--nocapture",
            "--test-threads=1",
        ])
        .output()
        .expect("complete publication-integrity owning tests execute");
    let stdout = String::from_utf8(output.stdout).expect("owning test stdout is UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("owning test stderr is UTF-8");
    assert!(output.status.success(), "{stdout}\n{stderr}");
    let actual = stdout
        .lines()
        .filter_map(|line| {
            line.strip_prefix("test ")
                .and_then(|line| line.strip_suffix(" ... ok"))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        BTreeSet::from(TESTS),
        "owning test names cannot omit, substitute or ignore a gate"
    );
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.starts_with("test result:"))
            .count(),
        1
    );
    assert!(
        stdout.contains("test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured;"),
        "{stdout}"
    );
}
