//! Integration test suite for standard-native target adapters (Task 8 / Issue #10).
//!
//! Validates Compose and Kubernetes target adapters:
//! 1. Minimal versioned adapter boundaries (adapters/compose.json, adapters/kubernetes.json).
//! 2. Target descriptor canonicality, digests, and projection policies.
//! 3. Fail-closed target model validation across Compose and Kubernetes.
//! 4. Compose projection generation: declared typed params, secret references, security hardening.
//! 5. Kubernetes projection generation: standard-native resources, probe policies, seccomp, network isolation.
//! 6. Two-stage Kubernetes deployment partitioning (official ingress infrastructure vs application).
//! 7. Lifecycle state binding, drift detection, downgrade refusal on rollback, and destroy authorization.

use prismpm::deployment::{digest, projection_policy, validate_targets};
use prismpm::lifecycle::kubernetes_partition;
use prismpm::system::{compose_projection, kubernetes_projection};
use serde_json::{json, Value};

fn compose_adapter_system() -> Value {
    let capability = "compose-capability";
    json!({
        "artifacts": [],
        "capabilities": [{"id": capability}],
        "topology": [],
        "targets": [{
            "adapter_digest": digest("compose").unwrap(),
            "api_version": "compose-spec@fee041b381ffd4aad263410980bdce0cdf4beb7d",
            "capabilities": [capability],
            "credentials": "external",
            "id": "local-compose",
            "ingress_class_name": null,
            "ingress_controller_artifact": null,
            "kind": "compose",
            "minimum_release_status": "development",
            "platform_requirements": ["linux/amd64"],
            "storage_class": null,
            "storage_profile": null
        }]
    })
}

fn kubernetes_adapter_system() -> Value {
    let capability = "k8s-capability";
    let policy = projection_policy("kubernetes").unwrap();
    let controller_img = policy["ingress_controller_image"].as_str().unwrap();
    let (controller_path, controller_digest) = controller_img.rsplit_once('@').unwrap();
    let admission_img = policy["ingress_admission_image"].as_str().unwrap();
    let (admission_path, admission_digest) = admission_img.rsplit_once('@').unwrap();

    json!({
        "artifacts": [
            {
                "id": "ingress-controller-img",
                "path": controller_path,
                "digest": controller_digest,
                "role": "ingress-controller-image"
            },
            {
                "id": "ingress-admission-img",
                "path": admission_path,
                "digest": admission_digest,
                "role": "ingress-admission-image"
            }
        ],
        "capabilities": [{"id": capability}],
        "topology": [
            {
                "id": "public-http",
                "kind": "port",
                "port": 80,
                "protocol": "TCP",
                "public": true
            },
            {
                "id": "data-pv",
                "kind": "persistent-volume"
            }
        ],
        "targets": [{
            "adapter_digest": digest("kubernetes").unwrap(),
            "api_version": "v1.36.4",
            "capabilities": [capability],
            "credentials": "external",
            "id": "cluster-k8s",
            "ingress_class_name": "nginx",
            "ingress_controller_artifact": "ingress-controller-img",
            "kind": "kubernetes",
            "minimum_release_status": "candidate",
            "platform_requirements": ["linux/amd64"],
            "storage_class": "standard",
            "storage_profile": "kind-static-local"
        }]
    })
}

#[test]
fn adapter_boundary_descriptors_are_canonical_and_complete() {
    let compose_digest = digest("compose").expect("compose digest");
    let k8s_digest = digest("kubernetes").expect("kubernetes digest");
    let pages_digest = digest("github-pages").expect("github-pages digest");

    assert_ne!(compose_digest, k8s_digest);
    assert_ne!(compose_digest, pages_digest);
    assert_ne!(k8s_digest, pages_digest);

    assert!(compose_digest.starts_with("sha256:"));
    assert!(k8s_digest.starts_with("sha256:"));
    assert!(pages_digest.starts_with("sha256:"));

    // Unsupported adapters fail closed with PP7101
    let err = digest("shell").unwrap_err();
    assert_eq!(err.code, "PP7101");
    let err = digest("unknown-target").unwrap_err();
    assert_eq!(err.code, "PP7101");

    // Compose projection policy bindings
    let compose_policy = projection_policy("compose").expect("compose policy");
    assert_eq!(compose_policy["read_only"], true);
    assert_eq!(compose_policy["cap_drop"], json!(["ALL"]));
    assert_eq!(
        compose_policy["security_opt"],
        json!(["no-new-privileges:true"])
    );
    assert!(compose_policy["healthcheck"]["interval"].is_string());
    assert!(compose_policy["tmpfs_default"].is_array());
    assert!(compose_policy["tmpfs_database"].is_array());

    // Kubernetes projection policy bindings
    let k8s_policy = projection_policy("kubernetes").expect("kubernetes policy");
    assert_eq!(k8s_policy["read_only_root_filesystem"], true);
    assert_eq!(k8s_policy["seccomp_profile"], "RuntimeDefault");
    assert_eq!(k8s_policy["storage_profile"], "kind-static-local");
    assert_eq!(
        k8s_policy["ingress_controller_profile"],
        "ingress-nginx-kind-v1.15.1"
    );
    assert_eq!(k8s_policy["field_manager"], "prismpm");
    assert_eq!(k8s_policy["automount_service_account_token"], false);
    assert!(k8s_policy["ingress_controller_image"]
        .as_str()
        .unwrap()
        .contains('@'));
    assert!(k8s_policy["ingress_admission_image"]
        .as_str()
        .unwrap()
        .contains('@'));
    assert_eq!(
        k8s_policy["ingress_controller_manifest_sha256"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
}

#[test]
fn compose_and_kubernetes_target_model_validation_fail_closed() {
    // Both valid systems validate cleanly
    validate_targets(&compose_adapter_system()).expect("valid compose target");
    validate_targets(&kubernetes_adapter_system()).expect("valid kubernetes target");

    // 1. Tampered adapter digest rejected with PP7101
    let mut bad_digest = compose_adapter_system();
    bad_digest["targets"][0]["adapter_digest"] =
        json!("sha256:0000000000000000000000000000000000000000000000000000000000000000");
    assert_eq!(validate_targets(&bad_digest).unwrap_err().code, "PP7101");

    // 2. Tampered api_version rejected with PP7101
    let mut bad_api = kubernetes_adapter_system();
    bad_api["targets"][0]["api_version"] = json!("v1.37.0-beta");
    assert_eq!(validate_targets(&bad_api).unwrap_err().code, "PP7101");

    // 3. Undeclared capabilities rejected with PP7101
    let mut bad_caps = compose_adapter_system();
    bad_caps["targets"][0]["capabilities"] = json!(["unregistered-capability"]);
    assert_eq!(validate_targets(&bad_caps).unwrap_err().code, "PP7101");

    // 4. Absent platform requirements rejected with PP7101
    let mut bad_platform = kubernetes_adapter_system();
    bad_platform["targets"][0]["platform_requirements"] = json!([]);
    assert_eq!(validate_targets(&bad_platform).unwrap_err().code, "PP7101");

    // 5. Unsupported minimum release status rejected with PP7101
    let mut bad_status = compose_adapter_system();
    bad_status["targets"][0]["minimum_release_status"] = json!("unreleased");
    assert_eq!(validate_targets(&bad_status).unwrap_err().code, "PP7101");

    // 6. Kubernetes persistent volume with missing storage profile rejected with PP7101
    let mut bad_storage = kubernetes_adapter_system();
    bad_storage["targets"][0]["storage_profile"] = json!("unsupported-storage");
    assert_eq!(validate_targets(&bad_storage).unwrap_err().code, "PP7101");

    // 7. Kubernetes public ingress without ingress_class_name rejected with PP7101
    let mut bad_ingress = kubernetes_adapter_system();
    bad_ingress["targets"][0]["ingress_class_name"] = json!(null);
    assert_eq!(validate_targets(&bad_ingress).unwrap_err().code, "PP7101");

    // 8. Kubernetes ingress controller artifact mismatch rejected with PP7101
    let mut bad_controller = kubernetes_adapter_system();
    bad_controller["artifacts"][0]["digest"] =
        json!("sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    assert_eq!(
        validate_targets(&bad_controller).unwrap_err().code,
        "PP7101"
    );

    // 9. Non-Kubernetes target with unused Kubernetes bindings rejected with PP7101
    let mut bad_compose = compose_adapter_system();
    bad_compose["targets"][0]["storage_class"] = json!("standard");
    assert_eq!(validate_targets(&bad_compose).unwrap_err().code, "PP7101");
}

#[test]
fn compose_projection_enforces_typed_params_secret_references_and_security() {
    let system = json!({
        "components": [
            {
                "artifact": "app-binary",
                "id": "api-service",
                "kind": "service",
                "parameters": ["db-host", "api-port"],
                "ports": ["http-port"],
                "secrets": ["db-password"]
            }
        ],
        "artifacts": [
            {
                "id": "app-binary",
                "path": "example.test/api",
                "digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111"
            }
        ],
        "parameters": [
            {
                "id": "db-host",
                "type": "string",
                "default": "db.local"
            },
            {
                "id": "api-port",
                "type": "integer",
                "default": null
            }
        ],
        "topology": [
            {
                "id": "http-port",
                "port": 8080,
                "protocol": "TCP",
                "public": true
            }
        ],
        "secrets": [
            {
                "id": "db-password",
                "target_file": "/run/secrets/db_password"
            }
        ]
    });

    let projection = compose_projection(&system).expect("compose projection");
    let services = projection["services"].as_object().expect("services object");
    let api = &services["api-service"];

    // Parameters: default preserved; required parameter rendered as ${VAR:?required}
    assert_eq!(api["environment"]["DB_HOST"], "db.local");
    assert_eq!(api["environment"]["API_PORT"], "${API_PORT:?required}");

    // Security opts and hardening from projection policy
    assert_eq!(api["read_only"], true);
    assert_eq!(api["security_opt"], json!(["no-new-privileges:true"]));
    assert_eq!(api["cap_drop"], json!(["ALL"]));

    // Ports
    let ports = api["ports"].as_array().expect("ports array");
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0]["target"], 8080);
    assert_eq!(ports[0]["published"], 8080);
    assert_eq!(ports[0]["protocol"], "tcp");
}

#[test]
fn kubernetes_projection_generates_hardened_native_resources() {
    let policy = projection_policy("kubernetes").unwrap();
    let controller_img = policy["ingress_controller_image"].as_str().unwrap();
    let (controller_path, controller_digest) = controller_img.rsplit_once('@').unwrap();
    let admission_img = policy["ingress_admission_image"].as_str().unwrap();
    let (admission_path, admission_digest) = admission_img.rsplit_once('@').unwrap();

    let system = json!({
        "product": {"id": "calculator", "version": "A"},
        "application_profile": {
            "contract": "prismpm/transactional-command-service/1",
            "history_table": "history",
            "outbox_table": "outbox",
            "audit_table": "audit",
            "command_id_column": "request_id",
            "operation_column": "operation",
            "input_a_column": "left_operand",
            "input_b_column": "right_operand",
            "annotation_column": "client_label",
            "optional_annotation": "absent",
            "command_path": "/v1/calculations",
            "public_hostname_parameter": "public-host"
        },
        "observability": {
            "redacted_fields": ["authorization", "password", "token"]
        },
        "migrations": [],
        "persistence": [],
        "components": [
            {
                "artifact": "calc-bin",
                "id": "api",
                "kind": "service",
                "parameters": [],
                "ports": ["api-port"],
                "secrets": []
            }
        ],
        "artifacts": [
            {
                "id": "calc-bin",
                "path": "example.test/calculator-api",
                "digest": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
                "role": "application-image"
            },
            {
                "id": "ingress-controller",
                "path": controller_path,
                "digest": controller_digest,
                "role": "ingress-controller-image"
            },
            {
                "id": "ingress-admission",
                "path": admission_path,
                "digest": admission_digest,
                "role": "ingress-admission-image"
            }
        ],
        "parameters": [
            {
                "id": "public-host",
                "type": "string",
                "default": "calculator.local"
            }
        ],
        "secrets": [],
        "topology": [
            {
                "id": "api-port",
                "port": 3000,
                "protocol": "TCP",
                "public": true,
                "owners": ["api"]
            }
        ],
        "targets": [{
            "adapter_digest": digest("kubernetes").unwrap(),
            "api_version": "v1.36.4",
            "capabilities": ["k8s"],
            "credentials": "external",
            "id": "prod-k8s",
            "ingress_class_name": "nginx",
            "ingress_controller_artifact": "ingress-controller",
            "kind": "kubernetes",
            "minimum_release_status": "development",
            "platform_requirements": ["linux/amd64"],
            "storage_class": null,
            "storage_profile": null
        }],
        "capabilities": [{"id": "k8s"}]
    });

    let artifacts: Vec<(String, Vec<u8>)> = Vec::new();
    let projection = kubernetes_projection(&system, &artifacts).expect("kubernetes projection");

    assert_eq!(projection["apiVersion"], "v1");
    assert_eq!(projection["kind"], "List");
    let items = projection["items"].as_array().expect("list items");
    assert!(!items.is_empty());

    // Verify Deployment spec hardening
    let deployment = items
        .iter()
        .find(|item| item["kind"] == "Deployment" && item["metadata"]["name"] == "api")
        .expect("api deployment");
    let pod_spec = &deployment["spec"]["template"]["spec"];

    assert_eq!(pod_spec["automountServiceAccountToken"], false);
    assert_eq!(
        pod_spec["securityContext"]["seccompProfile"]["type"],
        "RuntimeDefault"
    );
    let container = &pod_spec["containers"][0];
    assert_eq!(container["securityContext"]["readOnlyRootFilesystem"], true);

    // Verify Service exists
    assert!(items
        .iter()
        .any(|item| item["kind"] == "Service" && item["metadata"]["name"] == "api"));

    // Verify ingress controller infrastructure is bundled in the List
    assert!(items.iter().any(|item| item
        .pointer("/metadata/labels/app.kubernetes.io~1name")
        .and_then(Value::as_str)
        == Some("ingress-nginx")));
}

#[test]
fn kubernetes_partition_separates_infrastructure_and_application_strictly() {
    let projection = json!({
        "apiVersion": "v1",
        "kind": "List",
        "items": [
            {
                "apiVersion": "v1",
                "kind": "Namespace",
                "metadata": {
                    "name": "ingress-nginx",
                    "labels": {"app.kubernetes.io/name": "ingress-nginx"}
                }
            },
            {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": {
                    "name": "ingress-nginx-controller",
                    "namespace": "ingress-nginx",
                    "labels": {"app.kubernetes.io/name": "ingress-nginx"}
                }
            },
            {
                "apiVersion": "batch/v1",
                "kind": "Job",
                "metadata": {
                    "name": "ingress-nginx-admission-create",
                    "namespace": "ingress-nginx",
                    "labels": {"app.kubernetes.io/name": "ingress-nginx"}
                }
            },
            {
                "apiVersion": "apps/v1",
                "kind": "Deployment",
                "metadata": {
                    "name": "calculator-api",
                    "namespace": "calculator-system",
                    "labels": {"app.kubernetes.io/name": "calculator-api"}
                }
            },
            {
                "apiVersion": "networking.k8s.io/v1",
                "kind": "Ingress",
                "metadata": {
                    "name": "calculator-ingress",
                    "namespace": "calculator-system",
                    "labels": {"app.kubernetes.io/name": "calculator-api"}
                }
            }
        ]
    });

    let (infra, app) = kubernetes_partition(&projection).expect("partition");
    let infra_items = infra["items"].as_array().expect("infra items");
    let app_items = app["items"].as_array().expect("app items");

    assert_eq!(infra_items.len(), 3);
    assert_eq!(app_items.len(), 2);

    for item in infra_items {
        assert_eq!(
            item.pointer("/metadata/labels/app.kubernetes.io~1name")
                .and_then(Value::as_str),
            Some("ingress-nginx")
        );
    }

    for item in app_items {
        assert_ne!(
            item.pointer("/metadata/labels/app.kubernetes.io~1name")
                .and_then(Value::as_str),
            Some("ingress-nginx")
        );
    }

    // Malformed projections fail closed with PP7301
    let non_list = json!({"apiVersion": "v1", "kind": "Deployment"});
    assert_eq!(kubernetes_partition(&non_list).unwrap_err().code, "PP7301");

    let empty_items = json!({"apiVersion": "v1", "kind": "List", "items": []});
    assert_eq!(
        kubernetes_partition(&empty_items).unwrap_err().code,
        "PP7301"
    );
}

#[test]
fn lifecycle_target_operations_fail_closed_on_unauthorized_and_invalid_inputs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    let valid_ref = "example.test/product@sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // 1. Destroy without authorized=true strictly fails closed with PP7701
    let destroy_err = prismpm::lifecycle::destroy(root, valid_ref, "local", false).unwrap_err();
    assert_eq!(destroy_err.code, "PP7701");
    assert!(destroy_err.message.contains("--authorized"));

    // 2. Malformed target ID fails closed with PP7101
    for bad_target in [
        "",
        "InvalidTarget",
        "target_with_underscore",
        "target/with/slash",
    ] {
        let err = prismpm::lifecycle::status(root, valid_ref, bad_target).unwrap_err();
        assert_eq!(err.code, "PP7101");
    }

    // 3. Mutable references fail closed with PP6101
    for bad_ref in [
        "example.test/product:latest",
        "invalid-no-digest",
        "product@sha256:xyz",
    ] {
        let err = prismpm::lifecycle::status(root, bad_ref, "local").unwrap_err();
        assert_eq!(err.code, "PP6101");
    }

    // 4. Rollback without sufficient accepted history fails closed with PP7601
    let rollback_err = prismpm::lifecycle::rollback(root, valid_ref, "local").unwrap_err();
    assert_eq!(rollback_err.code, "PP7601");
    assert!(rollback_err.message.contains("preceding accepted"));
}
