//! Independent upstream bytes exercised by generated std/no_std codec exports.

use prism_stdlib as wire;
use serde_json::Value;

fn fixture(browser: bool) -> Value {
    serde_json::from_str(if browser {
        include_str!("data/holo-browser-codec-v1.json")
    } else {
        include_str!("data/holo-codec-v1.json")
    })
    .unwrap()
}

fn bytes(value: &Value) -> Vec<u8> {
    let value = value.as_str().unwrap();
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|part| u8::from_str_radix(std::str::from_utf8(part).unwrap(), 16).unwrap())
        .collect()
}

fn manifest(data: &Value) -> Vec<u8> {
    let labels = data["manifest"]["references"].as_array().unwrap();
    assert_eq!(labels.len(), 3);
    wire::browserAppManifest(
        labels[0].as_str().unwrap().as_bytes().to_vec(),
        labels[1].as_str().unwrap().as_bytes().to_vec(),
        labels[2].as_str().unwrap().as_bytes().to_vec(),
    )
    .unwrap()
}

fn body(data: &Value, manifest: Vec<u8>, blobs: &[Vec<u8>]) -> Option<Vec<u8>> {
    assert_eq!(blobs.len(), 4);
    let inputs = &data["archive_inputs"];
    wire::browserArchiveBody(
        manifest,
        bytes(&inputs["metadata_hex"]),
        bytes(&inputs["directory_json_hex"]),
        bytes(&inputs["provenance_json_hex"]),
        blobs[0].clone(),
        blobs[1].clone(),
        blobs[2].clone(),
        blobs[3].clone(),
    )
    .unwrap()
}

fn blobs(data: &Value) -> Vec<Vec<u8>> {
    data["archive_inputs"]["blobs_hex"]
        .as_array()
        .unwrap()
        .iter()
        .map(bytes)
        .collect()
}

#[test]
fn exact_upstream_manifest_and_every_reference() {
    let data = fixture(true);
    let actual = manifest(&data);
    assert_eq!(actual, bytes(&data["manifest"]["bytes_hex"]));
    assert_eq!(actual.len(), 365);
    assert!(wire::browserValidAppManifest(&actual));
    for index in 0..3 {
        assert_eq!(
            wire::browserManifestReference(actual.clone(), index).unwrap(),
            Some(
                data["manifest"]["references"][index as usize]
                    .as_str()
                    .unwrap()
                    .as_bytes()
                    .to_vec()
            )
        );
    }
    for index in [3, 4, u32::MAX as u64, u64::MAX] {
        assert_eq!(
            wire::browserManifestReference(actual.clone(), index).unwrap(),
            None
        );
    }
    let digest = format!("blake3:{}", blake3::hash(&actual));
    assert_eq!(digest, data["manifest"]["kappa"]);
}

#[test]
fn exact_upstream_archive_and_all_accessors() {
    let data = fixture(true);
    let actual_body = body(&data, manifest(&data), &blobs(&data)).unwrap();
    assert_eq!(actual_body, bytes(&data["archive"]["body_hex"]));
    assert!(wire::browserValidArchiveBody(&actual_body).unwrap());
    let footer = blake3::hash(&actual_body).as_bytes().to_vec();
    assert_eq!(footer, bytes(&data["archive"]["footer_hex"]));
    let actual = wire::browserFrameArchive(actual_body.clone(), footer.clone())
        .unwrap()
        .unwrap();
    assert_eq!(actual, bytes(&data["archive"]["bytes_hex"]));
    assert!(wire::browserValidArchiveFrame(&actual).unwrap());
    assert_eq!(
        wire::browserArchiveBodyBytes(actual.clone()).unwrap(),
        Some(actual_body)
    );
    assert_eq!(
        wire::browserArchiveFooter(actual.clone()).unwrap(),
        Some(footer)
    );
    let sections = data["archive"]["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 8);
    for (index, section) in sections.iter().enumerate() {
        assert_eq!(
            wire::browserArchiveSection(actual.clone(), index as u64).unwrap(),
            Some(bytes(&section["payload_hex"]))
        );
    }
    for index in [8, 9, u32::MAX as u64, u64::MAX] {
        assert_eq!(
            wire::browserArchiveSection(actual.clone(), index).unwrap(),
            None
        );
    }
    for (index, field) in [(0, "directory_json_hex"), (1, "provenance_json_hex")] {
        assert_eq!(
            wire::browserArchiveExtension(actual.clone(), index).unwrap(),
            Some(bytes(&data["archive_inputs"][field]))
        );
    }
    for index in [2, 3, u32::MAX as u64, u64::MAX] {
        assert_eq!(
            wire::browserArchiveExtension(actual.clone(), index).unwrap(),
            None
        );
    }
}

#[test]
fn profiles_do_not_accept_each_others_bytes() {
    let portable = fixture(false);
    let browser = fixture(true);
    let old_manifest = bytes(&portable["manifest"]["bytes_hex"]);
    let new_manifest = manifest(&browser);
    assert!(wire::validAppManifest(&old_manifest));
    assert!(!wire::validAppManifest(&new_manifest));
    assert!(!wire::browserValidAppManifest(&old_manifest));
    assert_eq!(
        wire::browserManifestReference(old_manifest.clone(), 0).unwrap(),
        None
    );
    assert_eq!(body(&browser, old_manifest, &blobs(&browser)), None);
    let old_body = bytes(&portable["archive"]["body_hex"]);
    let old_frame = bytes(&portable["archive"]["bytes_hex"]);
    let new_frame = bytes(&browser["archive"]["bytes_hex"]);
    assert!(!wire::browserValidArchiveBody(&old_body).unwrap());
    assert!(!wire::browserValidArchiveFrame(&old_frame).unwrap());
    assert_eq!(
        wire::browserArchiveBodyBytes(old_frame.clone()).unwrap(),
        None
    );
    assert_eq!(wire::browserArchiveFooter(old_frame.clone()).unwrap(), None);
    assert_eq!(
        wire::browserArchiveSection(old_frame.clone(), 0).unwrap(),
        None
    );
    assert_eq!(wire::browserArchiveExtension(old_frame, 0).unwrap(), None);
    assert_eq!(
        wire::browserFrameArchive(old_body, vec![0; 32]).unwrap(),
        None
    );
    assert!(!wire::validArchiveFrame(&new_frame).unwrap());
}

#[test]
fn exact_manifest_profile_and_length_are_closed() {
    let data = fixture(true);
    let valid = manifest(&data);
    for end in 0..valid.len() {
        let truncated = valid[..end].to_vec();
        assert!(
            !wire::browserValidAppManifest(&truncated),
            "truncation {end}"
        );
        assert_eq!(wire::browserManifestReference(truncated, 0).unwrap(), None);
    }
    let mut trailing = valid.clone();
    trailing.push(0);
    assert!(!wire::browserValidAppManifest(&trailing));
    // All non-reference bytes are the exact independently encoded closed profile.
    for index in (0..57).chain(270..valid.len()) {
        let mut changed = valid.clone();
        changed[index] ^= 1;
        assert!(
            !wire::browserValidAppManifest(&changed),
            "profile byte {index}"
        );
    }
    let labels = data["manifest"]["references"].as_array().unwrap();
    for slot in 0..3 {
        for bad in [vec![], vec![b'x'; 71], vec![b'a'; 70], vec![b'a'; 72]] {
            let mut inputs = labels
                .iter()
                .map(|v| v.as_str().unwrap().as_bytes().to_vec())
                .collect::<Vec<_>>();
            inputs[slot] = bad;
            assert_eq!(
                wire::browserAppManifest(inputs[0].clone(), inputs[1].clone(), inputs[2].clone()),
                None
            );
        }
    }
}

#[test]
fn malformed_archives_preserve_the_cryptography_boundary() {
    let data = fixture(true);
    let malformed = data["malformed_archives"].as_array().unwrap();
    assert_eq!(malformed.len(), 12);
    for case in malformed {
        let value = bytes(&case["bytes_hex"]);
        let name = case["name"].as_str().unwrap();
        // A footer bit flip is structurally valid; digest verification is not
        // claimed by the pure codec and must independently reject its content.
        assert_eq!(
            wire::browserValidArchiveFrame(&value).unwrap(),
            name == "bad-footer",
            "{name}"
        );
        if name == "bad-footer" {
            let footer = value.len() - 32;
            assert_ne!(&value[footer..], blake3::hash(&value[..footer]).as_bytes());
        } else {
            assert_eq!(
                wire::browserArchiveBodyBytes(value.clone()).unwrap(),
                None,
                "{name}"
            );
            assert_eq!(
                wire::browserArchiveSection(value.clone(), 0).unwrap(),
                None,
                "{name}"
            );
            assert_eq!(
                wire::browserArchiveExtension(value, 1).unwrap(),
                None,
                "{name}"
            );
        }
    }
}

#[test]
fn composition_rejects_duplicate_unsorted_missing_and_escalated_blobs() {
    let data = fixture(true);
    let original = blobs(&data);
    let mut duplicate = original.clone();
    duplicate[1] = duplicate[0].clone();
    assert_eq!(body(&data, manifest(&data), &duplicate), None);
    let mut unsorted = original.clone();
    unsorted.swap(0, 1);
    assert_eq!(body(&data, manifest(&data), &unsorted), None);
    let labels = data["manifest"]["references"].as_array().unwrap();
    for slot in 0..3 {
        let mut changed = labels
            .iter()
            .map(|v| v.as_str().unwrap().as_bytes().to_vec())
            .collect::<Vec<_>>();
        changed[slot] = format!("blake3:{}", "f".repeat(64)).into_bytes();
        let changed =
            wire::browserAppManifest(changed[0].clone(), changed[1].clone(), changed[2].clone())
                .unwrap();
        assert_eq!(body(&data, changed, &original), None);
    }
    let capabilities = labels[0].as_str().unwrap().as_bytes();
    let mut escalated = original;
    let blob = escalated
        .iter_mut()
        .find(|b| b.starts_with(capabilities))
        .unwrap();
    blob[71] ^= 1;
    assert_eq!(body(&data, manifest(&data), &escalated), None);
    let substituted = &data["capability_substitution"];
    let substituted_blobs = substituted["blobs_hex"]
        .as_array()
        .unwrap()
        .iter()
        .map(bytes)
        .collect::<Vec<_>>();
    assert_eq!(substituted["upstream"]["stage"], "accepted-physical-archive");
    let substituted_manifest = bytes(&substituted["manifest_hex"]);
    assert!(wire::browserValidAppManifest(&substituted_manifest));
    assert_eq!(body(&data, substituted_manifest, &substituted_blobs), None);
    assert!(!wire::browserValidArchiveFrame(&bytes(&substituted["archive_hex"])).unwrap());
    let good_body = bytes(&data["archive"]["body_hex"]);
    for size in [0, 1, 31, 33] {
        assert_eq!(
            wire::browserFrameArchive(good_body.clone(), vec![0; size]).unwrap(),
            None
        );
    }
}
