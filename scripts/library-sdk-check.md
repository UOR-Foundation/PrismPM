# Installed native-library gate

`bash scripts/library-sdk-check.sh IMAGE@sha256:DIGEST SOURCE_COMMIT` checks
DK-17 through the installed CLI in the exact current native SDK. Both SDK
architectures must pass in the release reproducibility jobs; DK-07–16 remain
separate mandatory checks.

The gate compares the closed source/compiler/schema/fixture/helper inventory,
then executes non-root, read-only and offline with private temporary caches.
Fresh library roots must reproduce all build bytes and execute their generated
crate in std and no_std modes. Read-only checks, missing/wrong-typed roots,
nominal impostors, a false modeled acceptance and product-release refusal are
required. Restoring the source must restore successful verification.

DK-29 additionally binds two fresh native compiler constructions to the
captured image inventory. The first seed is reused for cold/relocated kernel
checks at two roots and eight exact exports. Original records and bytes are
retained. Construction runs non-root in bounded temporary filesystems; a
separate root-only fixture checks filesystem custody. No extra image is built.

`node scripts/library-sdk-check.mjs tests` runs the complete owning boundary
and recording-Docker shell tests; omissions and skips fail. These tests do not
establish installed-image acceptance. Source-free release
refusal remains covered by the Rust DK-17 case, not this CLI gate. Full SDK
V&V, fresh locked acquisition and product/deployment acceptance remain separate.
