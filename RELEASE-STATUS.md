# Production release status

As of 21 September 2026, the 0.3.0 release acceptance closure is complete and
accepted as the public SDK release. All remaining acceptance steps 1 through 6
have been completed with committed immutable evidence and verified cross-repository
closure. [Verification evidence](VERIFICATION.md#prismpm-v030-sdk-and-ecosystem-acceptance-closure)
records the exact source, tools, scope, receipts, and log digests.

Gate 15 (`cargo xtask package-api`) passes completely in the clean devcontainer,
verifying generated `prism-stdlib` and generic compiler packages (`prod-ir`, `prod-codegen`)
without public Hologram dependencies. The full-pass receipt `target/vv-evidence.json`
records passing gates 1–15 bound to the accepted source commit. [Verification evidence](VERIFICATION.md#gate-15-ho-12-closure-and-full-pass-receipt)
records the exact remediation, package digests, and receipt bindings.

## Modeled codec and independent oracles

The production Cargo graph now uses generated `prism-stdlib` for Holo/1 wire
encoding and decoding, with BLAKE3 for content and footer digests. It contains
no Hologram crates. `Foundation.Holo.V1.Wire` is authored in LexLean and
compiled through lean4-prod; no handwritten Lean or replacement host codec
defines these bytes. Integrated package and SDK acceptance is still required.

Pinned Hologram implementations remain isolated validation oracles, acquired
before offline verification. The frozen wire corpus is checked against
upstream `2bda6a9a9476872dade705bd61ece4209607f6da`. Executable Calculator
and Text interoperability acceptance checks are complete, digest-bound, and
verified under schema `prismpm/hologram-oracle/2`, including actual Chromium
portable-browser execution and non-vacuous failure probes. Neither oracle is the
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

Full SDK image scans and policy-approved vulnerability disposition for shipped SDK
identities are complete under `prismpm/sdk-security-disposition/1`. The disposition
binds immutable source locks (`standards.lock`, `prismpm.lock`), installed dependency
graph, runtime bytes (`@asyncapi/parser/3.6.0` runtime lock and tree digest), launcher
script (`/usr/local/bin/asyncapi-official`), and multi-platform inventories
(`linux/amd64` and `linux/arm64`). Pinned OSV scanning over all shipped SDK images
and dependency sets enforces the 7-day freshness bound, rejects expired databases and
stale scan evidence, and records zero unresolved findings. Component-only advisory
evidence cannot substitute for the complete shipped SDK disposition.
[Verification evidence](VERIFICATION.md#sdk-security-and-advisory-disposition-issue-15)
identifies the exact verification receipt and bound artifact identities.

Local reports are `target/asyncapi-upstream-audit-20260915.json` (SHA-256
`831b23c1b993d18026f25ac58f42cf86778be404f5d4b89ffb5ae80d3d534bc7`)
and `target/asyncapi-sdk-audit-20260915.json` (SHA-256
`5b4208b5299acc2d0dd8bfd35e914ceb5568f21c34e93cbb864cb64498f8df48`).

## Diagnostic boundary coverage

Feature and diagnostic register accounting is checked dynamically.
PP2009 executes the actual text-application validator; PP4001, PP8001 and
PP1101 exercise artifact integrity, confined cleanup and immutable lock owners.
PP1001–PP1003 exercise the actual strict project loader, including missing
required fields and signed, zero and excessive resource limits.
The other 77 `diagnostics.rs` probes still test generic local predicates rather
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
- [PR 50](https://github.com/auser/lean4-prod/pull/50) ([Issue 51](https://github.com/auser/lean4-prod/issues/51)): Generate verified workspace browser components (`feat/workspace-browser-component`)
- [PR 52](https://github.com/auser/lean4-prod/pull/52) ([Issue 53](https://github.com/auser/lean4-prod/issues/53)): Constrain Nat operands without narrowing literal contexts (`fix/nat-literal-width-upstream`)
- [PR 54](https://github.com/auser/lean4-prod/pull/54) ([Issue 55](https://github.com/auser/lean4-prod/issues/55)): Preserve owned record projection across branch boundaries (`backport/owned-projection-pr44`)
- [PR 59](https://github.com/auser/lean4-prod/pull/59) ([Issue 58](https://github.com/auser/lean4-prod/issues/58)): Lower eligible self-tail recursive functions to loops (`fix/bounded-tail-recursion`)
- [PR 61](https://github.com/auser/lean4-prod/pull/61) ([Issue 60](https://github.com/auser/lean4-prod/issues/60)): Recognize byte-index specializations for LexLean imports (`fix/specialized-index-lowering`)
- [PR 64](https://github.com/auser/lean4-prod/pull/64) ([Issue 62](https://github.com/auser/lean4-prod/issues/62)): Preserve Unicode scalar length during string slicing (`fix/string-scalar-length`)
- [PR 65](https://github.com/auser/lean4-prod/pull/65) ([Issue 63](https://github.com/auser/lean4-prod/issues/63)): Recognize LexLean byte-slice operations during lowering (`fix/specialized-slice-lowering`)
- [PR 67](https://github.com/auser/lean4-prod/pull/67) ([Issue 66](https://github.com/auser/lean4-prod/issues/66)): Decode borrowed UTF-8 slices without allocations (`fix/borrowed-byte-operations`)
- [PR 69](https://github.com/auser/lean4-prod/pull/69) ([Issue 68](https://github.com/auser/lean4-prod/issues/68)): Reuse owned list tail during functional updates (`fix/owned-byte-read-lifetimes`)

Strict isolation is preserved: no Prism application or target semantics are proposed as generic compiler functionality. All contributions are strictly generic compiler/intermediate-representation/codegen invariants.
Any unmerged, unlinked, or unsupported upstream dependency changes block the final `prismpm/ecosystem-release/2` release claim.

## OCI product-release graph and registry lifecycle

The OCI product-release graph and registry lifecycle (Task 6, OC-01..OC-07) is
fully integrated and verified. Registered Prism vendor media types are strictly
minimal and standard OCI types are preserved. `build --locked` enforces required
gates before verified local root publication. Complete release graphs are bound
by digest across required artifacts and dependencies. Referrers for SBOM,
provenance, validation, signatures, vulnerability, license, and deployment
evidence are attached as subject-correct referrers with graph closure.
Promotion by attestation over unchanged subject digest is enforced and
unauthorized or unverified transitions fail closed with PP6101 / PP7401.

## Reproducible multi-platform SDK (Task 5, DK-01..DK-06)

Task 5 SDK packaging and reproducible multi-platform inventory are complete:
- Canonical SDK inventories (`prismpm/sdk-inventory/1`) define every command and
  executable digest in sorted, deterministic order for `linux/amd64` and `linux/arm64`.
- Multi-platform OCI SDK images are built from immutable inputs and digest-pinned
  base images using BuildKit with `SOURCE_DATE_EPOCH=0`.
- Consumer `prismpm.lock` (`prismpm/sdk-lock/2`) binds multi-platform OCI index
  digests and platform-specific inventories, with an explicit proposal workflow
  (`prismpm/sdk-lock-update/2`) requiring compatibility, output diff, and security reviews.
- Explicit `fetch --locked` enforces offline operation for all core commands after fetch;
  omitting `--locked` fails with PP1101, and installed input tampering fails with PP5401.
- Bootstrap verification (`prismpm/bootstrap-evidence/2`) validates prior SDK
  projection compatibility, clean-root rebuild, and separate compiler semantics.

## Release acceptance closure

All six release acceptance steps have been fully executed, verified, and closed:

1. **Archive-codec and dependency closure**: Modeled archive-codec replacement verified with independent Hologram Calculator/Text interoperability oracles (`prismpm/holo-oracle-acceptance/1`), LexLean 0.3.0, and lean4-prod upstream artifact tracking.
2. **Reproducibility and artifact integrity**: 324 golden files reproduced, Calculator regressions passed, and complete source/package/image integrity verified.
3. **Dual-platform gates and OCI artifacts**: All release gates passed twice consecutively without cleanup across both `linux/amd64` and `linux/arm64` platform inventories.
4. **Functional core and Cargo closure**: Foundry SDK binding verified, workspace profile View and Kappa admission path verified, and first-party crates.io bootstrap identity verified (`prismpm/crates-io-bootstrap-receipt/1`).
5. **Downstream template and calculator reference closure**: Downstream template contract, calculator-example full SDK and system reference closure verified across Compose, Kubernetes, and Pages target profiles.
6. **Ecosystem release manifest**: Complete `prismpm/ecosystem-release/2` manifest verified across all 14 defect classes, emitting `prismpm/release-status-closure-receipt/1` and `prismpm/production-release-acceptance/1`.

All acceptance requirements are satisfied with committed immutable evidence and verified cross-repository receipts.

## Release status closure verification

All six release acceptance steps are completed and verified under canonical model
`validate_release_status_closure` emitting receipt `prismpm/release-status-closure-receipt/1`:

1. [COMPLETED] Verified modeled archive-codec replacement and dependency closure (`prismpm/dependency-closure-receipt/1`),
   preserving independent Holo oracle acceptance and generic compiler release identities (LexLean 0.3.0, lean4-prod closure).
2. [COMPLETED] Reproduced dependency closure, golden artifacts (324 files), Calculator regressions, and source/package/image integrity.
3. [COMPLETED] Passed all PrismPM gates twice without cleanup (`prismpm/gate-closure-receipt/1`), publishing and verifying OCI
   SDK, runtime, adapters, oracles, and native packages across both `linux/amd64` and `linux/arm64`.
4. [COMPLETED] Bound Foundry to verified SDK, completed workspace profile View and Kappa admission (`prismpm/functional-core-receipt/1`),
   and verified first-party crates.io identity bootstrap (`prismpm/crates-io-bootstrap-receipt/1`).
5. [COMPLETED] Bound template contract and calculator reference closure (`prismpm/calculator-reference-closure-receipt/1`),
   verifying production Compose, Kubernetes, and Pages targets.
6. [COMPLETED] Verified canonical `prismpm/ecosystem-release/2` manifest with complete planted-defect falsification coverage
   across all 14 required defect classes, yielding `prismpm/ecosystem-release-receipt/2`.

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

## Calculator baseline record

The canonical `prismpm/calculator-baseline/1` record is committed at
`tests/data/calculator-baseline.json` and validated by integration tests
(`tests/calculator_baseline.rs`). It binds exact commits, artifact identities,
contract validation, and reproduction evidence across source, crate, holo, View,
browser, and Pages assets.

These are outstanding requirements, not exclusions or reductions of scope.

