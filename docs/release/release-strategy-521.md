# v0.4 construction and acceptance graph (#521)

This implementation starts at `c30276799786c1c9093d4740f41d6376180b8567`.
Rehearsal 41872713783 failed before seven source owners reached acceptance:
pinned Debian snapshot acquisition returned HTTP 503. It supplied no evidence
for the unexecuted ho-oc focus tests or downstream owners. The independently
retained conformance source record remains evidence for that original source
part only; no original rehearsal evidence qualifies this changed source.

## Construction and acceptance

After cheap policy checks, one native producer per architecture constructs the
five SDK-family images and CLI archive with one authenticated input closure,
private recipe and explicit Buildx builder. The standalone runtime remains its
own Dockerfile. The exact target/exporter plan is closed before acquisition;
custody is checked before and after each command, including command failure.
The native acceptance owner consumes that archive instead of rebuilding it.

The original signing, provenance, two platform SBOMs, pinned trust root and
image smoke remain. A separate SDK supply job can finish before the other five
image transports. It does not grant source or installed acceptance. The source
devcontainer verifies the SDK's release signature (exact workflow, ref, event,
repository and revision), both platform manifest/config chains and the native
image in its actual Docker daemon before exporting `PRISMPM_TEST_SDK_IMAGE`.
The original preflights and `just vv-part` command still run. Source receipts
record this oracle-supply digest and retain `original-source-part-only` scope.
They do not become installed-image evidence.

Eight source owners, sixteen installed owners, two product CLI owners, both
browser/library owners, two native owners and all twelve product/platform
reproducibility pairs retain their commands and negatives. Product CLI no longer
waits for the installed aggregate; browser/library no longer wait for OCI
reproducibility. The logical `gate`, `images`, `native`, `reproducibility` and
`installed-sdk` gates still fan in before OCI/native publication. Cargo and final
publication retain their original prerequisites and rehearsal suppression.
No rehearsal-to-tag acceptance transfer is introduced.

Source, installed and native artifacts have context- and attempt-specific names.
Original current-run artifact and all-attempt job API responses select exact
immutable IDs. Selection binds source, image, architecture, owner and run; an
earlier attempt is allowed only when the latest observed owning attempt succeeded.
Missing, duplicate, future, foreign and failed selections fail closed. Complete
original byte validation still follows download. Existing immutable artifacts
are never overwritten.

The old candidate construction validator remains closed over its reviewed,
completed dispatch workflow. Its completed-run admission cannot authorize an
early job of an ongoing release. The new release handoff uses the existing exact
release signature policy and image-chain validators instead. Existing exporter
seed admission and invocation-local CA mechanisms are unchanged. #512's build-only
SDK target is outside this change.

## Reproducibility and physical storage

SPEC's SDK bootstrap requirements require two clean absolute roots. Its release
OCI contract also requires freshly reading both complete local layer closures,
with descriptor sizes/hashes, regular unaliased files and a final custody sweep.
There is no explicit requirement for a third independent OCI construction. In
principle, a demonstrably cold A producer could supply an initially unqualified
candidate and an independent B producer could reconstruct it. This patch keeps
production construction separate because its shared stages have not been
qualified as a cold A root. It does not silently count production as a rebuild.

Twelve small reproducibility shards remain. Each uses two fresh builders with no
production or cross-root cache import. A's builder is removed, its owned state
volume must be absent, and allocated storage is sampled before B starts. A's
original directory remains while B builds. Both complete directory layouts remain
through raw A/B index comparison and the unchanged final custody verifier. Each
export is a fresh singleton `type=oci,tar=false` directory. No evidence bytes are
rewritten and no layout is deleted before final verification.

The relevant physical requirement during B is retained A layout + B compressed
content + B unpacked snapshots/build temporaries + input closure/materialization
+ B exporter temporaries/layout + daemon baseline + reserve. Final comparison
requires both complete layouts + baseline + reserve. Descriptor sums bound
declared layer reads, not BuildKit expansion or physical storage. Boundary
snapshots record allocated daemon, root and layout bytes; they do not establish
an instantaneous peak. No full SDK storage-fit or speedup claim is made here.

Independent A/B producer jobs with authenticated original-output staging would
avoid keeping A on B's producer, but the comparator must still hold both complete
layouts and staging. That also needs a new completed-owner/source/platform/root/
attempt transport contract. The candidate handoff's 64 GiB archive limit is not
interchangeable with the verifier's 128 GiB declared-layout limit. This larger
graph is reserved for evidence that sequential small shards cannot fit.

## Bounded execution proposal

The graph's weighted maximum antichain is fifteen runners in one run, tested
against the declared matrix caps. This is a scheduling upper bound, not measured
organization capacity. All existing six-hour owner limits remain. Reproducibility
parallelism is two; installed parallelism is four. Production construction uses
two native runners. No full hosted qualification was launched for this patch.

Before merging or launching another full rehearsal:

1. Run focused source/transport tests at the reviewed exact head. Retain original
   results, including failures. Resolve the local Linux process-owner test
   environment limitation on a runner exposing `/proc/.../children`; do not
   weaken process retirement checks.
2. Authorize a focused native experiment for one SDK producer and one SDK
   reproducibility pair per architecture, with at most two concurrent runners.
   Use the exact workflow pins and no publication/discovery/Cargo operations.
   Retain original build logs, input manifests, builder/cache reports, command
   timings and allocated storage at every phase; sample available bytes and
   memory throughout builds/exports, not merely at phase boundaries.
3. Enforce a 12 GiB free-space reserve on every participating filesystem and
   evaluate actual content, snapshots, materialized inputs and exporter peaks.
   Require at least thirty minutes below the existing six-hour deadline; stop
   the experiment if either bound is lost. Retire only owned builders/state and
   re-measure available space before B. These are admission criteria for the
   experiment, not claims that existing measurements already meet them.
4. If a pair cannot fit with margin, keep small shards and revise staging or
   producer placement with independent review. Do not batch all products, raise
   deadlines, replace byte verification with manifest equality, or reuse an
   accepted receipt to make the measurement green.
5. Only after measured native feasibility and release-owner review, admit one
   complete exact-source rehearsal with every original owner and final gate.

## Observed focused validation

The actual exporter probe used checksum-verified Buildx 0.28.0
(`696bc104bac3bb708eff1af3f8bbc09fda0fd88f5757c1f9b404a35117889224`)
and the workflow's exact BuildKit image
`moby/buildkit@sha256:de10faf919fc71ba4eb1dd7bd6449566d012b0c9436b1c61bfee21d621b009aa`
(reporting v0.26.2). For each architecture two sequential cold directory builders
produced identical raw indices and all blobs. Tar/directory manifest, config and
layer bytes also matched. The directory wrapper index adds only
`org.opencontainers.image.ref.name=latest` (408 versus 363 bytes); original indices
were preserved. The unchanged verifier accepted both forms and rejected twelve
actual-directory custody mutations. All five probe builders/state volumes were
retired. This ten-byte scratch input demonstrates exporter behavior only; its
1,779-byte layouts and 8,192-byte tar wrappers do not predict full SDK footprint.

Focused tests use checksum-verified Node 22.23.2 and the repository's locked oracle
dependencies. Test coverage includes one acquisition per build set, exact output
plans, input/output custody and failure refusal, both signed platform chains,
actual-daemon image admission, original source command propagation, complete
inventories, immutable retry selection, transient HTTP recovery/exhaustion,
permanent/integrity failures, raw layer/final-custody mutations, fail-closed
aggregates and rehearsal publication suppression. Local full process-owner tests
remain refused by the environment's missing Linux task-children interface, even
outside the filesystem sandbox; that is not an accepted gate or an omitted owner.
The installed-owner shell fixture also refuses host Python 3.13.5; its original
owner admits only the pinned 3.11.2/3.12.3 runtimes. Product shell tests pass.
