# Installed binary-package qualification

`bash scripts/binary-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT [NEW_EVIDENCE_DIR]` qualifies
DK-29 through the installed CLI in the exact current native SDK. The release
reproducibility matrix requires it on both native amd64 and arm64. Existing
browser, native-library, full SDK V&V and product gates remain mandatory.

The wrapper binds the complete selected source/compiler/schema/fixture/helper
closure to a clean exact Git revision and the shipped image. It checks native
architecture, immutable image identity, SDK entrypoint bytes, current-source
inventory and actual installed CLI bytes. Execution uses the existing SDK
session initializer, uid 1000, a read-only image, no network, no capabilities,
no host source or dependency mounts, and a fresh private temporary home.

Fresh fixture roots must reproduce all generated package bytes. Current source
is compiled and proved, then regenerated artifacts are replayed and all modeled
raw-byte vectors execute in std, no_std, Core-Wasm, CLI stdio, CLI file and mixed
file/stdio modes. Empty bytes, NUL, every byte value and malformed UTF-8 are
included. A mandatory generated CLI unit test also exercises real
Vec capacity overflow and its AdapterAllocation mapping, with exactly one pass
and no ignored or filtered tests. The receipt also requires Linux I/O coverage and a real output-write failure
for this fixture, which has nonempty responses; omitted or unexercised coverage
is rejected. Physical OOM is unmeasured. ComputeFailure
classifies generated Result errors; panics, aborts and signals remain failed
processes without a typed-diagnostic guarantee. The checked model, full program
descriptor, build inputs, every file,
complete declaration audit, process records and acceptance modes are bound into
closed canonical receipts. The outer verifier copies the actual generated build,
LexLean attestation and profile acceptance out of the container, then rechecks
the complete byte-bound closure against its result. The release and PR workflows
retain those artifacts with the exact source and inventory bindings. A previous
receipt is never sufficient for a fresh run.

The gate additionally requires read-only checks, invalid root/type/arity and
nominal impostor rejection, a false modeled acceptance, product-release refusal,
and missing/changed proof, acceptance and generated-package byte rejection.
Restoring source and bytes must restore acceptance. Failed and skipped owning
tests are rejected.

`node scripts/binary-sdk-check.mjs tests` (or `just binary-sdk-check-tests`) runs
the complete parser and recording-Docker boundary suite. Its synthetic records
are test-only. They do not establish installed-image execution. The separate
PR qualification workflow executes real SDK images built from the exact PR
head on each native architecture, with only runner-local OCI transport and no
publication credentials. It retains profile evidence while the ordinary source
CI gates continue independently.

The scope is binary-package-only. Application, browser, Holo, production release
and deployment remain explicitly unclaimed. Single-platform PR qualification
also does not establish full SDK release, multi-platform product construction,
public package availability or compression M0 acceptance.

The PR workflow first runs an independent strict source-format prerequisite.
Its pinned existing SDK formats only an isolated temporary source copy and
records the actual SDK inventory. It always retains a review-only patch; any
nonempty patch fails the prerequisite and blocks both native image jobs. The
checkout and remote remain untouched. A maintainer must review and import the
patch before a new exact-source run can proceed. Formatting does not establish
compilation or acceptance and cannot substitute for the reconciled compiler.

Formatter startup waits for a bounded readiness marker written only after the
normal SDK initializer hands off. A failed or stalled initializer fails the
job before a second command can race its immutable-cache seed. The actual SDK
inventory is retained before formatting starts, including on formatter failure.

The formatter uses the repository-owned `cargo fmt` / `cargo fmt --check`
commands. Only workspace `crates/*` and `xtask` Rust changes can enter its review
patch; excluded generated stdlib and vendored dependencies remain writer-owned.
Temporary source and binary proof bytes are streamed from the live container
namespace before its tmpfs disappears. The binary result is accepted only after
the actual container exits successfully and the exported proof is revalidated.

## Deliberate generated-source preparation

`.github/workflows/binary-generated-review.yml` is a review-only preparation
route. Point the dedicated `work/binary-generated-review` helper branch at the
selected exact prerequisite commit only after the reconciled compiler import is
reviewed. Ordinary feature-branch pushes do not launch this expensive writer
run. A future `workflow_dispatch` route requires the input commit to equal the
dispatched ref SHA; GitHub requires that workflow file on the default branch
before manual dispatch is available.

The workflow uses the existing pinned development container, runs `codegen`,
`fixtures-write`, `golden-write`, and `stdlib-package-write`, packages the stdlib
archive, and runs the existing golden, stdlib, fixture, and package checks without
altering their environment rules. The artifact records the selected source/tree,
dependency register, root lock, actual generation log, and patch digests. Failed
preparation uploads diagnostics only; it cannot yield an accepted patch.
Review and import successful generated bytes into a new source commit, then run
all normal source, native golden, installed-SDK and release gates. This workflow
never commits, pushes, publishes an image, or creates a release.

The host qualification runner additionally requires Python 3 (standard library
only) for confined source acquisition. Docker copy output is treated as an
untrusted archive: the extractor parses the authoritative source capture once,
requires complete non-overlapping root coverage, and validates each entire
archive before writing any of its members. It checks the exact existing capture
for the complete member set, path/type, file size/digest,
and registered alias targets. Duplicate, omitted, unexpected, traversal, special,
and symlink-ancestor members fail closed. Image ownership and modes are never
applied to private comparison copies; the SDK remains immutable. Whole-source
verification remains a separate mandatory step before installed execution.
