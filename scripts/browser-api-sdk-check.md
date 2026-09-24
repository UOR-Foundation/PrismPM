# Installed browser prerequisite gate

`bash scripts/browser-api-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT`
requires a clean, exact source commit and the immutable native SDK built from
it. Both native release jobs run the gate independently of source V&V.

The gate compares the complete installed host-module directory and measured
source/compiler/fixture closure, then executes all DK-07–16, DK-19, DK-20,
DK-23–25 Node owners offline with read-only sources and private temporary caches.
Every selected file must register passing tests; missing, skipped or invented
completion records reject. Driver manifests and locks are acquisition-pinned.
DK-21/DK-22 execute in the separate mandatory full installed native V&V.

`node --test scripts/browser-api-sdk-check.test.mjs scripts/fetch-oracle-cargo.test.mjs`
checks these boundaries, including module-tree and byte-equality mutations.
Those tests do not establish installed-image acceptance. The private effect,
compiler, presentation, operation-journal and credential-custody prerequisites do not open
`PP2011`, issue grants or accept an application, deployment or account/recovery
journey. The operation journal retains its complete 28-test owner and the
unchanged effect owner; neither is replaced by inventory checks.

## Verification

2026-09-20, non-root devcontainer: inventory/acquisition tests reproduced the
missing entries before correction; all 37 boundary/acquisition/candidate tests
and complete `xtask validate` (163 audit tests) then passed without skips.
The subsequent accepted DK-25 inventory update reproduced its five missing
closure boundaries and passed those same complete gates again.
The accepted DK-24 update reproduced four missing boundaries, retained its
28-test owner and unchanged 18-test effect regression, then passed all 37 scoped
tests (18.34 s) and the complete 163-test audit (45.09 s) in the devcontainer.
The immutable two-architecture installed-image gate remains required; it was
not executed for this packaging-only change.
