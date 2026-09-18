# Workspace command prerequisite

Pure modeled preparation is not authorization, cryptographic verification,
persistence, peer acceptance, or a Foundry application. Journal remains the
sole role/state reducer after actual authentication.

WC-01: generated command construction and correlated hash/sign completion.
WC-02: complete native/no_std/Core-Wasm byte parity and resource bounds.
WC-03: actual browser crypto, authenticated Journal append/replay and adversarial
completion/stale/identity/signature journeys; no caller state or receipt API.

Actions retain Workspace codes: genesis0, contributor1, reader2, revoke3, post4.
Prepare derives event version, parent and sequence from a fully validated head.
Body limits/UTF-8 are unchanged. No duplicate role policy is added here.

Closed ABI (big-endian lengths):
- op0: nonce32 | publicKey65 | principal32 | workspace32 | headLen:u24 | head |
  action:u8 | bodyLen:u16 | body.
- Pending: PWC01 magic(50 57 43 01) | phase:u8 | eventId32 | exact op0 payload.
  Phase0 requires zero eventId; phase1/2 require nonzero eventId.
- op1/2: pendingLen:u24 | pending | currentHeadLen:u24 | currentHead |
  nonce32 | actualPublicKey65 | effectInputLen:u16 | effectInput |
  effectResultLen:u16 | effectResult.
- Success: 00 | nextPendingLen:u24 | nextPending | effectLen:u16 | effect.
  Prepare emits full domain-separated hash preimage. Hash completion emits
  unsigned event bytes for signBytes(context=prismpm/workspace-event/1).
  Signature completion emits the canonical complete PWE01 envelope.
- Error: one typed code, no partial next pending or effect.

The private host supplies current replay-derived head and selected persisted
identity, owns pending/correlation, runs actual SHA-256/P-256 operations, and
consumes phases once. Model completion checks exact key/nonce/input/head binding,
not the truth of supplied hash/signature bytes. Final actual Journal append and
replay must reject changed crypto bytes. Raw callers cannot supply pending,
state, authenticated flags or completion receipts through that host boundary.
Terminal reuse, current-head changes, malformed/trailing data and overlimits
are rejected; no automatic retry or branch selection.

Closed syntax caps: prepare payload69,837; pending69,874; head65,574;
unsigned event4,198; signing preimage4,253; envelope4,363 bytes.
The signing prefix is28-byte domain +2-byte context length +25-byte context.
The request cap139,873 includes operation1 + pending length3 + pending69,874
+ head length3 + head65,574 + nonce32 + key65 + input length2 + input4,253
+ result length2 + result64. Output cap74,243 includes status1 + pending
length3 + pending69,874 + effect length2 + envelope4,363.
These are closed framing limits, not assertions that every maximum field can
be valid simultaneously. An admitted preparation has at most1,023 head events:
largest legal Hash request139,713; largest terminal response74,179.

Error codes:01 BadEncoding;02 InvalidCommand;03 InvalidHead;
04 WorkspaceMismatch;05 EventLimit;06 WrongPhase;07 CorrelationMismatch;
08 StaleHead;09 EffectMismatch;0a InvalidResult;0b UnknownOperation;
0c InvalidPending. Shape is checked before content, pending validity before
phase, then exact current head, key/nonce, generated input and result shape.

WC-02 includes75 literal vectors, unchanged Workspace/Envelope/Journal model
bytes, all typed failures, all five actions through every completion phase,
the full1,024-event rejection and maximum UTF-8 body at1,023 head events.

The command-only hard budget is64 pages (4 MiB), checked by actual guest
allocation/growth traps and complete
native/no_std/Wasm/Chromium replay. Measurement is finite corpus evidence,
not a universal memory proof. Workspace512 and Journal640 remain unchanged.
Internal binary plans are not Hologram's legacy 64-KiB portable intent transport.

## Private command adapter

Local SDK prerequisite only: no Foundation authority, Kappa, replication, or
read-admission claim. The generated Command constructs all event/envelope bytes;
the generated authenticated Journal owns admission and state transitions.

Trusted bootstrap supplies verified Command/Journal modules and actual storage.
The adapter loads, validates, and privately captures the persisted identity.
Public methods: submit({action,body,workspace}), refresh(), status(), close().
Status and refresh return only frozen principal/refresh-barrier metadata;
successful submit adds only committed:true. No raw head, state, event rows or
Journal object escapes through this API. All application reads use the separately
admitted Query boundary. This closes a local API bypass, not storage/network
privacy or erasure; trusted bootstrap and test observers retain storage access.
Inputs have exactly three own data properties and are synchronously copied.
No actor, head, state, pending, authentication flag, or receipt is accepted.

Each instance serializes commands. Actual journal refresh after each async hash
and signature precedes the correlated generated completion. A durable competing
head therefore fails stale completion without retry or branch selection.
Storage/replay failure erects an explicit refresh barrier; no partial state is
promoted. Closing before append prevents the write. Closing during an already
started durable append cannot promise rollback: return commit-outcome-unknown,
then reopen/replay to learn the durable result. No late result can reopen a
closed adapter or automatically retry a failed command.

The explicit local host admission budget is two outstanding operations total:
one active and at most one queued; submit and refresh share the budget. Excess
calls reject `adapter-busy` before capturing inputs. Completion/error releases
capacity; close immediately rejects and releases queued entries. One already
started effect remains owned until it settles, without claiming cancellation.
The private Journal is never exposed and receives at most one adapter operation
at a time. The standalone SDK Journal has its separately registered budget.

Fresh Command instance per call: 139873 input, 74243 output, 64 pages. Existing
Journal640/Workspace512-page budgets and 75/61/43/45 vectors remain unchanged.

CA-01: closed API/private identity/actual generated bytes and crypto.
CA-02: cross-tab concurrency and stale effect boundaries.
CA-03: failed/uncertain effects, close, reopen, no automatic retry.
CA-04: complete native replay of actual Command/Journal transcripts.
