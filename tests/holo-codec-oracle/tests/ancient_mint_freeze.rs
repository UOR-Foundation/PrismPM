//! Legacy pinned reference wire-codec regressions only.
//! These synthetic vectors neither construct Ancient Mint nor qualify the
//! current hologram-live format, a PrismPM application, or release acceptance.
use hologram::archive::{HoloLoader, HoloWriter, SectionKind};
use hologram::space::{address_bytes, AppManifest, Realization};
use serde_json::{json, Value};

fn fixture() -> Value {
    let value: Value = serde_json::from_str(include_str!("../../data/holo-codec-v1.json"))
        .expect("repository-owned reference codec vector");
    assert_eq!(value["schema"], "prismpm/holo-codec-oracle/1");
    assert_eq!(
        value["oracle"]["revision"],
        "2bda6a9a9476872dade705bd61ece4209607f6da"
    );
    assert_eq!(
        value["scope"],
        "Synthetic wire-codec vectors; not executable-application, conformance, or release acceptance."
    );
    value
}

fn unhex(value: &Value) -> Vec<u8> {
    let text = value.as_str().expect("reference vector hex string");
    assert_eq!(text.len() % 2, 0);
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

fn section_kind(value: &Value) -> SectionKind {
    match value.as_u64().expect("reference section kind") {
        15 => SectionKind::AppManifest,
        8 => SectionKind::Metadata,
        14 => SectionKind::Extension,
        16 => SectionKind::ContentBlob,
        kind => panic!("unexpected reference section kind {kind}"),
    }
}

// Match the existing pinned producer's complete physical parse stages, not a
// new strict-profile or application-acceptance predicate.
fn physical_outcome(bytes: &[u8]) -> Value {
    let loader = match HoloLoader::from_bytes(bytes) {
        Ok(loader) => loader,
        Err(error) => return json!({"stage": "header-footer", "error": format!("{error:?}")}),
    };
    let plan = match loader.into_plan() {
        Ok(plan) => plan,
        Err(error) => return json!({"stage": "section-table", "error": format!("{error:?}")}),
    };
    for section in plan.sections() {
        if let Err(error) = plan.section(section.kind) {
            return json!({"stage": "section-payload", "error": format!("{error:?}")});
        }
    }
    if let Err(error) = plan.content_blobs() {
        return json!({"stage": "content-blob", "error": format!("{error:?}")});
    }
    if let Err(error) = plan.extensions() {
        return json!({"stage": "extension", "error": format!("{error:?}")});
    }
    json!({"stage": "accepted-physical-archive"})
}

#[test]
fn pinned_reference_archive_round_trips_every_section() {
    let vector = fixture();
    let archive = unhex(&vector["archive"]["bytes_hex"]);
    let body = unhex(&vector["archive"]["body_hex"]);
    let footer = unhex(&vector["archive"]["footer_hex"]);
    assert!(archive.starts_with(b"HOLO\x04\x00"));
    assert_eq!([body.as_slice(), footer.as_slice()].concat(), archive);
    assert_eq!(footer, blake3::hash(&body).as_bytes().to_vec());
    assert_eq!(
        address_bytes(&archive).to_string(),
        vector["archive"]["kappa"].as_str().unwrap()
    );

    let rows = vector["archive"]["sections"].as_array().unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["kind"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![15, 8, 14, 14, 16, 16, 16, 16]
    );
    let sections = rows
        .iter()
        .map(|row| (section_kind(&row["kind"]), unhex(&row["payload_hex"])))
        .collect::<Vec<_>>();
    assert_eq!(HoloWriter::assemble(sections), archive);

    let plan = HoloLoader::from_bytes(&archive)
        .unwrap()
        .into_plan()
        .unwrap();
    assert_eq!(plan.sections().len(), rows.len());
    for (actual, expected) in plan.sections().iter().zip(rows) {
        assert_eq!(actual.kind as u8 as u64, expected["kind"].as_u64().unwrap());
        assert_eq!(actual.offset, expected["offset"].as_u64().unwrap());
        assert_eq!(actual.length, expected["length"].as_u64().unwrap());
        let start = usize::try_from(actual.offset).unwrap();
        let length = usize::try_from(actual.length).unwrap();
        assert_eq!(
            &archive[start..start + length],
            unhex(&expected["payload_hex"])
        );
    }
    let manifest = unhex(&vector["manifest"]["bytes_hex"]);
    assert_eq!(plan.app_manifest().unwrap(), manifest);
    assert_eq!(
        AppManifest::decode(&manifest).unwrap().canonicalize(),
        manifest
    );
    let references = <AppManifest as Realization>::references(&manifest).unwrap();
    let expected = vector["manifest"]["references"].as_array().unwrap();
    assert_eq!(references.len(), expected.len());
    for (actual, expected) in references.iter().zip(expected) {
        assert_eq!(actual.to_string(), expected.as_str().unwrap());
    }
    let directory = unhex(&vector["archive_inputs"]["directory_json_hex"]);
    let provenance = unhex(&vector["archive_inputs"]["provenance_json_hex"]);
    assert_eq!(
        plan.extensions().unwrap(),
        vec![
            (
                "https://hologram.foundation/extension/application-directory/v1",
                directory.as_slice()
            ),
            (
                "https://uor.foundation/extension/prismpm-model/v1",
                provenance.as_slice()
            )
        ]
    );
    let blobs = vector["archive_inputs"]["blobs_hex"].as_array().unwrap();
    let parsed_blobs = plan.content_blobs().unwrap();
    assert_eq!(parsed_blobs.len(), 4);
    assert_eq!(blobs.len(), 4);
    let label_length = address_bytes(b"").as_bytes().len();
    for ((actual_label, actual_content), blob) in parsed_blobs.iter().zip(blobs) {
        let payload = unhex(blob);
        assert!(payload.len() >= label_length);
        assert_eq!(*actual_label, &payload[..label_length]);
        assert_eq!(*actual_content, &payload[label_length..]);
        assert_eq!(address_bytes(actual_content).as_bytes(), *actual_label);
    }
    assert_eq!(
        physical_outcome(&archive),
        json!({"stage": "accepted-physical-archive"})
    );
}

#[test]
fn pinned_reference_loader_preserves_all_existing_malformed_outcomes() {
    let vector = fixture();
    let cases = vector["malformed_archives"].as_array().unwrap();
    assert_eq!(
        cases
            .iter()
            .map(|row| row["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![
            "bad-magic",
            "older-version",
            "newer-version",
            "nonzero-flags",
            "nonzero-padding",
            "wrong-section-count",
            "unknown-section-kind",
            "out-of-range-offset",
            "payload-inside-table",
            "out-of-range-length",
            "bad-footer",
            "trailing-payload-byte"
        ]
    );
    for case in cases {
        assert_eq!(case["strict_profile_accepts"], false);
        // Some legacy upstream parses succeed. Preserve those exact observed
        // outcomes without claiming the strict PrismPM profile accepted them.
        assert_eq!(
            physical_outcome(&unhex(&case["bytes_hex"])),
            case["upstream"],
            "{}",
            case["name"]
        );
    }
}

#[test]
fn pinned_reference_archive_rejects_unsealed_mutations_and_truncation() {
    let vector = fixture();
    let archive = unhex(&vector["archive"]["bytes_hex"]);
    let expected = &vector["malformed_archives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "bad-footer")
        .unwrap()["upstream"];
    let payload =
        usize::try_from(vector["archive"]["sections"][0]["offset"].as_u64().unwrap()).unwrap();
    for index in [payload, archive.len() - 1] {
        let mut changed = archive.clone();
        changed[index] ^= 1;
        assert_eq!(&physical_outcome(&changed), expected);
    }
    for length in [0, 1, 5, 9, archive.len() - 1] {
        assert!(
            HoloLoader::from_bytes(&archive[..length]).is_err(),
            "truncated length {length}"
        );
    }
}
