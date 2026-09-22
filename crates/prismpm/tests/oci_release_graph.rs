//! Verification and independent acceptance evidence for Task 6: Implement OCI
//! product-release graph and registry lifecycle (OC-01..OC-07).

use jsonschema::validator_for;
use prismpm::oci;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate is below repository root")
        .to_path_buf()
}

#[test]
fn registered_prism_vendor_media_types_closure() {
    // Only minimal Prism-owned document media types and standard OCI types are registered
    assert_eq!(
        oci::PRISM_RELEASE,
        "application/vnd.prismpm.product.release.v1+json"
    );
    assert_eq!(
        oci::PRISM_VALIDATION,
        "application/vnd.prismpm.validation.v1+json"
    );
    assert_eq!(
        oci::PRISM_VERIFICATION,
        "application/vnd.prismpm.verification.v1+json"
    );
    assert_eq!(oci::INTOTO, "application/vnd.in-toto+json");
    assert_eq!(oci::SPDX, "application/spdx+json;version=3.0.1");
    assert_eq!(
        oci::PRISM_SUPPLY_CHAIN,
        "application/vnd.prismpm.supply-chain.v1+json"
    );
    assert_eq!(
        oci::PRISM_PRODUCTION_ACCEPTANCE,
        "application/vnd.prismpm.production.acceptance.v1+json"
    );
    assert_eq!(
        oci::PRISM_EVIDENCE_SIGNATURE,
        "application/vnd.prismpm.evidence.signature.v1+json"
    );
    assert_eq!(
        oci::PRISM_DEPLOYMENT_EVIDENCE,
        "application/vnd.prismpm.deployment.evidence.v1+json"
    );
    assert_eq!(
        oci::PRISM_PROMOTION_POLICY,
        "application/vnd.prismpm.promotion-policy.v1+json"
    );
    assert_eq!(
        oci::PRISM_PROMOTION,
        "application/vnd.prismpm.promotion.v1+json"
    );
    assert_eq!(
        oci::COSIGN_SIGNATURE,
        "application/vnd.dev.cosign.artifact.sig.v1+json"
    );
    assert_eq!(
        oci::COSIGN_SIMPLE_SIGNING,
        "application/vnd.dev.cosign.simplesigning.v1+json"
    );

    // Standard OCI container types
    assert_eq!(
        oci::OCI_MANIFEST,
        "application/vnd.oci.image.manifest.v1+json"
    );
    assert_eq!(oci::OCI_INDEX, "application/vnd.oci.image.index.v1+json");
    assert_eq!(oci::OCI_EMPTY, "application/vnd.oci.empty.v1+json");
}

#[test]
fn oci_reference_validation_and_security_boundaries() {
    // Valid references without digest requirement
    assert!(oci::validate_reference("localhost:5000/product:tag", false).is_ok());
    assert!(oci::validate_reference("ghcr.io/uor-foundation/prismpm:0.3.0", false).is_ok());

    let valid_digest = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let digest_ref = format!("example.test/product@{valid_digest}");
    assert!(oci::validate_reference(&digest_ref, false).is_ok());
    assert!(oci::validate_reference(&digest_ref, true).is_ok());

    // Requiring digest must reject tag references
    let error = oci::validate_reference("localhost:5000/product:tag", true).unwrap_err();
    assert_eq!(error.code, "PP6101");

    // Path traversal / escape must be rejected
    assert!(oci::validate_reference("../escape:tag", false).is_err());
    assert!(oci::validate_reference("..:tag", false).is_err());
    assert!(oci::validate_reference("/absolute:tag", false).is_err());

    // Bare repository name without namespace must be rejected
    assert!(oci::validate_reference("product:latest", false).is_err());

    // Uppercase characters must be rejected per OCI distribution naming specs
    assert!(oci::validate_reference("localhost:5000/Product:tag", false).is_err());
}

#[test]
fn product_release_schema_conformance_and_falsification() {
    let schema_bytes =
        fs::read(root().join("schemas/product-release.schema.json")).expect("read release schema");
    let schema_val: Value = serde_json::from_slice(&schema_bytes).expect("parse schema");
    let validator = validator_for(&schema_val).expect("compile validator");

    let valid_release = json!({
        "schema": "prismpm/product-release/1",
        "product": "calculator",
        "release": "0.3.0",
        "status": "candidate",
        "model_digest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "sdk_digest": "sha256:1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "sdk_lock": "sha256:2123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "standards_lock": "sha256:3123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "artifacts": [
            {
                "digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
                "media_type": "application/vnd.prismpm.holo.artifact.v1+tar",
                "role": "application",
                "size": 1024,
                "annotations": {
                    "org.opencontainers.image.title": "Calculator.holo"
                }
            }
        ],
        "external_artifacts": []
    });

    assert!(
        validator.is_valid(&valid_release),
        "valid product release must pass schema validation"
    );

    // Empty artifacts list fails minItems: 1
    let mut no_artifacts = valid_release.clone();
    no_artifacts["artifacts"] = json!([]);
    assert!(
        !validator.is_valid(&no_artifacts),
        "empty artifacts list must fail validation (minItems: 1)"
    );

    // Negative size fails minimum: 0
    let mut negative_size = valid_release.clone();
    negative_size["artifacts"][0]["size"] = json!(-1);
    assert!(
        !validator.is_valid(&negative_size),
        "negative size must fail validation"
    );

    // Invalid SHA-256 pattern fails
    let mut bad_digest = valid_release.clone();
    bad_digest["artifacts"][0]["digest"] = json!("sha256:not-a-valid-hex");
    assert!(
        !validator.is_valid(&bad_digest),
        "invalid digest pattern must fail validation"
    );

    // Invalid status enum fails
    let mut bad_status = valid_release.clone();
    bad_status["status"] = json!("unapproved");
    assert!(
        !validator.is_valid(&bad_status),
        "invalid status enum value must fail validation"
    );

    // Unknown field fails additionalProperties: false
    let mut extra = valid_release.clone();
    extra["injected_field"] = json!("malicious");
    assert!(
        !validator.is_valid(&extra),
        "unknown property must fail validation"
    );
}

#[test]
fn product_release_result_schema_conformance() {
    let schema_bytes = fs::read(root().join("schemas/product-release-result.schema.json"))
        .expect("read release result schema");
    let schema_val: Value = serde_json::from_slice(&schema_bytes).expect("parse schema");
    let validator = validator_for(&schema_val).expect("compile validator");

    let valid_result = json!({
        "schema": "prismpm/product-release-result/1",
        "reference": "localhost:5000/calculator:0.3.0",
        "product_digest": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "release_digest": "sha256:1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "model_digest": "sha256:2123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "build_digest": "sha256:3123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "evidence_path": ".prism/build/evidence.json"
    });

    assert!(
        validator.is_valid(&valid_result),
        "valid product release result must pass schema validation"
    );

    // Missing required fields must fail
    for field in [
        "schema",
        "reference",
        "product_digest",
        "release_digest",
        "model_digest",
        "build_digest",
        "evidence_path",
    ] {
        let mut missing = valid_result.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            !validator.is_valid(&missing),
            "omitting required field '{field}' must fail validation"
        );
    }

    // Absolute evidence path must fail pattern
    let mut bad_path = valid_result.clone();
    bad_path["evidence_path"] = json!("/absolute/path/evidence.json");
    assert!(
        !validator.is_valid(&bad_path),
        "absolute evidence_path must fail validation"
    );

    // Path traversal in evidence_path must fail pattern
    let mut traversal_path = valid_result.clone();
    traversal_path["evidence_path"] = json!("../escape/evidence.json");
    assert!(
        !validator.is_valid(&traversal_path),
        "path traversal in evidence_path must fail validation"
    );
}

#[test]
fn oci_graph_closure_and_referrers_fail_closed() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = oci::Store::open(temp.path()).expect("open store");

    let subject_digest = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let evidence_bytes = prismpm::holo::canonical::encode_value(&json!({
        "from": "development",
        "signature": { "verified": true },
        "subject": subject_digest,
        "to": "candidate"
    }))
    .expect("canonical encode");

    // Attempting to attach referrer to an unverified subject fails with PP6101
    let err = oci::attach_referrer(
        temp.path(),
        subject_digest,
        oci::PRISM_PROMOTION,
        &evidence_bytes,
    )
    .unwrap_err();
    assert_eq!(
        err.code, "PP6101",
        "unverified subject must be rejected with PP6101"
    );

    // Attempting to attach referrer with unregistered media type fails with PP6101
    let err = oci::attach_referrer(
        temp.path(),
        subject_digest,
        "application/vnd.unregistered.media.type",
        &evidence_bytes,
    )
    .unwrap_err();
    assert_eq!(
        err.code, "PP6101",
        "unregistered media type must be rejected with PP6101"
    );

    // verify_graph on non-existent root manifest fails with PP6101
    let err = oci::verify_graph(&store, subject_digest).unwrap_err();
    assert_eq!(
        err.code, "PP6101",
        "non-existent manifest must fail graph verification"
    );

    // read_descriptor for non-existent blob fails with PP6101
    let desc = oci::Descriptor {
        media_type: oci::OCI_EMPTY.to_owned(),
        digest: subject_digest.to_owned(),
        size: 100,
        artifact_type: None,
        annotations: None,
    };
    let err = oci::read_descriptor(temp.path(), &desc).unwrap_err();
    assert_eq!(err.code, "PP6101", "missing blob must fail descriptor read");
}
