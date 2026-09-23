# ADR-003: Independent Oracle Verification Scope

## Status
Accepted

## Context
README.md and RELEASE-STATUS.md claim "Independent Hologram Oracle" verification as a completed acceptance task. However:
- RELEASE-STATUS.md:23-30: "Pinned Hologram implementations remain isolated validation oracles, acquired before offline verification... Neither oracle is the application authority or a deployed Foundry service. Publishing Hologram is not a PrismPM dependency-resolution step."
- SPEC.md:182-193: Application verification "independently rechecks... the pinned Hologram Live source oracle. The oracle inspects and plans the archive, asserts the exact headless portable-surface blocker, opens a session with a display-independent portable surface, invokes every vector directly and through `application.invoke`, and checks reverse detach, stale-handler rejection, and idempotent shutdown."

The oracle is "independent" only in the sense of being a separate binary executed in a sandbox — not independent in deployment, operation, or authority.

## Decision
**Redefine "independent oracle" verification scope with explicit boundaries:**

### What "Independent Oracle" Means in PrismPM v0.3.0
1. **Binary independence**: Oracle is a separate executable (Hologram Live) not built from PrismPM source
2. **Execution isolation**: Runs in network-denied, capability-free sandbox with read-only inputs
3. **Input fidelity**: Validates exact `.holo` archive bytes produced by PrismPM
4. **Behavioral verification**: Executes modeled request/response vectors against Core-Wasm runtime
5. **Surface validation**: Exercises portable View (HOLOVIEW) through headless Chromium

### What It Does NOT Mean
- ❌ Deployed service with uptime/SLA guarantees
- ❌ Independent security audit or certification
- ❌ Separate organizational authority (same UOR-Foundation control)
- ❌ Production deployment validation
- ❌ Substitute for Foundry portal integration

### Required Documentation Updates
- Replace "Independent Hologram Oracle" with **"Isolated Hologram Oracle Execution"** in all acceptance tables
- Add oracle scope boundary diagram to VERIFICATION.md
- Document oracle version pinning in `model/dependencies.toml` (currently `2bda6a9a9476872dade705bd61ece4209607f6da`)

## Consequences
- **Positive**: Eliminates misleading "independent" claim; sets accurate expectations for consumers
- **Negative**: Reduces perceived verification strength; must compensate with stronger SDK/reproducibility evidence
- **Action**: Update README.md acceptance table; add oracle scope section to VERIFICATION.md

## References
- SPEC.md §8 (Verification and attestation)
- RELEASE-STATUS.md § "Modeled codec and independent oracles"
- VERIFICATION.md § "Original full-gate falsification campaign" Gate 7