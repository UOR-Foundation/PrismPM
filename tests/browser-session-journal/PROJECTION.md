# Private projection component

27 September 2026: freshly captured LexLean/kernel/axiom/LCNF verification,
native std/no_std, and two independent Wasm builds per entry passed:

- 904 predecessor, 895 unchanged Session, 596 observation vectors, each twice.
- 27 predecessor and 21 observation aggregate/limit vectors, including actual
  64-MiB frames. Peak Wasm memory: 805,240,832 / 486,932,480 bytes, below 1 GiB.
- 821 source/compiler/tool/fixture inputs captured and rehashed, including
  transitive static ESM dependencies; completed private tool caches retired.

Source: `e58517e729ef02d005f048254814af9e603eb6a4135e27ab4ef028f272bd5712`.
Attestation: `1fc94a1d3a1d7f7efa0cc0349fdeda4975a13da7c1a407318b98df9843c1d08a`.
Component receipt: `b3ae6d4d7142d0b3557b2ed82b9236753b9a8727b80f266158e36d8f3d887d69`.

Owning functions: `verifyProjection`, `verifyProjectionMaxima` and
`verifyObservationMaxima` in `projection-checks.mjs`. Run in the devcontainer.
Construction tests independently bind the original complete parser shape and
reject captured-source substitution and incomplete native result inventories.

The current private `projection-owner.test.mjs` additionally requires eight
compiled source defects, all 48 actual maxima and before/after executable
identity checks. Complete native/Wasm package inventories are captured immediately
after generation and checked before/after compilation. Substituted native source
must fail before either observer is created. Thirteen construction/provenance
checks pass; the new complete captured closure has not yet passed the full owner.
The historical receipt above does not attest these new harness inputs.

This is not complete DK-30 acceptance. Compiled source mutants, authenticated
journal/host composition, freshness, retirement, browser faults and installed
SDK verification remain mandatory. No public application gate is opened.
