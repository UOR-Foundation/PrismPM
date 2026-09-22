//! Comprehensive contract validation tests for Task 2 production-system schemas.

use prismpm::contracts::CanonicalDocument;
use serde_json::json;
use std::path::Path;

fn dummy_sha256(c: char) -> String {
    format!("sha256:{}", c.to_string().repeat(64))
}

#[test]
fn test_task_2_schemas_reject_unknown_fields() {
    let cases = vec![
        (
            "prismpm/product-release/1",
            json!({
                "artifacts": [{
                    "annotations": {},
                    "digest": dummy_sha256('a'),
                    "media_type": "application/octet-stream",
                    "role": "application",
                    "size": 1
                }],
                "external_artifacts": [{
                    "digest": dummy_sha256('b'),
                    "license_expression": "Apache-2.0",
                    "media_type": "application/vnd.docker.distribution.manifest.list.v2+json",
                    "reference": format!("registry.example/product@{}", dummy_sha256('b')),
                    "role": "runtime-image"
                }],
                "model_digest": dummy_sha256('d'),
                "product": "fixture",
                "release": "1",
                "schema": "prismpm/product-release/1",
                "sdk_digest": dummy_sha256('e'),
                "sdk_lock": dummy_sha256('1'),
                "standards_lock": dummy_sha256('f'),
                "status": "development"
            }),
        ),
        (
            "prismpm/product-release-result/1",
            json!({
                "build_digest": dummy_sha256('a'),
                "evidence_path": "evidence.json",
                "model_digest": dummy_sha256('b'),
                "product_digest": dummy_sha256('c'),
                "reference": format!("registry.example/product@{}", dummy_sha256('c')),
                "release_digest": dummy_sha256('d'),
                "schema": "prismpm/product-release-result/1"
            }),
        ),
        (
            "prismpm/deployment-plan/1",
            json!({
                "authorization": "satisfied",
                "changes": [{
                    "action": "create",
                    "downtime": false,
                    "id": "app-service",
                    "risk": "none",
                    "rollback": "delete app-service"
                }],
                "desired_digest": dummy_sha256('a'),
                "expires_after_state": dummy_sha256('b'),
                "observed_digest": dummy_sha256('c'),
                "plan_digest": dummy_sha256('d'),
                "release_digest": dummy_sha256('e'),
                "schema": "prismpm/deployment-plan/1",
                "secret_references": ["SECRET_KEY"],
                "target": "prod-us-east"
            }),
        ),
        (
            "prismpm/deployment-state/1",
            json!({
                "desired_digest": dummy_sha256('a'),
                "drift": "none",
                "last_applied_digest": dummy_sha256('b'),
                "migration_phase": "contract-finalized",
                "migration_release_digest": dummy_sha256('c'),
                "observed_digest": dummy_sha256('d'),
                "ready": true,
                "release_digest": dummy_sha256('e'),
                "resources": {
                    "deployment/app": dummy_sha256('f')
                },
                "schema": "prismpm/deployment-state/1",
                "target": "prod-us-east",
                "target_profile": "kubernetes-ha"
            }),
        ),
        (
            "prismpm/deployment-evidence/1",
            json!({
                "checks": [{
                    "evidence_digest": dummy_sha256('a'),
                    "id": "health-check",
                    "kind": "readiness",
                    "status": "passed"
                }],
                "evidence_digest": dummy_sha256('b'),
                "observed_state": dummy_sha256('c'),
                "operation": "deploy",
                "plan_digest": dummy_sha256('d'),
                "release_digest": dummy_sha256('e'),
                "schema": "prismpm/deployment-evidence/1",
                "status": "accepted",
                "target": "prod-us-east"
            }),
        ),
    ];

    for (schema_id, valid_doc) in cases {
        let doc = CanonicalDocument::from_value(schema_id, valid_doc.clone())
            .unwrap_or_else(|e| panic!("valid document failed for {}: {}", schema_id, e));
        assert_eq!(doc.schema(), schema_id);

        let mut invalid_doc = valid_doc;
        invalid_doc["unknown_field_probe"] = json!("forbidden");
        assert!(
            CanonicalDocument::from_value(schema_id, invalid_doc).is_err(),
            "schema {} did not reject unknown property",
            schema_id
        );
    }
}

#[test]
fn test_calculator_baseline_compatibility_if_present() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let baseline_path = manifest_dir.join("../../tests/data/calculator-baseline.json");
    if baseline_path.exists() {
        let baseline_bytes = std::fs::read(&baseline_path).expect("read calculator baseline");
        let baseline_val: serde_json::Value =
            serde_json::from_slice(&baseline_bytes).expect("parse calculator baseline");
        let doc = CanonicalDocument::from_value("prismpm/calculator-baseline/1", baseline_val)
            .expect("calculator baseline validates against schema");
        assert_eq!(doc.schema(), "prismpm/calculator-baseline/1");
    }
}
