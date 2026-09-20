# Installed browser prerequisite gate

`bash scripts/browser-api-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT`
requires a clean, exact source commit and the immutable native SDK built from
it. Both native release jobs run the gate independently of source V&V.

The gate compares the complete installed host-module directory and measured
source/compiler/fixture closure, then executes all DK-07–16, DK-19, DK-20 and
DK-23 Node owners offline with read-only sources and private temporary caches.
Every selected file must register passing tests; missing, skipped or invented
completion records reject. Driver manifests and locks are acquisition-pinned.
DK-21/DK-22 execute in the separate mandatory full installed native V&V.

`node --test scripts/browser-api-sdk-check.test.mjs scripts/fetch-oracle-cargo.test.mjs`
checks these boundaries, including module-tree and byte-equality mutations.
Those tests do not establish installed-image acceptance. The private effect,
compiler and presentation prerequisites do not open `PP2011`, issue grants or
accept an application, deployment or account/recovery journey. DK-24/DK-25
require their own accepted inventory integration before installation.

## Verification

2026-09-20, non-root devcontainer: inventory/acquisition tests reproduced the
missing entries before correction; all 37 boundary/acquisition/candidate tests
and complete `xtask validate` (163 audit tests) then passed without skips.
The immutable two-architecture installed-image gate remains required; it was
not executed for this packaging-only change.
