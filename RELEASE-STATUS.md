# Production release status

As of 9 September 2026, the 0.3.0 source is under verification and is not an
accepted public SDK release. Clean commits and passing component checks do not
replace the cross-repository acceptance contract in `current/tasks.md` of the
development workspace.

## Public dependency prerequisite

Both the standalone package gate and Cargo's own online package preparation
fail because `uor-hologram` is absent from crates.io:

```console
cargo xtask package-api
cargo package --package prismpm --no-verify
```

The package gate resolves registry dependencies as Cargo does when publishing;
the development Git checkout is not a substitute for a public registry crate.
The former Hologram 0.12.1 source predates registry versions on its internal
path dependencies. PrismPM now pins the publishable Hologram 0.13.1 source at
`96769f16be454ab1572fddff4613704ccfbebf5e`; the corresponding packages must be
published before the public PrismPM package can be accepted.

Hologram's existing publication workflow publishes its complete 19-crate
workspace. Its last real upload failed with HTTP 403 because
`CARGO_REGISTRY_TOKEN` lacked the required permissions. Subsequent successful
workflow runs were dry runs. The repository owner must supply a credential
authorized to publish that package closure; credentials must never be
committed or included in an SDK. The failure is recorded in the
[upstream publishing job](https://github.com/Hologram-Technologies/hologram/actions/runs/34018837931/job/101447544153).

## Remaining release acceptance

After dependency publication is authorized, complete the existing release plan:

1. Accept and publish the Hologram dependency closure, and verify the downloaded
   bytes. Verify the generic compiler release packages and their publishing
   identities, including LexLean 0.3.0 and the lean4-prod fork/upstream changes.
2. Reproduce the PrismPM dependency closure and its package, golden
   artifacts, Calculator regression, and all source/package/image checks.
3. Pass every PrismPM gate twice without cleanup. Publish and independently
   verify the exact PrismPM/stdlib crates, SDK, runtime, adapters, and oracles.
4. Bind template and Calculator locks/workflows to those public immutable
   artifacts. Regenerate their source projections and preserve their accepted
   application baseline. Run their complete local and CI acceptance, including
   both production releases, deployments, rollback, recovery, and Pages.
5. Verify the complete `prismpm/ecosystem-release/2` manifest and only then date
   the changelog, create release tags, and claim completion.

These are outstanding requirements, not exclusions or reductions of scope.
