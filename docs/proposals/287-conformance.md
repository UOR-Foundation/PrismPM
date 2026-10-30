# Proposal 287: `@prism-code/conformance` and `COMPLIANCE.md`

| | |
| --- | --- |
| Issue | [#287](https://github.com/UOR-Foundation/PrismPM/issues/287) |
| Milestone | M3 v0.6 Replay, capabilities, conformance |
| Status | Proposed, Experimental. JavaScript runtime only; the Rust and Python columns wait for their entrypoints |
| Depends on | #263 (`@prism-code/spec`, the revision table), #284 (histories and replay), #271 (`inspect`), #270 (`check`, the verdict and its exit codes) |
| Used by | #291 (feature freeze), the Rust runtime (#272 and successors), the Python runtime |
| Precedent | `sigstore-conformance` (one client entrypoint per implementation, `xfail` per client); LangChain `standard-tests`; LangGraph `checkpoint-conformance`; the OpenTelemetry spec-compliance-matrix |
| Reference implementation | `sdk/npm/conformance` (this change) |

## Summary

`@prism-code/conformance` is one executable suite that every Prism runtime runs in CI.
A scenario is a JSON file: an action, its inputs and what the outcome must look like.
The suite never imports a runtime.
It invokes a runtime through an entrypoint shim, one process per step, hands it the step as a file and reads one JSON outcome back.
`prism-code` has its shim today; a runtime in another language adds one script and a declared list of expected failures, and gets a column.
`COMPLIANCE.md` at the repository root is the matrix of feature by runtime, generated from the suite's reports, and CI refuses it when it is stale.
A holo format revision cannot merge unless at least one scenario names it and loads an archive of it.

## Motivation

The repository has a conformance register, `CONFORMANCE.md`, and a Rust crate that runs one named test per id.
Both are about PrismPM.
They say nothing about a runtime that was not written here, and the docs already promise that every runtime passes `@prism-code/conformance` before release.
What a second runtime needs is a suite it can run without reading the Rust sources: a fixed set of inputs, a fixed expectation, and a way to say which cases it does not pass yet.

### Goals

1. One suite, one scenario format, run the same way by every runtime, from this package alone.
2. Scenarios cite the ids of `CONFORMANCE.md`. The suite adds no second id scheme.
3. A matrix a reader can trust: a cell is `+` only when every scenario of that feature passed on that runtime in the run that generated the file.
4. A merge gate on the format: a new revision in `@prism-code/spec` with no scenario fails CI.
5. Expected failures are declared, per runtime, in a file the suite reads. An expected failure that passes fails the run.

### Non-goals

- Replacing `crates/conformance`. The Rust crate stays the owner of every id; this suite is one more owning test it runs.
- Running a browser. The suite drives command lines. The browser closure is owned by HO-20.
- Signing, Rekor and the transparency log. Those cases need a network or a fixture bundle, and HO-28 to HO-30 own them.
- A Rust or Python shim. Each lands with its runtime. The matrix shows `-` for them until it does.
- Writing `COMPLIANCE.md` from CI with a push. The file is committed; CI regenerates it and fails on a difference, as it does for `CONFORMANCE.md`.

## Proposal

### How the two registries relate

There is one registry, `model/ids.toml`, rendered to `CONFORMANCE.md` and the SPEC appendix by `cargo xtask validate-model --write`.
A scenario in this suite has a `cites` member, the ids it exercises.
The suite's own tests refuse a scenario that cites an id the registry does not hold.
A scenario that exercises something no id covers is new, and the change that adds it registers the id in `model/ids.toml`, one Gherkin scenario in `features/suites/` and one named test in `crates/conformance`, as RP-08 requires.

This change registers two:

| Id | Suite | What it owns |
| --- | --- | --- |
| `HO-31` | `holo` | The executable suite, the entrypoint contract, and `prism-code` passing every scenario over every reference archive, with expected failures declared per runtime. |
| `RP-13` | `repository` | `COMPLIANCE.md` generated from the suite's reports and refused when stale, and the format-revision gate. |

Both Rust cases run the suite's owning Node test files with their exact test inventory, the way HO-20 and HO-29 run theirs.

Today's scenarios cite HO-20 (open an archive, verify it, dispatch its vectors), HO-29 (replay) and HO-31.
The verdict scenarios cite HO-31 alone, because the ids of `check` (AL-03, AL-05 and AL-06 in #270's branch) are not on this branch yet.
When that branch merges, the `cites` lists grow, and nothing else changes.

### The scenario

`scenarios/<id>.json`, schema `prism/conformance-scenario/1`.

```json
{
  "schema": "prism/conformance-scenario/1",
  "id": "replay-unchanged",
  "feature": "replay",
  "cites": ["HO-29", "HO-31"],
  "revision": "2026-10-23",
  "archive": "each",
  "statement": "A recorded history replays against the build it came from with no divergence, and the statement names the guest, the model and the runtime.",
  "steps": [
    {
      "action": "replay",
      "archive": "$archive",
      "histories": ["$history"],
      "expect": {
        "exit": 0,
        "result": {
          "schema": "prismpm/replay-result/1",
          "divergenceCount": 0,
          "events": { "$min": 1 },
          "statement": { "predicateType": "https://uor.foundation/prism/replay/v1" }
        }
      }
    }
  ]
}
```

| Member | Meaning |
| --- | --- |
| `id` | The scenario's name, lowercase words joined by hyphens, unique across the suite. |
| `feature` | One row of the matrix, from `features.json`. |
| `cites` | The registered ids the scenario exercises. At least one. |
| `revision` | The holo format revision the archives are of. The gate below reads it. |
| `archive` | `each`: run once per reference archive in `fixtures/*.holo`, with `$archive` and `$history` bound to it and its sibling `<name>.history.json`. A path: run once with that file. |
| `steps` | Entrypoint invocations, in order. Each is one process. |
| `steps[].expect` | What the outcome must match: `exit`, `result` and `error`. |

`archive: each` is how the second reference archive joins.
Payment Approval lands as two files in `sdk/npm/prism-code/fixtures/`, `scripts/vendor.mjs` copies them, and every `each` scenario runs over both.

An expectation is a pattern.
A primitive matches by equality.
An object matches when every member it names matches; members it does not name are free.
An array matches element by element, with the same length.
Six operators cover what equality cannot: `$type`, `$min`, `$max`, `$regex`, `$length`, and `$some`, which is true when one element of an array matches.
The matcher reports every mismatch by path, so a failed scenario says exactly what differed.

### The entrypoint contract

The suite invokes `<entrypoint> <step-file>` once per step.
The entrypoint is the runtime's shim: a program the runtime's authors write, which translates one step into the runtime's own command line and the command line's output into one outcome.
This is the sigstore-conformance shape: the suite owns the cases, the client owns the translation.

The step file, schema `prism/conformance-step/1`, holds the action and its inputs with every path absolute, and `scratch`, a directory the shim may write to.

| Action | Inputs | `result` on success |
| --- | --- | --- |
| `inspect` | `archive` | The `prismpm/holo-inspection/1` object. |
| `vectors` | `archive` | `{ "passed": n, "total": n }`. |
| `history` | `archive` | The `prism/history/1` object. |
| `replay` | `archive`, `histories` | The `prismpm/replay-result/1` object. |
| `check` | `archive`, `policy` (an object, or absent for the default policy) | The `prism/verdict/1` object, on exit 0 and 1. |

The outcome, written to stdout as one JSON object, schema `prism/conformance-outcome/1`:

```json
{
  "schema": "prism/conformance-outcome/1",
  "runtime": { "name": "prism-code", "version": "0.3.0" },
  "exit": 2,
  "result": null,
  "error": { "code": "archive.magic", "message": "expected magic HOLO" }
}
```

`exit` is the runtime's exit code for that action, as the runtime documents it (STABILITY.md section 6.1).
`result` is the runtime's JSON output, or null when it printed none.
`error` is the runtime's refusal, with its stable code when the runtime has one.

The shim's own exit code is 0 when it produced an outcome, whatever the runtime's exit was.
Any other exit, or stdout that is not one outcome, is a suite error for that scenario, reported as `error`, never as a pass.
A shim that does not know an action exits 2 with the reason on stderr.

`shims/prism-code.mjs` is the JavaScript shim.
It finds `prism-code` through `PRISM_CODE` (a path to its `bin/prism-code.mjs`), the repository sibling `sdk/npm/prism-code`, or the installed package, in that order, so that in the repository the runtime under test is the checkout and never an older copy installed up the tree.
`check` writes the policy to `scratch` and passes `--policy`; `vectors` reads the one line `prism-code vectors` prints.

### Runtimes and expected failures

`runtimes.json`, schema `prism/conformance-runtimes/1`, lists every runtime the matrix has a column for:

```json
{
  "prism-code": { "language": "JavaScript", "entrypoint": ["node", "shims/prism-code.mjs"], "xfail": "xfail/prism-code.json" },
  "prismpm": { "language": "Rust", "entrypoint": null, "xfail": "xfail/prismpm.json" },
  "prism-python": { "language": "Python", "entrypoint": null, "xfail": "xfail/prism-python.json" }
}
```

A runtime with no entrypoint has no report, and every cell of its column is `-` until it has one.
`prism-conformance run --runtime <name> --entrypoint <command>` overrides the entrypoint, so a runtime's own CI points the suite at a shim that lives in its repository.

`xfail/<runtime>.json`, schema `prism/conformance-xfail/1`:

```json
{
  "schema": "prism/conformance-xfail/1",
  "runtime": "prismpm",
  "xfail": { "history-is-vectors": "prismpm history is not implemented; #284 follow-up" },
  "notApplicable": { "vectors": "the Rust crate has no Wasm engine; vectors run through Node" }
}
```

| Member | Rule |
| --- | --- |
| `xfail` | Scenario id, or expanded id such as `replay-unchanged[Calculator]`, to a reason. The scenario runs. A failure is reported as `xfail` and the cell is `-`. A pass is `unexpected-pass`, and the run fails: the declaration is stale. |
| `notApplicable` | Feature to a reason. Its scenarios do not run, and the cell is `N/A`. |

An id that no scenario has is an error, so a renamed scenario cannot leave a dangling declaration.
`prism-code` declares nothing today.

### The report

`prism-conformance run --runtime <name> --report <file>` writes `prism/conformance-report/1`: the runtime, every expanded scenario with its feature, cites, status and failures, and the counts.
Statuses are `passed`, `failed`, `xfail`, `unexpected-pass`, `error` and `not-applicable`.
The command exits 0 only when no scenario is `failed`, `unexpected-pass` or `error`.
The report holds no clock and no host path, so two runs of the same runtime on the same fixtures write the same bytes.

### The matrix

`prism-conformance matrix <report>... --out COMPLIANCE.md` renders the file from the reports it is given, one per runtime; a runtime in `runtimes.json` with no report is rendered as having none.
`--check COMPLIANCE.md` renders and compares instead, and exits 1 when the committed file differs, after normalising line endings.

| Cell | Meaning |
| --- | --- |
| `+` | Every scenario of the feature passed on that runtime. |
| `-` | At least one did not: failed, errored, or expected to fail. Also every cell of a runtime with no report. |
| `N/A` | The runtime declares the feature not applicable, with its reason below the table. |

Below the table the file lists every scenario under its feature with its cited ids and its status per runtime, and every expected failure with its reason.
The header names the generator and says not to edit.

The features are the rows of `features.json`: `load`, `inspect`, `vectors`, `history`, `replay`, `check`, `world`, `attestations`.
Each feature cites the proposal or ids that define it, so the matrix is traceable without this document.

CI regenerates the file on every pull request that touches the suite, a runtime, the fixtures or the spec constants, and fails on a difference.
A contributor regenerates it with `npm run matrix` in `sdk/npm/conformance` and commits it with the change that moved a cell.

### The merge gate on the format

A holo does not carry its revision date.
`FORMAT_REVISIONS` in `sdk/npm/spec/constants.mjs` is the one declaration of every revision: the identifiers a reader recognizes it by, each copied from `Foundation.Holo.V1` in `stdlib/src/Foundation/Holo/V1/` (physical version, flags, section order, extension names, the manifest URI, the Core-Wasm contract, the HOLOVIEW version and the document schema names).
The spec's agreement test already fails when the stdlib's wire model and the table disagree, so a change to `Wire.lex.tex` that alters an identifier cannot merge without a new revision there.

`prism-conformance gate` closes the other half.
It fails unless

1. every key of `FORMAT_REVISIONS` is the `revision` of at least one scenario,
2. every scenario's `revision` is a key of `FORMAT_REVISIONS`,
3. for every revision, at least one scenario naming it has an `inspect` step whose expectation pins `revision` to that date, so the revision is evidenced by an archive that exists and reads, not by a label, and
4. every reference archive in `fixtures/*.holo` is loaded by an `each` scenario.

So a revision cannot be declared without a scenario, a scenario cannot name a revision that does not exist, and the scenario must load a real archive of it.
The CI job runs the gate on every change to the suite, the spec constants or the wire model.
`test/gate.test.mjs` holds the same rules against the committed tree and against a scenario set with a revision removed.

### The scenarios today

| Scenario | Feature | Cites | What it holds |
| --- | --- | --- | --- |
| `load-archive` (each) | load | HO-20, HO-31 | `inspect` exits 0; the footer, every label and the cross-references verify; physical version 4; revision `2026-10-23`. |
| `inspect-documents` (each) | inspect | HO-20, HO-31 | Eight sections; the manifest names the Core-Wasm contract and entry; the model and provenance carry their schema names and dual digests. |
| `vectors-pass` (each) | vectors | HO-20, HO-31 | Every acceptance vector passes. |
| `history-is-vectors` (each) | history | HO-29, HO-31 | `history` writes `prism/history/1` with at least one event, named after the archive. |
| `replay-unchanged` (each) | replay | HO-29, HO-31 | Replay of the recorded history diverges nowhere; the statement names the guest and the model. |
| `replay-divergence` | replay | HO-29, HO-31 | The changed model diverges on the recorded history: exit 1, every divergence with its index, request, expected and actual, and the statement records the count. |
| `replay-not-a-history` | replay | HO-29, HO-31 | A file that is not a history cannot be replayed: exit 2, `replay.history`. |
| `check-default-policy` (each) | check | HO-31 | The default policy requires P2; an archive of this revision is P0, with `provenance.statement`, `proof.statement` and `replay.statement` among its reasons: exit 1. |
| `check-minimum-p0` (each) | check | HO-31 | A policy with minimum P0 passes the same archive: exit 0. |
| `check-pinned-digest` (each) | check | HO-31 | A policy pinning a wrong guest digest fails with `policy.digests.guest`: exit 1. |
| `check-no-verdict` | check | HO-31 | A file that is not a holo gets no verdict: exit 2, `archive.magic`. |
| `world-declared` (each) | world | HO-31 | `inspect` reports the world: its contract, name, package, dual digests, imports, exports, `conforms` and `pure`. |
| `world-pure-policy` (each) | world | HO-31 | The pure-world policy, `effects: []`, passes. |
| `world-pinned-contract` (each) | world | HO-31 | A policy pinning another contract fails with `world.hash`. |
| `attestations-absent` (each) | attestations | HO-31 | An archive of revision `2026-10-23` carries no attestation section: `attestations` is empty. |
| `attestations-present` | attestations | HO-31 | An archive with an attestation section lists its statement by predicate type and subjects, and `check` reaches no verdict for it, because no revision admits the section yet: exit 2. |

The archive with an attestation section is derived, not reference evidence: `scripts/vendor.mjs` appends the unsigned provenance bundle fixture of #268 to the Calculator with `appendAttestations`, and `--check` holds the derived bytes.
When #267's wire change admits the section into a revision, that scenario's `check` expectation changes in the same pull request, which is the gate working as intended.

### The package

`sdk/npm/conformance`, `@prism-code/conformance` 0.1.0, no dependencies, Node 22 or newer, like its siblings.

| Path | What it is |
| --- | --- |
| `bin/prism-conformance.mjs` | `run`, `matrix`, `gate`, `list`. |
| `src/` | `scenarios.mjs` (load, validate, expand), `runner.mjs` (spawn the entrypoint, match), `match.mjs`, `report.mjs`, `matrix.mjs`, `gate.mjs`, `runtimes.mjs`. |
| `scenarios/`, `features.json`, `runtimes.json`, `xfail/` | The suite as data. |
| `shims/prism-code.mjs` | The JavaScript shim. |
| `fixtures/` | Copies of the reference archives and histories, the changed model, and the derived attested archive, written by `scripts/vendor.mjs` and held by `--check`. |

The package carries its fixtures because a runtime in another repository installs it from npm and must find them.

### Test plan

| Where | What it holds |
| --- | --- |
| `test/scenarios.test.mjs` | Every scenario file validates; ids are unique; every cited id is in `CONFORMANCE.md`; every feature is declared; `each` expands over every reference archive. |
| `test/match.test.mjs` | Each operator, subset matching, arrays by length, and the mismatch paths. |
| `test/suite.test.mjs` | The suite against `prism-code`: every scenario passes, the report is deterministic, and a scenario made to fail reports its mismatch. |
| `test/shim.test.mjs` | The shim contract: each action's outcome shape, an unknown action, a missing step file. |
| `test/xfail.test.mjs` | A declared failure that fails is `xfail`; one that passes is `unexpected-pass` and fails the run; a declaration naming no scenario is an error; `notApplicable` skips the feature. |
| `test/matrix.test.mjs` | Cells from statuses; a runtime without a report; `--check` passes on the committed file and fails on a changed one. |
| `test/gate.test.mjs` | The four rules, on the committed tree and on a tree with a revision or a scenario removed. |
| `.github/workflows/npm-conformance.yml` | Unit tests on Node 22 and 24 on three systems; the suite against `prism-code`; `COMPLIANCE.md` current; the gate; vendored fixtures current. |

HO-31 runs `suite`, `shim` and `xfail`; RP-13 runs `matrix` and `gate`, each with its exact test count.

### Graduation criteria

- Experimental (M3). This document, the package, the JavaScript shim and `COMPLIANCE.md` are merged, and the gate runs in CI.
- Beta (v0.6 freeze, #291). The Rust shim exists and its column is generated from a real report; the scenario and outcome shapes have gone one release without change.
- Stable (1.0). Any change to the scenario, step or outcome shape is a `/2`.

## Precedent

- `sigstore-conformance`: a GitHub Action that drives a client through an entrypoint script with a fixed command grammar, and lets a client list the tests it expects to fail. Copied: the entrypoint, the per-client expected failures, the strict handling of an unexpected pass. Not copied: passing every input as flags; a step file carries structured inputs better, and a policy is an object.
- LangChain `standard-tests`: a base test class every integration subclasses. Not copied: it needs the runtime's language. The suite here is language-neutral by construction.
- LangGraph `checkpoint-conformance`: one suite run against every checkpointer. Copied: parametrisation over the implementations and the fixtures.
- OpenTelemetry spec-compliance-matrix: a hand-maintained table of feature by language. Copied: the shape, `+`, `-`, `N/A`. Not copied: hand maintenance; here the table is generated and checked.

## Alternatives

- Import each runtime as a library. Rejected: it binds the suite to one language and one process; the entrypoint lets Rust and Python run the same scenarios unchanged.
- One scenario per cited id. Rejected: an id such as HO-20 holds more than one observable behaviour, and the matrix is read by feature.
- Let the matrix be written by CI with a commit. Rejected: the repository commits generated documents and checks them, and a bot commit on every pull request is noise.
- Name the archive's revision in the archive. Out of scope: #263 chose a revision table over a carried date, and the gate works with the table.
- Put the fixtures behind a path option instead of copying them. Rejected for the published package: a runtime in another repository has no `../prism-code`.

## Unresolved questions

1. Where does the Rust shim live: `crates/prismpm` as a binary, or a script beside the suite? Recommendation: a binary in the crate, so it versions with the runtime, run through `--entrypoint`.
2. Should `notApplicable` be allowed for a feature a runtime could implement? Today nothing stops it; the reason is published, which is the check.
3. The `check` scenarios cite HO-31 alone until the ids of #270 are on main. Should the suite accept a cite of an id that is registered on another open branch? Recommendation: no; the cite is added when the id is.
