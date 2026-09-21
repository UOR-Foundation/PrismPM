# Production release status

As of 18 September 2026, the 0.3.0 source is under verification and is not an
accepted public SDK release. Clean commits and passing component checks do not
replace the cross-repository acceptance contract in `current/tasks.md` of the
development workspace.

Complete source-devcontainer `just vv` at `d1b8506` passed gates 1–14,
including actual portable browser execution and cross-root reproducibility,
then failed gate 15 on the former public Hologram dependency. No full-pass receipt was
produced. [Verification evidence](VERIFICATION.md#portable-view-execution-and-tool-integrity-ho-12)
records the exact source, tools, scope, and log digest; the development SDK
binding is not a newly accepted production release.

## Modeled codec and independent oracles

The production Cargo graph now uses generated `prism-stdlib` for Holo/1 wire
encoding and decoding, with BLAKE3 for content and footer digests. It contains
no Hologram crates. `Foundation.Holo.V1.Wire` is authored in LexLean and
compiled through lean4-prod; no handwritten Lean or replacement host codec
defines these bytes. Integrated package and SDK acceptance is still required.

Pinned Hologram implementations remain isolated validation oracles, acquired
before offline verification. The frozen wire corpus is checked against
upstream `2bda6a9a9476872dade705bd61ece4209607f6da`; executable Calculator
and Text interoperability checks remain required. Neither oracle is the
application authority or a deployed Foundry service. Publishing Hologram is
not a PrismPM dependency-resolution step.

The first-party names `lexlean`, `prod-ir`, `prod-codegen`, `prism-stdlib` and
`prismpm` are not yet registered on crates.io. Their initial uploads require
registry-owner credentials before trusted publishing can be configured.
The current OIDC-only workflows cannot perform that bootstrap unaided;
an owner-controlled first upload and exact package checks remain required.
The owner requires Foundry publication and live verification before those
uploads. Public Cargo publication is therefore not a prerequisite for the
Foundry stage: use a separately verified, immutable OCI SDK with its complete
offline dependency closure. This changes publication order, not verification
requirements or the definition of a complete ecosystem release.

## SDK oracle advisory disposition

On 15 September, devcontainer `npm audit --package-lock-only --ignore-scripts
--json` reported eight affected package records (seven high, one moderate) in
the pinned AsyncAPI official-example harness lockfile. Its affected packages
are Spectral core/functions, Ajv, brace-expansion, fast-uri, js-yaml, Lodash,
and minimatch; this is not a count of distinct advisories. The separate
`sdk/oracles` lockfile used for submitted documents reported zero findings.
These audits cover lockfile graphs, not installed trees or the complete SDK.

The original upstream source and historical lock remain unchanged. The SDK now
installs a separately Prism-owned compatible runtime lock with parser 3.6.0;
this is not an upstream-reviewed update or historical-lock replay. Its installed
graph audit reported zero findings. The owning oracle passed all 24 documents,
89 embedded examples, negative probes and five runtime-integrity test groups
in a read-only, network-disabled development image. Source, installed bytes,
launcher and inventory are bound; skipped tests fail acceptance.

This is component evidence, not a vulnerability-clean shipped SDK. Complete
image scans and policy-approved disposition remain required; no waiver is
accepted. [Verification evidence](VERIFICATION.md#owned-asyncapi-runtime)
identifies the exact component results.

Local reports are `target/asyncapi-upstream-audit-20260915.json` (SHA-256
`831b23c1b993d18026f25ac58f42cf86778be404f5d4b89ffb5ae80d3d534bc7`)
and `target/asyncapi-sdk-audit-20260915.json` (SHA-256
`5b4208b5299acc2d0dd8bfd35e914ceb5568f21c34e93cbb864cb64498f8df48`).

## Diagnostic boundary coverage

Feature and diagnostic register accounting is checked dynamically.
PP2009 executes the actual text-application validator; PP4001, PP8001 and
PP1101 exercise artifact integrity, confined cleanup and immutable lock owners.
The other 80 `diagnostics.rs` probes still test generic local predicates rather
than their owning implementation boundaries. Passing those probes or counting their
IDs is not evidence that all public error paths work. Complete real positive
and malformed-input subsystem coverage, including emitted-code and execution
evidence checks, remains required for production SDK acceptance; it is not
excluded by the current text-profile work.

## Upstream generic compiler dependency tracking (lean4-prod)

PrismPM 0.3.0 depends on generic Lean 4 compiler improvements authored in `afflom/lean4-prod`
and tracked upstream in `auser/lean4-prod`. The authoritative dependency specification is
committed in `model/dependencies.toml` at revision `ac84a4de575e2e531ddb6453c86b84a6794fe48b`
for Lean 4.32.1.

Vendored release artifacts and tree manifests are locked to immutable SHA-256 digests:
- `vendor/lean4-prod/lean.tar`: `74eb4600836c873f9ffdff30f8062c5dc1314aba572c36afd1c851434affadc5`
- `vendor/lean4-prod/rust/MANIFEST.sha256`: `3b976d0bf2c0509c28b069417bf9bdb8d12f68d9c6ba383910f03b75e7a303dc`
- `vendor/lean4-prod/crates/prod-alloc-counter-0.1.0.crate`: `3072374800280030ab1f03db059676f93d7f3d62df431895e32fe8c9eae8229e`
- `vendor/lean4-prod/crates/prod-codegen-0.1.0.crate`: `5b56d5ed74c05404e21e29daeda69c11eebb3e23cf923757fb5faddc70a05be6`
- `vendor/lean4-prod/crates/prod-ir-0.1.0.crate`: `9c54edb43e1dd317cbca1b2a4109cfd0b0103cbb75e4b41ca3dc7a99b6ddd2c5`

Because `afflom/lean4-prod` has issues disabled, dependency tracking and closure evidence
are maintained in PrismPM release tracking documents and anchored to upstream issue IDs.
All 15 generic compiler contributions follow established fork-PR flows to `auser/lean4-prod`:
- [Issue 70](https://github.com/auser/lean4-prod/issues/70): Release Dependency Closure: Provide immutable release/provenance identities required by PrismPM 0.3.0 acceptance
- [PR 38](https://github.com/auser/lean4-prod/pull/38) ([Issue 37](https://github.com/auser/lean4-prod/issues/37)): Adapt fallible Bytes CoreWasm entries (`fix/core-wasm-fallible-bytes`, commit `ac84a4d`)
- [PR 40](https://github.com/auser/lean4-prod/pull/40) ([Issue 39](https://github.com/auser/lean4-prod/issues/39)): Preserve UTF-8 validity on borrowed slice inputs (`fix/borrowed-utf8-encoding`)
- [PR 42](https://github.com/auser/lean4-prod/pull/42) ([Issue 41](https://github.com/auser/lean4-prod/issues/41)): Preserve collection ownership through scalar matches (`fix/collection-ownership-and-typed-decimal`)
- [PR 44](https://github.com/auser/lean4-prod/pull/44) ([Issue 43](https://github.com/auser/lean4-prod/issues/43)): Preserve ownership across SplitExact UInt32 bounds (`fix/split-exact-uint32-bound`)
- [PR 46](https://github.com/auser/lean4-prod/pull/46) ([Issue 45](https://github.com/auser/lean4-prod/issues/45)): Preserve scalar parameter hygiene in SDK generation (`fix/scalar-sdk-parameter-hygiene`)
- [PR 48](https://github.com/auser/lean4-prod/pull/48) ([Issue 47](https://github.com/auser/lean4-prod/issues/47)): Normalize raw local identifiers during IR lowering (`fix/raw-local-identifier-hygiene`)
- [PR 50](https://github.com/auser/lean4-prod/pull/50) ([Issue 49](https://github.com/auser/lean4-prod/issues/49)): Generate verified workspace browser components (`feat/workspace-browser-component`)
- [PR 52](https://github.com/auser/lean4-prod/pull/52) ([Issue 51](https://github.com/auser/lean4-prod/issues/51)): Constrain Nat operands without narrowing literal contexts (`fix/nat-literal-width-upstream`)
- [PR 54](https://github.com/auser/lean4-prod/pull/54) ([Issue 53](https://github.com/auser/lean4-prod/issues/53)): Preserve owned record projection across branch boundaries (`backport/owned-projection-pr44`)
- [PR 59](https://github.com/auser/lean4-prod/pull/59) ([Issue 58](https://github.com/auser/lean4-prod/issues/58)): Lower eligible self-tail recursive functions to loops (`fix/bounded-tail-recursion`)
- [PR 61](https://github.com/auser/lean4-prod/pull/61) ([Issue 60](https://github.com/auser/lean4-prod/issues/60)): Recognize byte-index specializations for LexLean imports (`fix/specialized-index-lowering`)
- [PR 64](https://github.com/auser/lean4-prod/pull/64) ([Issue 62](https://github.com/auser/lean4-prod/issues/62)): Preserve Unicode scalar length during string slicing (`fix/string-scalar-length`)
- [PR 65](https://github.com/auser/lean4-prod/pull/65) ([Issue 63](https://github.com/auser/lean4-prod/issues/63)): Recognize LexLean byte-slice operations during lowering (`fix/specialized-slice-lowering`)
- [PR 67](https://github.com/auser/lean4-prod/pull/67) ([Issue 66](https://github.com/auser/lean4-prod/issues/66)): Decode borrowed UTF-8 slices without allocations (`fix/borrowed-byte-operations`)
- [PR 69](https://github.com/auser/lean4-prod/pull/69) ([Issue 68](https://github.com/auser/lean4-prod/issues/68)): Reuse owned list tail during functional updates (`fix/owned-byte-read-lifetimes`)

Strict isolation is preserved: no Prism application or target semantics are proposed as generic compiler functionality. All contributions are strictly generic compiler/intermediate-representation/codegen invariants.
Any unmerged, unlinked, or unsupported upstream dependency changes block the final `prismpm/ecosystem-release/2` release claim.

## Remaining release acceptance

The signed-envelope and authenticated browser-journal prerequisite gates pass,
including offline browser crypto/storage checks and complete bounded replay.
They do not yet implement the generated workspace application profile, its View,
Kappa replication/read admission or the Foundry portal. These remain required
for the authorized functional-core release; component tests do not replace
live faculty/participant journey acceptance.

Complete the existing release plan without treating a prototype implementation
as a mandatory application dependency:

1. Verify the modeled archive-codec replacement and complete resulting public
   dependency closure. Preserve independent Holo oracle
   acceptance. Verify the generic compiler release packages and their publishing
   identities, including LexLean 0.3.0 and the lean4-prod fork/upstream changes
   (pinned at revision `ac84a4de575e2e531ddb6453c86b84a6794fe48b` and tracked
   through upstream issue 70 and PRs 38–69).
2. Reproduce the PrismPM dependency closure and its package, golden
   artifacts, Calculator regression, and all source/package/image checks.
3. Pass every PrismPM gate twice without cleanup. Publish and independently
   verify the exact OCI SDK, runtime, adapters, oracles, and native packages.
   Bind acceptance to the shipped digests and both platform inventories;
   development-candidate smoke checks are insufficient.
4. Bind Foundry to that verified SDK, implement its authorized functional core,
   and publish and independently verify its unchanged Pages artifacts and
   complete core journeys. Then publish and verify the first-party Cargo
   closure. Neither phase may claim the other has completed.
5. Bind template and Calculator locks/workflows to those public immutable
   artifacts. Regenerate their source projections and preserve their accepted
   application baseline. Run their complete local and CI acceptance, including
   both production releases, deployments, rollback, recovery, and Pages.
6. Verify the complete `prismpm/ecosystem-release/2` manifest and only then date
   the changelog, create release tags, and claim completion.

These are outstanding requirements, not exclusions or reductions of scope.
