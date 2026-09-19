# Bounded deterministic CBOR primitives

`Primitive.lex.tex` is an internal codec prerequisite. It implements the
specified primitive subset, not a universal CBOR decoder, application wire,
browser application profile, or complete standards certification.

## Representation

RFC 8949 §§3.1, 3.3 and 4.2.1 govern unsigned integers, byte/text strings,
booleans, null and definite-array heads. Integer and length arguments use the
shortest permitted encoding. Integers and lengths are limited to uint32.
Eight-byte arguments, negative integers, maps, tags, floats, undefined,
unassigned simple values, indefinite lengths and reserved heads are rejected.
Text must be valid UTF-8; there is no replacement decoding, normalization,
BOM removal or transformation of embedded zero/control characters.

`CborValue` contains `Unsigned`, `ByteString`, `TextString`, `Boolean` and
`Null`. `readCborPrimitive` returns that value and the next `BoundedCursor`;
`writeCborPrimitive` returns its canonical bytes. `readCborArrayHead` returns
only a count and next cursor. It does not inspect or accept an array body.
For example, `81` is a valid one-element array *head*, not a complete array.
The schema-specific caller must consume exactly that many admitted elements.
`writeCborArrayHead` similarly emits a head, not a complete nonempty array.

`finishCborCursor` checks that no bytes remain inside the admitted cursor
window. It cannot establish that previous calls consumed an array body or
that a schema-specific value was decoded correctly. Prefix decoding and
whole-value validation are deliberately distinct.

The included CDDL records the primitive value domain using RFC 8610 §§3.4
and 3.8.1. CDDL value/size constraints alone do not require deterministic
encoding; the RFC 8949 rules above remain mandatory. There is no generic
CDDL parser, arbitrary tree traversal or claim that every CDDL construct
is implemented.

## Resource boundary

`CborLimits` explicitly admits maximum input bytes, output bytes, byte-string
bytes, text-string UTF-8 bytes and array item count. Each limit is 0..uint32
maximum; zero is a real zero budget. These are per-invocation limits, not
platform capacity or authority. The caller must admit the limits and raw
input in its actual runtime before allocating/copying untrusted input.

Every public reader validates `offset <= limit <= bytes.length` and the
input budget. It checks announced payload length against its own budget
and remaining cursor bytes before slicing or UTF-8 decoding. Header reads
have fixed widths of at most five octets; an array count never starts a loop
or allocates an array. Arithmetic follows bound checks and does not wrap.
Readers preserve the original byte sequence and window limit in their
returned cursor. No failed read returns a changed cursor or partial value.

Encoders enforce their payload and complete output budgets. Typed strings
are already allocated inputs: a preliminary character-length check bounds
UTF-8 conversion, and the exact UTF-8 byte count controls acceptance. This
is not a promise that arbitrary typed values can be created inside Wasm.
Internal helper declarations require their checked callers' invariants.

`canonicalCborPrimitiveBytes` is an internal, finite execution-test entry:
it admits at most 4,202,618 input bytes, at most 4,202,612 payload bytes and
at most 4,202,617 output bytes. It decodes one primitive, checks the full
input window and re-encodes it. Any rejection returns empty bytes, which
cannot be an encoding of a supported primitive. It does not process arrays.
The payload maximum is a required existing Holo-codec test point, not an
assertion that uint32-sized inputs or a future Effects envelope fit the
same Wasm budget. Actual native/Wasm maximum-size execution and allocation
bounds are separate mandatory evidence; source typing is insufficient.

## Rejection and oracle

Errors are closed: `BadLimits`, `BadCursor`, `Truncated`, `WrongType`,
`UnsupportedHead`, `NonCanonical`, `ValueLimit`, `InvalidUtf8` and
`TrailingInput`. Public entry validation precedes parsing. Type selection
precedes argument decoding; unsupported eight-byte/reserved/indefinite
heads are rejected without trusting their lengths. Payload budget rejection
precedes missing-payload rejection. Nonminimal supported-width arguments
are `NonCanonical`. Valid-but-out-of-profile data is not called invalid
universal CBOR.

Exact RFC 8949, RFC 8610 and RFC 3629 source texts and their original notices
are retained under `tests/fixtures/codec/cbor/`. Their SHA-256 identities,
applicable source vectors, profile exclusions and independently authored
boundary cases belong to the owning oracle. RFC 3629 §§3–4 supplies the
UTF-8 boundary cases required by RFC 8949's text type. Finite tests and
planted defects must not be described as universal proof of those standards.

Executable definitions declare their exact transitive dependencies: UInt8 bit
operations inherit `Quot.sound` and `propext`; Lean's `String.fromUTF8?` and
`String.length` also inherit `Classical.choice` through their UTF-8 proofs. Unrelated
declarations retain empty policies. Every observed set must match its source
declaration exactly; these are not axiom-free theorem claims. No theorem policy
is relaxed.
