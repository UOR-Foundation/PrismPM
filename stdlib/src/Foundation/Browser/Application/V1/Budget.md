# Private effect budgets

`Budget.lex.tex` supplements DK-18 requests with concrete per-resource byte
limits. It does not issue grants, authenticate a policy or enable public builds.

An `EffectBudget` binds 32-byte application, manifest, requested-policy and
effective-policy references. Its 1–64 resource rows exactly match every grant
in the independently admitted manifest, in strictly increasing resource order.
Missing, extra, duplicate and reordered rows reject. Random maxima are positive
and at most 65,536 bytes; Digest, Sign and Verify maxima are positive and at most
1,048,576 bytes. Guest and Store rows must equal their actual manifest input and
object-byte maxima; their remaining limits stay owned by that manifest.

`effectiveRequestFits` checks the full manifest, both externally bound policy
references, the exact 32-byte execution session and the complete DK-18 request.
It then checks the selected resource's concrete limit against the actual request:
random count, guest input, digest/signature payload, verification payload or every
committed object. Reads retain the manifest's existing object/head checks.
It does not infer a successful effect or bypass counter/queue admission.

The generated application wrapper must derive these rows from its verified
source declaration and independently admitted effective grants. The requested
and effective identities must bind those actual values; matching digest labels
alone prove nothing. Signing must also satisfy DK-25's exact custody-slot,
public-key, context and payload policy. This predicate supplements, never
replaces, those obligations or DK-24/DK-26 durable contextual admission.

The source wrapper must check the exact derived request before durable prepare
and one-shot effect release, and reconstruct the same check on authenticated
replay. Public callers must not supply a budget, manifest, policy reference,
execution context or an admission boolean. No production wire ABI or host
permission API is introduced here.
