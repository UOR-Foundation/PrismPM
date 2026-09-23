# Internal effect wire ABI

Authority: `EffectsWire.lex.tex`; typed transitions: `Effects.lex.tex` (DK-18).
Grammar: `EffectsWire.cddl`. Internal prerequisite only; no application profile.

CBOR uses definite arrays, shortest unsigned 32-bit integers, byte/text strings
and booleans. UTF-8 is strict; BOM and normalization are preserved. Maps, tags,
negative integers, floats, indefinite lengths and trailing bytes reject.
Record fields and union tags follow the companion grammar. Options are tagged
arrays, not CBOR null. Framing is never identity or provenance authentication.

Bounds: frame 67,108,864 bytes; guest payload 2,097,152 bytes; text 512 UTF-8
bytes; grants 64; commit objects 16. Identity fields are at most 32 bytes,
P-256 public keys 65, signatures 64, random results 65,536. DK-18 imposes exact
widths and its stricter resource/operation rules after decoding. Guest manifest
input/output maxima also cannot exceed 2 MiB: admission cannot authorize an
effect whose completion exceeds the finite transport envelope. Fixed record
depth and bounded list recursion exclude arbitrary nested value expansion.
Cryptographic and storage payloads remain at most 1 MiB. The envelope covers
the complete declared DK-10 Workspace and DK-12 Journal guest bounds without
claiming their application compositions are implemented by this bridge.

Requests: begin `[1,0,manifest,session]`; admit `[1,1,state,request]`;
complete `[1,2,state,completion]`; close `[1,3,state]`.
Responses: success `[1,0,state]`; protocol error `[1,1,[tag]]`;
CBOR error `[1,2,tag]`. Error tags are declaration order in the typed modules;
`model/browser-effect-diagnostics.json` records the closed inventory.

`sdk/browser/effects.mjs` is a private generic host for an independently
accepted bootstrap. It exposes only intent submission, close and coarse
status; no caller completions, raw state or adapter callbacks. All grant and
artifact bytes are synchronously captured. Hash consistency does not approve
an artifact's producer, entry semantics, application permissions or standards.
Unknown outcomes retain active/waiter facts, never imply rollback or retry;
close suppresses late promotion without undoing a durable effect.
Session custody is in-memory. Durable application recovery, independently
authenticated reconciliation and public authorization are separate model and
release obligations; reopening this host does not reconcile a prior session.

Bootstrap artifact admission permits at most 256 MiB for the wire module plus
all supplied guest modules, counting repeated references separately, with at
most 64 MiB per module and 64 guest entries. Native branded byte lengths are
checked before large copies, hashing, compilation or storage; detached, shared
and out-of-bounds resizable views reject. This operational budget does not
reduce protocol bounds or establish whole-browser heap fit. Complete application
acceptance must account for copies, manifests, compilation and concurrent runtime.
