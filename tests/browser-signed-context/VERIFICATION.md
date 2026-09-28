# DK-32 verification

Status: **incomplete; not accepted**. No SDK image, public Browser runtime,
account, mailbox, organization or Foundry release is accepted by this work.

Pinned Noble SDK/devcontainer image:
`sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`.
Rust 1.97.1, Lean 4.32.1, Node 22.23.2, Playwright/core 1.62.1.

The first complete attempt reached a genuine provider failure after 254.758 s:
source/kernel/observed axioms, native std/no_std, two independent Wasm builds,
all 1,220 vectors, artifact/compiler substitution guards and Chromium journeys
passed. Firefox's original ECDSA WPT entry failed 24 cases. The subsequent
23 source mutations were not reached; this attempt is not complete evidence.

Historical component identities:

- Source: `a1b4bbbfd7fcf4569e8ac9c6588cb439b012ff9cd97200f76148a6963238469f`.
- Attestation: `454795c005098c097b68a2eb0fc84f045a7b156331ee7b22edd5dc39d23d38f3`.
- IR: `1ee683789640c1a3b98fbf6231d07b34f2ce83f758197250d02bd65f06ad1fcc`.
- Wasm: `38d630d941a2b5f9c62dafbeb2724e07b809778380d969e78bcb2424c9697661`.

A fixture-only correction serves immutable original WPT scripts through actual
loopback HTTP, including worker imports. A separate diagnostic collected all
1,944 cases: Chromium 151.0.7922.34 passes 324 per realm; Firefox 153.0 and
WebKit 26.5 each fail 24 per realm. Every failure remains a failure. Exact raw
results SHA-256: `95bfef514cb5629a9a19ff6275cb3e0bc1a345fb36c45aae000aeae5e3603011`.

The transfer cases observe two algorithm-name getter reads in Firefox/WebKit.
The unresolved normative clarification is [WebCrypto issue 563](https://github.com/w3c/webcrypto/issues/563).
The original source is not patched and upstream failure metadata is not an
exemption. SDK captured-byte behavior is a distinct, narrower API; its separate
three-engine diagnostic is not provider conformance. WebKit's independently
observed invalid-point import also requires generated point admission before
key facts can be trusted.

Current orchestration retains every required check, collects independent
browser/source failures, and throws an aggregate before any success receipt.
Generated baseline and artifact integrity remain safety prerequisites. A
failed pristine browser cannot authorize host-mutant evidence. Three new
construction tests include actual Node failed-child propagation, missing check
refusal and an executed removed-final-guard defect. Source/installed owner
minimum is 17; every original case remains required.

Current bounded checks: all 39 construction/complete SDK boundary tests pass
(81.704 s), including the actual removed-final-guard defect and complete static
owning closure. Normal model writer/readback and specification bijection pass
(186 IDs, 86 error codes). Earlier authored-format owner: 5/5 (4.53 s).
These are not a fresh complete component or installed-image receipt.

The complete aggregate attempt at source commit `d135287` finished in 1,865.991 s:
13/17 tests passed, four failed, none skipped or cancelled. Generated 1,220-case
parity, artifact substitution checks and final closure passed. Chromium and
Firefox each passed 29 SDK journeys, exact native replay and six host mutants.
All 23 source mutants ran: 21 produced genuine compiled counterexamples;
`ContextValidity` and `KeyValidity` failed strict `LLV7006` unused-parameter
warnings, not behavioral verification. Their constant-body fixtures need correction.

The original raw WPT failures remain unchanged. WebKit's pristine SDK journey
also failed: its invalid-point verification returned `signature-rejected`, not
the required invalid-point error. A separately labeled historical-artifact
diagnostic reproduced that exact branded counterexample; it did not authenticate
the invalid key. WebKit host-mutant acceptance was correctly refused without a
passing pristine baseline. Generated point admission remains required.

Attempt receipt: `fc165e27ec33946b7730561aa0aea3a00a8576a7b8ea3998806816bfd18c732d`;
all 846 original input files were independently rehashed afterward. Compiler
caches retired normally, and no successful owner receipt was written. Logs:
`/tmp/dk32-owner-report-VODIOl`; artifacts:
`/tmp/prismpm-signed-context-qtdZY7` in `prismpm-signed-context-owner`.

Remaining: authoritative edition/profile clarification for the raw provider
oracle, generated curve-point validation, corrected mutation fixtures, fresh
complete three-engine/source/host verification and installed SDK acceptance.
The normative original-WPT requirement is unchanged.

Local diagnostic logs are retained in `prismpm-signed-context-owner` at
`/tmp/dk32-owner-report-SSdk6q`; the source/artifact component is retained at
`/tmp/prismpm-signed-context-6zns2l`. Workspace copies of logs and raw reports
are under `PrismPM/target/dk32-verification-am2VCNyb/`.
