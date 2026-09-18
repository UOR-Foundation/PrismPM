# Workspace journal

The journal owns bounded append, replay and commit-completion decisions.
It is not a replication protocol, read-admission policy, Kappa implementation
or Foundation identity model. Workspace roles and limits remain unchanged.

Event ID is SHA-256 of the generated unsigned-event signing preimage;
object ID is SHA-256 of the complete signed envelope. The head is
`50 57 4a 01 | workspace32 | countBE16 | rows(eventId32, objectId32)`.
Counts are 1–1,024; workspace and digests are nonzero; both columns are
independently unique. Empty bytes mean no journal. Maximum head size is
65,574 bytes; events and heads each fit the storage primitive's 1 MiB cap.

The pure `workspaceJournalBytes` operations are:

| Byte | Payload | Success |
|---|---|---|
| 0 | head | exact validated head |
| 1 | headLen24, stateLen24, head, state, object32, envelope | append plan |
| 2 | session129, receipt161 | bound terminal completion |
| 3 | targetLen24, target, operation-1 payload | replay plan |
| 4 | targetLen24, headLen24, stateLen24, target, head, state | complete state |
| 5 | expected32, next32, object32 | commit-intent preimage |
| 6 | Envelope operation, envelope | generated projection |

Success is `00 | payload`; plan payload is `headLen24 | stateLen24 | head |
state`. Lengths are unsigned big-endian. Errors: 01 malformed, 02 head,
03 state binding, 04 object ID, 05 duplicate object, 06 event limit,
07 replay step, 08 incomplete replay, 09 completion binding, 0a terminal
session, 0b unknown operation. `10 | WorkspaceError` preserves reducer errors.
Storage completion statuses 21–25 mean conflict, limit, quota, closed and
unavailable, followed by terminal phase 02. Success uses phase 01.

Session is `phase8 | attempt32 | expected32 | next32 | intent32`; receipt is
`status8 | same128bindings | actualHead32`. Zero expected means absence.
Intent hashes `prismpm/journal-commit/1\0 | expected32 | next32 | object32`.
Matching bytes alone do not authenticate storage or event signatures.

Only trusted SDK bootstrap supplies the verified module and actual store.
The private adapter exposes append(envelope), refresh() and copied snapshots;
it accepts no caller state, authentication flag, receipt or completion token.
State is derived from complete authenticated genesis replay. All event
author/signature/preimage bindings are checked before modeled transitions.
Append captures bytes synchronously, serializes local operations, commits
exact immutable objects/head with real CAS, and promotes only on bound
generated completion. Failure or uncertainty requires replay, never automatic
retry, merge or branch selection. Unknown storage exceptions become
`storage-outcome-unknown`, without retained message, payload or cause.
Known host codes retain their documented BrowserEffectError namespace.

Journal guest bounds are 1,235,980 input bytes, 1,166,008 output bytes and
640 pages (40 MiB). Every command needs a fresh guest; compiled modules may
be cached. The underlying Workspace guest retains its 512-page contract.
Native input rejection and guest allocation traps are distinct tests.

DK-12 owns all 61 vectors twice across generated native/no_std/Wasm, both
full 1,024-event Grant/Post histories, generated output-head closure and
real crypto/storage fault/concurrency tests with native transcript replay.
It includes zero-workspace rejection and signature/capture/CAS/transcript
mutants. Private-guard negative tests are distinct from public journeys.
These finite tests do not prove every possible trace or internet availability.
Replication still requires an explicit verified transport budget or modeled
fragmentation: the maximum head exceeds the current peer default by 38 bytes.
