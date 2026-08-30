# Changelog

All notable changes to PrismPM will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-30

### Added
- Initial UOR-Foundation workspace layout and devcontainer.
- LexLean-authored Prism facet lexicons (`prism.arch`, `prism.sec`, `prism.qual`).
- Formal metamodel in `.lex.tex` defining ISO 42010, ISO 27034, ISO 27005, and ISO 25010 primitives.
- Canonical JSON Holo format (`prismpm/holo/1`) emitter and validator.
- Controller API, CLI commands (`check`, `build`, `verify`), and `vv` acceptance gates.

### Compatibility

- The `/1` project, Holo, build, verification, and evidence schemas are the
  initial stable schemas. An incompatible schema change requires a new schema
  identifier and a SemVer-major PrismPM release; compatible additive behavior
  may not weaken closed-object decoding or existing verification guarantees.
