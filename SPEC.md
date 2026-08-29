# PrismPM Specification

This document is the normative specification for PrismPM.

## 1. Scope & Standards Profile

PrismPM formalizes and models information systems across multiple viewpoints:
- **ISO/IEC/IEEE 42010:2022**: Systems and software engineering — Architecture description.
- **ISO/IEC 27034-1:2011 & ISO/IEC 27034-5:2017**: Information technology — Application security.
- **ISO/IEC 27005:2022**: Information security risk management.
- **ISO/IEC 25010:2023**: Systems and software product quality model.

## 2. Architecture & Pipeline

PrismPM operates through a formally verified multi-stage pipeline:
1. Systems and facets are authored in `.lex.tex` using LexLean 1.1 lexicons.
2. LexLean lowers the model to Lean 4 for formal verification (`leanchecker` and axiom audit).
3. The Prism Holo emitter produces canonical `prismpm/holo/1` artifacts from LexLean semantic snapshots.
4. `lean4-prod` extracts compilable LCNF from the verified Lean declarations and emits `kernel.ir`.
5. `prod-codegen` compiles zero-cost, `no_std` Rust validators that execute over normalized Holo models.

## 3. Conformance ID Registry

Every row below is normative, has honesty level `build`, and is copied byte-for-byte into `model/ids.toml`.

| ID | Suite | Normative statement | Primary specification |
|---|---|---|---|
| `RP-01` | `repository` | PrismPM is structured as a virtual Rust workspace with pinned toolchains. | §1 |
| `RP-02` | `repository` | Toolchain versions for Rust and Lean are pinned in rust-toolchain.toml and lean-toolchain. | §1 |
| `RP-03` | `repository` | Every model claim is authored in model/*.toml and validated by cargo xtask validate-model. | §1 |
| `RP-04` | `repository` | The devcontainer and CI enforce hermetic build and test environments. | §1 |
| `RP-05` | `repository` | All public crates forbid unsafe code and enforce strict lints. | §1 |
| `RP-06` | `repository` | The Justfile exposes vv as the primary acceptance gate. | §1 |
| `RP-07` | `repository` | The SPEC.md capability table and model/ids.toml are bijective and consistent. | §1 |
| `RP-08` | `repository` | Every public capability has exactly one Gherkin scenario and one named conformance test. | §1 |
| `RP-09` | `repository` | Two clean builds in distinct absolute directories produce byte-identical platform-independent outputs. | §1 |
| `RP-10` | `repository` | All generated documentation is regenerated and validated against model registers. | §1 |
| `RP-11` | `repository` | License metadata and files conform to dual MIT/Apache-2.0 requirements. | §1 |
| `RP-12` | `repository` | Release artifacts are refused unless all release criteria and verifications hold. | §1 |
| `FT-01` | `facets` | prism.arch defines ISO/IEC/IEEE 42010:2022 architecture description concepts in LexLean. | §2 |
| `FT-02` | `facets` | prism.sec defines ISO/IEC 27034 application security concepts in LexLean. | §2 |
| `FT-03` | `facets` | prism.sec defines ISO/IEC 27034-5 application security control data structures. | §2 |
| `FT-04` | `facets` | prism.sec defines ISO/IEC 27005:2022 information security risk management concepts. | §2 |
| `FT-05` | `facets` | prism.qual defines ISO/IEC 25010:2023 product quality characteristics and requirements. | §2 |
| `FT-06` | `facets` | Facet packages use closed lexical signatures and renderer-token coverage. | §2 |
| `FT-07` | `facets` | Shared terms across facets have unique ownership and acyclic dependencies. | §2 |
| `FT-08` | `facets` | Every standards entry links to exactly one row in model/standards.toml. | §2 |
| `FT-09` | `facets` | Facet lexicons produce deterministic canonical LaTeX and Lean lowerings. | §2 |
| `FT-10` | `facets` | Lexicon package locks reproduce across multiple directory roots. | §2 |
| `HO-01` | `holo` | Holo artifacts conform to the prismpm/holo/1 JSON schema. | §3 |
| `HO-02` | `holo` | Holo serialization follows canonical JSON formatting rules with sorted ASCII keys. | §3 |
| `HO-03` | `holo` | Holo content ID is the exact SHA-256 hash of its canonical UTF-8 bytes. | §3 |
| `HO-04` | `holo` | The Holo projector is a total function from LexLean semantic snapshots. | §3 |
| `HO-05` | `holo` | Holo entity identifiers are qualified strings assigned deterministic zero-based indexes. | §3 |
| `HO-06` | `holo` | Holo serialization excludes host paths, timestamps, and unstable environment data. | §3 |
| `HO-07` | `holo` | Holo decoding rejects noncanonical representations and malformed values. | §3 |
| `HO-08` | `holo` | The emitter-semantics ID uniquely identifies the Holo projector inputs. | §3 |
| `HO-09` | `holo` | Shared golden vectors prove correspondence between Lean normalized models and Holo DTOs. | §3 |
| `HO-10` | `holo` | Holo validation verifies internal cross-references before emission. | §3 |
| `CT-01` | `controller` | The Controller API exposes owned request and result types for load, check, and build. | §4 |
| `CT-02` | `controller` | The Controller encapsulates LexLean Engine operations without exposing internal compiler types. | §4 |
| `CT-03` | `controller` | prismpm check validates models in memory without modifying the filesystem. | §4 |
| `CT-04` | `controller` | prismpm build atomically publishes artifacts under content-addressed build directories. | §4 |
| `CT-05` | `controller` | The CLI produces structured machine-readable JSON output on stdout. | §4 |
| `CT-06` | `controller` | The Controller enforces positive resource and entity limits. | §4 |
| `CT-07` | `controller` | All public errors carry registered PP diagnostic codes. | §4 |
| `CT-08` | `controller` | The Controller operates completely offline during build and check. | §4 |
| `CT-09` | `controller` | Project configuration is strictly validated against schemas/project.schema.json. | §4 |
| `CT-10` | `controller` | The Controller preserves cause chains for underlying LexLean diagnostics. | §4 |
| `ST-01` | `stdlib` | Prism-stdlib defines core ISO 42010 architectural primitives in Foundation.Arch. | §5 |
| `ST-02` | `stdlib` | Prism-stdlib defines ISO 27034 security primitives in Foundation.Sec. | §5 |
| `ST-03` | `stdlib` | Prism-stdlib defines ISO 27005 risk primitives in Foundation.Sec. | §5 |
| `ST-04` | `stdlib` | Prism-stdlib defines ISO 25010 quality primitives in Foundation.Qual. | §5 |
| `ST-05` | `stdlib` | Prism-stdlib is authored strictly in .lex.tex with no handwritten Lean source. | §5 |
| `ST-06` | `stdlib` | Prism-stdlib models allow cyclic component graphs while rejecting dangling references. | §5 |
| `ST-07` | `stdlib` | Prism-stdlib proves cross-facet consistency theorems with empty observed axiom sets. | §5 |
| `ST-08` | `stdlib` | Prism-stdlib exports registered runtime validator roots. | §5 |
| `ST-09` | `stdlib` | Prism-stdlib includes golden test outputs for all published artifacts. | §5 |
| `ST-10` | `stdlib` | Prism-stdlib models validate through the Holo projector and Lean kernel. | §5 |
| `AR-01` | `artifacts` | Build artifacts are published under content-addressed .prism/build/<id> paths. | §6 |
| `AR-02` | `artifacts` | Every build directory contains a canonical manifest of file paths, sizes, and hashes. | §6 |
| `AR-03` | `artifacts` | Artifact content IDs are derived from deterministic SHA-256 digests. | §6 |
| `AR-04` | `artifacts` | Published artifacts contain no absolute paths, timestamps, or host identifiers. | §6 |
| `AR-05` | `artifacts` | Artifact publication is atomic and leaves no partial state on failure. | §6 |
| `AR-06` | `artifacts` | Rebuilding an identical project verifies existing artifacts before reuse. | §6 |
| `AR-07` | `artifacts` | Tampered build artifacts are detected and rejected. | §6 |
| `AR-08` | `artifacts` | Canonical LaTeX and Lean artifacts reproduce byte-for-byte across runs. | §6 |
| `AR-09` | `artifacts` | Source maps provide complete token traceability between .lex.tex and generated Lean. | §6 |
| `AR-10` | `artifacts` | Coverage reports document all modeled and unmodeled elements. | §6 |
| `EX-01` | `execution` | lean4-prod exports compilable LCNF from LexLean-generated Lean modules. | §7 |
| `EX-02` | `execution` | lean4-prod transitive closure extraction resolves all dependencies without opaque gaps. | §7 |
| `EX-03` | `execution` | Generated Rust code is zero-cost, no_std compatible, and avoids heap allocation. | §7 |
| `EX-04` | `execution` | The Rust host strictly decodes canonical Holo and normalizes string IDs to indexes. | §7 |
| `EX-05` | `execution` | Generated Rust validators correctly evaluate normalized Holo models. | §7 |
| `EX-06` | `execution` | Exhaustive evaluation matches Lean native results on small bounded models. | §7 |
| `EX-07` | `execution` | Property-based tests with committed seeds verify validator behavior on arbitrary bounded inputs. | §7 |
| `EX-08` | `execution` | Generated Rust validators never panic on caller-controlled inputs. | §7 |
| `EX-09` | `execution` | Erased proof fields are not claimed as runtime checks. | §7 |
| `EX-10` | `execution` | Type boundary conversions between Lean Nat and Rust u64 are explicitly bounded. | §7 |
| `VR-01` | `verification` | LexLean verifies formal models against the Lean 4.32.1 kernel. | §8 |
| `VR-02` | `verification` | Verification records executable digests and normalized toolchain commands. | §8 |
| `VR-03` | `verification` | Lake stages generated Lean modules in isolated temporary workspaces. | §8 |
| `VR-04` | `verification` | leanchecker replays compiled module environments independently. | §8 |
| `VR-05` | `verification` | Axiom audits enforce that all PrismPM-owned theorems have empty observed axiom sets. | §8 |
| `VR-06` | `verification` | Controller::verify publishes verified attestations under .prism/verified/<id>. | §8 |
| `VR-07` | `verification` | Verification manifests bind Holo, Lean, LCNF, and Rust execution evidence. | §8 |
| `VR-08` | `verification` | Verification fails if any Lean elaboration, replay, or axiom check fails. | §8 |
| `VR-09` | `verification` | Verification performs no hidden network access. | §8 |
| `VR-10` | `verification` | Process execution limits output size and execution time strictly. | §8 |
| `VR-11` | `verification` | Planted defects in proofs or validators reliably trigger verification failure. | §8 |
| `VR-12` | `verification` | Repeated fixed-seed verification runs produce identical attestation digests. | §8 |
| `SE-01` | `security` | All filesystem operations are confined within configured output roots. | §9 |
| `SE-02` | `security` | Symlinks and relative paths that escape project boundaries are rejected. | §9 |
| `SE-03` | `security` | Process execution executes child binaries directly without invoking a shell. | §9 |
| `SE-04` | `security` | Environment variables passed to child processes are sanitized and normalized. | §9 |
| `SE-05` | `security` | File publications use atomic rename operations to prevent partial writes. | §9 |
| `SE-06` | `security` | Memory safety is enforced across all Rust crates with unsafe code forbidden. | §9 |
| `SE-07` | `security` | Input parsing rejects excessively large inputs before memory exhaustion. | §9 |
| `SE-08` | `security` | Dependency audits with cargo-deny enforce license and advisory policies. | §9 |

