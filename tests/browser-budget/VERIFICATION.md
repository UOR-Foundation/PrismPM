# DK-27 verification

Private conditional per-resource admission only. No grant issuance, policy
authentication, public browser build or deployment acceptance is implied.

The registered `conformance_dk_27` owner passed in 805.20 seconds in the
`lucid_ride` devcontainer: 13 tests, no failures, skips or unfinished tests.
It verified actual LexLean sources and axioms, kernel export, generated native
`std`/`no_std` and two byte-identical Core-Wasm guests. Each mode repeated all
179 vectors and 25 maxima. Combined coverage includes 64 maximal resource names
and sixteen 1 MiB committed objects, plus a wire-representable one-over rejection.
Peak Wasm memory was 134,348,800 bytes; the 1 GiB and 64 MiB limits were unchanged.

Seven freshly compiled source mutants exercised identity, policy, manifest,
request, coverage, ordering and payload-limit guards. The complete 811-file
source/compiler/helper/pin closure stayed fixed across the positive and mutants.
Every completed build retained original tool identities and proof/artifact
outputs while retiring only its private Cargo-driver and Lake-exporter caches.

- Source: `13474ebab78c210f9d3fa125c49fda955bf940dfa88250b1d9c098bae81b2c0f`
- Attestation: `27141a110e8f93a799d9f8bcb6bce1eb7de85d55045e24dc9af5ec9ecf329213`
- IR: `271c02f5f4b74df8226f3ecfe934a9342c78ca524b3aa65de267630eda6e0e58`
- Wasm: `13fec53fd02714d70629b23edee141d2007748832b2beabe036702b9859c4d8c`

Evidence: `/tmp/prismpm-budget-4BwJ6k/budget-{wire-evidence,acceptance}.json`.
Owning log: `/tmp/prismpm-budget-checks-BYJhAt/target/dk27-owner-final.log`.
An earlier 23-maximum diagnostic was deliberately stopped and is not acceptance.
Normal model generation, Cargo-entry audit (175 tests), authored formatting
(5 tests), scheduling (3 tests) and cache-lifecycle checks (3 tests) passed.

Combined with OC-09, the normal Cargo-entry audit passed all 189 tests.
Independent golden generation and readback matched 357 files: the new Budget
source and actual compiler-attestation bindings were the only baseline changes.
The 53 generated modules were unchanged. Full authored formatting (5 tests),
scheduling (3 tests) and expanded cache-lifecycle checks (4 tests) passed.
Golden readback log SHA-256:
`92949353c03a152b1c0f82dff3cb223d9ed3c0cc0b276cbef3cac55034e02fb6`.
These source checks do not establish native SDK or Foundry release acceptance.
