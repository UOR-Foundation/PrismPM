//! Private package custody, never a reusable cache or an acceptance receipt.
use crate::PrismError;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::{Component, Path};

const FILES: usize = 4096;
const FILE_BYTES: u64 = 256 * 1024 * 1024;
const TREE_BYTES: u64 = 512 * 1024 * 1024;

fn failure() -> PrismError {
    PrismError::new(
        "PP5008",
        "exporter package source or build-tree custody changed",
    )
}

#[derive(Debug, PartialEq, Eq)]
struct Identity {
    directory: bool,
    length: u64,
    modified: Option<std::time::SystemTime>,
    #[cfg(unix)]
    native: (u64, u64, u32, u64, i64, i64),
}

fn identity(metadata: &std::fs::Metadata) -> Identity {
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    Identity {
        directory: metadata.is_dir(),
        length: metadata.len(),
        modified: metadata.modified().ok(),
        #[cfg(unix)]
        native: (
            metadata.dev(),
            metadata.ino(),
            metadata.mode(),
            metadata.nlink(),
            metadata.ctime(),
            metadata.ctime_nsec(),
        ),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Snapshot(BTreeMap<String, (Identity, Option<String>)>);

struct SourceMember {
    directory: bool,
    mode: u32,
    bytes: Vec<u8>,
}

fn member_name(path: &Path) -> Result<String, PrismError> {
    path.components()
        .map(|part| match part {
            Component::Normal(name) => name.to_str().ok_or_else(failure),
            _ => Err(failure()),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))
}

fn charge_build(entries: &mut usize, bytes: &mut u64, length: u64) -> Result<(), PrismError> {
    *entries = entries.checked_add(1).ok_or_else(failure)?;
    *bytes = bytes.checked_add(length).ok_or_else(failure)?;
    if *entries > FILES || length > FILE_BYTES || *bytes > TREE_BYTES {
        return Err(failure());
    }
    Ok(())
}

fn source_members() -> Result<BTreeMap<String, SourceMember>, PrismError> {
    let mut members = BTreeMap::new();
    let mut total = 0_u64;
    for entry in tar::Archive::new(Cursor::new(super::ARCHIVE))
        .entries()
        .map_err(|_| failure())?
    {
        let mut entry = entry.map_err(|_| failure())?;
        let path = entry.path().map_err(|_| failure())?.into_owned();
        let name = member_name(&path)?;
        if name == ".lake" || name.starts_with(".lake/") || name.is_empty() {
            return Err(failure());
        }
        let directory = entry.header().entry_type().is_dir();
        if !directory && !entry.header().entry_type().is_file() {
            return Err(failure());
        }
        let mode = entry.header().mode().map_err(|_| failure())?;
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failure())?;
        total = total.checked_add(bytes.len() as u64).ok_or_else(failure)?;
        if bytes.len() as u64 > FILE_BYTES
            || total > TREE_BYTES
            || members.len() >= FILES
            || members
                .insert(
                    name,
                    SourceMember {
                        directory,
                        mode,
                        bytes,
                    },
                )
                .is_some()
        {
            return Err(failure());
        }
    }
    Ok(members)
}

pub(super) fn capture(root: &Path) -> Result<Snapshot, PrismError> {
    if root.canonicalize().ok().as_deref() != Some(root) {
        return Err(failure());
    }
    let mut sources = source_members()?;
    // Seed bounds count .lake and its descendants, not the pinned source
    // members or the package root. Preserve that full admitted domain.
    let maximum_names = sources.len() + FILES + 1;
    let mut observed = BTreeMap::new();
    let mut build_entries = 0;
    let mut build_bytes = 0;
    // Bound names before buffering them. WalkDir may collect an unbounded
    // remainder of a directory when its open-descriptor limit is reached.
    let mut pending = vec![root.to_owned()];
    while let Some(owned_path) = pending.pop() {
        let path = owned_path.as_path();
        let relative = member_name(path.strip_prefix(root).map_err(|_| failure())?)?;
        let metadata = std::fs::symlink_metadata(path).map_err(|_| failure())?;
        if observed.len() >= maximum_names
            || path.canonicalize().ok().as_deref() != Some(path)
            || metadata.file_type().is_symlink()
            || !(metadata.is_file() || metadata.is_dir())
        {
            return Err(failure());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if metadata.mode() & 0o7000 != 0 || (metadata.is_file() && metadata.nlink() != 1) {
                return Err(failure());
            }
        }
        let before = identity(&metadata);
        if relative == ".lake" || relative.starts_with(".lake/") {
            charge_build(
                &mut build_entries,
                &mut build_bytes,
                if metadata.is_file() {
                    metadata.len()
                } else {
                    0
                },
            )?;
        }
        let mut digest = None;
        let source = sources.remove(&relative);
        if !relative.is_empty()
            && relative != ".lake"
            && !relative.starts_with(".lake/")
            && source.is_none()
        {
            return Err(failure());
        }
        if let Some(SourceMember {
            directory,
            mode: _mode,
            ..
        }) = &source
        {
            if *directory != metadata.is_dir() {
                return Err(failure());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                if metadata.mode() & 0o777 != *_mode & 0o777 {
                    return Err(failure());
                }
            }
        }
        if metadata.is_file() {
            if metadata.len() > FILE_BYTES {
                return Err(failure());
            }
            let mut options = std::fs::OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.custom_flags(
                    (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32,
                );
            }
            let mut file = options.open(path).map_err(|_| failure())?;
            if identity(&file.metadata().map_err(|_| failure())?) != before {
                return Err(failure());
            }
            let mut hash = Sha256::new();
            let mut count = 0_u64;
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let length = file.read(&mut buffer).map_err(|_| failure())?;
                if length == 0 {
                    break;
                }
                count += length as u64;
                if count > metadata.len() {
                    return Err(failure());
                }
                hash.update(&buffer[..length]);
            }
            if count != metadata.len()
                || identity(&file.metadata().map_err(|_| failure())?) != before
            {
                return Err(failure());
            }
            let actual = format!("{:x}", hash.finalize());
            if let Some(SourceMember {
                bytes: expected, ..
            }) = &source
            {
                if count != expected.len() as u64
                    || actual != format!("{:x}", Sha256::digest(expected))
                {
                    return Err(failure());
                }
            }
            digest = Some(actual);
        }
        observed.insert(relative, (before, digest));
        if metadata.is_dir() {
            // Read one entry at a time, retaining at most maximum_names names.
            // The directory identity was captured before opening its stream
            // and is checked again with every other member below.
            for entry in std::fs::read_dir(path).map_err(|_| failure())? {
                if observed.len() + pending.len() >= maximum_names {
                    return Err(failure());
                }
                pending.push(entry.map_err(|_| failure())?.path());
            }
        }
    }
    if !sources.is_empty() || !observed.contains_key(".lake/build/bin/prod-export") {
        return Err(failure());
    }
    // Recheck all names and identities, including directories after traversal.
    // Hash equality alone cannot accept same-byte replacement or namespace drift.
    for (relative, (before, _)) in &observed {
        let path = root.join(relative);
        if path.canonicalize().ok().as_deref() != Some(path.as_path())
            || identity(&std::fs::symlink_metadata(path).map_err(|_| failure())?) != *before
        {
            return Err(failure());
        }
    }
    Ok(Snapshot(observed))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().join("package");
        super::super::acquire(&root).unwrap();
        std::fs::create_dir_all(root.join(".lake/build/bin")).unwrap();
        // Capture-only fixture, never compiler or execution acceptance.
        std::fs::write(
            root.join(".lake/build/bin/prod-export"),
            b"measurement fixture",
        )
        .unwrap();
        assert!(capture(&root).is_ok());
        (owner, root)
    }

    #[test]
    fn every_pinned_source_member_is_required_and_byte_bound() {
        let (_owner, root) = fixture();
        for (name, member) in source_members().unwrap() {
            if member.directory {
                continue;
            }
            let path = root.join(name);
            std::fs::remove_file(&path).unwrap();
            assert!(capture(&root).is_err(), "missing source member");
            std::fs::write(&path, b"changed").unwrap();
            assert!(capture(&root).is_err(), "changed source member");
            std::fs::write(&path, member.bytes).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(member.mode))
                    .unwrap();
            }
            assert!(capture(&root).is_ok());
        }
        std::fs::write(root.join("unregistered-source"), b"extra").unwrap();
        assert!(capture(&root).is_err());
    }

    #[test]
    fn build_tree_replacement_and_capture_bounds_are_enforced() {
        let (_owner, root) = fixture();
        let member = root.join(".lake/build/input");
        std::fs::write(&member, b"same bytes").unwrap();
        let original = capture(&root).unwrap();
        let replacement = root.join(".lake/build/replacement");
        std::fs::write(&replacement, b"same bytes").unwrap();
        std::fs::rename(replacement, &member).unwrap();
        assert_ne!(capture(&root).unwrap(), original);
        std::fs::OpenOptions::new()
            .write(true)
            .open(&member)
            .unwrap()
            .set_len(FILE_BYTES + 1)
            .unwrap();
        assert!(capture(&root).is_err());
        std::fs::remove_file(member).unwrap();
        let mut nested = root.join(".lake");
        for _ in 0..64 {
            nested = nested.join("nested");
            std::fs::create_dir(&nested).unwrap();
        }
        let complete = capture(&root).unwrap();
        std::fs::write(nested.join("deep-member"), b"captured").unwrap();
        assert_ne!(
            capture(&root).unwrap(),
            complete,
            "deep members are not omitted"
        );
        let (_other_owner, other) = fixture();
        let mut wide = other.join(".lake");
        for _ in 0..12 {
            wide = wide.join("deep");
            std::fs::create_dir(&wide).unwrap();
        }
        for index in 0..FILES {
            std::fs::write(wide.join(format!("entry-{index}")), b"").unwrap();
        }
        assert!(capture(&other).is_err(), "deep and wide entry count bound");
        let (_maximum_owner, maximum) = fixture();
        // Existing .lake, build, bin and prod-export are four seed entries. The
        // source inventory and roots must not consume the seed's allowance.
        for index in 4..FILES {
            std::fs::write(maximum.join(format!(".lake/entry-{index}")), b"").unwrap();
        }
        assert!(capture(&maximum).is_ok(), "complete seed entry allowance");
        std::fs::write(maximum.join(".lake/overflow"), b"").unwrap();
        assert!(capture(&maximum).is_err());
        let (mut entries, mut bytes) = (0, 0);
        charge_build(&mut entries, &mut bytes, FILE_BYTES).unwrap();
        charge_build(&mut entries, &mut bytes, FILE_BYTES).unwrap();
        assert_eq!(bytes, TREE_BYTES);
        assert!(charge_build(&mut entries, &mut bytes, 1).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn aliases_links_special_files_and_permissions_are_refused() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let (owner, root) = fixture();
        let alias = owner.path().join("alias");
        symlink(&root, &alias).unwrap();
        assert!(capture(&alias).is_err());
        let member = root.join(".lake/alias");
        symlink(root.join("lean-toolchain"), &member).unwrap();
        assert!(capture(&root).is_err());
        std::fs::remove_file(&member).unwrap();
        std::fs::hard_link(root.join("lean-toolchain"), &member).unwrap();
        assert!(capture(&root).is_err());
        std::fs::remove_file(&member).unwrap();
        let source = root.join("lean-toolchain");
        let original = std::fs::metadata(&source).unwrap().permissions();
        std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(capture(&root).is_err());
        std::fs::set_permissions(&source, original).unwrap();
        assert!(std::process::Command::new("mkfifo")
            .arg(&member)
            .status()
            .unwrap()
            .success());
        assert!(capture(&root).is_err());
        std::fs::remove_file(&member).unwrap();
        assert!(capture(&root).is_ok());
    }
}
