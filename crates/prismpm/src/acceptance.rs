//! Digest-bound closure of executed production acceptance evidence.

use crate::contracts::CanonicalDocument;
use crate::error::PrismError;
use crate::oci::{self, PRISM_PRODUCTION_ACCEPTANCE};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn release_digest(reference: &str) -> Result<&str, PrismError> {
    let (_, digest) = reference.rsplit_once('@').ok_or_else(|| {
        PrismError::new(
            "PP6002",
            "production acceptance requires an immutable release reference",
        )
    })?;
    let Some(hex) = digest.strip_prefix("sha256:") else {
        return Err(PrismError::new("PP6002", "release digest is malformed"));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(PrismError::new("PP6002", "release digest is malformed"));
    }
    Ok(digest)
}

fn registered_values(
    source: &str,
    table: &str,
    field: &str,
) -> Result<BTreeSet<String>, PrismError> {
    let value: toml::Value = source
        .parse()
        .map_err(|error| PrismError::new("PP9001", format!("acceptance register: {error}")))?;
    value
        .get(table)
        .and_then(toml::Value::as_array)
        .ok_or_else(|| PrismError::new("PP9001", "acceptance register rows are absent"))?
        .iter()
        .map(|row| {
            row.get(field)
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| PrismError::new("PP9001", "acceptance register value is absent"))
        })
        .collect()
}

pub(crate) fn verify_closure(value: &Value) -> Result<(), PrismError> {
    if value["status"] != "accepted" {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance transcript is not complete",
        ));
    }
    verify_transcript(value)
}

pub(crate) fn verify_transcript(value: &Value) -> Result<(), PrismError> {
    let features = registered_values(include_str!("../model/ids.toml"), "id", "id")?;
    let diagnostics = registered_values(include_str!("../model/errors.toml"), "error", "code")?;
    let mut observed_features = BTreeSet::new();
    let mut observed_diagnostics = BTreeSet::new();
    if !matches!(value["status"].as_str(), Some("accepted" | "passed")) {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance transcript status is invalid",
        ));
    }
    for row in value["cases"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6002", "production acceptance cases are absent"))?
    {
        if row["status"] != "passed" {
            return Err(PrismError::new(
                "PP6002",
                "production acceptance contains an unpassed execution",
            ));
        }
        match row["kind"].as_str() {
            Some("feature") => {
                let feature = row["feature_id"].as_str().ok_or_else(|| {
                    PrismError::new("PP6002", "feature result has no feature identity")
                })?;
                if !row["diagnostic"].is_null()
                    || !features.contains(feature)
                    || !observed_features.insert(feature.to_owned())
                {
                    return Err(PrismError::new(
                        "PP6002",
                        "production acceptance has an unknown, duplicate, or malformed feature result",
                    ));
                }
            }
            Some("diagnostic") => {
                let code = row["diagnostic"].as_str().ok_or_else(|| {
                    PrismError::new("PP6002", "diagnostic result has no diagnostic code")
                })?;
                if !row["feature_id"].is_null()
                    || !diagnostics.contains(code)
                    || !observed_diagnostics.insert(code.to_owned())
                {
                    return Err(PrismError::new(
                        "PP6002",
                        "production acceptance has an unknown, duplicate, or malformed diagnostic result",
                    ));
                }
            }
            _ => {
                return Err(PrismError::new(
                    "PP6002",
                    "production acceptance result kind is invalid",
                ));
            }
        }
    }
    if (value["status"] == "accepted" && observed_features != features)
        || value["feature_count"].as_u64() != Some(observed_features.len() as u64)
    {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance does not execute every registered public feature exactly once",
        ));
    }
    if (value["status"] == "accepted" && observed_diagnostics != diagnostics)
        || value["diagnostic_count"].as_u64() != Some(observed_diagnostics.len() as u64)
    {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance does not trigger every registered diagnostic exactly once",
        ));
    }
    Ok(())
}

pub(crate) fn verify_result_counts(value: &Value) -> Result<(), PrismError> {
    let features = registered_values(include_str!("../model/ids.toml"), "id", "id")?;
    let diagnostics = registered_values(include_str!("../model/errors.toml"), "error", "code")?;
    if value["feature_count"].as_u64() != Some(features.len() as u64)
        || value["diagnostic_count"].as_u64() != Some(diagnostics.len() as u64)
    {
        return Err(PrismError::new(
            "PP6002",
            "conformance result counts do not match the complete current register",
        ));
    }
    Ok(())
}

fn verify_coverage_bindings(value: &Value, coverage_bytes: &[u8]) -> Result<(), PrismError> {
    let coverage = CanonicalDocument::parse("prismpm/capability-coverage/1", coverage_bytes)?;
    let feature_rows = coverage.value()["features"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6002", "capability feature coverage is absent"))?;
    let diagnostic_rows = coverage.value()["diagnostics"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6002", "diagnostic coverage is absent"))?;
    let mut covered_features = BTreeSet::new();
    let mut covered_diagnostics = BTreeSet::new();
    for row in feature_rows {
        let id = row["feature_id"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP6002", "covered feature has no identity"))?;
        let scenario = id.to_ascii_lowercase();
        if row["command"] != format!("prismpm-conformance --feature {id}")
            || row["model_source"] != format!("model/ids.toml#{id}")
            || row["generated_outputs"]
                != json!([format!("projections/capability-coverage.json#feature={id}")])
            || row["standard_authorities"] != crate::system::authority_binding_for_feature(id)?
            || row["visibility"] != "public"
            || row["positive_scenario"] != format!("{scenario}-positive")
            || row["negative_scenario"] != format!("{scenario}-negative")
            || row["positive_evidence"] != format!("production-acceptance://feature/{id}#positive")
            || row["negative_evidence"]
                != format!("production-acceptance://feature/{id}#planted-defect")
            || !covered_features.insert(id.to_owned())
        {
            return Err(PrismError::new(
                "PP6002",
                "feature coverage does not name its exact executable positive and planted-defect evidence",
            ));
        }
    }
    for row in diagnostic_rows {
        let code = row["code"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP6002", "covered diagnostic has no identity"))?;
        if row["command"] != format!("prismpm-conformance --diagnostic {code}")
            || row["model_source"] != format!("model/errors.toml#{code}")
            || row["generated_outputs"]
                != json!([format!(
                    "projections/capability-coverage.json#diagnostic={code}"
                )])
            || row["standard_authorities"]
                != json!({
                    "kind":"none",
                    "reason":"Prism-owned diagnostic contract; no external standard owns this error code"
                })
            || row["visibility"] != "public"
            || row["scenario"] != format!("diagnostic-{}", code.to_ascii_lowercase())
            || row["evidence"] != format!("production-acceptance://diagnostic/{code}")
            || !covered_diagnostics.insert(code.to_owned())
        {
            return Err(PrismError::new(
                "PP6002",
                "diagnostic coverage does not name its exact executable evidence",
            ));
        }
    }
    let transcript_cases = value["cases"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP6002", "production acceptance cases are absent"))?;
    let executed_features = transcript_cases
        .iter()
        .filter_map(|row| row["feature_id"].as_str().map(str::to_owned))
        .collect::<BTreeSet<_>>();
    let executed_diagnostics = transcript_cases
        .iter()
        .filter_map(|row| row["diagnostic"].as_str().map(str::to_owned))
        .collect::<BTreeSet<_>>();
    if covered_features != executed_features || covered_diagnostics != executed_diagnostics {
        return Err(PrismError::new(
            "PP6002",
            "capability coverage and executed production acceptance differ",
        ));
    }
    for row in transcript_cases {
        let expected = if let Some(id) = row["feature_id"].as_str() {
            format!("prismpm-conformance --feature {id}")
        } else if let Some(code) = row["diagnostic"].as_str() {
            format!("prismpm-conformance --diagnostic {code}")
        } else {
            return Err(PrismError::new(
                "PP6002",
                "production acceptance case has no executable identity",
            ));
        };
        if row["command"] != expected {
            return Err(PrismError::new(
                "PP6002",
                "production acceptance command does not match its covered identity",
            ));
        }
    }
    Ok(())
}

fn sdk_digest(root: &Path) -> Result<String, PrismError> {
    let lock = crate::sdk::execution_lock(root)?;
    lock.value()["sdk_image"]
        .as_str()
        .and_then(|value| value.rsplit_once('@'))
        .map(|(_, digest)| digest.to_owned())
        .ok_or_else(|| PrismError::new("PP5401", "SDK image digest is absent"))
}

fn verify_bindings(
    root: &Path,
    digest: &str,
    document: &CanonicalDocument,
) -> Result<String, PrismError> {
    if document.value()["release_digest"].as_str() != Some(digest) {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance evidence names a different release digest",
        ));
    }
    let coverage = oci::artifact(root, digest, "projections/capability-coverage.json")?;
    if document.value()["coverage_digest"].as_str() != Some(sha(&coverage).as_str()) {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance evidence names stale capability coverage",
        ));
    }
    let sdk_digest = sdk_digest(root)?;
    if document.value()["sdk_digest"].as_str() != Some(sdk_digest.as_str()) {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance evidence was executed with a different SDK digest",
        ));
    }
    verify_closure(document.value())?;
    verify_coverage_bindings(document.value(), &coverage)?;
    Ok(sdk_digest)
}

fn attach_document(
    root: &Path,
    digest: &str,
    document: &CanonicalDocument,
) -> Result<(String, String), PrismError> {
    let sdk_digest = verify_bindings(root, digest, document)?;
    let descriptor =
        oci::attach_referrer(root, digest, PRISM_PRODUCTION_ACCEPTANCE, document.bytes())?;
    Ok((descriptor, sdk_digest))
}

/// Verify and attach independently produced execution evidence to its exact OCI subject.
pub(crate) fn attach(root: &Path, reference: &str, input: &Path) -> Result<Value, PrismError> {
    let digest = release_digest(reference)?;
    let bytes = std::fs::read(input)
        .map_err(|error| PrismError::new("PP6002", format!("acceptance evidence: {error}")))?;
    let document = CanonicalDocument::parse("prismpm/production-acceptance/1", &bytes)?;
    let (descriptor, sdk_digest) = attach_document(root, digest, &document)?;
    Ok(json!({
        "acceptance_digest": sha(document.bytes()),
        "referrer_digest": descriptor,
        "release_digest": digest,
        "schema": "prismpm/production-acceptance-result/1",
        "sdk_digest": sdk_digest,
        "status": "accepted"
    }))
}

fn result_document(
    transcript: &CanonicalDocument,
    descriptor: &str,
    release: &str,
    runner: &str,
    sdk: &str,
    path: &str,
) -> Result<Value, PrismError> {
    verify_closure(transcript.value())?;
    CanonicalDocument::from_value(
        "prismpm/conformance-result/1",
        json!({
            "diagnostic_count":transcript.value()["diagnostic_count"],
            "feature_count":transcript.value()["feature_count"],
            "referrer_digest":descriptor,
            "release_digest":release,
            "runner_digest":runner,
            "schema":"prismpm/conformance-result/1",
            "sdk_digest":sdk,
            "status":"accepted",
            "transcript_digest":transcript.digest(),
            "transcript_path":path
        }),
    )
    .map(|document| document.value().clone())
}

/// Execute the shipped conformance and diagnostic corpus and attach its exact transcript.
pub(crate) fn run(root: &Path, reference: &str) -> Result<Value, PrismError> {
    let digest = release_digest(reference)?;
    let coverage = oci::artifact(root, digest, "projections/capability-coverage.json")?;
    let coverage_digest = sha(&coverage);
    let locked_sdk = sdk_digest(root)?;
    let runner = crate::sdk::executable("prismpm-conformance")?;
    let shipped_root = Path::new("/opt/prismpm/share/conformance-root");
    let source_root = std::env::var_os("PRISMPM_CONFORMANCE_ROOT")
        .map(std::path::PathBuf::from)
        .or_else(|| shipped_root.is_dir().then(|| shipped_root.to_owned()))
        .or_else(|| {
            root.join("features/suites")
                .is_dir()
                .then(|| root.to_owned())
        })
        .ok_or_else(|| PrismError::new("PP5401", "SDK conformance source closure is absent"))?;
    let args = vec![
        "--source-root".to_owned(),
        source_root.to_string_lossy().into_owned(),
        "--release-digest".to_owned(),
        digest.to_owned(),
        "--coverage-digest".to_owned(),
        coverage_digest,
        "--sdk-digest".to_owned(),
        locked_sdk.clone(),
    ];
    let record = crate::verification::run_process_limited(
        "prismpm-conformance",
        &runner,
        &args,
        root,
        &BTreeMap::new(),
        &[(root, "$PROJECT"), (&source_root, "$CONFORMANCE")],
        "PP6002",
        "3600",
        16_777_216,
    )?;
    let bytes = record
        .stdout
        .strip_suffix('\n')
        .unwrap_or(&record.stdout)
        .as_bytes();
    let document = CanonicalDocument::parse("prismpm/production-acceptance/1", bytes)?;
    let expected_runner = format!("sha256:{}", record.executable_sha256);
    if document.value()["runner_digest"].as_str() != Some(expected_runner.as_str()) {
        return Err(PrismError::new(
            "PP6002",
            "production acceptance names a different conformance runner executable",
        ));
    }
    let (descriptor, sdk_digest) = attach_document(root, digest, &document)?;
    let directory = root.join(".prism/conformance");
    std::fs::create_dir_all(&directory).map_err(|error| {
        PrismError::new(
            "PP6002",
            format!("create conformance evidence directory: {error}"),
        )
    })?;
    let transcript_digest = document.digest();
    let path = directory.join(format!("{}.json", &transcript_digest[7..]));
    let mut staged = tempfile::NamedTempFile::new_in(&directory)
        .map_err(|error| PrismError::new("PP6002", format!("stage transcript: {error}")))?;
    staged
        .write_all(document.bytes())
        .and_then(|()| staged.as_file().sync_all())
        .map_err(|error| PrismError::new("PP6002", format!("write transcript: {error}")))?;
    staged.persist(&path).map_err(|error| {
        PrismError::new("PP6002", format!("publish conformance transcript: {error}"))
    })?;
    result_document(
        &document,
        &descriptor,
        digest,
        &expected_runner,
        &sdk_digest,
        &path.strip_prefix(root).unwrap_or(&path).to_string_lossy(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn execution_boundary_rejects_wrong_native_inventory() {
        crate::sdk::execution_binding_regression(
            "acceptance::tests::execution_boundary_rejects_wrong_native_inventory",
            |root| super::sdk_digest(root).map(|_| ()),
        );
    }

    use super::{registered_values, result_document, verify_closure};
    use crate::contracts::CanonicalDocument;
    use serde_json::{json, Value};

    fn complete() -> Value {
        let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let mut cases = registered_values(include_str!("../model/ids.toml"), "id", "id")
            .unwrap()
            .into_iter()
            .map(|id| {
                json!({
                    "command":format!("prismpm-conformance --feature {id}"),
                    "diagnostic":Value::Null,
                    "evidence_digest":digest,
                    "feature_id":id,
                    "kind":"feature",
                    "status":"passed"
                })
            })
            .collect::<Vec<_>>();
        let feature_count = cases.len();
        cases.extend(
            registered_values(include_str!("../model/errors.toml"), "error", "code")
                .unwrap()
                .into_iter()
                .map(|code| {
                    json!({
                        "command":format!("prismpm-conformance --diagnostic {code}"),
                        "diagnostic":code,
                        "evidence_digest":digest,
                        "feature_id":Value::Null,
                        "kind":"diagnostic",
                        "status":"passed"
                    })
                }),
        );
        let diagnostic_count = cases.len() - feature_count;
        json!({
            "cases":cases,
            "coverage_digest":digest,
            "diagnostic_count":diagnostic_count,
            "feature_count":feature_count,
            "release_digest":digest,
            "runner_digest":digest,
            "schema":"prismpm/production-acceptance/1",
            "sdk_digest":digest,
            "status":"accepted"
        })
    }

    #[test]
    fn incomplete_acceptance_is_rejected() {
        let value = json!({"cases":[]});
        let error = verify_closure(&value).unwrap_err();
        assert_eq!(error.code.as_str(), "PP6002");
    }

    #[test]
    fn exact_feature_and_diagnostic_closure_is_required() {
        let value = complete();
        verify_closure(&value).unwrap();
        CanonicalDocument::from_value("prismpm/production-acceptance/1", value.clone()).unwrap();
        let mut missing = value;
        missing["cases"].as_array_mut().unwrap().pop();
        assert_eq!(
            verify_closure(&missing).unwrap_err().code.as_str(),
            "PP6002"
        );
    }

    #[test]
    fn result_counts_come_from_the_complete_current_transcript() {
        let value = complete();
        assert!(value["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["feature_id"] == "HO-11"));
        assert!(value["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["diagnostic"] == "PP2009"));
        let transcript =
            CanonicalDocument::from_value("prismpm/production-acceptance/1", value.clone())
                .unwrap();
        let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let result = result_document(
            &transcript,
            digest,
            digest,
            digest,
            digest,
            ".prism/transcript.json",
        )
        .unwrap();
        assert_eq!(result["feature_count"], value["feature_count"]);
        assert_eq!(result["diagnostic_count"], value["diagnostic_count"]);
        for field in ["feature_count", "diagnostic_count"] {
            let mut stale = result.clone();
            stale[field] = json!(stale[field].as_u64().unwrap() - 1);
            assert_eq!(
                CanonicalDocument::from_value("prismpm/conformance-result/1", stale)
                    .unwrap_err()
                    .code,
                "PP6002"
            );
        }
    }

    #[test]
    fn accepted_transcript_rejects_missing_new_cases_even_with_adjusted_counts() {
        for (field, identity, count) in [
            ("feature_id", "HO-11", "feature_count"),
            ("diagnostic", "PP2009", "diagnostic_count"),
        ] {
            let mut missing = complete();
            missing["cases"]
                .as_array_mut()
                .unwrap()
                .retain(|row| row[field] != identity);
            missing[count] = json!(missing[count].as_u64().unwrap() - 1);
            assert_eq!(verify_closure(&missing).unwrap_err().code, "PP6002");
            assert_eq!(
                CanonicalDocument::from_value("prismpm/production-acceptance/1", missing)
                    .unwrap_err()
                    .code,
                "PP6002"
            );
        }
    }

    #[test]
    fn passed_slices_require_exact_cases_and_counts_but_are_not_accepted() {
        let mut value = complete();
        value["status"] = json!("passed");
        value["cases"]
            .as_array_mut()
            .unwrap()
            .retain(|row| row["feature_id"] != "HO-11");
        value["feature_count"] = json!(value["feature_count"].as_u64().unwrap() - 1);
        let transcript =
            CanonicalDocument::from_value("prismpm/production-acceptance/1", value.clone())
                .unwrap();
        assert_eq!(verify_closure(&value).unwrap_err().code, "PP6002");
        let digest = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        assert_eq!(
            result_document(
                &transcript,
                digest,
                digest,
                digest,
                digest,
                ".prism/transcript.json"
            )
            .unwrap_err()
            .code,
            "PP6002"
        );
        for (pointer, replacement) in [
            (
                "/feature_count",
                json!(value["feature_count"].as_u64().unwrap() + 1),
            ),
            ("/cases/0/feature_id", json!("ZZ-99")),
            ("/cases/0/status", json!("skipped")),
        ] {
            let mut bad = value.clone();
            *bad.pointer_mut(pointer).unwrap() = replacement;
            assert!(CanonicalDocument::from_value("prismpm/production-acceptance/1", bad).is_err());
        }
        let mut duplicate = value;
        let row = duplicate["cases"][0].clone();
        duplicate["cases"].as_array_mut().unwrap().insert(0, row);
        duplicate["feature_count"] = json!(duplicate["feature_count"].as_u64().unwrap() + 1);
        assert!(
            CanonicalDocument::from_value("prismpm/production-acceptance/1", duplicate).is_err()
        );
    }
}
