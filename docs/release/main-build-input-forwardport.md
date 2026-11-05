# Main build-input forward-port for #521

This focused forward-port starts from main commit
`ecb594bd5640f3cb4bdc567c096c20f03209bd0d`. The release/v0.4 implementation is
separate (#524). No branch merge, version-policy change, generated probe output,
or acceptance receipt is imported here.

## Construction and ownership

A short `policy` job keeps main's existing 0.4.0 publication rules: an explicit
workflow dispatch on `main`, or the `v0.4.0` tag. Cargo remains an explicit
publication choice, and accepted discovery aliases still require the original
complete prerequisites. The publication and signature policies are unchanged.

Each native architecture owns one authenticated SDK input acquisition, one
private verified recipe context, and one pinned BuildKit builder. Its closed
build set constructs the five existing sdk/Dockerfile products and the CLI
archive. The standalone runtime keeps its separate Dockerfile. All six original
image products, both architectures, source labels, complete SPDX transport,
signature verification, and exact source-addressed index identity remain.

`sdk-supply` signs and verifies the SDK index independently of the other image
supply rows. The original source devcontainer verifies that exact SDK through the
pinned trust root, exact source/ref/trigger signer identity, both native platform
manifests and configurations, and its own loaded Docker image identity before
exporting `PRISMPM_TEST_SDK_IMAGE`. It then executes both unchanged full source
`just vv` runs, retaining both complete bootstrap closures under the existing v1
source schema. A transport receipt is retained separately and grants no source,
SDK, or release acceptance.

Native equivalence consumes the CLI archive from that construction owner and
runs its original independent comparison. The installed-SDK owner still runs
both complete isolated native verification passes and the product CLI check.
Browser/native-library checks, twelve independent reproducibility comparisons,
all original final gates, and Cargo-dependent release prerequisites remain.

Main preserves its full v1 owners. Installed-SDK verification and reproducibility
still depend on `gate` and `images`; this is not release/v0.4's finer verification
fanout. Full source and installed runs retain their existing six-hour job bounds.

## Reruns and retained originals

Source, installed-SDK, native equivalence, and native archive artifacts use names
bound to source revision, architecture, SDK digest, run ID, and producing attempt.
The collector retains unmodified current-run artifact and all-attempt job API
records. Selection downloads immutable artifact IDs from the latest successful
owning attempt. A failed, skipped, incomplete, future, foreign, duplicate, or
missing latest owner cannot fall back to an earlier pass.

Main's source/native byte validators still require the exact producing attempt.
Packaging supplies the selected original attempt rather than relabeling old
records as the current attempt. Both source runs and both bootstrap closures
remain mandatory. Installed evidence packaging descends into its original
`sdk-ARCH-full-sdk-vv` directory; the sibling outer-owner records remain retained,
and private Docker configuration is excluded from upload.

## Storage and execution measurements

Each reproducibility root owns a fresh pinned builder with no imported build
cache. After exporting its complete OCI layout, the workflow removes that
builder and refuses a retained state volume before starting the next root. Both
raw layouts remain available through complete byte/custody verification. Direct
OCI-directory export avoids a duplicate outer tar archive.

The hosted workflow records allocated storage before construction, after each
builder retirement, and after final comparison. Observations include the actual
Docker root, independent OCI layouts, filesystem free space, and Docker storage.
These records are diagnostics, not acceptance or a capacity proof. Descriptor
sizes do not establish unpacked or allocated disk use.

A hosted execution should record, per architecture, construction acquisition
count and elapsed time; builder lifetime; source/installed/native owner times;
allocated bytes at each checkpoint; complete resulting image/archive identities;
and the final gate results. Compare equivalent committed inputs on comparable
runner images, preserving raw records and original failed executions. No speed,
space, reproducibility, hardware, or SDK-acceptance claim is made by local unit
fixtures. No hosted run or release qualification is included in this change.

## Local verification limits

The bounded local checks cover the actual producer/caller shell transport,
closed build sets and custody adversaries, main publication and signer policy,
provider-based rerun selection, complete v1 evidence retention, independent-root
lifecycle, and installer retry/integrity handling. Synthetic transport fixtures
cannot qualify SDK outputs.

This environment lacks Cargo and the installed historical bootstrap runtime; its
system Buildx is 0.31.0 rather than the required 0.28.0. The full kernel process
owner tests also require Linux child-observation support absent here. Those
existing integration checks must run in their owning pinned CI/devcontainer.
