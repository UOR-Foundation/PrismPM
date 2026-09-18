# Admitted workspace query prerequisite

Generated LexLean read decisions; no Foundation identity, Kappa,
network authentication, or global instantaneous revocation claim.

Trusted context is privately derived from complete authenticated Journal replay
and a validated possessed identity. Bootstrap retains a fresh private32-byte
session. Public intent supplies only table, workspace and cursor. No public
principal, state, head, digest, authentication flag, or receipt exists.

Fixed engineering page size16. Members include the immutable owner(role00),
then the ordered contributor01/reader02 rows. Messages retain exact event ID,
author and UTF-8 body bytes. All64 members/256 messages/4096-byte bodies/1024
events remain representable. Owners, contributors and readers may read;
unknown/revoked principals cannot. Authorization is evaluated on each query.

Private request: PWQ01(50 57 51 01) | principal32 | session32 | headId32 |
headLen24 | stateLen24 | exact head | exact state | intentLen16 | intent.
Public intent: table8(00 members,01 messages) | workspace32 | cursorLen16 |
cursor. An absent cursor starts at0. Cursor: PQC01(50 51 43 01) | session32 |
principal32 | workspace32 | headId32 | table8 | nextOffset16 (135bytes).
Nonempty positions must be positive page boundaries strictly before total.
Final page returns no continuation; empty messages returns a complete empty
page. A cursor is an untrusted versioned position, never an authorization grant.

Success: 00 | table8 | workspace32 | headId32 | total16 | offset16 |
rowCount8 | nextCursorLen16 | nextCursor | rowsLen24 | rows.
Rows are exact33-byte member records or existing66+body message records.
Maximum public intent170bytes, private request1166279bytes and page66803bytes.
The response fits the existing1-MiB response boundary, not the peer64-KiB frame.

The host hashes the exact captured head with actual SHA-256, refreshes again,
and refuses a changed head before invoking or exposing the generated read.
Every query starts with fresh replay. Results are explicitly as-of their bound
head; a later concurrent commit does not retroactively erase disclosed bytes.
Private context mutation across awaits and revocation between pages are owning
negative tests. Syntax/state validation alone never authenticates a snapshot.

The returned private host API is `query({table,workspace,cursor})` and `close()`;
it has no snapshot, head, state, principal-switch or receipt method. Bootstrap
alone supplies verified modules and storage. A page exposes only its table,
workspace, head ID, total/offset/count, continuation and admitted row bytes.
At most two host operations may be outstanding (one active, one waiter).
Capacity is reserved before public input inspection; close drops pending work
and prevents late disclosure. This host admission budget changes no model cap.

Typed model failure bytes01..0d are BadEncoding, InvalidHead, InvalidState,
InvalidContext, WrongWorkspace, NotAdmitted, InvalidCursor, StaleCursor,
CursorSession, CursorPrincipal, CursorTable, CursorRange and UnknownTable.
The adapter preserves those as `query-rejected` with the exact code. Lifecycle,
input, effect and generated-output failures remain separate host errors.

The literal maximum fixture retains the accepted Workspace corpus bytes but
uses synthetic object IDs: it proves structural/parity/traversal behavior,
not cryptographic history. Actual signature and replay journeys independently
exercise the host trust boundary. Neither the structural model nor its caller-
supplied internal headId can authenticate arbitrary context by itself.

QY-01: closed codec, complete state/head binding, current role admission.
QY-02: bound cursor, exact complete traversal, all original maxima retained.
QY-03: fresh normal C/native/no_std/CoreWasm/Chromium parity and measured budget.
QY-04: actual private browser identity/replay, head-race and input mutation tests.

Query guest budget512pages/32MiB is explicit. The modeled borrowed-octet
decoder preserves every original62 vector byte. All256octets plus empty/long helper
inputs are independently checked in a test-only generated probe. This is a hard
runtime cap and finite measured coverage, not a universal termination/memory
proof. Workspace512/Journal640/Command64 budgets remain unchanged.
