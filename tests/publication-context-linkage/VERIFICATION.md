# OC-10 conditional checkpoint

Status: incomplete. No SDK acceptance, authentication, readiness or deployment
authority. Public document versions, PP2011 and compiler/SDK pins are unchanged.

## Stable source and generated package

Normal `xtask validate-model --write` and readback passed. Normal
`xtask stdlib-package --write` and readback passed with all 57 registered exports.
The crate archive/release metadata were not republished.

| Artifact | SHA-256 |
| --- | --- |
| LinkageV1 source | `ddad045aaf666d136267df0570a831924636bf007c98d8f3ba53e085bfb40d70` |
| LinkageV1Wire source | `c4f9fe47c4af5eddee2300a4eff1fc3a19f07c731faa7f3db380bca94e3e070a` |
| Branch-specific source archive | `a4597c7ee72efd65060737852ca71b1d9642c1af87a7dad138821bd1b47d159b` |
| Generated library | `d6c0df179f1c6bf7df194156140d87c31772ffe29da4cfe4baccca8ae631e1f6` |
| Generation manifest | `dd1c8892481d8608f1b8f8954eb72781f0b5976aa902d62061a6f32c18d208c5` |

The six CBOR primitive source/test changes are identical to reviewed PR #51
commit `377f2b4`; its differently based source archive was not copied here.
That separate exact-candidate ST-16 gate passed 194 roots, 342 kernel audits,
274 Wasm cases, 71 independent byte witnesses and five source mutants.

## Actual private capture

23 focused Rust boundary tests passed with zero ignored; the latest checkpoint
rerun passed in 5.09 seconds after the new graph-negative owner compiled.
The fresh actual
source-free capture test passed in 1,084.98 seconds, including a disk-floor pause.
It checked the actual fixture Git commit, kernel-verified build, complete OCI
replay, all SDK inventory rows, seven generated linkage preimages, generated
six-field context and unchanged OC-09 roundtrip, three publisher substitutions
and four invalid attempt bounds.

Runtime: `127.0.0.1:5000/prismpm-sdk@sha256:f0f5b5c2ea7fc216ffb309616e42ddf5ffd541855a8c502f71df9b212cbc4891`.
Test executable: `9523de17a0437f82dc28d8f71e14361a04c7dfbd51ee7997994bd2ef88230667`.
Receipt: `3b7eac38f8cebfc0ac7e00848247b589e7d83cfc87139293a40221ed9cf1f1b0`.
Retained in container `prismpm-publication-capture-compatible`, directory
`/tmp/prismpm-oc10-captured-release-1oKJ5s`.

The receipt reports `installed_sdk_source_matches=false`. The older image's
complete provenance remains unchanged; only the private measured-fixture source
constructor succeeds. Image `60226bc7…` independently failed PP5301 because its
actual inventory lacks `playwright-driver`; no inventory rows were fabricated.

## Required remaining gates

- Execute the newly compiled 22 actual changed/missing OCI blob negatives,
  including exact refusal reasons and pristine replay between mutations.
- Finish the fresh capture owner, complete source/executable custody and independent
  captured-metadata/preimage oracle; `capture-owner.mjs` is intentionally absent,
  so the registered full OC-10 owner cannot pass accidentally.
- Pass all 33 unchanged resource cases with the pinned compiler. Conditional
  unmerged-compiler diagnostics pass 119 vectors twice in std/no_std/Wasm and
  31 maxima; both widest-services-plus-64-MiB cases still trap under the 1-GiB cap.
- Complete all 17 compiled source mutants and artifact/receipt substitutions in
  the full owning gate; retain unchanged OC-07/08/09 requirements.
- Rebuild the normal SDK, verify its complete installed inventory/source/corpus
  bindings and owning gates, then integrate the separately authenticated host.

Collector/bitset experiments remain outside committed source. Direct conditional
witnesses pass 260 collector cases and 132,320 bit/coverage/fuel cases in both
native modes; they do not close the two failing aggregate Wasm cases.
