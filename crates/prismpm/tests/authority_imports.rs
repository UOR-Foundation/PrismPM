//! Integration tests for immutable authority/oracle imports and verification.

use prismpm::authority::{inspect, resolve, run_oracle};
use prismpm::contracts::CanonicalDocument;
use prismpm::holo::canonical::encode_value;
use serde_json::json;
use std::io::Write;

#[test]
fn test_authority_resolve_inspect_and_locked_drift_rejection() {
    let root = tempfile::tempdir().expect("temp root");

    // 1. Initially, --locked resolve must fail if standards.lock is absent
    let locked_err = resolve(root.path(), true).unwrap_err();
    assert_eq!(locked_err.code.as_str(), "PP1101");

    // 2. Unlocked resolve succeeds and writes canonical standards.lock
    let res = resolve(root.path(), false).expect("unlocked resolve succeeds");
    assert_eq!(res.schema, "prismpm/authority-result/1");
    assert_eq!(res.path, "standards.lock");
    assert!(res.authorities > 0);
    assert!(res.oracles > 0);
    assert!(!res.unchanged);

    // 3. Subsequent --locked resolve succeeds and reports unchanged
    let locked_res = resolve(root.path(), true).expect("locked resolve succeeds when unchanged");
    assert!(locked_res.unchanged);

    // 4. Inspect validates without network or execution
    let inspected = inspect(root.path()).expect("inspect succeeds on valid lock");
    assert_eq!(inspected["schema"], "prismpm/standards-lock/1");

    // Verify canonical contract of generated standards.lock
    let lock_bytes = std::fs::read(root.path().join("standards.lock")).unwrap();
    let doc = CanonicalDocument::parse("prismpm/standards-lock/1", &lock_bytes)
        .expect("standards.lock validates against prismpm/standards-lock/1");
    let val = doc.value();

    let authorities = val["authorities"].as_array().expect("authorities array");
    for auth in authorities {
        assert_eq!(auth["schema"], "prismpm/authority-binding/1");
        assert!(auth["id"].as_str().is_some());
        assert!(auth["canonical_id"].as_str().is_some());
        assert!(auth["edition"].as_str().is_some());
        assert!(auth["issuer"].as_str().is_some());
        assert!(auth["name"].as_str().is_some());
        assert_eq!(auth["supersession"], "explicit-lock-update-only");
    }

    let oracles = val["oracles"].as_array().expect("oracles array");
    for oracle in oracles {
        assert!(oracle["id"].as_str().is_some());
        assert!(oracle["edition"].as_str().is_some());
        assert!(oracle["executable"].as_str().is_some());
        assert_eq!(oracle["output_schema"], "prismpm/validation-result/1");
        assert_eq!(oracle["normalization"], "prismpm/oracle-diagnostics/1");
    }

    // 5. Tampered standards.lock causes inspect and --locked resolve to reject drift
    std::fs::write(root.path().join("standards.lock"), b"{\"tampered\": true}").unwrap();
    assert!(inspect(root.path()).is_err());
    let drift_err = resolve(root.path(), true).unwrap_err();
    assert_eq!(drift_err.code.as_str(), "PP1101");
}

#[test]
fn test_openapi_oracle_verification_and_planted_defect() {
    let valid = json!({
        "info": {
            "title": "Calculator Service",
            "version": "1.0.0"
        },
        "openapi": "3.2.0",
        "paths": {}
    });

    let mut valid_file = tempfile::NamedTempFile::new().unwrap();
    valid_file
        .write_all(&encode_value(&valid).unwrap())
        .unwrap();
    let res = run_oracle("openapi", valid_file.path()).expect("valid OpenAPI 3.2 doc passes");
    assert!(res.valid);
    assert_eq!(res.oracle, "openapi-3.2-schema");

    // Planted defect: unknown root field
    let invalid = json!({
        "info": {
            "title": "Calculator Service",
            "version": "1.0.0"
        },
        "openapi": "3.2.0",
        "paths": {},
        "planted_defect_probe": true
    });

    let mut invalid_file = tempfile::NamedTempFile::new().unwrap();
    invalid_file
        .write_all(&encode_value(&invalid).unwrap())
        .unwrap();
    let err = run_oracle("openapi", invalid_file.path()).unwrap_err();
    assert_eq!(err.code.as_str(), "PP5404");
}
