# PrismPM falsifiability and verification record

## Ordered control coverage (SY-07)

Before implementation, `cargo test --locked -p repo-conformance --test
conformance conformance_sy_07 -- --exact --nocapture` failed in the devcontainer
because the semantic snapshot lacked `Production.ControlCoverage`. The log is
`target/control-coverage-red.log`. The capability is structural composition,
not OSCAL conformance or evidence authentication. Its modeled finite corpus
separately witnesses permitted inheritance and rejects origin relabeling,
missing/residual obligations, cycles, duplicate/dangling records, and exact
binding mutations; those cases do not substitute for a universal proof.

The combined dependency-cycle case was strengthened so its two provider edges
are already topologically ordered and neither downstream residual list is
dropped. Replacing only the residual-order predicate with `true` then made the
generated Lean verifier reject `coverageCaseCombinedDependencyCycle = false`
and `coverageCorpusPassed = true`: `decide` proved both propositions false
(`PP5001`, cause `LLV7002`; `target/control-coverage-residual-mutation.log`).
The formal source was restored byte-for-byte after that observed failure.

A separate generated-source probe found that generic `Bytes` equality in the
pinned LexLean runtime does not kernel-reduce, although its String equality
and Bytes-length probes pass. The complete failed formal probe is retained in
`target/control-coverage-bytes-reduction-reproducer.lex.tex`, with diagnostic
log `target/control-coverage-probe.log`. This remains an upstream compiler
issue, not a claimed passing capability. Control coverage instead models
identities as exactly 32 UInt8 octets and compares them by structural recursion;
no equality assertion or proof check is omitted.

The following self-contained semantic declaration preserves the failing probe
in the repository (the larger diagnostic workspace above is ignored). Add it
to a LexLean semantic module and run its normal generated-Lean verification;
the pinned runtime cannot reduce the declared Bytes equality through `decide`.

```json
{"kind":"theorem","name":"coverageBytesEqualityProbe","parameters":[],"proof":{"kind":"decide"},"statement":{"kind":"eq","left":{"arguments":[{"hex":"1111111111111111111111111111111111111111111111111111111111111111","kind":"bytes"},{"hex":"1111111111111111111111111111111111111111111111111111111111111111","kind":"bytes"}],"kind":"primitive","operation":"equal","result":{"kind":"bool"}},"right":{"kind":"bool","value":true}}}
```

Native verification with lean4-prod revision
`853534bddf17690ce45601977db21a795a03d4c8` passed, producing attestation
`01672f7e680cf6b996f65f6be273f743193418a5f535724980439bff25f2e9c6`
for build `a70c44cde661c11cdea4aa75d8ce41666b3d14f36fc30ed979d3d9fcf3382fc7`.
The emitted Rust contains all 54 case definitions, all 54 distinct aggregate
calls, and actual calls to `validateControlCoverage`, not a constant-true
replacement. The native executable ran twice deterministically: 597 list
inputs plus 54 separately accounted control cases (6 positive/48 negative),
with the measured validator probes reporting no allocation or panic. The
digest ABI is owned `Vec<u8>` fields behind borrowed policy/submission inputs;
the modeled exactly-32-octet check remains mandatory. Rust compilation emitted
warnings, recorded in its process evidence; successful execution is not a
claim of warning-free generated code or complete release acceptance.

The upstream ownership failure and its reviewed generic correction are
tracked in [lean4-prod issue 30](https://github.com/auser/lean4-prod/issues/30)
and [PR 29](https://github.com/auser/lean4-prod/pull/29). Its focused native
regression failed before the correction, and the complete devcontainer
`CARGO_NET_OFFLINE=true just ci` passed before committing the accepted fix.
Ordinary Prism verification now checks the exact finite corpus structure and
count bindings before emitting execution evidence; six metadata/source
mutations are rejected with `PP5006` by
`execution_rejects_invented_or_short_circuited_control_case_counts`.

## Generated standard-library package boundary

Regeneration exposed a producer-owned formatting defect: Cargo `src/lib.rs`
ended with redundant blank lines, so faithful generated output failed
`git diff --check`. The empty-module regression failed before the correction.
lean4-prod commit `081ec576f10c7ea5698641483db455962c567e76` normalizes only
the complete generated source to one final LF before its manifest hashes are
computed. Empty, single-definition, and multiple-definition regressions check
that boundary, the final source hash, and byte-preservation of supplied license
and README assets. All 66 codegen unit tests, denied-warning Clippy, and the
complete devcontainer `CARGO_NET_OFFLINE=true just ci` passed. The source is
tracked by [issue 31](https://github.com/auser/lean4-prod/issues/31) and
[PR 32](https://github.com/auser/lean4-prod/pull/32); generated files were not
edited by hand.
The upstream [CI verification run](https://github.com/auser/lean4-prod/actions/runs/35029493850)
also passed for `081ec576`; PR 32 was still open at this checkpoint, not merged.

The source-package comparison also rejected an intermediate package after its
authoritative license inputs changed (`target/control-coverage-stdlib-upstream-check.log`):
`stdlib package drifted; review just stdlib-package-write output`.
This was an observed negative check, not a passing package acceptance result.

The additional required accounting and export bindings are deliberately new
closed evidence contracts: `execution-evidence/2` and
`verification-manifest/2`. Focused negative tests reject the former `/1`
schema values; Holo/1, the CLI result, and application verification retain
their existing schema identities. The successful `/1` union-export preflight
was not accepted as final `/2` evidence.

Fresh `/2` source-package generation and an independent check reproduced
attestation `ad4ae79ee9f78158dd35db0faec6f29e49315ea5707197420cf866b5fa0bafd9`
and LCNF SHA-256
`181c4cf507d3b9ffde50471e14aa825df4c8a767a71c309fe18e0003369b65ff`.
The accepted request contains 51 validator roots and 20 additional package
roots, with all 71 names covered by the one verified export. Runtime evidence
separately reports 597 list inputs and 54 control cases (6 positive/48 negative).
The original 31 application-function signatures and two public types/traits
compile and execute in both `std` and `no_std` plus `alloc` consumer builds;
boundary cases cover bytes, UTF-8, checked integers, and Holo validators.
These component checks do not substitute for clean-commit full VV or registry
release acceptance. Final goldens bind the executable after archive sealing;
embedding changed archive/release bytes can legitimately change the enclosing
LexLean executable hash and its derived attestation identity.

After sealing the standard-library archive (`eae96b8d0a6627fc25cc3b9b73f61c4b91830673ca316deaf0b5e1c4b5928053`),
the final native verification and independent golden comparison accepted
attestation `457c17ac4dfaaec185f882037eba8a82ae9f8b885bd1e7b7f0d79559cfaac004`
for build `416d0d8d2033be2322366eed17fcd26bf4ccc9067d5175216ed496150df96f20`.
All 234 reviewed golden files matched (233 listed files plus the manifest).
Archive reproduction, all 16 fixture comparisons, 18 verifier unit tests,
15 package-orchestration unit tests, model/spec/source audits, formatting,
and denied-warning Clippy passed. The exact package also built offline for
`wasm32-unknown-unknown` with default features disabled, without changing its
source. The post-seal Calculator example reproduced build
`6d7c9c3791c2ac639453291289809deb4afe57b7f5b29960971fcb30c1386999`
and verified attestation
`5f8610adabb35c3f5586e5abeeeba0c7c1fbc324659cfc4bb08ce407cae0a993`.
Calculator's arithmetic and View sources were unchanged; compiler/stdlib
provenance and package-byte changes legitimately change its artifact identity.

The direct 148-case conformance run, with `PRISMPM_TEST_SDK_IMAGE` unset,
reported **139 passed, 9 failed, 0 ignored**. SY-07, ST-07/ST-08 and every
EX-01 through EX-10 passed. AU-05 rejected the absent immutable SDK image;
AU-06, LC-03, OC-01, OC-05, SC-01, SC-02, SY-04 and SY-05 rejected the same
missing prerequisite with `PP5403`. The complete failed run is retained in
`target/control-coverage-conformance-sealed.log`; no old image was substituted
and no test was skipped. This is not full acceptance: normal clean-HEAD
`just vv` must initialize the current-source SDK through its isolated local
registry and rerun the complete gates.

## Original full-gate falsification campaign

This record covers every gate in `cargo xtask vv`. A gate is considered armed only
after an intentional defect, or the equivalent committed negative fixture, makes
that gate fail for the expected reason. Temporary defects were removed byte for
byte after observation. The clean restoring commit for the final falsification
pass is `7752e4ea7f156ec613f7b052af9b430d6e24d61d`; the implementation baseline is
`2745b0fb710c5fc1b13c38e82ff101106056c287`.

The historical restoration/source commits above identify the original
falsification campaign, not the PrismPM 0.3.0 release identity. Every current
`just vv` run writes `target/vv-evidence.json` against the exact checked-out
commit and all 15 gates; release-check rejects stale evidence or a different
commit.

The records below distinguish a gate failure from a later gate incidentally
noticing the same defect. Exact diagnostics are included where they are stable;
otherwise the asserted semantic outcome is recorded. No defect is present in the
release tree.

| Gate | Acceptance surface | Planted or observed defect | Required failure | Restoration |
|---:|---|---|---|---|
| 1 | Rust formatting | Unformatted conformance edit | `cargo fmt --check` exits nonzero | Formatter applied; restoring commit above |
| 2 | Models and generated docs | Stale generated emitter digest | Model/generated-document audit rejects stale bytes | Digest and generated bytes regenerated |
| 3 | SPEC link closure | RP-01 statement changed only in `SPEC.md` | Statement bijection mismatch | Exact SPEC bytes restored |
| 4 | Source and dependency audits | Handwritten `Planted.lean` added | Generated-Lean-only audit names the file | File removed |
| 5 | Clippy | Large error result and collapsible conditional | Warnings denied | Findings corrected |
| 6 | Workspace tests | Meta-test weakened to accept hidden tests | Named unit test fails | Assertion restored |
| 7 | Fixtures | Dangling Holo edge endpoint | Registered PP diagnostic is required | Fixture and oracle corrected |
| 8 | Lean verification | Axiom mismatch and failed child processes | Verification refuses publication | Permanent negative tests retained |
| 9 | LexLean and Prism operations | Noncanonical LexLean source/lock state | `fmt --check`/`lock --check` rejects it | Source formatted and lock synchronized |
| 10 | Schema and goldens | Nondeterministic Lake ordinals/root-bound evidence | Exact golden comparison fails | Output normalized and evidence root-independent |
| 11 | Export and execution | Generated harness used `usize` where `u64` is required | Rust compilation fails | Harness uses the declared ABI type |
| 12 | Reproducibility | `.olean` bytes and Lake output depended on absolute root/schedule | Two-root byte comparison fails | Root-independent compile and normalization |
| 13 | Authoritative upstream conformance | Invalid OTLP/HTTP identifiers, mutated OCI layout, and always-pass/mismatched oracle fixtures | Pinned upstream runner or official corpus rejects the subject at its registered diagnostic | Canonical fixtures and exact authority bindings restored |
| 14 | Dependency policy | Workspace `serde` requirement changed to `*` | `cargo deny` bans check fails | Exact requirement restored |
| 15 | Package/public API | `vendor/**` removed from Cargo package selection | Package gate reports omitted Lean payload | Restoring commit `7752e4e` |

## Gate 1 — formatting

An intentionally unformatted edit in the conformance implementation caused
`cargo fmt --all -- --check` to emit a diff and exit nonzero. Applying the pinned
formatter restored the tree. This proves formatting is checked, rather than
silently rewritten, by the acceptance command.

## Gate 2 — model, diagnostics, standards, and generated documents

During implementation, changing the emitter without refreshing its registered
digest caused the model/generated-document check to reject the stale digest.
The same gate contains negative model fixtures for unknown fields, unsupported
standards claims, missing editions, diagnostic closure, and byte-current
`CONFORMANCE.md`/`ERRORS.md`. Regeneration is a separate command and is not run
by `vv`.

## Gate 3 — SPEC/register/scenario/test links

The RP-01 statement in `SPEC.md` was changed from “pinned toolchains” to “pinned
tools” while the register remained unchanged. `cargo xtask validate-spec-links`
failed with:

```text
gate failed: RP-07: `RP-01` statement mismatch:
  table:    PrismPM is structured as a virtual Rust workspace with pinned tools.
  register: PrismPM is structured as a virtual Rust workspace with pinned toolchains.
```

The exact SPEC bytes were restored. The gate therefore checks semantic text and
the capability/register/scenario/named-test bijection, not merely identifier
presence.

## Gate 4 — source, error, unsafe, dependency, and generated-file audits

`crates/prismpm/src/Planted.lean` was added temporarily. `cargo xtask validate`
failed with:

```text
gate failed: no-handwritten-lean audit failed: found .../crates/prismpm/src/Planted.lean
```

The file was removed. A separate real failure in this gate detected a stale
vendored LexLean `.cargo_vcs_info.json`; checksum-aware vendor synchronization
and the tree manifest corrected it. The audit also rejects `unsafe`, forbidden
generated files, unregistered public errors, wildcard/mutable dependencies,
unapproved paths, and mismatched vendor manifests.

## Gate 5 — Clippy with warnings denied

Intermediate implementations containing a large `Result` error and a
collapsible conditional failed
`cargo clippy --workspace --all-targets --all-features -- -D warnings`. Both
findings were corrected. The final gate covers every target and feature.

## Gate 6 — workspace unit and property tests

The conformance-discovery meta-test was temporarily inverted so that a planted
hidden/non-test macro was accepted. Running its exact test failed:

```text
test tests::conformance_discovery_rejects_hidden_or_non_test_macros ... FAILED
assertion failed: !flagged.is_empty()
test result: FAILED. 0 passed; 1 failed
```

The assertion was restored. Discovery now rejects empty fixture sets and flags
`#[ignore]`, `#[cfg(...)]`, `#[cfg_attr(...)]`, `#[should_panic]`, and capability
macros that do not expand to an unconditional `#[test]`.

## Gate 7 — feature, conformance, and negative fixtures

The minimal Holo fixture initially contained dangling references. The fixture
runner rejected it with the registered path-specific diagnostic instead of
accepting structurally valid JSON. Permanent negative fixtures cover all public
diagnostic families, while exact named scenario discovery prevents an empty or
partially discovered suite from passing.

## Gate 8 — generated Lean build, replay, axiom audit, and source audit

Negative verification tests plant an unexpected axiom, child-process failure,
timeout, malformed/unsupported LCNF, incomplete coverage, and partial-publication
conditions. Each causes verification to return the registered error and leaves
no published verification manifest. The positive path builds only
LexLean-generated Lean, replays it with `leanchecker`, audits the exact axiom
sets, and then repeats the handwritten-Lean source audit.

## Gate 9 — LexLean format/lock plus Prism check/build/verify

Noncanonical vendored LexLean source and a stale package lock were each rejected
by the vendored `fmt --check` and `lock --check` commands. After correction, the
gate runs the public Prism controller's `check`, `build`, and `verify` paths; it
does not substitute a fixture-only implementation.

The immutable integrations used by the release are LexLean
`0b53334e5846a1f5e5d9bb3bf6959c085daa4d08` and `lean4-prod`
`c4078cf96537cd71c0818bbed0aa82300ef66786`.

## Gate 10 — Holo schema and reviewed golden bytes

Successful Lake output originally contained scheduling ordinals and durations,
and LexLean `.olean`/attestation evidence originally captured absolute source
roots. Exact golden comparison exposed both changes. Successful tool output is
now normalized only for those declared volatile fields; status, arguments,
errors, and content remain evidence. The reviewed set contains 167 files with:

```text
build_id      4b144f83991cd970ac3dd2dc44aa65679a56c2f099a4c5545347352f42fc4e60
attestation   9a790d121dd59c7a70dd3e3858cf8b392cc1e1fff1e460993de4d86b5b83f486
review reason Make PrismPM package assets relocatable and downstream-compilable
```

The acceptance gate only compares bytes and never regenerates goldens.

## Gate 11 — named export, coverage, Rust compilation, and execution

An intermediate generated harness represented a declared `u64` value as
`usize`; the generated Rust compilation step failed. The corrected ABI compiles
the named validator closure and executes the deterministic corpus twice. The
evidence asserts complete named-root coverage, empty unsupported-node sets, 597
cases, no panic, no allocation, equal repeated output, and a published result
bound to the verified Prism model document and kernel IR.

## Gate 12 — two-absolute-directory reproducibility

Building in two fresh absolute directories exposed source-root bytes in `.olean`
artifacts and Lake scheduling ordinals in captured output. LexLean now invokes
Lean with an explicit logical root and verifies equality from two fresh roots;
Prism normalizes only the permitted successful-output volatility. The restored
gate compares 134 build artifacts and 8 verification artifacts byte for byte and
requires equal build and attestation identities.

## Gate 13 — authoritative upstream corpora, registry, and runtime conformance

This gate executes imported, immutable authority assets rather than a
Prism-authored substitute: the official JSON Schema and Unicode corpora,
upstream OCI Image and Runtime tests, the official OCI Distribution suite
against a clean registry, and every external oracle runner in the SDK's
network-denied sandbox. Permanent mutations cover an oracle that always
passes, never executes, targets a different file or edition, changes bytes, or
accepts a malformed subject. During implementation, base64 OTLP trace/span IDs
were rejected because OTLP/HTTP JSON requires fixed-length hexadecimal IDs;
the corrected corpus preserves that negative boundary.

## Gate 14 — dependency policy

The workspace `serde` requirement was temporarily changed to `*`.
`cargo deny --frozen --all-features check` reported wildcard requirements for
`prismpm`, `repo-conformance`, and `repo-model`, marked the bans check failed,
and exited with status 2. Restoring the exact requirement made the policy pass.
Duplicate-version notices remain warnings for reviewed transitive dependency
families; licenses, advisories, wildcard requirements, registries, and the
exact pinned Hologram Git source remain deny-level checks.

## Gate 15 — packaged crate and downstream public API

Test commit `8c644c90ec5bcb8b7db6aa2e5b8f8a951004e0ff` removed
`vendor/**` from `crates/prismpm/Cargo.toml`, making Cargo omit the vendored Lean
payload. An isolated temporary command route invoked the same
`package_api_check` function used by gate 15 and failed with:

```text
gate failed: packaged crate omits vendor/lean4-prod/lean.tar
```

The full `vv` entry point also rejected the manifest mutation at gate 10 because
the package manifest is a build input; the isolated call establishes that gate
15 independently rejects the package closure. Restoring commit
`7752e4ea7f156ec613f7b052af9b430d6e24d61d` removed the temporary route and
restored the include. The gate builds a Cargo-selected package tree,
verifies required payload and forbidden-cache closure, rewrites only the
temporary dependency sources for the offline test, and compiles/runs a
downstream consumer against the public Controller API.

## Release criterion

Only a clean, annotated `v0.3.0` tag whose exact commit has produced
`target/vv-evidence.json` with all gates 1 through 15 may pass
`just release-check`. Release artifacts and every platform-specific SDK,
runtime, adapter, and oracle image are built twice and must be identical;
their checksums, SPDX SBOMs, provenance attestations, and signatures are
produced only for those accepted bytes.
