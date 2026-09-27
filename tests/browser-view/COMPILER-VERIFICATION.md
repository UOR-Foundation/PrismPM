# Compiler cache integrity

2026-09-27, pinned development container:

- Five new exporter adversaries failed against the inherited cache: planted
  executable, linked cache, changed source, aliased source, existing destination.
  The planted executables actually ran; existence was incorrectly authoritative.
- The added Cargo adversary proves an unchanged-source build can return success
  while retaining an executable replaced after compilation. A new private target
  rebuilds the actual program and refuses adoption of the poisoned target.
- All twelve cache tests pass, including the four original retirement checks,
  hard-linked source refusal and aliased private-driver parent refusal.
  `scripts/compiler-driver-cache.test.mjs` builds real pinned exporter and Cargo
  fixtures; negative executables are test data, never acceptance compilers.
- All ten affected browser drivers now create private targets. Exporters compile
  captured, hash-verified source privately with the verified SDK toolchain.

Complete affected browser owners, combined installed-SDK execution and full
release gates remain required. Cache-integrity tests do not establish application
or deployment acceptance. Original logs: `target/audit-27sep26/`.
