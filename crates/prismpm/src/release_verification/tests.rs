use super::*;

#[test]
fn evidence_paths_reject_aliases_and_escape() {
    for path in [
        "",
        "/manifest.json",
        "../x",
        "a/../x",
        "./x",
        "a//x",
        "a/./x",
        "a\\x",
        "a/",
        "a\0x",
    ] {
        assert!(safe_path(path).is_err(), "accepted {path:?}");
    }
    safe_path("proof/nested/manifest.json").unwrap();
}

#[test]
fn evidence_json_preserves_lexlean_framing_and_rejects_duplicate_keys() {
    for bytes in [
        b"{\"a\":1,\"a\":1}".as_slice(),
        b"{ \"a\":1}",
        b"{\"a\":1}\n",
    ] {
        assert!(canonical_json(bytes, false).is_err());
    }
    assert!(canonical_json(b"{\"a\":1}", true).is_err());
    assert!(canonical_json(b"{\"a\":1}\n\n", true).is_err());
    canonical_json(b"{\"a\":1}\n", true).unwrap();
    canonical_json(b"{\"a\":1}", false).unwrap();
}

#[test]
fn evidence_descriptors_bind_size_hash_and_closed_fields() {
    let bytes = b"retained evidence";
    let valid = json!({"byte_length":bytes.len(), "sha256":hex(bytes)});
    descriptor(&valid, bytes).unwrap();
    for changed in [
        json!({"byte_length":bytes.len()+1, "sha256":hex(bytes)}),
        json!({"byte_length":bytes.len(), "sha256":"0".repeat(64)}),
        json!({"byte_length":bytes.len(), "sha256":hex(bytes), "passed":true}),
    ] {
        assert!(descriptor(&changed, bytes).is_err());
    }
}

#[test]
fn native_export_framing_is_the_imported_named_export_format() {
    let bytes = b"{\n  \"included_definitions\": [\"PrismPM.root\"],\n  \"requested_roots\": [\"PrismPM.root\"]\n}\n";
    let value = named_export_json(bytes).unwrap();
    assert!(named_export_json(&encode_value(&value).unwrap()).is_err());
    assert!(named_export_json(b"{\n  \"a\": [],\n  \"a\": []\n}\n").is_err());
}

/// The owning runtime test supplies genuinely produced evidence. No synthetic
/// record can establish the baseline used by these coherent mutations.
pub(crate) fn reject_mutations(
    build_manifest: &[u8],
    build_files: &BTreeMap<String, Vec<u8>>,
    verification_files: &BTreeMap<String, Vec<u8>>,
) {
    let binding = validate(build_manifest, build_files, verification_files).unwrap();
    let rejected_build = |manifest: &[u8], files: &BTreeMap<String, Vec<u8>>, expected: &str| {
        let error = validate(manifest, files, verification_files).unwrap_err();
        assert_eq!(error.code, "PP6101");
        assert!(error.message.contains(expected), "{}", error.message);
    };
    let rejected = |files: &BTreeMap<String, Vec<u8>>, expected: &str| {
        let error = validate(build_manifest, build_files, files).unwrap_err();
        assert_eq!(error.code, "PP6101");
        assert!(error.message.contains(expected), "{}", error.message);
    };
    for name in verification_files.keys() {
        let mut changed = verification_files.clone();
        changed.remove(name);
        rejected(&changed, "verification");
    }
    let mut changed = verification_files.clone();
    changed.insert("undeclared.json".into(), b"{}".to_vec());
    rejected(&changed, "verification");

    let mut manifest = canonical_json(&verification_files["manifest.json"], false).unwrap();
    manifest["build_id"] = json!("0".repeat(64));
    let mut changed = verification_files.clone();
    changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
    rejected(&changed, "build identity");

    for omission in [false, true] {
        let mut manifest = canonical_json(&verification_files["manifest.json"], false).unwrap();
        if omission {
            manifest["processes"].as_array_mut().unwrap().pop();
        } else {
            manifest["processes"][0]["exit_code"] = json!(1);
        }
        let mut changed = verification_files.clone();
        changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
        rejected(&changed, "process");
    }

    let original = canonical_json(&verification_files["manifest.json"], false).unwrap();
    for index in 0..original["processes"].as_array().unwrap().len() {
        let mut manifest = original.clone();
        manifest["processes"][index]["argv"] = json!(["--version"]);
        if manifest == original {
            manifest["processes"][index]["argv"] = json!(["--not-the-required-stage"]);
        }
        let mut changed = verification_files.clone();
        changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
        rejected(&changed, "process");
    }
    let mut manifest = original.clone();
    manifest["processes"][0]["stdout"] = json!("Lean unapproved-version\n");
    let mut changed = verification_files.clone();
    changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
    rejected(&changed, "process");

    // Recompute the LexLean framed identity and the outer descriptor too: an
    // omitted proof audit must fail for its incomplete declaration closure.
    let mut lex = canonical_json(&verification_files["lexlean-attestation.json"], true).unwrap();
    lex["declarations"].as_array_mut().unwrap().pop();
    let mut body = lex.clone();
    body.as_object_mut().unwrap().remove("attestation_id");
    lex["attestation_id"] = json!(lex_ids::attestation_id(
        std::str::from_utf8(&encode_value(&body).unwrap()).unwrap()
    )
    .to_hex());
    let mut lex_bytes = encode_value(&lex).unwrap();
    lex_bytes.push(b'\n');
    let mut manifest = original.clone();
    if binding.family == "application" {
        manifest["lexlean_attestation_sha256"] = json!(hex(&lex_bytes));
    } else {
        manifest["lexlean_attestation_id"] = lex["attestation_id"].clone();
        manifest["artifacts"]["lexlean_attestation"] =
            json!({"byte_length":lex_bytes.len(),"sha256":hex(&lex_bytes)});
    }
    let mut changed = verification_files.clone();
    changed.insert("lexlean-attestation.json".into(), lex_bytes);
    changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
    rejected(&changed, "declaration audit");

    // Rewrite the manifest too: the intended failure is the inner evidence,
    // not a stale outer checksum or an OCI descriptor mismatch.
    let mut changed = verification_files.clone();
    let mut manifest = canonical_json(&changed["manifest.json"], false).unwrap();
    if binding.family == "application" {
        for mutation in 0..4 {
            let path = "application/lexlean-build-manifest.json";
            let mut selected = canonical_json(&build_files[path], true).unwrap();
            match mutation {
                0 => {
                    selected["inputs"].as_array_mut().unwrap().remove(0);
                }
                1 => {
                    let duplicate = selected["inputs"][0].clone();
                    selected["inputs"]
                        .as_array_mut()
                        .unwrap()
                        .insert(0, duplicate);
                }
                2 => {
                    selected["selection"].as_array_mut().unwrap().clear();
                }
                3 => {
                    selected["outputs"].as_array_mut().unwrap().remove(0);
                }
                _ => unreachable!(),
            }
            let mut bytes = encode_value(&selected).unwrap();
            bytes.push(b'\n');
            let mut build = canonical_json(build_manifest, false).unwrap();
            let descriptor = build["files"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|row| row["path"] == path)
                .unwrap();
            descriptor["byte_length"] = json!(bytes.len());
            descriptor["sha256"] = json!(hex(&bytes));
            let mut files = build_files.clone();
            files.insert(path.into(), bytes);
            rejected_build(
                &encode_value(&build).unwrap(),
                &files,
                "application selected",
            );
        }
        let mut acceptance =
            canonical_json(&changed["application-acceptance.json"], false).unwrap();
        acceptance["modeled_vectors"] = json!(0);
        let bytes = encode_value(&acceptance).unwrap();
        manifest["acceptance_sha256"] = json!(hex(&bytes));
        changed.insert("application-acceptance.json".into(), bytes);
        changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
        rejected(&changed, "application acceptance");

        let mut changed = verification_files.clone();
        let mut manifest = canonical_json(&changed["manifest.json"], false).unwrap();
        let oracle = manifest["processes"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["tool"] == "hologram-oracle")
            .unwrap();
        let mut report: Value = serde_json::from_str(oracle["stdout"].as_str().unwrap()).unwrap();
        report["portable_browser"]["skipped"] = json!(1);
        oracle["stdout"] = json!(serde_json::to_string(&report).unwrap());
        changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
        rejected(&changed, "oracle");
    } else {
        let mut execution = canonical_json(&changed["execution.json"], false).unwrap();
        execution["case_count"] = json!(0);
        let bytes = encode_value(&execution).unwrap();
        manifest["execution"] = execution;
        manifest["artifacts"]["execution_evidence"] =
            json!({"byte_length":bytes.len(),"sha256":hex(&bytes)});
        for row in manifest["processes"].as_array_mut().unwrap() {
            if row["tool"] == "generated-validator" {
                row["stdout"] = json!(String::from_utf8(bytes.clone()).unwrap());
            }
        }
        changed.insert("execution.json".into(), bytes);
        changed.insert("manifest.json".into(), encode_value(&manifest).unwrap());
        rejected(&changed, "native execution");
    }
}
