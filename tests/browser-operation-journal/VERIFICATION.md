# DK-24 verification candidate

Private durable operation journal only; SPEC §12.12. This does not accept a
public application, account recovery, distributed consensus or Foundry release.
`PP2011` remains enforced. Final acceptance is pending below.

2026-09-20, non-root PrismPM devcontainer, pinned locked/offline toolchains:

```sh
cargo test --locked --offline --jobs 1 \
  --config profile.dev.debug=0 --config profile.test.debug=0 \
  --config build.incremental=false -p repo-conformance --test conformance \
  conformance_dk_24 -- --exact --nocapture
```

Completed predecessor gates: one registered owner, 25 underlying tests in
603.49 seconds; then 26 tests in 647.22 seconds, no skips. The latter includes
the direct partition-boundary source mutant and one-free-record reservation.
This follow-up changes host descriptor capture; final acceptance remains pending.

- Fresh LexLean source/kernel/axiom verification and independent complete native,
  journal Wasm and partition Wasm artifact equality.
- 1,092 journal/history vectors and eleven exact/over-limit cases, replayed twice
  in generated std/no_std and Wasm where within the declared allocation bound.
  Includes 1,024 records, maximum actual request/result closure, exact 64 MiB
  partitioning, native maximum+1 rejection and actual Wasm allocation traps.
- 21 real-browser journeys, 577 journal/effect/guest calls and 171 custody calls
  replayed in generated std/no_std; altered transcript rejected. Covers real
  competing contexts, staging races, 4,096-object exhaustion, strict transaction
  faults, changed bindings, forged signatures and opaque source-bounded signing.
- Actual 64 MiB browser/native fixture SHA-256
  `a5f6c73b36931fda5707b7783abcaf63a16541d291a5e382126d75a8c4618ac7`;
  peak generated memory 135,266,304 / 1,073,741,824 bytes.
- Ten host mutants and four genuine source mutants rejected.

Source `c37cf171545580646bbcfe3a49ff7ac9eb9c38dd6140faf7dc11d0af78b36c7f`;
journal Wasm `fa556d64725738a3046a86bc02c66c74c0551bbfb0a75f49d05b363413b1bb4c`;
partition Wasm `623e275ce5744c79445851eb4e7b0e8be2636983ea024cca8edc3f975fe379b2`.
Retained predecessor evidence `/tmp/prismpm-operation-journal-pWcOqZ/`:
acceptance JSON `449eb28e1476a399d6f3c6627ea3b236b5192294ef7f1a4ae6c5509f0d2ace9d`.
These local diagnostics are never acceptance inputs or caches.

Additional executed RED: valid own descriptors with hostile Proxy getters caused
`descriptor snapshot invoked caller property getter`. The corrected host captures
object values and array length once; actual browser tests cover nested objects,
arrays, payload transport and rejected accessors. Two corresponding host mutants
are registered. Full browser evidence is now retained and hashed by the owner.

Pending: fresh complete 28-test owner, unchanged full DK-20 regression and final
ancillary checks. Combined package proofs, installed-SDK and product release
acceptance remain independent integration gates.
