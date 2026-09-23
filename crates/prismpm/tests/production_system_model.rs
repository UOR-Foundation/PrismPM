//! Task 4: Production-system model, formal closure, validators, and deterministic projections.
//!
//! Validates:
//! 1. Complete production-system concepts across all 29 model collections.
//! 2. Global identity uniqueness and cross-collection collision rejection (PP2101).
//! 3. Acyclic dependency ordering and cycle/dangling reference rejection (PP2101).
//! 4. Formal validation certificate generation and verification.
//! 5. Deterministic standard-native projections (OpenAPI, AsyncAPI, CloudEvents, SPDX, OpenTelemetry, Compose, Kubernetes).
//! 6. prism-stdlib 0.2.0 packaging, licenses, and provenance.

use prismpm::contracts::CanonicalDocument;
use prismpm::system::{projections, validate, validation_certificate};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate is below repository root")
        .to_path_buf()
}

fn model_field_name(record: Option<&str>, name: &str) -> String {
    match name {
        "authentication" => return "auth".to_owned(),
        "defaultValue" => return "default".to_owned(),
        "fromComponent" if record == Some("Flow") => return "from".to_owned(),
        "interfaceId" if record == Some("Flow") => return "interface".to_owned(),
        "publiclyAccessible" => return "public".to_owned(),
        "toComponent" if record == Some("Flow") => return "to".to_owned(),
        "valueType" => return "type".to_owned(),
        _ => {}
    }
    let mut out = String::with_capacity(name.len() + 4);
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            out.push('_');
            out.push(character.to_ascii_lowercase());
        } else {
            out.push(character);
        }
    }
    out
}

fn normalize(value: &Value, record: Option<&str>) -> Value {
    match value {
        Value::Array(rows) => rows.iter().map(|row| normalize(row, record)).collect(),
        Value::Object(fields) => fields
            .iter()
            .map(|(name, value)| {
                (
                    model_field_name(record, name),
                    normalize(value, (name == "flows").then_some("Flow")),
                )
            })
            .collect(),
        _ => value.clone(),
    }
}

fn fixture_model() -> Value {
    let r = root();
    let fixture_path = r.join("crates/prismpm/src/system/release-fixture.json");
    let fixture: Value =
        serde_json::from_slice(&std::fs::read(&fixture_path).expect("read release fixture"))
            .expect("parse release fixture");
    let mut model = normalize(&fixture["model"], None);
    model["schema"] = json!("prismpm/system-model/1");
    CanonicalDocument::from_value("prismpm/system-model/1", model)
        .expect("canonical system-model document")
        .value()
        .clone()
}

fn formal_manifest(model: &Value) -> Value {
    let mut certificate = validation_certificate(model).expect("certificate");
    certificate.as_object_mut().unwrap().remove("schema");
    certificate
}

#[test]
fn test_production_system_concepts_and_all_collections() {
    let model = fixture_model();

    // Verify product definition
    assert_eq!(model["product"]["id"], "calculator-system");
    assert_eq!(model["product"]["version"], "A");

    // All 29 collections are present and populated
    const MODEL_ID_COLLECTIONS: [&str; 29] = [
        "acceptance",
        "alerts",
        "architecture",
        "artifacts",
        "backups",
        "calls",
        "capabilities",
        "components",
        "controls",
        "drifts",
        "events",
        "flows",
        "identity_requirements",
        "interfaces",
        "migrations",
        "parameters",
        "platform_requirements",
        "persistence",
        "retirements",
        "rollbacks",
        "rollouts",
        "scaling_policies",
        "schemas",
        "secret_references",
        "slis",
        "slos",
        "targets",
        "topology",
        "storage_classes",
    ];

    let mut all_ids = BTreeMap::new();
    let product_id = model["product"]["id"].as_str().unwrap();
    all_ids.insert(product_id.to_owned(), "product");

    for collection in MODEL_ID_COLLECTIONS {
        let rows = model[collection].as_array().expect("collection is array");
        for row in rows {
            let id = row["id"].as_str().expect("id is string");
            assert!(!id.is_empty(), "id must not be empty in {collection}");
            assert!(
                all_ids.insert(id.to_owned(), collection).is_none(),
                "duplicate ID {id} found in {collection}"
            );
        }
    }

    assert_eq!(all_ids.len(), 108, "total globally unique modeled entities");

    // Negative probe: duplicate across collections is rejected
    for (pointer, duplicate) in [
        ("/product/id", model["components"][0]["id"].clone()),
        ("/product/id", model["secret_references"][0]["id"].clone()),
        (
            "/secret_references/0/id",
            model["components"][0]["id"].clone(),
        ),
    ] {
        let mut tampered = model.clone();
        *tampered.pointer_mut(pointer).unwrap() = duplicate;
        let err = validation_certificate(&tampered).expect_err("collision must be rejected");
        assert_eq!(err.code, "PP2101");
    }
}

#[test]
fn test_production_system_acyclic_dependencies_and_cycle_rejection() {
    let model = fixture_model();
    let certificate = formal_manifest(&model);
    validate(&model, &certificate).expect("valid model passes validation");

    // Negative probe: introduce cycle in rollouts
    let mut cyclic = model.clone();
    if let Some(rollouts) = cyclic["rollouts"].as_array_mut() {
        if rollouts.len() >= 2 {
            let first_id = rollouts[0]["id"].as_str().unwrap().to_owned();
            let second_id = rollouts[1]["id"].as_str().unwrap().to_owned();
            rollouts[0]["depends_on"] = json!([second_id]);
            rollouts[1]["depends_on"] = json!([first_id]);
            let err = validation_certificate(&cyclic).expect_err("cycle must be rejected");
            assert_eq!(err.code, "PP2101");
        }
    }

    // Negative probe: introduce dangling dependency
    let mut dangling = model.clone();
    if let Some(components) = dangling["components"].as_array_mut() {
        if let Some(first) = components.first_mut() {
            first["depends_on"] = json!(["nonexistent-component-id"]);
            let err = validation_certificate(&dangling)
                .expect_err("dangling dependency must be rejected");
            assert_eq!(err.code, "PP2101");
        }
    }
}

#[test]
fn test_production_system_formal_validation_certificate() {
    let model = fixture_model();
    let certificate = formal_manifest(&model);

    assert_eq!(certificate["closure"]["bound"], 108);
    assert_eq!(
        certificate["uniqueness"]["values"],
        json!((0..108).collect::<Vec<_>>())
    );
    for relation in [
        "referential_integrity",
        "rollback_safety",
        "evidence_closure",
    ] {
        assert_eq!(certificate[relation]["bound"], 108);
    }

    validate(&model, &certificate).expect("validation certificate validates cleanly");

    // Tampered bounds fail closed
    for bound in [100, 107, 109] {
        let mut stale = certificate.clone();
        stale["closure"]["bound"] = json!(bound);
        let err = validate(&model, &stale).expect_err("stale bound must be rejected");
        assert_eq!(err.code, "PP2101");
    }

    // Incomplete uniqueness set fails closed
    let mut incomplete = certificate.clone();
    incomplete["uniqueness"]["values"]
        .as_array_mut()
        .unwrap()
        .pop();
    let err = validate(&model, &incomplete).expect_err("incomplete uniqueness must be rejected");
    assert_eq!(err.code, "PP2101");
}

#[test]
fn test_production_system_deterministic_projections() {
    let model = fixture_model();
    let doc =
        CanonicalDocument::from_value("prismpm/system-model/1", model).expect("canonical document");

    let first = projections(&doc, &[]).expect("generate projections");
    let second = projections(&doc, &[]).expect("generate projections again");

    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.media_type, b.media_type);
        assert_eq!(
            a.bytes, b.bytes,
            "projections must be deterministic: {}",
            a.path
        );
    }

    let paths = first.iter().map(|p| p.path.as_str()).collect::<Vec<_>>();
    assert!(paths.contains(&"projections/openapi.json"));
    assert!(paths.contains(&"projections/asyncapi.json"));
    assert!(paths.contains(&"projections/cloudevents.schema.json"));
    assert!(paths.contains(&"projections/spdx.json"));
    assert!(paths.contains(&"projections/opentelemetry-collector.json"));
    assert!(paths.contains(&"projections/compose.json"));
    assert!(paths.contains(&"projections/kubernetes.json"));
    assert!(paths.contains(&"projections/capability-coverage.json"));
    assert!(paths.contains(&"projections/system-validation-certificate.json"));
    assert!(paths.contains(&"projections/history.sql"));

    // Verify OpenAPI projection content
    let openapi = first
        .iter()
        .find(|p| p.path == "projections/openapi.json")
        .unwrap();
    let openapi_val: Value = serde_json::from_slice(&openapi.bytes).unwrap();
    assert_eq!(openapi_val["openapi"], "3.2.0");

    // Verify AsyncAPI projection content
    let asyncapi = first
        .iter()
        .find(|p| p.path == "projections/asyncapi.json")
        .unwrap();
    let asyncapi_val: Value = serde_json::from_slice(&asyncapi.bytes).unwrap();
    assert_eq!(asyncapi_val["asyncapi"], "3.1.0");

    // Verify CloudEvents projection content
    let cloudevents = first
        .iter()
        .find(|p| p.path == "projections/cloudevents.schema.json")
        .unwrap();
    let cloudevents_val: Value = serde_json::from_slice(&cloudevents.bytes).unwrap();
    assert_eq!(
        cloudevents_val["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );

    // Verify SPDX projection content
    let spdx = first
        .iter()
        .find(|p| p.path == "projections/spdx.json")
        .unwrap();
    let spdx_val: Value = serde_json::from_slice(&spdx.bytes).unwrap();
    assert_eq!(
        spdx_val["@graph"][0]["creationInfo"]["specVersion"],
        "3.0.1"
    );
}

#[test]
fn test_prism_stdlib_0_2_0_packaging_and_contracts() {
    let r = root();
    let stdlib_pkg = r.join("stdlib/generated/package");

    // Cargo.toml version is 0.2.0
    let cargo_toml = std::fs::read_to_string(stdlib_pkg.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains("version = \"0.2.0\""));
    assert!(cargo_toml.contains("name = \"prism-stdlib\""));

    // Generation manifest exists and binds inputs
    let manifest_bytes = std::fs::read(stdlib_pkg.join("generation-manifest.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert_eq!(manifest["schema"], "lean4-prod/cargo-package-manifest/1");
    assert_eq!(manifest["module"], "PrismPM");
    assert!(manifest["files"].as_array().unwrap().len() >= 6);

    // Licenses and README
    assert!(stdlib_pkg.join("LICENSE-MIT").is_file());
    assert!(stdlib_pkg.join("LICENSE-APACHE").is_file());
    assert!(stdlib_pkg.join("README.md").is_file());

    // Both crate archives exist
    assert!(r
        .join("stdlib/generated/prism-stdlib-0.1.0.crate")
        .is_file());
    assert!(r
        .join("stdlib/generated/prism-stdlib-0.2.0.crate")
        .is_file());

    // Lean production-system sources exist in stdlib/src/Production/
    for module in [
        "Core",
        "Interface",
        "Runtime",
        "Operations",
        "Validation",
        "System",
        "SystemValidation",
        "ControlCoverage",
    ] {
        assert!(
            r.join(format!("stdlib/src/Production/{module}.lex.tex"))
                .is_file(),
            "Production.{module}.lex.tex must be present"
        );
    }
}
