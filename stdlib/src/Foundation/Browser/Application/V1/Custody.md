# Private credential custody

Authority: `Custody.lex.tex` and `CustodyWire.lex.tex` (DK-25).
Framing: `Custody.cddl`. This is an internal SDK prerequisite, not enrollment.

Application and policy references are exactly 32 bytes. Slots and resources
are unique, byte-sorted ASCII slugs (1–128 bytes, alphanumeric first, then
alphanumeric or `._-`). Every slot is used. Signing contexts additionally allow
`/` but never `..`. Resources may share a slot with distinct contexts/maxima.
Public bindings include raw 65-byte uncompressed P-256 keys and SHA-256
principals; the host independently validates actual key possession.

The host uses a separate `prismpm.browser.custody.v1/<application-hex>` database.
Its version-1 `header` and `keys` stores have no key paths, auto-increment or
indexes. The immutable header includes the complete modeled public snapshot.
Every secret-key row repeats its application, policy, slot, public key and
principal. Missing, extra, substituted or invalid rows reject.

`openCredentialCustody({wire, wireDigest, policy, mode})` accepts only a trusted
private bootstrap; digest equality does not authenticate a producer. `mode`
is `initialize` or `open`. Atomic upgrade creates all records together. Both
modes require a subsequent strict readwrite transaction that validates and
rewrites only those identical records, followed by actual key-possession checks.
Open never creates keys. An unacknowledged creation/barrier may have committed;
explicit Open, not regeneration, determines the retained complete inventory.

`credentialPublicBindings(handle)` returns fresh public byte copies only.
`checkCredentialSigning(handle, resource, bytes)` synchronously executes the
generated admission check and returns no permit or key. `signCredential`
rechecks the same immutable binding and returns a domain-separated P1363
signature. `closeCredentialCustody` invalidates the branded handle; late
completions cannot reopen it. No reset/replacement/callback/identity fallback
exists. Same-origin trusted host code is not an isolation boundary against
other malicious same-origin code.

Limits: frame 2,097,152 bytes; output 65,536; text 128; slots/resources 64;
signing bytes 1,048,576; Wasm memory 2,048 pages; module bytes 64 MiB. All inputs
are captured before asynchronous work. Module and protocol bounds do not claim
whole-browser memory fit. Complete application custody/recovery and provenance
are separate obligations; public browser application builds remain refused.

Source tests: `tests/browser-custody/`; browser owner:
`sdk/browser/credential-custody-test.mjs`. Diagnostic tags are declaration order
and recorded in `model/browser-custody-diagnostics.json`.
