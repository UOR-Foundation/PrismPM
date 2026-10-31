//! Keep nested Cargo feature builds from replacing the executing VV driver.

use crate::Fail;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

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
