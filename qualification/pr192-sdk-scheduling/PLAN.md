# PR192 scheduling qualification

Private preparation branch only. Do not merge this replacement of the existing
`osv-input-qualification.yml` entrypoint into a product branch.

Authority:

- Reusable workflow: `PhaseMirror/PrismPM/.github/workflows/sdk.yml@83b7d9a9905c1bfef787e418ec5e9cc8e0f46453`.
- Adapter: `UOR-Foundation/PrismPM/action@f785ee46d794886c29b7efe80c2b261e48733a15`.
- Historical SDK: `ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`.
- SDK source: `d0174e1d64339f73091fe4c59d5d6bf532a37d1f`; its adapter files are byte-identical to f785.
- Consumer: the original HO11 text application and exact standards lock from d017;
  `consumer-source.json` records all 28 original d017 blobs and SHA-256 values.
  The required SDK lock is copied byte-for-byte from calculator-example commit
  `fb4658adbc2c4b68e6537fd616cfb6896001e3d0`, not manufactured. Its exact SDK
  image and standards-lock digest agree with the selected historical inputs.
  The
  six original application vectors and all original compiler limits are intact.
  HO10 was rejected during source preparation because it lacks the general
  runtime roots required by a full build. HO11 follows the SDK's original
  complete application build/verification branch; compatibility still requires
  the real execution and is not inferred from this source analysis.

One authorized dispatch runs three sequential waves:

1. Default check and build: both genuine SDK jobs must overlap and succeed.
2. Two fetch operations with the same explicit key: both jobs must overlap and succeed;
   fetch is mutating and must not enter the read-only supersession queue.
3. Older build/key-A and isolated build/key-B start together. Only after provider
   metadata observes both actual post-fetch SDK steps running may the newer
   build/key-A be scheduled. Older A's actual build must be cancelled, newer A
   and B must succeed, and B must remain live across the actual replacement.
   B need not outlast the newer runner's entire cold image import.

Seven SDK invocations are the minimal direct witnesses for these three pairwise
claims. This is scheduling qualification, not acceptance of the historical SDK,
the current SDK release, the full CLI, CI, VV, or issue187. Original policy units
cover the other command classifications; fetch does not exercise deployment.
The expected cancelled job remains visible; no continue-on-error or status rewrite
is permitted. The provider's overall run may therefore be cancelled/failed even
when the narrowly scoped scheduling assertions pass. External terminal readback
is required, including if cancellation prevents the final job from running.

Cost and capacity:

- A cold AMD64 runner imports 16 actual compressed layers totaling
  4,162,398,277 bytes. Seven cold imports total 29,136,787,939 bytes (~27.14 GiB).
- Expanded image storage and locked authority downloads are additional and are
  not inferred from compressed descriptors. Fetch uses the unchanged complete
  authoritative catalog, including its large locked vulnerability snapshots.
- Fetch and the following operation share one runner's local image, not one
  image across jobs. No registry publication or image rebuild is performed.
- At most three SDK invocations are in a wave; validator/control jobs add only
  ordinary runners. Original ubuntu-24.04 SDK runners, 360-minute job clocks,
  image entrypoint, Docker limits and capacity checks are unchanged.
- Do not dispatch until the maintainer has reviewed current organization queue,
  runner/storage headroom, and priority of the existing main-repair owners.

Failure rules:

- No synthetic work, sleeps in SDK workloads, fake filesystem/IP, test filtering,
  retries, manual cancellation, second dispatch, or success substitution.
- Bounded polling waits only for genuine provider observations. If operations
  finish too quickly to observe the required state, qualification fails rather
  than inventing a witness or extending the workload.
- The final reader checks exact caller/source/run/attempt, the immutable reusable
  workflow, complete job membership and actual operation steps/intervals.
- Full bounded raw logs and external terminal provider readbacks remain required
  for acceptance; API metadata alone does not prove SDK product correctness or
  physical runner cleanup.

Evidence bounds:

- The observer's unchanged 32-MiB limit charges exact encoded API response bodies,
  transport-header sidecars, observation/result JSON, and the observation bytes
  appended to `GITHUB_OUTPUT`, before writing. A refusal never authorizes overflow
  or a retry; owned evidence files are exclusive and never overwritten.
- Source-preparation manifests, source-native custody and later external review
  receipts have their own explicit bounds outside this runtime observer budget.
