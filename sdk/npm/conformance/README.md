# @prism-code/conformance

The executable conformance suite every Prism runtime runs before it ships.
It holds scenarios over the reference `.holo` archives and their recorded histories, drives a runtime through a small entrypoint shim, and generates the compliance matrix, [COMPLIANCE.md](../../../COMPLIANCE.md) at the repository root.

This is 0.1.0 and Experimental (see [STABILITY.md](../../../STABILITY.md)).
The design is [docs/proposals/287-conformance.md](../../../docs/proposals/287-conformance.md).

## Install

```bash
npm install --save-dev @prism-code/conformance
```

The package has no dependencies.
It carries the archives it runs over: copies of `prism-code`'s reference fixtures, written by `scripts/vendor.mjs` and checked against their source in CI.

## Run the suite against a runtime

```bash
prism-conformance run --runtime prism-code --report prism-code.json
```

`runtimes.json` registers every runtime the matrix has a column for, with its entrypoint and its expected-failures file.
`prism-code` has its shim in `shims/prism-code.mjs`.
A runtime in another repository points the suite at its own shim:

```bash
prism-conformance run --runtime prismpm --entrypoint cargo --entrypoint run --entrypoint -q --entrypoint -- --report prismpm.json
```

Each `--entrypoint` is one element of the command.
The command is run once per scenario step with one more argument, the step file.

The command exits 0 only when no scenario is `failed`, `unexpected-pass` or `error`.

## The entrypoint contract

A shim reads one step file and prints one outcome.
Its own exit code is 0 whenever it produced an outcome, whatever the runtime's exit was; 2 when it cannot understand the step.

The step, `prism/conformance-step/1`, with every path absolute and `scratch` a directory the shim may write to:

| Action | Inputs | `result` on success |
|---|---|---|
| `inspect` | `archive` | The `prismpm/holo-inspection/1` object. |
| `vectors` | `archive` | `{ "passed": n, "total": n, "failed": n }`. |
| `history` | `archive` | The `prism/history/1` object. |
| `replay` | `archive`, `histories` | The `prismpm/replay-result/1` object, on exit 0 and 1. |
| `check` | `archive`, `policy` (an object, or absent for the default policy) | The `prism/verdict/1` object, on exit 0 and 1. |

The outcome, `prism/conformance-outcome/1`, one JSON object on stdout:

```json
{
  "schema": "prism/conformance-outcome/1",
  "runtime": { "name": "prism-code", "version": "0.3.0" },
  "exit": 2,
  "result": null,
  "error": { "code": "archive.magic", "message": "expected magic HOLO" }
}
```

`exit` is the runtime's exit code for that action.
`result` is the JSON it printed, or null.
`error` is its refusal with its stable code, or null.

## Scenarios

One JSON file per scenario in `scenarios/`, schema `prism/conformance-scenario/1`.
A scenario names a feature of `features.json`, cites the ids of [CONFORMANCE.md](../../../CONFORMANCE.md) it exercises, the holo format revision its archives are of, and its steps with their expectations.
`"archive": "each"` runs the scenario once per reference archive in `fixtures/*.holo`, with `$archive` and `$history` bound.

An expectation is a pattern: primitives by equality, objects by the members they name, arrays by length and element, and six operators, `$type`, `$min`, `$max`, `$regex`, `$length` and `$some`.
A failed scenario reports every mismatch by path.

## Expected failures

`xfail/<runtime>.json` declares, per runtime, the scenarios expected to fail and the features that do not apply:

```json
{
  "schema": "prism/conformance-xfail/1",
  "runtime": "prismpm",
  "xfail": { "history-is-vectors": "not implemented yet" },
  "notApplicable": { "vectors": "the Rust crate has no Wasm engine" }
}
```

A declared failure that fails is `xfail`.
One that passes is `unexpected-pass`, and the run fails: the declaration is stale.
A declaration naming no scenario or no feature is refused.

## The matrix

```bash
prism-conformance matrix prism-code.json prismpm.json --out COMPLIANCE.md
prism-conformance matrix prism-code.json --check COMPLIANCE.md
```

A cell is `+` when every scenario of the feature passed on that runtime, `-` when any did not or the runtime has no report, and `N/A` when the runtime declares the feature not applicable.
`--check` exits 1 when the committed file is not the rendering.
In this repository `npm run matrix` runs the suite against `prism-code` and rewrites the file; CI refuses a stale one.

## The format gate

```bash
prism-conformance gate
```

Fails unless every revision `FORMAT_REVISIONS` in `@prism-code/spec` declares is named by a scenario with an `inspect` step that pins `revision` to it, every scenario names a declared revision, and every reference archive is loaded by a scenario.
So a holo format change cannot merge without a scenario.

## Library

```js
import { runSuite, renderMatrix, gate } from '@prism-code/conformance';

const report = runSuite({ runtime: 'prism-code', scratch: '/tmp/conformance' });
const markdown = renderMatrix({ reports: { 'prism-code': report } });
const { problems } = await gate();
```

`src/index.mjs` exports the loaders, the matcher, the runner, the matrix and the gate.

## Development

```bash
npm test                        # the package's own tests; they run the suite against the prism-code checkout
node scripts/vendor.mjs         # copy the fixtures from sdk/npm/prism-code; --check in CI
npm run matrix                  # regenerate COMPLIANCE.md
npm run gate
```

A second reference archive joins the suite by landing in `sdk/npm/prism-code/fixtures/` with its history and running `node scripts/vendor.mjs`.

## License

MIT
