//! One fresh acquisition boundary for the pinned Lean exporter package.

use crate::error::PrismError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read};
use std::path::Path;

const ARCHIVE: &[u8] = include_bytes!("../vendor/lean4-prod/lean.tar");

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ExecutableMeasurement {
    pub(crate) byte_length: u64,
    pub(crate) mode: u32,
    pub(crate) sha256: String,
}

/// Measure the actual child executable, not the Lake launcher. Reject aliases,
/// hard links, unbounded inputs and concurrent replacement before execution.
pub(crate) fn measure_executable(path: &Path) -> Result<ExecutableMeasurement, PrismError> {
    let fail = || {
        PrismError::new(
            "PP5008",
            "exporter executable is missing, aliased, oversized, non-executable, or changed",
        )
    };
    if path.canonicalize().map_err(|_| fail())? != path {
        return Err(fail());
    }
    let before = std::fs::symlink_metadata(path).map_err(|_| fail())?;
    if !before.is_file() || before.len() == 0 || before.len() > 256 * 1024 * 1024 {
        return Err(fail());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        if before.nlink() != 1 || before.mode() & 0o111 == 0 || before.mode() & 0o7000 != 0 {
            return Err(fail());
        }
        options.custom_flags(
            (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
        );
    }
    let mut file = options.open(path).map_err(|_| fail())?;
    let same = |metadata: &std::fs::Metadata| {
        let common = metadata.is_file()
            && metadata.len() == before.len()
            && metadata.modified().ok() == before.modified().ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            common
                && metadata.dev() == before.dev()
                && metadata.ino() == before.ino()
                && metadata.nlink() == 1
                && metadata.mode() == before.mode()
                && metadata.ctime() == before.ctime()
                && metadata.ctime_nsec() == before.ctime_nsec()
        }
        #[cfg(not(unix))]
        {
            common
        }
    };
    if !same(&file.metadata().map_err(|_| fail())?) {
        return Err(fail());
    }
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut length = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|_| fail())?;
        if count == 0 {
            break;
        }
        length += count as u64;
        if length > before.len() {
            return Err(fail());
        }
        hash.update(&buffer[..count]);
    }
    if length != before.len()
        || !same(&file.metadata().map_err(|_| fail())?)
        || !same(&std::fs::symlink_metadata(path).map_err(|_| fail())?)
        || path.canonicalize().map_err(|_| fail())? != path
    {
        return Err(fail());
    }
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::MetadataExt;
        before.mode() & 0o777
    };
    #[cfg(not(unix))]
    let mode = 0;
    Ok(ExecutableMeasurement {
        byte_length: length,
        mode,
        sha256: format!("{:x}", hash.finalize()),
    })
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExporterExecution {
    pub(crate) schema: &'static str,
    pub(crate) source_archive_sha256: String,
    pub(crate) executable: ExecutableMeasurement,
}

/// Every invocation still goes through actual `lake exe prod-export` after the
/// ordinary build. Measurement refuses an executable that changed across it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_export(
    tool: &str,
    program: &Path,
    args: &[String],
    cwd: &Path,
    environment: &std::collections::BTreeMap<String, String>,
    replacements: &[(&Path, &str)],
    failure_code: &'static str,
) -> Result<crate::verification::ProcessRecord, PrismError> {
    if args.first().map(String::as_str) != Some("exe")
        || args.get(1).map(String::as_str) != Some("prod-export")
    {
        return Err(PrismError::new(
            "PP9001",
            "exporter invocation must use the actual Lake executable path",
        ));
    }
    let child = cwd.join(".lake/build/bin/prod-export");
    let identity_before = std::fs::symlink_metadata(&child)
        .map_err(|error| PrismError::new("PP5008", error.to_string()))?;
    let before = measure_executable(&child)?;
    let mut record = crate::verification::run_process(
        tool,
        program,
        args,
        cwd,
        environment,
        replacements,
        failure_code,
    )?;
    if measure_executable(&child)? != before {
        return Err(PrismError::new(
            "PP5008",
            "exporter executable changed during execution",
        ));
    }
    let identity_after = std::fs::symlink_metadata(&child)
        .map_err(|error| PrismError::new("PP5008", error.to_string()))?;
    let same_identity = identity_before.modified().ok() == identity_after.modified().ok();
    #[cfg(unix)]
    let same_identity = {
        use std::os::unix::fs::MetadataExt;
        same_identity
            && identity_before.dev() == identity_after.dev()
            && identity_before.ino() == identity_after.ino()
            && identity_before.ctime() == identity_after.ctime()
            && identity_before.ctime_nsec() == identity_after.ctime_nsec()
    };
    if !same_identity {
        return Err(PrismError::new(
            "PP5008",
            "exporter executable identity changed during execution",
        ));
    }
    record.exporter = Some(ExporterExecution {
        schema: "prismpm/exporter-execution/1",
        source_archive_sha256: format!("{:x}", Sha256::digest(ARCHIVE)),
        executable: before,
    });
    Ok(record)
}

/// Materialize the compiled-in source in a new, caller-owned private directory.
/// This is deliberately a cold acquisition: no caller-selected Lake cache is
/// consulted, and callers must still execute the actual exporter build.
pub(crate) fn acquire(destination: &Path) -> Result<(), PrismError> {
    std::fs::create_dir(destination)
        .map_err(|error| PrismError::new("PP5008", format!("fresh exporter directory: {error}")))?;
    tar::Archive::new(Cursor::new(ARCHIVE))
        .unpack(destination)
        .map_err(|error| PrismError::new("PP5008", format!("pinned exporter archive: {error}")))?;
    if !destination.join("Prod/Export.lean").is_file() || destination.join(".lake").exists() {
        return Err(PrismError::new(
            "PP5008",
            "pinned exporter source is incomplete or contains build state",
        ));
    }
    Ok(())
}

/// A repository-backed verifier must use the same pinned source as builders.
pub(crate) fn verify_source(repository_root: &Path) -> Result<(), PrismError> {
    let fail = || {
        PrismError::new("PP5008", "repository exporter archive is missing, aliased, changed, or differs from the compiled pin")
    };
    let path = repository_root.join("vendor/lean4-prod/lean.tar");
    if path.canonicalize().map_err(|_| fail())? != path {
        return Err(fail());
    }
    let before = std::fs::symlink_metadata(&path).map_err(|_| fail())?;
    if !before.is_file() || before.len() != ARCHIVE.len() as u64 {
        return Err(fail());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        if before.nlink() != 1 {
            return Err(fail());
        }
        options.custom_flags(
            (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
        );
    }
    let mut file = options.open(&path).map_err(|_| fail())?;
    let same = |metadata: &std::fs::Metadata| {
        let common = metadata.is_file()
            && metadata.len() == before.len()
            && metadata.modified().ok() == before.modified().ok();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            common
                && metadata.dev() == before.dev()
                && metadata.ino() == before.ino()
                && metadata.nlink() == 1
                && metadata.mode() == before.mode()
                && metadata.ctime() == before.ctime()
                && metadata.ctime_nsec() == before.ctime_nsec()
        }
        #[cfg(not(unix))]
        {
            common
        }
    };
    if !same(&file.metadata().map_err(|_| fail())?) {
        return Err(fail());
    }
    let mut bytes = Vec::with_capacity(ARCHIVE.len());
    file.by_ref()
        .take(ARCHIVE.len() as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| fail())?;
    if !same(&file.metadata().map_err(|_| fail())?)
        || !same(&std::fs::symlink_metadata(&path).map_err(|_| fail())?)
        || path.canonicalize().map_err(|_| fail())? != path
    {
        return Err(fail());
    }
    if bytes != ARCHIVE {
        return Err(PrismError::new(
            "PP5008",
            "repository exporter archive differs from the compiled pin",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn invocation_guard_rejects_actual_child_mutation_and_same_bytes_replacement() {
        use std::os::unix::fs::PermissionsExt;
        // Real process boundary fixture, not Lean/compiler acceptance. The
        // production conformance cases separately execute actual Lake exports.
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(".lake/build/bin")).unwrap();
        let child = root.path().join(".lake/build/bin/prod-export");
        let original = b"#!/bin/sh\nexit 0\n";
        std::fs::write(&child, original).unwrap();
        std::fs::set_permissions(&child, std::fs::Permissions::from_mode(0o755)).unwrap();
        let invoke = || {
            run_export(
                "exporter-identity-test",
                Path::new("/usr/bin/sh"),
                &["exe".to_owned(), "prod-export".to_owned()],
                root.path(),
                &std::collections::BTreeMap::new(),
                &[],
                "PP5004",
            )
        };
        std::fs::write(root.path().join("exe"), b"./.lake/build/bin/prod-export\n").unwrap();
        let record = invoke().unwrap();
        assert_eq!(
            record.exporter.unwrap().executable,
            measure_executable(&child).unwrap()
        );
        std::fs::write(
            root.path().join("exe"),
            b"./.lake/build/bin/prod-export\nprintf changed > .lake/build/bin/prod-export\n",
        )
        .unwrap();
        assert_eq!(invoke().unwrap_err().code, "PP5008");
        std::fs::write(&child, original).unwrap();
        std::fs::write(root.path().join("exe"), b"./.lake/build/bin/prod-export\ncp -p .lake/build/bin/prod-export replacement\nmv replacement .lake/build/bin/prod-export\n").unwrap();
        assert_eq!(invoke().unwrap_err().code, "PP5008");
        assert_eq!(std::fs::read(child).unwrap(), original);
    }

    #[cfg(unix)]
    #[test]
    fn exporter_measurement_binds_the_actual_file_and_refuses_unsafe_inputs() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("prod-export");
        std::fs::write(&path, b"measurement fixture, not executed").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let before = measure_executable(&path).unwrap();
        assert_eq!(before.byte_length, 33);
        assert_eq!(before.mode, 0o755);
        std::fs::write(&path, b"different measurement fixture...").unwrap();
        assert_ne!(measure_executable(&path).unwrap(), before);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(measure_executable(&path).unwrap_err().code, "PP5008");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o4755)).unwrap();
        assert!(measure_executable(&path).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let alias = root.path().join("alias");
        symlink(&path, &alias).unwrap();
        assert!(measure_executable(&alias).is_err());
        std::fs::remove_file(&alias).unwrap();
        std::fs::hard_link(&path, &alias).unwrap();
        assert!(measure_executable(&path).is_err());
        std::fs::remove_file(&alias).unwrap();
        let file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.set_len(8 * 1024 * 1024 * 1024).unwrap();
        assert!(measure_executable(&path).is_err());
        file.set_len(0).unwrap();
        assert!(measure_executable(&path).is_err());
        std::fs::remove_file(&path).unwrap();
        assert!(measure_executable(&path).is_err());
    }

    #[test]
    fn fresh_acquisition_has_sources_but_no_cached_acceptance() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first");
        let second = root.path().join("second");
        acquire(&first).unwrap();
        acquire(&second).unwrap();
        assert_eq!(
            std::fs::read(first.join("Prod/Export.lean")).unwrap(),
            std::fs::read(second.join("Prod/Export.lean")).unwrap()
        );
        assert!(!first.join(".lake").exists());
        assert!(!second.join(".lake").exists());
        std::fs::write(first.join("sentinel"), b"retain").unwrap();
        assert!(acquire(&first).is_err());
        assert_eq!(std::fs::read(first.join("sentinel")).unwrap(), b"retain");
    }

    #[test]
    fn repository_source_must_equal_compiled_pin() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("vendor/lean4-prod")).unwrap();
        let archive = root.path().join("vendor/lean4-prod/lean.tar");
        std::fs::write(&archive, ARCHIVE).unwrap();
        verify_source(root.path()).unwrap();
        let mut changed = ARCHIVE.to_vec();
        changed[0] ^= 1;
        std::fs::write(&archive, changed).unwrap();
        assert_eq!(verify_source(root.path()).unwrap_err().code, "PP5008");
        std::fs::write(&archive, b"not the registered compiler").unwrap();
        assert_eq!(verify_source(root.path()).unwrap_err().code, "PP5008");
        let file = std::fs::File::create(&archive).unwrap();
        file.set_len(8 * 1024 * 1024 * 1024).unwrap();
        assert_eq!(verify_source(root.path()).unwrap_err().code, "PP5008");
        std::fs::remove_file(&archive).unwrap();
        assert_eq!(verify_source(root.path()).unwrap_err().code, "PP5008");
    }

    #[cfg(unix)]
    #[test]
    fn repository_source_rejects_aliases_and_hard_links() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("vendor/lean4-prod")).unwrap();
        let archive = root.path().join("vendor/lean4-prod/lean.tar");
        let retained = root.path().join("retained");
        std::fs::write(&retained, ARCHIVE).unwrap();
        std::os::unix::fs::symlink(&retained, &archive).unwrap();
        assert_eq!(verify_source(root.path()).unwrap_err().code, "PP5008");
        std::fs::remove_file(&archive).unwrap();
        std::fs::hard_link(&retained, &archive).unwrap();
        assert_eq!(verify_source(root.path()).unwrap_err().code, "PP5008");
    }

    #[cfg(unix)]
    #[test]
    fn acquisition_refuses_existing_alias_or_file() {
        let root = tempfile::tempdir().unwrap();
        let actual = root.path().join("actual");
        std::fs::create_dir(&actual).unwrap();
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&actual, &alias).unwrap();
        assert!(acquire(&alias).is_err());
        assert_eq!(std::fs::read_dir(&actual).unwrap().count(), 0);
        let file = root.path().join("file");
        std::fs::write(&file, b"retain").unwrap();
        assert!(acquire(&file).is_err());
        assert_eq!(std::fs::read(file).unwrap(), b"retain");
    }
}
