//! First-party crates.io identity bootstrap and trusted-publishing readiness tests.

use prismpm::supply_chain::{
    validate_crates_io_bootstrap, CratesIoBootstrap, CratesIoPackageUpload, DownstreamLockBinding,
    TrustedPublishingConfig,
};

fn sample_checksum(char_byte: u8) -> String {
    let mut s = String::with_capacity(64);
    for _ in 0..64 {
        s.push(char_byte as char);
    }
    s
}

fn sample_commit() -> String {
    "2772411ed127411ed127411ed127411ed127411e".to_owned()
}

fn valid_bootstrap_fixture() -> CratesIoBootstrap {
    let packages = vec![
        CratesIoPackageUpload {
            name: "prod-ir".to_owned(),
            version: "0.1.0".to_owned(),
            checksum: sample_checksum(b'1'),
            crate_bytes_digest: sample_checksum(b'a'),
            source_commit: sample_commit(),
            upload_method: "owner-token".to_owned(),
            uploader: "owner:uor-foundation".to_owned(),
            published_at_unix: 1_000,
        },
        CratesIoPackageUpload {
            name: "prod-codegen".to_owned(),
            version: "0.1.0".to_owned(),
            checksum: sample_checksum(b'2'),
            crate_bytes_digest: sample_checksum(b'b'),
            source_commit: sample_commit(),
            upload_method: "owner-token".to_owned(),
            uploader: "owner:uor-foundation".to_owned(),
            published_at_unix: 1_100,
        },
        CratesIoPackageUpload {
            name: "lexlean".to_owned(),
            version: "0.3.0".to_owned(),
            checksum: sample_checksum(b'3'),
            crate_bytes_digest: sample_checksum(b'c'),
            source_commit: sample_commit(),
            upload_method: "owner-token".to_owned(),
            uploader: "owner:uor-foundation".to_owned(),
            published_at_unix: 1_200,
        },
        CratesIoPackageUpload {
            name: "prism-stdlib".to_owned(),
            version: "0.2.0".to_owned(),
            checksum: sample_checksum(b'4'),
            crate_bytes_digest: sample_checksum(b'd'),
            source_commit: sample_commit(),
            upload_method: "owner-token".to_owned(),
            uploader: "owner:uor-foundation".to_owned(),
            published_at_unix: 1_300,
        },
        CratesIoPackageUpload {
            name: "prismpm".to_owned(),
            version: "0.3.0".to_owned(),
            checksum: sample_checksum(b'5'),
            crate_bytes_digest: sample_checksum(b'e'),
            source_commit: sample_commit(),
            upload_method: "owner-token".to_owned(),
            uploader: "owner:uor-foundation".to_owned(),
            published_at_unix: 1_400,
        },
    ];

    let trusted_publishing = TrustedPublishingConfig {
        enabled: true,
        provider: "github-actions".to_owned(),
        repository: "UOR-Foundation/PrismPM".to_owned(),
        workflow: "release.yml".to_owned(),
        configured_after_bootstrap: true,
    };

    let downstream_locks = vec![
        DownstreamLockBinding {
            lock_path: "Cargo.lock".to_owned(),
            package_name: "prism-stdlib".to_owned(),
            version: "0.2.0".to_owned(),
            checksum: sample_checksum(b'4'),
        },
        DownstreamLockBinding {
            lock_path: "examples/Calculator/Cargo.lock".to_owned(),
            package_name: "prismpm".to_owned(),
            version: "0.3.0".to_owned(),
            checksum: sample_checksum(b'5'),
        },
    ];

    CratesIoBootstrap {
        schema: "prismpm/crates-io-bootstrap/1".to_owned(),
        registry: "https://crates.io".to_owned(),
        packages,
        trusted_publishing,
        downstream_locks,
    }
}

#[test]
fn valid_crates_io_bootstrap_passes_and_produces_receipt() {
    let bootstrap = valid_bootstrap_fixture();
    let receipt = validate_crates_io_bootstrap(&bootstrap, 2_000).expect("bootstrap must pass");

    assert_eq!(receipt["schema"], "prismpm/crates-io-bootstrap-receipt/1");
    assert_eq!(receipt["registry"], "https://crates.io");
    assert_eq!(receipt["package_count"], 5);
    assert_eq!(receipt["downstream_locks_verified"], 2);
    assert_eq!(receipt["trusted_publishing_ready"], true);
    assert_eq!(receipt["result"], "verified");
    assert_eq!(receipt["status"], "passed");
    assert_eq!(receipt["verified_at_unix"], 2_000);
}

#[test]
fn rejects_invalid_schema_or_unsupported_registry() {
    let mut bad_schema = valid_bootstrap_fixture();
    bad_schema.schema = "prismpm/crates-io-bootstrap/2".to_owned();
    let error = validate_crates_io_bootstrap(&bad_schema, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut bad_registry = valid_bootstrap_fixture();
    bad_registry.registry = "https://untrusted-crates.example.com".to_owned();
    let error = validate_crates_io_bootstrap(&bad_registry, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
}

#[test]
fn rejects_missing_duplicate_or_foreign_packages() {
    let mut missing = valid_bootstrap_fixture();
    missing.packages.pop();
    let error = validate_crates_io_bootstrap(&missing, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut duplicate = valid_bootstrap_fixture();
    duplicate.packages[4] = duplicate.packages[0].clone();
    let error = validate_crates_io_bootstrap(&duplicate, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut foreign = valid_bootstrap_fixture();
    foreign.packages[4].name = "some-foreign-crate".to_owned();
    let error = validate_crates_io_bootstrap(&foreign, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
}

#[test]
fn rejects_dependency_order_violations() {
    // prod-codegen before prod-ir
    let mut order1 = valid_bootstrap_fixture();
    order1.packages.swap(0, 1);
    let error = validate_crates_io_bootstrap(&order1, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
    assert!(error
        .to_string()
        .contains("prod-ir must precede prod-codegen"));

    // lexlean before prod-codegen
    let mut order2 = valid_bootstrap_fixture();
    order2.packages.swap(1, 2);
    let error = validate_crates_io_bootstrap(&order2, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
    assert!(error
        .to_string()
        .contains("prod-codegen must precede lexlean"));

    // prismpm before lexlean
    let mut order3 = valid_bootstrap_fixture();
    order3.packages.swap(2, 4);
    let error = validate_crates_io_bootstrap(&order3, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
    assert!(error.to_string().contains("lexlean must precede prismpm"));

    // prismpm before prism-stdlib
    let mut order4 = valid_bootstrap_fixture();
    order4.packages.swap(3, 4);
    let error = validate_crates_io_bootstrap(&order4, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
    assert!(error
        .to_string()
        .contains("prism-stdlib must precede prismpm"));
}

#[test]
fn rejects_malformed_checksums_digests_and_commits() {
    let mut bad_checksum = valid_bootstrap_fixture();
    bad_checksum.packages[0].checksum = "short-checksum".to_owned();
    let error = validate_crates_io_bootstrap(&bad_checksum, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut bad_digest = valid_bootstrap_fixture();
    bad_digest.packages[1].crate_bytes_digest = "not-a-valid-hex-digest".to_owned();
    let error = validate_crates_io_bootstrap(&bad_digest, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut bad_commit = valid_bootstrap_fixture();
    bad_commit.packages[2].source_commit = "not-a-40-hex-commit".to_owned();
    let error = validate_crates_io_bootstrap(&bad_commit, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
}

#[test]
fn rejects_unaided_oidc_bootstrap_shortcut() {
    let mut oidc = valid_bootstrap_fixture();
    oidc.packages[0].upload_method = "unaided-oidc".to_owned();
    let error = validate_crates_io_bootstrap(&oidc, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
    assert!(error
        .to_string()
        .contains("unaided OIDC bootstrap shortcut prohibited"));

    let mut empty_uploader = valid_bootstrap_fixture();
    empty_uploader.packages[1].uploader = "".to_owned();
    let error = validate_crates_io_bootstrap(&empty_uploader, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
}

#[test]
fn rejects_trusted_publishing_before_owner_bootstrap() {
    let mut unverified_tp = valid_bootstrap_fixture();
    unverified_tp.trusted_publishing.enabled = true;
    unverified_tp.trusted_publishing.configured_after_bootstrap = false;
    let error = validate_crates_io_bootstrap(&unverified_tp, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
    assert!(error
        .to_string()
        .contains("trusted publishing configured before owner bootstrap validation"));

    let mut empty_repo = valid_bootstrap_fixture();
    empty_repo.trusted_publishing.repository = "".to_owned();
    let error = validate_crates_io_bootstrap(&empty_repo, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
}

#[test]
fn rejects_downstream_lock_mismatches_and_absence() {
    let mut no_locks = valid_bootstrap_fixture();
    no_locks.downstream_locks.clear();
    let error = validate_crates_io_bootstrap(&no_locks, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut wrong_checksum = valid_bootstrap_fixture();
    wrong_checksum.downstream_locks[0].checksum = sample_checksum(b'9');
    let error = validate_crates_io_bootstrap(&wrong_checksum, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut wrong_version = valid_bootstrap_fixture();
    wrong_version.downstream_locks[0].version = "9.9.9".to_owned();
    let error = validate_crates_io_bootstrap(&wrong_version, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");

    let mut unknown_pkg = valid_bootstrap_fixture();
    unknown_pkg.downstream_locks[0].package_name = "unknown-pkg".to_owned();
    let error = validate_crates_io_bootstrap(&unknown_pkg, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
}

#[test]
fn rejects_future_publication_timestamps() {
    let mut future = valid_bootstrap_fixture();
    future.packages[0].published_at_unix = 3_000;
    let error = validate_crates_io_bootstrap(&future, 2_000).unwrap_err();
    assert_eq!(error.code, "PP4103");
    assert!(error
        .to_string()
        .contains("publication timestamp is in the future"));
}