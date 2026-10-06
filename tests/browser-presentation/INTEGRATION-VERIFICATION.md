# Integrated status owner

Port of PR50 onto the current compiler-custody and browser-environment stack.
Presentation model, wire tags, structural limits and the stdlib archive are
unchanged. No semantic model or TextChoice prerequisite is silently imported.

The private adapter retains its connected atomic status node and inserts text
only after mounting its role/live semantics. Identical text is not rewritten.
Mount/removal failure terminates ownership and removes listeners.

DK23 owns 37 checks: wire 5, DOM 5, replay 2, provenance 7 and generated SDK owner
18. DK15 retains 34; DK29 retains the separate 35-check exporter-seed owner.
The compiler uses the current exact compiler-only input inventory and closed
family API, with the wider presentation inputs checked separately. Original
compiler substitution and terminal-retirement controls remain required.

Component run integrated-status-owning-v1: 49/49 passing, no omissions, 45.394s,
exit 0/no OOM. Real Chromium, Firefox and WebKit execute all private journeys;
all actual adapter mutants are killed. The complete original SDK-input harness
and package/Wasm/provenance/replay owners also pass.
Log SHA256: b6027fef281ee15d8267bf9db481256ace4c2d1b225d87454f2bdd616dda784b.
Environment: existing Noble browser image 60226bc7, exact npm inputs copied
from pinned devcontainer f061023a. This is component evidence, not accepted
SDK, regenerated/native/Wasm execution or installed downstream qualification.

Pinned devcontainer formatting of all four owned crates passes. A broader
cargo fmt --all also visits excluded generated dependencies and reports their
existing unformatted generated code; that diagnostic is retained, not corrected
by editing generated sources.

Required before acceptance: exact-head independent review, current native CI,
complete 37-check generated owner with unchanged five compiled source mutants
and every 64-MiB maximum, then unchanged full V&V and immutable installed SDK/
downstream qualification. No component check satisfies those obligations.
