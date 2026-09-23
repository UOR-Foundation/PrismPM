//! Integration tests for SDK security and advisory disposition (Issue #15).
//!
//! Validates:
//! 1. Complete multi-platform SDK security disposition verification (`prismpm/sdk-security-disposition/1`).
//! 2. Binding of source lock, installed dependency graph, runtime bytes, launcher, and platform inventories.
//! 3. Strict rejection of component-only advisory evidence as a substitute for full shipped SDK disposition.
//! 4. Enforcement of advisory freshness policy: maximum age bounds and expired database rejection.
//! 5. Zero unresolved rejected findings enforcement for production release policy.
//! 6. Fail-closed behavior on tampered digests, duplicate entries, and malformed inputs.

use prismpm::sdk::{
    validate_sdk_security_disposition, GraphBinding, LauncherBinding, LockBinding,
    PlatformInventoryBinding, RuntimeBinding, SdkAdvisoryPolicy, SdkSecurityDisposition,
};
use prismpm::supply_chain::AdvisoryScanFact;

fn sha256_hex(c: char) -> String {
    c.to_string().repeat(64)
}

fn sha256_digest(c: char) -> String {
    format!("sha256:{}", sha256_hex(c))
}

fn valid_disposition(now_unix: u64) -> SdkSecurityDisposition {
    SdkSecurityDisposition {
        schema: "prismpm/sdk-security-disposition/1".to_owned(),
        source_locks: vec![
            LockBinding {
                path: "standards.lock".to_owned(),
                digest: sha256_digest('1'),
            },
            LockBinding {
                path: "prismpm.lock".to_owned(),
                digest: sha256_digest('2'),
            },
        ],
        installed_graph: GraphBinding {
            lockfile_path: "sdk/asyncapi-runtime/package-lock.json".to_owned(),
            lockfile_digest: sha256_digest('3'),
            installed_tree_digest: sha256_digest('4'),
        },
        runtime_bytes: RuntimeBinding {
            parser_version: "@asyncapi/parser/3.6.0".to_owned(),
            runtime_lock_digest: sha256_digest('5'),
            tree_digest: sha256_digest('6'),
        },
        launcher: LauncherBinding {
            path: "/usr/local/bin/asyncapi-official".to_owned(),
            digest: sha256_digest('7'),
        },
        platform_inventories: vec![
            PlatformInventoryBinding {
                platform: "linux/amd64".to_owned(),
                sdk_image_digest: sha256_digest('a'),
                inventory_digest: sha256_digest('b'),
            },
            PlatformInventoryBinding {
                platform: "linux/arm64".to_owned(),
                sdk_image_digest: sha256_digest('c'),
                inventory_digest: sha256_digest('d'),
            },
        ],
        scan_facts: vec![
            AdvisoryScanFact {
                subject_digest: sha256_digest('a'),
                subject_kind: "sdk-image".to_owned(),
                database_id: "OSV-LOCKED-DB-2026".to_owned(),
                database_digest: sha256_digest('e'),
                scanned_at_unix: now_unix - 3600,
                database_expires_unix: now_unix + 86400 * 7,
                rejected_findings: 0,
                result_digest: sha256_digest('f'),
            },
            AdvisoryScanFact {
                subject_digest: sha256_digest('c'),
                subject_kind: "sdk-image".to_owned(),
                database_id: "OSV-LOCKED-DB-2026".to_owned(),
                database_digest: sha256_digest('e'),
                scanned_at_unix: now_unix - 3600,
                database_expires_unix: now_unix + 86400 * 7,
                rejected_findings: 0,
                result_digest: sha256_digest('0'),
            },
        ],
        policy: SdkAdvisoryPolicy {
            database_id: "OSV-LOCKED-DB-2026".to_owned(),
            database_digest: sha256_digest('e'),
            database_expires_unix: now_unix + 86400 * 7,
            max_age_seconds: 604_800,
            require_full_sdk_scan: true,
            allowed_rejected_findings: 0,
        },
    }
}

#[test]
fn valid_sdk_security_disposition_passes_and_produces_receipt() {
    let now = 1_789_000_000;
    let disposition = valid_disposition(now);
    let receipt =
        validate_sdk_security_disposition(&disposition, now).expect("valid disposition passes");

    assert_eq!(
        receipt["schema"],
        "prismpm/sdk-security-disposition-receipt/1"
    );
    assert_eq!(receipt["result"], "verified");
    assert_eq!(receipt["platform_count"], 2);
    assert_eq!(receipt["unresolved_findings"], 0);
    assert_eq!(receipt["verified_at_unix"], now);
    let platforms = receipt["platforms"].as_array().expect("platforms list");
    assert!(platforms.contains(&serde_json::json!("linux/amd64")));
    assert!(platforms.contains(&serde_json::json!("linux/arm64")));
}

#[test]
fn component_only_advisory_evidence_is_rejected_as_substitute_for_shipped_sdk() {
    let now = 1_789_000_000;
    let mut disposition = valid_disposition(now);

    // Replace full SDK image scan facts with component-only / package-only scan facts
    disposition.scan_facts = vec![AdvisoryScanFact {
        subject_digest: sha256_digest('9'),
        subject_kind: "component-image".to_owned(),
        database_id: "OSV-LOCKED-DB-2026".to_owned(),
        database_digest: sha256_digest('e'),
        scanned_at_unix: now - 3600,
        database_expires_unix: now + 86400 * 7,
        rejected_findings: 0,
        result_digest: sha256_digest('8'),
    }];

    let error = validate_sdk_security_disposition(&disposition, now)
        .expect_err("component-only evidence must fail closed");
    assert_eq!(error.code, "PP7801");
    assert!(
        error.message.contains(
            "component-only advisory evidence cannot substitute for full shipped SDK disposition"
        ),
        "Expected anti-substitution message, got: {error}"
    );
}

#[test]
fn missing_platform_identity_fails_closed() {
    let now = 1_789_000_000;

    // 1. Missing arm64 in platform_inventories
    let mut missing_platform = valid_disposition(now);
    missing_platform.platform_inventories.pop();
    let err1 = validate_sdk_security_disposition(&missing_platform, now)
        .expect_err("missing platform inventory fails");
    assert_eq!(err1.code, "PP7801");
    assert!(err1.message.contains("omits required platform linux/arm64"));

    // 2. Platform inventory present, but scan facts only cover amd64
    let mut missing_scan = valid_disposition(now);
    missing_scan.scan_facts.pop();
    let err2 = validate_sdk_security_disposition(&missing_scan, now)
        .expect_err("uncovered platform scan fails");
    assert_eq!(err2.code, "PP7801");
    assert!(err2
        .message
        .contains("advisory scans do not cover every shipped SDK platform identity"));
}

#[test]
fn advisory_freshness_policy_rejects_stale_or_future_evidence() {
    let now = 1_789_000_000;

    // 1. Stale scan exceeding max_age_seconds (604,800s = 7 days)
    let mut stale_scan = valid_disposition(now);
    stale_scan.scan_facts[0].scanned_at_unix = now - 604_801;
    let err_stale = validate_sdk_security_disposition(&stale_scan, now)
        .expect_err("stale scan evidence must fail closed");
    assert_eq!(err_stale.code, "PP7801");
    assert!(err_stale
        .message
        .contains("exceeds maximum permitted age under freshness policy"));

    // 2. Future-dated scan timestamp
    let mut future_scan = valid_disposition(now);
    future_scan.scan_facts[0].scanned_at_unix = now + 100;
    let err_future = validate_sdk_security_disposition(&future_scan, now)
        .expect_err("future-dated scan must fail closed");
    assert_eq!(err_future.code, "PP7801");
    assert!(err_future
        .message
        .contains("advisory scan timestamp is in the future"));

    // 3. Expired advisory database
    let mut expired_db = valid_disposition(now);
    expired_db.policy.database_expires_unix = now - 1;
    let err_expired = validate_sdk_security_disposition(&expired_db, now)
        .expect_err("expired database must fail closed");
    assert_eq!(err_expired.code, "PP7801");
    assert!(err_expired
        .message
        .contains("advisory database has expired under production freshness policy"));
}

#[test]
fn unresolved_rejected_findings_fail_closed() {
    let now = 1_789_000_000;
    let mut with_findings = valid_disposition(now);
    with_findings.scan_facts[0].rejected_findings = 1;

    let error = validate_sdk_security_disposition(&with_findings, now)
        .expect_err("unresolved findings must fail closed");
    assert_eq!(error.code, "PP7801");
    assert!(error
        .message
        .contains("advisory scan contains unresolved rejected findings"));
}

#[test]
fn tampered_identities_and_malformed_digests_fail_closed() {
    let now = 1_789_000_000;

    // Bad schema
    let mut bad_schema = valid_disposition(now);
    bad_schema.schema = "prismpm/sdk-security-disposition/2".to_owned();
    assert_eq!(
        validate_sdk_security_disposition(&bad_schema, now)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // Malformed source lock digest
    let mut bad_lock = valid_disposition(now);
    bad_lock.source_locks[0].digest = "not-a-sha256".to_owned();
    assert_eq!(
        validate_sdk_security_disposition(&bad_lock, now)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // Malformed installed graph digest
    let mut bad_graph = valid_disposition(now);
    bad_graph.installed_graph.installed_tree_digest = "sha256:1234".to_owned();
    assert_eq!(
        validate_sdk_security_disposition(&bad_graph, now)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // Empty launcher path
    let mut bad_launcher = valid_disposition(now);
    bad_launcher.launcher.path = "   ".to_owned();
    assert_eq!(
        validate_sdk_security_disposition(&bad_launcher, now)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // Database mismatch
    let mut db_mismatch = valid_disposition(now);
    db_mismatch.scan_facts[0].database_id = "WRONG-DB-ID".to_owned();
    assert_eq!(
        validate_sdk_security_disposition(&db_mismatch, now)
            .unwrap_err()
            .code,
        "PP7801"
    );

    // Relaxed policy attempting to bypass full scan requirement
    let mut relaxed_policy = valid_disposition(now);
    relaxed_policy.policy.require_full_sdk_scan = false;
    assert_eq!(
        validate_sdk_security_disposition(&relaxed_policy, now)
            .unwrap_err()
            .code,
        "PP7801"
    );
}
