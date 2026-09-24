//! Explicit execution environments for reviewed raw golden records.

/// A process environment, not evidence of physical native hardware.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Platform {
    /// The normative Debian source-development environment.
    DevelopmentAmd64,
    /// The Ubuntu AMD64 SDK environment.
    SdkAmd64,
    /// The Ubuntu ARM64 SDK environment.
    SdkArm64,
}

impl Platform {
    /// Select only known Linux GNU environments; never infer a compatible one.
    pub fn select(os: &str, arch: &str, environment: &str, release: &str) -> Result<Self, String> {
        if os != "linux" || environment != "gnu" || release.len() > 16384 || release.contains('\0')
        {
            return Err("unsupported golden execution environment".to_owned());
        }
        let mut fields = std::collections::BTreeMap::new();
        for line in release.lines().map(str::trim) {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            if !matches!(key, "ID" | "VERSION_ID") {
                continue;
            }
            let value =
                if let Some(quote) = value.chars().next().filter(|ch| matches!(ch, '\'' | '"')) {
                    value
                        .strip_prefix(quote)
                        .and_then(|value| value.strip_suffix(quote))
                        .ok_or("invalid golden OS release quoting")?
                } else {
                    value
                };
            if value.is_empty()
                || !value.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'_' | b'-')
                })
                || fields.insert(key, value).is_some()
            {
                return Err("ambiguous golden OS release identity".to_owned());
            }
        }
        match (
            arch,
            fields.get("ID").copied(),
            fields.get("VERSION_ID").copied(),
        ) {
            ("x86_64", Some("debian"), Some("12")) => Ok(Self::DevelopmentAmd64),
            ("x86_64", Some("ubuntu"), Some("24.04")) => Ok(Self::SdkAmd64),
            ("aarch64", Some("ubuntu"), Some("24.04")) => Ok(Self::SdkArm64),
            _ => Err("unsupported golden execution environment".to_owned()),
        }
    }

    /// Read the actual process and OS identity, with no caller override.
    pub fn current() -> Result<Self, String> {
        use std::io::Read;
        let mut release = String::new();
        std::fs::File::open("/etc/os-release")
            .map_err(|error| error.to_string())?
            .take(16385)
            .read_to_string(&mut release)
            .map_err(|error| error.to_string())?;
        Self::select(
            std::env::consts::OS,
            std::env::consts::ARCH,
            if cfg!(target_env = "gnu") {
                "gnu"
            } else {
                "unsupported"
            },
            &release,
        )
    }

    /// Fixed repository destination; missing reviewed records are an error.
    pub fn directory(self) -> &'static str {
        match self {
            Self::DevelopmentAmd64 => "tests/golden/stdlib",
            Self::SdkAmd64 => "tests/golden/native/linux-amd64-ubuntu-24.04",
            Self::SdkArm64 => "tests/golden/native/linux-arm64-ubuntu-24.04",
        }
    }

    /// Rust's architecture name used in the raw LexLean host record.
    pub fn architecture(self) -> &'static str {
        match self {
            Self::DevelopmentAmd64 | Self::SdkAmd64 => "x86_64",
            Self::SdkArm64 => "aarch64",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_profiles_are_exact_and_unknown_environments_fail_closed() {
        assert_eq!(
            Platform::select("linux", "x86_64", "gnu", "ID=debian\nVERSION_ID=\"12\"\n").unwrap(),
            Platform::DevelopmentAmd64
        );
        assert_eq!(
            Platform::select(
                "linux",
                "x86_64",
                "gnu",
                "ID=ubuntu\nVERSION_ID=\"24.04\"\n"
            )
            .unwrap(),
            Platform::SdkAmd64
        );
        assert_eq!(
            Platform::select("linux", "aarch64", "gnu", "ID=ubuntu\nVERSION_ID=24.04\n").unwrap(),
            Platform::SdkArm64
        );
        for (os, arch, environment, release) in [
            ("linux", "x86_64", "musl", "ID=debian\nVERSION_ID=12\n"),
            ("darwin", "aarch64", "gnu", "ID=ubuntu\nVERSION_ID=24.04\n"),
            ("linux", "aarch64", "gnu", "ID=debian\nVERSION_ID=12\n"),
            ("linux", "x86_64", "gnu", "ID=ubuntu\nVERSION_ID=26.04\n"),
            (
                "linux",
                "x86_64",
                "gnu",
                "ID=ubuntu\nID=debian\nVERSION_ID=12\n",
            ),
            (
                "linux",
                "x86_64",
                "gnu",
                "ID=debian\nVERSION_ID=12\nVERSION_ID=12\n",
            ),
            ("linux", "x86_64", "gnu", "ID=debian\nVERSION_ID=\"12\n"),
            (
                "linux",
                "x86_64",
                "gnu",
                "ID='debian'\nVERSION_ID=$(echo 12)\n",
            ),
            ("linux", "x86_64", "gnu", "ID=debian\n"),
        ] {
            assert!(
                Platform::select(os, arch, environment, release).is_err(),
                "unexpected profile for {release:?}"
            );
        }
    }
}
