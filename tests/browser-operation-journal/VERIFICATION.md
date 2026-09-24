# DK-24 verification

Private durable operation journal only; SPEC §12.12. This does not accept a
public application, account recovery, distributed consensus or Foundry release.
`PP2011` remains enforced. Acceptance below is limited to this prerequisite.

2026-09-20, non-root PrismPM devcontainer, pinned locked/offline toolchains:

```sh
cargo test --locked --offline --jobs 1 \
  --config profile.dev.debug=0 --config profile.test.debug=0 \
  --config build.incremental=false -p repo-conformance --test conformance \
  conformance_dk_24 -- --exact --nocapture
```

Source-exact owner passed on `4d8c2e5`: 28 underlying tests, 723.03 seconds,
no skips. The unchanged `conformance_dk_20` regression passed with the same
command options: 18 underlying tests, 293.84 seconds, no test edits or skips.

- Fresh LexLean source/kernel/axiom verification and independent complete native,
  journal Wasm and partition Wasm artifact equality.
- 1,092 journal/history vectors and eleven exact/over-limit cases, replayed twice
  in generated std/no_std and Wasm where within the declared allocation bound.
  Includes 1,024 records, maximum actual request/result closure, exact 64 MiB
  partitioning, native maximum+1 rejection and actual Wasm allocation traps.
- 22 real-browser journeys, 651 journal/effect/partition/guest calls and 213 custody calls
  replayed in generated std/no_std; altered transcript rejected. Covers real
  competing contexts, staging races, 4,096-object exhaustion, strict transaction
  faults, changed bindings, forged signatures and opaque source-bounded signing.
- Actual 64 MiB browser/native fixture SHA-256
  `a5f6c73b36931fda5707b7783abcaf63a16541d291a5e382126d75a8c4618ac7`;
  peak generated memory 135,266,304 / 1,073,741,824 bytes.
- Twelve host mutants and five genuine source mutants rejected, including
  exact partition boundaries, terminal-slot reservation and descriptor capture.
- Non-writing model/spec-link audits passed: 178 IDs, 86 diagnostics. Scoped
  repo-conformance formatting and warning-denying clippy passed; diff clean.

Source `c37cf171545580646bbcfe3a49ff7ac9eb9c38dd6140faf7dc11d0af78b36c7f`;
journal Wasm `fa556d64725738a3046a86bc02c66c74c0551bbfb0a75f49d05b363413b1bb4c`;
partition Wasm `623e275ce5744c79445851eb4e7b0e8be2636983ea024cca8edc3f975fe379b2`.
Retained evidence `/tmp/prismpm-operation-journal-jSD87Z/`:
acceptance JSON `671f6eea6a4f751225f03709daf17cb24261b46f4d8e889e5509be53f21fd85c`;
browser evidence `35bdb94d6c3a6837ad573caf4315b88b99aab99a007b847ae2594b616954d977`.
These local diagnostics are never acceptance inputs or caches.

Additional executed RED: valid own descriptors with hostile Proxy getters caused
`descriptor snapshot invoked caller property getter`. The corrected host captures
object values and array length once; actual browser tests cover nested objects,
arrays, payload transport and rejected accessors. Both corresponding host mutants
failed the actual browser oracle. Full browser evidence is retained and hashed.

Combined package proofs, installed-SDK and product release acceptance remain
independent integration gates. No public application acceptance is asserted.
