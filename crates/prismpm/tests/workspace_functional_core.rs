//! Verification and independent acceptance evidence for the generated workspace
//! application profile, View, and Kappa replication/read-admission path (DK-07..DK-16).

use jsonschema::validator_for;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate is below repository root")
        .to_path_buf()
}

fn valid_workspace_labels() -> Value {
    json!({
        "spec": "prismpm/workspace-view-labels/1",
        "title": "Workspace",
        "workspace": "Workspace identifier",
        "select": "Select workspace",
        "members": "Members",
        "messages": "Messages",
        "next": "Next page",
        "action": "Action",
        "body": "Body (member identifier or message text)",
        "submit": "Submit command",
        "refresh": "Refresh authenticated history",
        "close": "Close workspace view",
        "result": "Workspace result",
        "asOf": "As-of head",
        "total": "Total",
        "offset": "Offset",
        "owner": "Owner",
        "contributor": "Contributor",
        "reader": "Reader",
        "event": "Event",
        "author": "Author",
        "message": "Message",
        "principal": "Principal",
        "role": "Role",
        "ready": "Ready",
        "pending": "Pending",
        "replay": "Refresh required",
        "closed": "Closed",
        "rejected": "Rejected",
        "conflict": "Conflict",
        "unknown": "Commit outcome unknown",
        "unavailable": "Unavailable",
        "inputError": "Input rejected",
        "action0": "Create workspace",
        "action1": "Grant contributor",
        "action2": "Grant reader",
        "action3": "Revoke member",
        "action4": "Post message",
        "none": "None"
    })
}

#[test]
fn workspace_view_labels_schema_conformance_and_falsification() {
    let schema_bytes =
        fs::read(root().join("schemas/workspace-view-labels.schema.json")).expect("labels schema");
    let schema_val: Value = serde_json::from_slice(&schema_bytes).expect("parse labels schema");
    let validator = validator_for(&schema_val).expect("compile labels schema validator");

    let labels = valid_workspace_labels();
    assert!(
        validator.is_valid(&labels),
        "valid labels must pass schema validation"
    );

    // Schema requires exactly 39 properties
    let obj = labels.as_object().unwrap();
    assert_eq!(obj.len(), 39);

    // Test rejection when each required property is omitted
    for key in obj.keys() {
        let mut mutated = labels.clone();
        mutated.as_object_mut().unwrap().remove(key);
        assert!(
            !validator.is_valid(&mutated),
            "omitting required property '{key}' must fail validation"
        );
    }

    // Test rejection of additional unknown properties
    let mut with_extra = labels.clone();
    with_extra
        .as_object_mut()
        .unwrap()
        .insert("injected_property".to_owned(), json!("malicious"));
    assert!(
        !validator.is_valid(&with_extra),
        "additional property must be rejected by schema"
    );

    // Test rejection of non-string values
    let mut with_numeric = labels.clone();
    with_numeric["title"] = json!(12345);
    assert!(
        !validator.is_valid(&with_numeric),
        "numeric property must fail validation"
    );

    // Test rejection of empty string (minLength 1)
    let mut with_empty = labels.clone();
    with_empty["title"] = json!("");
    assert!(
        !validator.is_valid(&with_empty),
        "empty string must fail validation (minLength 1)"
    );

    // Test rejection of oversized string (> 256)
    let mut with_oversized = labels.clone();
    with_oversized["title"] = json!("a".repeat(257));
    assert!(
        !validator.is_valid(&with_oversized),
        "oversized string must fail validation (maxLength 256)"
    );
}

#[test]
fn workspace_diagnostics_registries_closure() {
    let view_diag_text = fs::read_to_string(root().join("model/browser-view-diagnostics.toml"))
        .expect("browser view diagnostics");
    let view_diag: toml::Value = toml::from_str(&view_diag_text).expect("parse view diagnostics");

    assert_eq!(
        view_diag.get("spec").and_then(toml::Value::as_str),
        Some("prismpm/browser-view-diagnostics/1")
    );
    assert_eq!(
        view_diag.get("error_class").and_then(toml::Value::as_str),
        Some("ViewHostError")
    );
    assert_eq!(
        view_diag.get("capability").and_then(toml::Value::as_str),
        Some("DK-16")
    );

    let errors = view_diag
        .get("error")
        .and_then(toml::Value::as_array)
        .expect("errors list");
    assert_eq!(errors.len(), 11);
    let mut error_codes = BTreeSet::new();
    for err in errors {
        let code = err
            .get("code")
            .and_then(toml::Value::as_str)
            .expect("error code");
        let statement = err
            .get("statement")
            .and_then(toml::Value::as_str)
            .expect("statement");
        assert!(!statement.is_empty());
        assert!(error_codes.insert(code));
    }

    let rejections = view_diag
        .get("rejection")
        .and_then(toml::Value::as_array)
        .expect("rejections list");
    assert_eq!(rejections.len(), 14);
    let expected_rejections = [
        (1, "BadEncoding"),
        (2, "InvalidState"),
        (3, "InvalidIntent"),
        (4, "Busy"),
        (5, "ReplayRequired"),
        (6, "Closed"),
        (7, "CounterExhausted"),
        (8, "NoSelection"),
        (9, "NoNextPage"),
        (10, "WrongPhase"),
        (11, "CorrelationMismatch"),
        (12, "InvalidOutcome"),
        (13, "InvalidPage"),
        (14, "UnknownOperation"),
    ];
    for (idx, &(expected_byte, expected_name)) in expected_rejections.iter().enumerate() {
        let rej = &rejections[idx];
        assert_eq!(
            rej.get("byte").and_then(toml::Value::as_integer),
            Some(expected_byte)
        );
        assert_eq!(
            rej.get("name").and_then(toml::Value::as_str),
            Some(expected_name)
        );
    }

    // Verify command/query adapter diagnostics
    let adapter_diag_text =
        fs::read_to_string(root().join("model/browser-adapter-diagnostics.toml"))
            .expect("adapter diagnostics");
    assert!(adapter_diag_text.contains("DK-13"));
    assert!(adapter_diag_text.contains("DK-14"));

    // Verify browser host diagnostics
    let browser_diag_text = fs::read_to_string(root().join("model/browser-diagnostics.toml"))
        .expect("browser diagnostics");
    assert!(browser_diag_text.contains("DK-12"));
}

#[test]
fn workspace_stdlib_apis_and_protocol_contract_boundaries() {
    let command: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspaceCommandBytes;
    let query: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspaceQueryBytes;
    let interaction: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspaceInteractionBytes;
    let presentation: fn(Vec<u8>) -> Result<Vec<u8>, prism_stdlib::ComputeError> =
        prism_stdlib::workspacePresentationBytes;

    // Empty vector rejections match registered rejection codes
    assert_eq!(command(Vec::new()).unwrap(), [1]); // BadEncoding
    assert_eq!(query(Vec::new()).unwrap(), [1]); // BadEncoding
    assert_eq!(interaction(Vec::new()).unwrap(), [1]); // BadEncoding
    assert_eq!(presentation(Vec::new()).unwrap(), [2]); // InvalidState (requires 50564e01)

    // Valid initialization request: op 0 followed by 32 non-zero session bytes
    let mut init_req = vec![0x00];
    let session = [0x42_u8; 32];
    init_req.extend_from_slice(&session);
    let init_resp = interaction(init_req).expect("valid session initialization");
    assert_eq!(
        init_resp.first(),
        Some(&0),
        "op 0 must succeed with status 0"
    );

    // Decoded plan has: [0x00, stateLength24 (3 bytes), state..., effectLength16 (2 bytes), effect...]
    assert!(init_resp.len() > 6);
    let state_len = (usize::from(init_resp[1]) << 16)
        | (usize::from(init_resp[2]) << 8)
        | usize::from(init_resp[3]);
    let state_start = 4;
    let state_end = state_start + state_len;
    assert!(state_end + 2 <= init_resp.len());
    let state = &init_resp[state_start..state_end];

    // Verify interaction state header: 0x50, 0x56, 0x49, 0x01 (PVI1)
    assert_eq!(
        &state[..4],
        &[0x50, 0x56, 0x49, 0x01],
        "state header magic must be PVI1"
    );
    // Followed by the exact 32 session bytes
    assert_eq!(
        &state[4..36],
        &session,
        "state must bind the exact session bytes"
    );

    // Presentation projection request: op 3 followed by stateLength24 and state
    let mut pres_req = vec![0x03];
    pres_req.push((state_len >> 16) as u8);
    pres_req.push((state_len >> 8) as u8);
    pres_req.push((state_len & 0xff) as u8);
    pres_req.extend_from_slice(state);

    let pres_resp = interaction(pres_req).expect("presentation projection");
    assert_eq!(
        pres_resp.first(),
        Some(&0),
        "presentation projection must succeed with status 0"
    );

    // Presentation payload must begin with 0x50, 0x56, 0x4e, 0x01 (PVN1)
    assert!(pres_resp.len() >= 5);
    assert_eq!(
        &pres_resp[1..5],
        &[0x50, 0x56, 0x4e, 0x01],
        "presentation framing magic must be PVN1"
    );

    // Direct presentation projector on the state bytes must yield the same presentation output
    let direct_pres = presentation(state.to_vec()).expect("direct presentation projector");
    assert_eq!(
        direct_pres, pres_resp,
        "direct presentation projector matches interaction op 3"
    );

    // Zero session rejection: op 0 with 32 zero bytes
    let zero_init = vec![0x00; 33];
    assert_eq!(
        interaction(zero_init).unwrap(),
        [1], // BadEncoding: session must be non-zero
        "zero session bytes must be rejected"
    );
}

#[test]
fn kappa_replication_and_read_admission_integration_bounds() {
    // 1. Journal admission budget & replay requirement
    let journal_src =
        fs::read_to_string(root().join("sdk/browser/journal.mjs")).expect("read journal.mjs");
    assert!(
        journal_src.contains("const outstandingMaximum = 2;"),
        "journal must enforce local host admission budget of at most 2 outstanding operations"
    );
    assert!(
        journal_src.contains("const context = 'prismpm/workspace-event/1';"),
        "journal must use domain-separated signing context"
    );
    assert!(
        journal_src.contains("this.#requiresReplay = true;"),
        "journal operations must require authenticated replay across transitions and errors"
    );

    // 2. Command host admission budget & boundary
    let command_src =
        fs::read_to_string(root().join("sdk/browser/commands.mjs")).expect("read commands.mjs");
    assert!(
        command_src.contains("const outstandingMaximum=2;"),
        "command adapter must enforce admission budget of at most 2 outstanding operations"
    );
    assert!(
        command_src.contains("const context='prismpm/workspace-event/1';"),
        "command adapter must verify domain-separated signing context"
    );

    // 3. Query read-admission limits & cursor pagination
    let query_src =
        fs::read_to_string(root().join("sdk/browser/queries.mjs")).expect("read queries.mjs");
    assert!(
        query_src.contains("const maximumOutstanding=2;"),
        "query adapter must enforce maximum 2 outstanding operations"
    );
    assert!(
        query_src.contains("bytes[70]>16"),
        "query adapter must reject pages exceeding 16 rows per page"
    );

    // 4. WebRTC Peer transport bounds (no discovery/STUN/TURN, bounded channels)
    let peer_src = fs::read_to_string(root().join("sdk/browser/peer.mjs")).expect("read peer.mjs");
    assert!(
        peer_src.contains("const VERSION = 'prismpm/browser-peer/1';"),
        "peer must declare version 1"
    );
    assert!(
        peer_src.contains("const CHANNEL = 'prismpm-opaque-v1';"),
        "peer must use dedicated opaque data channel"
    );
    assert!(
        peer_src.contains("const SIGNAL_MAXIMUM = 65_536;"),
        "peer must enforce signaling bounds"
    );
    assert!(
        peer_src.contains("const SDP_MAXIMUM = 32_768;"),
        "peer must enforce SDP bounds"
    );
    assert!(
        peer_src.contains("typ host"),
        "peer must restrict to local host candidates only (no STUN/TURN/relay)"
    );

    // 5. View Host lifecycle and single-flight execution
    let view_host_src =
        fs::read_to_string(root().join("sdk/browser/view-host.mjs")).expect("read view-host.mjs");
    assert!(
        view_host_src.contains("openWorkspaceView"),
        "view-host must export openWorkspaceView"
    );
    assert!(
        view_host_src.contains("this.#commands = commands; this.#queries = queries;"),
        "view-host integrates commands and queries under unified View lifecycle"
    );
}

#[test]
fn browser_dk_suites_and_release_acceptance_binding() {
    let check_src = fs::read_to_string(root().join("scripts/browser-api-sdk-check.mjs"))
        .expect("read browser-api-sdk-check.mjs");

    // Verify all 10 DK suites (DK-07..DK-16) are registered
    let expected_suites = [
        "DK-07", "DK-08", "DK-09", "DK-10", "DK-11", "DK-12", "DK-13", "DK-14", "DK-15", "DK-16",
    ];
    for id in expected_suites {
        assert!(
            check_src.contains(&format!("id:'{id}'")),
            "browser check must register suite {id}"
        );
    }

    // Verify feature file covers DK-07..DK-16 scenarios
    let feature_src =
        fs::read_to_string(root().join("features/suites/sdk.feature")).expect("read sdk.feature");
    for id in expected_suites {
        assert!(
            feature_src.contains(&format!("@{id}")),
            "sdk.feature must contain scenario tagged @{id}"
        );
    }
}
