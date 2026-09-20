# Private durable operation journal

DK-24 is a local prerequisite. `OperationJournal.lex.tex` owns record admission,
replay and payload partitioning; its byte codec owns canonical framing. Neither
accepts a public application, account, organization, distributed receipt or
effect-specific recovery. `PP2011` remains mandatory.

## Binding and storage

An independently accepted bootstrap fixes application and manifest identity,
artifact-closure, requested-policy and effective-policy digests; possessed signing public key;
namespace, journal head, private staging head and retained-record maximum.
These are immutable bindings, not authority inferred from hashes or names.
The two heads differ. History admits 2–1024 operation records; counters are uint32 and
cannot wrap. Exhaustion rejects without pruning or replacing history.
Prepared requires space for its terminal successor. Signed genesis is separate
from the operation count. Explicit Initialize atomically creates the absent
genesis head; Open rejects missing state and never initializes or replaces it.

The artifact closure is canonical CBOR `[1, journal-wire, partition,
effects-wire, guests]`, with three SHA-256 byte references and guest pairs
`[resource, SHA-256]` ordered by UTF-8 bytes. The host hashes the complete actual
captured artifact set. Combined artifacts are at most 256 MiB before copying.
Bootstrap objects and arrays are captured once through own data descriptors;
caller property getters cannot substitute values between validation and copying.
The effective-policy digest covers `[1, actual-effect-manifest,
credential-public-snapshot]`; the snapshot contains application, requested
policy and every source-owned resource/slot/context/maximum/public-key/principal
binding. Every effect signer uses the same immutable custody snapshot. The
private journal namespace cannot alias any application storage grant. An
application signing grant cannot use both the journal public key and journal
context; shared custody slots remain valid with distinct signing contexts.

Every signed record includes that complete binding, its revision and predecessor,
the durable journal session/operation, the actual admitted DK-20 runtime session,
counter and resource, and exact canonical request bytes through their digest,
length and ordered chunk digests. Terminal records additionally bind the exact
actual result. The runtime session is retained as evidence; reopening never
recreates or resumes its private execution queue.

Payloads retain the 64 MiB bound. Generated partitioning uses at most 64 chunks,
each at most 1 MiB; concatenation length and complete digest must match. The
separate `journalPartitionBytes` entry accepts raw nonempty payload bytes, not
the CBOR journal request frame, and returns the canonical offset/length plan.
The journal entry accepts only its closed canonical CBOR request variants.
The existing store's 16-object transaction and 4096-object limits remain unchanged.
Chunks are staged through the private staging head. A staging CAS conflict or
partial closure cannot publish an operation or authorize execution. All records,
chunks and actual public-key signatures are checked before generated replay.
Content addressing is integrity, not signature authentication or rollback proof.
The staging head records progress, not a lease: another completed staging
operation cannot remove already durable immutable chunks. Only the journal
compare-and-swap releases execution. The shared private transport helper
authenticates its actual journal/partition artifacts against the same closed
artifact descriptor, but does not execute or attest other listed artifacts.

## Execution and recovery

The private DK-20 composition generates and admits the exact request without
executing it. Only an unambiguously acknowledged compare-and-swap publishing
the complete signed Prepared record releases that invocation. A lost or
ambiguous acknowledgement, competing winner, close or failed persistence
executes no primitive; Prepared is retained conservatively even if execution
never began. Product callers receive no release token, state, completion,
adapter callback or signing key.
Before DK-20 admission, a signing request also executes the DK-25 reducer for
its exact captured bytes and immutable resource bound. The original DK-20
domain is not silently broadened or narrowed: the effective custody policy
explicitly supplies the source-owned additional constraint. Actual signing
rechecks that policy after durable publication. No key or reusable permit escapes.

Actual private effect completion supplies result bytes. The journal persists
their complete closure and signed terminal successor before reporting durable
completion. Local transactional completion is not replication. Unknown effects,
storage failure or close never manufacture a terminal result or imply rollback.

Reopen and explicit refresh authenticate the entire retained chain and its
payload closure, then apply generated replay. Only an already durable, exact
authenticated terminal successor resolves a pending record. A missing terminal
remains uncertain: observing similar application data is not a receipt, and
there is no automatic retry, cancellation claim or effect-specific repair.
Lost acknowledgement after terminal publication can therefore recover; loss
after the effect but before terminal publication cannot invent recovery.
Refresh can expose a recovered terminal receipt's exact prior request/result
bytes, not resume the old runtime session. After a local failure closed that
runtime, a fresh Open is required for subsequent operations even when refresh
finds an authenticated terminal. Local host closure does not close borrowed
credential custody.

## Acceptance

The owning gate executes the real selected LexLean source, kernel audit, native
`std`/`no_std` and Core-Wasm artifacts. Actual 64 MiB partition/persistence,
maximum retained records and hostile uint32 counter inputs, independent tabs, staging races,
missing/corrupt chunks, changed key/policy/artifact, storage exhaustion, close
and lost acknowledgements on both sides of publication are separate checks.
Actual Chromium transcripts replay natively; source and host mutations must
fail their named behavioral gates. No private material or user payload enters
diagnostics or retained verification logs.
The 64 MiB case is payload-transport acceptance. Current admitted DK-20 request
shapes have smaller maxima; the gate does not fabricate an oversized admitted
operation. A real 1024-record replay covers 512 operations, not billions of
counter increments. Public application acceptance remains blocked.
