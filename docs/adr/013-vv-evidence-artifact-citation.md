# ADR-013: `target/vv-evidence.json` as Cited Release Evidence

## Status
Proposed

## Context
Multiple release documents present `target/vv-evidence.json` as the completed, verifiable receipt for Gate 15 and the full vv gate run:

- README.md:101 (Gate 15 row, "Verification" column): `` `target/vv-evidence.json` ``.
- RELEASE-STATUS.md:9-13: "Gate 15 (`cargo xtask package-api`) passes completely in the clean devcontainer ... The full-pass receipt `target/vv-evidence.json` records passing gates 1–15 bound to the accepted source commit."
- VERIFICATION.md:655 ("`just vv` run writes `target/vv-evidence.json`"), :1061 ("receipt `target/vv-evidence.json` binds all 15 gates as passed"), :1695.

But the artifact cannot serve as independently verifiable evidence:

- It is transient by design: xtask/src/main.rs:415 removes any prior `target/vv-evidence.json` at the start of each `run_vv` so it never survives a run and is never committed (it lives under gitignored `target/`).
- It is produced only by the docker-requiring vv pipeline (`scripts/vv.sh`), which cannot run in this repository's host environment (no docker, no pinned devcontainer), so it is not reproducible here.
- VERIFICATION.md:1049 itself records that the original vv run "exited 1 without `target/vv-evidence.json`; no full-pass or release receipt" — i.e. the very receipt README.md:101 presents as completed evidence was, at the time, absent.
- Separately, README.md:118's "Diagnostic Boundaries" row uses a branch name (`fix/issue-14-diagnostic-boundary-coverage`) as its Verification column instead of a test or schema, the same evidence-class problem (citing something that is not a verifiable artifact).

## Decision
**Stop presenting `target/vv-evidence.json` as stable release evidence; qualify it and point at reproducible artifacts:**

1. Treat `target/vv-evidence.json` strictly as a run-local artifact bound to source commit and log digest, and state in RELEASE-STATUS.md the exact invocation, environment (pinned devcontainer), and its non-committed status whenever it is cited.
2. Change README.md:101's Gate 15 "Verification" citation to the reproducible gate identity (the xtask gate driver / `prismpm/sdk-security-disposition/1` OCI disposition) instead of the transient file, mirroring the remediation already documented at VERIFICATION.md:1049.
3. Change README.md:118's Verification column from the branch name to the conformance case or test that covers PP1001–PP1003.
4. Commit an immutable digest of a clean full-pass vv run (e.g. `prismpm/vv-evidence/1` referencing the source commit and log digest) if a committed receipt is required.

## Consequences
- **Positive**: evidence claims become reproducible or explicitly transient; no future reader can mistake a run artifact for committed evidence.
- **Negative**: README.md/RELEASE-STATUS.md lose a convenient shorthand; VERIFICATION.md:655 etc. need parallel qualification.
- **Action**: apply doc edits; add a conformance case rejecting acceptance claims that cite non-committed paths under `target/`.

## References
- README.md:101,118; RELEASE-STATUS.md:9-13; VERIFICATION.md:655,1049,1061,1695
- `xtask/src/main.rs:415-419,633-637`, `scripts/vv.sh`
- ADR-008 (same README.md:118 row qualification)