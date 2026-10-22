# DK-23 component verification

## Status-region correction — 28 September 2026

The real-browser regression first failed because the status message was
populated before its region was connected. The adapter now preserves one
connected, explicitly atomic region and writes only changed messages.

The exact four registered Node files passed all 30 checks in 917.67 seconds
inside the pinned SDK container, without skipped or unfinished tests. All 28
journeys execute in Chromium 151.0.7922.34, Firefox 153.0 and WebKit 26.5. Each
engine's 475 generated calls replay identically in native std/no_std execution.
All 28 actual adapter defects and five freshly compiled source defects fail.
All existing structural, secret and progress maxima remain required, including
nine exact 64 MiB frames rendered in Chromium. Peak observed generated-Wasm
memory is 757,661,696 bytes, below the unchanged 1 GiB ceiling.

The separate presentation/semantic/harness suite passed 14 checks, including
the unchanged pinned axe oracle, six semantic defects and oracle-failure checks.
Independent review caught a three-engine native replay filename collision;
closed engine-specific filenames now refuse overwrite. Actual initial-mount
and retained-child-removal failures terminate ownership and clear listeners.

WebKit's global `SharedArrayBuffer` constructor is absent in this pinned build.
Its actual shared WebAssembly memory still supplies the native shared-buffer
brand; every engine must reject that buffer even with a spoofed own `buffer`
property. No synthetic replacement or omitted shared-memory refusal is used.

An initial setup run failed on a missing offline `syn 3.0.5` archive. Restoring
the exact Cargo-lock checksum from the existing SDK cache allowed the complete
run; no dependency pin changed. Formatting, model regeneration/readback and
independent source review passed.

- Source: `9e7df7539b43d7af9b809209b4e5a7d719e3b2f18360ba66ffb90c79c7b17bc2`
- Attestation: `5fb87434a8902ddbe437a1ea0bc920d1d34a73bf7f8fe1f206469e8f6f9a9e98`
- IR: `e12fd1f1f6fc986be5afb039cd8bf43b5620b9dcd6c36ed9de5247b42f20fe4d`
- Wire Wasm: `77c7cba7449ed9add9b5adf1cf11cf0ebbf1acd06d1c4c25937c90df43034e38`
- Each browser transcript: `9d820cbf4da6b230f273628f861f3bfae7bc956e9fb52aded1be0b2ea0f14a15`

This is private component evidence. Stronger complete generated-package and
observer capture, combined SDK gates, installed SDK and Foundry integration
remain required. DOM tests do not measure screen-reader speech or establish
complete accessibility, application or deployment acceptance.

## Earlier component receipt

The registered `conformance_dk_23` gate passed in the pinned devcontainer:
1 passed, 0 failed, 0 ignored, 178 filtered out; 666.44 seconds.
All nine selected Node tests completed without skips or unfinished cases.
Normal source-archive/document regeneration and `validate-model` passed
(179 capabilities, 86 diagnostics). Scoped `repo-conformance`/`xtask` Clippy
with warnings denied and formatting checks passed; these are not full SDK V&V.

Fresh LexLean source/kernel verification audited 257 declarations before
generated std, no_std and Core-Wasm execution. Coverage includes 121 wire
vectors, 17 typed field/intent cases, eight secret-route cases, 17 Chromium
journeys and native replay of all 207 observed generated calls. Four rebuilt
source mutants and 14 actual DOM implementation mutants were rejected.

All nine existing exact 64 MiB presentation frames remain required in native,
Wasm and Chromium execution. Secret coverage additionally executes the raw
field predicate at 64 MiB and one byte over, and the largest canonically framed
secret intent through the real codec, bound route, ephemeral modeled sink and
browser capture. Native and browser request/result hashes must agree. Field
and frame overruns cannot reach either dispatcher; there is no truncation.

Public framing remains bounded at 67,108,864 bytes; generated-Wasm memory
remains bounded at 1,073,741,824 bytes. Only the test-only raw field probe admits
67,108,865 input bytes to execute the semantic rejection. This is not a public
request limit or a whole-browser memory guarantee.

The initial raw UTF-8 fixture failed `LLV7005`: its decoder requires the same
exact `Classical.choice`, `Quot.sound`, `propext` policy as the existing fixture
decoder. The corrected fixture's exact observed audit passed. All 44 existing
Model and 134 Wire declaration policies are unchanged; all five new production
predicates passed `none` policies with no observed axioms.

- Source: `ee2b32ef77760c60cd62297b836fef775326213c1cd48028e4759e66f9352bb5`
- Attestation: `51325e0d29c9409f0f0f71a11816e27e1ef3ea2bd5c127e48bafe705533c695b`
- IR: `0216c078dea833875cc512c6a2e3516d64ca897a3e9a84aca6c753ae965e5850`
- Wire Wasm: `d05824fac2f4feb24f3037d557f1efc2a04079a0ecaab6e62a9364d3fd8193a5`
- Browser transcript: `4b3eba82547424c4d616783a658a14a4ce586d179587102ac032279f7f20049d`

The adapter clears live secret controls before the private sink, across modeled
context/lifecycle changes and on close. It does not log, persist or supply
secret defaults. Test transcripts contain synthetic public strings only.
Trusted sinks must sanitize evidence before durable application admission;
arbitrary callback behavior and JavaScript memory erasure are not guaranteed.

This is private component evidence, not account enrollment, mailbox proof,
recovery, accessibility certification, a public application, SDK release or
Foundry deployment. `PP2011` remains required. Integration must still regenerate
and verify the combined stdlib package, goldens and installed SDK.
