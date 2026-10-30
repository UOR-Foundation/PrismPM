//! Scratch diagnostic, not for merge: the counted entities of the root model.

use lexlean::{CheckRequest as LexCheckRequest, Engine, Selection};
use prismpm::holo::canonical::encode_canonical;
use prismpm::holo::projector::project_snapshot;
use std::path::Path;

#[test]
fn print_counted_entities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).unwrap();
    let project = camino::Utf8PathBuf::from_path_buf(root.join("lexlean.toml")).unwrap();
    let snapshot = Engine::load(&project)
        .unwrap()
        .snapshot(LexCheckRequest { selection: Selection::Entrypoints })
        .unwrap();
    let doc = project_snapshot(&snapshot).unwrap();
    let value: serde_json::Value =
        serde_json::from_slice(&encode_canonical(&doc).unwrap()).unwrap();
    let mut total = 0;
    for (section, fields) in [
        ("architecture", &["components", "edges", "stakeholders", "concerns", "viewpoints", "views"][..]),
        ("security", &["assets", "threats", "risks", "controls", "activities", "measurements"][..]),
        ("quality", &["characteristics", "subcharacteristics", "requirements", "measures"][..]),
    ] {
        for field in fields {
            let rows = value[section][field].as_array().cloned().unwrap_or_default();
            total += rows.len();
            let names: Vec<String> = rows
                .iter()
                .map(|row| {
                    row.get("id").or_else(|| row.get("name")).map(|v| v.to_string())
                        .unwrap_or_else(|| row.to_string().chars().take(120).collect())
                })
                .collect();
            println!("ENTITY {section}.{field} {} {:?}", rows.len(), names);
        }
    }
    println!("ENTITY total {total}");
}
