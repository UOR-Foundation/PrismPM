# DK-22 compiler verification

Scope: private, unaccepted compiler artifacts. Public browser builds retain
`PP2011`; no View, credential custody, durable recovery or deployment is accepted.

Devcontainer, 2026-09-20:

- Registered `conformance_dk_22`: passed in 178.28 s; all three owning tests ran.
- Two independent checked-source, complete axiom-audit and kernel/export builds
  produced identical retained artifacts for six root/budget tuples and seven roles.
- Each build executed 23 native vectors with `std` and `no_std`, 21 Wasm vectors,
  five exact input/output maxima (including 2 MiB), six input/memory limit checks,
  and two valid-input output overruns rejected by actual generated Wasm.
- Every retained-file omission/change and coherent artifact, proof, root, policy
  or budget substitution rejected independent replay.
- Actual subprocess negatives rejected incompatible toolchain selection and
  executable PATH Cargo substitution. No synthetic compiler success was used.
- Source normalization/changed-source checks and all three scheduler tests passed.
- Final `xtask validate`: passed, including 158 infrastructure tests. Normal
  register generation/readback, scoped Clippy (`-D warnings`), formatting and
  `git diff --check` passed.

Independent review identified toolchain binding, scheduler and output-overrun
coverage gaps; all three were corrected before the registered gate above.
