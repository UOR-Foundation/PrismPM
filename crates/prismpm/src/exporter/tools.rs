//! Private source-built compiler inputs; not SDK authority or model evidence.
use super::{custody, directory};
use crate::verification::{run_process, ProcessRecord};
use crate::PrismError;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use super::ARCHIVE;

pub(crate) fn eligible() -> bool {
    cfg!(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))
}

fn failure(message: &str) -> PrismError {
    PrismError::new("PP5008", message)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

struct Package {
    namespace: Option<directory::Directory>,
    directory: directory::Directory,
    path: PathBuf,
    snapshot: custody::Snapshot,
    seed: Option<Arc<ToolSeed>>,
}

impl Package {
    fn cold() -> Result<Self, PrismError> {
        custody::validate_archive()?;
        let mut directory =
            directory::Directory::temporary("prismpm-library-exporter-", None, Default::default())?;
        let path = directory.path().join("lean4-prod");
        std::fs::create_dir(&path).map_err(|error| failure(&format!("fresh exporter: {error}")))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))
                .map_err(|error| failure(&format!("private exporter: {error}")))?;
        }
        tar::Archive::new(std::io::Cursor::new(ARCHIVE))
            .unpack(&path)
            .map_err(|error| failure(&format!("pinned exporter source: {error}")))?;
        let snapshot = custody::capture_source(&path)?;
        directory.bind_package(&path, &snapshot)?;
        let namespace = directory::Directory::existing(&path, directory.scope())?;
        Ok(Self {
            namespace: Some(namespace),
            directory,
            path,
            snapshot,
            seed: None,
        })
    }

    fn inputs(&self) -> Result<(), PrismError> {
        if let Some(seed) = &self.seed {
            seed.verify()?;
        }
        Ok(())
    }

    fn verify(&self) -> Result<(), PrismError> {
        self.inputs()?;
        self.directory.ready()?;
        self.namespace
            .as_ref()
            .ok_or_else(|| failure("exporter namespace absent"))?
            .ready()?;
        if custody::capture(&self.path)? != self.snapshot {
            return Err(failure("original cold exporter custody changed"));
        }
        Ok(())
    }

    fn retire(self) -> Result<(), PrismError> {
        self.verify()
            .inspect_err(|_| self.directory.scope().uncertain())?;
        if let Some(seed) = &self.seed {
            seed.verify_sdk()
                .inspect_err(|_| self.directory.scope().uncertain())?;
        }
        drop(self.namespace);
        self.directory.close()
    }

    #[allow(clippy::too_many_arguments)]
    fn run(
        &mut self,
        tool: &str,
        program: &Path,
        args: &[String],
        cwd: &Path,
        environment: &BTreeMap<String, String>,
        replacements: &[(&Path, &str)],
        code: &'static str,
        build: bool,
    ) -> Result<ProcessRecord, PrismError> {
        self.inputs()?;
        self.directory.ready()?;
        self.namespace
            .as_ref()
            .ok_or_else(|| failure("exporter namespace absent"))?
            .ready()?;
        let before = custody::capture_optional(&self.path)?;
        if before != self.snapshot {
            return Err(failure("exporter changed before actual command"));
        }
        let launcher = custody::capture_launcher(program)?;
        let held_cwd = custody::Cwd::open(cwd)?;
        let result = run_process(tool, program, args, cwd, environment, replacements, code);
        let checked: Result<(), PrismError> = (|| {
            held_cwd.verify()?;
            self.inputs()?;
            self.directory.ready()?;
            self.namespace
                .as_ref()
                .ok_or_else(|| failure("exporter namespace absent"))?
                .ready()?;
            if custody::capture_launcher(program)? != launcher {
                return Err(failure("exporter command launcher changed"));
            }
            let after = custody::capture_optional(&self.path)?;
            if !(if build {
                before.same_source(&after)
            } else {
                before == after
            }) {
                return Err(failure("exporter changed during actual command"));
            }
            self.directory.bind_package(&self.path, &after)?;
            self.snapshot = after;
            if result.is_ok() {
                custody::require_executable(&self.path)?;
            }
            Ok(())
        })();
        match (result, checked) {
            (Err(mut original), Err(custody)) => {
                original.message.push_str(&format!(
                    "; exporter custody also failed: {}",
                    custody.message
                ));
                Err(original)
            }
            (Err(original), _) => Err(original),
            (Ok(_), Err(custody)) => Err(custody),
            (Ok(record), Ok(())) => Ok(record),
        }
    }
}

/// A compiler-only input is constructed before the first model-tool command.
/// Every use owns a new native directory and executes all ordinary commands.
/// No caller can deserialize a seed, supply its path, or persist it as evidence.
pub(crate) struct CompilerTools {
    project: Option<custody::ProjectOrigin>,
    seed: OnceLock<Arc<ToolSeed>>,
}

impl std::fmt::Debug for CompilerTools {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("CompilerTools")
            .field("constructed", &self.seed.get().is_some())
            .finish()
    }
}

pub(crate) struct ToolSeed {
    project: Arc<custody::Cwd>,
    archive_path: PathBuf,
    archive_input: Option<custody::FileWitness>,
    archive_sha256: String,
    launchers: Vec<(PathBuf, custody::LauncherWitness)>,
    sdk: Option<serde_json::Value>,
    capsule: custody::Capsule,
    capsule_files: serde_json::Value,
    capsule_sha256: String,
    construction: ProcessRecord,
}

impl ToolSeed {
    pub(super) fn receipt(&self) -> Result<serde_json::Value, PrismError> {
        self.verify()?;
        let (revision, toolchain) = super::compiler_identity()?;
        let files = &self.capsule_files;
        let target = files
            .as_array()
            .and_then(|rows| {
                rows.iter()
                    .find(|r| r["path"] == ".lake/build/bin/prod-export")
            })
            .ok_or_else(|| failure("capsule executable absent"))?;
        let executable = serde_json::json!({"byte_length":target["byte_length"],"mode":target["mode"],"sha256":target["sha256"]});
        Ok(
            serde_json::json!({"schema":"prismpm/exporter-acquisition/1","mode":"controller-capsule",
            "archive_sha256":self.archive_sha256,"compiler_revision":revision,"toolchain":toolchain,
            "platform":format!("linux/{}",match std::env::consts::ARCH {"x86_64"=>"amd64","aarch64"=>"arm64",_=>return Err(failure("capsule platform unavailable"))}),
            "construction":self.construction,"executable_sha256":executable["sha256"],"executable":executable,
            "lean_sha256":self.launchers[1].1.resolved_sha256()}),
        )
    }
    pub(super) fn bind(
        &self,
        receipt: &serde_json::Value,
        executable: &str,
    ) -> Result<(), PrismError> {
        self.verify()?;
        if self.receipt()? != *receipt || receipt["executable_sha256"] != executable {
            return Err(failure(
                "private constructor does not bind this capsule receipt",
            ));
        }
        Ok(())
    }
    pub(super) fn before_build(
        &self,
        project: &Path,
        package: &Path,
        receipt: &serde_json::Value,
        expected: &custody::Snapshot,
    ) -> Result<(), PrismError> {
        self.project.verify_path(project)?;
        self.bind(
            receipt,
            &super::measure_executable(&package.join(".lake/build/bin/prod-export"))?.sha256,
        )?;
        if custody::capture(package)? != *expected {
            return Err(failure("materialized compiler native custody changed"));
        }
        Ok(())
    }

    pub(super) fn verify(&self) -> Result<(), PrismError> {
        self.project.verify()?;
        if hash(ARCHIVE) != self.archive_sha256
            || custody::archive_input(&self.archive_path)? != self.archive_input
        {
            return Err(failure("compiler capsule source archive input changed"));
        }
        for (path, witness) in &self.launchers {
            if custody::capture_launcher(path)? != *witness {
                return Err(failure("compiler capsule launcher input changed"));
            }
        }
        if self.construction.tool != "lean4-prod-build"
            || self.construction.exit_code != 0
            || self.construction.argv != ["build", "prod-export"]
        {
            return Err(failure("compiler capsule construction is incomplete"));
        }
        // The complete physical cache is a private immutable input, not a
        // reproducible model artifact. Never substitute its public projection
        // for complete private capture, inventory, or copy authority.
        custody::validate_capsule_inventory(&self.capsule_files)?;
        if hash(&crate::holo::canonical::encode_value(&self.capsule_files)?) != self.capsule_sha256
        {
            return Err(failure("private compiler cache inventory changed"));
        }
        Ok(())
    }

    pub(super) fn verify_sdk(&self) -> Result<(), PrismError> {
        self.verify()?;
        if crate::sdk::verify_environment()? != self.sdk {
            return Err(failure("compiler capsule SDK input changed"));
        }
        Ok(())
    }
}

impl CompilerTools {
    pub(crate) fn new(project: &Path) -> Result<Self, PrismError> {
        // The existing descriptor-qualified owner is Linux-only. Other hosts
        // keep their original cold construction, without adding a directory
        // open requirement to their public Controller loader.
        let project = eligible()
            .then(|| custody::ProjectOrigin::capture(project))
            .transpose()?;
        Ok(Self {
            project,
            seed: OnceLock::new(),
        })
    }

    fn selected(&self, project: &Path) -> Result<Arc<ToolSeed>, PrismError> {
        let held = self
            .project
            .as_ref()
            .ok_or_else(|| failure("compiler capsule host unavailable"))?;
        held.verify_path(project)?;
        if let Some(seed) = self.seed.get() {
            seed.project.verify_path(project).map_err(|error| {
                error.with_note("compiler tools cached selection: original project")
            })?;
            seed.verify_sdk().map_err(|error| {
                error.with_note(
                    "compiler tools cached selection: original archive, launchers and SDK",
                )
            })?;
            return Ok(Arc::clone(seed));
        }
        let archive_path = project.join("vendor/lean4-prod/lean.tar");
        let archive_input = custody::archive_input(&archive_path)?;
        let project = Arc::new(held.open(project)?);
        let sdk = crate::sdk::verify_environment()?;
        let launchers = ["lake", "lean"]
            .into_iter()
            .map(|name| {
                let path = crate::verification::executable(name)?;
                let witness = custody::capture_launcher(&path)?;
                Ok((path, witness))
            })
            .collect::<Result<Vec<_>, PrismError>>()?;
        let mut package = Package::cold()?;
        let path = package.path.clone();
        let construction = package.run(
            "lean4-prod-build",
            &launchers[0].0,
            &["build".to_owned(), "prod-export".to_owned()],
            &path,
            &BTreeMap::new(),
            &[(&path, "$LEAN4_PROD")],
            "PP5004",
            true,
        )?;
        package.verify()?;
        let capsule = custody::Capsule::capture(&package.path, &package.snapshot)?;
        let capsule_files = capsule.inventory();
        let capsule_sha256 = hash(&crate::holo::canonical::encode_value(&capsule_files)?);
        let seed = Arc::new(ToolSeed {
            project,
            archive_path,
            archive_input,
            archive_sha256: hash(ARCHIVE),
            launchers,
            sdk,
            capsule,
            capsule_files,
            capsule_sha256,
            construction,
        });
        seed.verify_sdk()?;
        // The source-only constructor retires before publishing the input.
        // Concurrent first users may construct independently; no model result
        // or failed/unretired compiler can win this cell.
        package.retire()?;
        seed.verify_sdk()?;
        let _ = self.seed.set(Arc::clone(&seed));
        Ok(seed)
    }

    pub(super) fn admit(
        &self,
        project: &Path,
        package: &Path,
        mut input: super::Acquisition,
    ) -> Result<super::Acquisition, PrismError> {
        if !eligible() || input.receipt != super::cold_acquisition() {
            return Ok(input);
        }
        if custody::capture_source(package)? != input.snapshot {
            return Err(failure(
                "cold archive changed before compiler-input admission",
            ));
        }
        let seed = self.selected(project)?;
        input.snapshot = seed.capsule.materialize(package)?;
        input.receipt = seed.receipt()?;
        seed.before_build(project, package, &input.receipt, &input.snapshot)?;
        input.seed = Some(seed);
        Ok(input)
    }
    #[cfg(test)]
    pub(crate) fn package(&self, project: &Path) -> Result<CompilerPackage, PrismError> {
        let seed = self.selected(project).map_err(|error| {
            error.with_note("compiler test package: select admitted compiler inputs")
        })?;
        let mut package = Package::cold().map_err(|error| {
            error.with_note("compiler test package: fresh original cold source")
        })?;
        seed.verify().map_err(|error| {
            error.with_note("compiler test package: selected compiler input verification")
        })?;
        let snapshot = seed.capsule.materialize(&package.path).map_err(|error| {
            error.with_note("compiler test package: materialize complete compiler inputs")
        })?;
        package
            .directory
            .bind_package(&package.path, &snapshot)
            .map_err(|error| {
                error.with_note("compiler test package: bind actual descendant retirement custody")
            })?;
        package.snapshot = snapshot;
        package.seed = Some(seed);
        package.verify().map_err(|error| {
            error.with_note("compiler test package: complete fresh-package custody")
        })?;
        Ok(CompilerPackage(package))
    }
}

#[cfg(test)]
pub(crate) struct CompilerPackage(Package);

#[cfg(test)]
impl CompilerPackage {
    pub(crate) fn path(&self) -> &Path {
        &self.0.path
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn run(
        &mut self,
        tool: &str,
        program: &Path,
        args: &[String],
        environment: &BTreeMap<String, String>,
        replacements: &[(&Path, &str)],
        code: &'static str,
        build: bool,
    ) -> Result<ProcessRecord, PrismError> {
        let cwd = self.0.path.clone();
        self.0.run(
            tool,
            program,
            args,
            &cwd,
            environment,
            replacements,
            code,
            build,
        )
    }

    pub(crate) fn finish(self) -> Result<(), PrismError> {
        self.0.retire()
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn private_compiler_inputs_reject_a_different_or_replaced_project_before_construction() {
        let original = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let tools = CompilerTools::new(original.path()).unwrap();
        assert!(tools.package(other.path()).is_err());
        assert!(tools.seed.get().is_none());
        let original_path = original.path().join("original");
        std::fs::create_dir(&original_path).unwrap();
        let tools = CompilerTools::new(&original_path).unwrap();
        let moved = original.path().join("moved");
        std::fs::rename(&original_path, &moved).unwrap();
        std::fs::create_dir(&original_path).unwrap();
        assert!(tools.package(&original_path).is_err());
        assert!(tools.seed.get().is_none());
    }

    #[test]
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    fn actual_retired_compiler_seed_gives_two_fresh_native_packages_real_warm_builds() {
        let project = tempfile::tempdir().unwrap();
        let project_path = project.path().join("owned");
        std::fs::create_dir(&project_path).unwrap();
        let tools = CompilerTools::new(&project_path).unwrap();
        let started = std::time::Instant::now();
        let mut first = tools.package(&project_path).unwrap();
        let construction_elapsed = started.elapsed();
        let seed = Arc::clone(
            tools
                .seed
                .get()
                .expect("only a successfully retired constructor publishes"),
        );
        assert_eq!(seed.construction.tool, "lean4-prod-build");
        assert_eq!(seed.construction.argv, ["build", "prod-export"]);
        assert_eq!(seed.construction.exit_code, 0);
        let space = rustix::fs::statvfs(first.path()).unwrap();
        eprintln!(
            "compiler-tool-before-second-copy {}",
            serde_json::json!({
                "scope":"actual bounded diagnostics only; no execution authority",
                "tmpfs_block_size":space.f_frsize,
                "tmpfs_available_blocks":space.f_bavail,
                "constructor":seed.capsule.origin_witness(),
                "first_materialization":first.0.snapshot.witness(),
            })
        );
        let mut second = tools.package(&project_path).unwrap();
        assert!(Arc::ptr_eq(first.0.seed.as_ref().unwrap(), &seed));
        assert!(Arc::ptr_eq(second.0.seed.as_ref().unwrap(), &seed));
        assert_ne!(first.path(), second.path());
        let first_before = first.0.snapshot.clone();
        let second_before = second.0.snapshot.clone();
        assert_ne!(
            first_before, second_before,
            "copies must not share native identity"
        );
        let receipt = seed.receipt().unwrap();
        let target = receipt["executable_sha256"].as_str().unwrap();
        super::super::validate_acquisition(&receipt, target).unwrap();
        seed.bind(&receipt, target).unwrap();
        for (pointer, replacement) in [
            ("/mode", serde_json::json!("sdk-seed")),
            (
                "/construction/tool",
                serde_json::json!("not-the-constructor"),
            ),
            (
                "/construction/argv",
                serde_json::json!(["build", "PrismGenerated"]),
            ),
            ("/construction/exit_code", serde_json::json!(1)),
            ("/archive_sha256", serde_json::json!("0".repeat(64))),
            ("/executable_sha256", serde_json::json!("1".repeat(64))),
        ] {
            let mut changed = receipt.clone();
            *changed.pointer_mut(pointer).unwrap() = replacement;
            assert!(
                seed.bind(&changed, target).is_err(),
                "private witness accepted {pointer}"
            );
            assert!(
                super::super::validate_acquisition(&changed, target).is_err(),
                "closed structure accepted {pointer}"
            );
        }
        // Full cache bytes belong to private immutable custody, not stable
        // public compiler identity. Retain every original inventory refusal
        // at the private validator rather than serialize volatile cache bits.
        let inventory = seed.capsule_files.clone();
        custody::validate_capsule_inventory(&inventory).unwrap();
        let mut reordered_files = inventory.clone();
        reordered_files.as_array_mut().unwrap().swap(0, 1);
        let reordered =
            String::from_utf8(crate::holo::canonical::encode_value(&reordered_files).unwrap())
                .unwrap();
        assert!(custody::validate_capsule_inventory(&reordered_files).is_err());
        // These are controls of the existing generic canonical codec, not a
        // private cache importer: no production API accepts cache JSON.
        let decode_inventory = |document: &str| {
            let files = crate::holo::canonical::decode_value(
                document.as_bytes(),
                "compiler inventory diagnostic",
            )?;
            custody::validate_capsule_inventory(&files)
        };
        assert!(decode_inventory(&reordered).is_err());
        for document in [
            format!(
                "{}\n",
                String::from_utf8(crate::holo::canonical::encode_value(&inventory).unwrap())
                    .unwrap()
            ),
            " ".repeat(16 * 1024 * 1024 + 1),
            "[]".to_owned(),
            "[{\"kind\":\"file\",\"kind\":\"directory\"}]".to_owned(),
            "[[[]]]".to_owned(),
            "[\"\\ud800\"]".to_owned(),
        ] {
            assert!(
                decode_inventory(&document).is_err(),
                "noncanonical, duplicate, overbound or malformed nested inventory"
            );
        }
        for field in ["files_document", "capsule_sha256"] {
            let mut bad = receipt.clone();
            bad[field] = serde_json::json!("private data cannot become public authority");
            assert!(super::super::validate_acquisition(&bad, target).is_err());
        }
        // Private decoder boundaries only. Invented JSON rows still cannot
        // mint the real private producer/native witness used for execution.
        let mut boundary_files = inventory.as_array().unwrap().clone();
        let existing = boundary_files
            .iter()
            .filter(|row| {
                row["path"]
                    .as_str()
                    .is_some_and(|p| p == ".lake" || p.starts_with(".lake/"))
            })
            .count();
        let empty_sha = hash(b"");
        for n in existing..4096 {
            boundary_files.push(serde_json::json!({"path":format!(".lake/capsule-boundary-{n:04}"),"kind":"file", "mode":0o444,"byte_length":0,"sha256":empty_sha}));
        }
        boundary_files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        let boundary_bytes =
            crate::holo::canonical::encode_value(&serde_json::json!(boundary_files)).unwrap();
        custody::validate_capsule_inventory(&serde_json::json!(boundary_files)).unwrap();
        decode_inventory(std::str::from_utf8(&boundary_bytes).unwrap()).unwrap();
        assert_eq!(
            boundary_files.len(),
            4167,
            "70 pinned source rows plus root plus 4096 build rows"
        );
        boundary_files.push(serde_json::json!({"path":".lake/capsule-boundary-excess","kind":"file","mode":0o444,"byte_length":0,"sha256":empty_sha}));
        boundary_files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
        let excess =
            crate::holo::canonical::encode_value(&serde_json::json!(boundary_files)).unwrap();
        assert!(
            custody::validate_capsule_inventory(&serde_json::json!(boundary_files)).is_err(),
            "4097 build entries must refuse"
        );
        assert!(decode_inventory(std::str::from_utf8(&excess).unwrap()).is_err());
        let mut extra = receipt.clone();
        extra["caller_selected_authority"] = serde_json::json!(true);
        assert!(super::super::validate_acquisition(&extra, target).is_err());
        // Authentic bytes/JSON are still insufficient without the live input.
        let fake = super::super::run_export_with_custody(
            "prod-export",
            Path::new("/not-launched"),
            &["exe".to_owned(), "prod-export".to_owned()],
            first.path(),
            &BTreeMap::new(),
            &[],
            "PP5004",
            &receipt,
            &first_before,
            None,
        )
        .unwrap_err();
        assert!(fake.message.contains("live private compiler input"));
        let mut records = Vec::new();
        let mut elapsed = Vec::new();
        let mut snapshots = Vec::new();
        for package in [&mut first, &mut second] {
            let path = package.path().to_owned();
            let before = custody::capture(&path).unwrap();
            let start = std::time::Instant::now();
            records.push(
                package
                    .run(
                        "lean4-prod-build",
                        &seed.launchers[0].0,
                        &["build".to_owned(), "prod-export".to_owned()],
                        &BTreeMap::new(),
                        &[(&path, "$LEAN4_PROD")],
                        "PP5004",
                        true,
                    )
                    .unwrap(),
            );
            elapsed.push(start.elapsed());
            let after = custody::capture(&path).unwrap();
            snapshots.push(serde_json::json!({"before":before.witness(),"after":after.witness()}));
            assert!(
                before.same_compiler(&after),
                "a genuine hit cannot replace a compiled member or executable"
            );
        }
        assert_eq!(records[0].stdout, records[1].stdout);
        assert_eq!(records[0].stderr, records[1].stderr);
        assert_eq!(records[0].argv, records[1].argv);
        assert_eq!(records[0].executable_sha256, records[1].executable_sha256);
        assert!(records.iter().all(|record| record.exit_code == 0));
        let first_path = first.path().parent().unwrap().to_owned();
        let second_path = second.path().parent().unwrap().to_owned();
        first.finish().unwrap();
        second.finish().unwrap();
        assert!(!first_path.exists() && !second_path.exists());
        assert!(Arc::ptr_eq(tools.seed.get().unwrap(), &seed));
        eprintln!(
            "compiler-tool-capsule-measurement {}",
            serde_json::json!({
                "scope":"actual compiler-only constructor and two fresh native package builds; no model acceptance",
                "stream_policy":"genuine normalized ProcessRecords under unchanged run_process policy, not raw OS streams",
                "construction":seed.construction,
                "construction_origin":seed.capsule.origin_witness(),
                "construction_seconds":construction_elapsed.as_secs_f64(),
                "warm_processes":records,
                "warm_seconds":elapsed.iter().map(std::time::Duration::as_secs_f64).collect::<Vec<_>>(),
                "native_snapshots":snapshots,
                "retired_package_paths":[first_path,second_path]
            })
        );
        // The project archive is an input, not a caller-selected cache. Even
        // a newly appearing byte-identical archive changes the admitted input.
        let archive = project_path.join("vendor/lean4-prod/lean.tar");
        std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
        std::fs::write(&archive, ARCHIVE).unwrap();
        assert!(tools.package(&project_path).is_err());
        assert!(Arc::ptr_eq(tools.seed.get().unwrap(), &seed));
        std::fs::remove_file(&archive).unwrap();
        let moved = project.path().join("moved");
        std::fs::rename(&project_path, &moved).unwrap();
        std::fs::create_dir(&project_path).unwrap();
        assert!(tools.package(&project_path).is_err());
        assert!(Arc::ptr_eq(tools.seed.get().unwrap(), &seed));
        std::fs::remove_dir(&project_path).unwrap();
        std::fs::rename(&moved, &project_path).unwrap();
        actual_concurrent_first_users_and_cached_archive_drift();
    }

    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    fn actual_concurrent_first_users_and_cached_archive_drift() {
        let project = tempfile::tempdir().unwrap();
        let archive = project.path().join("vendor/lean4-prod/lean.tar");
        std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
        std::fs::write(&archive, ARCHIVE).unwrap();
        let tools = Arc::new(CompilerTools::new(project.path()).unwrap());
        let ready = Arc::new(std::sync::Barrier::new(2));
        // Both actual first callers create, use and retire their !Send package
        // entirely inside their worker. No test hook forces constructor order.
        let results = std::thread::scope(|scope| {
            let workers =
                (0..2)
                    .map(|_| {
                        let tools = Arc::clone(&tools);
                        let ready = Arc::clone(&ready);
                        let project = project.path();
                        scope.spawn(move || {
                    assert!(tools.seed.get().is_none());
                    ready.wait();
                    let started = std::time::Instant::now();
                    let mut package = tools.package(project).unwrap();
                    let seed = Arc::clone(package.0.seed.as_ref().unwrap());
                    let path = package.path().to_owned();
                    let before = custody::capture(&path).unwrap();
                    let record = package.run("lean4-prod-build", &seed.launchers[0].0,
                        &["build".to_owned(), "prod-export".to_owned()], &BTreeMap::new(),
                        &[(&path, "$LEAN4_PROD")], "PP5004", true).unwrap();
                    let after = custody::capture(&path).unwrap();
                    assert!(before.same_compiler(&after));
                    let owner = path.parent().unwrap().to_owned();
                    package.finish().unwrap();
                    assert!(!owner.exists());
                    let observation = serde_json::json!({
                        "construction":seed.construction,
                        "construction_origin":seed.capsule.origin_witness(),
                        "warm_process":record,
                        "native_before":before.witness(),"native_after":after.witness(),
                        "retired_package_path":owner,"seconds":started.elapsed().as_secs_f64()
                    });
                    (seed, observation)
                })
                    })
                    .collect::<Vec<_>>();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect::<Vec<_>>()
        });
        let selected = tools.seed.get().unwrap();
        assert!(results.iter().any(|(seed, _)| Arc::ptr_eq(selected, seed)));
        // Equal private winners do not prove independent construction. If the
        // genuine race selected one seed before the second acquisition, build
        // one independent compiler input rather than guessing portability.
        let independently_constructed = if Arc::ptr_eq(&results[0].0, &results[1].0) {
            let independent_tools = CompilerTools::new(project.path()).unwrap();
            let package = independent_tools.package(project.path()).unwrap();
            let input = Arc::clone(package.0.seed.as_ref().unwrap());
            let owner = package.path().parent().unwrap().to_owned();
            package.finish().unwrap();
            assert!(!owner.exists());
            input
        } else {
            Arc::clone(&results[1].0)
        };
        assert!(!Arc::ptr_eq(&results[0].0, &independently_constructed));
        let first_receipt = results[0].0.receipt().unwrap();
        let independent_receipt = independently_constructed.receipt().unwrap();
        let first_bytes = crate::holo::canonical::encode_value(&first_receipt).unwrap();
        let independent_bytes = crate::holo::canonical::encode_value(&independent_receipt).unwrap();
        // Retain both complete private inventories without claiming that
        // independently built Lake caches are model artifacts. The complete
        // public construction receipt itself must still be byte-deterministic.
        for input in [&results[0].0, &independently_constructed] {
            custody::validate_capsule_inventory(&input.capsule_files).unwrap();
            assert_eq!(
                hash(&crate::holo::canonical::encode_value(&input.capsule_files).unwrap()),
                input.capsule_sha256
            );
        }
        eprintln!(
            "compiler-tool-independent-public-identity {}",
            serde_json::json!({
                "scope":"two genuine independent source-only compiler constructors; not model acceptance",
                "first_receipt":first_receipt,"independent_receipt":independent_receipt,
                "first_canonical_receipt_sha256":hash(&first_bytes),
                "independent_canonical_receipt_sha256":hash(&independent_bytes),
                "first_private_inventory":results[0].0.capsule_files,
                "first_private_inventory_sha256":results[0].0.capsule_sha256,
                "independent_private_inventory":independently_constructed.capsule_files,
                "independent_private_inventory_sha256":independently_constructed.capsule_sha256,
                "first_private_origin":results[0].0.capsule.origin_witness(),
                "independent_private_origin":independently_constructed.capsule.origin_witness()
            })
        );
        assert_eq!(
            first_bytes, independent_bytes,
            "actual normalized public constructor provenance is not deterministic"
        );
        assert_eq!(
            results[0].1["warm_process"]["stdout"],
            results[1].1["warm_process"]["stdout"]
        );
        assert_eq!(
            results[0].1["warm_process"]["stderr"],
            results[1].1["warm_process"]["stderr"]
        );
        assert_ne!(results[0].1["native_before"], results[1].1["native_before"]);
        eprintln!(
            "compiler-tool-capsule-concurrent-first-users {}",
            serde_json::json!({
                "scope":"two actual first callers; compiler-only, not parallel model acceptance",
                "stream_policy":"genuine normalized ProcessRecords, not raw OS streams",
                "distinct_successful_constructor_inputs":!Arc::ptr_eq(&results[0].0, &results[1].0),
                "observations":results.iter().map(|(_, observation)| observation).collect::<Vec<_>>()
            })
        );
        // Change only this test's genuine private archived source input.
        // Immutable successful provenance must not adopt changed bytes, an
        // identical rewrite with changed native metadata, or replacement.
        let mut changed = ARCHIVE.to_vec();
        changed[0] ^= 1;
        std::fs::write(&archive, &changed).unwrap();
        assert!(tools.package(project.path()).is_err());
        std::fs::write(&archive, ARCHIVE).unwrap();
        assert!(tools.package(project.path()).is_err());
        std::fs::remove_file(&archive).unwrap();
        std::fs::write(&archive, ARCHIVE).unwrap();
        assert!(tools.package(project.path()).is_err());
        assert!(Arc::ptr_eq(tools.seed.get().unwrap(), selected));
    }
}
