# DK-23 component verification

The registered `conformance_dk_23` gate passed in the PrismPM devcontainer:
1 passed, 0 failed, 174 filtered out; 407.83 seconds. It invokes
`node --test --test-concurrency=1 --test-timeout=3600000`
with `wire.test.mjs`, `dom.test.mjs` and `sdk/browser/presentation.test.mjs`.

Fresh LexLean source, kernel and exact axiom checks preceded generated std,
no_std and two byte-identical Core-Wasm builds. Acceptance covered 109 wire
vectors, ten typed intent-binding cases, nine exact 64 MiB frames, one-over
and aggregate-fuel refusals, 12 actual browser journeys, native replay of 117
observed generated calls, nine DOM mutations and two rebuilt source mutations.
Every maximum frame also executed and rendered in Chromium; no test was skipped.

`xtask validate` passed: current generated documents/source archive, 175 registered
capabilities, 86 diagnostics and 158/158 audit tests. The audit used a disposable
copy of the same devcontainer image, UID 1000 with its Docker group, both workspace
path bindings for linked Git metadata, locked oracle dependencies, authenticated
bootstrap runtime and the existing offline Cargo registry cache. Scoped Clippy
(`repo-conformance`, `xtask`, all targets, warnings denied) and Rust formatting
also passed. No test or source change accommodated the environment failures.

- Source: `476dfc5bfe4b6bedc0ebcca8b1d06807b1473a5a75b5a853096bbfccd81c071a`
- IR: `041be04af3a4fb6881a66aa0b31536f75ecc6ca33df0314c8e52f312ee61eeb3`
- Wire Wasm: `7ff5772e8a9f2a3cf7ae46a0caa10002ca334cd515c7e0d786fa1c79a4eded09`

The unchanged frame and Wasm limits are 67,108,864 and 1,073,741,824 bytes.
Measured generated-Wasm memory for the eight combined-frame cases:

| Case | Payload first | Payload last |
| --- | ---: | ---: |
| 256 nodes | 674,430,976 | 540,278,784 |
| 4096 one-cell rows | 750,125,056 | 683,081,728 |
| 4096 cells, 16 columns | 744,620,032 | 677,511,168 |
| Combined structural maxima | 757,661,696 | 690,552,832 |

These are observations, not a whole-browser heap-fit guarantee. The source
serializer preserves the flat canonical wire while using bounded private row
chunks and one output accumulator; no compiler or resource-limit change was
needed. Isolated maximum tests do not replace combined-shape acceptance.

This is private component evidence, not public application, credential custody,
accessibility certification, SDK release, Foundry or deployment acceptance.
`PP2011` remains required. Integration must run the ordinary combined model,
golden, package and installed-SDK gates; recorded hashes never replace execution.
