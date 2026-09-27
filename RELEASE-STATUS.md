# Production release status

**Not accepted — 27 September 2026.** Earlier statements that all six release
steps, public Cargo uploads and production SDK acceptance were complete are
withdrawn. Contract fixtures, named receipt schemas and component passes do not
prove those external events or the complete application.

The normative requirements remain in [SPEC.md](SPEC.md), the generated
[conformance inventory](CONFORMANCE.md), and the exact-source release gates.
Historical component evidence remains in [VERIFICATION.md](VERIFICATION.md).
This correction removes no required capability, test or release condition.

## Current boundaries

| Boundary | Status |
| --- | --- |
| Complete source gate | PR #48's first `just vv` passed all 15 gates for its tested merge revision. Its second full invocation was cancelled at GitHub's six-hour job limit; the PR is not accepted. |
| Immutable SDK | Current combined, installed and dual-platform release acceptance remains required. A development candidate is not a production SDK. |
| `Browser` application profile | `PP2011` remains closed. Pure session, presentation, crypto and storage components do not implement authenticated complete application execution/recovery. |
| Foundry | Reachable modeled services, mailbox enrollment/recovery, peer interoperability, coherent design and full user-journey acceptance remain required. |
| Publication | Producer acceptance, authenticated first-publication handoff, unchanged source-free export and exact deployed-byte/live-journey verification remain required. |
| Cargo ecosystem | The owner has deferred first-party crates.io publication until Foundry is published and verified. No bootstrap fixture authorizes an upload or establishes registry ownership. |

The source run is
[36331431587](https://github.com/UOR-Foundation/PrismPM/actions/runs/36331431587),
tested merge revision `08861e3caf1db0c8c9332d750b513506dec4b8e8`.
Its first invocation exited zero; the second did not finish. This is not a
two-pass receipt, branch-head receipt, SDK release or Foundry acceptance.

## Authority and implementation

Application behavior is authored in LexLean/PrismPM. No handwritten publisher
application, copied standard-library namespace, placeholder binary or caller
success Boolean may replace a missing implementation.

[`hologram-live`](https://github.com/Hologram-Technologies/hologram-live)
is the `.holo` authority. `holo/1` is the Prism profile; the upstream physical
archive version is distinct. Exact pinned archive/executor interoperability is
required. Independent Hologram validation does not make Hologram a deployed
Foundry service or require publishing its crates.

OSCAL schema validation establishes structural properties, not implementation
of controls. Imported standards require their actual applicable oracles and
criterion-level evidence. Automated accessibility checks do not replace required
complete-process and human assessment.

Generic compiler corrections must pass upstream verification/review and be
repinned reproducibly. [lean4-prod PR #73](https://github.com/auser/lean4-prod/pull/73)
and [PR #75](https://github.com/auser/lean4-prod/pull/75) have passed their full
hosted checks and independent review; maintainer merge and downstream adoption
remain required. Local source patches are not accepted compiler dependencies.

An accepted offline OCI SDK can precede Cargo publication, but it must still
pass its complete source, artifact, security, platform and installed-execution
gates. Only actual subject-bound release and deployment evidence closes the
remaining ecosystem requirements.
