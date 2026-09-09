//! Internal SDK evidence tool for cross-platform product-build comparison.

use clap::Parser;
use prismpm::contracts::CanonicalDocument;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

#[derive(Parser)]
#[command(name = "prismpm-platform-equivalence")]
struct Arguments {
    #[arg(long)]
    left_root: PathBuf,
    #[arg(long)]
    right_root: PathBuf,
    #[arg(long)]
    left_architecture: String,
    #[arg(long)]
    right_architecture: String,
    #[arg(long)]
    release_digest: String,
    #[arg(long)]
    sdk_digest: String,
    #[arg(long)]
    system_model: PathBuf,
}

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn valid_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn files(root: &Path) -> Result<BTreeMap<String, (String, u64)>, String> {
    let canonical = root
        .canonicalize()
        .map_err(|error| format!("comparison root {}: {error}", root.display()))?;
    if !canonical.is_dir() {
        return Err(format!(
            "comparison root is not a directory: {}",
            root.display()
        ));
    }
    let mut rows = BTreeMap::new();
    for entry in walkdir::WalkDir::new(&canonical).follow_links(false) {
        let entry = entry.map_err(|error| format!("comparison tree: {error}"))?;
        if entry.path() == canonical {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(&canonical)
            .map_err(|_| "comparison path escaped its root".to_owned())?;
        if relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        }) {
            return Err("comparison path is not relative".to_owned());
        }
        let name = relative
            .to_str()
            .ok_or_else(|| "comparison path is not UTF-8".to_owned())?
            .replace('\\', "/");
        let kind = entry.file_type();
        if kind.is_dir() {
            continue;
        }
        if !kind.is_file() {
            return Err(format!(
                "comparison tree contains a non-regular entry: {name}"
            ));
        }
        let bytes = std::fs::read(entry.path())
            .map_err(|error| format!("read comparison entry {name}: {error}"))?;
        rows.insert(name, (sha(&bytes), bytes.len() as u64));
    }
    if rows.is_empty() {
        return Err("comparison tree has no files".to_owned());
    }
    Ok(rows)
}

fn platform_artifacts(model: &Value) -> Result<Vec<Value>, String> {
    let requirements = model["platform_requirements"]
        .as_array()
        .ok_or_else(|| "system model platform requirements are absent".to_owned())?
        .iter()
        .map(|row| {
            row["id"]
                .as_str()
                .map(|id| (id.to_owned(), row))
                .ok_or_else(|| "platform requirement ID is absent".to_owned())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let mut artifacts = Vec::new();
    for artifact in model["artifacts"]
        .as_array()
        .ok_or_else(|| "system model artifacts are absent".to_owned())?
    {
        let artifact_id = artifact["id"]
            .as_str()
            .ok_or_else(|| "artifact ID is absent".to_owned())?;
        let mut requirement_ids = artifact["platform_requirements"]
            .as_array()
            .ok_or_else(|| format!("artifact {artifact_id} platform requirements are absent"))?
            .iter()
            .map(|id| {
                id.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("artifact {artifact_id} has a malformed platform ID"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        requirement_ids.sort();
        let mut platforms = requirement_ids
            .iter()
            .map(|id| {
                let row = requirements.get(id).ok_or_else(|| {
                    format!("artifact {artifact_id} names unknown platform requirement {id}")
                })?;
                let mut architectures = row["architectures"]
                    .as_array()
                    .ok_or_else(|| format!("platform requirement {id} architectures are absent"))?
                    .clone();
                architectures.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
                Ok(json!({
                    "architectures":architectures,
                    "os":row["os"],
                    "requirement_id":id,
                    "runtime":row["runtime"],
                    "runtime_version":row["runtime_version"]
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        platforms.sort_by(|left, right| {
            left["requirement_id"]
                .as_str()
                .cmp(&right["requirement_id"].as_str())
        });
        artifacts.push(json!({
            "artifact_id":artifact_id,
            "platform_requirements":requirement_ids,
            "platforms":platforms
        }));
    }
    artifacts.sort_by(|left, right| {
        left["artifact_id"]
            .as_str()
            .cmp(&right["artifact_id"].as_str())
    });
    Ok(artifacts)
}

fn run(arguments: Arguments) -> Result<Vec<u8>, String> {
    for architecture in [
        arguments.left_architecture.as_str(),
        arguments.right_architecture.as_str(),
    ] {
        if !matches!(architecture, "amd64" | "arm64") {
            return Err(format!(
                "unsupported comparison architecture {architecture}"
            ));
        }
    }
    if arguments.left_architecture == arguments.right_architecture {
        return Err("comparison architectures must be distinct".to_owned());
    }
    if !valid_digest(&arguments.release_digest) || !valid_digest(&arguments.sdk_digest) {
        return Err("release or SDK identity is not an immutable SHA-256 digest".to_owned());
    }
    let left = files(&arguments.left_root)?;
    let right = files(&arguments.right_root)?;
    if left != right {
        let paths = left
            .keys()
            .chain(right.keys())
            .filter(|path| left.get(*path) != right.get(*path))
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        return Err(format!(
            "platform-independent build trees differ at: {}",
            paths.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    let model_bytes =
        std::fs::read(&arguments.system_model).map_err(|error| format!("system model: {error}"))?;
    let model = CanonicalDocument::parse("prismpm/system-model/1", &model_bytes)
        .map_err(|error| format!("system model: {error}"))?;
    let file_rows = left
        .iter()
        .map(|(path, (digest, size))| json!({"digest":digest,"path":path,"size":size}))
        .collect::<Vec<_>>();
    let tree_digest = sha(&prismpm::holo::canonical::encode_value(&Value::Array(
        file_rows.clone(),
    ))
    .map_err(|error| format!("comparison manifest: {error}"))?);
    let mut architectures = vec![arguments.left_architecture, arguments.right_architecture];
    architectures.sort();
    let value = json!({
        "architectures":architectures,
        "differences":[],
        "file_count":file_rows.len(),
        "files":file_rows,
        "model_digest":model.digest(),
        "modeled_platform_artifacts":platform_artifacts(model.value())?,
        "platform_independent_tree_digest":tree_digest,
        "release_digest":arguments.release_digest,
        "schema":"prismpm/platform-equivalence/1",
        "sdk_digest":arguments.sdk_digest,
        "status":"accepted"
    });
    CanonicalDocument::from_value("prismpm/platform-equivalence/1", value)
        .map(|document| document.bytes().to_vec())
        .map_err(|error| format!("platform equivalence: {error}"))
}

fn main() {
    let arguments = Arguments::parse();
    match run(arguments) {
        Ok(bytes) => println!("{}", String::from_utf8_lossy(&bytes)),
        Err(error) => {
            eprintln!("platform equivalence failed: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::files;

    #[test]
    fn tree_comparison_rejects_content_changes_and_links() {
        let left = tempfile::tempdir().unwrap();
        let right = tempfile::tempdir().unwrap();
        std::fs::write(left.path().join("artifact"), b"same").unwrap();
        std::fs::write(right.path().join("artifact"), b"same").unwrap();
        assert_eq!(files(left.path()).unwrap(), files(right.path()).unwrap());
        std::fs::write(right.path().join("artifact"), b"changed").unwrap();
        assert_ne!(files(left.path()).unwrap(), files(right.path()).unwrap());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(left.path().join("artifact"), left.path().join("link"))
                .unwrap();
            assert!(files(left.path()).unwrap_err().contains("non-regular"));
        }
    }
}
