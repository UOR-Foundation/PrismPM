# Same-run source-CI input contract

Draft, unused by workflows. No provider, OCI, SDK, gate, or performance acceptance.

`sdk-source-ci-contract.mjs` is a separate metadata-only join. It does not extend
the dual-native `workflow_dispatch` authority in `sdk-construction-handoff.mjs`.
The exact current run/attempt may be queued or in progress while its selected producer,
ordered construction/upload steps, and original artifact are terminal-successful.
The selected PR event head, synthetic checkout, base, tree, ordered parents, and
workflow revision are distinct fields. Inputs bind the complete original
`buildImageWithInputs` policy/manifest, source standards, and original OCI
manifest/config/archive identities. JSON fixtures confer no authentication.

The fixed active budget is producer job start-to-completion plus consumer job
start-to-completion, including setup, original construction, upload, acquisition,
verification, import, owning execution, and retirement. Provider admission/queue
wait is recorded separately: GitHub's original job timeout excludes that wait.
Four seconds conservatively covers timestamp resolution. The original five
180/120/120/180/90-minute budgets are not parameters. A final metadata accounting
check is not live child-start authorization or proof of actual test execution.

`sdk-source-ci-archive-loan.mjs` checks actual private sealed regular-file bytes,
single-link ownership, read-only held descriptors, native9, and full digest before
and after an exclusive data loan. It never deletes caller files. A timed-out
unsettled borrower quarantines its descriptor; a settled Promise is not evidence
of child retirement. The enclosing process owner must establish that separately.
No normalized Docker archive, writable descriptor, or second tar is produced.

Before wiring executable reuse, the following owners remain mandatory:

- Authenticated bounded provider acquisition and before/after closure: exact
  current run/attempt, repository IDs, source/workflow Git objects/tree/parent
  proofs, source-bound workflow profile, job/step/upload and artifact joins.
  No caller selection, `needs` output, JSON handle, or signature flag is authority.
- Independently verify the complete selected source/advisory/bootstrap closure;
  compare the producer's actual retained buffers, not a hand-authored build result.
- Verify the entire closed stored ZIP/ZIP64, CRC, lengths, digest, and original OCI
  DAG, then stage one sealed tar under the original byte-plus-12GiB admission.
  The DATA loan is not that verifier and cannot mint its integrity handle.
- Own a fresh consumer root/nonce/registry/CA, original ORAS1.3 import and immutable
  distribution/config readback, credential removal, actual child lifecycle, live
  active-budget enforcement, primary-plus-cleanup failure evidence, and retirement.
- Preserve all twelve existing names/commands/timeouts, seven source-only owners,
  max3 fanout, original full VV and dual-native release qualification. Producer
  failure must not skip the seven source-only owners or pass the five SDK owners.

These are remaining implementation obligations, not deferred acceptance or issue187
closure. No workflow or existing authority is changed by this first source draft.
