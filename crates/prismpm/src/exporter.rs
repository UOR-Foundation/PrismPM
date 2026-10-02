//! One fresh acquisition boundary for the pinned Lean exporter package.

use crate::error::PrismError;
use std::io::{Cursor, Read};
use std::path::Path;

const ARCHIVE: &[u8] = include_bytes!("../vendor/lean4-prod/lean.tar");

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
