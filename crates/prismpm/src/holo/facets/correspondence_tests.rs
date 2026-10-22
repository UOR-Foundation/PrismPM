//! Foundation.Arch correspondences: projection and resolution of their claims.

use super::records::assemble;
use super::*;
use serde_json::json;

const MODULE: &str = "App.Architecture";

fn nat_field(name: &str, value: u64) -> Value {
    json!({"field": name, "value": {"kind": "nat", "value": value.to_string()}})
}

fn string_field(name: &str, value: &str) -> Value {
    json!({"field": name, "value": {"kind": "string", "value": value}})
}

fn claim_kind_field(constructor: &str) -> Value {
    json!({
        "field": "claimKind",
        "value": {"kind": "constructor", "constructor": {"name": constructor}},
    })
}

fn record(type_name: &str, fields: Vec<Value>) -> Value {
    json!({
        "result": {"member": {"name": type_name, "module": "Foundation.Arch"}},
        "body": {"kind": "record", "fields": fields},
    })
}

fn correspondence(concern: u64, view: u64, module: &str, name: &str, kind: &str) -> Value {
    record(
        "Correspondence",
        vec![
            nat_field("index", 0),
            nat_field("concernIndex", concern),
            nat_field("viewIndex", view),
            string_field("claimModule", module),
            string_field("claimName", name),
            claim_kind_field(kind),
        ],
    )
}

fn concern() -> Value {
    record(
        "Concern",
        vec![
            nat_field("index", 0),
            string_field("name", "Parity of web and CLI"),
            string_field("text", "Both give the same response."),
        ],
    )
}

fn view() -> Value {
    record(
        "View",
        vec![
            nat_field("index", 0),
            nat_field("viewpointIndex", 0),
            nat_field("modelKindIndex", 0),
        ],
    )
}

/// A model with one concern, one view, the given correspondence, the theorem
/// `Engine::parity` and the definition `Specification::dispatch`.
fn resolve(claim: &Value) -> Result<(), PrismError> {
    let concern = concern();
    let view = view();
    let opaque = json!({});
    let declared = [
        (MODULE, "concernParity", "definition", &concern),
        (MODULE, "viewParity", "definition", &view),
        (MODULE, "parityClaim", "definition", claim),
        ("Engine", "parity", "theorem", &opaque),
        ("Specification", "dispatch", "definition", &opaque),
    ]
    .map(|(module, logical_id, kind, linked)| Declared {
        module,
        logical_id,
        kind,
        linked,
    });
    resolve_correspondences(&table()?, &declared)
}

#[test]
fn a_correspondence_resolves_its_concern_view_and_claim() {
    resolve(&correspondence(
        0,
        0,
        "Engine",
        "parity",
        "ClaimKind.theoremClaim",
    ))
    .unwrap();
    resolve(&correspondence(
        0,
        0,
        "Specification",
        "dispatch",
        "Foundation.Arch::ClaimKind.definitionClaim",
    ))
    .unwrap();
}

#[test]
fn a_correspondence_naming_an_unknown_claim_is_refused() {
    let error = resolve(&correspondence(
        0,
        0,
        "Engine",
        "missing",
        "ClaimKind.theoremClaim",
    ))
    .unwrap_err();
    assert_eq!(error.code, "PP2013");
    assert!(
        error.message.contains("Engine::missing"),
        "{}",
        error.message
    );
    let error = resolve(&correspondence(
        0,
        0,
        "Nowhere",
        "parity",
        "ClaimKind.theoremClaim",
    ))
    .unwrap_err();
    assert_eq!(error.code, "PP2013");
}

#[test]
fn a_correspondence_claiming_the_wrong_declaration_kind_is_refused() {
    let error = resolve(&correspondence(
        0,
        0,
        "Specification",
        "dispatch",
        "ClaimKind.theoremClaim",
    ))
    .unwrap_err();
    assert_eq!(error.code, "PP2013");
    assert!(
        error.message.contains("which is a definition"),
        "{}",
        error.message
    );
}

#[test]
fn a_correspondence_naming_an_unknown_concern_or_view_is_refused() {
    for (concern, view) in [(1, 0), (0, 3)] {
        let error = resolve(&correspondence(
            concern,
            view,
            "Engine",
            "parity",
            "ClaimKind.theoremClaim",
        ))
        .unwrap_err();
        assert_eq!(error.code, "PP2013");
    }
}

#[test]
fn a_model_without_correspondences_is_not_checked() {
    let concern = concern();
    let declared = [Declared {
        module: MODULE,
        logical_id: "concernParity",
        kind: "definition",
        linked: &concern,
    }];
    resolve_correspondences(&table().unwrap(), &declared).unwrap();
}

#[test]
fn foundation_sample_rows_are_not_resolved() {
    // A vendored Foundation.Arch copy declares records of its own types; only
    // application modules are checked.
    let claim = correspondence(9, 9, "Nowhere", "nothing", "ClaimKind.theoremClaim");
    let declared = [Declared {
        module: "Foundation.Arch",
        logical_id: "sample",
        kind: "definition",
        linked: &json!({
            "result": {"member": {"name": "Correspondence"}},
            "body": claim["body"].clone(),
        }),
    }];
    resolve_correspondences(&table().unwrap(), &declared).unwrap();
}

#[test]
fn correspondences_are_projected_and_omitted_when_absent() {
    let table = table().unwrap();
    let claim = correspondence(0, 0, "Engine", "parity", "ClaimKind.theoremClaim");
    let concern = concern();
    let definitions = [
        (MODULE, "concernParity", &concern),
        (MODULE, "parityClaim", &claim),
    ];
    let entities = collect_entities(&table, definitions).unwrap();
    let records = assemble(&table, BTreeMap::new(), entities)
        .unwrap()
        .architecture;
    let row = &records.correspondences[0];
    assert_eq!(row.id, "App.Architecture::parityClaim");
    assert_eq!(
        (row.concern_index, row.view_index),
        (0, 0),
        "indexes are carried unchanged"
    );
    assert_eq!(
        (row.claim_module.as_str(), row.claim_name.as_str()),
        ("Engine", "parity")
    );
    assert_eq!(row.claim_kind, "theorem");

    let entities = collect_entities(&table, [(MODULE, "concernParity", &concern)]).unwrap();
    let records = assemble(&table, BTreeMap::new(), entities)
        .unwrap()
        .architecture;
    let value = serde_json::to_value(&records).unwrap();
    assert!(value.get("correspondences").is_none(), "{value}");
}

#[test]
fn an_unknown_claim_kind_is_refused() {
    let table = table().unwrap();
    let claim = correspondence(0, 0, "Engine", "parity", "ClaimKind.lemma");
    let entities = collect_entities(&table, [(MODULE, "parityClaim", &claim)]).unwrap();
    let error = assemble(&table, BTreeMap::new(), entities).err().unwrap();
    assert_eq!(error.code, "PP2013");
}
