# Agent Guidelines for PrismPM

- SPEC.md is normative; README prose is explanatory.
- No handwritten PrismPM `.lean` source or `lakefile.lean` may be committed. All PrismPM Lean code is generated from `.lex.tex` by LexLean. The sole upstream-source exception is `vendor/lexlean/language/lcnf-1.2/extract.lean`, admitted only through the registered LexLean package, complete dependency/tree checksums, and matching package VCS provenance. It is dependency-owned authority plumbing, never a PrismPM model or proof.
- Do not introduce placeholder URLs, wildcards in internal dependencies, unpinned toolchains, or unstaged modifications.
- Every public capability and diagnostic is registered in `model/*.toml` and covered by conformance fixtures.

