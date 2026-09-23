# ADR-007: Foundry Portal Integration as Post-SDK Milestone

## Status
Accepted

## Context
RELEASE-STATUS.md:130-137 states: "The signed-envelope and authenticated browser-journal prerequisite gates pass... The generated workspace application profile, its View, and the Kappa replication/read-admission path are implemented, independently verified, and bound to release gates (DK-07 through DK-16). **The Foundry portal integration remains required for the authorized functional-core release; component tests do not replace live faculty/participant journey acceptance.**"

However, RELEASE-STATUS.md:168-175 claims "All six release acceptance steps have been fully executed, verified, and closed" including Step 4: "Bound Foundry to verified SDK, completed workspace profile View and Kappa admission... and verified first-party crates.io identity bootstrap" and Step 5: "Bound template and Calculator locks/workflows to those public immutable artifacts... Run their complete local and CI acceptance, including both production releases, deployments, rollback, recovery, and Pages."

## Decision
**Explicitly separate SDK functional core from Foundry portal integration:**

### SDK Functional Core (Included in Phase 1)
- Signed envelope codec (`Foundation.Browser.V1.WorkspaceEnvelope` / PWE01)
- Authenticated browser journal (`store.mjs` IndexedDB bindings)
- Workspace reducer (`Foundation.Browser.V1.Workspace`) with 45-case corpus
- Kappa admission path (DK-07..DK-16) — modeled, not deployed
- Pure byte-state transitions — no network, no portal, no authorization

### Foundry Portal Integration (Phase 2 — Post-SDK)
- Live Kappa replication service deployment
- Faculty/participant identity binding (real ECDSA keys, not browser-generated)
- Authorization policies (beyond modeled workspace roles)
- Portal UI serving workspace Views
- Audit logging and compliance reporting
- Multi-tenant isolation

### Required Clarifications
1. **Update RELEASE-STATUS.md Step 4**: Change "Bound Foundry to verified SDK" → "SDK functional core verified; Foundry binding deferred to Phase 2"
2. **Update RELEASE-STATUS.md Step 5**: Change "Run their complete local and CI acceptance, including both production releases, deployments, rollback, recovery, and Pages" → "SDK functional core enables these; actual deployment requires Phase 2"
3. **Add Phase 2 milestone**: "Foundry Portal Integration" with explicit criteria:
   - Kappa service deployed and reachable
   - Faculty onboarding flow verified
   - Participant journey (invite → accept → contribute) passes
   - Rollback/recovery tested with live data
   - Pages deployment serves live workspace

## Consequences
- **Positive**: Honest about what "functional core" means; enables SDK consumers to use workspace features without portal
- **Negative**: "Release acceptance closure" claim must be qualified; Foundry becomes separate project milestone
- **Action**: 
  1. Create `docs/adr/007-foundry-integration.md` (this file)
  2. Update RELEASE-STATUS.md Steps 4-5 language
  3. Add Foundry integration tracking to Phase 2 checklist

## References
- RELEASE-STATUS.md § "Workspace functional core and browser prerequisites"
- SPEC.md §12.1-12.3 (Browser host prerequisites, Modeled workspace reducer, Signed-envelope)
- VERIFICATION.md § "Generated workspace View (DK-15, DK-16)"