//! Exact Text/4 logical-boundary contract admission and resource refusals.
use prismpm::contracts::CanonicalDocument;
use serde_json::{json, Value};

#[test]
fn research_workbench_contract_admits_actual_modeled_logical_boundaries() {
    let mut model: Value = serde_json::from_slice(include_bytes!(
        "../../../tests/data/research-workbench-model-document.json"
    ))
    .unwrap();
    model["application"]["request_maximum"] = json!(2_097_152);
    model["application"]["response_maximum"] = json!(4_194_304);
    model["application"]["guest_allocation_maximum"] = json!(4_194_304);
    model["application"]["core_memory_pages"] = json!(512);
    let vectors = model["application"]["acceptance_vectors"]
        .as_array_mut()
        .unwrap();
    let boundary = vectors.last_mut().unwrap();
    boundary["request"] = json!(vec![120u8; 4_194_304]);
    let response = boundary["response"].clone();
    vectors.push(json!({"request":vec![120u8;2_097_152],"response":response}));
    let bytes = prismpm::holo::canonical::encode_value(&model).unwrap();
    let result = CanonicalDocument::parse("prismpm/model-document/7", &bytes);
    assert!(result.is_ok(),"new Text/4 contract must admit real logical request/allocation boundary vectors: {result:?}");
}

#[test]
fn research_workbench_contract_refuses_first_over_byte_bound_and_preserves_legacy_bound() {
    for (schema, maximum) in [
        ("prismpm/model-document/7", 67_108_864),
        ("prismpm/model-document/6", 16_777_216),
    ] {
        let bytes = vec![b' '; maximum + 1];
        let error = CanonicalDocument::parse(schema, &bytes).unwrap_err();
        assert_eq!(error.code.as_str(), "PP7601");
        assert_eq!(
            error.message,
            format!("{schema} exceeds its {maximum} byte limit")
        );
    }
}

#[test]
fn research_workbench_contract_refuses_actual_first_over_recursive_item_bound() {
    // The object contributes three items, and each array element two. This
    // exact legal JSON is below64MiB but first exceeds the33,554,432 item cap.
    let count = 16_777_215usize;
    let mut bytes = Vec::with_capacity(count * 2 + 70);
    bytes.extend_from_slice(b"{\"oversized\":[");
    for index in 0..count {
        if index > 0 {
            bytes.push(b',');
        }
        bytes.push(b'0');
    }
    bytes.extend_from_slice(b"],\"schema\":\"prismpm/model-document/7\"}");
    let error = CanonicalDocument::parse("prismpm/model-document/7", &bytes).unwrap_err();
    assert_eq!(error.code.as_str(), "PP7601");
    assert_eq!(
        error.message,
        "prismpm/model-document/7 exceeds its 33554432 item limit"
    );
}
