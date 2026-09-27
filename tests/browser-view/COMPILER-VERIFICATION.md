# Compiler cache integrity

2026-09-27, pinned development container:

- Five new exporter adversaries failed against the inherited cache: planted
  executable, linked cache, changed source, aliased source, existing destination.
  The planted executables actually ran; existence was incorrectly authoritative.
- The added Cargo adversary proves an unchanged-source build can return success
  while retaining an executable replaced after compilation. A new private target
  rebuilds the actual program and refuses adoption of the poisoned target.
- All thirteen cache tests pass, including the four original retirement checks,
  hard-linked source refusal and aliased private-driver parent refusal.
  `scripts/compiler-driver-cache.test.mjs` builds real pinned exporter and Cargo
  fixtures; negative executables are test data, never acceptance compilers.
- All ten affected browser drivers now create private targets. Exporters compile
  captured, hash-verified source privately with the verified SDK toolchain.
- The first fresh DK-28 integration rejected a missing Cargo cache tag: creating
  an empty target before Cargo suppressed its initialization. The owning Cargo
  adversary reproduced this failure. Targets now must be absent beneath an owned
  private parent; Cargo creates and tags them itself. No cache tag is fabricated.
- All exporters now remain within owning fixture work. Retained fixtures retire
  completed tools after their last invocation; workspace/envelope tests already
  remove their exact owned work. The six added retirement owners preserve source,
  proof, IR and Wasm evidence, and record original tool hashes. Repository
  manifests are read from closed registered paths, never deleted or modified.
  Incomplete and source-only diagnostic builds remain retained, not accepted.

Complete affected browser owners, combined installed-SDK execution and full
release gates remain required. Cache-integrity tests do not establish application
or deployment acceptance. Original logs: `target/audit-27sep26/`.
