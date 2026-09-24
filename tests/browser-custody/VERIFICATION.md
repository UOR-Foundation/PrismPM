# DK-25 verification

Private credential custody only; SPEC §12.13. This does not accept enrollment,
mailbox proof, account authorization, credential replacement, recovery, a public
browser application, or Foundry deployment. `PP2011` remains enforced.

2026-09-20, non-root PrismPM devcontainer, locked/offline pinned toolchains:

```sh
cargo test --locked --offline --jobs 1 \
  --config profile.dev.debug=0 --config profile.test.debug=0 \
  --config build.incremental=false -p repo-conformance --test conformance \
  conformance_dk_25 -- --exact --nocapture
```

Passed: one registered owner, eleven underlying tests, no skipped tests;
370.29 seconds (390.74 including compilation).

- Fresh LexLean source/kernel/axiom verification and pinned exporter.
- Generated native std/no_std and two byte-identical independent Wasm builds.
- 90 vectors and ten exact/over-limit cases; native and Wasm replay twice.
- 35 real-browser journeys, 173 generated calls replayed in std/no_std;
  altered transcript rejected. Includes 64 real nonextractable keys,
  128-byte names, 1-MiB signing, shared-slot contexts, strict durability,
  concurrent creation, missing/corrupt records, lost acknowledgments and close.
- Eight host mutations and three independently generated source mutations
  rejected by their actual owning execution.
- Largest observed Wasm memory: 6,094,848 / 134,217,728 bytes.

Accepted source `f265d67f99544262438a7b1ebf88961dc25075cee173644d6963bf72c69121cd`;
attestation `38238a2b23ab05f1ff310e5b6e58322aaf9a1ca4b08bef65ef709c6f30dab36f`;
IR `ab2b5cf2d7c2f2eb67f1882834e8d7dad648d1d6591d42e6333d0e191e7b570e`;
Wasm `daa36d3e6fd2b2563c37bd346963b572e7b52160c058a6f9232470f43f3fd563`.

Retained run: `/tmp/prismpm-custody-BxxxOz/`. Its
`custody-acceptance.json` SHA-256 is
`cc6ce761e06ecd2cd3a39d088f579443a982a99bc980c5e7f1e9880a17300a44`;
wire evidence `ca7250e77547c0c39e27d4d2ac13f3c2374f45b3642172c0e9e99f6c91059e78`;
browser evidence `f46adb5bc9bf6a2421f4a3c048926336164744929d64e3d23f2722f2e83e76b2`.
These local evidence paths are not build inputs or acceptance caches.

Observer: minimum free disk 16,984,104,960 bytes; minimum available RAM
19,465,318,400 bytes; container peak 16,328,663,040 bytes including concurrent
owners. No resource abort. Log SHA-256:
`02afd3e773e5e9bcdfcda4cba81865d3b085e1e06813092b0e9f258fe9f42152`.

Normal source archive/register regeneration is included. Combined SDK package
proof, integration-wide goldens, full-suite and installed-product publication
remain integration gates; this component result does not substitute for them.

Also passed: non-writing `validate-model` (177 IDs, 86 diagnostics),
`validate-spec-links` (177 rows), exact compiler-owner scheduling regression,
`cargo clippy -p repo-conformance --all-targets -- -D warnings`, scoped Rust
formatting, and `git diff --check`.
