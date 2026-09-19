# Internal browser effect protocol

`Effects.lex.tex` owns bounded sequencing and typed request/result admission.
This kernel is not an application profile, browser dispatcher, authentication
mechanism, storage implementation, or deployment acceptance gate.

## Admitted facts

Trusted bootstrap supplies an independently verified manifest and a fresh
private session. The manifest binds the application and manifest references,
unique resource IDs, and exact adapter grants. A guest grant binds its artifact
reference, qualified `Bytes → Bytes` entry, protocol identifier, and positive
input/output/memory budgets. No Workspace role or fixed guest topology is
implied. Identifiers and structurally valid records do not establish authority,
artifact integrity, or an entry's actual type.

Each captured manifest admits 1–64 uniquely named resource grants. Resource
and protocol references contain 1–128 UTF-8 bytes; entry references contain
1–512 ASCII bytes in qualified identifier form. Application, manifest, session
and guest-artifact references contain exactly 32 bytes. These are per-session
protocol bounds, not platform totals. Guest budgets are positive, no greater
than independently admitted host limits, and representable by Wasm32: each
byte limit is at most 4,294,967,295, memory is at most 65,536 pages, and summed
input/output limits fit the declared pages. This arithmetic is necessary but
not sufficient for execution: allocator overhead and actual guest demand need
independent compiled-guest resource acceptance. No universal 2-MiB ceiling is
imposed, and finite tests do not execute every guest at its declared maximum.

Private host completions must originate from the corresponding captured
operation. Matching modeled bytes alone do not authenticate a completion or
prove that cryptography, execution, or persistence occurred. Raw product callers
must not supply manifests, sessions, pending state, or host completions.

## Closed effects

The inventory binds only existing SDK primitives:

- Generated guest invocation: exact manifest guest ID, artifact, entry,
  protocol, bounded request, and bounded result; fresh instance per invocation.
- `identity.mjs`: random bytes, SHA-256 digest, P-256 signing and verification.
  Signing grants bind an already possessed private key's public reference and
  exact signing context; private keys never enter the model. Verification grants
  bind the exact public key and context. No key creation, account enrollment,
  mailbox assertion, credential replacement, or recovery is implied.
- `store.mjs`: object read, named-head read, and atomic object/head commit in a
  fixed admitted namespace. Commit binds expected head, next digest and every
  supplied object. No deletion, replication, cross-device CAS, or global receipt
  is implied.

Random requests admit 1–65,536 bytes. Digest/sign/verify payloads admit at most
1,048,576 bytes. Public keys are raw 65-byte uncompressed P-256 points and
signatures are 64-byte P1363 values; shape does not prove curve membership or a
valid signature. Contexts use the existing 1–128-byte ASCII host grammar.
Storage retains its existing 1 MiB/object, 4,096-object, 64-head and
16-object/commit limits, with immutable smaller per-namespace limits. Object
digests use the host's exact lowercase `sha256:` representation, not Kappa IDs.

No network, provider, email, UI, arbitrary JavaScript callback, source URL,
unimplemented adapter name, or general exception payload is admitted.

## Sequencing

Every request binds application, manifest, private session, monotonically
allocated operation number, resource, typed operation, and complete payload.
The session admits one active request and at most one waiter. A third request
fails before admission. Operation numbers never wrap. A waiter cannot complete
before its own promotion and does not execute merely because it was admitted.

A completion repeats the entire request binding and contains exactly one
operation-compatible result: completed, definitively rejected, or unknown.
Accepted completed/rejected outcomes consume the active operation once and
promote the already-bound waiter. No result can select another guest/resource,
alter payload bytes, or supply a larger budget. Stale, duplicate, reordered,
cross-application and cross-session completions reject without state change.

Unknown outcome retains both outstanding records and blocks further admission,
dispatch and automatic retry. In particular, a lost commit acknowledgment is
not a failed write. Clearing this barrier requires separately verified
reconciliation; this kernel supplies no invented recovery success operation.
Close also retains outstanding records, prevents promotion and rejects later
completion. It does not cancel or roll back an already started durable effect.
Persistent custody of those records is an application/storage obligation.

Read failures include `crypto-unavailable`: object/head reads verify stored
bytes with the existing digest primitive. They also preserve the storage
adapter's `storage-quota` mapping. Neither read changes durable state. A commit's
generic `storage-unavailable`, unexpected exception, cancellation or lost
acknowledgment cannot be inferred to mean rollback; its private adapter must
report unknown unless it independently establishes a definitive rejection.

## Acceptance scope

The finite corpus covers each typed operation, exact resource and payload
binding, resource exhaustion, waiter promotion, consume-once, unknown outcomes,
and close retention. Source/kernel/native acceptance belongs to DK-18; actual
browser adapters, artifact closure, Core-Wasm execution, complete application
composition, publication and release remain separately required. Existing
DK-10–16 guests are composition fixtures, not mandatory application roles.
