# PrismPM (Prism Platform Model)

## Overview
PrismPM is a first-release framework for modeling software and systems architectures with a fixed architecture, application-security/risk, and product-quality profile. It provides kernel-checked generated declarations and bounded executable-validator evidence for the claims registered in this repository.

## Core Architecture & Verification Pipeline

```text
Prism .lex.tex + prism.* lexicons (Language 1.1)
        │
        ▼
LexLean 1.1 linked IR ───► stable semantic snapshot ───► Prism Holo emitter
        │                                                        │
        ├────► canonical Lean + canonical LaTeX                  └────► model.holo
        │             │
        │             ▼
        │     LexLean verify + leanchecker + axiom audit
        │             │
        │             ▼
        └────► named-definition LCNF export with lean4-prod (Lean 4.32.1)
                      │
                      ▼
               generated Rust validators over normalized Holo data
```

### 1. Generated-Lean-Only Boundary
Every PrismPM-owned type, validator, theorem, and proof is authored in `.lex.tex` and lowered by LexLean. PrismPM contains **no handwritten `.lean` source** or `lakefile.lean`. Generated Lean exists only inside content-addressed build/verification outputs or provenance-checked goldens.

### 2. Lexicon Facets & Standards Profile
PrismPM organizes domain primitives into versioned LexLean 1.1 lexicon packages containing repository-authored interpretations of concepts from the fixed standards profile:
- **`prism.arch`**: Architecture-description concepts from **ISO/IEC/IEEE 42010:2022**.
- **`prism.sec`**: Application-security and risk-management concepts from **ISO/IEC 27034-1:2011/Cor 1:2014**, **ISO/IEC 27034-5:2017**, and **ISO/IEC 27005:2022**.
- **`prism.qual`**: Product-quality characteristics and requirements from **ISO/IEC 25010:2023**.

PrismPM validates its own formal profile. It does not reproduce standards text, certify an organization or product, or grant ISO conformance or certification.

### 3. Holo Format (`.holo`) as PrismIR
The Holo format (`prismpm/holo/1`) is one UTF-8 canonical JSON value emitted by the Prism projector from stable LexLean semantic snapshots; it is not an archive container. Entity identifiers are normalized into deterministic zero-based numeric indexes.

### 4. Verified Execution via `lean4-prod`
`lean4-prod` extracts compilable Lean Compiler Normal Form (LCNF) from verified Lean modules. `prod-codegen` produces `no_std` Rust validators that allocate no heap memory during validator calls and execute over the normalized Holo representation. Deterministic property tests cover recorded finite strategies and bounds.

## Model-View-Controller Implementation
- **Model**: `Prism-stdlib` and facet definitions authored in `.lex.tex`.
- **View**: Canonical `.holo` values and canonical LaTeX documentation.
- **Controller**: The `prismpm` Rust library and CLI orchestrating LexLean and artifact generation.

## Verification & Acceptance Gates
- `just vv` runs the 14-stage non-mutating acceptance gate in strict normative order.
- Oracles include Lean 4.32.1 elaboration, `leanchecker` same-kernel replay, `#print axioms` auditing (verifying empty observed axiom sets), and `lean4-prod` execution property testing.
