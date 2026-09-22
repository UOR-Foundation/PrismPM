# Architecture Decision Records (ADRs) Index

This directory contains Architecture Decision Records for PrismPM, documenting significant architectural decisions, contradictions resolved, and policy choices.

## ADR List

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](001-release-acceptance-definition.md) | Release Acceptance Definition for PrismPM v0.3.0 | Accepted | 2026-09-22 |
| [002](002-crates-io-bootstrap.md) | Crates.io Bootstrap Strategy for First-Party Crates | Proposed | 2026-09-22 |
| [003](003-independent-oracle-scope.md) | Independent Oracle Verification Scope | Accepted | 2026-09-22 |
| [004](004-gate15-package-api-criteria.md) | Gate 15 Package-API Pass Criteria | Accepted | 2026-09-22 |
| [005](005-reproducibility-gate-criteria.md) | Reproducibility Gate Criteria and CI Failures | Accepted | 2026-09-22 |
| [006](006-lean4-prod-upstream.md) | Lean4-Prod Upstream Dependency Management | Proposed | 2026-09-22 |
| [007](007-foundry-integration.md) | Foundry Portal Integration as Post-SDK Milestone | Accepted | 2026-09-22 |
| [008](008-diagnostic-coverage-tiers.md) | Diagnostic Boundary Coverage Completeness | Accepted | 2026-09-22 |

## Summary of Resolved Dissonance

### Contradictions Addressed
1. **"Release complete" vs. "blocked on crates.io"** → ADR-001: Phased release definition
2. **"Gate 15 passes" vs. "package-api fails"** → ADR-004: Split gate criteria
3. **"Independent oracle" vs. "local binary"** → ADR-003: Explicit oracle scope
4. **"All gates passed twice" vs. "CI failures fixed post-hoc"** → ADR-005: Environment vs. implementation classification

### Hidden Assumptions Made Explicit
1. **OIDC publishing works for first upload** → ADR-002: Manual bootstrap required
2. **Lean4-prod upstream will merge** → ADR-006: Vendored artifacts are release artifacts
3. **Foundry integration == SDK release** → ADR-007: Separated milestones
4. **Diagnostic probe count == boundary coverage** → ADR-008: Tiered coverage model

### Gaps Documented with Paths Forward
- **crates.io publication**: ADR-002 provides bootstrap protocol
- **Lean4-prod upstream**: ADR-006 defines vendoring policy and fork contingency
- **Foundry portal**: ADR-007 defers to Phase 2 with explicit criteria
- **Diagnostic coverage**: ADR-008 classifies tiers and sets Phase 2 target

## Usage
- All ADRs are immutable once Accepted
- Proposed ADRs require team review before Acceptance
- Reference ADR numbers in commit messages, PRs, and release notes
- Update this index when adding new ADRs