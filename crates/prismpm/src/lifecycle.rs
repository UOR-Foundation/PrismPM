//! Digest-only planning, deployment, drift, rollback, and local execution.

use crate::contracts::CanonicalDocument;
use crate::error::PrismError;
use crate::holo::canonical::encode_value;
use crate::oci;
use base64::Engine;
use fs4::fs_std::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once};
use std::thread;
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const DEPLOYMENT_EVIDENCE: &str = "application/vnd.prismpm.deployment.evidence.v1+json";
static RUN_CANCELLED: AtomicBool = AtomicBool::new(false);
static SIGNAL_HANDLER: Once = Once::new();
static SIGNAL_HANDLER_FAILED: AtomicBool = AtomicBool::new(false);

struct OperationGuard(File);

impl Drop for OperationGuard {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

fn operation_guard(root: &Path, target: &str) -> Result<OperationGuard, PrismError> {
    target_id(target)?;
    let directory = root.join(".prism/operations");
    std::fs::create_dir_all(&directory)
        .map_err(|error| PrismError::new("PP7201", format!("operation lock: {error}")))?;
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join(format!("{target}.lock")))
        .map_err(|error| PrismError::new("PP7201", format!("operation lock: {error}")))?;
    file.try_lock_exclusive().map_err(|_| {
        PrismError::new(
            "PP7201",
            format!("another lifecycle operation owns target {target}"),
        )
    })?;
    Ok(OperationGuard(file))
}

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn root_digest(reference: &str) -> Result<&str, PrismError> {
    oci::validate_reference(reference, true)
}

fn reference_with_digest(reference: &str, digest: &str) -> Result<String, PrismError> {
    root_digest(reference)?;
    let (name, _) = reference
        .rsplit_once('@')
        .ok_or_else(|| PrismError::new("PP6101", "immutable OCI reference has no digest"))?;
    if !digest.starts_with("sha256:") {
        return Err(PrismError::new("PP6101", "release digest is malformed"));
    }
    let previous = format!("{name}@{digest}");
    root_digest(&previous)?;
    Ok(previous)
}

fn target_id(value: &str) -> Result<&str, PrismError> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().enumerate().all(|(index, byte)| {
            if index == 0 {
                byte.is_ascii_lowercase()
            } else {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
            }
        })
    {
        return Err(PrismError::new("PP7101", "target ID is malformed"));
    }
    Ok(value)
}

fn system(root: &Path, digest: &str) -> Result<Value, PrismError> {
    let bytes = oci::artifact(root, digest, "system.prism.json")?;
    Ok(CanonicalDocument::parse("prismpm/system-model/1", &bytes)?
        .value()
        .clone())
}

fn target<'a>(system: &'a Value, id: &str) -> Result<&'a Value, PrismError> {
    system["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|row| row["id"] == id)
        .ok_or_else(|| PrismError::new("PP7101", format!("release does not model target {id}")))
}

fn projection(root: &Path, digest: &str, kind: &str) -> Result<(String, Vec<u8>), PrismError> {
    let title = match kind {
        "compose" => "projections/compose.json",
        "kubernetes" => "projections/kubernetes.json",
        "github-pages" => {
            return Err(PrismError::new(
                "PP7101",
                "GitHub Pages publication uses its protected Pages workflow",
            ))
        }
        _ => return Err(PrismError::new("PP7101", "target adapter is unsupported")),
    };
    Ok((title.to_owned(), oci::artifact(root, digest, title)?))
}

fn state_path(root: &Path, target: &str) -> PathBuf {
    root.join(".prism/targets").join(target).join("state.json")
}

fn history_path(root: &Path, target: &str) -> PathBuf {
    root.join(".prism/targets")
        .join(target)
        .join("history.json")
}

fn empty_observation(target: &str) -> Result<Vec<u8>, PrismError> {
    encode_value(&json!({"resources":{},"schema":"prismpm/target-observation/1","target":target}))
}

fn recorded_state(root: &Path, target: &str) -> Result<(Vec<u8>, Option<Value>), PrismError> {
    let path = state_path(root, target);
    if !path.exists() {
        return Ok((empty_observation(target)?, None));
    }
    let bytes = std::fs::read(&path)
        .map_err(|error| PrismError::new("PP7201", format!("target state: {error}")))?;
    let state = CanonicalDocument::parse("prismpm/deployment-state/1", &bytes)?;
    Ok((bytes, Some(state.value().clone())))
}

fn projection_path(root: &Path, target: &str) -> PathBuf {
    root.join(".prism/targets")
        .join(target)
        .join("projection.json")
}

fn persist_projection(root: &Path, target: &str, bytes: &[u8]) -> Result<PathBuf, PrismError> {
    let path = projection_path(root, target);
    std::fs::create_dir_all(path.parent().expect("projection parent"))
        .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    if path.exists()
        && std::fs::read(&path).map_err(|error| PrismError::new("PP7301", error.to_string()))?
            == bytes
    {
        return Ok(path);
    }
    atomic(&path, bytes)?;
    Ok(path)
}

fn adapter_environment() -> Result<BTreeMap<String, String>, PrismError> {
    let mut environment = BTreeMap::new();
    for name in ["DOCKER_CONFIG", "KUBECONFIG", "PRISMPM_SECRET_DIR"] {
        if let Some(value) = std::env::var_os(name) {
            let value = value
                .into_string()
                .map_err(|_| PrismError::new("PP7801", format!("{name} is not UTF-8")))?;
            if value.is_empty() || value.bytes().any(|byte| byte.is_ascii_control()) {
                return Err(PrismError::new("PP7801", format!("{name} is malformed")));
            }
            environment.insert(name.to_owned(), value);
        }
    }
    Ok(environment)
}

fn adapter_process(
    root: &Path,
    tool: &str,
    arguments: Vec<String>,
) -> Result<crate::verification::ProcessRecord, PrismError> {
    let executable = crate::sdk::executable(tool)?;
    let environment = adapter_environment()?;
    crate::verification::run_process(
        tool,
        &executable,
        &arguments,
        root,
        &environment,
        &[(root, "$PROJECT")],
        "PP7501",
    )
}

fn adapter_process_limited(
    root: &Path,
    tool: &str,
    arguments: Vec<String>,
    output_limit: usize,
) -> Result<crate::verification::ProcessRecord, PrismError> {
    let executable = crate::sdk::executable(tool)?;
    let environment = adapter_environment()?;
    crate::verification::run_process_limited(
        tool,
        &executable,
        &arguments,
        root,
        &environment,
        &[(root, "$PROJECT")],
        "PP7901",
        "300s",
        output_limit,
    )
}

fn compose_observation(root: &Path, target: &str, projection: &Path) -> Result<Value, PrismError> {
    let record = adapter_process(
        root,
        "docker",
        vec![
            "compose".to_owned(),
            "--project-name".to_owned(),
            target.to_owned(),
            "--file".to_owned(),
            projection.to_string_lossy().into_owned(),
            "ps".to_owned(),
            "--format".to_owned(),
            "json".to_owned(),
        ],
    )?;
    let text = record.stdout.trim();
    let rows = if text.is_empty() {
        Vec::new()
    } else if let Ok(rows) = serde_json::from_str::<Vec<Value>>(text) {
        rows
    } else {
        text.lines()
            .map(|line| {
                serde_json::from_str(line).map_err(|error| {
                    PrismError::new("PP7501", format!("Compose observation: {error}"))
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let mut normalized = rows
        .into_iter()
        .map(|row| {
            json!({
                "health":row.get("Health").or_else(||row.get("health")).cloned().unwrap_or(Value::Null),
                "image":row.get("Image").or_else(||row.get("image")).cloned().unwrap_or(Value::Null),
                "name":row.get("Name").or_else(||row.get("name")).cloned().unwrap_or(Value::Null),
                "service":row.get("Service").or_else(||row.get("service")).cloned().unwrap_or(Value::Null),
                "state":row.get("State").or_else(||row.get("state")).cloned().unwrap_or(Value::Null)
            })
        })
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    Ok(
        json!({"kind":"compose","resources":normalized,"schema":"prismpm/target-observation/1","target":target}),
    )
}

fn kubernetes_observation(root: &Path, target: &str, namespace: &str) -> Result<Value, PrismError> {
    let namespace_record = adapter_process(
        root,
        "kubectl",
        vec![
            "get".to_owned(),
            "namespace".to_owned(),
            namespace.to_owned(),
            "--ignore-not-found".to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ],
    )?;
    if namespace_record.stdout.trim().is_empty() {
        return Ok(
            json!({"kind":"kubernetes","resources":[],"schema":"prismpm/target-observation/1","target":target}),
        );
    }
    let record = adapter_process(
        root,
        "kubectl",
        vec![
            "get".to_owned(),
            "all,configmap,pvc,networkpolicy,role,rolebinding,serviceaccount,ingress,poddisruptionbudget".to_owned(),
            "--namespace".to_owned(),
            namespace.to_owned(),
            "--output".to_owned(),
            "json".to_owned(),
        ],
    )?;
    let document: Value = serde_json::from_str(record.stdout.trim())
        .map_err(|error| PrismError::new("PP7501", format!("Kubernetes observation: {error}")))?;
    let mut rows = document["items"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|row| {
            json!({
                "apiVersion":row["apiVersion"],
                "kind":row["kind"],
                "name":row["metadata"]["name"],
                "spec":row["spec"],
                "status":row["status"]
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        (left["kind"].as_str(), left["name"].as_str())
            .cmp(&(right["kind"].as_str(), right["name"].as_str()))
    });
    Ok(
        json!({"kind":"kubernetes","resources":rows,"schema":"prismpm/target-observation/1","target":target}),
    )
}

fn live_observation(
    root: &Path,
    target: &str,
    kind: &str,
    model: &Value,
    projection: &Path,
) -> Result<Vec<u8>, PrismError> {
    let value = match kind {
        "compose" => compose_observation(root, target, projection)?,
        "kubernetes" => kubernetes_observation(
            root,
            target,
            model["product"]["id"].as_str().unwrap_or_default(),
        )?,
        _ => return Err(PrismError::new("PP7101", "unsupported observation adapter")),
    };
    encode_value(&value)
}

fn observation_ready(bytes: &[u8]) -> Result<bool, PrismError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| PrismError::new("PP7401", format!("target observation: {error}")))?;
    let rows = value["resources"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7401", "target observation has no resources"))?;
    if rows.is_empty() {
        return Ok(false);
    }
    match value["kind"].as_str() {
        Some("compose") => Ok(rows.iter().all(|row| {
            row["state"]
                .as_str()
                .is_some_and(|state| state.eq_ignore_ascii_case("running"))
                && row["health"].as_str().is_none_or(|health| {
                    health.is_empty() || health.eq_ignore_ascii_case("healthy")
                })
        })),
        Some("kubernetes") => Ok(rows.iter().all(|row| match row["kind"].as_str() {
            Some("Deployment") => {
                row["status"]["readyReplicas"].as_u64().unwrap_or(0)
                    >= row["spec"]["replicas"].as_u64().unwrap_or(1)
            }
            Some("StatefulSet") => {
                row["status"]["readyReplicas"].as_u64().unwrap_or(0)
                    >= row["spec"]["replicas"].as_u64().unwrap_or(1)
            }
            Some("Job") => row["status"]["succeeded"].as_u64().unwrap_or(0) >= 1,
            Some("Pod") => matches!(
                row["status"]["phase"].as_str(),
                Some("Running" | "Succeeded")
            ),
            _ => true,
        })),
        _ => Err(PrismError::new(
            "PP7401",
            "target observation kind is unsupported",
        )),
    }
}

fn wait_ready(root: &Path, kind: &str, model: &Value) -> Result<(), PrismError> {
    if kind != "kubernetes" {
        return Ok(());
    }
    let namespace = model["product"]["id"].as_str().unwrap_or_default();
    for arguments in [
        vec![
            "wait",
            "--for=condition=Available",
            "deployment",
            "--all",
            "--timeout=300s",
        ],
        vec![
            "rollout",
            "status",
            "statefulset",
            "--all",
            "--timeout=300s",
        ],
        vec![
            "wait",
            "--for=condition=complete",
            "job",
            "--all",
            "--timeout=300s",
        ],
    ] {
        let mut arguments = arguments.into_iter().map(str::to_owned).collect::<Vec<_>>();
        arguments.push("--namespace".to_owned());
        arguments.push(namespace.to_owned());
        adapter_process(root, "kubectl", arguments)?;
    }
    Ok(())
}

fn acceptance_runner<'a>(
    model: &'a Value,
    target_name: &str,
) -> Result<(&'a str, Vec<String>), PrismError> {
    let row = model["acceptance"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|row| {
            row["target"] == target_name
                && matches!(row["kind"].as_str(), Some("positive" | "load"))
                && row["command"].as_array().is_some_and(|command| {
                    command
                        .iter()
                        .any(|argument| argument.as_str() == Some("acceptance"))
                })
        })
        .ok_or_else(|| {
            PrismError::new(
                "PP7401",
                "target has no modeled executable post-deployment acceptance",
            )
        })?;
    let component = row["component"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7401", "acceptance component is absent"))?;
    let arguments = row["command"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|argument| {
            argument.as_str().map(str::to_owned).ok_or_else(|| {
                PrismError::new("PP7401", "acceptance argv contains a non-string value")
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if arguments.is_empty()
        || arguments
            .iter()
            .any(|argument| argument.bytes().any(|byte| byte.is_ascii_control()))
    {
        return Err(PrismError::new(
            "PP7401",
            "acceptance argv is absent or malformed",
        ));
    }
    Ok((component, arguments))
}

fn acceptance_process(
    root: &Path,
    kind: &str,
    target_name: &str,
    target_profile: &str,
    model: &Value,
    projection: &Path,
) -> Result<crate::verification::ProcessRecord, PrismError> {
    let (component, command) = acceptance_runner(model, target_profile)?;
    let (tool, arguments) = match kind {
        "compose" => {
            let mut arguments = vec![
                "compose".to_owned(),
                "--project-name".to_owned(),
                target_name.to_owned(),
                "--file".to_owned(),
                projection.to_string_lossy().into_owned(),
                "exec".to_owned(),
                "-T".to_owned(),
                component.to_owned(),
            ];
            arguments.extend(command);
            ("docker", arguments)
        }
        "kubernetes" => {
            let namespace = model["product"]["id"].as_str().unwrap_or_default();
            let mut arguments = vec![
                "exec".to_owned(),
                "--namespace".to_owned(),
                namespace.to_owned(),
                format!("deployment/{component}"),
                "--".to_owned(),
            ];
            arguments.extend(command);
            ("kubectl", arguments)
        }
        _ => {
            return Err(PrismError::new(
                "PP7401",
                "target cannot execute post-deployment acceptance",
            ))
        }
    };
    adapter_process_limited(root, tool, arguments, 8 * 1024 * 1024)
}

fn telemetry_log_process(
    root: &Path,
    kind: &str,
    target_name: &str,
    model: &Value,
    projection: &Path,
) -> Result<crate::verification::ProcessRecord, PrismError> {
    match kind {
        "compose" => adapter_process_limited(
            root,
            "docker",
            vec![
                "compose".to_owned(),
                "--project-name".to_owned(),
                target_name.to_owned(),
                "--file".to_owned(),
                projection.to_string_lossy().into_owned(),
                "logs".to_owned(),
                "--no-color".to_owned(),
                "telemetry".to_owned(),
            ],
            16 * 1024 * 1024,
        ),
        "kubernetes" => adapter_process_limited(
            root,
            "kubectl",
            vec![
                "logs".to_owned(),
                "--namespace".to_owned(),
                model["product"]["id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned(),
                "deployment/telemetry".to_owned(),
                "--tail=-1".to_owned(),
            ],
            16 * 1024 * 1024,
        ),
        _ => Err(PrismError::new(
            "PP7401",
            "target cannot collect telemetry evidence",
        )),
    }
}

fn first_u64(value: &str) -> Option<u64> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|part| !part.is_empty())?
        .parse()
        .ok()
}

fn slo_threshold(model: &Value, id: &str) -> Result<u64, PrismError> {
    model["slos"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|row| row["id"] == id)
        .and_then(|row| row["value"].as_str())
        .and_then(first_u64)
        .ok_or_else(|| PrismError::new("PP7401", format!("modeled SLO {id} is not measurable")))
}

fn scan_secret_values(root: &Path, text: &str) -> Result<(), PrismError> {
    let Some(directory) = std::env::var_os("PRISMPM_SECRET_DIR") else {
        return Ok(());
    };
    let directory = PathBuf::from(directory);
    scan_secret_directory(&directory, text)?;
    let _ = root;
    Ok(())
}

fn scan_secret_directory(directory: &Path, text: &str) -> Result<(), PrismError> {
    let metadata = std::fs::symlink_metadata(directory)
        .map_err(|error| PrismError::new("PP7801", format!("secret directory: {error}")))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(PrismError::new(
            "PP7801",
            "PRISMPM_SECRET_DIR does not identify a real directory",
        ));
    }
    let mut pending = vec![(directory.to_path_buf(), 0_usize)];
    let mut file_count = 0_usize;
    let mut total_bytes = 0_usize;
    while let Some((directory, depth)) = pending.pop() {
        if depth > 16 {
            return Err(PrismError::new(
                "PP7801",
                "secret directory nesting exceeds 16 levels",
            ));
        }
        let mut entries = std::fs::read_dir(&directory)
            .map_err(|error| PrismError::new("PP7801", format!("secret directory: {error}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| PrismError::new("PP7801", format!("secret entry: {error}")))?;
        entries.sort_by_key(std::fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            let metadata = std::fs::symlink_metadata(&path)
                .map_err(|error| PrismError::new("PP7801", format!("secret entry: {error}")))?;
            if metadata.file_type().is_symlink() {
                return Err(PrismError::new(
                    "PP7801",
                    "secret directory contains a symbolic link",
                ));
            }
            if metadata.is_dir() {
                pending.push((path, depth + 1));
                continue;
            }
            if !metadata.is_file() {
                return Err(PrismError::new(
                    "PP7801",
                    "secret directory contains a non-regular entry",
                ));
            }
            file_count += 1;
            if file_count > 4096 || metadata.len() > 64 * 1024 {
                return Err(PrismError::new(
                    "PP7801",
                    "secret directory or reference exceeds its bounded size",
                ));
            }
            let bytes = std::fs::read(&path)
                .map_err(|error| PrismError::new("PP7801", format!("secret reference: {error}")))?;
            total_bytes = total_bytes.checked_add(bytes.len()).ok_or_else(|| {
                PrismError::new("PP7801", "secret directory byte count overflowed")
            })?;
            if total_bytes > 16 * 1024 * 1024 {
                return Err(PrismError::new("PP7801", "secret directory exceeds 16 MiB"));
            }
            let mut start = 0;
            let mut end = bytes.len();
            while start < end && bytes[start].is_ascii_whitespace() {
                start += 1;
            }
            while end > start && bytes[end - 1].is_ascii_whitespace() {
                end -= 1;
            }
            let value = &bytes[start..end];
            if value.is_empty() {
                return Err(PrismError::new("PP7801", "secret reference is empty"));
            }
            let observed = text.as_bytes();
            let raw_found = observed.windows(value.len()).any(|window| window == value);
            let standard = base64::engine::general_purpose::STANDARD.encode(value);
            let url = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(value);
            if raw_found
                || (standard.len() >= 8 && text.contains(&standard))
                || (url.len() >= 8 && text.contains(&url))
            {
                return Err(PrismError::new(
                    "PP7801",
                    "observed acceptance or telemetry contains a secret value",
                ));
            }
        }
    }
    if text.contains("PRISMPM-REDACTION-CANARY") {
        return Err(PrismError::new(
            "PP7801",
            "OpenTelemetry redaction canary escaped into observed output",
        ));
    }
    Ok(())
}

fn post_deployment_checks(
    root: &Path,
    kind: &str,
    target_name: &str,
    target_profile: &str,
    model: &Value,
    projection: &Path,
) -> Result<Vec<Value>, PrismError> {
    let record = acceptance_process(root, kind, target_name, target_profile, model, projection)?;
    let text = record.stdout.trim();
    let mut result: Value = serde_json::from_str(text).map_err(|error| {
        PrismError::new(
            "PP7401",
            format!("post-deployment acceptance output: {error}"),
        )
    })?;
    if encode_value(&result)? != text.as_bytes()
        || result["schema"] != "prismpm/runtime-acceptance/1"
        || result["status"] != "passed"
    {
        return Err(PrismError::new(
            "PP7401",
            "post-deployment acceptance result is not canonical or passing",
        ));
    }
    crate::operations::redact(model, &mut result)?;
    let availability = result["availability_millionths"]
        .as_u64()
        .ok_or_else(|| PrismError::new("PP7401", "availability measurement is absent"))?;
    let outbox_lag = result["outbox_lag_p99_millis"]
        .as_u64()
        .ok_or_else(|| PrismError::new("PP7401", "outbox-lag measurement is absent"))?;
    if availability < slo_threshold(model, "slo-availability")?
        || outbox_lag > slo_threshold(model, "slo-outbox-lag")?
    {
        return Err(PrismError::new("PP7401", "a modeled SLO was not met"));
    }
    let receipt = &result["telemetry_receipt"];
    if !["logs", "metrics", "traces"]
        .iter()
        .all(|name| receipt[*name] == true)
    {
        return Err(PrismError::new(
            "PP7401",
            "the OpenTelemetry Collector did not receive every modeled signal",
        ));
    }
    let acceptance_bytes = encode_value(&result)?;
    scan_secret_values(root, text)?;

    let mut telemetry_text = String::new();
    for _ in 0..20 {
        let logs = telemetry_log_process(root, kind, target_name, model, projection)?;
        telemetry_text = format!("{}\n{}", logs.stdout, logs.stderr);
        let lower = telemetry_text.to_ascii_lowercase();
        if lower.contains("acceptance-signal")
            && lower.contains("logs")
            && lower.contains("metrics")
            && (lower.contains("spans") || lower.contains("traces"))
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    let lower = telemetry_text.to_ascii_lowercase();
    if !lower.contains("acceptance-signal")
        || !lower.contains("logs")
        || !lower.contains("metrics")
        || (!lower.contains("spans") && !lower.contains("traces"))
    {
        return Err(PrismError::new(
            "PP7401",
            "collector logs do not prove logs, metrics, and traces receipt",
        ));
    }
    scan_secret_values(root, &telemetry_text)?;
    let telemetry_digest = sha(telemetry_text.as_bytes());
    let acceptance_digest = sha(&acceptance_bytes);
    Ok(vec![
        json!({"evidence_digest":acceptance_digest,"id":"post-deployment-acceptance","kind":"contract","status":"passed"}),
        json!({"evidence_digest":telemetry_digest,"id":"opentelemetry-receipt-redaction","kind":"telemetry","status":"passed"}),
        json!({"evidence_digest":sha(&encode_value(&json!({"availability_millionths":availability,"outbox_lag_p99_millis":outbox_lag}))?),"id":"modeled-slo-measurements","kind":"slo","status":"measured"}),
    ])
}

fn resource_rows(kind: &str, projection: &Value) -> Result<BTreeMap<String, String>, PrismError> {
    let mut rows = BTreeMap::new();
    match kind {
        "compose" => {
            for (name, service) in projection["services"]
                .as_object()
                .ok_or_else(|| PrismError::new("PP7101", "Compose projection has no services"))?
            {
                let bytes = encode_value(service)?;
                rows.insert(format!("service/{name}"), sha(&bytes));
            }
        }
        "kubernetes" => {
            for resource in projection["items"]
                .as_array()
                .ok_or_else(|| PrismError::new("PP7101", "Kubernetes projection has no items"))?
            {
                let kind = resource["kind"].as_str().ok_or_else(|| {
                    PrismError::new("PP7101", "Kubernetes resource kind is absent")
                })?;
                let name = resource["metadata"]["name"].as_str().ok_or_else(|| {
                    PrismError::new("PP7101", "Kubernetes resource name is absent")
                })?;
                rows.insert(format!("{kind}/{name}"), sha(&encode_value(resource)?));
            }
        }
        _ => return Err(PrismError::new("PP7101", "unsupported target projection")),
    }
    Ok(rows)
}

fn state_resources(state: Option<&Value>) -> BTreeMap<String, String> {
    state
        .and_then(|value| value.get("resources"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(id, digest)| Some((id.clone(), digest.as_str()?.to_owned())))
        .collect()
}

/// Canonical result returned by plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanResult {
    /// Plan contract.
    pub schema: String,
    /// Digest of the core plan fields, excluding this field.
    pub plan_digest: String,
    /// Immutable release digest.
    pub release_digest: String,
    /// Target ID.
    pub target: String,
    /// Desired target-native document digest.
    pub desired_digest: String,
    /// Observed state digest at planning time.
    pub observed_digest: String,
    /// State digest against which this plan remains valid.
    pub expires_after_state: String,
    /// Required secret-reference IDs.
    pub secret_references: Vec<String>,
    /// Whether separate deployment authorization is required.
    pub authorization: String,
    /// Complete normalized change rows.
    pub changes: Vec<Value>,
}

fn create_plan_bound(
    root: &Path,
    reference: &str,
    target_name: &str,
    target_profile: &str,
) -> Result<(PlanResult, Value, Vec<u8>), PrismError> {
    let digest = root_digest(reference)?;
    target_id(target_name)?;
    target_id(target_profile)?;
    let model = system(root, digest)?;
    let target = target(&model, target_profile)?;
    let kind = target["kind"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7101", "target kind is absent"))?;
    let (_, projection_bytes) = projection(root, digest, kind)?;
    let projection_value: Value = serde_json::from_slice(&projection_bytes)
        .map_err(|error| PrismError::new("PP7101", format!("target projection: {error}")))?;
    let desired = resource_rows(kind, &projection_value)?;
    let projection_path = persist_projection(root, target_name, &projection_bytes)?;
    let observed_bytes = live_observation(root, target_name, kind, &model, &projection_path)?;
    let (_, prior_state) = recorded_state(root, target_name)?;
    let previous = state_resources(prior_state.as_ref());
    let mut changes = Vec::new();
    for (id, desired_digest) in &desired {
        let (action, risk) = match previous.get(id) {
            None => ("create", "low"),
            Some(current) if current == desired_digest => ("retain", "none"),
            Some(_) if id.starts_with("Job/") => ("replace", "high"),
            Some(_) => ("update", "medium"),
        };
        changes.push(json!({"action":action,"downtime":false,"id":id,"risk":risk,"rollback":"restore the previously accepted release digest"}));
    }
    for id in previous.keys().filter(|id| !desired.contains_key(*id)) {
        changes.push(json!({"action":"delete","downtime":true,"id":id,"risk":"destructive","rollback":"reapply the previously accepted release digest"}));
    }
    for persistence in model["persistence"].as_array().into_iter().flatten() {
        for migration in persistence["migration_order"]
            .as_array()
            .into_iter()
            .flatten()
        {
            changes.push(json!({"action":"migrate","downtime":false,"id":format!("migration/{}", migration.as_str().unwrap_or_default()),"risk":"high","rollback":model["lifecycle"]["rollback"]}));
        }
    }
    changes.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    let mut secrets = model["secret_references"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row["id"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    secrets.sort();
    let observed_digest = sha(&observed_bytes);
    let authorization = if changes.iter().any(|row| row["risk"] == "destructive") {
        "required"
    } else {
        "not-required"
    };
    let core = json!({
        "authorization":authorization,"changes":changes,"desired_digest":sha(&projection_bytes),
        "expires_after_state":observed_digest,"observed_digest":observed_digest,
        "release_digest":digest,"schema":"prismpm/deployment-plan/1",
        "secret_references":secrets,"target":target_name
    });
    let plan_digest = sha(&encode_value(&core)?);
    let mut complete = core;
    complete
        .as_object_mut()
        .expect("plan object")
        .insert("plan_digest".to_owned(), Value::String(plan_digest.clone()));
    let document = CanonicalDocument::from_value("prismpm/deployment-plan/1", complete)?;
    let plan: PlanResult = serde_json::from_value(document.value().clone())
        .map_err(|error| PrismError::new("PP7201", error.to_string()))?;
    Ok((plan, projection_value, projection_bytes))
}

fn replace_immutable_jobs(
    root: &Path,
    kind: &str,
    model: &Value,
    plan: &PlanResult,
) -> Result<(), PrismError> {
    if kind != "kubernetes" {
        return Ok(());
    }
    let jobs = plan
        .changes
        .iter()
        .filter(|row| row["action"] == "replace")
        .filter_map(|row| row["id"].as_str()?.strip_prefix("Job/"))
        .collect::<Vec<_>>();
    if jobs.is_empty() {
        return Ok(());
    }
    let namespace = model["product"]["id"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7301", "Kubernetes namespace is absent"))?;
    let mut arguments = vec!["delete".to_owned(), "job".to_owned()];
    arguments.extend(jobs.into_iter().map(str::to_owned));
    arguments.extend([
        "--namespace".to_owned(),
        namespace.to_owned(),
        "--ignore-not-found=true".to_owned(),
        "--wait=true".to_owned(),
    ]);
    adapter_process(root, "kubectl", arguments)?;
    Ok(())
}

/// Create and atomically record a state-bound deterministic deployment plan.
pub fn plan(root: &Path, reference: &str, target: &str) -> Result<PlanResult, PrismError> {
    plan_bound(root, reference, target, target)
}

fn plan_bound(
    root: &Path,
    reference: &str,
    target_name: &str,
    target_profile: &str,
) -> Result<PlanResult, PrismError> {
    let (plan, _, _) = create_plan_bound(root, reference, target_name, target_profile)?;
    let bytes = encode_value(
        &serde_json::to_value(&plan)
            .map_err(|error| PrismError::new("PP9001", error.to_string()))?,
    )?;
    let path = root.join(".prism/plans").join(format!(
        "{}.json",
        plan.plan_digest.trim_start_matches("sha256:")
    ));
    std::fs::create_dir_all(path.parent().expect("plan parent"))
        .map_err(|error| PrismError::new("PP7201", error.to_string()))?;
    atomic(&path, &bytes)?;
    Ok(plan)
}

fn compose_arguments(target: &str, path: &Path, mode: &str) -> Result<Vec<String>, PrismError> {
    let mut arguments = vec![
        "compose".to_owned(),
        "--project-name".to_owned(),
        target.to_owned(),
        "--file".to_owned(),
        path.to_string_lossy().into_owned(),
    ];
    match mode {
        "deploy" => arguments.extend(
            ["up", "--detach", "--wait", "--no-build"]
                .into_iter()
                .map(str::to_owned),
        ),
        "destroy" => arguments.extend(["down", "--remove-orphans"].into_iter().map(str::to_owned)),
        "logs" => arguments.extend(
            ["logs", "--follow", "--no-color", "--timestamps=false"]
                .into_iter()
                .map(str::to_owned),
        ),
        _ => return Err(PrismError::new("PP7301", "unsupported Compose operation")),
    }
    Ok(arguments)
}

fn kubernetes_is_ingress_infrastructure(item: &Value) -> bool {
    item.pointer("/metadata/labels/app.kubernetes.io~1name")
        .and_then(Value::as_str)
        == Some("ingress-nginx")
}

/// Partition a Kubernetes projection into official ingress infrastructure and application resources.
pub fn kubernetes_partition(projection: &Value) -> Result<(Value, Value), PrismError> {
    if projection["apiVersion"].as_str() != Some("v1")
        || projection["kind"].as_str() != Some("List")
    {
        return Err(PrismError::new(
            "PP7301",
            "Kubernetes projection is not a v1 List",
        ));
    }
    let items = projection["items"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP7301", "Kubernetes projection items are absent"))?;
    if items.is_empty() {
        return Err(PrismError::new(
            "PP7301",
            "Kubernetes projection contains no resources",
        ));
    }
    let (infrastructure, application): (Vec<_>, Vec<_>) = items
        .iter()
        .cloned()
        .partition(kubernetes_is_ingress_infrastructure);
    Ok((
        json!({"apiVersion":"v1","items":infrastructure,"kind":"List"}),
        json!({"apiVersion":"v1","items":application,"kind":"List"}),
    ))
}

fn kubernetes_projection_path(path: &Path, stage: &str) -> Result<PathBuf, PrismError> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| PrismError::new("PP7301", "Kubernetes projection path is malformed"))?;
    Ok(path.with_file_name(format!("{name}.{stage}.json")))
}

fn kubectl_apply(root: &Path, path: &Path) -> Result<(), PrismError> {
    adapter_process(
        root,
        "kubectl",
        vec![
            "apply".to_owned(),
            "--server-side=true".to_owned(),
            "--field-manager=prismpm".to_owned(),
            "--filename".to_owned(),
            path.to_string_lossy().into_owned(),
        ],
    )?;
    Ok(())
}

fn kubectl_wait_for_ingress(root: &Path) -> Result<(), PrismError> {
    for job in [
        "job/ingress-nginx-admission-create",
        "job/ingress-nginx-admission-patch",
    ] {
        adapter_process(
            root,
            "kubectl",
            vec![
                "wait".to_owned(),
                "--namespace".to_owned(),
                "ingress-nginx".to_owned(),
                "--for=condition=complete".to_owned(),
                job.to_owned(),
                "--timeout=180s".to_owned(),
            ],
        )?;
    }
    adapter_process(
        root,
        "kubectl",
        vec![
            "rollout".to_owned(),
            "status".to_owned(),
            "--namespace".to_owned(),
            "ingress-nginx".to_owned(),
            "deployment/ingress-nginx-controller".to_owned(),
            "--timeout=180s".to_owned(),
        ],
    )?;
    Ok(())
}

fn kubernetes_deploy(root: &Path, path: &Path) -> Result<(), PrismError> {
    let bytes = std::fs::read(path)
        .map_err(|error| PrismError::new("PP7301", format!("Kubernetes projection: {error}")))?;
    let projection: Value = serde_json::from_slice(&bytes)
        .map_err(|error| PrismError::new("PP7301", format!("Kubernetes projection: {error}")))?;
    let (infrastructure, application) = kubernetes_partition(&projection)?;
    let infrastructure_items = infrastructure["items"]
        .as_array()
        .expect("partitioned Kubernetes items");
    let application_items = application["items"]
        .as_array()
        .expect("partitioned Kubernetes items");
    if !infrastructure_items.is_empty() {
        let infrastructure_path = kubernetes_projection_path(path, "infrastructure")?;
        atomic(&infrastructure_path, &encode_value(&infrastructure)?)?;
        kubectl_apply(root, &infrastructure_path)?;
        kubectl_wait_for_ingress(root)?;
    }
    if application_items.is_empty() {
        return Err(PrismError::new(
            "PP7301",
            "Kubernetes projection contains no product resources",
        ));
    }
    let application_path = kubernetes_projection_path(path, "application")?;
    atomic(&application_path, &encode_value(&application)?)?;
    kubectl_apply(root, &application_path)
}

fn invoke(
    root: &Path,
    kind: &str,
    target: &str,
    path: &Path,
    mode: &str,
) -> Result<(), PrismError> {
    if (kind, mode) == ("kubernetes", "deploy") {
        return kubernetes_deploy(root, path);
    }
    let env = adapter_environment()?;
    let (tool, program, arguments) = match (kind, mode) {
        ("compose", "deploy") => (
            "docker",
            crate::sdk::executable("docker")?,
            compose_arguments(target, path, "deploy")?,
        ),
        ("compose", "destroy") => (
            "docker",
            crate::sdk::executable("docker")?,
            compose_arguments(target, path, "destroy")?,
        ),
        ("kubernetes", "destroy") => (
            "kubectl",
            crate::sdk::executable("kubectl")?,
            [
                "delete",
                "--filename",
                path.to_str().unwrap_or_default(),
                "--wait=true",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        ),
        _ => return Err(PrismError::new("PP7301", "unsupported adapter operation")),
    };
    crate::verification::run_process(
        tool,
        &program,
        &arguments,
        root,
        &env,
        &[(root, "$PROJECT")],
        "PP7301",
    )?;
    Ok(())
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<(), PrismError> {
    let parent = path
        .parent()
        .ok_or_else(|| PrismError::new("PP9001", "state has no parent"))?;
    let mut temp = tempfile::Builder::new()
        .prefix("state-")
        .tempfile_in(parent)
        .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    temp.write_all(bytes)
        .and_then(|()| temp.as_file().sync_all())
        .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    temp.persist(path)
        .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    Ok(())
}

fn materialize_release(root: &Path, digest: &str, target: &str) -> Result<PathBuf, PrismError> {
    let parent = root.join(".prism/targets").join(target);
    std::fs::create_dir_all(&parent)
        .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    let staging = tempfile::Builder::new()
        .prefix("release-")
        .tempdir_in(&parent)
        .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    let mut count = 0_u64;
    for descriptor in oci::layers(root, digest)? {
        let title = descriptor
            .annotations
            .as_ref()
            .and_then(|values| values.get("org.opencontainers.image.title"))
            .map(String::as_str)
            .unwrap_or_default();
        let destination = if title.starts_with("core-wasm/") && title.ends_with(".wasm") {
            Some("core.wasm".to_owned())
        } else if let Some(path) = title.strip_prefix("view/browser/") {
            Some(format!("browser/{path}"))
        } else if let Some(path) = title.strip_prefix("production-browser/") {
            Some(format!("production-browser/{path}"))
        } else if let Some(path) = title.strip_prefix("projections/") {
            matches!(
                path,
                "contract.sql"
                    | "history.sql"
                    | "runtime-contract.json"
                    | "opentelemetry-collector.json"
            )
            .then(|| path.to_owned())
        } else {
            None
        };
        let Some(destination) = destination else {
            continue;
        };
        if destination.contains("..") || destination.starts_with('/') {
            return Err(PrismError::new(
                "PP7301",
                "release materialization path escaped",
            ));
        }
        let path = staging.path().join(&destination);
        std::fs::create_dir_all(path.parent().expect("materialized file parent"))
            .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
        std::fs::write(&path, oci::read_descriptor(root, &descriptor)?)
            .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
        count += 1;
    }
    if count < 4 {
        return Err(PrismError::new(
            "PP7301",
            "release lacks executable runtime artifacts",
        ));
    }
    let destination = parent.join("release");
    if destination.exists() {
        std::fs::remove_dir_all(&destination)
            .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    }
    std::fs::rename(staging.path(), &destination)
        .map_err(|error| PrismError::new("PP7301", error.to_string()))?;
    Ok(destination)
}

fn parameter_default<'a>(model: &'a Value, id: &str) -> Result<&'a str, PrismError> {
    model["parameters"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|row| row["id"] == id)
        .and_then(|row| row["default"].as_str())
        .ok_or_else(|| PrismError::new("PP7901", format!("parameter {id} has no default")))
}

fn normalized_dump(value: &str) -> Vec<u8> {
    let mut out = value
        .replace("\r\n", "\n")
        .lines()
        .filter(|line| {
            !line.starts_with("--")
                && !line.starts_with("\\restrict ")
                && !line.starts_with("\\unrestrict ")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .into_bytes();
    out.push(b'\n');
    out
}

fn database_pod(root: &Path, namespace: &str) -> Result<String, PrismError> {
    let record = adapter_process(
        root,
        "kubectl",
        vec![
            "get".to_owned(),
            "pods".to_owned(),
            "--namespace".to_owned(),
            namespace.to_owned(),
            "--selector".to_owned(),
            "app.kubernetes.io/name=database".to_owned(),
            "--output".to_owned(),
            "jsonpath={.items[0].metadata.name}".to_owned(),
        ],
    )?;
    let pod = record.stdout.trim();
    if pod.is_empty()
        || !pod.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
    {
        return Err(PrismError::new(
            "PP7901",
            "database pod could not be identified",
        ));
    }
    Ok(pod.to_owned())
}

fn logical_dump(
    root: &Path,
    kind: &str,
    target_name: &str,
    model: &Value,
    projection_path: &Path,
) -> Result<Vec<u8>, PrismError> {
    let user = parameter_default(model, "postgres-user")?;
    let database = parameter_default(model, "postgres-db")?;
    let arguments = match kind {
        "compose" => vec![
            "compose".to_owned(),
            "--project-name".to_owned(),
            target_name.to_owned(),
            "--file".to_owned(),
            projection_path.to_string_lossy().into_owned(),
            "exec".to_owned(),
            "-T".to_owned(),
            "database".to_owned(),
            "pg_dump".to_owned(),
            "--username".to_owned(),
            user.to_owned(),
            "--dbname".to_owned(),
            database.to_owned(),
            "--format=plain".to_owned(),
            "--no-owner".to_owned(),
            "--no-privileges".to_owned(),
        ],
        "kubernetes" => {
            let namespace = model["product"]["id"].as_str().unwrap_or_default();
            let pod = database_pod(root, namespace)?;
            vec![
                "exec".to_owned(),
                "--namespace".to_owned(),
                namespace.to_owned(),
                pod,
                "--".to_owned(),
                "pg_dump".to_owned(),
                "--username".to_owned(),
                user.to_owned(),
                "--dbname".to_owned(),
                database.to_owned(),
                "--format=plain".to_owned(),
                "--no-owner".to_owned(),
                "--no-privileges".to_owned(),
            ]
        }
        _ => return Err(PrismError::new("PP7901", "backup target is unsupported")),
    };
    let tool = if kind == "compose" {
        "docker"
    } else {
        "kubectl"
    };
    let record = adapter_process_limited(root, tool, arguments, 536_870_912)?;
    Ok(normalized_dump(&record.stdout))
}

fn recovery_policy(model: &Value) -> Result<(), PrismError> {
    let backup = model["lifecycle"]["backup"].as_str().unwrap_or_default();
    let recovery = model["lifecycle"]["recovery"].as_str().unwrap_or_default();
    if backup.is_empty()
        || recovery.is_empty()
        || model["backups"].as_array().is_none_or(Vec::is_empty)
    {
        return Err(PrismError::new(
            "PP7901",
            "release does not model backup and recovery",
        ));
    }
    Ok(())
}

struct DeploymentEvidenceInput<'a> {
    target_profile: &'a str,
    observed_state: &'a str,
    operation: &'a str,
    status: &'a str,
    model: &'a Value,
    release_trust: Option<&'a Value>,
    extra_checks: &'a [Value],
}

fn deployment_evidence(
    root: &Path,
    plan: &PlanResult,
    input: DeploymentEvidenceInput<'_>,
) -> Result<Vec<u8>, PrismError> {
    let release = oci::release_metadata(&oci::Store::open(root)?, &plan.release_digest)?;
    let standards_lock = release["standards_lock"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7401", "release standards-lock identity is absent"))?;
    let modeled_policy = sha(&encode_value(&json!({
        "lifecycle":input.model["lifecycle"],
        "target":target(input.model, input.target_profile)?
    }))?);
    let trust_policy = input
        .release_trust
        .and_then(|trust| trust["policy_digest"].as_str())
        .unwrap_or(&modeled_policy);
    let signature = if let Some(trust) = input.release_trust {
        json!({"evidence_digest":sha(&encode_value(trust)?),"id":"release-signature","kind":"signature","status":"passed"})
    } else {
        json!({"evidence_digest":plan.release_digest,"id":"release-signature","kind":"signature","status":"not-executable"})
    };
    let mut checks = vec![
        json!({"evidence_digest":standards_lock,"id":"authority-lock","kind":"authority","status":"passed"}),
        json!({"evidence_digest":trust_policy,"id":"release-policy","kind":"policy","status":"passed"}),
        signature,
        json!({"evidence_digest":input.observed_state,"id":"target-readiness","kind":"readiness","status":"passed"}),
        json!({"evidence_digest":plan.desired_digest,"id":"desired-drift","kind":"drift","status":"passed"}),
    ];
    checks.extend(crate::operations::model_evidence(
        input.model,
        &plan.release_digest,
    )?);
    checks.extend_from_slice(input.extra_checks);
    checks.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    let core = json!({
        "checks":checks,
        "observed_state":input.observed_state,"operation":input.operation,"plan_digest":plan.plan_digest,
        "release_digest":plan.release_digest,"schema":"prismpm/deployment-evidence/1",
        "status":input.status,"target":plan.target
    });
    let digest = sha(&encode_value(&core)?);
    let mut complete = core;
    complete
        .as_object_mut()
        .expect("evidence object")
        .insert("evidence_digest".to_owned(), Value::String(digest));
    CanonicalDocument::from_value("prismpm/deployment-evidence/1", complete)
        .map(|doc| doc.bytes().to_vec())
}

fn update_history(root: &Path, target: &str, release: &str) -> Result<(), PrismError> {
    let path = history_path(root, target);
    let mut rows = if path.exists() {
        let bytes =
            std::fs::read(&path).map_err(|error| PrismError::new("PP7601", error.to_string()))?;
        serde_json::from_slice::<Vec<String>>(&bytes)
            .map_err(|error| PrismError::new("PP7601", error.to_string()))?
    } else {
        Vec::new()
    };
    if rows.last().map(String::as_str) != Some(release) {
        rows.push(release.to_owned());
    }
    atomic(
        &path,
        &encode_value(&serde_json::to_value(rows).expect("history"))?,
    )
}

fn read_plan(root: &Path, digest: &str) -> Result<PlanResult, PrismError> {
    let hex = digest
        .strip_prefix("sha256:")
        .filter(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
        .ok_or_else(|| PrismError::new("PP7201", "plan digest is malformed"))?;
    let bytes = std::fs::read(root.join(".prism/plans").join(format!("{hex}.json")))
        .map_err(|_| PrismError::new("PP7201", "deployment plan is absent"))?;
    let document = CanonicalDocument::parse("prismpm/deployment-plan/1", &bytes)?;
    let plan: PlanResult = serde_json::from_value(document.value().clone())
        .map_err(|error| PrismError::new("PP7201", error.to_string()))?;
    if plan.plan_digest != digest {
        return Err(PrismError::new(
            "PP7201",
            "deployment plan identity changed",
        ));
    }
    Ok(plan)
}

fn enforce_data_compatibility(
    root: &Path,
    target_name: &str,
    requested_release: &str,
) -> Result<Option<Value>, PrismError> {
    let (_, state) = recorded_state(root, target_name)?;
    let Some(state) = state else {
        return Ok(None);
    };
    if state["migration_phase"] != "contract-finalized" {
        return Ok(Some(state));
    }
    let floor = state["migration_release_digest"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7601", "finalized migration has no release floor"))?;
    if requested_release == floor {
        return Ok(Some(state));
    }
    let history = accepted_history(root, target_name)?;
    let floor_index = history
        .iter()
        .position(|digest| digest == floor)
        .ok_or_else(|| {
            PrismError::new(
                "PP7601",
                "finalized migration floor is absent from accepted release history",
            )
        })?;
    if history
        .iter()
        .position(|digest| digest == requested_release)
        .is_some_and(|index| index < floor_index)
    {
        return Err(PrismError::new(
            "PP7601",
            "data contract was finalized and the requested release is an incompatible downgrade",
        ));
    }
    Ok(Some(state))
}

struct ApplyRequest<'a> {
    target_profile: &'a str,
    requested_plan: Option<&'a str>,
    operation: &'a str,
    accepted_status: &'a str,
    automatic_rollback: bool,
}

fn require_deployable_release(root: &Path, release_digest: &str) -> Result<Value, PrismError> {
    let trust = crate::supply_chain::verify_release_trust(root, release_digest)?;
    if !matches!(trust["status"].as_str(), Some("candidate" | "accepted")) {
        return Err(PrismError::new(
            "PP7401",
            "deployment requires a cryptographically verified candidate or accepted release",
        ));
    }
    Ok(trust)
}

fn release_trust_for_target(
    root: &Path,
    release_digest: &str,
    model: &Value,
    target_profile: &str,
) -> Result<Option<Value>, PrismError> {
    let minimum = target(model, target_profile)?["minimum_release_status"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7401", "target minimum release status is absent"))?;
    match minimum {
        "development" => {
            crate::supply_chain::verify_release_for_development(root, release_digest)?;
            let status = oci::verified_promotion_status(root, release_digest)?;
            if status == "development" {
                Ok(None)
            } else {
                require_deployable_release(root, release_digest).map(Some)
            }
        }
        "candidate" => require_deployable_release(root, release_digest).map(Some),
        "accepted" => {
            let trust = require_deployable_release(root, release_digest)?;
            if trust["status"] != "accepted" {
                return Err(PrismError::new(
                    "PP7401",
                    "target requires a cryptographically verified accepted release",
                ));
            }
            Ok(Some(trust))
        }
        _ => Err(PrismError::new(
            "PP7401",
            "target minimum release status is unsupported",
        )),
    }
}

/// Apply a current state-bound plan using the target's unmodified standard tool.
fn apply_plan_inner(
    root: &Path,
    reference: &str,
    target_name: &str,
    request: ApplyRequest<'_>,
) -> Result<Value, PrismError> {
    let requested_release = root_digest(reference)?;
    let model = system(root, requested_release)?;
    let release_trust =
        release_trust_for_target(root, requested_release, &model, request.target_profile)?;
    let selected = if let Some(digest) = request.requested_plan {
        read_plan(root, digest)?
    } else {
        plan(root, reference, target_name)?
    };
    if selected.release_digest != requested_release || selected.target != target_name {
        return Err(PrismError::new(
            "PP7201",
            "deployment plan subject or target does not match the request",
        ));
    }
    let prior_state = enforce_data_compatibility(root, target_name, &selected.release_digest)?;
    let (plan, projection, projection_bytes) =
        create_plan_bound(root, reference, target_name, request.target_profile)?;
    if plan.plan_digest != selected.plan_digest {
        return Err(PrismError::new(
            "PP7201",
            "target state or desired release changed after planning",
        ));
    }
    if plan.authorization == "required" {
        return Err(PrismError::new(
            "PP7701",
            "destructive plan requires explicit destroy authorization",
        ));
    }
    let kind = target(&model, request.target_profile)?["kind"]
        .as_str()
        .unwrap_or_default();
    let projection_path = persist_projection(root, target_name, &projection_bytes)?;
    let current_bytes = live_observation(root, target_name, kind, &model, &projection_path)?;
    if sha(&current_bytes) != plan.expires_after_state {
        return Err(PrismError::new(
            "PP7201",
            "target state changed after planning",
        ));
    }
    // Every modeled state transition snapshots the exact ready release that
    // is about to be changed. The target's minimum release status determines
    // whether that evidence is unsigned development or production-trusted;
    // the recovery safety property itself is never skipped.
    let predeployment_backup = prior_state
        .as_ref()
        .and_then(|state| state["release_digest"].as_str())
        .filter(|prior| *prior != plan.release_digest)
        .map(|prior| {
            let previous = reference_with_digest(reference, prior)?;
            backup_inner(root, &previous, target_name)
        })
        .transpose()?;
    let deploy_result = (|| {
        materialize_release(root, &plan.release_digest, target_name)?;
        replace_immutable_jobs(root, kind, &model, &plan)?;
        invoke(root, kind, target_name, &projection_path, "deploy")?;
        wait_ready(root, kind, &model)?;
        let observed = live_observation(root, target_name, kind, &model, &projection_path)?;
        if !observation_ready(&observed)? {
            return Err(PrismError::new(
                "PP7401",
                "target did not satisfy modeled readiness",
            ));
        }
        let checks = post_deployment_checks(
            root,
            kind,
            target_name,
            request.target_profile,
            &model,
            &projection_path,
        )?;
        Ok((observed, checks))
    })();
    let (observed_bytes, mut observed_checks) = match deploy_result {
        Ok(observed) => observed,
        Err(failure) if request.automatic_rollback && request.operation == "deploy" => {
            return Err(automatic_rollback_after_failure(
                root,
                target_name,
                request.target_profile,
                &plan,
                &model,
                failure,
            ));
        }
        Err(failure) => return Err(failure),
    };
    if let Some(backup) = predeployment_backup {
        observed_checks.push(json!({
            "evidence_digest":backup["snapshot_digest"],
            "id":"pre-deployment-logical-backup",
            "kind":"backup",
            "status":"passed"
        }));
    }
    let resources = resource_rows(kind, &projection)?;
    let (migration_phase, migration_release_digest) = prior_state
        .as_ref()
        .filter(|state| state["migration_phase"] == "contract-finalized")
        .map_or(("expand-compatible", Value::Null), |state| {
            (
                "contract-finalized",
                state["migration_release_digest"].clone(),
            )
        });
    let state_core = json!({
        "desired_digest":plan.desired_digest,"drift":"none","last_applied_digest":plan.desired_digest,
        "migration_phase":migration_phase,"migration_release_digest":migration_release_digest,
        "ready":true,"release_digest":plan.release_digest,"resources":resources,
        "schema":"prismpm/deployment-state/1","target":target_name,
        "target_profile":request.target_profile
    });
    let observed_digest = sha(&observed_bytes);
    let mut state = state_core;
    state.as_object_mut().expect("state object").insert(
        "observed_digest".to_owned(),
        Value::String(observed_digest.clone()),
    );
    let state = CanonicalDocument::from_value("prismpm/deployment-state/1", state)?;
    let path = state_path(root, target_name);
    atomic(&path, state.bytes())?;
    update_history(root, target_name, &plan.release_digest)?;
    let evidence = deployment_evidence(
        root,
        &plan,
        DeploymentEvidenceInput {
            target_profile: request.target_profile,
            observed_state: &observed_digest,
            operation: request.operation,
            status: request.accepted_status,
            model: &model,
            release_trust: release_trust.as_ref(),
            extra_checks: &observed_checks,
        },
    )?;
    oci::attach_referrer(root, &plan.release_digest, DEPLOYMENT_EVIDENCE, &evidence)?;
    serde_json::from_slice(&evidence).map_err(|error| PrismError::new("PP9001", error.to_string()))
}

fn accepted_history(root: &Path, target_name: &str) -> Result<Vec<String>, PrismError> {
    let path = history_path(root, target_name);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(
        &std::fs::read(path)
            .map_err(|error| PrismError::new("PP7601", format!("rollback history: {error}")))?,
    )
    .map_err(|error| PrismError::new("PP7601", format!("rollback history: {error}")))
}

fn automatic_rollback_after_failure(
    root: &Path,
    target_name: &str,
    target_profile: &str,
    failed_plan: &PlanResult,
    failed_model: &Value,
    failure: PrismError,
) -> PrismError {
    let history = match accepted_history(root, target_name) {
        Ok(history) => history,
        Err(history_failure) => {
            return failure
                .with_note("the failed rollout was stopped, but accepted release history could not be read")
                .with_note(format!(
                    "automatic rollback failure {}: {}",
                    history_failure.code.as_str(),
                    history_failure.message
                ));
        }
    };
    let Some(previous) = history
        .last()
        .filter(|digest| digest.as_str() != failed_plan.release_digest)
    else {
        return failure.with_note(
            "the failed rollout was stopped; no preceding accepted release exists for automatic rollback",
        );
    };
    let rollback = apply_plan_inner(
        root,
        previous,
        target_name,
        ApplyRequest {
            target_profile,
            requested_plan: None,
            operation: "automatic-rollback",
            accepted_status: "rolled-back",
            automatic_rollback: false,
        },
    );
    let (rollback_status, rollback_evidence_digest) = match rollback {
        Ok(evidence) => (
            "succeeded",
            sha(&encode_value(&evidence).unwrap_or_else(|_| previous.as_bytes().to_vec())),
        ),
        Err(rollback_failure) => {
            return failure
                .with_note("automatic rollback was attempted and failed")
                .with_note(format!(
                    "rollback target {} failed with {}: {}",
                    previous,
                    rollback_failure.code.as_str(),
                    rollback_failure.message
                ));
        }
    };
    let failed_observation = live_observation(
        root,
        target_name,
        target(failed_model, target_profile)
            .ok()
            .and_then(|row| row["kind"].as_str())
            .unwrap_or_default(),
        failed_model,
        &projection_path(root, target_name),
    )
    .unwrap_or_else(|_| b"unavailable".to_vec());
    let record = encode_value(&json!({
        "failure":{"code":failure.code.as_str(),"message":failure.message},
        "failed_observation_digest":sha(&failed_observation),
        "failed_plan_digest":failed_plan.plan_digest,
        "failed_release_digest":failed_plan.release_digest,
        "rollback_evidence_digest":rollback_evidence_digest,
        "rollback_release_digest":previous,
        "rollback_status":rollback_status,
        "schema":"prismpm/failed-rollout/1",
        "target":target_name
    }))
    .unwrap_or_else(|_| b"failed-rollout-evidence-unavailable".to_vec());
    let attachment = oci::attach_referrer(
        root,
        &failed_plan.release_digest,
        "application/vnd.prismpm.failed-rollout.v1+json",
        &record,
    );
    let mut returned = PrismError::new(
        "PP7401",
        "rollout failed and the preceding accepted release was restored automatically",
    )
    .with_note(format!(
        "original failure {}: {}",
        failure.code.as_str(),
        failure.message
    ))
    .with_note(format!("restored release digest: {previous}"));
    match attachment {
        Ok(descriptor) => {
            returned = returned.with_note(format!("failed-rollout evidence: {descriptor}"));
        }
        Err(error) => {
            returned = returned.with_note(format!(
                "failed-rollout evidence attachment failed with {}: {}",
                error.code.as_str(),
                error.message
            ));
        }
    }
    returned
}

/// Apply a current state-bound plan using the target's unmodified standard tool.
pub fn deploy_planned(
    root: &Path,
    reference: &str,
    target_name: &str,
    requested_plan: Option<&str>,
) -> Result<Value, PrismError> {
    let _guard = operation_guard(root, target_name)?;
    apply_plan_inner(
        root,
        reference,
        target_name,
        ApplyRequest {
            target_profile: target_name,
            requested_plan,
            operation: "deploy",
            accepted_status: "accepted",
            automatic_rollback: true,
        },
    )
}

/// Plan and deploy directly while retaining the exact state-bound plan evidence.
pub fn deploy(root: &Path, reference: &str, target_name: &str) -> Result<Value, PrismError> {
    deploy_planned(root, reference, target_name, None)
}

/// Report desired, last-applied, and observed state identities separately.
pub fn status(root: &Path, reference: &str, target_name: &str) -> Result<Value, PrismError> {
    status_bound(root, reference, target_name, target_name)
}

fn status_bound(
    root: &Path,
    reference: &str,
    target_name: &str,
    target_profile: &str,
) -> Result<Value, PrismError> {
    let digest = root_digest(reference)?;
    target_id(target_name)?;
    let (_, state) = recorded_state(root, target_name)?;
    let mut state =
        state.ok_or_else(|| PrismError::new("PP7501", "target has no Prism deployment state"))?;
    let model = system(root, digest)?;
    let target_model = target(&model, target_profile)?;
    let kind = target_model["kind"].as_str().unwrap_or_default();
    let (_, projection_bytes) = projection(root, digest, kind)?;
    let projection = persist_projection(root, target_name, &projection_bytes)?;
    let live = live_observation(root, target_name, kind, &model, &projection)?;
    let live_digest = sha(&live);
    if state["release_digest"] != digest || state["observed_digest"] != live_digest {
        state
            .as_object_mut()
            .expect("state object")
            .insert("drift".to_owned(), Value::String("managed".to_owned()));
        state
            .as_object_mut()
            .expect("state object")
            .insert("ready".to_owned(), Value::Bool(false));
    }
    state
        .as_object_mut()
        .expect("state object")
        .insert("observed_digest".to_owned(), Value::String(live_digest));
    Ok(state)
}

/// Reapply the preceding accepted digest without rebuilding.
pub fn rollback(root: &Path, reference: &str, target_name: &str) -> Result<Value, PrismError> {
    let _guard = operation_guard(root, target_name)?;
    let requested = root_digest(reference)?;
    let history = accepted_history(root, target_name)?;
    if history.len() < 2 || history[history.len() - 2] != requested {
        return Err(PrismError::new(
            "PP7601",
            "requested digest is not the preceding accepted release",
        ));
    }
    apply_plan_inner(
        root,
        reference,
        target_name,
        ApplyRequest {
            target_profile: target_name,
            requested_plan: None,
            operation: "rollback",
            accepted_status: "rolled-back",
            automatic_rollback: false,
        },
    )
}

fn execute_contract_migration(
    root: &Path,
    digest: &str,
    target_name: &str,
    kind: &str,
    model: &Value,
    projection_path: &Path,
) -> Result<(), PrismError> {
    match kind {
        "compose" => {
            let arguments = vec![
                "compose".to_owned(),
                "--project-name".to_owned(),
                target_name.to_owned(),
                "--file".to_owned(),
                projection_path.to_string_lossy().into_owned(),
                "run".to_owned(),
                "--rm".to_owned(),
                "migrate".to_owned(),
                "node".to_owned(),
                "/opt/prismpm/runtime/runtime.mjs".to_owned(),
                "contract".to_owned(),
            ];
            adapter_process(root, "docker", arguments)?;
        }
        "kubernetes" => {
            let bytes = std::fs::read(projection_path)
                .map_err(|error| PrismError::new("PP7601", format!("projection: {error}")))?;
            let projection: Value = serde_json::from_slice(&bytes)
                .map_err(|error| PrismError::new("PP7601", format!("projection: {error}")))?;
            let mut job = projection["items"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|row| row["kind"] == "Job")
                .cloned()
                .ok_or_else(|| PrismError::new("PP7601", "modeled migration Job is absent"))?;
            let base_name = job["metadata"]["name"]
                .as_str()
                .ok_or_else(|| PrismError::new("PP7601", "migration Job name is absent"))?;
            let name = format!(
                "{}-contract-{}",
                base_name,
                &digest.trim_start_matches("sha256:")[..12]
            );
            job["metadata"]["name"] = Value::String(name.clone());
            let command = job
                .pointer_mut("/spec/template/spec/containers/0/command")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| PrismError::new("PP7601", "migration Job command is absent"))?;
            let role = command
                .last_mut()
                .ok_or_else(|| PrismError::new("PP7601", "migration Job command is empty"))?;
            if role.as_str() != Some("migrate") {
                return Err(PrismError::new(
                    "PP7601",
                    "migration Job is not the modeled generic migration runner",
                ));
            }
            *role = Value::String("contract".to_owned());
            let document = json!({"apiVersion":"v1","items":[job],"kind":"List"});
            let path = projection_path.with_file_name("contract-migration.json");
            atomic(&path, &encode_value(&document)?)?;
            let namespace = model["product"]["id"]
                .as_str()
                .ok_or_else(|| PrismError::new("PP7601", "Kubernetes namespace is absent"))?;
            let apply = adapter_process(
                root,
                "kubectl",
                vec![
                    "apply".to_owned(),
                    "--server-side=true".to_owned(),
                    "--field-manager=prismpm".to_owned(),
                    "--filename".to_owned(),
                    path.to_string_lossy().into_owned(),
                ],
            )
            .and_then(|_| {
                adapter_process(
                    root,
                    "kubectl",
                    vec![
                        "wait".to_owned(),
                        "--for=condition=complete".to_owned(),
                        format!("job/{name}"),
                        "--namespace".to_owned(),
                        namespace.to_owned(),
                        "--timeout=300s".to_owned(),
                    ],
                )
            });
            let cleanup = adapter_process(
                root,
                "kubectl",
                vec![
                    "delete".to_owned(),
                    format!("job/{name}"),
                    "--namespace".to_owned(),
                    namespace.to_owned(),
                    "--ignore-not-found=true".to_owned(),
                    "--wait=true".to_owned(),
                ],
            );
            apply?;
            cleanup?;
        }
        _ => {
            return Err(PrismError::new(
                "PP7601",
                "contract migration target is unsupported",
            ));
        }
    }
    Ok(())
}

/// Execute the separately authorized contract phase and advance the data compatibility floor.
pub fn finalize_contract(
    root: &Path,
    reference: &str,
    target_name: &str,
    authorized: bool,
) -> Result<Value, PrismError> {
    let _guard = operation_guard(root, target_name)?;
    if !authorized {
        return Err(PrismError::new(
            "PP7701",
            "contract migration requires explicit protected-environment authorization",
        ));
    }
    let digest = root_digest(reference)?;
    let model = system(root, digest)?;
    if !model["migrations"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|row| row["kind"] == "contract")
    {
        return Err(PrismError::new(
            "PP7601",
            "release has no modeled contract migration",
        ));
    }
    let (_, state) = recorded_state(root, target_name)?;
    let mut state = state.ok_or_else(|| {
        PrismError::new("PP7601", "contract migration requires a ready deployment")
    })?;
    let target_profile = state["target_profile"]
        .as_str()
        .unwrap_or(target_name)
        .to_owned();
    let release_trust = release_trust_for_target(root, digest, &model, &target_profile)?;
    if state["release_digest"].as_str() != Some(digest)
        || state["ready"] != true
        || state["drift"] != "none"
    {
        return Err(PrismError::new(
            "PP7601",
            "contract migration requires the exact ready, drift-free release",
        ));
    }
    if state["migration_phase"] == "contract-finalized"
        && state["migration_release_digest"].as_str() != Some(digest)
    {
        return Err(PrismError::new(
            "PP7601",
            "a different data compatibility floor is already finalized",
        ));
    }
    oci::artifact(root, digest, "projections/contract.sql")?;
    let target_model = target(&model, &target_profile)?;
    let kind = target_model["kind"].as_str().unwrap_or_default();
    let (_, projection_bytes) = projection(root, digest, kind)?;
    let projection_path = persist_projection(root, target_name, &projection_bytes)?;
    execute_contract_migration(root, digest, target_name, kind, &model, &projection_path)?;
    state["migration_phase"] = Value::String("contract-finalized".to_owned());
    state["migration_release_digest"] = Value::String(digest.to_owned());
    let state = CanonicalDocument::from_value("prismpm/deployment-state/1", state)?;
    atomic(&state_path(root, target_name), state.bytes())?;
    let plan = plan(root, reference, target_name)?;
    let check = json!({
        "evidence_digest":sha(state.bytes()),
        "id":"data-contract-finalized",
        "kind":"migration",
        "status":"passed"
    });
    let evidence = deployment_evidence(
        root,
        &plan,
        DeploymentEvidenceInput {
            target_profile: &target_profile,
            observed_state: state.value()["observed_digest"]
                .as_str()
                .unwrap_or_default(),
            operation: "contract-migration",
            status: "accepted",
            model: &model,
            release_trust: release_trust.as_ref(),
            extra_checks: &[check],
        },
    )?;
    oci::attach_referrer(root, digest, DEPLOYMENT_EVIDENCE, &evidence)?;
    serde_json::from_slice(&evidence).map_err(|error| PrismError::new("PP9001", error.to_string()))
}

/// Remove only an explicitly modeled deployment after separate authorization.
pub fn destroy(
    root: &Path,
    reference: &str,
    target_name: &str,
    authorized: bool,
) -> Result<Value, PrismError> {
    let _guard = operation_guard(root, target_name)?;
    if !authorized {
        return Err(PrismError::new("PP7701", "destroy requires --authorized"));
    }
    let digest = root_digest(reference)?;
    let model = system(root, digest)?;
    if model["lifecycle"]["retirement"]
        .as_str()
        .unwrap_or_default()
        .is_empty()
    {
        return Err(PrismError::new("PP7701", "retirement policy is absent"));
    }
    let (_, recorded) = recorded_state(root, target_name)?;
    let target_profile = recorded
        .as_ref()
        .and_then(|state| state["target_profile"].as_str())
        .unwrap_or(target_name);
    let release_trust = release_trust_for_target(root, digest, &model, target_profile)?;
    let target_model = target(&model, target_profile)?;
    let kind = target_model["kind"].as_str().unwrap_or_default();
    let (_, bytes) = projection(root, digest, kind)?;
    let plan = plan_bound(root, reference, target_name, target_profile)?;
    let file = persist_projection(root, target_name, &bytes)?;
    invoke(root, kind, target_name, &file, "destroy")?;
    let observed_digest = sha(&empty_observation(target_name)?);
    let evidence = deployment_evidence(
        root,
        &plan,
        DeploymentEvidenceInput {
            target_profile,
            observed_state: &observed_digest,
            operation: "destroy",
            status: "destroyed",
            model: &model,
            release_trust: release_trust.as_ref(),
            extra_checks: &[],
        },
    )?;
    oci::attach_referrer(root, digest, DEPLOYMENT_EVIDENCE, &evidence)?;
    let state = state_path(root, target_name);
    if state.exists() {
        std::fs::remove_file(state)
            .map_err(|error| PrismError::new("PP7701", error.to_string()))?;
    }
    serde_json::from_slice(&evidence).map_err(|error| PrismError::new("PP9001", error.to_string()))
}

fn install_signal_handler() -> Result<(), PrismError> {
    SIGNAL_HANDLER.call_once(|| {
        if ctrlc::set_handler(|| RUN_CANCELLED.store(true, Ordering::SeqCst)).is_err() {
            SIGNAL_HANDLER_FAILED.store(true, Ordering::SeqCst);
        }
    });
    if SIGNAL_HANDLER_FAILED.load(Ordering::SeqCst) {
        return Err(PrismError::new(
            "PP7301",
            "SIGINT/SIGTERM cancellation handler could not be installed",
        ));
    }
    RUN_CANCELLED.store(false, Ordering::SeqCst);
    Ok(())
}

fn pump_logs<R: Read, W: Write>(mut reader: R, sink: &Arc<Mutex<W>>) -> Result<(), String> {
    let mut buffer = [0_u8; 8192];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            return Ok(());
        }
        let mut sink = sink
            .lock()
            .map_err(|_| "log sink lock poisoned".to_owned())?;
        sink.write_all(&buffer[..count])
            .and_then(|()| sink.flush())
            .map_err(|error| error.to_string())?;
    }
}

fn supervise_process<W: Write + Send + 'static>(
    program: &Path,
    arguments: &[String],
    root: &Path,
    cancelled: &AtomicBool,
    sink: Arc<Mutex<W>>,
) -> Result<(), PrismError> {
    let mut command = Command::new(program);
    command
        .args(arguments)
        .current_dir(root)
        .env_clear()
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in [
        "PATH",
        "HOME",
        "XDG_CACHE_HOME",
        "DOCKER_CONFIG",
        "KUBECONFIG",
    ] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
    let mut child = command
        .spawn()
        .map_err(|error| PrismError::new("PP7301", format!("start Compose logs: {error}")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| PrismError::new("PP7301", "Compose log stdout pipe is absent"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| PrismError::new("PP7301", "Compose log stderr pipe is absent"))?;
    let stdout_sink = Arc::clone(&sink);
    let stdout_thread = thread::spawn(move || pump_logs(stdout, &stdout_sink));
    let stderr_thread = thread::spawn(move || pump_logs(stderr, &sink));
    let status = loop {
        if cancelled.load(Ordering::SeqCst) {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| PrismError::new("PP7301", format!("Compose logs: {error}")))?
            {
                break (status, true);
            }
            child.kill().map_err(|error| {
                PrismError::new("PP7301", format!("stop Compose logs: {error}"))
            })?;
            let status = child.wait().map_err(|error| {
                PrismError::new("PP7301", format!("wait Compose logs: {error}"))
            })?;
            break (status, true);
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| PrismError::new("PP7301", format!("Compose logs: {error}")))?
        {
            break (status, false);
        }
        thread::sleep(Duration::from_millis(50));
    };
    for result in [stdout_thread.join(), stderr_thread.join()] {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                return Err(PrismError::new(
                    "PP7301",
                    format!("stream Compose logs: {error}"),
                ))
            }
            Err(_) => return Err(PrismError::new("PP7301", "Compose log pump panicked")),
        }
    }
    if !status.1 {
        return Err(PrismError::new(
            "PP7301",
            format!(
                "Compose log stream exited before cancellation: {}",
                status.0
            ),
        ));
    }
    Ok(())
}

fn supervise_compose(root: &Path, target_name: &str, projection: &Path) -> Result<(), PrismError> {
    let docker = crate::sdk::executable("docker")?;
    let arguments = compose_arguments(target_name, projection, "logs")?;
    supervise_process(
        &docker,
        &arguments,
        root,
        &RUN_CANCELLED,
        Arc::new(Mutex::new(std::io::stderr())),
    )
}

fn supervise_and_shutdown<S, D>(
    supervise: S,
    shutdown: D,
) -> (Result<(), PrismError>, Result<(), PrismError>)
where
    S: FnOnce() -> Result<(), PrismError>,
    D: FnOnce() -> Result<(), PrismError>,
{
    let supervision = supervise();
    let shutdown = shutdown();
    (supervision, shutdown)
}

fn require_clean_run_namespace(
    root: &Path,
    digest: &str,
    target_name: &str,
    model: &Value,
) -> Result<(), PrismError> {
    if recorded_state(root, target_name)?.1.is_some() {
        return Err(PrismError::new(
            "PP7201",
            "local run requires a clean isolated target namespace",
        ));
    }
    let (_, bytes) = projection(root, digest, "compose")?;
    let path = persist_projection(root, target_name, &bytes)?;
    let observation: Value = serde_json::from_slice(&live_observation(
        root,
        target_name,
        "compose",
        model,
        &path,
    )?)
    .map_err(|error| PrismError::new("PP7501", format!("run observation: {error}")))?;
    if observation["resources"]
        .as_array()
        .is_none_or(|resources| !resources.is_empty())
    {
        return Err(PrismError::new(
            "PP7201",
            "local run namespace already contains Compose resources",
        ));
    }
    Ok(())
}

fn clear_run_state(root: &Path, target_name: &str) -> Result<(), PrismError> {
    for path in [
        state_path(root, target_name),
        history_path(root, target_name),
    ] {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(PrismError::new(
                    "PP7301",
                    format!("run state cleanup: {error}"),
                ))
            }
        }
    }
    Ok(())
}

/// Run an exact Compose release in an isolated namespace.
///
/// Foreground mode streams logs to stderr until SIGINT/SIGTERM, then always
/// executes Compose's graceful reverse-dependency shutdown. Detached mode is
/// explicit and returns only after the existing readiness and acceptance gates.
pub fn run(
    root: &Path,
    reference: &str,
    target_name: &str,
    detach: bool,
) -> Result<Value, PrismError> {
    target_id(target_name)?;
    let digest = root_digest(reference)?.to_owned();
    let model = system(root, &digest)?;
    let kind = target(&model, target_name)?["kind"]
        .as_str()
        .unwrap_or_default();
    if kind != "compose" {
        return Err(PrismError::new(
            "PP7101",
            "local run supports only an explicitly modeled Compose target",
        ));
    }
    require_clean_run_namespace(root, &digest, target_name, &model)?;
    if detach {
        let evidence = deploy(root, reference, target_name)?;
        return Ok(json!({
            "deployment":evidence,
            "detached":true,
            "release_digest":digest,
            "schema":"prismpm/run-result/1",
            "status":"active",
            "target":target_name
        }));
    }
    install_signal_handler()?;
    let _guard = operation_guard(root, target_name)?;
    let projection = projection_path(root, target_name);
    let evidence = match apply_plan_inner(
        root,
        reference,
        target_name,
        ApplyRequest {
            target_profile: target_name,
            requested_plan: None,
            operation: "deploy",
            accepted_status: "accepted",
            automatic_rollback: true,
        },
    ) {
        Ok(evidence) => evidence,
        Err(error) if RUN_CANCELLED.load(Ordering::SeqCst) => {
            let shutdown = projection
                .is_file()
                .then(|| invoke(root, "compose", target_name, &projection, "destroy"));
            return match shutdown {
                Some(Err(shutdown)) => Err(error.with_note(format!(
                    "cancellation cleanup failed with {}: {}",
                    shutdown.code.as_str(),
                    shutdown.message
                ))),
                _ => {
                    clear_run_state(root, target_name)?;
                    Err(error.with_note("run was cancelled during deployment"))
                }
            };
        }
        Err(error) => return Err(error),
    };
    let (supervision, shutdown) = supervise_and_shutdown(
        || supervise_compose(root, target_name, &projection),
        || invoke(root, "compose", target_name, &projection, "destroy"),
    );
    if shutdown.is_ok() {
        clear_run_state(root, target_name)?;
    }
    match (supervision, shutdown) {
        (Ok(()), Ok(())) => Ok(json!({
            "deployment":evidence,
            "detached":false,
            "release_digest":digest,
            "schema":"prismpm/run-result/1",
            "shutdown":"compose-reverse-dependency-graceful",
            "status":"stopped",
            "target":target_name
        })),
        (Err(supervision), Ok(())) => Err(supervision),
        (Ok(()), Err(shutdown)) => Err(shutdown),
        (Err(supervision), Err(shutdown)) => Err(supervision.with_note(format!(
            "graceful Compose shutdown also failed with {}: {}",
            shutdown.code.as_str(),
            shutdown.message
        ))),
    }
}

/// Capture a complete logical PostgreSQL snapshot from a modeled live target.
/// The returned snapshot remains pending until `restore` proves it in a clean target.
fn backup_inner(root: &Path, reference: &str, target_name: &str) -> Result<Value, PrismError> {
    let digest = root_digest(reference)?;
    target_id(target_name)?;
    let model = system(root, digest)?;
    let (_, recorded) = recorded_state(root, target_name)?;
    let target_profile = recorded
        .as_ref()
        .and_then(|state| state["target_profile"].as_str())
        .unwrap_or(target_name);
    release_trust_for_target(root, digest, &model, target_profile)?;
    recovery_policy(&model)?;
    let target_model = target(&model, target_profile)?;
    let kind = target_model["kind"].as_str().unwrap_or_default();
    let (_, projection_bytes) = projection(root, digest, kind)?;
    let projection_path = persist_projection(root, target_name, &projection_bytes)?;
    let observed = live_observation(root, target_name, kind, &model, &projection_path)?;
    if !observation_ready(&observed)? {
        return Err(PrismError::new(
            "PP7901",
            "backup requires a ready modeled deployment",
        ));
    }
    let dump = logical_dump(root, kind, target_name, &model, &projection_path)?;
    let snapshot_digest = sha(&dump);
    let directory = root.join(".prism/backups").join(target_name);
    std::fs::create_dir_all(&directory)
        .map_err(|error| PrismError::new("PP7901", format!("backup directory: {error}")))?;
    let dump_path = directory.join(format!(
        "{}.sql",
        snapshot_digest.trim_start_matches("sha256:")
    ));
    if dump_path.exists() {
        if std::fs::read(&dump_path)
            .map_err(|error| PrismError::new("PP7901", error.to_string()))?
            != dump
        {
            return Err(PrismError::new(
                "PP7901",
                "backup snapshot identity changed",
            ));
        }
    } else {
        atomic(&dump_path, &dump)?;
    }
    let captured_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrismError::new("PP7901", format!("system clock: {error}")))?
        .as_secs();
    let metadata = json!({
        "captured_unix":captured_unix,
        "dump_path":dump_path.strip_prefix(root).map_err(|_| PrismError::new("PP7901", "backup path escaped"))?.to_string_lossy().replace('\\', "/"),
        "release_digest":digest,
        "schema":"prismpm/backup-snapshot/1",
        "snapshot_digest":snapshot_digest,
        "source_observation":sha(&observed),
        "source_target":target_name,
        "status":"pending-restore"
    });
    let metadata_bytes = encode_value(&metadata)?;
    let metadata_path = directory.join(format!(
        "{}.json",
        snapshot_digest.trim_start_matches("sha256:")
    ));
    atomic(&metadata_path, &metadata_bytes)?;
    Ok(metadata)
}

/// Capture a complete logical PostgreSQL snapshot from a modeled live target.
/// The returned snapshot remains pending until `restore` proves it in a clean target.
pub fn backup(root: &Path, reference: &str, target_name: &str) -> Result<Value, PrismError> {
    let _guard = operation_guard(root, target_name)?;
    backup_inner(root, reference, target_name)
}

fn restore_compose_database(
    root: &Path,
    target_name: &str,
    projection_path: &Path,
    model: &Value,
    dump_path: &Path,
) -> Result<(), PrismError> {
    adapter_process(
        root,
        "docker",
        vec![
            "compose".to_owned(),
            "--project-name".to_owned(),
            target_name.to_owned(),
            "--file".to_owned(),
            projection_path.to_string_lossy().into_owned(),
            "up".to_owned(),
            "--detach".to_owned(),
            "--wait".to_owned(),
            "--no-build".to_owned(),
            "database".to_owned(),
        ],
    )?;
    adapter_process(
        root,
        "docker",
        vec![
            "compose".to_owned(),
            "--project-name".to_owned(),
            target_name.to_owned(),
            "--file".to_owned(),
            projection_path.to_string_lossy().into_owned(),
            "cp".to_owned(),
            dump_path.to_string_lossy().into_owned(),
            "database:/tmp/prismpm-restore.sql".to_owned(),
        ],
    )?;
    adapter_process(
        root,
        "docker",
        vec![
            "compose".to_owned(),
            "--project-name".to_owned(),
            target_name.to_owned(),
            "--file".to_owned(),
            projection_path.to_string_lossy().into_owned(),
            "exec".to_owned(),
            "-T".to_owned(),
            "database".to_owned(),
            "psql".to_owned(),
            "--username".to_owned(),
            parameter_default(model, "postgres-user")?.to_owned(),
            "--dbname".to_owned(),
            parameter_default(model, "postgres-db")?.to_owned(),
            "--set=ON_ERROR_STOP=1".to_owned(),
            "--file=/tmp/prismpm-restore.sql".to_owned(),
        ],
    )?;
    Ok(())
}

fn restore_kubernetes_database(
    root: &Path,
    model: &Value,
    dump_path: &Path,
) -> Result<(), PrismError> {
    let namespace = model["product"]["id"].as_str().unwrap_or_default();
    let pod = database_pod(root, namespace)?;
    adapter_process(
        root,
        "kubectl",
        vec![
            "cp".to_owned(),
            dump_path.to_string_lossy().into_owned(),
            format!("{namespace}/{pod}:/tmp/prismpm-restore.sql"),
        ],
    )?;
    for arguments in [
        vec![
            "exec",
            "--namespace",
            namespace,
            pod.as_str(),
            "--",
            "psql",
            "--username",
            parameter_default(model, "postgres-user")?,
            "--dbname",
            parameter_default(model, "postgres-db")?,
            "--set=ON_ERROR_STOP=1",
            "--command=DROP SCHEMA public CASCADE; CREATE SCHEMA public;",
        ],
        vec![
            "exec",
            "--namespace",
            namespace,
            pod.as_str(),
            "--",
            "psql",
            "--username",
            parameter_default(model, "postgres-user")?,
            "--dbname",
            parameter_default(model, "postgres-db")?,
            "--set=ON_ERROR_STOP=1",
            "--file=/tmp/prismpm-restore.sql",
        ],
    ] {
        adapter_process(
            root,
            "kubectl",
            arguments.into_iter().map(str::to_owned).collect(),
        )?;
    }
    Ok(())
}

/// Restore a pending snapshot into a distinct clean target and accept the
/// backup only after exact logical-content and deployment-readiness checks.
pub fn restore(
    root: &Path,
    reference: &str,
    source_target: &str,
    restore_target: &str,
    snapshot: &Path,
) -> Result<Value, PrismError> {
    let _guard = operation_guard(root, restore_target)?;
    let started = Instant::now();
    let digest = root_digest(reference)?;
    target_id(source_target)?;
    target_id(restore_target)?;
    let model = system(root, digest)?;
    let release_trust = release_trust_for_target(root, digest, &model, source_target)?;
    if source_target == restore_target {
        return Err(PrismError::new(
            "PP7901",
            "restore target must be distinct from the source target",
        ));
    }
    if state_path(root, restore_target).exists() {
        return Err(PrismError::new("PP7901", "restore target is not clean"));
    }
    let snapshot_path = if snapshot.is_absolute() {
        snapshot.to_path_buf()
    } else {
        root.join(snapshot)
    };
    let canonical_snapshot = snapshot_path
        .canonicalize()
        .map_err(|_| PrismError::new("PP7901", "backup metadata is absent"))?;
    let backup_root = root
        .join(".prism/backups")
        .canonicalize()
        .map_err(|_| PrismError::new("PP7901", "backup root is absent"))?;
    if !canonical_snapshot.starts_with(&backup_root) {
        return Err(PrismError::new("PP7901", "backup metadata path escaped"));
    }
    let metadata: Value = serde_json::from_slice(
        &std::fs::read(&canonical_snapshot)
            .map_err(|error| PrismError::new("PP7901", format!("backup metadata: {error}")))?,
    )
    .map_err(|error| PrismError::new("PP7901", format!("backup metadata: {error}")))?;
    if metadata["schema"] != "prismpm/backup-snapshot/1"
        || metadata["status"] != "pending-restore"
        || metadata["release_digest"] != digest
        || metadata["source_target"] != source_target
    {
        return Err(PrismError::new(
            "PP7901",
            "backup metadata does not match the restore request",
        ));
    }
    let dump_relative = metadata["dump_path"]
        .as_str()
        .ok_or_else(|| PrismError::new("PP7901", "backup dump path is absent"))?;
    let dump_path = root
        .join(dump_relative)
        .canonicalize()
        .map_err(|_| PrismError::new("PP7901", "backup dump is absent"))?;
    if !dump_path.starts_with(&backup_root) {
        return Err(PrismError::new("PP7901", "backup dump path escaped"));
    }
    let expected_dump = std::fs::read(&dump_path)
        .map_err(|error| PrismError::new("PP7901", format!("backup dump: {error}")))?;
    if metadata["snapshot_digest"] != sha(&expected_dump) {
        return Err(PrismError::new("PP7901", "backup dump digest changed"));
    }
    recovery_policy(&model)?;
    let target_model = target(&model, source_target)?;
    let kind = target_model["kind"].as_str().unwrap_or_default();
    let (_, projection_bytes) = projection(root, digest, kind)?;
    let projection_path = persist_projection(root, restore_target, &projection_bytes)?;
    let clean_observation = live_observation(root, restore_target, kind, &model, &projection_path)?;
    if clean_observation != empty_observation(restore_target)?
        && serde_json::from_slice::<Value>(&clean_observation)
            .ok()
            .and_then(|value| value["resources"].as_array().map(Vec::len))
            .unwrap_or(1)
            != 0
    {
        return Err(PrismError::new(
            "PP7901",
            "restore target contains observed resources",
        ));
    }
    materialize_release(root, digest, restore_target)?;
    match kind {
        "compose" => {
            restore_compose_database(root, restore_target, &projection_path, &model, &dump_path)?
        }
        "kubernetes" => {
            invoke(root, kind, restore_target, &projection_path, "deploy")?;
            wait_ready(root, kind, &model)?;
            restore_kubernetes_database(root, &model, &dump_path)?;
        }
        _ => return Err(PrismError::new("PP7901", "restore target is unsupported")),
    }
    let _ = apply_plan_inner(
        root,
        reference,
        restore_target,
        ApplyRequest {
            target_profile: source_target,
            requested_plan: None,
            operation: "restore",
            accepted_status: "accepted",
            automatic_rollback: false,
        },
    )?;
    let restored_dump = logical_dump(root, kind, restore_target, &model, &projection_path)?;
    if restored_dump != expected_dump {
        return Err(PrismError::new(
            "PP7901",
            "restored logical database content differs from the snapshot",
        ));
    }
    let completed_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrismError::new("PP7901", format!("system clock: {error}")))?
        .as_secs();
    let captured_unix = metadata["captured_unix"]
        .as_u64()
        .ok_or_else(|| PrismError::new("PP7901", "backup capture time is absent"))?;
    let actual_rpo_seconds = completed_unix.saturating_sub(captured_unix);
    let actual_rto_millis = started.elapsed().as_millis() as u64;
    let persistence = model["persistence"]
        .as_array()
        .into_iter()
        .flatten()
        .next()
        .ok_or_else(|| PrismError::new("PP7901", "persistence model is absent"))?;
    if actual_rpo_seconds > persistence["rpo_seconds"].as_u64().unwrap_or(0)
        || actual_rto_millis > persistence["rto_seconds"].as_u64().unwrap_or(0) * 1000
    {
        return Err(PrismError::new(
            "PP7901",
            "restore exceeded the modeled RPO or RTO",
        ));
    }
    let recovery = encode_value(&json!({
        "actual_rpo_seconds":actual_rpo_seconds,
        "actual_rto_millis":actual_rto_millis,
        "completed_unix":completed_unix,
        "integrity_digest":sha(&restored_dump),
        "release_digest":digest,
        "restored_target":restore_target,
        "schema":"prismpm/recovery-result/1",
        "snapshot_digest":metadata["snapshot_digest"],
        "source_target":source_target,
        "status":"accepted"
    }))?;
    let recovery_referrer = oci::attach_referrer(
        root,
        digest,
        "application/vnd.prismpm.recovery.v1+json",
        &recovery,
    )?;
    let plan = plan_bound(root, reference, restore_target, source_target)?;
    let observed = status_bound(root, reference, restore_target, source_target)?;
    let observed_digest = sha(&encode_value(&observed)?);
    let checks = vec![
        json!({"evidence_digest":metadata["snapshot_digest"],"id":"logical-backup","kind":"backup","status":"passed"}),
        json!({"evidence_digest":sha(&restored_dump),"id":"clean-target-restore","kind":"restore","status":"passed"}),
        json!({"evidence_digest":sha(&recovery),"id":"rpo-rto-measurement","kind":"restore","status":"measured"}),
    ];
    let evidence = deployment_evidence(
        root,
        &plan,
        DeploymentEvidenceInput {
            target_profile: source_target,
            observed_state: &observed_digest,
            operation: "restore",
            status: "accepted",
            model: &model,
            release_trust: release_trust.as_ref(),
            extra_checks: &checks,
        },
    )?;
    let deployment_referrer = oci::attach_referrer(root, digest, DEPLOYMENT_EVIDENCE, &evidence)?;
    Ok(json!({
        "deployment_evidence":serde_json::from_slice::<Value>(&evidence).map_err(|error| PrismError::new("PP9001", error.to_string()))?,
        "deployment_referrer":deployment_referrer,
        "recovery":serde_json::from_slice::<Value>(&recovery).map_err(|error| PrismError::new("PP9001", error.to_string()))?,
        "recovery_referrer":recovery_referrer,
        "schema":"prismpm/restore-result/1"
    }))
}

#[cfg(test)]
mod tests {
    use super::{
        clear_run_state, compose_arguments, history_path, kubernetes_partition,
        reference_with_digest, scan_secret_directory, state_path, supervise_and_shutdown,
        supervise_process, target_id,
    };
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn malformed_and_injectable_targets_are_rejected() {
        assert!(target_id("compose-local").is_ok());
        assert!(target_id("../bad").is_err());
        assert!(target_id("bad; touch owned").is_err());
    }

    #[test]
    fn immutable_release_name_is_preserved_when_selecting_a_prior_digest() {
        let digest = format!("sha256:{}", "b".repeat(64));
        assert_eq!(
            reference_with_digest(
                &format!("registry.example/uor/calculator@sha256:{}", "a".repeat(64)),
                &digest
            )
            .unwrap(),
            format!("registry.example/uor/calculator@{digest}")
        );
        assert!(reference_with_digest("calculator:latest", &digest).is_err());
        assert!(reference_with_digest(
            &format!("registry.example/uor/calculator@sha256:{}", "a".repeat(64)),
            "not-a-digest"
        )
        .is_err());
    }

    #[test]
    fn compose_run_argv_is_isolated_digest_only_and_never_builds() {
        let projection = std::path::Path::new("/project/.prism/targets/local/projection.json");
        assert_eq!(
            compose_arguments("local", projection, "deploy").unwrap(),
            [
                "compose",
                "--project-name",
                "local",
                "--file",
                "/project/.prism/targets/local/projection.json",
                "up",
                "--detach",
                "--wait",
                "--no-build"
            ]
        );
        assert_eq!(
            compose_arguments("local", projection, "logs").unwrap(),
            [
                "compose",
                "--project-name",
                "local",
                "--file",
                "/project/.prism/targets/local/projection.json",
                "logs",
                "--follow",
                "--no-color",
                "--timestamps=false"
            ]
        );
        assert_eq!(
            compose_arguments("local", projection, "destroy").unwrap(),
            [
                "compose",
                "--project-name",
                "local",
                "--file",
                "/project/.prism/targets/local/projection.json",
                "down",
                "--remove-orphans"
            ]
        );
    }

    #[test]
    fn kubernetes_deployment_stages_only_official_ingress_infrastructure() {
        let projection = json!({
            "apiVersion":"v1",
            "kind":"List",
            "items":[
                {"apiVersion":"v1","kind":"Namespace","metadata":{"labels":{"app.kubernetes.io/name":"ingress-nginx"},"name":"ingress-nginx"}},
                {"apiVersion":"apps/v1","kind":"Deployment","metadata":{"labels":{"app.kubernetes.io/name":"ingress-nginx"},"name":"ingress-nginx-controller","namespace":"ingress-nginx"}},
                {"apiVersion":"networking.k8s.io/v1","kind":"Ingress","metadata":{"labels":{"app.kubernetes.io/name":"calculator-api"},"name":"calculator","namespace":"calculator-system"}},
                {"apiVersion":"apps/v1","kind":"Deployment","metadata":{"labels":{"app.kubernetes.io/name":"api"},"name":"api","namespace":"calculator-system"}}
            ]
        });
        let (infrastructure, application) = kubernetes_partition(&projection).unwrap();
        assert_eq!(infrastructure["items"].as_array().unwrap().len(), 2);
        assert_eq!(application["items"].as_array().unwrap().len(), 2);
        assert!(infrastructure["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| {
                item.pointer("/metadata/labels/app.kubernetes.io~1name")
                    .and_then(serde_json::Value::as_str)
                    == Some("ingress-nginx")
            }));
        assert!(application["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| { item["kind"] == "Ingress" && item["metadata"]["name"] == "calculator" }));
    }

    #[test]
    fn foreground_shutdown_clears_deployment_and_rollback_state() {
        let directory = tempfile::tempdir().unwrap();
        let state = state_path(directory.path(), "local");
        let history = history_path(directory.path(), "local");
        std::fs::create_dir_all(state.parent().unwrap()).unwrap();
        std::fs::write(&state, b"state").unwrap();
        std::fs::write(&history, b"history").unwrap();
        clear_run_state(directory.path(), "local").unwrap();
        assert!(!state.exists());
        assert!(!history.exists());
        clear_run_state(directory.path(), "local").unwrap();
    }

    #[test]
    fn secret_redaction_scans_nested_raw_and_encoded_values_and_rejects_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        let nested = directory.path().join("product/provider");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("password"), b"correct horse battery staple\n").unwrap();
        assert!(scan_secret_directory(directory.path(), "redacted").is_ok());
        assert!(scan_secret_directory(directory.path(), "correct horse battery staple").is_err());
        assert!(scan_secret_directory(
            directory.path(),
            "Y29ycmVjdCBob3JzZSBiYXR0ZXJ5IHN0YXBsZQ=="
        )
        .is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(nested.join("password"), directory.path().join("alias"))
                .unwrap();
            assert!(scan_secret_directory(directory.path(), "redacted").is_err());
        }
    }

    #[test]
    fn shutdown_runs_even_when_foreground_supervision_fails() {
        let shutdown_called = AtomicBool::new(false);
        let (supervision, shutdown) = supervise_and_shutdown(
            || Err(super::PrismError::new("PP7301", "planted log failure")),
            || {
                shutdown_called.store(true, Ordering::SeqCst);
                Ok(())
            },
        );
        assert!(supervision.is_err());
        assert!(shutdown.is_ok());
        assert!(shutdown_called.load(Ordering::SeqCst));
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_stops_log_process_and_keeps_logs_out_of_result_channel() {
        assert_cancellation_drains_logs("");
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_log_assertions_wait_for_a_delayed_child() {
        assert_cancellation_drains_logs("sleep 0.25\n");
    }

    #[cfg(unix)]
    fn assert_cancellation_drains_logs(startup: &str) {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let program = directory.path().join("fake-docker");
        let argv = directory.path().join("fake-docker.argv");
        std::fs::write(
            &program,
            format!(
                "#!/bin/sh\n{startup}printf '%s\\n' \"$@\" > '{}'\nprintf 'stdout-log\\n'\nprintf 'stderr-log\\n' >&2\nwhile :; do :; done\n",
                argv.display()
            ),
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&program).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&program, permissions).unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let trigger = Arc::clone(&cancelled);
        let sink = Arc::new(Mutex::new(Vec::new()));
        let observed_sink = Arc::clone(&sink);
        let trigger_thread = std::thread::spawn(move || {
            // Cancel after observing both streams, not after assuming the child
            // received CPU time. The deadline still cancels a broken fixture.
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            let observed = loop {
                let logs = observed_sink.lock().unwrap();
                let both = logs
                    .windows(b"stdout-log".len())
                    .any(|part| part == b"stdout-log")
                    && logs
                        .windows(b"stderr-log".len())
                        .any(|part| part == b"stderr-log");
                drop(logs);
                if both {
                    break true;
                }
                if std::time::Instant::now() >= deadline {
                    break false;
                }
                std::thread::sleep(Duration::from_millis(5));
            };
            trigger.store(true, Ordering::SeqCst);
            observed
        });
        let result = supervise_process(
            &program,
            &[
                "compose".to_owned(),
                "logs".to_owned(),
                "--follow".to_owned(),
            ],
            directory.path(),
            &cancelled,
            Arc::clone(&sink),
        );
        let observed = trigger_thread.join().unwrap();
        result.unwrap();
        assert!(
            observed,
            "child logs were not observed before the cancellation deadline"
        );
        let logs = String::from_utf8(sink.lock().unwrap().clone()).unwrap();
        assert!(logs.contains("stdout-log"));
        assert!(logs.contains("stderr-log"));
        assert_eq!(
            std::fs::read_to_string(argv).unwrap(),
            "compose\nlogs\n--follow\n"
        );
    }
}
