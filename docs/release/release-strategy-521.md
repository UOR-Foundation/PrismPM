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

The separately monitored main run 41900101049 at ecb594bd ended at 23:04:52 UTC
with an Actions-runner `System.IO.IOException` reporting no space left on device.
Its final cleanup/upload was not reached and its job logs were unavailable.
Twenty finalized checkpoints survived; the last shows 9.42 GB available, target
allocations about 31.96 GB, an entry-limited broader scan and unavailable fixed
Docker-path accounting. This establishes exhaustion, not its final growth owner.
The diagnostic therefore measures the actual participating data roots/devices,
live reserve and complete bounded scans; it does not rerun that failed layout.


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
2. Freeze the reviewed release source, including the accepted #529 BinaryProgram
   counterpart and its separate original acceptance owners. The Binary owner
   must consume the existing authenticated SDK supply on both architectures and
   join the final acceptance fan-in; it must not add another construction family.
   The legacy `raw-file-cli/1` profile does not qualify #519's full streaming profile.
3. Review and authorize the exact diagnostic dispatch below. Retain failures and
   complete resource observations. The proposed limits are admission criteria,
   not claims that the SDK fits them. Retire only owned builders/state and prove
   their absence before B. If measurements are unavailable or incomplete, fail.
4. If a pair cannot fit with margin, keep small shards and revise staging or
   producer placement with independent review. Do not batch all products, raise
   deadlines, replace byte verification with manifest equality, or reuse an
   accepted receipt to make the measurement green.
5. Only after measured native feasibility and release-owner review, admit one
   complete exact-source rehearsal with every original owner and final gate.

### Prepared diagnostic triggers (not launched)

The registered `.github/workflows/release.yml` has an exclusive
`resource-diagnostic-source` dispatch mode and a separate exact-source push route.
Dispatch retains its exact `fix/521-release-build-sets` ref, explicit version and
rehearsal/publication flags, with input, checkout, workflow and run source equal.
Push admits only `resource-diagnostic/<full-lowercase-source-SHA>`. The suffix
must equal `GITHUB_SHA`, `GITHUB_WORKFLOW_SHA`, the checked-out commit, original
push `after` and non-null `head_commit.id`. The bounded original event file must
name the canonical repository and ref, `deleted:false`, `forced:false` and a
non-fork repository. Two disjoint transitions are supported: `created:true`
with an all-zero `before`, or `created:false` with `before` exactly the frozen
release ancestor `c30276799786c1c9093d4740f41d6376180b8567`. The latter requires
an offline strict-ancestor proof against the exact checked-out Git graph, with
replacement objects, grafts, shallow history and lazy fetching refused. The
proof has a ten-second budget narrowed by the inherited command deadline.
Null/missing heads, tags, partial/foreign/stale refs, other update ancestors,
forced writes and deletions fail. Proven trigger fields are retained in the
inherited policy, and each command/metadata owner rechecks the original event.
Missing dispatch inputs cannot select publication.

The normal policy excludes the entire diagnostic branch prefix, including a
malformed suffix; all acceptance/publication jobs retain that policy ancestor
and normal success gating. Only the three diagnostic jobs admit the dedicated
push or explicit diagnostic dispatch. Push filters retain the original release
branch/tag and add only `resource-diagnostic/*`, without path filters, create
events or reusable-workflow callers. Pushing the issue branch does not launch
this workflow. Diagnostic jobs have read permissions and bounded artifact
uploads; no signing, registry upload, acceptance or publication occurs.

The connected GitHub catalog has no new `workflow_dispatch` action. A read-only
authenticated `gh api --method GET /user` check exited 1; no credential was read
or extracted. This is a capability gap, not an action denial. Dispatch support
remains available if an authenticated route is established later:

```sh
source_head=$(git rev-parse HEAD)
gh workflow run release.yml --repo UOR-Foundation/PrismPM \
  --ref fix/521-release-build-sets \
  -f resource-diagnostic-source="$source_head" \
  -f version=0.4.0 -f publish-crates=false -f rehearsal=true
```

This command has not been run. The source must still be that remote branch's
exact head when GitHub creates the dispatch. Source changes require a new review;
the exact-source policy refuses a stale input. The #529 owner/code and its final
fan-in remain prerequisites to launch; parent review of raw #528 evidence is
complete, as recorded below.

After the reviewed union is committed and saved, the parent can use its
existing authenticated leased-ref route. It first verifies the exact source,
workflow and frozen strict ancestor, verifies the diagnostic ref is absent, then
creates `refs/heads/resource-diagnostic/<full-source-SHA>` at that frozen ancestor.
It must inspect both the actual ref and Actions runs after creation. If creation
already emitted an exact-source diagnostic run, it must not trigger another.
Otherwise, after verifying zero matching runs, it performs exactly one
`force=false`, expected-head leased fast-forward from frozen c302 to the reviewed
full source SHA. The source suffix, event `after`, workflow and checkout must all
match. A previous connector producer demonstrated that REST creation at an
existing commit did not emit a push workflow, while its subsequent leased
fast-forward emitted one run; that observation motivates this bounded alternative.

No trigger ref was created during source preparation. Successful REST ref
creation alone is not an observed Actions launch, and `GITHUB_TOKEN`-originated
pushes can be suppressed by GitHub. The parent owns the one final launch and
must inspect its resulting exact-source event/run. Do not substitute another SHA,
broaden triggers, advance an unexpected ref, or retry an uncertain write without
checking the ref and runs. A staged tree is review evidence, not the final source
commit to put in the trigger name.

GitHub documents that push workflows can run from an unmerged branch and
defines `after` and `head_commit` in its push payload. This policy conservatively
requires a non-null head rather than inferring one from a default branch.
See [push workflow events](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#push),
[push payload](https://docs.github.com/en/webhooks/webhook-events-and-payloads#push)
and [trigger restrictions](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow).

Two native production jobs run first, one per architecture. Each constructs only
the SDK `runtime` target and CLI archive in one authenticated build set. Two
native pair jobs then independently reconstruct only `runtime` in cold A/B roots.
A's builder retires before B starts; both raw layouts remain until the unchanged
byte verifier and final custody sweep finish. Provider originals select only the
current run/attempt's successful native production records. Their metadata names
an unpublished diagnostic subject and grants no installed or release acceptance.
This experiment does not measure the other four SDK-family exports, the separate
runtime Dockerfile, registry exporter storage or complete release fit.

The policy sets one common work deadline at 5h30 and a supervised-command/cleanup
deadline at six hours; downstream jobs inherit both. GitHub action setup/uploads
have separate job timeouts and are not covered by the private command owner.
At most two native runners operate at once. Every runner must initially have at least 54 GiB on the backing
filesystem: a fully allocated 40 GiB ext4 loop file, 12 GiB backing reserve and
2 GiB allowance for observed growth in Docker/output allocations outside the loop
after a source-bound complete post-pull baseline. Gross baseline allocations are
retained; deletion on one filesystem cannot offset growth on another. ext4
formatting and mounting disable discard, and the backing inode and allocated
blocks are repeatedly checked. A deliberate bounded ENOSPC probe must demonstrate
the store's limit before any SDK work. The sampled 12 GiB reserve applies to every
participating outer filesystem, including a separately mounted Docker storage
device; the inner filesystem instead has a sampled 1 GiB stop threshold.
ENOSPC and temporary sampling gaps cannot qualify a successful measurement.

All source roots, private input materializations, Buildx configuration, builder
state/run directories, temporary files and OCI exports live inside that hard
store. The exact pinned BuildKit container has a read-only root, 256 MiB `/tmp`
tmpfs, two CPUs, at most 12 GiB memory while leaving 2 GiB host memory, no extra
swap, 4,096 PIDs and bounded Docker logs. Its exact CID, label, image, configuration,
live cgroup limits and actual bind inodes are checked before/after bootstrap and
throughout observation. A disposable tiny export checks that arrangement before
the SDK builds. Inherited Docker credentials/proxy configuration is preserved.

One-second observations check actual Docker storage, participating filesystem
capacities, backing allocation, memory, cgroup OOM/PID events and owner custody.
Complete allocated-block scans have 30-second and two-million-entry bounds;
unavailable or over-budget scans fail. Allocations are reported by filesystem,
with inode aliases deduplicated and merged overlay views explicitly excluded.
The backing file and its inner filesystem are separate accounting layers, not
summed as a physical total. These are interval observations and sampled minima,
not instantaneous peak claims.

The original private Linux command owner supervises commands and the privileged
observer, including descendants. A stale/dead observer, lost reserve, sparse or
replaced backing file, limit event, deadline, custody failure or uncertain process
retirement stops the diagnostic. Cleanup requires proven process exhaustion,
exact stopped container identity, emptiness or absence of its originally observed
BuildKit cgroup before container/state deletion, ordinary unmount, exact loop
detachment and the original backing inode. Uncertain cleanup preserves the
remaining owned storage and records failure; there is no broad prune or lazy unmount.
The required observation interval includes the final scan/sample before stopping
the builder. Post-stop cgroup counters are retained when available; an exhausted
cgroup removed by Docker is explicitly unavailable, never reported as zero.
Observed terminal OOM/PID events fail the diagnostic while permitting proven safe
cleanup. This does not waive any missing observation during construction/export.

Expected retained output comprises create-only `result.json`, bounded original
command/cleanup logs, the policy/owner plans, post-pull outside-storage baseline,
before/after snapshots, interval samples, original-cgroup retirement proofs and
explicit stop/finish receipts. Production also retains fully read
manifest/config/index metadata and the complete CLI archive digest. Pair output
retains the unchanged verifier's result and original provider selection records.
Private command-control files and full OCI/archive payloads are not uploaded.
Successful resource records remain diagnostic evidence only.

### Reader fix provenance and evidence status

The #528 commit `70943f419f4af2a619d74ea9943549305eeb6d23`, based directly
on frozen c302, is included with its original commit provenance. It closes #526
and binds runtime declarations to the hash-bound source-derived runtime closure.
The integrated library workflow test now follows the independent native browser/
library owners while preserving both architectures, full evidence and final gates.
Its exact imported patch is independently reviewed. The composed source now
passes all nineteen original library/shell tests using the actual pinned-Rust
semantic helper. This host evidence does not qualify the final SDK or substitute
for fresh DK-17 on the final source.

The producer froze the private raw-log bundle at version 1, 50,816 bytes;
its persistent Library identity remains in the private artifact index. Its ZIP
SHA-256 is `25cb2ee0a602b0e3fe3a2c72fbc43cf33a775c578c58ee36eb0a93adf1174828`
and expected `INDEX.json` SHA-256 is
`0b10c82ce3651de7c00e5f578f60ff34a3e4a1c2a6730dabd918b3a4d85a2eb8`.
Both supported consumer-local transfer attempts failed before installing a file.
The parent independently materialized the exact frozen version through the
supported Library helper, verified both hashes and all 19 indexed payloads, and
checked 20 unique safe CRC-checked members. Its raw-log review confirms fresh
DK-17 passed (1/0/0, 207 filtered, 134.94 seconds); the exact native owner invokes
the bounded real replay and requires baseline pass, 16 mutations rejected and
restored baseline pass. The retained replay has that receipt. Library/shell units
have 19 passes and targeted browser capture has three, with no failures or skips.
This is parent independent review, not a successful local download here. The
bundle contains no separate retained fresh native package and does not prove full
SDK or rehearsal acceptance. Compatible retained pre-fix outputs remain
separately labeled compatibility evidence.

### Binary source, prerequisites and version contexts

The reviewed #529 source carrier is
`0b9030962e83c68d7123962a24ae8c274ad82068`, tree
`53a9f5bad39679e4b5ed551d79f17e588856dae8`, based on frozen c302.
Its scoped patch is composed with #528 and this construction graph; no main or
release branch is merged. The separate semantic-helper setup is the seven-file
bounded delta at `867f02751885dae79c1229752f773bd803267893` over
`d00f336720823426c3b6951ae15d80d645137298`, adapted to preserve both language
consumers. The parent independently reviewed the original private #529 bundle
and its 133 payload hashes, including DK-36 at d80650ee (242.95 seconds), its
11-mutant language-1.2 replay and original component tests. Those records remain
component provenance, not acceptance of this composed source.

The existing two native SDK owners consume the authenticated immutable SDK for
both languages 1.1 and 1.2. They retain separate original logs and complete hidden
proof directories. The existing final gate requires four archives:
`sdk-amd64-binary-sdk.tar`, `sdk-arm64-binary-sdk.tar`,
`sdk-amd64-binary-sdk-language-1.2.tar` and
`sdk-arm64-binary-sdk-language-1.2.tar`. Their packer re-runs the original
source/inventory/result/proof readers, reads both complete evidence trees and
checks custody again before publishing each create-only archive. It grants no
new acceptance from archive construction. All original six products, sixty-four
contracts and five separate Binary contracts remain. `raw-file-cli/1` keeps its
bounded scope; this change makes no full streaming claim.

Current SDK capture, schemas and consumers require version 0.4.0. The separately
pinned immutable 0.3.0 formatter retains its explicit historical context; a
self-reported lock cannot set the expected current SDK version. The reviewed
Binary controller source pin is updated without changing the existing
application-generator baseline. Normal source writers, genuine final-source
DK-17, both native installed Binary owners and released-template verification
remain required before release qualification.

Host proof readers need both the real Rust semantic helper and locked Node
oracles. The setup conserves the selected clean source/tree, Cargo lock, Rust pin
and oracle manifests; it builds Rust offline after locked acquisition, installs
Node oracles in a fresh private sibling directory with `npm ci --ignore-scripts`,
and probes actual YAML/Ajv modules from that directory. Both readiness paths are
published only after all checks. Host `NODE_PATH` binds that private dependency
closure; it does not override the SDK's own installed dependencies. The hosted
#530 Rust preparation evidence covers its original prerequisite only; this Node
addition and the final composed source need their own verification.

The concrete failure is the workflow readers' call to
`createRequire('/opt/prismpm/oracles/package.json')('js-yaml')` in
`library-sdk-check.test.mjs` and `binary-sdk-check.test.mjs`: their actual selected
tests return `MODULE_NOT_FOUND` with pinned Node and `NODE_PATH` unset. The
same dependency is loaded at the diagnostic policy test's entrypoint. A shared
`source-oracles-bootstrap.sh` supplies the locked Node closure alone to that
policy after plan admission, and supplies the Node component of the four
compound host preparations. Compound readiness stays unpublished until Rust and
all final source checks also pass. No original test is skipped.

Read-only inspection of exact #530 source
`fc0bee6383ec146c578045114b123b3fa67477e2` found that its original host boundary
tests do not have those YAML workflow-reader imports. This does not establish a
matching defect in its ongoing SDK builds; that branch was not changed or rerun.
The separate genuine cold npm component installed 171 packages with Node 22.23.2
and bundled npm 10.9.8 in 3.379 seconds, using an empty private cache/config and
no ambient `NODE_PATH`. Actual confined YAML, Ajv, Draft-04 and Ajv 2020 probes
passed, and source/copied manifests remained unchanged. This component result
is not clean-source compound readiness or installed SDK acceptance.

The original pure-Rust model writer and its read-only check now pass on the
composed functional source. The generated delta adds the DK-36 conformance row,
five contract rows and the new Binary stdlib source to `sdk/stdlib-sources.tar`;
the spec row ordering follows the model. Required `cargo fmt --check` also passes.
Fixture outputs, shared/native golden records and the generated stdlib crate and
release sealing still need their original owning environments and commands.


Current executor preflight finds the exact Rust and Node tools, but no admitted
Elan/Lean/Lake. Its `timeout` 9.7 fails the exact 9.1 launcher pin, and its Debian
13 host is outside the shared/native golden platform contract. No development
or SDK image is present locally. The normal writer chain therefore still needs
the pinned Debian 12 environment, followed by both actual native Ubuntu 24.04
owners. The shared golden currently lacks the new Binary model source, so native
preflight correctly refuses it. These execution requirements are not waived by
the source review or by the original component logs.


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
