# ADR-009: Model Command Register Count Reconciliation

## Status
Proposed

## Context
Three documents assert "26 model-defined commands", but the model register and the executable command hierarchy say otherwise:

- README.md:106 claims "26 model-defined commands verified".
- VERIFICATION.md:1499 claims the lifecycle suite "asserts full coverage of all 26 model-defined CLI commands against `model/commands.toml`", and VERIFICATION.md:1508 claims shell completions cover "all 26 model commands".
- VERIFICATION.md:1500-1503 then enumerates 28 distinct command names (`fetch`, `build`, `push`, `pull`, `inspect`, `run`, `plan`, `deploy`, `status`, `rollback`, `destroy`, `clean`, `verify`, `check`, `export-browser`, `lock`, `authority`, `conformance`, `verify-release`, `prepare-promotion`, `sign`, `sign-evidence`, `verify-signature`, `promote`, `backup`, `restore`, `template`, `finalize-contract`).
- `model/commands.toml` registers **29** commands via `[[command]]` (line 3 through 171), including `completion`, which is omitted from the VERIFICATION.md:1500-1503 list.
- `crates/prismpm/src/cli.rs:39` (`enum Commands`) exposes 29 visible subcommands plus two `hide = true` modes (`Acceptance` at cli.rs:255, `Oracle` at cli.rs:285).
- The runtime completion path enforces register/executable equality: cli.rs:428-432 fail with PP9001 "`executable commands disagree with model/commands.toml`" whenever the visible subcommand names and the register diverge.
- `tests/controller_cli_lifecycle.rs:229-251` only asserts a 15-command *subset* is present in the register; it does not assert "full coverage of all 26" or any other total count.

The "26" figure is therefore contradicted by the register (29), by the CLI (29 visible), by the docs' own enumeration (28), and is not what the named test actually verifies (subset of 15).

## Decision
**Adopt `model/commands.toml` (29 commands) as the single normative command register**, and make every count claim derivable from it:

1. Update README.md:106 and VERIFICATION.md:1499,1508 to state 29 model-defined commands.
2. Add the missing `completion` command to the enumeration in VERIFICATION.md:1500-1503.
3. Strengthen `tests/controller_cli_lifecycle.rs` to assert the full register-to-CLI equality (the contract already enforced at runtime by cli.rs:428-432 for completions), not a 15-name subset, so test and doc totals cannot drift silently again.
4. Keep `Acceptance` and `Oracle` hidden: they are execution modes, not model commands. Document in the register spec that the 29-command register covers only visible commands.

## Consequences
- **Positive**: command-count claims become machine-checkable against one register; PP9001 catches future additions that doc counts fail to track.
- **Negative**: "26", "28", and "29" appear in historical releases and release notes; no migration of command names is implied, only count and coverage language.
- **Action**: apply doc edits; extend the lifecycle test to full equality; add a conformance case asserting the doc count equals the register count.

## References
- `model/commands.toml` (29 `[[command]]` entries)
- `crates/prismpm/src/cli.rs:39`, `:254-285` (hidden commands), `:417-433` (register equality)
- `tests/controller_cli_lifecycle.rs:229-251`
- README.md:106, VERIFICATION.md:1499-1508