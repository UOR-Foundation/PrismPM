# ADR-005: Reproducibility Gate Criteria and CI Failures

## Status
Accepted

## Context
VERIFICATION.md documents multiple CI failures that required post-hoc fixes before "reproducibility" gates could pass:
- **Driver identity isolation** (VERIFICATION.md:455-512): First `just vv` failed at Gate 8 (`PP5001: LexLean verification failed`) because the running process's `/proc/<pid>/exe` link acquired `(deleted)` when nested cargo commands replaced the executable. Fix required isolating driver target directory.
- **Devcontainer Docker group readiness** (VERIFICATION.md:153-176): GitHub remapped devcontainer UID/GID (no longer group 1000), causing stale shell failures. Fix required lifecycle commands to wait for socket group and refresh credentials.
- **Bootstrap source history** (VERIFICATION.md:514-542): Shallow Git checkout in CI broke `git archive` for bootstrap source pin. Fix required full history checkout.
- **SDK registry transport** (VERIFICATION.md:422-454): Docker Engine 28.0.4 vs 29.1.3 media type incompatibility caused `MANIFEST_INVALID`. Fix required Zot `http.compat = ["docker2s2"]`.

RELEASE-STATUS.md:173 claims "Passed every PrismPM gate twice without cleanup" but these failures occurred *during* the gate runs that supposedly passed.

## Decision
**Distinguish between "gate implementation correctness" and "environment reproducibility":**

### Reproducibility Gate Scope (What Must Pass)
1. **Two-root byte reproduction**: Building in two fresh absolute directories produces byte-identical artifacts (Gate 12)
2. **Source-root independence**: No absolute paths in generated artifacts (Gate 12)
3. **Deterministic build outputs**: Same inputs → same outputs across runs (Gates 1, 10, 12)
4. **Lockfile determinism**: `prismpm.lock` and `standards.lock` produce identical dependency graphs

### Environment Reproducibility (Separate Concern)
- Devcontainer UID/GID mapping consistency
- Docker Engine version compatibility
- Git checkout depth in CI
- Host kernel /proc filesystem behavior

### Required Process Changes
1. **Pre-gate environment validation**: Add `scripts/verify-environment.sh` that checks:
   - Docker Engine version ≥ 29.1.3 (or Zot compat configured)
   - Git history depth = full (not shallow)
   - Devcontainer UID/GID matches expected (1000:1000 or documented remap)
   - Host /proc executable stability (no nested cargo in same target dir)

2. **Gate failure classification**: Each gate failure must be categorized as:
   - **Implementation defect** (code bug) — blocks release
   - **Environment defect** (CI/config) — blocks gate until fixed, but not a release blocker once fixed
   - **Flaky/transient** — document and monitor

3. **Documentation**: Update RELEASE-STATUS.md to specify which gates passed with environment fixes vs. clean

## Consequences
- **Positive**: Clear separation of code vs. environment issues; prevents conflating CI fixes with implementation maturity
- **Negative**: More complex gate reporting; requires maintaining environment validation script
- **Action**: 
  1. Create `scripts/verify-environment.sh`
  2. Update `cargo xtask vv` to run environment validation as Gate 0
  3. Annotate RELEASE-STATUS.md gate history with fix classifications

## References
- VERIFICATION.md § "Full-gate driver identity isolation", "Devcontainer Docker group readiness", "CI bootstrap source history", "SDK bootstrap registry transport compatibility"
- RELEASE-STATUS.md § "Reproducible multi-platform SDK"
- SPEC.md §12 (SDK and lock contract)