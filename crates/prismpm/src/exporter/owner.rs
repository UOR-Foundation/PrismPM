//! Invocation-local exporter custody. Never an authority-bearing caller cache.

use super::{custody, ExecutableMeasurement};
use crate::verification::ProcessRecord;
use crate::PrismError;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    ControllerBuild,
    Replay,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Self::ControllerBuild => "controller-build",
            Self::Replay => "replay",
        }
    }
}

pub(crate) trait ProductEvidence {
    fn role(&self) -> &'static str;
    fn artifacts(&self) -> &[(String, Vec<u8>)];
    fn processes(&self) -> &[ProcessRecord];
}

struct Package {
    directory: tempfile::TempDir,
    path: PathBuf,
    project: PathBuf,
    inventory: Option<String>,
    lock_authority: Option<String>,
    acquisition: Value,
    snapshot: Option<custody::Snapshot>,
    executable: Option<ExecutableMeasurement>,
}

impl Package {
    fn retire(self) -> Result<(), PrismError> {
        if self.snapshot.as_ref() != Some(&custody::capture(&self.path)?) {
            return Err(failure(
                "private exporter custody changed before retirement",
            ));
        }
        if crate::sdk::exporter_seed_inventory(&self.project)? != self.inventory {
            return Err(failure(
                "private exporter inventory authority changed before retirement",
            ));
        }
        if crate::sdk::exporter_lock_authority(&self.project)? != self.lock_authority {
            return Err(failure(
                "private exporter consumer lock authority changed before retirement",
            ));
        }
        self.directory
            .close()
            .map_err(|error| failure(&format!("private exporter retirement: {error}")))
    }
}

/// Constructed inside a verification worker and passed only through private
/// APIs. The owner cannot be cloned, shared, selected by a caller, or persisted.
#[derive(Default)]
pub(crate) struct VerifyExporterOwner {
    package: Option<Package>,
    phases: Vec<Value>,
    poisoned: bool,
}

fn failure(message: &str) -> PrismError {
    PrismError::new("PP5008", message)
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

impl VerifyExporterOwner {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn generate<T: ProductEvidence>(
        &mut self,
        project: &Path,
        phase: Phase,
        model: &[u8],
        lex_manifest: &[u8],
        generation: impl FnOnce(&Path, &Value) -> Result<T, PrismError>,
    ) -> Result<T, PrismError> {
        if !cfg!(all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )) {
            self.abort()?;
            return Err(failure(
                "private exporter custody requires a qualified native Linux SDK platform",
            ));
        }
        if self.poisoned
            || !matches!(
                (self.phases.len(), phase),
                (0, Phase::ControllerBuild) | (1, Phase::Replay)
            )
        {
            self.poisoned = true;
            self.abort()?;
            return Err(failure(
                "private exporter phase is repeated, reordered, or failed",
            ));
        }
        // Poison first: every error, including an unwind, prevents reuse.
        self.poisoned = true;
        let result = self.generate_inner(project, phase, model, lex_manifest, generation);
        if result.is_err() {
            self.abort()?;
        }
        result
    }

    fn abort(&mut self) -> Result<(), PrismError> {
        self.poisoned = true;
        self.phases.clear();
        if let Some(package) = self.package.take() {
            package.directory.close().map_err(|error| {
                failure(&format!("failed private exporter retirement: {error}"))
            })?;
        }
        Ok(())
    }

    fn generate_inner<T: ProductEvidence>(
        &mut self,
        project: &Path,
        phase: Phase,
        model: &[u8],
        lex_manifest: &[u8],
        generation: impl FnOnce(&Path, &Value) -> Result<T, PrismError>,
    ) -> Result<T, PrismError> {
        let project = project
            .canonicalize()
            .map_err(|_| failure("private exporter project is unavailable"))?;
        let inventory = crate::sdk::exporter_seed_inventory(&project)?;
        let lock_authority = crate::sdk::exporter_lock_authority(&project)?;
        if self.package.is_none() {
            let directory = tempfile::Builder::new()
                .prefix("prismpm-verify-exporter-")
                .tempdir()
                .map_err(|error| failure(&format!("private exporter owner: {error}")))?;
            let path = directory.path().join("lean4-prod");
            let acquisition = super::acquire_for(&project, &path)?;
            self.package = Some(Package {
                directory,
                path,
                project: project.clone(),
                inventory: inventory.clone(),
                lock_authority: lock_authority.clone(),
                acquisition,
                snapshot: None,
                executable: None,
            });
        }
        let package = self.package.as_mut().expect("private package established");
        if package.project != project
            || package.inventory != inventory
            || package.lock_authority != lock_authority
        {
            return Err(failure(
                "private exporter source or inventory authority changed",
            ));
        }
        if let Some(snapshot) = &package.snapshot {
            if &custody::capture(&package.path)? != snapshot {
                return Err(failure("private exporter custody changed between phases"));
            }
        }
        // Each generator still builds its fresh generated-module workspace,
        // executes real `lake build prod-export`, and performs its actual export.
        let product = generation(&package.path, &package.acquisition)?;
        let snapshot = custody::capture(&package.path)?;
        if package
            .snapshot
            .as_ref()
            .is_some_and(|prior| !prior.same_compiler(&snapshot))
        {
            return Err(failure(
                "private exporter source or executable identity changed during replay",
            ));
        }
        let executable =
            super::measure_executable(&package.path.join(".lake/build/bin/prod-export"))?;
        if package
            .executable
            .as_ref()
            .is_some_and(|prior| prior != &executable)
        {
            return Err(failure("private exporter executable changed across phases"));
        }
        let mut artifacts = product
            .artifacts()
            .iter()
            .map(|(path, bytes)| json!({"path":path,"byte_length":bytes.len(),"sha256":digest(bytes)}))
            .collect::<Vec<_>>();
        artifacts.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
        let record = json!({
            "phase":phase.name(),
            "role":product.role(),
            "model_sha256":digest(model),
            "lexlean_manifest_sha256":digest(lex_manifest),
            "artifacts":artifacts,
            "processes":product.processes(),
        });
        if let Some(prior) = self.phases.first() {
            for field in [
                "role",
                "model_sha256",
                "lexlean_manifest_sha256",
                "artifacts",
            ] {
                if prior[field] != record[field] {
                    return Err(failure(
                        "private exporter replay inputs or complete products differ",
                    ));
                }
            }
        }
        package.snapshot = Some(snapshot);
        package.executable = Some(executable);
        self.phases.push(record);
        self.poisoned = false;
        Ok(product)
    }

    /// Consume the owner and close its private directory before any acceptance
    /// evidence is published. No owner survives a second verification request.
    pub(crate) fn finish(mut self) -> Result<Value, PrismError> {
        if self.poisoned || self.phases.len() != 2 {
            return Err(failure(
                "private exporter requires both complete generation phases",
            ));
        }
        let record = json!({"schema":"prismpm/verification-exporter-owner/1","phases":self.phases});
        validate_record(
            &record,
            record["phases"][0]["model_sha256"]
                .as_str()
                .expect("recorded model digest"),
            record["phases"][0]["role"].as_str().expect("recorded role"),
        )?;
        let package = self
            .package
            .take()
            .expect("two phases retain one private package");
        package.retire()?;
        Ok(record)
    }
}

/// Structural replay is not producer authentication. The caller additionally
/// binds these original phase records to retained source/artifact/SDK evidence.
pub(crate) fn validate_record(
    value: &Value,
    model_sha256: &str,
    role: &str,
) -> Result<(), PrismError> {
    fn keys(value: &Value, expected: &[&str]) -> Result<(), PrismError> {
        let object = value
            .as_object()
            .ok_or_else(|| failure("exporter owner object required"))?;
        if object.len() != expected.len() || expected.iter().any(|key| !object.contains_key(*key)) {
            return Err(failure("exporter owner fields are not closed"));
        }
        Ok(())
    }
    fn hash(value: &Value) -> bool {
        value.as_str().is_some_and(|text| {
            text.len() == 64
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    }
    keys(value, &["phases", "schema"])?;
    if value["schema"] != "prismpm/verification-exporter-owner/1" {
        return Err(failure("exporter owner version differs"));
    }
    let phases = value["phases"]
        .as_array()
        .ok_or_else(|| failure("exporter owner phases required"))?;
    if phases.len() != 2 {
        return Err(failure("exporter owner requires exactly two phases"));
    }
    let tools = match role {
        "library" => vec![
            "lake-build-generated",
            "lean4-prod-build",
            "prod-export",
            "native-library-package",
        ],
        "application" => vec![
            "application-lean",
            "application-exporter",
            "application-export",
        ],
        _ => return Err(failure("exporter owner role differs")),
    };
    let mut executable = None;
    for (index, phase) in phases.iter().enumerate() {
        keys(
            phase,
            &[
                "artifacts",
                "lexlean_manifest_sha256",
                "model_sha256",
                "phase",
                "processes",
                "role",
            ],
        )?;
        if phase["phase"] != ["controller-build", "replay"][index]
            || phase["role"] != role
            || phase["model_sha256"] != model_sha256
            || !hash(&phase["model_sha256"])
            || !hash(&phase["lexlean_manifest_sha256"])
        {
            return Err(failure("exporter owner phase or input binding differs"));
        }
        let artifacts = phase["artifacts"]
            .as_array()
            .ok_or_else(|| failure("exporter owner artifacts required"))?;
        if artifacts.is_empty() {
            return Err(failure("exporter owner artifact closure is empty"));
        }
        let mut previous: Option<&str> = None;
        for row in artifacts {
            keys(row, &["byte_length", "path", "sha256"])?;
            let path = row["path"]
                .as_str()
                .ok_or_else(|| failure("exporter artifact path required"))?;
            if path.is_empty()
                || path.contains('\\')
                || path.starts_with('/')
                || path.split('/').any(|part| matches!(part, "" | "." | ".."))
                || previous.is_some_and(|prior| prior >= path)
                || row["byte_length"].as_u64().is_none()
                || !hash(&row["sha256"])
            {
                return Err(failure("exporter owner artifact binding differs"));
            }
            previous = Some(path);
        }
        let processes = crate::release_verification::process_records(&phase["processes"], false)?;
        if processes.len() != tools.len()
            || processes
                .iter()
                .zip(&tools)
                .any(|(row, tool)| row["tool"] != *tool)
            || processes[0]["argv"] != json!(["build", "PrismGenerated"])
            || processes[1]["argv"] != json!(["build", "prod-export"])
            || processes[..3]
                .iter()
                .any(|row| row["executable_sha256"] != processes[0]["executable_sha256"])
        {
            return Err(failure("exporter owner actual process closure differs"));
        }
        if role == "library"
            && (processes[3]["argv"]
                != json!(["package", "--locked", "--offline", "--allow-dirty"])
                || processes[3]["executable_sha256"]
                    != phases[0]["processes"][3]["executable_sha256"])
        {
            return Err(failure("exporter owner original package process differs"));
        }
        if let Some(prior) = executable {
            if prior != &processes[2]["exporter"] {
                return Err(failure(
                    "exporter owner source, executable or acquisition changed",
                ));
            }
        }
        executable = Some(&processes[2]["exporter"]);
    }
    for field in [
        "role",
        "model_sha256",
        "lexlean_manifest_sha256",
        "artifacts",
    ] {
        if phases[0][field] != phases[1][field] {
            return Err(failure("exporter owner complete phase bindings differ"));
        }
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    // Unaccepted custody fixture only. No compiler execution or production
    // acceptance is claimed; its empty transcript must never pass finish().
    struct Unaccepted(Vec<(String, Vec<u8>)>);
    impl ProductEvidence for Unaccepted {
        fn role(&self) -> &'static str {
            "library"
        }
        fn artifacts(&self) -> &[(String, Vec<u8>)] {
            &self.0
        }
        fn processes(&self) -> &[ProcessRecord] {
            &[]
        }
    }
    fn provisional(path: &Path, _: &Value) -> Result<Unaccepted, PrismError> {
        std::fs::create_dir_all(path.join(".lake/build/bin")).unwrap();
        let executable = path.join(".lake/build/bin/prod-export");
        std::fs::write(&executable, b"unaccepted custody fixture").unwrap();
        std::fs::set_permissions(executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(
            path.join(".lake/build/compiled.olean"),
            b"fixed compiled member",
        )
        .unwrap();
        std::fs::write(path.join(".lake/build/compiler.trace"), b"initial trace").unwrap();
        Ok(Unaccepted(vec![(
            "library/unaccepted".to_owned(),
            b"fixture".to_vec(),
        )]))
    }
    fn first(project: &Path) -> VerifyExporterOwner {
        let mut owner = VerifyExporterOwner::default();
        owner
            .generate(
                project,
                Phase::ControllerBuild,
                b"model",
                b"manifest",
                provisional,
            )
            .unwrap();
        owner
    }
    fn unused(_: &Path, _: &Value) -> Result<Unaccepted, PrismError> {
        panic!("forbidden generation invoked")
    }

    #[test]
    fn invalid_phase_order_never_acquires_and_repeated_phase_retires() {
        let project = tempfile::tempdir().unwrap();
        let mut empty = VerifyExporterOwner::default();
        assert!(empty
            .generate(project.path(), Phase::Replay, b"model", b"manifest", unused)
            .is_err());
        assert!(empty.package.is_none());
        let mut owner = first(project.path());
        let path = owner.package.as_ref().unwrap().directory.path().to_owned();
        assert!(owner
            .generate(
                project.path(),
                Phase::ControllerBuild,
                b"model",
                b"manifest",
                unused
            )
            .is_err());
        assert!(!path.exists());
        assert!(owner.poisoned);
        assert!(owner.finish().is_err());
    }

    #[test]
    fn generation_failure_in_either_phase_retires_and_cannot_retry() {
        let project = tempfile::tempdir().unwrap();
        for phase in [Phase::ControllerBuild, Phase::Replay] {
            let mut owner = if phase == Phase::Replay {
                first(project.path())
            } else {
                VerifyExporterOwner::default()
            };
            let mut path = PathBuf::new();
            let result = owner.generate::<Unaccepted>(
                project.path(),
                phase,
                b"model",
                b"manifest",
                |package, _| {
                    path = package.parent().unwrap().to_owned();
                    Err(PrismError::new("PP5004", "original generation failed"))
                },
            );
            assert_eq!(result.err().unwrap().message, "original generation failed");
            assert!(!path.exists());
            assert!(owner.package.is_none());
            assert!(owner
                .generate(project.path(), phase, b"model", b"manifest", unused)
                .is_err());
        }
    }

    #[test]
    fn source_executable_and_compiled_member_drift_refuse_before_replay() {
        let project = tempfile::tempdir().unwrap();
        for member in [
            "Prod/Export.lean",
            ".lake/build/bin/prod-export",
            ".lake/build/compiled.olean",
            ".lake/build/compiler.trace",
        ] {
            let mut owner = first(project.path());
            let package = owner.package.as_ref().unwrap();
            let path = package.directory.path().to_owned();
            let selected = package.path.join(member);
            let bytes = std::fs::read(&selected).unwrap();
            let replacement = selected.with_extension("substitution");
            std::fs::write(&replacement, bytes).unwrap();
            std::fs::set_permissions(
                &replacement,
                std::fs::metadata(&selected).unwrap().permissions(),
            )
            .unwrap();
            std::fs::rename(replacement, selected).unwrap();
            assert!(owner
                .generate(project.path(), Phase::Replay, b"model", b"manifest", unused)
                .is_err());
            assert!(!path.exists());
            assert!(owner.finish().is_err());
        }
    }

    #[test]
    fn replay_cannot_change_compiled_members_or_inputs() {
        let project = tempfile::tempdir().unwrap();
        for defect in [
            "compiled",
            "source",
            "executable-mode",
            "model",
            "manifest",
            "artifacts",
        ] {
            let mut owner = first(project.path());
            let path = owner.package.as_ref().unwrap().directory.path().to_owned();
            let model = if defect == "model" {
                b"other".as_slice()
            } else {
                b"model".as_slice()
            };
            let manifest = if defect == "manifest" {
                b"other".as_slice()
            } else {
                b"manifest".as_slice()
            };
            let result = owner.generate(
                project.path(),
                Phase::Replay,
                model,
                manifest,
                |package, _| {
                    match defect {
                        "compiled" => {
                            std::fs::write(package.join(".lake/build/compiled.olean"), b"changed")
                                .unwrap()
                        }
                        "source" => {
                            std::fs::write(package.join("Prod/Export.lean"), b"changed").unwrap()
                        }
                        "executable-mode" => std::fs::set_permissions(
                            package.join(".lake/build/bin/prod-export"),
                            std::fs::Permissions::from_mode(0o700),
                        )
                        .unwrap(),
                        _ => {}
                    }
                    Ok(Unaccepted(vec![(
                        "library/unaccepted".to_owned(),
                        if defect == "artifacts" {
                            b"other".to_vec()
                        } else {
                            b"fixture".to_vec()
                        },
                    )]))
                },
            );
            assert!(result.is_err(), "accepted defect {defect}");
            assert!(!path.exists());
            assert!(owner.finish().is_err());
        }
    }

    #[test]
    fn only_trace_refresh_survives_and_unexecuted_transcript_never_finishes() {
        let project = tempfile::tempdir().unwrap();
        let mut owner = first(project.path());
        let path = owner.package.as_ref().unwrap().directory.path().to_owned();
        let original = owner.package.as_ref().unwrap().path.clone();
        owner
            .generate(
                project.path(),
                Phase::Replay,
                b"model",
                b"manifest",
                |package, _| {
                    assert_eq!(package, original);
                    std::fs::write(
                        package.join(".lake/build/compiler.trace"),
                        b"legitimate refreshed trace",
                    )
                    .unwrap();
                    Ok(Unaccepted(vec![(
                        "library/unaccepted".to_owned(),
                        b"fixture".to_vec(),
                    )]))
                },
            )
            .unwrap();
        assert_eq!(owner.phases.len(), 2);
        assert!(owner.finish().is_err());
        assert!(!path.exists());
    }

    #[test]
    fn lock_addition_or_project_substitution_retires_cold_owner() {
        let project = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        for add_lock in [false, true] {
            let mut owner = first(project.path());
            let path = owner.package.as_ref().unwrap().directory.path().to_owned();
            if add_lock {
                std::fs::write(
                    project.path().join("prismpm.lock"),
                    b"invalid new authority",
                )
                .unwrap();
            }
            assert!(owner
                .generate(
                    if add_lock {
                        project.path()
                    } else {
                        other.path()
                    },
                    Phase::Replay,
                    b"model",
                    b"manifest",
                    unused
                )
                .is_err());
            assert!(!path.exists());
            assert!(owner.finish().is_err());
        }
    }

    #[test]
    fn unwind_poison_cannot_accept_and_drop_retires_owned_package() {
        let project = tempfile::tempdir().unwrap();
        let mut owner = VerifyExporterOwner::default();
        let mut path = PathBuf::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = owner.generate::<Unaccepted>(
                project.path(),
                Phase::ControllerBuild,
                b"model",
                b"manifest",
                |package, _| {
                    path = package.parent().unwrap().to_owned();
                    panic!("generation unwind")
                },
            );
        }));
        assert!(result.is_err());
        assert!(owner.poisoned);
        assert!(owner.finish().is_err());
        assert!(!path.exists());
    }

    #[test]
    fn concurrent_invocations_never_share_packages() {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let workers = (0..2)
            .map(|_| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let project = tempfile::tempdir().unwrap();
                    let owner = first(project.path());
                    let path = owner.package.as_ref().unwrap().directory.path().to_owned();
                    barrier.wait();
                    assert!(path.exists());
                    drop(owner);
                    assert!(!path.exists());
                    path
                })
            })
            .collect::<Vec<_>>();
        let paths = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        assert_ne!(paths[0], paths[1]);
    }

    #[test]
    fn completed_phase_pair_cannot_run_a_third_generation() {
        let project = tempfile::tempdir().unwrap();
        let mut owner = first(project.path());
        let path = owner.package.as_ref().unwrap().directory.path().to_owned();
        owner
            .generate(
                project.path(),
                Phase::Replay,
                b"model",
                b"manifest",
                |_, _| {
                    Ok(Unaccepted(vec![(
                        "library/unaccepted".to_owned(),
                        b"fixture".to_vec(),
                    )]))
                },
            )
            .unwrap();
        assert!(owner
            .generate(project.path(), Phase::Replay, b"model", b"manifest", unused)
            .is_err());
        assert!(!path.exists());
        assert!(owner.finish().is_err());
    }

    #[test]
    fn cleanup_error_is_not_a_successful_retirement() {
        let project = tempfile::tempdir().unwrap();
        let mut owner = VerifyExporterOwner::default();
        let mut path = PathBuf::new();
        let error = owner
            .generate::<Unaccepted>(
                project.path(),
                Phase::ControllerBuild,
                b"model",
                b"manifest",
                |package, _| {
                    path = package.parent().unwrap().to_owned();
                    assert!(path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("prismpm-verify-exporter-"));
                    std::fs::remove_dir_all(&path).unwrap();
                    std::fs::write(&path, b"owned cleanup-failure sentinel").unwrap();
                    Err(PrismError::new(
                        "PP5004",
                        "generation failed before cleanup",
                    ))
                },
            )
            .err()
            .unwrap();
        assert_eq!(error.code, "PP5008");
        assert!(error.message.contains("retirement"));
        assert!(owner.finish().is_err());
        assert!(std::fs::symlink_metadata(&path).unwrap().is_file());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn retirement_rechecks_owned_custody_without_synthetic_execution_acceptance() {
        // Exercise the exact final retirement guard in isolation. The fixture
        // never executes a compiler, and no acceptance record is returned.
        let project = tempfile::tempdir().unwrap();
        for member in [
            "Prod/Export.lean",
            ".lake/build/compiled.olean",
            ".lake/build/compiler.trace",
        ] {
            let mut owner = first(project.path());
            let package = owner.package.take().unwrap();
            let path = package.directory.path().to_owned();
            std::fs::write(package.path.join(member), b"retirement custody drift").unwrap();
            let error = package.retire().unwrap_err();
            assert_eq!(error.code, "PP5008");
            assert_eq!(
                error.message,
                if member == "Prod/Export.lean" {
                    "exporter package source or build-tree custody changed"
                } else {
                    "private exporter custody changed before retirement"
                }
            );
            assert!(!path.exists());
        }
    }
}

/// Bind both original generations to independently retained source, products
/// and native preflight; replay must also be the original top-level transcript.
#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_binding(
    value: &Value,
    model_sha256: &str,
    role: &str,
    lex_manifest: &[u8],
    artifacts: &Value,
    processes: &[Value],
    arguments: &[String],
) -> Result<(), PrismError> {
    validate_record(value, model_sha256, role)?;
    let phases = value["phases"].as_array().expect("validated owner phases");
    let replay = phases[1]["processes"]
        .as_array()
        .expect("validated processes");
    if processes
        .windows(replay.len())
        .filter(|rows| *rows == replay.as_slice())
        .count()
        != 1
    {
        return Err(failure(
            "original replay processes differ from retained transcript",
        ));
    }
    let lake = processes
        .iter()
        .find(|row| row["tool"] == "lake-version")
        .ok_or_else(|| failure("independent native Lake preflight is absent"))?;
    for phase in phases {
        if phase["lexlean_manifest_sha256"] != digest(lex_manifest)
            || phase["artifacts"] != *artifacts
            || phase["processes"][2]["argv"] != json!(arguments)
            || phase["processes"][0]["executable_sha256"] != lake["executable_sha256"]
        {
            return Err(failure(
                "exporter owner source, products, arguments or native tool differ",
            ));
        }
    }
    Ok(())
}
