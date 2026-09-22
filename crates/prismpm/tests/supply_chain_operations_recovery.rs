//! Integration test suite for supply-chain, operations, and recovery evidence (Task 9 / Issue #11).
//!
//! Validates:
//! 1. SPDX 3.0.1 BOM closure over the release graph and relationship validation.
//! 2. In-toto Statement v1 / SLSA Provenance v1 generation and policy enforcement.
//! 3. Sigstore CI identity bundle verification and strict policy rejection.
//! 4. Vulnerability/advisory coverage, freshness bounds, and unpinned DB rejection.
//! 5. OpenTelemetry and operational signal redaction with exact release-digest binding.
//! 6. Backup, restore, and disaster recovery evidence pipeline with fail-closed bounds.

use prismpm::oci::Descriptor;
use prismpm::operations::{model_evidence, redact};
use prismpm::supply_chain::{
    provenance_statement, validate_advisory_coverage, validate_provenance_policy,
    validate_sbom_closure, AdvisoryPolicy, AdvisoryScanFact, PromotionPolicy, ProvenanceInputs,
    VerifiedSignature, PRISM_BUILD_TYPE,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn sha256_hex(c: char) -> String {
    c.to_string().repeat(64)
}

fn sha256_digest(c: char) -> String {
    format!("sha256:{}", sha256_hex(c))
}

#[test]
fn spdx_sbom_closure_validates_graph_and_rejects_gaps_and_dangling_references() {
    let layer_digest = sha256_digest('1');
    let descriptor = Descriptor {
        media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_owned(),
        digest: layer_digest.clone(),
        size: 1024,
        artifact_type: None,
        annotations: Some(BTreeMap::new()),
    };

    let valid_spdx = json!({
        "@context": "https://spdx.org/rdf/3.0.1/spdx-context.jsonld",
        "@graph": [
            {
                "creationInfo": {
                    "created": "1970-01-01T00:00:00Z",
                    "createdBy": ["urn:spdx:prismpm:tool"],
                    "specVersion": "3.0.1",
                    "type": "CreationInfo"
                },
                "spdxId": "urn:spdx:prismpm:document",
                "type": "SpdxDocument"
            },
            {
                "name": "release-layer",
                "spdxId": "urn:spdx:prismpm:package:layer",
                "summary": "release-artifact",
                "type": "software_Package",
                "verifiedUsing": [{
                    "algorithm": "sha256",
                    "hashValue": sha256_hex('1'),
                    "type": "Hash"
                }]
            },
            {
                "from": "urn:spdx:prismpm:document",
                "relationshipType": "describes",
                "spdxId": "urn:spdx:prismpm:rel:1",
                "to": ["urn:spdx:prismpm:package:layer"],
                "type": "Relationship"
            }
        ]
    });

    // Valid graph succeeds
    validate_sbom_closure(&valid_spdx, std::slice::from_ref(&descriptor), &[])
        .expect("valid SPDX closure");

    // 1. Missing @graph fails closed with PP7801
    let bad_graph = json!({"documents": []});
    assert_eq!(
        validate_sbom_closure(&bad_graph, std::slice::from_ref(&descriptor), &[])
            .unwrap_err()
            .code,
        "PP7801"
    );

    // 2. Duplicate element spdxId fails closed with PP7801
    let mut dup_id = valid_spdx.clone();
    dup_id["@graph"].as_array_mut().unwrap().push(json!({
        "spdxId": "urn:spdx:prismpm:package:layer",
        "type": "software_Package"
    }));
    assert_eq!(
        validate_sbom_closure(&dup_id, std::slice::from_ref(&descriptor), &[])
            .unwrap_err()
            .code,
        "PP7801"
    );

    // 3. Dangling relationship target fails closed with PP7801
    let mut dangling = valid_spdx.clone();
    dangling["@graph"][2]["to"] = json!(["urn:spdx:prismpm:missing:target"]);
    assert_eq!(
        validate_sbom_closure(&dangling, std::slice::from_ref(&descriptor), &[])
            .unwrap_err()
            .code,
        "PP7801"
    );

    // 4. Missing required OCI layer descriptor fails closed with PP7801
    let missing_descriptor = Descriptor {
        media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_owned(),
        digest: sha256_digest('2'),
        size: 2048,
        artifact_type: None,
        annotations: Some(BTreeMap::new()),
    };
    assert_eq!(
        validate_sbom_closure(&valid_spdx, &[descriptor, missing_descriptor], &[])
            .unwrap_err()
            .code,
        "PP7801"
    );
}

#[test]
fn intoto_slsa_provenance_generation_and_policy_validation() {
    let release_digest = sha256_digest('a');
    let builder_id = "https://github.com/actions/runner-controller/builder@v1";
    let source_repo = "https://github.com/UOR-Foundation/PrismPM";
    let source_sha = sha256_hex('b');

    let inputs = ProvenanceInputs {
        subject_name: "prism-release".to_owned(),
        subject_digest: release_digest.clone(),
        builder_id: builder_id.to_owned(),
        invocation_id: "https://github.com/UOR-Foundation/PrismPM/actions/runs/12345".to_owned(),
        source_uri: source_repo.to_owned(),
        source_revision: source_sha.clone(),
        external_parameters: json!({"entrypoint": "main", "profile": "release"}),
        dependencies: vec![("docker.io/library/alpine".to_owned(), sha256_digest('c'))],
    };

    let statement_bytes = provenance_statement(&inputs).expect("generate provenance statement");
    let statement: Value = serde_json::from_slice(&statement_bytes).expect("parse statement JSON");

    // Standard envelope and predicate structure
    assert_eq!(statement["_type"], "https://in-toto.io/Statement/v1");
    assert_eq!(statement["predicateType"], "https://slsa.dev/provenance/v1");
    assert_eq!(
        statement.pointer("/predicate/buildDefinition/buildType"),
        Some(&json!(PRISM_BUILD_TYPE))
    );
    assert_eq!(
        statement.pointer("/subject/0/digest/sha256"),
        Some(&json!(sha256_hex('a')))
    );

    // Promotion policy enforcement
    let policy = PromotionPolicy {
        schema: "prismpm/promotion-policy/1".to_owned(),
        subject_name: "prism-release".to_owned(),
        release_digest: release_digest.clone(),
        builder_id: builder_id.to_owned(),
        source_uri: source_repo.to_owned(),
        source_revision: source_sha.clone(),
        external_parameters: json!({"entrypoint": "main", "profile": "release"}),
        required_dependencies: vec![("docker.io/library/alpine".to_owned(), sha256_digest('c'))],
        issuer: "https://token.actions.githubusercontent.com".to_owned(),
        identity_subject: "https://github.com/UOR-Foundation/PrismPM/.github/workflows/release.yml@refs/tags/v0.3.0".to_owned(),
        repository: "UOR-Foundation/PrismPM".to_owned(),
        workflow: "release.yml".to_owned(),
        git_ref: "refs/tags/v0.3.0".to_owned(),
        environment: "production".to_owned(),
        require_transparency_log: true,
        require_hosted_build: true,
        trusted_root_digest: sha256_digest('f'),
    };

    let signature = VerifiedSignature {
        subject_digest: release_digest.clone(),
        issuer: policy.issuer.clone(),
        identity_subject: policy.identity_subject.clone(),
        repository: policy.repository.clone(),
        workflow: policy.workflow.clone(),
        git_ref: policy.git_ref.clone(),
        environment: policy.environment.clone(),
        transparency_verified: true,
        hosted_build: true,
    };

    // Valid statement with matching verified signature accepted
    let accepted = validate_provenance_policy(&statement, Some(&signature), &policy);
    assert!(accepted.is_ok());

    // Without verified signature, rejected with PP7401 (unsigned evidence cannot be promoted)
    assert_eq!(
        validate_provenance_policy(&statement, None, &policy)
            .unwrap_err()
            .code,
        "PP7401"
    );

    // 1. Mismatched subject name rejected with PP7401
    let mut bad_subject = statement.clone();
    bad_subject["subject"][0]["name"] = json!("wrong-product");
    assert_eq!(
        validate_provenance_policy(&bad_subject, Some(&signature), &policy)
            .unwrap_err()
            .code,
        "PP7401"
    );

    // 2. Mismatched subject digest rejected with PP7401
    let mut bad_digest = statement.clone();
    bad_digest["subject"][0]["digest"]["sha256"] = json!(sha256_hex('9'));
    assert_eq!(
        validate_provenance_policy(&bad_digest, Some(&signature), &policy)
            .unwrap_err()
            .code,
        "PP7401"
    );

    // 3. Mismatched builder ID rejected with PP7401
    let mut bad_builder = statement.clone();
    bad_builder["predicate"]["runDetails"]["builder"]["id"] = json!("https://untrusted.builder/v1");
    assert_eq!(
        validate_provenance_policy(&bad_builder, Some(&signature), &policy)
            .unwrap_err()
            .code,
        "PP7401"
    );
}

#[test]
fn advisory_coverage_enforces_completeness_freshness_and_unpinned_rejection() {
    let subject_a = sha256_digest('1');
    let subject_b = sha256_digest('2');
    let policy = AdvisoryPolicy {
        required_subjects: vec![subject_a.clone(), subject_b.clone()],
        database_id: "OSV-LOCKED-DB-2026".to_owned(),
        database_digest: sha256_digest('d'),
        max_age_seconds: 3600,
    };

    let valid_facts = vec![
        AdvisoryScanFact {
            subject_digest: subject_a.clone(),
            subject_kind: "container-image".to_owned(),
            database_id: policy.database_id.clone(),
            database_digest: policy.database_digest.clone(),
            scanned_at_unix: 9500,
            database_expires_unix: 20000,
            rejected_findings: 0,
            result_digest: sha256_digest('a'),
        },
        AdvisoryScanFact {
            subject_digest: subject_b.clone(),
            subject_kind: "sdk-binary".to_owned(),
            database_id: policy.database_id.clone(),
            database_digest: policy.database_digest.clone(),
            scanned_at_unix: 9600,
            database_expires_unix: 20000,
            rejected_findings: 0,
            result_digest: sha256_digest('b'),
        },
    ];

    // Current evaluation time: 10000 (age 400-500s <= 3600s, expires at 20000)
    let coverage = validate_advisory_coverage(&valid_facts, &policy, 10000);
    assert!(coverage.is_ok());

    // 1. Incomplete coverage (missing subject_b) rejected with PP7801
    assert_eq!(
        validate_advisory_coverage(&valid_facts[..1], &policy, 10000)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // 2. Expired scan (evaluated at 15000: age 5400-5500s > 3600s) rejected with PP7801
    assert_eq!(
        validate_advisory_coverage(&valid_facts, &policy, 15000)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // 3. Expired database (evaluated at 25000 > 20000) rejected with PP7801
    assert_eq!(
        validate_advisory_coverage(&valid_facts, &policy, 25000)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // 4. Mismatched database ID rejected with PP7801
    let mut wrong_db = valid_facts.clone();
    wrong_db[0].database_id = "unpinned-live-db".to_owned();
    assert_eq!(
        validate_advisory_coverage(&wrong_db, &policy, 10000)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // 5. Findings with rejected count > 0 fail closed with PP7801
    let mut with_cve = valid_facts.clone();
    with_cve[0].rejected_findings = 1;
    assert_eq!(
        validate_advisory_coverage(&with_cve, &policy, 10000)
            .unwrap_err()
            .code,
        "PP7801"
    );
}

#[test]
fn telemetry_redaction_and_operational_slo_evidence_pipeline() {
    let model = json!({
        "alerts": [{"id": "error-rate-alert"}],
        "components": [{"health": "/healthz", "id": "calculator-api"}],
        "observability": {
            "alerts": ["error-rate-alert"],
            "logs": "otlp",
            "metrics": "otlp",
            "redacted_fields": ["authorization", "password", "token", "secret"],
            "slos": ["availability-slo"],
            "traces": "otlp"
        },
        "slis": [{"id": "availability-sli"}],
        "slos": [{"id": "availability-slo"}]
    });

    // Redaction: sensitive fields redacted, external safe identifiers preserved
    let mut observation = json!({
        "authorization": "Bearer secret-credential-value",
        "password": "correct-horse-battery-staple",
        "token": "ghp_1234567890abcdef",
        "secret": "db-secret",
        "status": "healthy",
        "version": "v0.3.0",
        "reference": "secret://external/ref"
    });

    redact(&model, &mut observation).expect("redact");
    assert_eq!(observation["authorization"], "[REDACTED]");
    assert_eq!(observation["password"], "[REDACTED]");
    assert_eq!(observation["token"], "[REDACTED]");
    assert_eq!(observation["secret"], "[REDACTED]");
    assert_eq!(observation["status"], "healthy");
    assert_eq!(observation["version"], "v0.3.0");
    assert_eq!(observation["reference"], "secret://external/ref");

    // Operational SLO evidence binds exact release digest
    let release_digest = sha256_digest('e');
    let evidence = model_evidence(&model, &release_digest).expect("operational evidence");
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0]["id"], "operations-model");
    assert_eq!(evidence[0]["status"], "passed");
    assert_eq!(evidence[0]["kind"], "contract");
    assert!(evidence[0]["evidence_digest"].as_str().is_some());

    // Empty or missing SLOs in model fail closed with PP7401
    let mut incomplete_model = model;
    incomplete_model["slos"] = json!([]);
    assert_eq!(
        model_evidence(&incomplete_model, &release_digest)
            .unwrap_err()
            .code,
        "PP7401"
    );
}

#[test]
fn disaster_recovery_lifecycle_fails_closed_on_invalid_inputs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let valid_ref = "example.test/product@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // 1. Backup with invalid target ID fails closed with PP7101
    for bad_target in ["", "InvalidTarget", "target/with/slash"] {
        let err = prismpm::lifecycle::backup(root, valid_ref, bad_target).unwrap_err();
        assert_eq!(err.code, "PP7101");
    }

    // 2. Restore with invalid source target ID fails closed with PP7101
    for bad_target in ["", "InvalidTarget", "target/with/slash"] {
        let err = prismpm::lifecycle::restore(
            root,
            valid_ref,
            bad_target,
            "restore-target",
            &root.join("missing.sql"),
        )
        .unwrap_err();
        assert_eq!(err.code, "PP7101");
    }

    // 3. Restore with invalid restore target ID fails closed with PP7101
    for bad_target in ["", "InvalidTarget", "target/with/slash"] {
        let err = prismpm::lifecycle::restore(
            root,
            valid_ref,
            "source-target",
            bad_target,
            &root.join("missing.sql"),
        )
        .unwrap_err();
        assert_eq!(err.code, "PP7101");
    }

    // 4. Restore with unmanifested release reference fails closed with PP6101
    let err = prismpm::lifecycle::restore(
        root,
        valid_ref,
        "source-target",
        "restore-target",
        &root.join("missing.sql"),
    )
    .unwrap_err();
    assert_eq!(err.code, "PP6101");

    // 5. Backup with mutable or non-digest reference fails closed with PP6101
    for bad_ref in ["example.test/product:latest", "product@sha256:short"] {
        let err = prismpm::lifecycle::backup(root, bad_ref, "local").unwrap_err();
        assert_eq!(err.code, "PP6101");
    }
}
