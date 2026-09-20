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
