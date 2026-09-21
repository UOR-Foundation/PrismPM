# PrismPM falsifiability and verification record

## Generated workspace View (DK-15, DK-16)

Registered owning tests first rejected the absent implementation. The complete
fresh modeled gate then passed all 206 vectors in native, no_std and bounded
Wasm execution, including actual compiled session/row mutations. The private
host gate freshly built View, Command, Query and Journal, exercised genuine
browser identity/storage journeys and replayed all 6,562 observed calls twice
natively. All ten host-suite tests passed without skips.

Review reproduced initialization failure leaving one mounted node and seven
listeners. The corrected constructor clears both before returning failure.
The complete host rerun covers three initialization faults, eight promotion
faults and nine host mutants; its log SHA-256 is
`cea54c6b5d65f98ce235ed9911653a5622478c50c51600fc0212f1e14bd54f98`.

Normal writers regenerated the model, stdlib and reviewed goldens. Independent
readers matched all 324 golden files and passed generated-package, downstream
API and all three actual Cargo archive comparisons. The package/API log SHA-256
is `037a3543c5c8a3230b4368d1bb37a1e54709360aabdcdbc5b8d438f92b9121ac`.
All 14 model tests, 15 contract tests, ten SDK/acquisition script tests and
14 release-phase tests passed, as did authored formatting and workspace Clippy.
Clean-tree full V&V and installed dual-architecture SDK acceptance remain
separate requirements. These components do not establish Foundation authority,
email ownership, Kappa replication or Foundry publication.

## Release archive target ownership

An inherited `CARGO_TARGET_DIR` sent verified archives outside their prepared
stage, causing the real archive check to fail. Packaging now selects its owned
target explicitly. The publication workflow selects the target of its expected
package/version archive while retaining verification and pre-publication byte
comparison. No Cargo publication was performed.

The real packaging regression and workflow validator both failed before the
fix. All 14 release-phase tests then passed in the devcontainer, including exact
archive equality, preservation of caller artifacts, seven workflow mutations,
and valid/invalid shell path selection. The log SHA-256 is
`097da6b212e1ce7707a184b14673e50f2aac3d33cabdaad3106410a32d6e0a3b`.

## Source-free browser export (OC-07)

`export-browser` replays the retained release closure and atomically publishes
only its six exact generated browser files. Genuine named releases A and B
are built and verified before source removal; an empty receiver then exports
their unchanged bytes. Missing or substituted evidence, changed assets,
symlinks, hardlinks, special files, unsafe paths, existing outputs and staged
directory replacement are rejected. The CLI first failed with the command
absent; removing the post-publication identity checks made the owning race
regression fail. Both defects were restored before verification.

All 157 library tests passed without skips or filters in the devcontainer;
`target/browser-export-library-complete.log` has SHA-256
`3fa9339257f1b7c3369eb16b449b6b1d3926701450d07a41a0dd8d4584a6f95a`.
All-target/all-feature Clippy, formatting, source/model/spec audits and all
33 model/driver/compatibility tests passed. The final complete conformance
package passed all 151 scenarios and eight unit tests, with none ignored or
filtered; log `target/browser-export-conformance-clean.log` has SHA-256
`dbe3311464c34b31ac0a3a9a3cb9a1b19141e3a78e1805f3abad635f7a5f1e48`.
Golden review preserved all 194 build files and native execution evidence;
only verifier provenance and its two derived records changed. An independent
check reproduced all 240 golden files. Final static checks and golden replay
are recorded in `target/browser-export-final-audit.log`, SHA-256
`4128bcfa6a31361d25b8f0a4dc327a9e306ce14cbf54add278a5cc689384dfeb`.

The real package/API gate still fails on the absent public `uor-hologram`
dependency. No gate is waived and no new SDK release, production-acceptance
receipt or Foundry deployment is claimed. The digest-bound development image
below supplies external oracle tools only. Export integrity does not establish
producer readiness, authorization, full artifact coverage or live acceptance.

## Fixture source isolation

The complete conformance rerun passed 149 cases and failed HO-11/OC-07 with
`ENOSPC`: the text fixture contained 17 GiB of ignored generated cache, which
the fixture copier duplicated into temporary projects. Only those confirmed
untracked generated directories were removed. The copier now prunes generated
state at the project root, preserves source/dotfiles/nested names, and rejects
unreadable, symlinked or nonregular source inputs instead of silently omitting
them. Four owning regressions failed before the correction; all five passed
afterward. Logs: `target/fixture-source-copy-red.log` and
`target/fixture-source-copy-green.log`. No expected outcome or gate was reduced.

## Complete generated-projection oracle execution

Real system projections exposed two additional failures: Compose could not
resolve its modeled secret-directory reference in the cleared sandbox, and
the Kubernetes wrapper rejected the generated generic `List` before checking
its resources. Compose now receives a fixed, isolated non-secret directory;
interpolation and consistency checks remain enabled. The Kubernetes wrapper
traverses every collection leaf through the unchanged pinned OpenAPI schema.
That schema then rejected the imported ingress ConfigMap's `data: null`;
the projection now emits its empty map without modifying imported source.

Compose positive/negative tests and all four Kubernetes wrapper tests pass.
The actual modeled system passes all seven locked projection oracles in
`target/projection-oracles-preflight-normalized.log`, after the recorded
failures in `target/compose-project-environment-red.log` and
`target/projection-oracles-preflight-complete.log`. Six validate generated
documents; CloudEvents uses the production runner's fixed envelope fixture,
not evidence of runtime-event acceptance. The development-only
oracle image is digest-bound at
`127.0.0.1:5000/prismpm-oracle-test@sha256:eb5205273cfcc84bb749272115a40a3b527403d2db5a8ec3751c345bbd7a1f7b`;
it runs the corrected wrapper tests and regenerates its real SDK inventory.
It supplies external tools, not an accepted SDK release or Foundry deployment.

## SPDX project-oracle schema binding

The genuine project runner rejected a valid SPDX document with `PP5403`:
the catalog's relative schema path was resolved from sandbox `/scratch`.
The SDK already includes and inventory-binds that exact authoritative schema.
The catalog now selects its existing absolute SDK path; the installed schema,
SDK wrapper and validation rules are unchanged. The regenerated lock also
retains the source hashes changed by adding the regression tests.

The focused regression runs without a project standards directory. It accepts
the valid corpus, rejects the invalid corpus with `PP5404`, and checks both
original attestations' subject, result and actual SDK image bindings. It failed
before the correction and passed afterward in the devcontainer. Logs are
`target/spdx-project-schema-red.log` and
`target/spdx-project-schema-green.log`. This fixes project-oracle execution,
not complete SDK release or Foundry acceptance.

## Devcontainer Docker group readiness

[Reproducibility run 35232240458](https://github.com/UOR-Foundation/PrismPM/actions/runs/35232240458)
failed before comparison: its lifecycle shell inherited groups before the root
initializer added Docker socket access. In an isolated copy of the actual
devcontainer image, that stale shell still failed after `usermod`, while a fresh
shell succeeded. Lifecycle commands now wait for the current socket group and
refresh their own non-root credentials when necessary; editor readiness waits
for the post-start hook. Existing terminals are not retroactively repaired.

All six devcontainer regression tests passed against the real daemon: the
unwrapped stale process fails, refreshed/fresh/restarted processes succeed,
arguments and exit status are preserved, and missing initialization/socket or
root execution fail closed. Host socket ownership/mode are unchanged; all
owned test containers were removed. The log SHA-256 is
`7a3f7947b3b1c5be51feec8b13849394bc8c0104e4336cb176e1994f3da92a08`.

Hosted VV/Bootstrap at `f01ae4c` then exposed a fixture assumption: GitHub's
remapped devcontainer no longer contained group `1000`. The production startup
hook succeeded; the test's hard-coded group failed before Cargo. Repeating the
actual UID/GID remap reproduced that failure. The fixture now derives existing
identities and exercises both original and remapped users; all 11 tests pass
without changing the production helper or omitting checks. Green log SHA-256:
`acb0e25ec498d495262d842ae2bd3c5a1af8dc7aaf55501f0ec90d3a6ccdab83`.

## Source-free release evidence (OC-02, OC-03, OC-04, OC-06)

OCI now retains the original build manifest, every path-bound output, and the
complete runtime/oracle proof closure. Replay checks the same records without
source access or execution. Native executable bytes remain in the real closure;
reviewed goldens retain their descriptor, as for generated Rust and kernel IR.
Integrity does not establish producer authorization or product acceptance.

The first real A/B application replay rejected selected-module source maps:
their identities legitimately differ from the containing system's full snapshot.
The controller now retains the actual selected snapshot, manifest and outputs;
no map is rewritten or comparison omitted. Both releases then passed runtime
verification and replay, including coherently rehashed evidence mutations.
The log SHA-256 is
`c783bbf2d956058e5063b0b16d02b05d24b18d6c245d80e5d3511082dcf829c3`.

The other 139 library tests passed with none ignored, including genuine native
evidence, missing executable/proof/build-manifest rejection, source-free OCI
round trips, path confinement, and coherently rehashed provenance substitutions.
The log SHA-256 is
`041402a8de8fcf01ff6d2bbd2119b17652cd8441b4c9619ea774ef898f8b509b`.
All four owning conformance scenarios passed; their log SHA-256 is
`f29dfef6632c376fed2a0029adae5b6e412a7d0e765bb4aa12026fc19151f255`.
These runs used the repository devcontainer and the previously authenticated
d017 SDK for external tooling, not as acceptance of this new SDK or Foundry.

The model gate exposed its stale 44-contract count after the two new contracts
were registered. The corrected exact 46-contract gate rejects every individual
omission and same-cardinality duplicate; generated contract documentation and
the model/spec linkage gate pass. Oracle attestations also preserve their
registered in-toto envelope rather than inventing a Prism `schema` property.

Denied-warning workspace Clippy passed with all targets and features. Fresh
stdlib generation checks and all three release-crate archive comparisons pass
without changing package bytes. The downstream `package-api` gate failed:
`uor-hologram` is absent from the public Cargo index. Its log SHA-256 is
`375ff6f77cf400b9532301832b9ba76a155cfdd539d8462634d50b4ddf8d1630`.
No Git/path substitute was introduced. Full release acceptance remains unmet.

The old golden comparison rejected the changed build/verifier identities.
After reviewed regeneration, an independent check matched all 240 files;
all 193 native build-output descriptors remain unchanged. Only the build
manifest, LexLean attestation, verification manifest and golden manifest changed.
The passing comparison log has SHA-256
`c35161c7f3d8b91500181207c6f07094bcbc2543aea3e5092f0f3a07dd2d85be`.

## OpenID source-closure reproducibility

[Bootstrap run 35226604960](https://github.com/UOR-Foundation/PrismPM/actions/runs/35226604960)
passed the repaired Docker/OCI integration, then eight conformance scenarios
failed at the same OpenID corpus digest check. Ten upstream `.claude`/`.idea`
files were present locally but ignored by Git. All 6,021 local files match the
unchanged pinned archive
`d33d7eb40b6db0080a48563fe6e8d1073393d18fe9f4afcb3de83f1679197bbb`.
Its complete tree hashes to the required
`a35012f67dccd4296ab0e380eb86a0e053dbbf6bf52522aca3ed961c83812197`;
the incomplete Git tree exactly reproduced CI's
`1cfc37b89be7e0e7325013deabd485e581c8ffff8aedcad143f51155d3e66dcd`.

Tracking those exact files restores the closure without changing oracle bytes
or pins. The source-audit regression failed before staging and passed afterward;
isolated-index missing/changed/extra/symlink mutations fail, and local untracked
bytes cannot satisfy it. This repairs the shared prerequisite, not evidence
that the eight complete external scenarios have already rerun successfully.

A subsequent complete devcontainer conformance run passed all 150 scenarios,
with zero failures, ignored cases or filters, including all eight previously
blocked scenarios. Log SHA-256:
`1c8419c046afb96043510e12f6e562fd02226b97c10b8c477187b568e04bf3a4`.
External tools used the previously authenticated d017 SDK image; this validates
the corrected source/OCI boundary, not a newly accepted SDK or Foundry release.
The separate public-package failure and absent complete producer/publisher
implementation remain production and Pages deployment blockers.

## Release identity and system certificates (OC-02, SY-02)

The host certificate omitted product and secret-reference identities and used
index order instead of dependency order. Corrected certificates satisfy both
real Calculator A/B `SystemReleaseReady` proofs and all 12 individual formal
predicates through LexLean; no formal rule or dependency was removed.

Explicit A and default B passed actual runtime verification with bound receipts.
Restoring default-only selection failed the receipt/build identity assertion;
disabling the mismatch guard failed the expected `PP6101` rejection. Both
mutations were removed byte-for-byte. The two controller tests and the other
122 library tests passed in separate devcontainer invocations, with none ignored.
External-oracle tests used the previously authenticated d017 SDK as tooling,
not as a newly accepted SDK. The positive controller log has SHA-256
`119bde7d6a503bd394061690b413f067ec35cd03c6ccb7ce84ba90e3bb12ab15`.

The separate public `build --locked --release A` attempt stopped at `PP5403`:
the copied fixture lacked an accepted canonical SDK lock. It produced no release
artifact. These regressions do not establish public product-build integration,
complete SDK acceptance, or Foundry deployment.

## Text application integration and artifact confinement

`application_export_imports_every_generated_module` failed against the previous
first-module-only exporter arguments: the declaration module was absent while
its imported type module was present. Importing every generated module made
the regression pass; a separate test preserves single-module arguments.
Two additional passing tests bind the text View to its exact model, core,
entrypoint and byte limits, and reject invalid evaluated labels.

`artifact_publication_rejects_escape_before_creating_files` reproduced a
`../escape.holo` write in an isolated temporary fixture. Publication now checks
every generated path before filesystem effects and again at the write/readback
boundaries. Parent, absolute, dot and empty-segment paths fail; nested output
and create-new overwrite protection pass. Both controller regression tests
passed inside the PrismPM devcontainer as `vscode`.

These are component checks, not complete SDK or Foundry acceptance.

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

## SDK bootstrap registry transport compatibility

At commit `954e83042816f59f4fad767068b43b96e17f439a`, GitHub runs
[Bootstrap Honesty Gate 35034380309](https://github.com/UOR-Foundation/PrismPM/actions/runs/35034380309)
and [Normative Verification & Validation 35034380342](https://github.com/UOR-Foundation/PrismPM/actions/runs/35034380342)
uploaded the SDK layers, then failed with repeated `MANIFEST_INVALID` before
the numbered VV gates began. The previous readiness error concealed a push
failure. CI used Docker Engine 28.0.4; the successful local push used Engine
29.1.3 with the containerd image store and an OCI index. The failed CI request's
media type was not captured, so its exact format remains an inference.

An isolated paired reproduction used the same pinned Zot image,
`ghcr.io/project-zot/zot@sha256:cd2aea942f428630bcb4190542be6abd35e14177aab84fc7ccad0dca8ecb363d`
(Zot 2.1.8). Without compatibility, a valid Docker schema-2 manifest returned
415 `MANIFEST_INVALID`, explicitly naming its unsupported media type. Adding
only [Zot's documented `http.compat = ["docker2s2"]` setting](https://github.com/project-zot/zot/blob/v2.1.8/examples/config-docker-compat.json)
made the identical manifest return 201. Both configurations accepted valid OCI
manifests; accepted manifests retained their exact bytes, media types and
SHA-256 digests. With compatibility enabled, malformed JSON and missing
referenced config blobs were rejected with 400 for both formats.

`scripts/vv.sh` enables this compatibility only in its ephemeral SDK registry.
Before the expensive SDK build, `scripts/registry-smoke.mjs` checks both valid
formats and their malformed/missing-blob negatives against that actual
registry. The tracked preflight was run as `vscode` in the pinned devcontainer:
the default configuration exited 1 at the Docker-format check and the
compatibility-enabled configuration exited 0. It uses the registry's inspected
default-bridge IP; Docker image push/pull still use the daemon-host loopback
endpoint. Push failures now report the failed operation and registry logs.
This is a transport regression check, not an authoritative conformance oracle
or proof of SDK/release acceptance. Prism's OCI artifact validation, independent
upstream OCI oracle configurations, and all remaining VV gates are unchanged.

## Full-gate driver identity isolation

The first normal clean-HEAD `just vv` at
`954e83042816f59f4fad767068b43b96e17f439a` stopped during SDK initialization
because `static.crates.io` temporarily failed DNS resolution while fetching
`indexmap 2.14.0`. The exact URL subsequently returned HTTP 200. One unchanged
retry built and published the current-source SDK at local digest
`sha256:a6ca6a0ef68697755ee7aa109e4d240dba6b386b9290639ebd48340aea59578f`.
Gates 1–7 then passed, including **276 workspace tests**, all **148 SDK-backed
conformance scenarios**, and all **16 fixtures**, with no skipped tests. The
production/property suite passed all 14 tests in 1,051.81 seconds. The nine
earlier SDK-prerequisite failures did not recur. Gate 8 nevertheless stopped
with `PP5001: LexLean verification failed`; gates 8–15 were not accepted.
Both runs' temporary registries and configuration volumes were removed by
the normal cleanup trap, and the source worktree remained unchanged.

A bounded diagnostic `prismpm --json verify` passed with the same build
`416d0d8d2033be2322366eed17fcd26bf4ccc9067d5175216ed496150df96f20` and a
CLI-specific attestation; this did not replace the golden identity or turn
the failed VV into a pass. A live-driver reproduction then established the
failure mechanism: `cargo xtask verify-examples` ran executable SHA256
`667530c0a4d418b86f39a10087524a8e0741ff97d63900ef42e7c08ce5f3bae8`.
Running the full gate's exact compilation flags,
`cargo test --workspace --all-features --locked --offline --no-run`, replaced
its installed path with the all-features executable SHA256
`7e1d696693d4b622c9c8ae291a37d541c52792bd720b0ab75448a2e17f47b9ec`.
The still-running process's `/proc/<pid>/exe` link acquired `(deleted)`, and
the pathname returned by `current_exe()` was no longer readable. The verifier
then reproduced `PP5001`. LexLean correctly refuses to invent or omit its
running-executable provenance.

The VV driver now gives only its nested workspace Clippy/test Commands a
separate persistent target directory. A canonical alternate is selected when
a caller's custom target contains the executing driver; conflicting symlink
aliases fail closed. Before and after each nested gate, both the live
`current_exe()` pathname and its exact SHA-256 must remain unchanged. No
global environment mutation, arbitrary target deletion, test-flag reduction,
LexLean provenance change, or attestation normalization is used.

Four focused tests passed for changed/deleted/same-content-replaced driver
rejection, custom-target and symlink isolation, and exact child flags/scoped
environment. The actual isolated all-features no-run reproduction preserved
the live driver pathname, inode and SHA256
`c4bb31116cb1d7329d43360bdcc10cc8da59c232894fe7e01ec70630f56ab988`.
Normal `verify-examples` then passed with stdlib attestation
`85367a8165bc8e9c7718c863e828a376d25e22c1b1cc56c5e42eb612fb5aed25`
and repeated Calculator attestation
`b8bff350326d7f45297fafcc168f7e00f78b0cec4de4a2addc175c9ddeb8d88d`.
Both build identities remained unchanged. Source/model/SPEC audits, formatting
and denied-warning all-target/all-feature xtask Clippy also passed.
Fresh golden generation and an independent comparison accepted all 234 files
with that same final driver. Only the executing-binary hash, derived
attestation/hash bindings and review reason changed in three golden JSON files;
the formal model, IR, execution corpus, package source, crate archive and
release metadata remained byte-identical.
Evidence is retained under `target/control-coverage-driver-*`; original full
run logs remain `target/control-coverage-vv-954e830{,-retry}.log`. These are
focused regression results, not completion of the remaining full VV gates.

## CI bootstrap source history

At `d8675e0`, [Verification & Validation 35040425560](https://github.com/UOR-Foundation/PrismPM/actions/runs/35040425560)
and [Bootstrap Honesty Gate 35040425613](https://github.com/UOR-Foundation/PrismPM/actions/runs/35040425613)
passed the SDK registry preflight/build/push and VV gates 1–3, then failed in
gate 4: `git archive` could not find the exact accepted bootstrap source
`f378fd3a8dc5711cb4b22cec9ee2f874353628c3`. Both workflows had checkout's
default shallow history. The source pin was valid and present in a full clone;
the failure was missing checkout input, not a failed model assertion.

The two workflows now request full history and check that exact source tree
before building the devcontainer. Direct `just vv` performs the same preflight
before SDK initialization. `bootstrap-verify.sh --check-source` checks only
source availability and emits no acceptance evidence; the normal no-argument
bootstrap verification, accepted SDK archive and source revision are unchanged.

A fresh depth-1 clone in the pinned devcontainer reproduced the exact
`not a tree object` error. The new preflight rejected it with an actionable
diagnostic before creating verification artifacts; unknown arguments also
failed. Fetching full history made the same check pass and reproduced the
historical archive byte-for-byte (SHA256
`5ee73267db9a7b9f3624e0d08ac85056b5a7382024e0c2b54f0dcf73cdd96db8`).
Normal complete bootstrap verification then passed for unchanged production
semantic identity `11bfa1b262f77554964c40ffaa1fc3f8f3dfbfeb66535b5cc4b0b7f68d361c29`.
Shell syntax, workflow YAML/step ordering, and existing model/SPEC/source audits
passed. Evidence is retained under `target/bootstrap-history-*`. No compiler,
model, generated package, golden or verification-gate semantics changed, and
these focused results do not establish completion of the remaining full VV.

## External-oracle input ownership

[CI run 35047052174](https://github.com/UOR-Foundation/PrismPM/actions/runs/35047052174)
failed because the non-root staging process could not chmod a copied input
owned by a different UID. Commit `208b9bc14428d5a96b95bcfb5c93d977554c38e9`
replaces that process with a deterministic archive: UID/GID 1000, directories
0555, files 0444, fixed timestamps and unchanged contents. Docker copies it
into a never-started holder. The executing oracle remains explicitly UID 1000,
network-denied, capability-free, and read-only, including its input volume.

The devcontainer's `vscode` user reproduced UID 1001 input ownership causing
UID 1000 chmod to fail with `Operation not permitted`. Normalized staging then
preserved bytes, owner and mode; the isolated oracle read the input and failed
to mutate it. Two archive unit tests reject escaping/duplicate paths, symlinks
and special files and verify metadata-independent bytes. Those tests, the
sandbox-argument test, all-target Clippy, and the complete existing locked
external-oracle corpus test passed; the latter took 18.20 seconds.

These component checks used the cached SDK image
`sha256:a6ca6a0ef68697755ee7aa109e4d240dba6b386b9290639ebd48340aea59578f`,
genuinely pushed to an owned ephemeral pinned-Zot registry to obtain its
distribution reference. Test containers, volumes, registry and temporary tag
were removed. The ignored `target/authority-staging-regression/` retains the
reproducer. This is not acceptance of the newly built SDK; final wrapper-bound
standards evidence and all full gates must be regenerated and pass.

## Development-only SDK candidate publication

Task 10.8 permits an immutable release-candidate SDK for model development;
it does not grant production acceptance or waive public Cargo qualification.
The separate `sdk-candidate.yml` workflow builds `sdk/Dockerfile`'s unchanged
`runtime` target from one exact main revision on native Linux amd64 and arm64.
Image version, source labels, and `SOURCE_DATE_EPOCH=0` match the production
recipe. Candidate identity is recorded outside the image, so registry copying
can preserve its bytes; no future acceptance or byte-equal rebuild is assumed.

Unprivileged build jobs transport their OCI-layout artifacts through the exact
pinned Zot registry and test those distribution digests as UID 1000, offline:
SDK inventory/CLI operation, the modeled Calculator's `check`, and rejection
of a shadowed tool with `PP5401`. Each retains the real platform inventory,
manifest/config, standards lock, test results, and an SPDX package inventory.
These are scoped development checks, not the full VV or formal acceptance gate.

Publication requires the pre-existing `sdk-candidate` environment with exactly
the `main` branch rule; GitHub enforces any configured reviewers. Its policy
is checked before building and again before registry login. The separate
credentialed job never runs candidate code: it verifies the source/platform
bindings, copies the tested OCI bytes to
`ghcr.io/uor-foundation/prismpm-sdk-candidate`, assembles the two-platform index,
and attaches source-bound build provenance and per-platform SBOM attestations.
Only `sha-<commit>` discovery tags are written; consumers use the resulting
manifest digest. No production tag, accepted promotion, or Cargo upload occurs.
`release.yml` and all full VV requirements remain unchanged.

Local devcontainer tests exercise exact revision/platform binding, refusal of
missing or weakened environment protection, incomplete/swapped/stale evidence,
and genuine ORAS inspection of valid and tampered OCI fixtures. A successful
test run is not evidence that the hosted workflow has published an image.
After a reviewed hosted run, the existing template bootstrap renderer must
derive locks from the actual digest-selected inventories; no digest or
cross-platform inventory equality may be invented. Production release remains
blocked until its independent public dependency and complete acceptance gates
pass for the exact bytes proposed for promotion.

The `sdk-candidate` environment's real GitHub metadata passed the main-only
policy validator. No independent reviewer requirement was invented; existing
reviewer rules remain GitHub-enforced. Both ORAS 1.3.0 native installer hashes
were checked against the official release checksum file and the SDK recipe.
Six candidate test groups passed as the devcontainer's `vscode` user,
including real offline ORAS OCI inspection, transfer, index assembly and
tampering rejection. The hosted publishing workflow has not yet run.
Compiler inventory metadata is derived from the exact registered dependency
revision and checked again when the runtime inventory is assembled; old
revision strings are rejected even if artifact bytes are otherwise present.
The corpus version identifies the stable `prismpm/ids/1` schema, not changing
feature or diagnostic counts. Negative cases reject stale authority metadata.

## Platform-indexed SDK inventory correction

The legacy lock rendered a single native inventory beneath a multi-platform
image identity. Native binaries necessarily differ, so that lock cannot
honestly stand for both architectures. The additive `prismpm/sdk-lock/2`
contract preserves `/1` validation and binds exact index bytes, both native
child manifests, and their distinct actual inventory hashes and artifact rows.
The SDK-owned bootstrap helper reads files copied from those image children;
the template does not execute foreign-architecture binaries or fabricate pins.
The SBOM retains both platform closures with distinct component identities.

The three Rust `sdk_lock_platforms` tests passed inside the devcontainer:
exact index membership; both correct native inventories; and unchanged strict
legacy comparison. They reject missing/duplicate/swapped platforms, changed
index bytes, unequal artifact identities, wrong architectures and partial
inventories. The DK-01 conformance case exercises the registered `/2` parser
and both native selections. Two Node bootstrap test groups passed with explicit
synthetic fixtures, including swapped inventories, wrong child-image metadata,
missing files and different standards locks. These fixtures are not publication
evidence. The full VV SDK boundary runs the candidate and platform-lock Node
suites offline as UID 1000 with the SDK's exact ORAS; workspace tests also run
the Rust negative cases. The first source regression attempt was blocked by
unrelated in-progress compiler imports and is not counted as falsification.

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

## Multi-platform SDK lock update component verification

The new `sdk-lock-update/2` path was verified in the configured PrismPM
devcontainer as `vscode`. It captures the exact OCI parent index and both child
inventories without starting either target image, checks the requested standards
digest, and produces a review-required proposal without adopting project files.
Legacy `/1` proposal behavior remains separately tested. The command register
records its Docker image-cache side effect; it does not claim to be globally
side-effect-free.

Focused results: four SDK Rust unit tests, three platform-lock integration tests,
ten model tests, four Node platform helper tests, four driver-isolation tests,
DK-01 conformance, and workspace all-target Clippy with warnings denied passed
in the source devcontainer. The combined candidate/platform Node suite initially
reported nine passes and one missing-ORAS prerequisite there. All ten then passed
with current sources mounted read-only into the existing SDK tooling image
`sha256:a6ca6a0ef68697755ee7aa109e4d240dba6b386b9290639ebd48340aea59578f`
as UID 1000 with networking disabled. This uses its installed tools, not its old
inventory as evidence for the current release. Capture mutations reject changed
index bytes, failed pulls/copies,
substituted child digests or architectures, mismatched standards, and symlinked
evidence; cleanup targets only the containers created by that capture.

`node sdk/platform-lock.integration.mjs target/debug/prismpm` also passed against
an isolated pinned Zot registry, using two generations of actual amd64/arm64
OCI transport-fixture images. Their inventory digests hash files copied into
those images; they are explicitly synthetic fixtures, not SDK releases or claims
that fixture commands execute. The current CLI returned all four changed fields
and the complete captured target lock; a wrong standards digest failed with
PP5401; both paths preserved the original lock bytes. The test rejects Docker
manifest lists and indexes with extra attestation descriptors rather than
loosening the required OCI two-platform shape. Actual execution exposed and
corrected a trailing newline in the capture producer; the canonical lock parser
was not weakened.

Ordinary VV now runs that real OCI regression with a CLI built through the
existing isolated Cargo target and driver identity guard. It intentionally runs
in the source-bootstrap environment: the shipped SDK correctly refuses a
synthetic current native inventory. The separate shipped-SDK runtime gate retains
its actual-image inventory checks. Temporary registry/container/config-volume
and fixture-image references are removed; logs remain ignored under
`target/sdk-update-v2-*.log`. These are component results, not a full VV or
production SDK-release acceptance claim.

## Text profile acceptance accounting and diagnostic boundary

The four-module HO-11 fixture passed actual native build and full verification
with compiler `ecd32508bb2e22c16188c54b12e7a0247507905e`, using the real
stdlib byte-length, append, and UTF-8 wrappers. All six modeled vectors passed
the native consumer and Hologram/Core-Wasm paths; generated browser artifact
verification is not a DOM-browser test. The unchanged test executable hashed
`3f460fcd6d4affe185a4fd72f9508a6e5bfa635d31e0ce6217b1269a163d647c`.
Build `7b03de371c913fde58ee0a75e2c26879d3a54001fcee57ba1260f3e11bac0581`
verified as `9c1ab3baa1cb0c9e56adedf567966ee848384ab6d059426007a1b88854aa29c5`.
Logs: `target/text-native-namefix-{build,verify}.json`; both stderr files empty.

That execution exposed a display-name/IR-identifier mismatch. The new parser
regression first failed on `Text Request`; text profiles now derive their IR
module from the Cargo identity, while the legacy Calculator path is unchanged.
Five focused application-builder tests pass, including names with spaces,
punctuation, and a leading digit. This is component evidence, not SDK release
or Foundry acceptance.

The current registers contain 149 features and 84 diagnostics. Acceptance
results now derive counts from the checked transcript, and the runner binds
its exact compiled register bytes. Focused regressions reject stale counts,
missing HO-11 or PP2009 even with adjusted counts, duplicate or skipped cases,
unregistered identities, and treating a partial `passed` transcript as accepted.
PP2009's positive and malformed specimens call the actual text-application
validator and preserve its real diagnostic. The focused component tests and
HO-11/PP2009 runner slice passed; the slice uses synthetic component identities
and is not an attachable production-acceptance result.

The other 83 entries in `diagnostics.rs` still use generic local `Rule`
predicates. Their execution establishes registry accounting and those predicate
results, not that each actual subsystem boundary emitted its public diagnostic.
This change does not prove that missing implementation-level coverage. Full SDK
acceptance still requires each such diagnostic to have real positive and
malformed-input execution against its owning subsystem, with the emitted code
and evidence bound to that execution. Count closure is not a substitute.

## SDK inventory evidence and execution binding

SDK lock `/2` now retains each platform's exact canonical inventory document,
including its optional final newline. Both documents must match their declared
SHA-256 and complete artifact rows. Closed command metadata, missing or swapped
documents, changed non-native rows, and changed digests are rejected. This binds
the recorded evidence consistently; it does not independently prove OCI layer
membership offline. The immutable child-image capture remains the source of
those bytes. Each inventory is bounded to 8 MiB, the index to 1 MiB, and update
capture output to 64 MiB to accommodate JSON escaping and repeated bounded rows.
The public aggregate contract matches that 64 MiB lock budget and permits
192 MiB for an update's old/new field evidence and complete proposed lock.
An actual canonical lock larger than the former 8 MiB aggregate limit and an
update larger than its former 32 MiB limit pass the public parsers; byte buffers
above the new aggregate limits fail before JSON decoding. Per-platform document
and index bounds remain unchanged. This regression failed at the old lock limit
before the aggregate-contract correction.

Both SDK lock versions are checked against the actual native inventory at SDK
inspection, template checking, acceptance, supply-chain generation, and existing
project-lock loading. An installed SDK's fixed inventory cannot be disabled by
an environment override or deleted inventory file. The existing SDK profile
marks an installed SDK; the source devcontainer's oracle-cache directory alone
does not. Unbound source-bootstrap examples and inert historical parsing retain
their distinct roles rather than claiming current SDK execution.
Lock presence uses symlink metadata: only a genuine missing file permits initial
bootstrap. Dangling/external symlinks, directories, and other inspection errors
fail with PP5401 instead of silently dropping the binding. A real red/green
regression reproduced the dangling-link bypass before this correction.

Initial isolated subprocess regressions reproduced wrong-native-inventory
acceptance before the correction; five public execution-boundary tests now
reject it with PP5401. Eight SDK unit tests, five platform-lock integration
tests, five Node test groups, and all-target Clippy for `prismpm` and
`repo-conformance` passed in the devcontainer. The whole library's 106 tests
also passed with the genuinely pushed old SDK oracle component image before
the additional transport-budget test. These remain component checks, not a
claim that an unpublished candidate or the full SDK release has been accepted.

## Portable View execution and tool integrity (HO-12)

The new oracle runs the archive's actual portable HTML/CSS/JavaScript in pinned
Chromium through the authoritative Hologram session, intent handler and
Core-Wasm. Its loopback Axum host is acceptance infrastructure, not a production
backend. HO-12 calls the owning PP5301 validator; synthetic report mutations
prove rejection, not browser execution. The current register is 150 features
and 84 diagnostics. Report `/2` binds all three current Holo identities and
requires the exact profile cases, applicable vector indices, actual engine,
one attempt per case, zero skips and zero retries. Declared requests above the
upstream 64 KiB intent limit or responses above its 1 MiB output limit fail
before invocation or model-sized probe allocation; no vectors are clamped away.

Real Chromium first exposed Calculator's native form GET disclosure when
JavaScript was unavailable. No host CSP masked the attachment defect. The
reviewed compiler correction is pinned at
`5fd0c82a70019e8033f2a6f449f3c919a4e37151`; full compiler CI passed, including
eight numeric and eleven Text browser tests. An independently planted
oracle-transport defect accepting a forged Origin also failed the real browser
suite; restoring the normal host restored byte-identical Text reports with
empty stderr. A separate labeled DOM-boundary mutation rendering the injected
markup response through innerHTML also failed the real text-rendering probe;
the restored driver produced a byte-identical report to the final public CLI.
Lifecycle evidence replays the same successful modeled intent
immediately before stop and then requires that request to be rejected after
stop. This does not substitute a hardcoded numeric request for a Text request.

The first complete public CLI runs correctly failed PP5301 because the scrubbed
subprocess environment did not pass a caller's browser-cache override. The
final correction passes verified absolute Node and browser executables and
checks installed Playwright driver/core and headless-shell tree bytes before
loading JavaScript. Source-bootstrap digests were independently reproduced
from locked npm inputs, a never-started checksum-pinned browser image and the
checksum-verified official Node archive. SDK execution instead requires its
native inventory's corresponding artifact digests. Actual modified driver,
PATH-shadowed Node, symlink, writable-file, oversized-file and malformed
inventory-row regressions reject the altered inputs.

Both final public `check`, `build` and `verify` sequences passed in the x64
source devcontainer using immutable CLI SHA256
`6f70b32a0afe6204ffcd150213e05bf4d4ab45d04f53f42df1e765ee1b63895f`,
with the deliberately incorrect caller browser path
`/tmp/forbidden-browser-override`:

- Calculator: build `b1abf58b789f0dd6013e8494ef67f8195f1ea79c6fcb746265f712d3cc6fb70b`,
  attestation `482530227a036eb54f904984d70e0726d6fb6eb663a548c1b0908d1f5c130194`;
  all eight actual numeric Chromium cases, browser vectors 0–14, 22 direct and
  resident vectors, and 21 UTF-8 intents passed.
- Text Request: build `00b3479c6d4962bf78913e21aa1241598cb8456143b212b74fcecb8141913974`,
  attestation `2ab666425e7b690a6c3609b5b115d63f3b2a533e848c8ea2f83f2640ba53d489`;
  all ten actual Text Chromium cases, browser vectors `[0,2,3]`, six direct and
  resident vectors, and five UTF-8 intents passed.

The copied CLI remained unchanged and oracle stderr was empty. Focused owning
validator/tool-integrity tests, all-target/all-feature Clippy, standalone
harness Clippy, Dockerfile build checks and syntax checks passed. Raw evidence
is retained under `target/portable-*-integrity-*` in the isolated development
worktree. These component results do not establish hosted SDK execution on
both architectures.

On 16 September 2026, complete source-devcontainer `just vv` at clean
`d1b8506876c28baf8277ad7e179f92edd31fe3a0` passed gates 1–14: 327 workspace
tests, 150 conformance cases within that count, 18 fixtures, repeated
Calculator/Text builds and actual eight/ten browser cases, 240 goldens, and
194 build plus nine verification artifacts identical across two absolute roots.
The canonical verifier remained
`5bf0bb397b9f64cab668438c012d0d683dfe5cd98004b31c15c2e325b4d1b701`.
External tool-corpus checks used the independently authenticated `d0174e1`
development SDK; portable browser checks used independently pinned source
tools. This is not execution of a newly published SDK.

Gate 15 verified the stdlib package, then failed the real downstream
`cargo check --offline`: `no matching package named uor-hologram found`.
A separate fresh official sparse-index request returned HTTP 404. The run
exited 1 without `target/vv-evidence.json`; no full-pass or release receipt
was produced. Log `target/portable-primary-vv-d1b8506.log` has SHA-256
`0cd4c0754c1875453f28a5f72b056d06d8a99e3dc04d4cd8167ed52e499af0ab`.

## Independent Hologram Calculator/Text interoperability acceptance

Independent Hologram oracle interoperability acceptance for Calculator and
Text applications is executed and digest-bound to release-closure records:

1. **Pinned oracle source and harness inputs**:
   - Upstream live source archive `crates/prismpm/vendor/hologram-live.tar` matches SHA-256 `caf5c34ef2b21d58c1aa12acf81cb13ace1adaffb3c69a641f54f490ed61cf66`.
   - Embedded oracle harness manifests (`hologram-oracle.Cargo.toml`, `hologram-oracle.Cargo.lock`, `hologram-oracle.main.rs`, `hologram-oracle.browser.mjs`) match pinned checksums and the independent oracle test harness.
   - Holo codec oracle manifests (`tests/holo-codec-oracle/Cargo.toml`, `Cargo.lock`) match pinned checksums and verify the frozen wire corpus against upstream revision `2bda6a9a9476872dade705bd61ece4209607f6da`.
2. **Calculator interoperability acceptance**:
   - Verified against schema `prismpm/hologram-oracle/2`: 22 direct vectors, 22 resident vectors, 21 UTF-8 intents, verified footer, verified guest allocation boundary, View attached/detached exactly once.
   - Headless Chromium portable-browser execution passed all 8 legacy numeric cases (`attachment-assets`, `modeled-vectors`, `input-validation-recovery`, `transport-failure-recovery`, `pre-init-privacy`, `delayed-init`, `intent-boundaries`, `detached-session`) with 0 skips and 0 retries.
3. **Text application interoperability acceptance**:
   - Verified against schema `prismpm/hologram-oracle/2`: 6 direct vectors, 6 resident vectors, 5 UTF-8 intents, verified footer, verified guest allocation boundary, View attached/detached exactly once.
   - Headless Chromium portable-browser execution passed all 10 UTF-8 text cases (including `text-response-bounds` and `text-safe-rendering`) with 0 skips and 0 retries.
4. **Non-vacuous failure probes**:
   - Rejection of stale/wrong schema editions (`prismpm/hologram-oracle/1` and unknown versions).
   - Rejection of mismatched or substituted application and archive identities (`application_kappa`, `archive_kappa`, `archive_fingerprint`).
   - Rejection of tampered vector counts, unverified footers, boundary failures, and failed/retried/skipped browser cases.
   - Fail-fast rejection of declared request/response limits exceeding pinned transport bounds (64 KiB / 1 MiB).
5. **Oracle isolation**:
   - Interoperability checks remain isolated validation oracles, not application authority, and run offline from locked inputs.

## Compiler-bound bootstrap compatibility

The original bootstrap equality check rejected the reviewed compiler update:
LexLean deliberately incorporates compiler semantics in `semantic_id`.
Closed evidence `/2` preserves the accepted `/1` contract and compares complete
linked snapshots, lexicon closures and projected model content, while separately
binding the genuine prior/current compiler, emitter, source and artifact IDs.
Actual check/build artifacts, their complete manifest closure, all tracked
source bytes and the modeled source-manifest definitions are verified.

The x64 source-devcontainer component run produced receipt
`d9e82d12088db37032c3699462d4ecca5d86f32f73fcca9058b84adc4782ffe0`:
the accepted 0.2.0 archive and current compiler agreed on the full 26-module
compatibility projection, with 18 entities. A separate current production
check passed. Independent review reconstructed every projection source from
the accepted historical tree and separately rehashed the captures and manifest.

Nine owning Node test groups, both Rust bootstrap contract tests, DK-05,
workspace all-target Clippy and normal model validation passed. Negatives cover
resealed semantic/proof/closure/domain/facet drift, substituted identities,
ambiguous locks, source changes, missing/extra/tampered artifacts, symlinks,
FIFOs, oversized files, duplicate/noncanonical JSON and untracked helpers.
A genuine isolated source mutation changed `Foundation.Core.portableTrue`
from true to false, then passed normal current check/build with the same entity
count; the compatibility validator rejected its complete semantic content.
Restoring the source reproduced the entire current capture byte-identically.
Probe log `target/bootstrap-semantic-probe.log` has SHA-256
`ef8c80aefb54469c47b76218d1353d1b25a55dd4b49338a0153fafdf613ebd1e`.
These are component checks, not a full VV or public SDK release receipt.

## Verified semantic compiler integration

LexLean `ee18ad907039a82ff5b11ff2117d6dfe95d80365` passed its complete
[upstream acceptance gate](https://github.com/afflom/LexLean/actions/runs/35112304941),
including all 222 conformance cases and the complete Atlas verification.
Two clean-source, offline Cargo package runs reproduced archive SHA-256
`1b39ba47a7d013a79f5463fc49ab891ef4ca62402b05cb5a2059191bb3d4b8dd`.
The vendored tree matches that exact package and its source revision.
Normal LexLean locking updated 19 compiler identities; the deliberately stale
negative fixture and every formal/application source remain unchanged.

Normal stdlib generation and an independent check reproduced the same
pre-seal attestation `6f4343171cee8d7341405c374da03778d735dc29d80a5ffac50c693802002fb2`.
The complete exported LCNF, generated package and all three release crate
archives remained byte-identical. Only the compiler-bound stdlib semantic
identity changed to `9d8880e4ab270f05f668bdbe6ad8440ec225dc32fec5b8987f086d2a2197f53a`.
Log `target/compiler-ee18ad9-stdlib.log` has SHA-256
`e98f873051adbbc2b5d353980e328becc1b41844344ef1c94a20a7e3fb34ff97`.
These component results do not establish a new SDK or Foundry acceptance.

The complete source-devcontainer `just vv` ran on clean commit
`47b7a8e6c8ff1aa4cbda8223ba22d02056b12fff`. Gates 1–14 passed, including
all 150 conformance cases, repeated Calculator/Text Request verification,
240 golden files, and 194 build plus nine verification artifacts reproduced
byte-for-byte across two absolute roots. The authenticated `d0174e1` SDK
supplied external oracle tools only; changed compiler and application checks
used current source. This does not qualify a new shipped SDK.

Gate 15 checked the stdlib package, then downstream `cargo check --offline`
failed because the public crates.io index lacks `uor-hologram`; a separate
HTTPS sparse-index request also returned 404. No dependency substitution or
gate waiver was used, and `target/vv-evidence.json` was absent. Log
`target/compiler-ee18ad9-vv-47b7a8e.log` has SHA-256
`7445dbd9d73c6757823d0cea6e3a7baa897f355a11b9de6c62faf8a5e062aa95`.

## Modeled browser and archive prerequisites

The 17 September source-devcontainer checks below are component evidence, not
a shipped SDK, Foundry application, Pages deployment, or full VV receipt.

| Boundary | Executed evidence |
| --- | --- |
| Holo/1 codec | All 57 modeled vectors matched generated native std/no_std execution; integrated archive and wire tests passed against the frozen independent upstream fixture, including resealed malformed archives. |
| Physical archive limit | An actual Calculator build failed with PP1003 at one byte below its physical archive size, preserved the prior build, and succeeded at the exact limit. |
| Browser host primitives | 48 identity, IndexedDB, boundary and two-context WebRTC tests passed without skips, including direct Chromium identity conformance. These are generic host facilities, not Kappa discovery or internet availability. |
| Workspace reducer | All 45 modeled cases replayed twice through generated std, no_std and Core-Wasm. A generated-native 1,024-event history reached the exact maximum response before rejecting the next event. |
| Read-only SDK inputs | The Workspace gate passed with source writes denied and a hostile inherited Cargo target overridden. Full Corpus Lean C generation passed; maximum Wasm memory was 28,180,480 bytes within 32 MiB. |
| Cold oracle closure | An initially empty Cargo cache acquired both pinned oracle graphs; subsequent offline builds reproduced the wire fixture and replayed retained Calculator artifacts, including eight actual browser cases. This is not acceptance of newly generated application artifacts. |

Ordinary `stdlib-package --write` verified all 42 modules and reproduced all
seven generated package files byte-for-byte against an independent bootstrap.
Its attestation is `d32fbcc639e69b4ed393204b57ae2c603c0cc6f22cee973f0c70165b0960c298`;
the exported IR is `43adea717fd65367f95e57ddf031dafbaa39ab8f5f4ba255fa63a7058b36865d`.
The exact modeled source remains authoritative; finite corpus agreement is not
a universal protocol-equivalence or authentication proof.

Retained local log SHA-256 values:

- `target/stdlib-ordinary-write.log`: `871108f3685ce68995bad9448c1e6ea8cfa4c03882a2aae9c4d9209c75e8731d`.
- `target/holo-archive-modeled-focused.log`: `35fb407901f3cd8f8eba0172ac659a6d66998dfbc1760cca31c38df2f37b17b2`.
- `target/browser-host-acceptance-with-chromium.tap`: `2fc48ce936db0c4d16622e9df099e32fa9c065c7ef68b9249d43e4563818c001`.
- `target/browser-workspace-reachable-readonly.log`: `e1291af949988a60de4a45e0579b66a539f1f34d0e0b3e434742201ffb6991e2`.

The DK-07 Chromium gate also rejected an isolated-copy mutation that made
signature verification return true. Its semantic rejection test failed while
the other three browser tests passed; the unchanged production module then
passed the complete suite. Log `target/identity-browser-mutant.log` has SHA-256
`253e45293c7ba619c14e7bde2a20b3791c90c7c6659df3e73b11d235ad2a2916`.

Source-free native verification rejects malformed process evidence before
expensive source reconstruction, without omitting semantic, proof or artifact
checks. The full native OCI test passed; ignoring semantic-validation failure
then made the altered-source test fail on false acceptance. Correct source was
restored byte-for-byte; primitive tests and full-workspace Clippy passed.
Baseline log `target/native-early-rejection-baseline.log` has SHA-256
`1b0432259ac20a68763807d59c130f187f8db3b5df80eca3a7765ad08c58adde`;
mutant log `target/native-early-rejection-mutant.log` has SHA-256
`48ba6e706c3c0840d015496ca2e1c2980e2adb40585c8d39a1fe21520e338b05`.

The devcontainer readiness gate passed all 15 real-container tests for image
and remapped users. A deterministic group/gshadow interleaving reproduced the
hosted restart failure; the helper now waits for noninteractive `sg`
authorization within the existing deadline. Commands remained non-root and
ran exactly once, including exit 17; withheld authorization timed out without
execution or password prompts. Socket permissions were unchanged. An isolated
old-helper mutation failed both new authorization cases for each user; source
files were never mutated and disposable containers were removed. Green log
`target/devcontainer-shadow-race-green.log` has SHA-256
`6923c7879c4f95d2d45d3c9d59703100540ef569046b1530de9d6714f3820ad3`;
mutant log `target/devcontainer-shadow-race-mutant.log` has SHA-256
`3eb92ea5fc8221ab66e7d759be7c7f483ec0d3b3b2657355b1ac214386e69aca`.

The subsequent complete offline workspace suite passed all 390 tests, with no
failures, ignored tests or filtered tests, including actual OCI/native and
source-free export checks. The separate package-API gate reproduced all three
archives and compiled the selected downstream package. Logs
`target/modeled-stdlib-workspace-oracles-current.log` and
`target/modeled-stdlib-package-api-fixed.log` have SHA-256
`7a6606e5cb81eb908bc79feeb179a9811a04598942e56702e5db27156690cd3d` and
`17d54e8aafab3e949f087bc01b1b44d87cf221a295ec22dae90b81cd840c618b`.
These precede the AsyncAPI runtime change below; they are not a full VV receipt.

## Owned AsyncAPI runtime

The unchanged upstream example harness uses parser 3.6.0 with the separately
owned runtime lock `5bd20ce206d3b3b76a7034951c9e19e15291c54eec0a1424139b465c980f1205`.
It is not a replay of the historical lock or an upstream-reviewed dependency
update. Lifecycle scripts are disabled. The installed-graph audit reported
zero findings; the retained historical-lock findings and whole-image acceptance
requirements remain unchanged.

The actual owning Rust gate passed all 24 documents, 89 embedded examples,
both negative probes and five runtime-integrity groups. It binds the installed
byte tree and actual launcher to the fixed SDK inventory; altered source,
dependencies, resolution, inventory and ambient preloads fail. A real Node run
that exited zero after skipping every group is explicitly rejected.
The development-only image was
`127.0.0.1:5000/prismpm-asyncapi-oracle-test@sha256:4de2cc5dd62c67e899e8941d87cfb9bf2d457af874890b87b9246fd6b8b2d459`;
it is not an accepted or published SDK.

Formatting, full-workspace/all-target/all-feature Clippy, model validation,
locked authority resolution and the actual package-API gate passed. Cargo
selected the TAP fixture and all five runtime assets; downstream compilation
passed. The 9,555-file source/package closure and all four stdlib seals remained
byte-identical across the package gate.

Normal golden generation and a separate full recheck matched all 264 files.
The reviewed three-file change binds the rebuilt in-process `xtask` executable
(`12071eba91640c3bc0ebfe186c1d985b9ada242d3a47faf077fa4a03db4a3290`)
and derived attestations. All formal sources, generated Lean, build files and
other verified artifacts remain byte-identical. The recheck log
`target/asyncapi-cohort-golden-recheck.log` has SHA-256
`b09f02b307eb1721a6874c0240a074928fbca7c0bdf95b08d710846d191ec8f9`.
No executable identity was removed or normalized.

Retained log SHA-256 values:

- `target/asyncapi-owning-rust-regression.log`: `1897578e056692608a121f0080ea3bb087b4c4ad98835a55f9be0a7fa8507d65`.
- `target/asyncapi-cohort-package-api.log`: `3e82472241cab075227aa424f0f3ebd8674fdccd103d0245058b7649294a64c5`.
- `target/asyncapi-separate-runtime-installed-audit.json`: `b8fde891683991e0012186fbc02104eb8cd7466830fed82e4ff975d11c03d404`.

## Signed envelopes and authenticated browser journals (DK-11, DK-12)

Both registered gates failed on their absent implementations before activation.
The complete owning gates now pass after fresh LexLean verification and normal
Lean C generation: 43 envelope vectors and 61 journal vectors execute twice on
generated native/no_std and Core-Wasm.
Actual browser cryptography, durable replay, storage faults and competing CAS
writes pass; signature, delayed-capture, CAS and transcript mutants fail their
owning assertions. Both complete 1,024-event Grant/Post histories pass native
replay. Maximum measured journal guest memory is 39,518,208 bytes within its
640-page limit; the existing Workspace limit is unchanged.

Compiler-environment mutations are rejected before child execution. A stubborn
descendant regression first exposed incomplete process cleanup; the corrected
runner terminates only its own isolated process group. All 48 existing browser
host tests pass. The browser-driver dependency acquisition now checks exact
manifest/lock bytes; its omitted-graph mutation failed before the correction.

The read-only, network-disabled rerun passed all 18 envelope/journal Node tests
without skips. It used the development oracle image recorded above, supplemented
with read-only current public Cargo caches and locked SDK oracle dependencies.
This is component evidence, not acceptance of a new SDK image, peer replication,
Kappa, organizational identity, a portal or availability guarantees.

Retained log SHA-256 values:

- `target/journal-integration-dk11-final.log`: `20a607517c53af389bc47d6f2818c1a2eb76327be3220c1e0d2f644edf2c95c4`.
- `target/journal-integration-dk12-final.log`: `d39f99c4dfbe6c317aae535c116ee0f1b537a872cb9fd756c1e0f5c3e8a823e3`.
- `target/journal-integration-readonly-closure.log`: `09f667c5232f574ba5a23c3c3de890dd7010fe10c877e771121574d584c74a95`.
- `target/journal-integration-host-regression.log`: `0697d5f53f5121cbd6f3945cbe4b098f6a79439dfe0577196cc2587a6ae48cf8`.

### Workspace corpus and package regression

The unchanged full package check exposed a 300-second Lean C-generation
timeout in WorkspaceCorpus. Factoring repeated literal data into bounded
helpers preserves all 45 probes and all 90 expanded request/response values
(4,502,371 bytes); the reducer IR and generated crate bytes are unchanged.
The complete DK-10 gate now passes in 178.72 seconds without raising timeouts
or reducing the corpus. An unused-helper mutation fails its owning assertion;
large byte mismatches fail with bounded diagnostics. Checks interrupted by the
host restart were rerun, not counted as passes.

The independent package-API gate passes fresh verification, exact stdlib
regeneration, Cargo-selected downstream compilation and all three archive
reproductions. The semantic identity is
`bf5f8375d421f8d2cb38724d03a16c31dca703b302a141831a25c649dd808859`;
the checked stdlib IR remains
`e4906b9dd20f7cb6f4d11775708fdf2574eee7d60ad8389f6ccd38333c314fea`.
Authored-source formatting, all-feature Clippy and source/model audits pass.
These results are not full SDK VV or product acceptance.

Normal regeneration and independent replay match all 288 golden files. The
four new modules add 24 records; 205 prior records are unchanged and none are
removed. Existing generated Lean changes only in Runtime imports and the
byte-preserving WorkspaceCorpus factoring. Unrelated source-map contents
retain their entries and change only source/semantic identities.

An earlier replay failed after a concurrent Cargo build replaced the verifier
executable; it is not acceptance evidence. Both successful runs kept the normal
verifier byte-identical; workspace compilation now uses a separate target.

- `target/journal-integration-dk10-post-restart.log`: `0232773de552666630d4551870c7aad5eb93f2181a2f6a1450d337642659e64a`.
- `target/journal-integration-package-api-post-restart.log`: `d2a2fb3aa4c7b1a6604ac6d37b0a88c92902ca2ac8e49bff4b4d7f44d9652cae`.
- `target/journal-integration-golden-write-stable.log`: `7b0d602fe8a50c730b4632537fabd66bba28b0caa79d8dd808d13b500d8b5a75`.
- `target/journal-integration-golden-check-stable.log`: `53c3f5ab595d448b2c543c34d55cdff6b7a9a4c32d8524dc7d2429cfdd15a6a6`.

### Integration regression and bounded admission

Full workspace execution exposed a Journal fixture-authoring stack overflow
after Workspace literal factoring. Restricting reuse to semantic fixture
boundaries restores byte-exact Journal corpus generation, all 61 vectors and
both complete histories; no stack, timeout or domain bound was raised.

The refreshed Journal gate passes all 13 tests without skips, including fresh
kernel/C/native/no_std/Wasm execution, both full-history append/replay paths,
34 storage faults, eight concurrency cases and all five deliberate defects.
The host now admits two outstanding append/refresh operations and rejects
overflow before capture or effects. All 18 browser diagnostics are registered;
the ten model tests include rejection of an omitted `journal-busy` entry.
Final authoring checks also reproduce the exact committed corpus bytes.
These are component results, not complete SDK or Foundry acceptance.

- `target/journal-authoring-red.log`: `830ee2c95a7fb4afcc7ba32d6e18da2b1bfe6612bb9f117d6864b54b4bcb1028`.
- `target/journal-final-authoring-admission-full.log`: `428916514c31b29c7ea68b83a7c778619ea38f57f0ea26b6227dddbfcd0198ad`.
- `target/journal-authoring-final-green.log`: `8a344071cf5a533852b396311c8384d657063488ef45b80724a20600858cd4bf`.

PP4001, PP8001 and PP1101 now execute their actual artifact, cleanup and lock
owners. Positive controls, exact public errors and unchanged outside sentinels
pass; replacing owner dispatch with synthetic errors fails the retained
regressions. The focused four-test replay is
`target/diagnostic-filesystem/authoritative-final.log`, SHA-256
`25fc8b546bf6bb69b76888e2a1bc3ad33f6e7a5907b53616b1551d69df9f4aaa`.
The remaining 80 synthetic probes are not counted as owner-boundary acceptance.

### Non-root oracle execution

The OCI corruption fixture now makes only its disposable blob writable;
OpenID makes only copied scratch directories writable. Distribution provisions
six confined report directories before running every oracle as UID 1000 with
a read-only root filesystem. Actual permission failures and omitted-operation
mutants reproduce the defects; immutable oracle inputs remain unchanged.

All eight affected conformance scenarios pass in
`target/openid-overlay/shared-eight.log`, SHA-256
`5ec61e1ea5ed69c900ad9aaba5010c71c6d7ba6f6101ba8bcb02851862f2e2ea`.
The official Distribution corpus covers all 79 cases across six configurations;
the selected OpenID conditions cover 11 positive and 18 negative cases, not
complete provider certification. The digest-bound development oracle image
`127.0.0.1:5000/prismpm-openid-oracle-test@sha256:db5ad1aaac7b1d847a5ec6fc2ed33688fc46dc31b449f7f5f2e7b83e70e7228d`
is component evidence, not an accepted SDK release.

The final Journal checkpoint independently reproduces all 288 golden files
with verifier SHA-256
`0797b8a14fa8b46922aa0155c86889bd8896d8eb827631e0fe403f0968afb327`.
Only verifier provenance and its dependent identities changed from the preceding
review; domain IR and execution bytes did not. The replay log
`target/journal-sealed-golden-check.log` has SHA-256
`53c3f5ab595d448b2c543c34d55cdff6b7a9a4c32d8524dc7d2429cfdd15a6a6`.
Final formatting, all-target/all-feature Clippy and source/model/spec audits
pass. Complete clean-tree SDK verification remains required.

### Boundary regression checkpoint — 18 September 2026

The workspace attempt passed 156 of 157 conformance cases; AU-01 failed on
stale oracle-wrapper bindings. Normal authority resolution refreshed those
bindings, and the independent locked AU-01 replay passed. This is not a
successful complete V&V run.

The real archive validator now reports PP3004 for each of the 32 corrupted
footer bytes; all five archive tests and full Clippy pass. Identity validation
contains hostile thrown values and strips injected payloads. Its actual Node
and Chromium regression first failed, then all 48 identity/storage/peer tests
passed without skips, including unavailable-provider recovery.

The independent golden reader matches all 288 files after regeneration with
verifier `9913b90f9e24e93814a98cedc98e2c414be09e482f48e20992e73bdc6120ea1a`.
The 12 changed JSON values bind archive-source/verifier provenance and dependent
identities only; domain IR and generated execution bytes remain unchanged.

Evidence SHA-256:

- `target/authority-wrapper-lock-regression.log`: `325f45872bf5f0554c766585ee6e3391bf97088a2cc67552c6c39b0df4b14880`.
- `target/diagnostic-footer-draft/archive-green.log`: `6b238e75ac6ac3d9a485cdf9cd157d1f4a523d1b71d938757c911127f0728467`.
- `target/identity-error-draft/host-regression.log`: `c2bec76c2a9c7aa9849e3f46e521353f84407dfc56afe7cf25455819be6e53dc`.
- `target/footer-sealed-golden-check.log`: `12d50e274c5123275aa9b94441e274cae35d837bdd1f996d0cd1d30549821d8e`.

## Shared Action browser export (TM-03)

The real shell adapter first rejected `export-browser`; its 11 owning tests now
pass, including exact argument forwarding, isolated host execution and rejection
of two planted argument/network defects. The recording Docker boundary is an
adapter test double, not evidence of release validity or installed SDK behavior.
Immutable references, verified release closure and atomic confined output remain
owned by the actual SDK CLI/OCI/controller tests.

TM-03 now requires the complete Node pass set with no skipped, cancelled or TODO
tests and a bounded deadline. Actual empty, incomplete, skipped, TODO, missing-file
and timed-out suites fail this gate. All seven conformance-library tests, all six
template conformance cases, authored-package formatting and Clippy pass in the
development container. This is not installed-SDK, producer or Pages acceptance.

- `target/action-export-browser-red.log`: `29a4c6ec3218104c92ffc2a8bd1a0ba6081b112d85f59f84834852387d8b3395`.
- `target/action-export-browser-final.log`: `5abdeac80f3a3f06c2fb4004466fd1befd529b91c5d8449d32f965c1c68167f6`.
- `target/action-export-browser-owning-final.log`: `8cd6d5ee6600827d8dfe31e7f7d8cb428c99df595b8d19517bdb625e3d0189d8`.

## Application build identity

The clean `d0ec75d` V&V run passed gates 1–7, then gate 8 rejected
`Calculator.holo` with PP4001. A fresh build reproduced the same prior identity
with ten different artifacts: embedded stdlib bytes were not fully bound.
Application build inputs `/2` now bind the complete generated artifact closure;
native inputs retain `/1`. Source-free verification independently checks that
closure and explicitly rejects legacy application `/1` evidence with PP6101.

The owning regression failed before the fix. All six release tests now pass,
including repeated actual builds, published-artifact tampering, closed input
schemas and resealed semantic mutants. Calculator builds successfully beside
the unchanged old cache. Scoped all-target/all-feature Clippy and formatting
pass. These are targeted checks, not complete SDK or Foundry acceptance.

- `target/build-identity-regression-red.log`: `5ce2b5bfe9943f17945251b2e6da44217e130a8b087e61bb74d2760103265157`.
- `target/build-identity-release-tests-green.log`: `d2696dc829d3c7bb08ec4e55a07a1cbb4733f3b178bc1f75d252702f4051142b`.
- `target/build-identity-retained-calculator-green.log`: `16bb493ce3d268c156e09a5aadcacd8db8bf2368f993a206c203c126ba23539b`.
- `target/build-identity-clippy.log`: `d3caf52295c1fad315ea4daa0b86b36ff6a673ce642d6278b0089867f4546934`.

## Release criterion

Only a clean, annotated `v0.3.0` tag whose exact commit has produced
`target/vv-evidence.json` with all gates 1 through 15 may pass
`just release-check`. Release artifacts and every platform-specific SDK,
runtime, adapter, and oracle image are built twice and must be identical;
their checksums, SPDX SBOMs, provenance attestations, and signatures are
produced only for those accepted bytes.
