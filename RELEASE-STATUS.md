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

## Remaining release acceptance

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
