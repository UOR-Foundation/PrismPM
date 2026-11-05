//! Additive scheduling of the unchanged fifteen verification gates.

use crate::gate_driver::{CapturedOutput, GateDriver, HeldHarness};
use crate::Fail;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const LIMIT: usize = 16 * 1024 * 1024;
const TOTAL_LIMIT: usize = 128 * 1024 * 1024;
const GROUPS: [&str; 7] = [
    "conformance",
    "ho-oc",
    "st-sy",
    "dk-0-1",
    "dk-2",
    "dk-3",
    "dk-4",
];
const STANDALONE: [&str; 6] = [
    "projection",
    "reservation",
    "payload",
    "recovery-frames",
    "journal-recovery",
    "retention",
];

fn known_part(part: &str) -> bool {
    matches!(part, "source" | "gates")
        || GROUPS.contains(&part)
        || part
            .strip_prefix("standalone-")
            .is_some_and(|slug| STANDALONE.contains(&slug))
}

fn filters(part: &str) -> Vec<&'static str> {
    match part {
        "ho-oc" => vec!["conformance_ho_", "conformance_oc_"],
        "st-sy" => vec!["conformance_st_", "conformance_sy_"],
        "dk-0-1" => vec!["conformance_dk_0", "conformance_dk_1"],
        "dk-2" => vec!["conformance_dk_2"],
        "dk-3" => vec!["conformance_dk_3"],
        "dk-4" => vec!["conformance_dk_40", "conformance_dk_41"],
        "conformance" => GROUPS
            .iter()
            .skip(1)
            .flat_map(|part| {
                filters(part)
                    .into_iter()
                    .flat_map(|filter| ["--skip", filter])
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn selected(name: &str, part: &str) -> bool {
    if part == "conformance" {
        return !GROUPS.iter().skip(1).any(|part| selected(name, part));
    }
    if part == "dk-4" {
        return matches!(name, "conformance_dk_40" | "conformance_dk_41");
    }
    filters(part).iter().any(|filter| name.starts_with(filter))
}

fn register(source: &str) -> Result<BTreeSet<String>, Fail> {
    let mut names = BTreeSet::new();
    for line in source
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("test_case!("))
    {
        let name = line
            .strip_prefix("test_case!(")
            .and_then(|line| line.split_once(','))
            .map(|(name, _)| name)
            .ok_or("malformed conformance test registration")?;
        if !name.starts_with("conformance_") || !names.insert(name.to_owned()) {
            return Err("invalid or duplicate conformance test registration".into());
        }
    }
    verify_partition(&names)?;
    Ok(names)
}

fn verify_partition(names: &BTreeSet<String>) -> Result<(), Fail> {
    if names.is_empty() {
        return Err("empty conformance inventory".into());
    }
    let mut selections = BTreeMap::new();
    for part in GROUPS {
        selections.insert(
            part.to_owned(),
            names
                .iter()
                .filter(|name| selected(name, part))
                .cloned()
                .collect(),
        );
    }
    for name in names {
        if name.starts_with("conformance_dk_")
            && !GROUPS.iter().skip(1).any(|part| selected(name, part))
        {
            return Err(format!("unowned DK test {name}").into());
        }
    }
    verify_union(names, &selections)
}

fn verify_union(
    expected: &BTreeSet<String>,
    selections: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), Fail> {
    let mut actual = BTreeSet::new();
    for (part, names) in selections {
        if names.is_empty() {
            return Err(format!("V&V part {part} selects no tests").into());
        }
        for name in names {
            if !actual.insert(name.clone()) {
                return Err(format!("overlapping test selection: {name}").into());
            }
        }
    }
    if &actual != expected {
        return Err("selected test union differs from the executable inventory".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Target {
    package: String,
    name: String,
    kind: String,
    source: String,
}

impl Target {
    fn key(&self) -> String {
        format!("{}/{}/{}", self.package, self.kind, self.name)
    }

    fn value(&self) -> Value {
        json!({"package":self.package,"name":self.name,"kind":self.kind,"source":self.source})
    }

    fn owner(&self) -> Result<&'static str, Fail> {
        if self.package != "repo-conformance" {
            return Ok("source");
        }
        match self.kind.as_str() {
            "lib" | "bin" => Ok("conformance"),
            "test" if self.name == "conformance" => Ok("integration"),
            _ => Err(format!("unowned Cargo target {}", self.key()).into()),
        }
    }
}

struct Metadata {
    packages: BTreeMap<String, String>,
    targets: BTreeMap<String, Target>,
    docs: BTreeMap<String, Target>,
}

fn text<'a>(value: &'a Value, field: &str) -> Result<&'a str, Fail> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string {field}").into())
}

fn array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>, Fail> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array {field}").into())
}

fn target(package: &str, value: &Value, root: &Path) -> Result<Target, Fail> {
    let kind = array(value, "kind")?;
    if kind.len() != 1 || !matches!(kind[0].as_str(), Some("lib" | "bin" | "test")) {
        return Err("unsupported testable Cargo target kind".into());
    }
    let source = Path::new(text(value, "src_path")?)
        .strip_prefix(root)?
        .to_str()
        .ok_or("non-UTF8 Cargo source")?
        .replace('\\', "/");
    Ok(Target {
        package: package.to_owned(),
        name: text(value, "name")?.to_owned(),
        kind: kind[0].as_str().unwrap().to_owned(),
        source,
    })
}

fn metadata(bytes: &[u8], root: &Path) -> Result<Metadata, Fail> {
    let value: Value = serde_json::from_slice(bytes)?;
    let members = array(&value, "workspace_members")?
        .iter()
        .map(|member| member.as_str().ok_or("non-string workspace member"))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let mut result = Metadata {
        packages: BTreeMap::new(),
        targets: BTreeMap::new(),
        docs: BTreeMap::new(),
    };
    for package in array(&value, "packages")? {
        let id = text(package, "id")?;
        if !members.contains(id) {
            continue;
        }
        let name = text(package, "name")?;
        if result
            .packages
            .insert(id.to_owned(), name.to_owned())
            .is_some()
        {
            return Err("duplicate Cargo package".into());
        }
        for row in array(package, "targets")? {
            if row.get("test").and_then(Value::as_bool) != Some(true)
                && row.get("doctest").and_then(Value::as_bool) != Some(true)
            {
                continue;
            }
            let target = target(name, row, root)?;
            target.owner()?;
            if row.get("test").and_then(Value::as_bool) == Some(true)
                && result
                    .targets
                    .insert(target.key(), target.clone())
                    .is_some()
            {
                return Err("duplicate Cargo test target".into());
            }
            if row.get("doctest").and_then(Value::as_bool) == Some(true)
                && (target.kind != "lib" || result.docs.insert(target.key(), target).is_some())
            {
                return Err("invalid or duplicate doctest target".into());
            }
        }
    }
    if result.packages.len() != members.len() || result.targets.is_empty() || result.docs.is_empty()
    {
        return Err("incomplete workspace Cargo metadata".into());
    }
    // The commands below must jointly own every declared runtime and doc target.
    for owner in ["source", "conformance", "integration"] {
        if !result
            .targets
            .values()
            .any(|target| target.owner().ok() == Some(owner))
        {
            return Err(format!("empty Cargo target owner {owner}").into());
        }
    }
    for owner in ["source", "conformance"] {
        if !result
            .docs
            .values()
            .any(|target| target.owner().ok() == Some(owner))
        {
            return Err(format!("empty doctest owner {owner}").into());
        }
    }
    Ok(result)
}

fn artifacts(
    bytes: &[u8],
    metadata: &Metadata,
    root: &Path,
    expected: &BTreeSet<String>,
) -> Result<BTreeMap<String, (Target, PathBuf)>, Fail> {
    let mut result = BTreeMap::new();
    let mut finished = false;
    for line in std::str::from_utf8(bytes)?
        .lines()
        .filter(|line| !line.is_empty())
    {
        let row: Value = serde_json::from_str(line)?;
        match row.get("reason").and_then(Value::as_str) {
            Some("build-finished") => {
                if finished || row.get("success").and_then(Value::as_bool) != Some(true) {
                    return Err("failed or duplicate Cargo build completion".into());
                }
                finished = true;
            }
            Some("compiler-artifact")
                if row.pointer("/profile/test").and_then(Value::as_bool) == Some(true) =>
            {
                let id = text(&row, "package_id")?;
                let Some(package) = metadata.packages.get(id) else {
                    continue;
                };
                let target = target(package, &row["target"], root)?;
                let key = target.key();
                if !expected.contains(&key) || metadata.targets.get(&key) != Some(&target) {
                    return Err(format!("unexpected Cargo test executable {key}").into());
                }
                let executable = PathBuf::from(text(&row, "executable")?);
                if result.insert(key, (target, executable)).is_some() {
                    return Err("duplicate Cargo test executable".into());
                }
            }
            Some("compiler-artifact" | "compiler-message" | "build-script-executed") => {}
            _ => return Err("unknown Cargo inventory JSON record".into()),
        }
    }
    if !finished || result.keys().cloned().collect::<BTreeSet<_>>() != *expected {
        return Err("Cargo executable target union differs from metadata".into());
    }
    Ok(result)
}

fn harness_list(bytes: &[u8]) -> Result<BTreeSet<String>, Fail> {
    let mut names = BTreeSet::new();
    for line in std::str::from_utf8(bytes)?
        .lines()
        .filter(|line| !line.is_empty())
    {
        let name = line
            .strip_suffix(": test")
            .or_else(|| line.strip_suffix(": benchmark"))
            .ok_or("unrecognized test harness list row")?;
        if name.is_empty() || !names.insert(name.to_owned()) {
            return Err("empty or duplicate executable test name".into());
        }
    }
    Ok(names)
}

fn doc_list(bytes: &[u8], target: &Target, root: &Path) -> Result<BTreeSet<String>, Fail> {
    let mut names = BTreeSet::new();
    let mut footer = None;
    let prefix = target
        .source
        .split("/src/")
        .next()
        .ok_or("invalid library source")?;
    for line in std::str::from_utf8(bytes)?
        .lines()
        .filter(|line| !line.is_empty())
    {
        if footer.is_some() {
            return Err("rustdoc output follows its test count".into());
        }
        if let Some((tests, benches)) = line.split_once(", ") {
            let tests = tests
                .strip_suffix(" tests")
                .or_else(|| tests.strip_suffix(" test"));
            let benches = benches
                .strip_suffix(" benchmarks")
                .or_else(|| benches.strip_suffix(" benchmark"));
            if let (Some(tests), Some(benches)) = (tests, benches) {
                let count = tests.parse::<usize>()?;
                if benches.parse::<usize>()? != 0 {
                    return Err("rustdoc reported an unexpected benchmark".into());
                }
                footer = Some(count);
                continue;
            }
        }
        let name = line
            .strip_suffix(": test")
            .ok_or("unrecognized rustdoc list row")?;
        let (path, _) = name
            .split_once(" - ")
            .ok_or("rustdoc row lacks its source path")?;
        let path = Path::new(path);
        let relative = if path.is_absolute() {
            path.strip_prefix(root)?
        } else {
            path
        };
        if !relative.starts_with(prefix)
            || !name.contains(" (line ")
            || !names.insert(name.to_owned())
        {
            return Err("rustdoc row is duplicated or belongs to another library".into());
        }
    }
    if footer != Some(names.len()) {
        return Err("rustdoc test count differs from its actual rows".into());
    }
    Ok(names)
}

struct Retention {
    directory: PathBuf,
    bytes: usize,
    files: usize,
}

impl Retention {
    fn fresh(root: &Path, part: &str) -> Result<Self, Fail> {
        let parent = root.join("target");
        std::fs::create_dir_all(&parent)?;
        if std::fs::symlink_metadata(&parent)?.file_type().is_symlink() {
            return Err("inventory target directory is aliased".into());
        }
        let directory = parent.join(format!("vv-inventory-{part}"));
        match std::fs::symlink_metadata(&directory) {
            Ok(_) => {
                return Err(
                    "inventory directory already exists; retain its original records separately"
                        .into(),
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        std::fs::create_dir(&directory)?;
        Ok(Self {
            directory,
            bytes: 0,
            files: 0,
        })
    }

    fn put(&mut self, name: &str, bytes: &[u8]) -> Result<(), Fail> {
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
            || bytes.len() > LIMIT
            || self.bytes + bytes.len() > TOTAL_LIMIT
            || self.files >= 1024
        {
            return Err("inventory retention bound exceeded".into());
        }
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.directory.join(name))?;
        use std::io::Write;
        file.write_all(bytes)?;
        self.bytes += bytes.len();
        self.files += 1;
        Ok(())
    }

    fn command(
        &mut self,
        label: &str,
        argv: Vec<String>,
        captured: CapturedOutput,
    ) -> Result<(Value, Vec<u8>), Fail> {
        let stdout = format!("{label}.stdout");
        let stderr = format!("{label}.stderr");
        self.put(&stdout, &captured.output.stdout)?;
        self.put(&stderr, &captured.output.stderr)?;
        let record = json!({"argv":argv,"stdout":stdout,"stderr":stderr,"status":captured.output.status.code()});
        if captured.overflow || !captured.output.status.success() {
            return Err(format!(
                "inventory command {label} failed: status={}, output_overflow={}",
                captured.output.status, captured.overflow
            )
            .into());
        }
        Ok((record, captured.output.stdout))
    }

    fn harness_command(
        &mut self,
        label: &str,
        held: &HeldHarness,
        args: &[&str],
        captured: CapturedOutput,
    ) -> Result<(Value, Vec<u8>), Fail> {
        let (mut record, bytes) = self.command(
            label,
            strings(held.original().to_str().ok_or("non-UTF8 executable")?, args),
            captured,
        )?;
        record.as_object_mut().unwrap().insert(
            "execution_path".to_owned(),
            json!(held
                .execution_path()
                .to_str()
                .ok_or("non-UTF8 descriptor route")?),
        );
        Ok((record, bytes))
    }
}

fn strings(program: &str, arguments: &[&str]) -> Vec<String> {
    std::iter::once(program.to_owned())
        .chain(arguments.iter().map(|s| (*s).to_owned()))
        .collect()
}

fn cargo_tests<'a>(selection: &[&'a str]) -> Vec<&'a str> {
    let mut arguments = vec!["test"];
    arguments.extend_from_slice(selection);
    arguments.extend(["--all-features", "--locked", "--offline"]);
    arguments
}

fn inventory(
    root: &Path,
    part: &str,
    revision: &str,
    driver: &GateDriver,
    registered: &BTreeSet<String>,
    source: &[u8],
) -> Result<(), Fail> {
    let mut retained = Retention::fresh(root, part)?;
    let args = [
        "metadata",
        "--no-deps",
        "--locked",
        "--offline",
        "--format-version",
        "1",
    ];
    let (metadata_record, bytes) = retained.command(
        "metadata",
        strings("cargo", &args),
        driver.capture_cargo(root, &args, LIMIT)?,
    )?;
    let metadata = metadata(&bytes, root)?;
    retained.put("conformance-register.rs", source)?;
    let mut cargo_records = Vec::new();
    let mut harnesses = Vec::new();
    let mut docs = Vec::new();
    let selections: Vec<(Vec<&str>, &str)> = match part {
        "source" => vec![(
            vec!["--workspace", "--exclude", "repo-conformance"],
            "source",
        )],
        "conformance" => vec![
            (
                vec!["--package", "repo-conformance", "--lib", "--bins"],
                "conformance",
            ),
            (
                vec!["--package", "repo-conformance", "--test", "conformance"],
                "integration",
            ),
        ],
        _ => vec![(
            vec!["--package", "repo-conformance", "--test", "conformance"],
            "integration",
        )],
    };
    for (index, (selection, owner)) in selections.iter().enumerate() {
        let expected = metadata
            .targets
            .values()
            .filter(|target| target.owner().ok() == Some(*owner))
            .map(Target::key)
            .collect::<BTreeSet<_>>();
        let mut args = cargo_tests(selection);
        args.extend(["--no-run", "--message-format=json"]);
        let (record, bytes) = retained.command(
            &format!("cargo-{index}"),
            strings("cargo", &args),
            driver.capture_cargo(root, &args, LIMIT)?,
        )?;
        cargo_records.push(record);
        for (target, executable) in artifacts(&bytes, &metadata, root, &expected)?.into_values() {
            let number = harnesses.len();
            let args = ["--list", "--format=terse"];
            let held = driver.hold_harness(&executable)?;
            let executable_sha256 = held.sha256();
            let (full, bytes) = retained.harness_command(
                &format!("harness-{number}-full"),
                &held,
                &args,
                driver.capture_harness(root, &held, &args, LIMIT)?,
            )?;
            let all = harness_list(&bytes)?;
            let selected_record = if *owner == "integration" {
                if &all != registered {
                    return Err(
                        "actual conformance test list differs from the source register".into(),
                    );
                }
                verify_partition(&all)?;
                let mut args = vec!["--list", "--format=terse"];
                args.extend(filters(part));
                let (record, bytes) = retained.harness_command(
                    &format!("harness-{number}-selected"),
                    &held,
                    &args,
                    driver.capture_harness(root, &held, &args, LIMIT)?,
                )?;
                let actual = harness_list(&bytes)?;
                let expected = all
                    .iter()
                    .filter(|name| selected(name, part))
                    .cloned()
                    .collect::<BTreeSet<_>>();
                if actual.is_empty() || actual != expected {
                    return Err("actual filtered test list differs from its closed part".into());
                }
                Some(record)
            } else {
                None
            };
            harnesses.push(json!({"target":target.value(),"executable_sha256":executable_sha256,"full":full,"selected":selected_record}));
        }
    }
    if matches!(part, "source" | "conformance") {
        for target in metadata
            .docs
            .values()
            .filter(|target| target.owner().ok() == Some(part))
        {
            let mut args = cargo_tests(&["--package", target.package.as_str()]);
            args.extend(["--doc", "--", "--list"]);
            let (mut record, bytes) = retained.command(
                &format!("doc-{}", docs.len()),
                strings("cargo", &args),
                driver.capture_cargo(root, &args, LIMIT)?,
            )?;
            doc_list(&bytes, target, root)?;
            record
                .as_object_mut()
                .unwrap()
                .insert("target_keys".to_owned(), json!([target.key()]));
            docs.push(record);
        }
    }
    let value = json!({"schema":"prismpm/vv-cargo-inventory/1","source_revision":revision,"part":part,"metadata":metadata_record,"cargo":cargo_records,"harnesses":harnesses,"docs":docs,"register":{"path":"conformance-register.rs","sha256":format!("{:x}",Sha256::digest(source))}});
    retained.put(
        "inventory.json",
        &prismpm::holo::canonical::encode_value(&value)?,
    )?;
    Ok(())
}

fn revision(root: &Path) -> Result<String, Fail> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err("cannot identify V&V source commit".into());
    }
    let revision = std::str::from_utf8(&output.stdout)?.trim();
    if revision.len() != 40
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("invalid V&V source commit".into());
    }
    Ok(revision.to_owned())
}

pub(crate) fn run(root: &Path, arguments: &[String]) -> Result<(), Fail> {
    let [part] = arguments else {
        return Err("usage: cargo xtask vv-part <part>".into());
    };
    if !known_part(part) {
        return Err(format!("unknown V&V part: {part}").into());
    }
    let evidence = root.join(format!("target/vv-part-{part}.json"));
    match std::fs::remove_file(&evidence) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    if part == "source" || GROUPS.contains(&part.as_str()) {
        match std::fs::symlink_metadata(root.join(format!("target/vv-inventory-{part}"))) {
            Ok(_) => {
                return Err(
                    "inventory directory already exists; a fresh part run is required".into(),
                )
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    crate::require_clean_worktree(root)?;
    let source = std::fs::read(root.join("crates/conformance/tests/conformance.rs"))?;
    let registered = register(std::str::from_utf8(&source)?)?;
    let driver = GateDriver::capture(root)?;
    let commit = revision(root)?;
    let revision = commit.as_str();
    match part.as_str() {
        "source" => {
            crate::run_vv_gates_1_to_5(root, &driver)?;
            inventory(root, part, revision, &driver, &registered, &source)?;
            driver.run_cargo(
                root,
                &cargo_tests(&["--workspace", "--exclude", "repo-conformance"]),
            )?;
        }
        "gates" => crate::run_vv_gates_7_to_15(root)?,
        _ if part.starts_with("standalone-") => crate::command(
            root,
            "node",
            &[
                "scripts/browser-api-sdk-check.mjs",
                "standalone-owner",
                part.strip_prefix("standalone-").unwrap(),
            ],
        )?,
        _ => {
            inventory(root, part, revision, &driver, &registered, &source)?;
            if part == "conformance" {
                driver.run_cargo(
                    root,
                    &cargo_tests(&["--package", "repo-conformance", "--lib", "--bins"]),
                )?;
                driver.run_cargo(
                    root,
                    &cargo_tests(&["--package", "repo-conformance", "--doc"]),
                )?;
            }
            let mut arguments =
                cargo_tests(&["--package", "repo-conformance", "--test", "conformance"]);
            arguments.push("--");
            arguments.extend(filters(part));
            driver.run_cargo(root, &arguments)?;
        }
    }
    crate::require_clean_worktree(root)?;
    driver.require_unchanged()?;
    if self::revision(root)? != revision {
        return Err("V&V source commit changed during part execution".into());
    }
    let value =
        json!({"schema":"prismpm/vv-part/1","commit":revision,"part":part,"status":"passed"});
    std::fs::create_dir_all(root.join("target"))?;
    std::fs::write(evidence, prismpm::holo::canonical::encode_value(&value)?)?;
    println!("VV part {part} PASSED for commit {revision}.");
    Ok(())
}

#[cfg(test)]
mod vv_part_tests {
    use super::*;

    fn metadata_fixture() -> Value {
        let row = |name: &str, kind: &str| json!({"name":name,"kind":[kind],"src_path":format!("/repo/{name}.rs"),"test":true,"doctest":kind=="lib"});
        json!({"workspace_members":["source-id","conformance-id"],"packages":[
            {"id":"source-id","name":"source","targets":[row("source","lib"),row("source","bin")]},
            {"id":"conformance-id","name":"repo-conformance","targets":[row("repo_conformance","lib"),row("runner","bin"),row("conformance","test")]}
        ]})
    }

    #[test]
    fn rustdoc_requires_one_actual_counted_listing_for_its_library() {
        let target = Target {
            package: "library".into(),
            name: "library".into(),
            kind: "lib".into(),
            source: "crates/library/src/lib.rs".into(),
        };
        assert!(
            doc_list(b"0 tests, 0 benchmarks\n", &target, Path::new("/repo"))
                .unwrap()
                .is_empty()
        );
        let one = b"crates/library/src/lib.rs - declaration (line 7): test\n1 test, 0 benchmarks\n";
        assert_eq!(doc_list(one, &target, Path::new("/repo")).unwrap().len(), 1);
        for bytes in [
            b"".as_slice(),
            b"1 test, 0 benchmarks\n",
            b"0 tests, 0 benchmarks\n0 tests, 0 benchmarks\n",
            b"crates/foreign/src/lib.rs - declaration (line 7): test\n1 test, 0 benchmarks\n",
            b"crates/library/src/lib.rs - declaration (line 7): test\n0 tests, 0 benchmarks\n",
        ] {
            assert!(doc_list(bytes, &target, Path::new("/repo")).is_err());
        }
    }

    #[test]
    fn stale_inventory_is_preserved_and_cannot_reuse_a_previous_part_receipt() {
        let root = tempfile::tempdir().unwrap();
        let mut retained = Retention::fresh(root.path(), "source").unwrap();
        retained
            .put("original.stdout", b"original diagnostic")
            .unwrap();
        assert!(Retention::fresh(root.path(), "source").is_err());
        assert_eq!(
            std::fs::read(retained.directory.join("original.stdout")).unwrap(),
            b"original diagnostic"
        );
        let receipt = root.path().join("target/vv-part-source.json");
        std::fs::write(&receipt, b"previous passed record").unwrap();
        assert!(run(root.path(), &["source".into()]).is_err());
        assert!(!receipt.exists());
        assert!(retained.put("../escape", b"x").is_err());
        retained.bytes = TOTAL_LIMIT;
        assert!(retained.put("overflow.stdout", b"x").is_err());
        retained.bytes = 0;
        retained.files = 1024;
        assert!(retained.put("extra.stdout", b"x").is_err());
    }

    fn artifact_fixture(metadata: &Metadata, expected: &BTreeSet<String>) -> Vec<u8> {
        let mut rows = Vec::new();
        for key in expected {
            let target = &metadata.targets[key];
            let package = metadata
                .packages
                .iter()
                .find(|(_, name)| *name == &target.package)
                .unwrap()
                .0;
            rows.push(json!({"reason":"compiler-artifact","package_id":package,"target":{"name":target.name,"kind":[target.kind],"src_path":format!("/repo/{}",target.source)},"profile":{"test":true},"executable":format!("/repo/target/{key}")}));
        }
        rows.push(json!({"reason":"build-finished","success":true}));
        rows.iter()
            .flat_map(|row| {
                let mut bytes = serde_json::to_vec(row).unwrap();
                bytes.push(b'\n');
                bytes
            })
            .collect()
    }

    #[test]
    fn cargo_target_and_doc_owners_form_an_exact_disjoint_union() {
        let data = metadata(
            &serde_json::to_vec(&metadata_fixture()).unwrap(),
            Path::new("/repo"),
        )
        .unwrap();
        let expected = data.targets.keys().cloned().collect();
        let selections = ["source", "conformance", "integration"]
            .into_iter()
            .map(|owner| {
                (
                    owner.to_owned(),
                    data.targets
                        .values()
                        .filter(|target| target.owner().unwrap() == owner)
                        .map(Target::key)
                        .collect(),
                )
            })
            .collect();
        verify_union(&expected, &selections).unwrap();
        let docs = data.docs.keys().cloned().collect();
        let selections = ["source", "conformance"]
            .into_iter()
            .map(|owner| {
                (
                    owner.to_owned(),
                    data.docs
                        .values()
                        .filter(|target| target.owner().unwrap() == owner)
                        .map(Target::key)
                        .collect(),
                )
            })
            .collect();
        verify_union(&docs, &selections).unwrap();
        let mut duplicate = metadata_fixture();
        let row = duplicate["packages"][0]["targets"][0].clone();
        duplicate["packages"][0]["targets"]
            .as_array_mut()
            .unwrap()
            .push(row);
        assert!(metadata(&serde_json::to_vec(&duplicate).unwrap(), Path::new("/repo")).is_err());
        let mut unknown = metadata_fixture();
        unknown["packages"][1]["targets"][2]["name"] = json!("unowned-integration");
        assert!(metadata(&serde_json::to_vec(&unknown).unwrap(), Path::new("/repo")).is_err());
        let mut bad_doc = metadata_fixture();
        bad_doc["packages"][0]["targets"][1]["doctest"] = json!(true);
        assert!(metadata(&serde_json::to_vec(&bad_doc).unwrap(), Path::new("/repo")).is_err());
    }

    #[test]
    fn actual_cargo_artifacts_reject_missing_extra_duplicate_or_failed_targets() {
        let data = metadata(
            &serde_json::to_vec(&metadata_fixture()).unwrap(),
            Path::new("/repo"),
        )
        .unwrap();
        let expected = data.targets.keys().cloned().collect::<BTreeSet<_>>();
        let bytes = artifact_fixture(&data, &expected);
        assert_eq!(
            artifacts(&bytes, &data, Path::new("/repo"), &expected)
                .unwrap()
                .len(),
            expected.len()
        );
        let mut smaller = expected.clone();
        smaller.pop_first();
        assert!(artifacts(&bytes, &data, Path::new("/repo"), &smaller).is_err());
        assert!(artifacts(
            &artifact_fixture(&data, &smaller),
            &data,
            Path::new("/repo"),
            &expected
        )
        .is_err());
        let first = bytes.split(|byte| *byte == b'\n').next().unwrap();
        let mut duplicate = first.to_vec();
        duplicate.push(b'\n');
        duplicate.extend_from_slice(&bytes);
        assert!(artifacts(&duplicate, &data, Path::new("/repo"), &expected).is_err());
        let failed = String::from_utf8(bytes.clone())
            .unwrap()
            .replace("\"success\":true", "\"success\":false");
        assert!(artifacts(failed.as_bytes(), &data, Path::new("/repo"), &expected).is_err());
        assert!(artifacts(&[], &data, Path::new("/repo"), &expected).is_err());
    }

    #[test]
    fn main_register_is_exhaustive_and_dk_40_and_41_have_a_closed_owner() {
        let mut names = register(include_str!(
            "../../crates/conformance/tests/conformance.rs"
        ))
        .unwrap();
        assert!(names.contains("conformance_rp_08"));
        names.insert("conformance_dk_36".to_owned());
        assert!(selected("conformance_dk_40", "dk-4"));
        names.insert("conformance_dk_41".to_owned());
        verify_partition(&names).unwrap();
        assert!(selected("conformance_dk_41", "dk-4"));
        names.insert("conformance_dk_42".to_owned());
        assert!(verify_partition(&names).is_err());
    }

    #[test]
    fn unknown_parts_empty_and_overlapping_selections_fail_closed() {
        assert!(!known_part("dk-42"));
        assert!(!known_part("standalone-unknown"));
        assert!(!known_part(""));
        let root = tempfile::tempdir().unwrap();
        assert!(run(root.path(), &[]).is_err());
        assert!(run(root.path(), &["unknown".into()]).is_err());
        assert!(!root.path().join("target").exists());
        assert!(register("").is_err());
        assert!(harness_list(b"same: test\nsame: test\n").is_err());
        let names = ["one".to_owned(), "two".to_owned()].into();
        assert!(verify_union(
            &names,
            &BTreeMap::from([
                ("first".to_owned(), BTreeSet::from(["one".to_owned()])),
                (
                    "second".to_owned(),
                    BTreeSet::from(["one".to_owned(), "two".to_owned()])
                )
            ])
        )
        .is_err());
        assert!(verify_union(
            &names,
            &BTreeMap::from([("empty".to_owned(), BTreeSet::new())])
        )
        .is_err());
    }

    #[test]
    fn harness_lists_preserve_names_and_reject_unknown_rows() {
        assert_eq!(
            harness_list(b"module::test: test\nother: benchmark\n").unwrap(),
            BTreeSet::from(["module::test".to_owned(), "other".to_owned()])
        );
        assert!(harness_list(b"passed\n").is_err());
        assert!(verify_union(
            &BTreeSet::from(["one".to_owned()]),
            &BTreeMap::from([("part".to_owned(), BTreeSet::from(["other".to_owned()]))])
        )
        .is_err());
    }

    #[test]
    fn residual_commands_retain_library_binary_and_documentation_targets() {
        assert_eq!(
            cargo_tests(&["--package", "repo-conformance", "--lib", "--bins"]),
            [
                "test",
                "--package",
                "repo-conformance",
                "--lib",
                "--bins",
                "--all-features",
                "--locked",
                "--offline"
            ]
        );
        assert_eq!(
            cargo_tests(&["--package", "repo-conformance", "--doc"]),
            [
                "test",
                "--package",
                "repo-conformance",
                "--doc",
                "--all-features",
                "--locked",
                "--offline"
            ]
        );
        assert_eq!(GROUPS.len() + STANDALONE.len() + 2, 15);
        assert_eq!(
            Target {
                package: "repo-conformance".into(),
                kind: "test".into(),
                name: "foreign".into(),
                source: "x.rs".into()
            }
            .owner()
            .unwrap_err()
            .to_string(),
            "unowned Cargo target repo-conformance/test/foreign"
        );
    }
}
