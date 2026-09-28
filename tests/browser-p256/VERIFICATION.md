# P-256 validation

DK-34 is unaccepted. The owner is registered before implementation and must
fail until the complete source, native, Wasm, authoritative-vector, browser,
mutation and consumer-integration checks exist and pass. WebCrypto import alone
does not establish curve membership on all pinned engines.

This component does not certify ECDSA, prove key possession, assign authority,
or replace account/session freshness and recovery acceptance.

The first fresh full owner passed all six construction checks and exact source/
kernel verification, then failed native generation. The exported constant limb
lists contain scalar literal `let` bindings; the pinned `prod-codegen`
`static_list` implementation refuses those before reaching the constant cons
chain. No arithmetic, frame, allocator, oracle or browser requirement was
relaxed. The source remains unchanged pending a verified compiler correction.

Retained diagnostic: `prismpm-p256-owner:/tmp/prismpm-p256-Th8ClG`; full log:
`/tmp/prismpm-p256-owner-run-YgQhBgyO/owner.log`. This is not native, Wasm,
browser, SDK or application acceptance.

## Conditional compiler diagnostic — 28 September UTC

Upstream PR #77 (`84ba0591cddbc7f327d7c09e04bd81bdc56b1a67`) has passing
complete upstream CI and independent review, but is not merged. Its CLI was
built in the devcontainer and used against the actual retained kernel IR
`53d2262cf2522fb7f1b59bb35d35fe9c59fcd00202cbb47be8b62c0b1090413f`.
All 1,505 vectors pass twice in native `std`, `no_std`, Node Wasm, Chromium
151.0.7922.34, Firefox 153.0 and WebKit 26.5. Observed Wasm memory is 5,242,880
bytes; allocation-overflow refusal remains mandatory.

Fresh diagnostic source/kernel/export/native/Wasm rebuilds detect all 22
semantic mutants. The previous right-half mutant failed structural-recursion
checking (`LLT4001`), which is not a behavioral counterexample. It now loses
the left-half accumulator while preserving structural descent; actual native
and Wasm execution detect the intended wrong result. Seven cheap construction,
parameter, corpus and original-oracle-integrity tests pass.

Diagnostics are retained in
`prismpm-p256-owner:/tmp/foundry-p256-codegen-diagnostic-fbtgEo`, with the final
mutation record under `mutations-VJ50x4/mutations-diagnostic.json`. Earlier
failed diagnostic attempts remain retained. No SDK/compiler pins, source
admission policies or production acceptance flags changed. This diagnostic
reuses captured verification/export tools and an explicitly different upstream
code generator; it is not the fresh pinned compiler-owner receipt. The full
DK-34 owner, installed SDK, account/signed-context integration and Foundry
acceptance still require completion with the accepted compiler closure.
