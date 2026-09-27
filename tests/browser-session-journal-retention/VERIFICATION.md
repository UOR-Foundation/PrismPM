# Private retention component

27 September 2026, pinned `prismpm-dev` devcontainer:

```sh
node --test --test-timeout=3600000 tests/browser-session-journal-retention/owner.test.mjs
```

Passed 10/10 TAP tests in 956.109 seconds, without skips or TODOs:

- Fresh pinned source, kernel, exact axiom inventory and LCNF verification.
- 86 independent vectors in native std/no_std and actual Wasm; two independent
  Wasm packages have identical bytes.
- Six joint maxima, preserving 64 roots × 4096 references, 4096 objects and
  16 additions. Peak Wasm memory: 842,858,496 bytes; unchanged 1-GiB memory,
  64-KiB stack and 64-MiB frame limits.
- Nine actual Chromium storage journeys: persistence, competing connections
  and tabs, process restart, shared/staging protection, abort, quota and delayed
  acknowledgement.
- Nine independently compiled source defects detected by native and Wasm
  output mismatch, including balanced traversal and canonical reference bytes.
- Frozen generated-package inventories before the first Cargo build; actual
  source/manifest/extra-file/hardlink and executable substitution refusals.

Receipt: `5f8a637551ca59ed81df15c67c60425eca3554b34e52d9ff232a015c8928ed64`.
Source: `8315ea4809fdbbd53cf0b059292bae26f25e022a65ef1d5c4e4b7d5213c709bf`.
Wasm: `92ce26f8513293215320f1aa34922ea4704c92539d63e5684bdd2c88479beddd`.
829 frozen inputs: `d9437e3a77cad619ca1345b1182abcf8ba532d32da9531a04218fe5cb9c52892`
(SHA-256 of `JSON.stringify(receipt.inputs)`). Log:
`target/retention-verification/run-Ftujw1az/owner.log`; complete evidence retained
in `/tmp/prismpm-session-retention-k5Njl7` inside that container.
The adjacent `positive-artifacts.tar.gz` retains source/proofs/IR/native/Wasm:
`0c43971c0156c8c30c8bd27a53c3db26da32c9dd7f4fec8eb7679336bc5263ea`.

The frozen storage host requested, but did not check, reported strict durability;
its correction requires separate evidence. This receipt does not establish
authenticated journal closure, freshness, native browser-transcript replay,
host mutation coverage, joint real IndexedDB capacity, all-browser support,
installed SDK acceptance, public runtime admission or Foundry acceptance.
