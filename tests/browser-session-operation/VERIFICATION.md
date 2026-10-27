# DK-35 private operation capture

The complete registered gate passed on 28 September 2026: 90 mandatory checks,
no skipped checks, 3,626.66 seconds including the unchanged payload owner.
DK-35 is a prerequisite of the separate full DK-30 session host, not its completion.

`session-operation-capture.mjs` captures one exact operation and derives its
predecessor, successor, observations and content descriptors through generated
LexLean kernels. Factory-owned handles expose defensive copies and are revoked
on close. Source errors never produce handles.

The predecessor is not authenticated or current. Caller digest agreement is
not artifact authority. This component neither signs journal records nor
releases effects, accepts a public application or bypasses `PP2011`. SDK
assembly must supply independently accepted artifacts and bind the result to
authenticated retained history, freshness, succession and durable publication.

Run in the pinned devcontainer:

```sh
node --test tests/browser-session-operation/boundary.test.mjs tests/browser-view/kernel-provenance.test.mjs tests/browser-view/local-module-inputs.test.mjs
node --test tests/browser-session-operation/owner.test.mjs
```

The owner requires fresh source/kernel verification; kernel-manifest/Lean byte
linkage; complete generated-package and original/private native/Wasm custody;
five independent component corpora; complete operation replay in three pinned
browsers; factory/lifecycle negatives; all 27 existing combined domain maxima
per browser; real source/host substitutions; and the unchanged full payload
owner. Both original and private native artifacts are checked before and after
every execution, including actual during-execution replacement negatives.
Phase records remain explicitly incomplete; only the complete owner emits its
component receipt. That receipt is not complete DK-30 or deployment evidence.

Native browser observations retain their global order and intentional duplicate
calls. The private transcript runner executes each row twice in both native
modes. A 4 MiB/4,096-row flush policy bounds batching; larger legal frames retain
the binary path. Immutable transcript bytes/identity and original/private ELF
custody are checked around every invocation. Altered, missing, reordered and
substituted transcripts must fail real generated execution or its owning guard.

The 28 September per-call replay run was intentionally cancelled for this
optimization, not accepted or reported as a semantic failure. Its unchanged
payload prerequisite, component checks, Chromium corpus/boundaries and all 27
Chromium maxima passed; Firefox/WebKit and mutants remained incomplete. Exact
863 inputs and nine phase receipts are retained in the diagnostic container at
`/tmp/operation-optimization-cancellation-z4fqap`. The subsequent complete run
retained the original deadline and every case; no cancelled-run verdict was reused.

## Complete component evidence

Container `prismpm-session-operation-dk35`, directory
`/tmp/prismpm-session-operation-GMePT9`, receipt `operation-capture-evidence.json`:

- Receipt SHA-256: `eedc9ff4bf3dac7bd78406a4265ed37b2f76144254b8a06f2108495418368dfe`.
- Kernel attestation: `38d143a10572b38dc750ccf847603cb1ccb607674beefb705430b23a4b53f688`.
- Exact input inventory: 865 files; all 42 phase records independently read back.
- Each browser: 895 cases, 8,753 generated calls, six boundary journeys and all 27 maxima.
- All nine host mutations and eight freshly compiled source mutations detected.
- Original/private native during-use substitutions and missing-post-guard counter-mutations passed.

Chromium corpus replay fell from 716.21 to 49.01 seconds. Independent comparison
confirmed identical ordered request/response hashes, lengths and Wasm memory for
all 8,753 calls. Full signed history, freshness, succession, host integration,
SDK release acceptance and Foundry deployment remain separate requirements.
