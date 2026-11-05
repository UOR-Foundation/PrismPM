//! Keep nested Cargo feature builds from replacing the executing VV driver.

use crate::Fail;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

pub(crate) struct CapturedOutput {
    pub(crate) output: Output,
    pub(crate) overflow: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileIdentity([u64; 10]);

impl FileIdentity {
    fn capture(metadata: &std::fs::Metadata) -> Result<Self, Fail> {
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err("inventory executable must be a regular file".into());
            }
            // This is only a streaming inventory probe bound. It does not
            // enlarge any test owner's memory, time, or output allowance.
            if metadata.len() == 0 || metadata.len() > 2 * 1024 * 1024 * 1024 {
                return Err("inventory executable exceeds its size bound".into());
            }
            Ok(Self([
                metadata.dev(),
                metadata.ino(),
                u64::from(metadata.mode()),
                metadata.nlink(),
                metadata.len(),
                metadata.mtime() as u64,
                metadata.mtime_nsec() as u64,
                metadata.ctime() as u64,
                metadata.ctime_nsec() as u64,
                u64::from(metadata.uid()),
            ]))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = metadata;
            Err("descriptor-bound test inventory requires Linux".into())
        }
    }
}

pub(crate) struct HeldHarness {
    file: std::fs::File,
    original: PathBuf,
    execution_path: PathBuf,
    identity: FileIdentity,
    sha256: String,
}

impl HeldHarness {
    fn open(original: PathBuf) -> Result<Self, Fail> {
        if original.canonicalize()? != original {
            return Err("test harness path contains an alias".into());
        }
        let identity = FileIdentity::capture(&std::fs::symlink_metadata(&original)?)?;
        let mut file = std::fs::File::open(&original)?;
        if FileIdentity::capture(&file.metadata()?)? != identity {
            return Err("test harness changed while opening its descriptor".into());
        }
        let mut digest = Sha256::new();
        let mut size = 0_u64;
        let mut buffer = [0; 65536];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            size += count as u64;
            if size > identity.0[4] {
                return Err("test harness grew while hashing".into());
            }
            digest.update(&buffer[..count]);
        }
        if size != identity.0[4] {
            return Err("test harness shrank while hashing".into());
        }
        #[cfg(target_os = "linux")]
        let execution_path = {
            use std::os::fd::AsRawFd;
            PathBuf::from(format!(
                "/proc/{}/fd/{}",
                std::process::id(),
                file.as_raw_fd()
            ))
        };
        #[cfg(not(target_os = "linux"))]
        let execution_path = PathBuf::new();
        let held = Self {
            file,
            original,
            execution_path,
            identity,
            sha256: format!("{:x}", digest.finalize()),
        };
        held.require_unchanged()?;
        Ok(held)
    }

    fn require_unchanged(&self) -> Result<(), Fail> {
        if self.original.canonicalize()? != self.original
            || FileIdentity::capture(&self.file.metadata()?)? != self.identity
            || FileIdentity::capture(&std::fs::symlink_metadata(&self.original)?)? != self.identity
        {
            return Err("held test harness identity changed during inventory".into());
        }
        Ok(())
    }

    pub(crate) fn original(&self) -> &Path {
        &self.original
    }
    pub(crate) fn execution_path(&self) -> &Path {
        &self.execution_path
    }
    pub(crate) fn sha256(&self) -> &str {
        &self.sha256
    }
}

fn read_bounded(mut reader: impl Read, limit: usize) -> std::io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    let mut overflow = false;
    let mut buffer = [0; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok((bytes, overflow));
        }
        let retained = count.min(limit.saturating_sub(bytes.len()));
        bytes.extend_from_slice(&buffer[..retained]);
        overflow |= retained != count;
    }
}

fn capture_bounded(mut command: Command, limit: usize) -> Result<CapturedOutput, Fail> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    // Drain both pipes even after their retention bound: neither a full stderr
    // pipe nor discarded output may hide a failed inventory command.
    let stdout = std::thread::spawn(move || read_bounded(stdout, limit));
    let stderr = std::thread::spawn(move || read_bounded(stderr, limit));
    let status = child.wait();
    let (stdout, stdout_overflow) = stdout.join().map_err(|_| "stdout reader panicked")??;
    let (stderr, stderr_overflow) = stderr.join().map_err(|_| "stderr reader panicked")??;
    Ok(CapturedOutput {
        output: Output {
            status: status?,
            stdout,
            stderr,
        },
        overflow: stdout_overflow || stderr_overflow,
    })
}

/// The verifier attests the exact executing binary, so its installed path must
/// remain readable and unchanged throughout all nested workspace Cargo gates.
pub(crate) struct GateDriver {
    executable: ExecutableIdentity,
    target_dir: PathBuf,
}

struct ExecutableIdentity {
    path: PathBuf,
    sha256: [u8; 32],
}

impl ExecutableIdentity {
    fn capture(path: PathBuf) -> Result<Self, Fail> {
        let sha256 = Sha256::digest(std::fs::read(&path)?).into();
        Ok(Self { path, sha256 })
    }

    fn require_unchanged(&self, current_path: &Path) -> Result<(), Fail> {
        if current_path != self.path {
            return Err("VV driver executable path changed during a nested Cargo gate".into());
        }
        let bytes = std::fs::read(&self.path)
            .map_err(|error| format!("VV driver executable is no longer readable: {error}"))?;
        let actual: [u8; 32] = Sha256::digest(bytes).into();
        if actual != self.sha256 {
            return Err("VV driver executable bytes changed during a nested Cargo gate".into());
        }
        Ok(())
    }
}

fn isolated_target(root: &Path, executable: &Path) -> Result<PathBuf, Fail> {
    let executable = executable.canonicalize()?;
    // The alternate also permits direct `cargo xtask vv` when the caller uses
    // our usual nested target as its own CARGO_TARGET_DIR. Never delete or
    // overwrite the caller's target, and resolve existing symlinks before
    // deciding whether a candidate can contain the executing driver.
    for name in ["vv-workspace", "vv-workspace-alternate"] {
        let candidate = root.join("target").join(name);
        std::fs::create_dir_all(&candidate)?;
        let candidate = candidate.canonicalize()?;
        if !executable.starts_with(&candidate) {
            return Ok(candidate);
        }
    }
    Err("cannot isolate nested Cargo gates from the executing VV driver".into())
}

impl GateDriver {
    pub(crate) fn capture(root: &Path) -> Result<Self, Fail> {
        let executable = ExecutableIdentity::capture(std::env::current_exe()?)?;
        let target_dir = isolated_target(root, &executable.path)?;
        Ok(Self {
            executable,
            target_dir,
        })
    }

    pub(crate) fn require_unchanged(&self) -> Result<(), Fail> {
        self.executable.require_unchanged(&std::env::current_exe()?)
    }

    fn cargo_command(&self, root: &Path, arguments: &[&str]) -> Command {
        let mut command = Command::new("cargo");
        command
            .args(arguments)
            .current_dir(root)
            .env("CARGO_NET_OFFLINE", "true")
            .env("CARGO_TARGET_DIR", &self.target_dir);
        command
    }

    pub(crate) fn run_cargo(&self, root: &Path, arguments: &[&str]) -> Result<(), Fail> {
        self.executable
            .require_unchanged(&std::env::current_exe()?)?;
        let outcome = self.cargo_command(root, arguments).status();
        self.executable
            .require_unchanged(&std::env::current_exe()?)?;
        let status = outcome?;
        if !status.success() {
            return Err(format!("cargo {} exited {status}", arguments.join(" ")).into());
        }
        Ok(())
    }

    pub(crate) fn capture_cargo(
        &self,
        root: &Path,
        arguments: &[&str],
        limit: usize,
    ) -> Result<CapturedOutput, Fail> {
        self.executable
            .require_unchanged(&std::env::current_exe()?)?;
        let outcome = capture_bounded(self.cargo_command(root, arguments), limit);
        self.executable
            .require_unchanged(&std::env::current_exe()?)?;
        outcome
    }

    pub(crate) fn hold_harness(&self, executable: &Path) -> Result<HeldHarness, Fail> {
        let canonical = executable.canonicalize()?;
        if canonical != executable || !canonical.starts_with(self.target_dir.canonicalize()?) {
            return Err("test harness is aliased or outside the isolated Cargo target".into());
        }
        HeldHarness::open(canonical)
    }

    pub(crate) fn capture_harness(
        &self,
        root: &Path,
        held: &HeldHarness,
        arguments: &[&str],
        limit: usize,
    ) -> Result<CapturedOutput, Fail> {
        held.require_unchanged()?;
        self.executable
            .require_unchanged(&std::env::current_exe()?)?;
        let mut command = Command::new(held.execution_path());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.arg0(held.original());
        }
        command.args(arguments).current_dir(root);
        let outcome = capture_bounded(command, limit);
        self.executable
            .require_unchanged(&std::env::current_exe()?)?;
        held.require_unchanged()?;
        outcome
    }

    /// CLI produced by a scoped Cargo build, never the running VV driver.
    pub(crate) fn prismpm_binary(&self) -> PathBuf {
        self.target_dir
            .join("debug")
            .join(format!("prismpm{}", std::env::consts::EXE_SUFFIX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_output_is_bounded_without_stopping_pipe_drainage() {
        let (bytes, overflow) = read_bounded(std::io::Cursor::new(b"abcdef"), 3).unwrap();
        assert_eq!(bytes, b"abc");
        assert!(overflow);
        let (bytes, overflow) = read_bounded(std::io::Cursor::new(b"abc"), 3).unwrap();
        assert_eq!(bytes, b"abc");
        assert!(!overflow);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn inventory_executes_the_held_inode_and_rejects_alias_or_replace_restore() {
        let work = tempfile::tempdir().unwrap();
        let driver = GateDriver::capture(work.path()).unwrap();
        std::fs::create_dir_all(&driver.target_dir).unwrap();
        let executable = driver.target_dir.join("held-test-harness");
        std::fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        let held = driver.hold_harness(&executable).unwrap();
        assert!(held
            .execution_path()
            .to_str()
            .unwrap()
            .starts_with(&format!("/proc/{}/fd/", std::process::id())));
        let capture = driver
            .capture_harness(
                work.path(),
                &held,
                &["--list", "--format=terse"],
                1024 * 1024,
            )
            .unwrap();
        assert!(capture.output.status.success());
        assert!(!capture.overflow);
        assert!(String::from_utf8(capture.output.stdout)
            .unwrap()
            .contains("inventory_executes_the_held_inode"));
        let alias = driver.target_dir.join("aliased-harness");
        std::os::unix::fs::symlink(&executable, &alias).unwrap();
        assert!(driver.hold_harness(&alias).is_err());
        let parked = driver.target_dir.join("parked-harness");
        std::fs::rename(&executable, &parked).unwrap();
        std::fs::write(&executable, b"foreign executable").unwrap();
        std::fs::rename(&parked, &executable).unwrap();
        // Even restoration of the original pathname and bytes cannot restore
        // its inode change time or authorize a different listing executable.
        assert!(held.require_unchanged().is_err());
        assert!(driver
            .capture_harness(work.path(), &held, &["--list"], 1024)
            .is_err());
    }

    #[test]
    fn executable_identity_rejects_changed_or_deleted_driver() {
        let work = tempfile::tempdir().unwrap();
        let path = work.path().join("driver");
        std::fs::write(&path, b"original executing driver").unwrap();
        let identity = ExecutableIdentity::capture(path.clone()).unwrap();
        identity.require_unchanged(&path).unwrap();
        std::fs::write(&path, b"replacement feature build").unwrap();
        assert!(identity
            .require_unchanged(&path)
            .unwrap_err()
            .to_string()
            .contains("bytes changed"));
        std::fs::remove_file(&path).unwrap();
        assert!(identity
            .require_unchanged(&path)
            .unwrap_err()
            .to_string()
            .contains("no longer readable"));
        // Reinstalling identical bytes does not repair the live executable's
        // deleted path: the actual current_exe() observation must also match.
        std::fs::write(&path, b"original executing driver").unwrap();
        identity.require_unchanged(&path).unwrap();
        assert!(identity
            .require_unchanged(&path.with_extension("deleted"))
            .unwrap_err()
            .to_string()
            .contains("path changed"));
    }

    #[test]
    fn nested_target_cannot_contain_the_running_driver() {
        let work = tempfile::tempdir().unwrap();
        let ordinary = work.path().join("target/debug/xtask");
        std::fs::create_dir_all(ordinary.parent().unwrap()).unwrap();
        std::fs::write(&ordinary, b"driver").unwrap();
        let first = isolated_target(work.path(), &ordinary).unwrap();
        assert!(first.ends_with("target/vv-workspace"));
        let custom = first.join("debug/xtask");
        std::fs::create_dir_all(custom.parent().unwrap()).unwrap();
        std::fs::write(&custom, b"custom target driver").unwrap();
        let alternate = isolated_target(work.path(), &custom).unwrap();
        assert!(alternate.ends_with("target/vv-workspace-alternate"));
        assert!(!custom.canonicalize().unwrap().starts_with(&alternate));
        // Both original executables remain intact; selection is non-destructive.
        assert_eq!(std::fs::read(ordinary).unwrap(), b"driver");
        assert_eq!(std::fs::read(custom).unwrap(), b"custom target driver");
    }

    #[test]
    fn nested_command_preserves_flags_and_scopes_target_environment() {
        let work = tempfile::tempdir().unwrap();
        let driver = GateDriver::capture(work.path()).unwrap();
        let inherited = std::env::var_os("CARGO_TARGET_DIR");
        let arguments = [
            "test",
            "--workspace",
            "--all-features",
            "--locked",
            "--offline",
        ];
        let command = driver.cargo_command(work.path(), &arguments);
        assert_eq!(command.get_args().collect::<Vec<_>>(), arguments);
        assert_eq!(command.get_current_dir(), Some(work.path()));
        let environment = command
            .get_envs()
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            environment.get(std::ffi::OsStr::new("CARGO_TARGET_DIR")),
            Some(&Some(driver.target_dir.as_os_str()))
        );
        assert_eq!(
            environment.get(std::ffi::OsStr::new("CARGO_NET_OFFLINE")),
            Some(&Some(std::ffi::OsStr::new("true")))
        );
        assert_eq!(std::env::var_os("CARGO_TARGET_DIR"), inherited);
        assert!(driver.prismpm_binary().starts_with(&driver.target_dir));
        assert_ne!(driver.prismpm_binary(), driver.executable.path);
    }

    #[cfg(unix)]
    #[test]
    fn nested_target_rejects_symlink_aliases_of_the_driver_target() {
        let work = tempfile::tempdir().unwrap();
        let shared = work.path().join("shared");
        std::fs::create_dir_all(shared.join("debug")).unwrap();
        let executable = shared.join("debug/xtask");
        std::fs::write(&executable, b"driver").unwrap();
        std::fs::create_dir_all(work.path().join("target")).unwrap();
        for name in ["vv-workspace", "vv-workspace-alternate"] {
            std::os::unix::fs::symlink(&shared, work.path().join("target").join(name)).unwrap();
        }
        assert!(isolated_target(work.path(), &executable).is_err());
        assert_eq!(std::fs::read(executable).unwrap(), b"driver");
    }
}
