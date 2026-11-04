//! Shared source-owned native core generation, completed by each package profile.

use crate::error::PrismError;
use crate::holo::canonical::content_id;
use crate::holo::model_document::ModelDocument;
use crate::verification::{executable, run_process, ProcessRecord};
use prod_codegen::{generate_cargo_package, CargoPackageSpec};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

// The pinned generator escapes type/field names but emits function names
// literally. Reject unsupported target spellings instead of guessing an alias.
const FUNCTION_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

pub(crate) mod binary;

pub(crate) type LibraryArtifact = (String, Vec<u8>);

pub(crate) struct GeneratedLibrary {
    pub(crate) artifacts: Vec<LibraryArtifact>,
    pub(crate) processes: Vec<ProcessRecord>,
}

impl crate::exporter::ProductEvidence for GeneratedLibrary {
    fn role(&self) -> &'static str {
        "library"
    }
    fn artifacts(&self) -> &[(String, Vec<u8>)] {
        &self.artifacts
    }
    fn processes(&self) -> &[ProcessRecord] {
        &self.processes
    }
}

pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn write(path: &Path, bytes: &[u8]) -> Result<(), PrismError> {
    let parent = path
        .parent()
        .ok_or_else(|| PrismError::new("PP4002", "native library artifact has no parent"))?;
    std::fs::create_dir_all(parent)
        .and_then(|()| std::fs::write(path, bytes))
        .map_err(|error| PrismError::new("PP4002", format!("{}: {error}", path.display())))
}

pub(crate) fn relative(path: &str) -> Result<(), PrismError> {
    if path.is_empty()
        || path.contains('\\')
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(PrismError::new(
            "PP4004",
            "native library artifact path is not confined",
        ));
    }
    Ok(())
}

pub(crate) fn regular_bytes(root: &Path, path: &str) -> Result<Vec<u8>, PrismError> {
    relative(path)?;
    if root.canonicalize().ok().as_deref() != Some(root) {
        return Err(PrismError::new(
            "PP4001",
            "native library input root contains a symlink",
        ));
    }
    let mut selected = root.to_path_buf();
    let components = Path::new(path).components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        selected.push(component);
        let metadata = std::fs::symlink_metadata(&selected)
            .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
        let expected = if index + 1 == components.len() {
            metadata.file_type().is_file()
        } else {
            metadata.file_type().is_dir()
        };
        if !expected {
            return Err(PrismError::new(
                "PP4001",
                "native library input contains a symlink or special file",
            ));
        }
    }
    std::fs::read(selected).map_err(|error| PrismError::new("PP4002", error.to_string()))
}

pub(crate) fn validate_export_identity(
    coverage: &Value,
    roots: &Value,
    module: &prod_ir::Module,
    exports: &[String],
    acceptance: &[String],
) -> Result<(), PrismError> {
    crate::verification::validate_coverage(coverage, exports)?;
    if *roots
        != json!({
            "erased_proof_dependencies":coverage["erased_proof_dependencies"],
            "included_definitions":coverage["included_definitions"],
            "requested_roots":coverage["requested_roots"]
        })
    {
        return Err(PrismError::new(
            "PP5004",
            "native named roots differ from export coverage",
        ));
    }
    let mut names = BTreeSet::new();
    for name in coverage["included_definitions"]
        .as_array()
        .expect("validated coverage")
    {
        let full = name.as_str().expect("validated coverage string");
        let leaf = full.rsplit('.').next().unwrap_or_default();
        // Pinned Prod.Lower.lastComponent strips namespaces. Bind its complete
        // closure to actual IR names before emitting or invoking Rust symbols.
        if leaf.is_empty()
            || !(leaf.as_bytes()[0].is_ascii_alphabetic() || leaf.starts_with('_'))
            || !leaf
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            || leaf == "_"
            || FUNCTION_KEYWORDS.contains(&leaf)
            || !names.insert(leaf.to_owned())
        {
            return Err(PrismError::new(
                "PP5004",
                "native export names collide or cannot map exactly to Rust",
            ));
        }
    }
    let ir_names = module
        .definitions
        .iter()
        .map(|definition| definition.name.clone())
        .collect::<BTreeSet<_>>();
    if ir_names.len() != module.definitions.len() || names != ir_names {
        return Err(PrismError::new(
            "PP5004",
            "native IR definitions differ from the complete named export closure",
        ));
    }
    for root in acceptance {
        let leaf = root.rsplit('.').next().expect("validated root");
        let definition = module
            .find_def(leaf)
            .ok_or_else(|| PrismError::new("PP5004", "native acceptance is absent from IR"))?;
        if !definition.params.is_empty() || definition.ret != prod_ir::Type::Bool {
            return Err(PrismError::new(
                "PP5004",
                "native acceptance IR signature is not a closed Boolean",
            ));
        }
    }
    Ok(())
}

/// Generate unaccepted native artifacts; verification owns acceptance evidence.
pub(crate) fn export_arguments(
    modules: &BTreeSet<String>,
    library: &crate::holo::model_document::ModelLibrary,
    export: &Path,
) -> Vec<String> {
    let mut arguments = vec!["exe".to_owned(), "prod-export".to_owned()];
    for module in modules {
        arguments.extend(["--module".to_owned(), module.clone()]);
    }
    for root in &library.export_roots {
        arguments.extend(["--root".to_owned(), root.clone()]);
    }
    arguments.extend([
        "--ir-module".to_owned(),
        library.cargo_name.replace('-', "_"),
        "--out".to_owned(),
        export.to_string_lossy().into_owned(),
    ]);
    arguments
}

/// Generate unaccepted native artifacts; verification owns acceptance evidence.
pub(crate) fn generate(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
) -> Result<Vec<LibraryArtifact>, PrismError> {
    generate_recorded(
        repository_root,
        model,
        model_bytes,
        lex_root,
        lex_manifest_bytes,
    )
    .map(|generated| generated.artifacts)
}

pub(crate) fn generate_recorded(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
) -> Result<GeneratedLibrary, PrismError> {
    generate_in(
        repository_root,
        model,
        model_bytes,
        lex_root,
        lex_manifest_bytes,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_owned(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
    owner: &mut crate::exporter::VerifyExporterOwner,
    phase: crate::exporter::Phase,
) -> Result<GeneratedLibrary, PrismError> {
    owner.generate_library(
        repository_root,
        phase,
        model_bytes,
        lex_manifest_bytes,
        |package, acquisition| {
            generate_in(
                repository_root,
                model,
                model_bytes,
                lex_root,
                lex_manifest_bytes,
                Some((package, acquisition)),
            )
        },
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeCoreProfile {
    Library,
    Binary,
}

impl NativeCoreProfile {
    fn directory_prefix(self) -> &'static str {
        match self {
            Self::Library => "prismpm-native-library-",
            Self::Binary => "prismpm-binary-program-",
        }
    }
    fn lake_name(self) -> &'static str {
        match self {
            Self::Library => "prismpm_native_library",
            Self::Binary => "prismpm_binary_program",
        }
    }
    fn work_label(self) -> &'static str {
        match self {
            Self::Library => "$LIBRARY_WORK",
            Self::Binary => "$BINARY_WORK",
        }
    }
    fn artifact_prefix(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Binary => "binary",
        }
    }
    fn package_tool(self) -> &'static str {
        match self {
            Self::Library => "native-library-package",
            Self::Binary => "binary-program-package",
        }
    }
    fn archive_context(self) -> &'static str {
        match self {
            Self::Library => "native library",
            Self::Binary => "binary program",
        }
    }
    fn acceptance_roots(self, library: &crate::holo::model_document::ModelLibrary) -> &[String] {
        match self {
            Self::Library => &library.acceptance_roots,
            Self::Binary => &[],
        }
    }
}

struct NativeCoreRequest<'a, 'context> {
    profile: NativeCoreProfile,
    repository_root: &'a Path,
    library: &'a crate::holo::model_document::ModelLibrary,
    lex_root: &'a Path,
    lex_manifest_bytes: &'a [u8],
    owned: Option<(&'a Path, &'a crate::exporter::GenerationContext<'context>)>,
}

// This is an unfinished core, deliberately without ProductEvidence. Its owner
// keeps the workspace and original exporter custody alive through completion.
struct CoreParts<'a> {
    workspace: &'a Path,
    replacements: &'a [(&'a Path, &'static str)],
    module: prod_ir::Module,
    kernel_sha256: String,
    archive_name: String,
    artifacts: Vec<LibraryArtifact>,
    processes: Vec<ProcessRecord>,
}

fn with_generated_core<T: crate::exporter::ProductEvidence>(
    request: NativeCoreRequest<'_, '_>,
    validate_module: impl FnOnce(&prod_ir::Module) -> Result<(), PrismError>,
    complete: impl FnOnce(CoreParts<'_>) -> Result<T, PrismError>,
) -> Result<T, PrismError> {
    let NativeCoreRequest {
        profile,
        repository_root,
        library,
        lex_root,
        lex_manifest_bytes,
        owned,
    } = request;
    let mut work = crate::exporter::directory::Directory::temporary(
        profile.directory_prefix(),
        None,
        Default::default(),
    )?;
    let workspace_path = work.path().to_owned();
    let workspace = workspace_path.as_path();
    let mut seed_custody = None;
    let (lean_package, exporter_acquisition) = match owned {
        Some((package, acquisition)) => (package.to_owned(), acquisition.receipt().clone()),
        None => {
            let package = workspace.join("lean4-prod");
            let acquisition =
                crate::exporter::acquire_for_owned(repository_root, &package, work.scope())?;
            work.protect_package(&package, &acquisition.snapshot)?;
            seed_custody = Some(acquisition.snapshot);
            (package, acquisition.receipt)
        }
    };

    let manifest: Value = serde_json::from_slice(lex_manifest_bytes)
        .map_err(|error| PrismError::new("PP4004", format!("LexLean manifest: {error}")))?;
    let outputs = manifest["outputs"]
        .as_array()
        .ok_or_else(|| PrismError::new("PP4004", "LexLean output manifest is absent"))?;
    let mut modules = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut generated = Vec::new();
    for output in outputs {
        if output["kind"] != "lean" {
            continue;
        }
        let path = output["path"]
            .as_str()
            .ok_or_else(|| PrismError::new("PP4004", "generated Lean path is absent"))?;
        relative(path)?;
        if !paths.insert(path) {
            return Err(PrismError::new(
                "PP4004",
                "generated Lean path is duplicated",
            ));
        }
        let local = Path::new(path)
            .strip_prefix("modules")
            .map_err(|_| PrismError::new("PP4004", "generated Lean is outside modules"))?;
        if local.extension().and_then(|extension| extension.to_str()) != Some("lean") {
            return Err(PrismError::new(
                "PP4004",
                "generated Lean extension differs",
            ));
        }
        let bytes = regular_bytes(lex_root, path)?;
        if output["sha256"].as_str() != Some(sha256(&bytes).as_str())
            || output["byte_length"].as_u64() != Some(bytes.len() as u64)
        {
            return Err(PrismError::new(
                "PP4001",
                "generated Lean differs from its manifest",
            ));
        }
        write(&workspace.join(local), &bytes)?;
        generated.push(bytes);
        modules.insert(local.with_extension("").to_string_lossy().replace('/', "."));
    }
    if modules.is_empty() {
        return Err(PrismError::new(
            "PP4004",
            "library has no generated Lean modules",
        ));
    }
    let runtime =
        crate::lean_project::stage_runtime(workspace, generated.iter().map(Vec::as_slice))?;
    write(
        &workspace.join("lakefile.toml"),
        crate::lean_project::configuration(
            profile.lake_name(),
            &crate::lean_project::roots(&runtime, &modules.iter().cloned().collect::<Vec<_>>()),
        )
        .as_bytes(),
    )?;
    write(
        &workspace.join("lean-toolchain"),
        b"leanprover/lean4:v4.32.1\n",
    )?;
    let lake = executable("lake")?;
    let replacements = [
        (lean_package.as_path(), "$LEAN4_PROD"),
        (workspace, profile.work_label()),
        (repository_root, "$PROJECT"),
    ];
    let no_env = BTreeMap::new();
    let mut processes = Vec::new();
    for (tool, directory, args) in [
        (
            "lake-build-generated",
            workspace,
            vec!["build".to_owned(), "PrismGenerated".to_owned()],
        ),
        (
            "lean4-prod-build",
            lean_package.as_path(),
            vec!["build".to_owned(), "prod-export".to_owned()],
        ),
    ] {
        if tool == "lean4-prod-build" {
            if let Some((_, context)) = owned {
                context.before_build(repository_root, &lean_package)?;
            } else {
                crate::exporter::before_first_build(
                    repository_root,
                    &lean_package,
                    &exporter_acquisition,
                    seed_custody
                        .as_ref()
                        .expect("first build retains original custody"),
                )?;
            }
        }
        processes.push(match owned {
            Some((_, context)) if profile == NativeCoreProfile::Library => context.run_library(
                tool,
                &lake,
                &args,
                directory,
                &no_env,
                &replacements,
                "PP5004",
            ),
            _ => run_process(
                tool,
                &lake,
                &args,
                directory,
                &no_env,
                &replacements,
                "PP5004",
            ),
        }?);
    }
    let export = workspace.join("export");
    let export_custody = match owned {
        Some((_, context)) => context.after_build(&lean_package)?,
        None => crate::exporter::after_build(
            &lean_package,
            &exporter_acquisition,
            seed_custody.as_ref().expect("original unowned custody"),
        )?,
    };
    let arguments = export_arguments(&modules, library, &export);
    let export_env = BTreeMap::from([(
        "LEAN_PATH".to_owned(),
        workspace
            .join(".lake/build/lib/lean")
            .to_string_lossy()
            .into_owned(),
    )]);
    processes.push(match owned {
        Some((_, context)) if profile == NativeCoreProfile::Library => context.run_library(
            "prod-export",
            &lake,
            &arguments,
            &lean_package,
            &export_env,
            &replacements,
            "PP5004",
        ),
        _ => crate::exporter::run_export_with_custody(
            "prod-export",
            &lake,
            &arguments,
            &lean_package,
            &export_env,
            &replacements,
            "PP5004",
            &exporter_acquisition,
            &export_custody,
        ),
    }?);
    let mut artifacts = Vec::new();
    for name in ["coverage.json", "kernel.ir", "roots.json"] {
        let bytes = std::fs::read(export.join(name))
            .map_err(|error| PrismError::new("PP5004", format!("{name}: {error}")))?;
        artifacts.push((format!("{}/{name}", profile.artifact_prefix()), bytes));
    }
    let kernel = &artifacts
        .iter()
        .find(|(path, _)| path == &format!("{}/kernel.ir", profile.artifact_prefix()))
        .expect("kernel recorded")
        .1;
    let text = std::str::from_utf8(kernel)
        .map_err(|error| PrismError::new("PP5004", error.to_string()))?;
    let module = crate::verification::parse_kernel(text)?;
    let document = |path: &str| -> Result<Value, PrismError> {
        serde_json::from_slice(
            &artifacts
                .iter()
                .find(|(name, _)| name == path)
                .expect("export recorded")
                .1,
        )
        .map_err(|error| PrismError::new("PP5004", error.to_string()))
    };
    validate_export_identity(
        &document(&format!("{}/coverage.json", profile.artifact_prefix()))?,
        &document(&format!("{}/roots.json", profile.artifact_prefix()))?,
        &module,
        &library.export_roots,
        profile.acceptance_roots(library),
    )?;
    validate_module(&module)?;
    let kernel_sha256 = sha256(kernel);
    let package = generate_cargo_package(
        &module,
        &CargoPackageSpec {
            name: library.cargo_name.clone(),
            version: library.cargo_version.clone(),
            description: library.cargo_description.clone(),
            repository: library.cargo_repository.clone(),
            homepage: library.cargo_homepage.clone(),
            readme: format!("# {}\n\n{}\n", library.name, library.cargo_description),
            license_mit: include_str!("../LICENSE-MIT").to_owned(),
            license_apache: include_str!("../LICENSE-APACHE").to_owned(),
            input_sha256: sha256(kernel),
            dependencies: Vec::new(),
        },
    )
    .map_err(|error| PrismError::new("PP4102", error.to_string()))?;
    let package_root = workspace.join("package");
    for file in package.files {
        relative(&file.path)?;
        write(&package_root.join(&file.path), &file.bytes)?;
        artifacts.push((
            format!("{}/package/{}", profile.artifact_prefix(), file.path),
            file.bytes,
        ));
    }
    let cargo_home = workspace.join("cargo-home");
    std::fs::create_dir(&cargo_home)
        .map_err(|error| PrismError::new("PP4002", error.to_string()))?;
    let cargo_env = BTreeMap::from([
        (
            "CARGO_HOME".to_owned(),
            cargo_home.to_string_lossy().into_owned(),
        ),
        ("CARGO_NET_OFFLINE".to_owned(), "true".to_owned()),
    ]);
    let cargo = executable("cargo")?;
    let package_arguments =
        ["package", "--locked", "--offline", "--allow-dirty"].map(str::to_owned);
    processes.push(match owned {
        Some((_, context)) if profile == NativeCoreProfile::Library => context.run_library(
            profile.package_tool(),
            &cargo,
            &package_arguments,
            &package_root,
            &cargo_env,
            &replacements,
            "PP4102",
        ),
        _ => run_process(
            profile.package_tool(),
            &cargo,
            &package_arguments,
            &package_root,
            &cargo_env,
            &replacements,
            "PP4102",
        ),
    }?);
    let archive_name = format!("{}-{}.crate", library.cargo_name, library.cargo_version);
    let crate_bytes = std::fs::read(package_root.join("target/package").join(&archive_name))
        .map_err(|error| {
            PrismError::new(
                "PP4102",
                format!("{} archive: {error}", profile.archive_context()),
            )
        })?;
    artifacts.push((
        format!("{}/{archive_name}", profile.artifact_prefix()),
        crate_bytes,
    ));
    let result = complete(CoreParts {
        workspace,
        replacements: &replacements,
        module,
        kernel_sha256,
        archive_name,
        artifacts,
        processes,
    })?;
    if seed_custody.is_some() {
        work.protect_verified_package(&lean_package, &export_custody)?;
    }
    work.close()?;
    Ok(result)
}

fn generate_in(
    repository_root: &Path,
    model: &ModelDocument,
    model_bytes: &[u8],
    lex_root: &Path,
    lex_manifest_bytes: &[u8],
    owned: Option<(&Path, &crate::exporter::GenerationContext<'_>)>,
) -> Result<GeneratedLibrary, PrismError> {
    let library = model
        .library
        .as_ref()
        .ok_or_else(|| PrismError::new("PP9001", "native generation requires a library profile"))?;
    with_generated_core(
        NativeCoreRequest {
            profile: NativeCoreProfile::Library,
            repository_root,
            library,
            lex_root,
            lex_manifest_bytes,
            owned,
        },
        |_| Ok(()),
        |core| {
            let CoreParts {
                mut artifacts,
                processes,
                ..
            } = core;
            let binding = crate::contracts::CanonicalDocument::from_value(
                "prismpm/library-build-binding/1",
                json!({
                    "acceptance_roots": library.acceptance_roots, "export_roots": library.export_roots,
                    "model_id": content_id(model_bytes), "profile": library.profile,
                    "schema": "prismpm/library-build-binding/1", "scope": "native-library-only"
                }),
            )?;
            artifacts.push((
                "library/model-binding.json".to_owned(),
                binding.bytes().to_vec(),
            ));
            artifacts.sort_by(|left, right| left.0.cmp(&right.0));
            Ok(GeneratedLibrary {
                artifacts,
                processes,
            })
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{regular_bytes, validate_export_identity};

    #[test]
    fn unfinished_core_cannot_be_submitted_as_product_evidence() {
        // A successful shared core still lacks profile completion. If it ever
        // implements ProductEvidence, inference becomes ambiguous here.
        trait AmbiguousIfEvidence<A> {
            fn check() {}
        }
        impl<T: ?Sized> AmbiguousIfEvidence<()> for T {}
        struct EvidenceMarker;
        impl<T: ?Sized + crate::exporter::ProductEvidence> AmbiguousIfEvidence<EvidenceMarker> for T {}
        let _ = <super::CoreParts<'static> as AmbiguousIfEvidence<_>>::check;
    }
    use prod_ir::{Definition, Expr, Module, Type};
    use serde_json::{json, Value};

    fn fixture(names: &[&str]) -> (Value, Value, Module, Vec<String>) {
        let roots = names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>();
        let coverage = json!({
            "requested_roots":roots,"included_definitions":roots,"erased_proof_dependencies":[],
            "external_calls":[],"opaque_nodes":[],"unsupported_types":[]
        });
        let roots_document = json!({"requested_roots":roots,"included_definitions":roots,"erased_proof_dependencies":[]});
        let module = Module {
            name: "probe".to_owned(),
            types: vec![],
            definitions: names
                .iter()
                .map(|name| Definition {
                    name: name.rsplit('.').next().unwrap().to_owned(),
                    params: vec![],
                    ret: Type::Bool,
                    body: Expr::Bool(true),
                })
                .collect(),
        };
        (coverage, roots_document, module, roots)
    }

    #[test]
    fn native_export_mapping_binds_real_ir_signatures_and_rejects_symbol_collisions() {
        let (coverage, roots_document, module, roots) = fixture(&["Owner.acceptance"]);
        validate_export_identity(&coverage, &roots_document, &module, &roots, &roots).unwrap();
        for mutation in 0..6 {
            let mut changed = module.clone();
            match mutation {
                0 => {
                    changed.definitions.pop();
                }
                1 => {
                    let mut extra = changed.definitions[0].clone();
                    extra.name = "extra".to_owned();
                    changed.definitions.push(extra);
                }
                2 => {
                    changed.definitions.push(changed.definitions[0].clone());
                }
                3 => changed.definitions[0].ret = Type::Nat,
                4 => changed.definitions[0].params = vec![("unmodeled".to_owned(), Type::Nat)],
                _ => changed.definitions[0].name = "substituted".to_owned(),
            }
            assert_eq!(
                validate_export_identity(&coverage, &roots_document, &changed, &roots, &roots)
                    .unwrap_err()
                    .code,
                "PP5004"
            );
        }
        for names in [
            vec!["First.same", "Second.same"],
            vec!["Owner.quoted'"],
            vec!["Owner.self"],
            vec!["Owner.crate"],
            vec!["Owner._"],
        ] {
            let (coverage, roots_document, module, roots) = fixture(&names);
            assert_eq!(
                validate_export_identity(&coverage, &roots_document, &module, &roots, &roots)
                    .unwrap_err()
                    .code,
                "PP5004"
            );
        }
        let (coverage, roots_document, module, roots) = fixture(&["Owner.match"]);
        assert!(prod_codegen::generate_module(&module)
            .unwrap()
            .contains("pub fn r#match("));
        assert_eq!(
            validate_export_identity(&coverage, &roots_document, &module, &roots, &roots)
                .unwrap_err()
                .code,
            "PP5004"
        );
    }

    #[test]
    fn native_named_roots_and_coverage_cannot_disagree_or_hide_gaps() {
        let (coverage, roots_document, module, roots) = fixture(&["Owner.acceptance"]);
        for field in [
            "requested_roots",
            "included_definitions",
            "erased_proof_dependencies",
        ] {
            let mut changed = roots_document.clone();
            changed[field] = json!(["Substituted.root"]);
            assert_eq!(
                validate_export_identity(&coverage, &changed, &module, &roots, &roots)
                    .unwrap_err()
                    .code,
                "PP5004"
            );
        }
        for field in [
            "external_calls",
            "opaque_nodes",
            "unsupported_types",
            "erased_proof_dependencies",
        ] {
            let mut changed = coverage.clone();
            changed[field] = json!(["undeclared"]);
            assert_eq!(
                validate_export_identity(&changed, &roots_document, &module, &roots, &roots)
                    .unwrap_err()
                    .code,
                "PP5004"
            );
        }
    }

    #[test]
    #[cfg(unix)]
    fn native_export_inputs_reject_symlinked_roots_directories_files_and_escape() {
        use std::os::unix::fs::symlink;
        let work = tempfile::tempdir().unwrap();
        let root = work.path().join("input");
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("source.lean"), b"generated input").unwrap();
        assert_eq!(
            regular_bytes(&root, "source.lean").unwrap(),
            b"generated input"
        );
        symlink(root.join("source.lean"), root.join("file-link.lean")).unwrap();
        symlink(&root, root.join("directory-link")).unwrap();
        symlink(&root, work.path().join("root-link")).unwrap();
        for path in ["file-link.lean", "directory-link/source.lean"] {
            assert_eq!(regular_bytes(&root, path).unwrap_err().code, "PP4001");
        }
        assert_eq!(
            regular_bytes(&work.path().join("root-link"), "source.lean")
                .unwrap_err()
                .code,
            "PP4001"
        );
        assert_eq!(
            regular_bytes(&root, "../outside.lean").unwrap_err().code,
            "PP4004"
        );
    }
}
