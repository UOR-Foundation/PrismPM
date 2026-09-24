# Browser-system source fixture

`Release.lex.tex` contains reviewed A/B models, manifests and generated-proof
declarations for the selected Calculator application. The owning test materializes
these files with the repository's Calculator and `Production/{Core,BrowserSystem}`
sources. It never replaces the committed application digest during verification.

After an intentional selected-application change, run in the devcontainer:

```sh
cargo test --locked --offline -p prismpm --lib \
  browser_system_source_selection_and_requirements_fail_closed -- --nocapture
```

A stale binding fails with the actual imported application-model digest. Review
that source change, then update the `applicationModelDigest` values in both
models and both manifests in `Release.lex.tex` to the reported digest. Do not
use the standalone Calculator project's digest: the owning test selects the
application through this fixture's actual LexLean configuration.

Run `conformance_sy_08` with the immutable oracle image and the installed-SDK
product gate afterward. Source/proof/transport acceptance does not replace
genuine installed-SDK, supply-chain or publication acceptance.

## Integrated verification, 20 September 2026

Registered `conformance_sy_08` passed in 534.63 seconds, including all three
source, named A/B proof/release and source-free export owners. The selected
application identity is
`sha256:f7cef5c6eedddb175e5fb9b6de5d6f834265c81702373efe789112510935cf59`;
its current reviewed emitter identity is
`8d827d30d39fba0cbadeec78ffada8cf96079edd6a130023af54483a56a9536c`.
All four source bindings were reviewed and updated after emitter changes;
verification never rewrites them.

The actual SPDX oracle ran with `PRISMPM_TEST_SDK_IMAGE` set to
`ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`.
Its separately executed production-runner positive/negative test also passed.
This uses that image's locked SPDX implementation only; it does not accept the
image as the current SDK, enable an effectful Browser application, or establish
Foundry producer/deployment acceptance.
