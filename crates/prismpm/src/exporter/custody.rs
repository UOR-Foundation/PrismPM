//! Private package custody, never a reusable cache or an acceptance receipt.
use crate::PrismError;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Component, Path};

const FILES: usize = 4096;
const FILE_BYTES: u64 = 256 * 1024 * 1024;
const TREE_BYTES: u64 = 512 * 1024 * 1024;
const SOURCE_BYTES: u64 = 16 * 1024 * 1024;

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

impl Snapshot {
    /// Lake may legitimately refresh build traces. Source and executable
    /// identities may not change merely because a second phase builds again.
    pub(super) fn same_compiler(&self, other: &Self) -> bool {
        if self.0.keys().ne(other.0.keys()) {
            return false;
        }
        self.0.iter().all(|(name, (before, hash))| {
            let (after, next_hash) = &other.0[name];
            if !name.starts_with(".lake/") && name != ".lake" {
                return before == after && hash == next_hash;
            }
            // Only existing Lake trace bodies may refresh. Compiled libraries,
            // objects, launcher configuration and the executable remain exact.
            if name.ends_with(".trace") && !before.directory && !after.directory {
                #[cfg(unix)]
                {
                    return before.native.0 == after.native.0 && before.native.2 == after.native.2;
                }
                #[cfg(not(unix))]
                {
                    return before == after && hash == next_hash;
                }
            }
            if before.directory && after.directory {
                #[cfg(unix)]
                {
                    return before.native.0 == after.native.0
                        && before.native.1 == after.native.1
                        && before.native.2 == after.native.2
                        && before.native.3 == after.native.3;
                }
                #[cfg(not(unix))]
                {
                    return before == after;
                }
            }
            before == after && hash == next_hash
        })
    }
}

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
    source_members_from(super::ARCHIVE)
}

fn source_members_from(archive: impl Read) -> Result<BTreeMap<String, SourceMember>, PrismError> {
    let mut members = BTreeMap::new();
    let mut total = 0_u64;
    for entry in tar::Archive::new(archive)
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
        let length = entry.size();
        // Charge the source-only budget before allocating or reading a body.
        // It must never borrow the independent 512-MiB build-tree allowance.
        total = total.checked_add(length).ok_or_else(failure)?;
        if members.len() >= FILES || total > SOURCE_BYTES || (directory && length != 0) {
            return Err(failure());
        }
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(length + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| failure())?;
        if bytes.len() as u64 != length
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

    fn source_archive(rows: &[(&str, u64)]) -> Vec<u8> {
        let mut archive = tar::Builder::new(Vec::new());
        for (name, length) in rows {
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(*length);
            header.set_cksum();
            archive
                .append_data(&mut header, name, std::io::repeat(0).take(*length))
                .unwrap();
        }
        archive.into_inner().unwrap()
    }

    #[test]
    fn source_inventory_has_its_own_exact_byte_budget() {
        assert_eq!(SOURCE_BYTES, 16 * 1024 * 1024, "SPEC source-only allowance");
        assert_eq!(TREE_BYTES, 512 * 1024 * 1024, "SPEC build-only allowance");
        let exact = source_archive(&[("first", SOURCE_BYTES - 1), ("last", 1)]);
        let members = source_members_from(exact.as_slice()).unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members["first"].bytes.len() as u64, SOURCE_BYTES - 1);
        assert_eq!(members["last"].bytes, [0]);
        drop(members);
        drop(exact);
        let excess = source_archive(&[("first", SOURCE_BYTES), ("last", 1)]);
        assert!(source_members_from(excess.as_slice()).is_err());
        drop(excess);
        assert!(
            source_members_from(source_archive(&[("oversized", SOURCE_BYTES + 1)]).as_slice())
                .is_err()
        );
        // Neither the source allowance nor the package root consumes build
        // capacity. Keep the full independently admitted 512-MiB build budget.
        let (mut entries, mut bytes) = (0, 0);
        charge_build(&mut entries, &mut bytes, FILE_BYTES).unwrap();
        charge_build(&mut entries, &mut bytes, FILE_BYTES).unwrap();
        assert_eq!(bytes, TREE_BYTES);
    }

    #[test]
    fn source_inventory_has_its_own_exact_entry_budget() {
        assert_eq!(FILES, 4096, "SPEC independent inventory entry allowance");
        let names = (0..=FILES)
            .map(|index| format!("source-{index}"))
            .collect::<Vec<_>>();
        let rows = names
            .iter()
            .map(|name| (name.as_str(), 0))
            .collect::<Vec<_>>();
        assert_eq!(
            source_members_from(source_archive(&rows[..FILES]).as_slice())
                .unwrap()
                .len(),
            FILES
        );
        assert!(source_members_from(source_archive(&rows).as_slice()).is_err());
    }

    #[test]
    fn source_inventory_rejects_declared_oversize_before_reading_payload() {
        assert_header_rejected_before_body(tar::EntryType::Regular, SOURCE_BYTES + 1);
        assert_header_rejected_before_body(tar::EntryType::Regular, u64::MAX);
    }

    #[test]
    fn source_inventory_rejects_directory_payload_before_reading_it() {
        assert_header_rejected_before_body(tar::EntryType::Directory, 1);
    }

    fn assert_header_rejected_before_body(kind: tar::EntryType, length: u64) {
        // No payload exists. Refusal must come from the declared source bound,
        // not allocation or the tar reader's subsequent truncation error.
        let mut header = tar::Header::new_gnu();
        header.set_path("member").unwrap();
        header.set_entry_type(kind);
        header.set_mode(0o644);
        header.set_size(length);
        header.set_cksum();
        struct HeaderOnly<'a> {
            header: std::io::Cursor<&'a [u8]>,
            body_read: &'a std::cell::Cell<bool>,
        }
        impl Read for HeaderOnly<'_> {
            fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
                if self.header.position() < 512 {
                    self.header.read(output)
                } else {
                    self.body_read.set(true);
                    Err(std::io::Error::other("body must not be read"))
                }
            }
        }
        let body_read = std::cell::Cell::new(false);
        let result = source_members_from(HeaderOnly {
            header: std::io::Cursor::new(header.as_bytes().as_slice()),
            body_read: &body_read,
        });
        let Err(error) = result else {
            panic!("oversized source must be refused");
        };
        assert_eq!(error.code.as_str(), "PP5008");
        assert!(
            !body_read.get(),
            "source budget must be checked before reading the body"
        );
    }

    #[test]
    fn source_inventory_rejects_truncated_member_without_partial_admission() {
        let mut header = tar::Header::new_gnu();
        header.set_path("truncated").unwrap();
        header.set_mode(0o644);
        header.set_size(2);
        header.set_cksum();
        let mut truncated = header.as_bytes().to_vec();
        truncated.push(0);
        assert!(source_members_from(truncated.as_slice()).is_err());
    }

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
    fn actual_capture_preserves_full_build_budget_beside_pinned_source() {
        let (_owner, root) = fixture();
        let child_bytes = std::fs::metadata(root.join(".lake/build/bin/prod-export"))
            .unwrap()
            .len();
        for (name, length) in [
            ("first", FILE_BYTES),
            ("last", TREE_BYTES - FILE_BYTES - child_bytes),
        ] {
            std::fs::File::create(root.join(".lake").join(name))
                .unwrap()
                .set_len(length)
                .unwrap();
        }
        assert!(
            capture(&root).is_ok(),
            "pinned source must not consume build bytes"
        );
        std::fs::write(root.join(".lake/overflow"), [0]).unwrap();
        assert!(
            capture(&root).is_err(),
            "actual build aggregate excess refused"
        );
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
