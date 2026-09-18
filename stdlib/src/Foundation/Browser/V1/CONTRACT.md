# Workspace reducer v1

`Workspace.lex.tex` owns the pure state transition and byte codec.
`WorkspaceCorpus.lex.tex` owns fixed request/response acceptance vectors.
This primitive is not an authenticating browser application or a Kappa network.

## Trust boundary

Only an authenticating adapter may supply `AuthenticatedEvent`. There is no
caller-asserted signature flag. The adapter must verify P-256 key possession,
bind `author` to SHA-256 of the raw 65-byte public key, verify the signature
over the complete domain-separated preimage, and bind `eventId` to SHA-256 of
that preimage. The signing context is `prismpm/workspace-event/1` under the
browser signature v1 host contract. `encodeUnsignedEvent` omits the `eventId`
field; `workspaceSigningPreimage` prepends the exact host prefix, NUL,
big-endian 16-bit context length, and context bytes. `eventId` hashes that
complete preimage, not the event encoding containing its own ID.
These hashes are not Kappa addresses or Foundation identity credentials.

State is obtained by replaying authenticated events from genesis. Received
snapshots are not trusted state. Structural state validation does not prove
signatures or establish a historical membership grant. Persisted state must
remain bound to its verified event history. The adapter must atomically compare
and replace the prior head; rejection never authorizes a write.

## Transition rules

Genesis establishes one immutable workspace owner. Only that owner grants or
revokes membership; an existing grant must be revoked before changing its role.
Owners and contributors may post; readers may not. These are workspace roles,
not Foundation appointments. Every accepted event extends the exact parent,
advances sequence once, and has an unseen, nonzero 32-byte event ID. Concurrent
branches are rejected as stale, not silently merged.

Engineering limits: 64 members including the owner; 256 messages; 1,024 events
including genesis; 1–4,096 UTF-8 bytes per message. Exhaustion rejects without
dropping history. Members are ordered 33-byte rows (`principal`, role byte
`01` contributor or `02` reader); the owner is implicit. Seen IDs are ordered
32-byte rows. Message rows are `eventId32 | author32 | lengthBE16 | body`.

## Byte contract

All integers are unsigned big-endian. Digests and principals are exactly
32 bytes. Reserved, truncated, oversized, and trailing input is rejected.

Event: `01 | action8 | workspace32 | eventId32 | parent32 | author32 |
sequence16 | bodyLength16 | body`. Actions: `00` genesis (empty body),
`01` grant contributor, `02` grant reader, `03` revoke (32-byte target),
`04` post (UTF-8 body). Genesis uses zero parent and sequence zero.

State: `01 | workspace32 | owner32 | head32 | sequence16 | messageCount16 |
membersLength16 | messagesLength24 | seenLength16 | members | messages | seen`.
Only genesis requests use an empty state. State is at most 1,100,427 bytes.

`reduceWorkspaceBytes` accepts `50575201 | stateLength24 | state | event`
(at most 1,104,664 bytes). Success returns `00 | state`; rejection returns
exactly one error byte, in declaration order starting at `01`:

`BadEncoding`, `BadState`, `BadIdentity`, `WrongWorkspace`, `Replay`,
`StaleParent`, `StaleSequence`, `NotOwner`, `OwnerImmutable`, `AlreadyMember`,
`UnknownMember`, `CannotPost`, `MemberLimit`, `MessageLimit`, `EventLimit`,
`MessageBodyLimit`, `InvalidUtf8`, `GenesisRequired`, `AlreadyInitialized`.

## Verification boundary

LexLean elaboration, axiom audit, and kernel replay validate the formal source.
Byte comparison inherits `propext`; UTF-8 decoding inherits exactly
`Classical.choice`, `Quot.sound`, and `propext`.
Byte acceptance vectors are runtime checks on the generated implementation;
they are not claimed to be kernel-reduced ByteArray theorems. Browser crypto,
storage, transport, authenticated replay, and application acceptance require
their own integration checks before this primitive can support a deployment.

Run `node --test sdk/browser/workspace-model-test.mjs` in the SDK/devcontainer.
The 45 modeled vectors run twice through freshly generated standard Rust,
`no_std` Rust, and CoreWasm. They include simultaneous maximum message bodies,
members, and event history, plus a reachable grant producing the maximum
successful response. Native generated replay reconstructs its prestate from
genesis, 256 posts, 62 member grants, and 352 grant/revoke pairs before the final
grant; every transition must be accepted and both exact states must match.
Shared corpus data is closed literal concatenation, not another reducer.

The guest admits at most 1,104,664 input bytes and 1,100,428 output bytes,
with a 512-page (32 MiB) memory ceiling per instance.
Oversized allocation traps before invocation; native oversized byte decoding
returns `BadEncoding`. These are distinct boundaries. The generated allocator
retains previous outputs: adapters must instantiate a fresh guest per command,
reusing only the compiled WebAssembly module. Resident reuse is not unbounded
and must never reset the allocator underneath earlier output views.
