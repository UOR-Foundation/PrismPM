# Workspace envelope

`WorkspaceEnvelope` is a bounded codec, not authentication or authorization.
PWE01 is `50 57 45 01 | publicKey65 | signature64 | event134+body`, 267–4,363
bytes. Public keys require uncompressed prefix `04`; signatures are opaque
raw P1363 bytes. Event lengths close the envelope exactly. Unknown versions,
actions, truncation, trailing bytes and over-limit bodies fail.

`workspaceEnvelopeBytes` accepts `operation8 | envelope`: 0 roundtrip,
1 unsigned event, 2 complete signing preimage, 3 public key, 4 signature,
5 event, 6 author, 7 event ID. Success is `00 | bytes`; malformed input is
`01`; unknown operation is `02`. Request/output caps are 4,364 bytes and
Core-Wasm admits 32 pages with a fresh instance per command.

The generated preimage uses fixed context `prismpm/workspace-event/1` under
the SDK signature domain. Before a journal transition, the private adapter
checks real P-256 key import, SHA-256 key principal against author, SHA-256
complete signing preimage against event ID, and the actual signature.
Parsing or signature verification alone cannot establish these bindings.
IDs are not Kappa addresses or organizational identities.

DK-11 owns all 43 modeled vectors twice in generated native/no_std/Wasm,
typed encode rejection, every truncated genesis prefix, full normal Lean C
generation and real Chromium cryptography with native projection replay.
