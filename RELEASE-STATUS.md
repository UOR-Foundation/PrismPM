# Production release status

As of 15 September 2026, the 0.3.0 source is under verification and is not an
accepted public SDK release. Clean commits and passing component checks do not
replace the cross-repository acceptance contract in `current/tasks.md` of the
development workspace.

## Public dependency prerequisite

Both the standalone package gate and Cargo's own online package preparation
have failed to resolve the required public `uor-hologram` package:

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

## SDK oracle advisory disposition

On 15 September, devcontainer `npm audit --package-lock-only --ignore-scripts
--json` reported eight affected package records (seven high, one moderate) in
the pinned AsyncAPI official-example harness lockfile. Its affected packages
are Spectral core/functions, Ajv, brace-expansion, fast-uri, js-yaml, Lodash,
and minimatch; this is not a count of distinct advisories. The separate
`sdk/oracles` lockfile used for submitted documents reported zero findings.
These audits cover lockfile graphs, not installed trees or the complete SDK.

The affected harness is retained in the SDK and executes pinned official
examples in a read-only, network-disabled container with a timeout. No
hostile-application exploit was demonstrated; neither these bounds nor passing
functional tests establish a vulnerability-clean SDK. Resolve the findings
through reviewed upstream dependency updates and complete oracle reruns, or
evidence-backed disposition under an approved policy. No disposition is accepted.

Local reports are `target/asyncapi-upstream-audit-20260915.json` (SHA-256
`831b23c1b993d18026f25ac58f42cf86778be404f5d4b89ffb5ae80d3d534bc7`)
and `target/asyncapi-sdk-audit-20260915.json` (SHA-256
`5b4208b5299acc2d0dd8bfd35e914ceb5568f21c34e93cbb864cb64498f8df48`).

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
