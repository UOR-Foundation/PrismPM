# Command memory regression

20 September 2026; source baseline `2f06ff29988989c84aef91bbdc19c1702a291f80`.

The complete owning test reproduced the Bootstrap CI failure from
[run 35489406200](https://github.com/UOR-Foundation/PrismPM/actions/runs/35489406200):
the adapter expected 58 pages, but measured 3,276,800 bytes (50 pages).
All 75 literal vectors, native/no_std executions, real Chromium crypto/journal
journeys and six adapter mutants passed before that obsolete assertion failed.
The subsequent missing `adapter.diagnostics` result was a consequence of the
failed adapter subtest, not a second production defect.

Both exact measured-peak assertions now require 50 pages. The independent
64-page compiled maximum, allocation rejection, literal corpus, real browser
journeys, native transcript replay and planted defects are unchanged. Failure
output now includes the actual bytes and pages.

Verification uses the repository devcontainer image
`sha256:e88f0de6f6ab3fa08f04445296d0eb0743eac82a1dc4cded5263d90c7e6ae955`,
UID 1000, pinned compilers, offline inputs and `CARGO_BUILD_JOBS=1`, without
compiler/profile overrides. A clean source copy outside the enclosing
workspace avoids inherited Cargo workspace settings.

Command: `node --test sdk/browser/command-model-test.mjs`.
Restored result: 12/12 passed, zero skipped, in 259.77 seconds. The status-only
adapter exercised 22 browser journeys and replayed 2,996 actual calls natively;
the separate crypto/journal closure replayed 1,636 calls. All six adapter
mutants and the capture/close omission mutant were rejected.
Adapter transcript SHA-256:
`0499538e83639344a876e28602aac2f8928de09da2a16d893fc8eb6aaa19c3fc`.
The generated command Wasm is
`09f1016cb34e818573f8ee2b04a19afc97fcae9f95abfbcc76440267477d3f80`;
its literal-corpus worst case is `SignMaximumHeadPost`.
This regression check does not establish SDK or Foundry release acceptance.
