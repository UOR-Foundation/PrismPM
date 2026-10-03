//! Bounded reads from one stable regular executable, without following a replaced leaf.

use sha2::{Digest, Sha256};
use std::fs::{File, Metadata};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

fn changed(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn same(left: &Metadata, right: &Metadata) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        left.dev() == right.dev()
            && left.ino() == right.ino()
            && left.mode() == right.mode()
            && left.len() == right.len()
            && left.mtime() == right.mtime()
            && left.mtime_nsec() == right.mtime_nsec()
            && left.ctime() == right.ctime()
            && left.ctime_nsec() == right.ctime_nsec()
    }
    #[cfg(not(unix))]
    {
        left.is_file() == right.is_file()
            && left.len() == right.len()
            && left.modified().ok() == right.modified().ok()
            && left.created().ok() == right.created().ok()
    }
}

/// One opened SDK executable and its original regular-file identity.
pub(crate) struct ExecutableFile {
    file: File,
    path: PathBuf,
    original: Metadata,
}

impl ExecutableFile {
    /// Open a canonical regular file; Unix replacement with a FIFO cannot block.
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() || path.canonicalize()? != path {
            return Err(changed("SDK executable path is not canonical"));
        }
        let original = std::fs::symlink_metadata(path)?;
        if !original.is_file() {
            return Err(changed("SDK executable is not a regular file"));
        }
        #[cfg(unix)]
        let file = File::from(rustix::fs::open(
            path,
            rustix::fs::OFlags::RDONLY
                | rustix::fs::OFlags::CLOEXEC
                | rustix::fs::OFlags::NOFOLLOW
                | rustix::fs::OFlags::NONBLOCK,
            rustix::fs::Mode::empty(),
        )?);
        #[cfg(not(unix))]
        let file = File::open(path)?;
        if !same(&original, &file.metadata()?) {
            return Err(changed("SDK executable changed while opening"));
        }
        Ok(Self {
            file,
            path: path.to_owned(),
            original,
        })
    }

    /// Byte length measured from the exact opened file before allocation.
    pub(crate) fn len(&self) -> u64 {
        self.original.len()
    }

    /// Check both the held file and its current canonical path after reading.
    pub(crate) fn verify(&self) -> io::Result<()> {
        if !same(&self.original, &self.file.metadata()?)
            || !same(&self.original, &std::fs::symlink_metadata(&self.path)?)
        {
            return Err(changed("SDK executable identity changed during read"));
        }
        Ok(())
    }

    /// Keep the selected command alias bound to the same opened canonical target.
    pub(crate) fn verify_reference(&self, selected: &Path) -> io::Result<()> {
        if selected.canonicalize()? != self.path {
            return Err(changed("SDK executable selection changed during read"));
        }
        self.verify()
    }

    /// Hash at most the original length plus one growth-detection byte.
    pub(crate) fn sha256(&mut self) -> io::Result<String> {
        let limit = self
            .len()
            .checked_add(1)
            .ok_or_else(|| changed("SDK executable size overflow"))?;
        let mut reader = (&mut self.file).take(limit);
        let mut sha = Sha256::new();
        let mut bytes = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            bytes += count as u64;
            sha.update(&buffer[..count]);
        }
        if bytes != self.len() {
            return Err(changed("SDK executable length changed during read"));
        }
        self.verify()?;
        Ok(format!("{:x}", sha.finalize()))
    }

    /// Read exactly an independently pinned size, without geometric Vec growth.
    pub(crate) fn read_exact_bytes(&mut self, expected: usize) -> io::Result<Vec<u8>> {
        if self.len() != expected as u64 {
            return Err(changed("SDK executable size is not pinned"));
        }
        let mut bytes = vec![0u8; expected];
        self.file.read_exact(&mut bytes)?;
        let mut extra = [0u8; 1];
        if self.file.read(&mut extra)? != 0 {
            return Err(changed("SDK executable grew during read"));
        }
        self.verify()?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn streaming_hash_matches_independent_bytes_across_chunk_boundaries() {
        for length in [0, 1, 65535, 65536, 65537, 131073] {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("tool");
            let bytes = (0..length)
                .map(|index| (index % 251) as u8)
                .collect::<Vec<_>>();
            std::fs::write(&path, &bytes).unwrap();
            let mut file = ExecutableFile::open(&path).unwrap();
            assert_eq!(
                file.sha256().unwrap(),
                format!("{:x}", Sha256::digest(&bytes))
            );
            let mut file = ExecutableFile::open(&path).unwrap();
            assert_eq!(file.read_exact_bytes(length).unwrap(), bytes);
        }
    }

    #[test]
    fn oversized_sparse_and_short_or_growing_files_refuse() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("tool");
        let sparse = File::create(&path).unwrap();
        sparse.set_len(1u64 << 40).unwrap();
        let mut file = ExecutableFile::open(&path).unwrap();
        assert!(file
            .read_exact_bytes(141_178_250)
            .unwrap_err()
            .to_string()
            .contains("size is not pinned"));
        sparse.set_len(0).unwrap();
        for exact in [false, true] {
            std::fs::write(&path, b"abc").unwrap();
            let mut file = ExecutableFile::open(&path).unwrap();
            std::fs::write(&path, b"a").unwrap();
            assert!(if exact {
                file.read_exact_bytes(3).map(|_| ())
            } else {
                file.sha256().map(|_| ())
            }
            .is_err());
            std::fs::write(&path, b"abc").unwrap();
            let mut file = ExecutableFile::open(&path).unwrap();
            File::options()
                .append(true)
                .open(&path)
                .unwrap()
                .write_all(b"d")
                .unwrap();
            assert!(if exact {
                file.read_exact_bytes(3).map(|_| ())
            } else {
                file.sha256().map(|_| ())
            }
            .is_err());
        }
    }

    #[cfg(unix)]
    #[test]
    fn replacement_alias_symlink_and_nonregular_files_refuse() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("tool");
        std::fs::write(&path, b"same").unwrap();
        let mut opened = ExecutableFile::open(&path).unwrap();
        std::fs::rename(&path, root.path().join("old")).unwrap();
        std::fs::write(&path, b"same").unwrap();
        assert!(opened
            .sha256()
            .unwrap_err()
            .to_string()
            .contains("identity changed"));
        let alias = root.path().join("alias");
        std::os::unix::fs::symlink(&path, &alias).unwrap();
        assert!(ExecutableFile::open(&alias).is_err());
        let mut opened = ExecutableFile::open(&path).unwrap();
        assert_eq!(opened.read_exact_bytes(4).unwrap(), b"same");
        opened.verify_reference(&alias).unwrap();
        std::fs::remove_file(&alias).unwrap();
        std::os::unix::fs::symlink(root.path().join("old"), &alias).unwrap();
        assert!(opened.verify_reference(&alias).is_err());
        assert!(ExecutableFile::open(root.path()).is_err());
        #[cfg(target_os = "linux")]
        {
            let fifo = root.path().join("fifo");
            rustix::fs::mkfifoat(
                rustix::fs::CWD,
                &fifo,
                rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR,
            )
            .unwrap();
            assert!(ExecutableFile::open(&fifo).is_err());
        }
    }
}
