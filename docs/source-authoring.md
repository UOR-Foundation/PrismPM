# Installed source-authoring interface

The SDK's `prismpm` executable exposes the pinned LexLean authoring API through
project-free JSON stdio:

```text
prismpm --json source read
prismpm --json source write
prismpm --json source to-notation
```

Send one UTF-8 JSON request on stdin and close stdin. `read` and `to-notation`
accept exactly `schema` and `source`:

```json
{"schema":"prismpm/source-authoring-request/1","source":"\\begin{semanticmodule}\n\\semanticdata{ {\"spec\":\"lexlean/semantic-module/2\",\"declarations\":[]} }\n\\end{semanticmodule}\n"}
```

`write` additionally requires `value`, the complete replacement parsed module
value. An explicit JSON `null` is a supplied value; omitting `value` is an error.
The pinned SDK decides which values can be written into each source form.
Unknown or repeated envelope members, an unsupported schema, invalid UTF-8 and
trailing JSON values are refused. The module payload keeps the official SDK's
JSON-value behavior, including unknown fields and explicit or absent defaults.

Successful stdout contains one compact JSON result followed by LF. A read
result has `schema: "prismpm/source-authoring-result/1"`, `operation: "read"`
and the complete `value`. A text result has the same schema, its operation
(`write` or `to-notation`) and `source`, the exact string returned by the SDK.
Newlines and other characters within source are JSON-escaped; decode the string
before saving it. Its final newline is independent of the envelope's framing LF.
Successful stderr is empty. Requests and results have registered JSON schemas.

Their distinct `sha256-source-authoring-json` identity hashes the complete
compact UTF-8 JSON serialization **including LF** after recursively sorting
object keys lexicographically. The pinned serde_json serializer determines
finite number representation, including negative zero and exponents. Request
identity is computed on this serialization of the accepted envelope; input
whitespace is not identity-bearing. Result bytes already use this encoding.
This codec is versioned separately from integer-only canonical model metadata.
The registry's signed-64-bit ceiling expresses target addressability; actual
runtime budgets are the explicit lower input/output settings described below.

The implementation calls `lexlean::sdk::read_module`, `write_module` and
`to_notation` in the linked pinned library. It accepts the module kinds and
data/notation forms supported by that SDK, including its explicit refusals for
unsupported conversions. It does not normalize through typed module DTOs. JSON
numbers, including floating-point values, use the pinned SDK/serde_json value
representation; original number spelling, whitespace, duplicate payload tokens
and JSON member order are not a source-text archive. Writing preserves or
changes source according to the official SDK operation, not a second parser.
The current pinned SDK's writer and notation converter require its canonical
JSON domain, which uses signed 64-bit integers and excludes `null` and
non-integer numbers. A raw read can
preserve those values; attempting to write them produces `PP5802` without a
successful source result. The adapter never drops them or changes their type
to manufacture a successful round trip.

The command does not load a project or lock, invoke a compiler/helper subprocess,
fetch packages, link a checked program or prove it. `prismpm build`'s checked
`lexlean/snapshot.json` remains a separate representation and acceptance surface.
Use the immutable installed SDK selected by the consumer's lock. A source-built
CLI result alone does not establish acceptance of that image or authorize a consumer lock change.

`--max-input-bytes` and `--max-output-bytes` default to 16 MiB and 64 MiB. They
can be overridden before or after the operation. Both must be positive and
addressable on the target. Input counts the entire request's encoded bytes;
output counts the entire escaped successful result envelope **and its LF**. These are
transport budgets, not language input-size limits. Input is read with a bounded
lookahead. The complete result is serialized into checked staging storage before
publication. The official SDK may allocate its returned JSON/source before that
output check; these budgets do not establish a bound on the process heap or make
every allocator failure recoverable. Existing process, stack and SDK parser
limits remain applicable.

Before publication, a refusal produces the ordinary `prismpm/error-result/1`
envelope and a nonzero status, with no successful result:

| Code | Exit | Meaning |
|---|---:|---|
| `PP5801` | 2 | Invalid request or byte-budget configuration |
| `PP5802` | 1 | Official SDK authoring refusal; available source byte offset retained |
| `PP5803` | 14 | Transport byte limit or checked staging allocation refusal |
| `PP5804` | 1 | Stdin read or stdout write/flush failure |

Error envelopes follow the existing diagnostic contract rather than the selected
successful-payload budget. Source-authoring messages are separately truncated
at a UTF-8 boundary after 2,048 bytes, with an explicit truncation marker.

A failed stdout write or flush may have published part or all of an envelope.
The command emits `PP5804` only on stderr, records stdout outcome **Unknown**,
and never appends an error envelope to that stream. A nonzero status invalidates
the result even if its bytes appear complete. Callers should stage and validate
the complete response and process status before replacing their source file.
The command itself does not publish source files or promise rollback of a pipe.

The normal SDK recipe runs a construction self-check after inventory creation.
The native SDK runtime gate runs the absolute installed executable offline,
without project/source directories, and retains an image-bound component receipt
containing the actual CLI and tool hashes, fixed inventory identity, SDK source
revision and captured source/lock/toolchain identities. Construction-only and
source-built results do not substitute for that installed-image acceptance.
