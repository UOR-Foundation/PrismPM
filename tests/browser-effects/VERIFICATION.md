# DK-20 component verification

Devcontainer command: `node --test --test-concurrency=1 --test-timeout=3600000 sdk/browser/effects-wire.test.mjs sdk/browser/effects-module.test.mjs sdk/browser/effects-test.mjs`.

Passed 18/18 tests in 314.169 seconds; no skips or cancellations. Fresh source,
kernel and axiom checks preceded generated std/no_std and reproducible Wasm.
The gate executed 242 wire vectors and five maxima twice per runtime, 30 real
browser journeys, 168 browser transcript calls in both native modes, ten host
mutations and three independently rebuilt LexLean mutations. Peak wire memory:
978,124,800 bytes within the declared 1,073,741,824-byte maximum.

- Source: `8f038f9e479c5dc024c4dd66448affce8323a3b1581557cd9835748ea91503d3`
- IR: `151701f2490d265381a117b97fd468a07a5c97831bd0739603ebb29a60131872`
- Wire Wasm: `c0e6497e7f8f99b6d951178e361cdc2e231c58687ffab58b7af8c65bdc477740`
- Guest Wasm: `09db3e9796b4ce7a567273495b6bcd69adb13d15b6b80d515cb0dd5b4866afe7`

This records component verification, not an application or SDK release. The
integration must regenerate the registered documents/source archive with the
established writer and pass its normal validation, golden, package and SDK gates.
Historical hashes never replace a fresh owning execution.
