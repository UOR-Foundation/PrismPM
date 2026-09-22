# PrismPM

PrismPM compiles authoritative Prism models into verified software artifacts. A
model is a closed `.lex.tex` source graph: LexLean produces its semantic
snapshot and Lean, Lean checks the proofs, `lean4-prod` exports only named
verified roots, and generic generators produce Cargo, Core-Wasm, View, browser,
and Hologram artifacts. Prism application behavior is never supplied by
handwritten Lean or target-specific application code.

The historical `v0.1.0` release is a systems-modeling prototype and `v0.2.0`
is the completed portable-application line. The production-system SDK is
`v0.3.0`, and it is releasable only when the generated
`prism-stdlib` and `prism-calculator` crates, `Calculator.holo`, the independent
Hologram execution evidence, and the public `calculator-example` Pages
application and complete CalculatorSystem reference all pass the atomic release
contract in [SPEC.md](SPEC.md).

Version 0.3.0 is the accepted production-system release. [Release status](RELEASE-STATUS.md) records
the completed public dependency and acceptance closure across all six release steps.

## Artifact model

```text
authoritative .lex.tex + lock
             |
             +-- LexLean snapshot, generated Lean/LaTeX, proof evidence
             +-- lean4-prod LCNF and named-root coverage
             +-- generated Cargo crate and registry package
             +-- import-free hologram:guest/core-wasm@1 guest
             +-- evaluated View -> portable HOLOVIEW + browser adapter/assets
             `-- binary Hologram v4 ApplicationName.holo
```

Holo/1 is the Prism profile defined by `prism-stdlib`; its physical container
is Hologram archive version 4. Every `.holo` begins with `HOLO\x04\x00`.
`model.prism.json` is the separate canonical Prism model document. JSON is
never accepted as a `.holo` archive.

## Calculator: getting started

Open this repository in its VS Code devcontainer. From the container shell:

```sh
cargo run --locked --offline -p prismpm -- \
  --project examples/Calculator check
cargo run --locked --offline -p prismpm -- \
  --project examples/Calculator build
cargo run --locked --offline -p prismpm -- \
  --project examples/Calculator verify
```

The project root is [examples/Calculator](examples/Calculator). Its sole
application authority is `src/Calculator.lex.tex` plus `lexlean.lock`.
Successful verification writes a content-addressed build beneath
`examples/Calculator/.prism/build/` and an acceptance result beneath
`examples/Calculator/.prism/verified/`. The build contains:

- `Calculator.holo`, the composed Core-Wasm plus portable View application;
- `cargo/prism-calculator-0.1.0.crate` and its complete generated package;
- `core-wasm/prism_calculator_core_wasm.wasm`;
- generated Hologram and browser View artifacts; and
- the model, Lean, LCNF, provenance, identity, and manifest evidence.

Inspect and headlessly plan the archive with the pinned Hologram Live binary:

```sh
hologram --json holo inspect path/to/Calculator.holo
hologram --json holo plan path/to/Calculator.holo
```

The ordinary headless plan intentionally reports that the `portable` View
surface is unavailable. `prismpm verify` also builds the pinned independent
Hologram oracle and opens the same archive with a display-independent portable
surface, runs every modeled request directly and through View intents, then
checks detach and idempotent shutdown.

After publication, ordinary Rust consumers use the generated API:

```toml
[dependencies]
prism-calculator = "=0.1.0"
```

```rust
use prism_calculator::{calculate, Operation};

assert_eq!(calculate(Operation::Add, 20, 22), Ok(42));
```

The public reference application is
[`UOR-Foundation/calculator-example`](https://github.com/UOR-Foundation/calculator-example).
It mirrors the exact content-addressed model, reruns PrismPM acceptance, imports
the registry crate, and deploys only the generated six-file browser closure.

## Verified Acceptance Capabilities (v0.3.0)

The following acceptance tasks are complete and verified:

| Task | Description | Verification |
|------|-------------|--------------|
| **Gate 15** | Full `cargo xtask package-api` pass with `prism-stdlib`, `prod-ir`, `prod-codegen` | `target/vv-evidence.json` |
| **Independent Hologram Oracle** | Calculator (8 legacy numeric + 10 UTF-8 text cases) and Text interoperability under `prismpm/hologram-oracle/2` | `prismpm/holo-oracle-acceptance/1` |
| **Reproducible SDK** | Multi-platform `linux/amd64` + `linux/arm64` with canonical inventories, lockfiles, bootstrap verification | `prismpm/bootstrap-evidence/2` |
| **OCI Product-Release Graph** | Complete SBOM, provenance, signatures, vulnerability, license, deployment referrers; attestation-gated promotion | `prismpm/sdk-security-disposition/1` |
| **Supply Chain & Recovery** | SPDX 3.0.1 graph closure, SLSA provenance, Sigstore verification, OSV advisory scans, OTEL redaction, disaster recovery lifecycle | `tests/supply_chain_operations_recovery.rs` |
| **Controller & CLI Lifecycle** | 26 model-defined commands verified; foreground/detach, destroy auth, completions, JSON output, exit code mapping | `tests/controller_cli_lifecycle.rs` |
| **Standard-Native Adapters** | Compose + Kubernetes with fail-closed validation, read-only roots, security profiles, two-stage deployment | `tests/standard_native_target_adapters.rs` |
| **Universal SDK Entrypoint** | Template contract R1-R6, anti-vacuity, pinned commits, reviewable updates, policy tree SHA-256 | `tests/universal_template_entrypoint.rs` |
| **Calculator Reference Closure** | Full SDK + system reference across Compose, Kubernetes, Pages; distinct Release A/B digests | `tests/calculator_reference_closure.rs` |
| **Lean4-Prod Dependency Closure** | 15 compiler contributions tracked via upstream issues/PRs; vendored artifacts with SHA-256 | `tests/lean4_prod_dependency.rs` |
| **Crates.io Bootstrap** | 5 first-party crates in dependency order; trusted publishing readiness; downstream lock bindings | `tests/crates_io_bootstrap.rs` |
| **Ecosystem Release Closure** | 5 repos, 3 packages, calculator baseline, dual-platform SDK, 14 falsification classes | `tests/ecosystem_release_closure.rs` |
| **Workspace Functional Core** | Signed envelopes, authenticated browser journal, View/Kappa admission (DK-07..DK-16) | `tests/workspace_functional_core.rs` |
| **Production Contracts** | System schemas and conformance model | `tests/production_contracts.rs` |
| **Authority Imports** | Immutable authority, locked drift rejection, oracle verification | `tests/authority_imports.rs` |
| **Production System Model** | Complete system model validation | `tests/production_system_model.rs` |
| **Release Status Closure** | 6-step canonical validation with receipts | `tests/release_status_closure.rs` |
| **Diagnostic Boundaries** | PP1001–PP1003 actual loader execution | `fix/issue-14-diagnostic-boundary-coverage` |

## Development and acceptance

The host needs only Git, Docker with Buildx, the Dev Container CLI, and
repository credentials. Rust, Lean, Wasm, Node, browser, and conformance tools
run inside the pinned devcontainers. Dependency acquisition is an explicit
setup phase; build and acceptance are locked and offline afterward.

```sh
just vv
```

`just vv` is the normative repository gate. Application completion is reported
only by `prismpm/application-acceptance/1`, and ecosystem completion additionally
requires the final cross-repository release manifest. See [CONFORMANCE.md](CONFORMANCE.md),
[ERRORS.md](ERRORS.md), and [VERIFICATION.md](VERIFICATION.md).

For a built immutable product release, `prismpm conformance
NAME@sha256:DIGEST` runs the SDK-contained registered feature and diagnostic corpus
and attaches its digest-bound canonical production-acceptance transcript. The
same command is available through the shared action; it does not rebuild the
release or infer coverage from registry membership.

`prismpm export-browser NAME@sha256:DIGEST --output site` exports an already
acquired release's exact browser files after source-free integrity replay.
The destination must be new, in a caller-owned project directory without group
or other write permission. This does not authorize publication or establish
product acceptance; a publisher must verify those separately before deployment.

## Key Documents

- [RELEASE-STATUS.md](RELEASE-STATUS.md) — Complete release acceptance closure across all 6 steps
- [VERIFICATION.md](VERIFICATION.md) — Detailed verification evidence, digests, and test descriptions
- [SPEC.md](SPEC.md) — Atomic release contract specification
- [CONFORMANCE.md](CONFORMANCE.md) — Conformance requirements
- [ERRORS.md](ERRORS.md) — Error code registry