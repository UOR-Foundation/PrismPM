# PrismPM (Prism Platform Model)

## Overview
PrismPM is a comprehensive, multi-layered framework for formal modeling of software and systems architectures. It provides verifiable abstractions spanning from conceptual architecture descriptions to application security and quality models.

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
PrismPM organizes domain primitives into versioned LexLean 1.1 lexicon packages mapping directly to authoritative standards:
- **`prism.arch`**: Architecture description concepts conforming to **ISO/IEC/IEEE 42010:2022**.
- **`prism.sec`**: Application security and risk management conforming to **ISO/IEC 27034-1:2011/Cor 1:2014**, **ISO/IEC 27034-5:2017**, and **ISO/IEC 27005:2022**.
- **`prism.qual`**: Product quality characteristics and requirements conforming to **ISO/IEC 25010:2023**.

### 3. Hologram Format (`.holo`) as PrismIR
The Hologram archive format (`prismpm/holo/1`) is a UTF-8 canonical JSON artifact emitted by the Prism projector from stable LexLean semantic snapshots. Entity identifiers are normalized into deterministic zero-based numeric indexes.

### 4. Verified Execution via `lean4-prod`
`lean4-prod` extracts compilable Lean Compiler Normal Form (LCNF) from verified Lean modules. `prod-codegen` produces zero-cost, `no_std`, heap-allocation-free Rust validators that execute over the normalized Holo representation, verified by bounded property testing.

## Model-View-Controller Implementation
- **Model**: `Prism-stdlib` and facet definitions authored in `.lex.tex`.
- **View**: Canonical `.holo` archives and Canonical LaTeX documentation.
- **Controller**: The `prismpm` Rust library and CLI orchestrating LexLean and artifact generation.

## Verification & Acceptance Gates
- `just vv` runs the 14-stage non-mutating acceptance gate in strict normative order.
- Oracles include Lean 4.32.1 elaboration, `leanchecker` same-kernel replay, `#print axioms` auditing (verifying empty observed axiom sets), and `lean4-prod` execution property testing.
