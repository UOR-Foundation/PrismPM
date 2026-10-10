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
