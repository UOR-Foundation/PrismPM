# DK-21 declaration verification

Scope: checked LexLean declarations, closed projection and early `PP2011` build
refusal. The fixture is not a browser runtime or Foundry acceptance oracle.

Source SHA-256:

- Model: `3d55c1e1ea39a429a65bbb0c35971516a5fb3d1bcf7b5eeb099e64c9d48925a6`
- Probe: `23cefd329823d56bd8b03664f0bd5c3fcd86f802be0f3a1fd7c0fba1c95cce23`

Devcontainer checks (2026-09-20):

- Registered `conformance_dk_21`: 1 passed; independent source snapshots,
  schema/canonical closure, declaration mutations and no-output build refusal.
- `browser_application` integration: 1 passed; `text_application`: 8 passed.
- `repo-model` library: 12 passed.
- Scoped Clippy (`-D warnings`), Rust formatting and `git diff --check`: passed.
- Normal `xtask validate-model --write` and readback: passed.
- `xtask validate`: passed, including all 158 infrastructure tests.

No kernel, generated application runtime, credential custody, durable recovery,
effective grants or deployment is accepted by these declaration checks. Combined
SDK stdlib/golden/runtime acceptance remains an integration gate.
