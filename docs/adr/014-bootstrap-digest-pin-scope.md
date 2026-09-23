# ADR-014: Partial Bootstrap Digest Pin Scope (PP5008)

## Status
Proposed

## Context
The bootstrap verifier is intended to fail closed on unpinned required executables, but the pin table covers only a minority of the tools PrismPM invokes:

- PP5008 ("required executable ... has an unpinned digest") is raised in crates/prismpm/src/verification.rs:275-304.
- Per-executable SHA-256 pins exist for only four name families (verification.rs:286-291):
  - `lake`/`lean` → `ELAN_PROXY_SHA256`
  - `rustc`/`rustfmt` → `RUSTUP_PROXY_SHA256`
  - `timeout` → `TIMEOUT_SHA256`
  - `node` → `NODE_SHA256`
  - every other name resolves to `None` and is accepted without a digest check.
- Tools invoked without a per-executable pin at bootstrap: `cargo`, `npm`, `docker`, `wasm-bindgen`, `wasm-tools`, `wasmtime` (and `curl`, `git`). Their pinning today exists only at the image level in .devcontainer/Dockerfile:54-99 (rustup-init 1.28.2 sha `20a06e64…`, Node 22.23.2 sha `d60acfe0…`, elan 4.2.3 sha `df0b2b3a…`, just 1.57.0 sha `45b54809…`, cargo-deny 0.20.2 sha `9f12ed4c…`), and `tools.lock` is x86_64-only.
- The proxy-based pins are also fragile across environments: `lake`/`lean` and `rustc`/`rustfmt` under elan/rustup canonicalize (verification.rs:269) to the real toolchain binaries (`~/.elan/toolchains/…`, `~/.rustup/toolchains/…`), whose digests cannot equal the pinned proxy-binary constants; the check is therefore sensitive to PATH and system layout, not merely to tool version. This is the class of failure PP5008 exhibits when PrismPM runs outside the pinned devcontainer (including this environment).

Hidden assumption: "the toolchain is pinned" means every tool PrismPM may execute is digest-bound at the point of use. Today only image-level pinning covers most tools, and the host-level check's coverage is 4 of the ~12 required tool families.

## Decision
**Define a two-layer digest policy and make the bootstrap check cover the full required-executable set:**

1. **Layer 1 — per-executable bootstrap pins (required for every tool the host invokes directly)**: extend the match in verification.rs:286-291 to `cargo`, `npm`, `docker`, `curl`, `git`, `wasm-bindgen`, `wasm-tools`, and `wasmtime`; absent pin ⇒ PP5008 failure. Derive the required set from the model's tool registry instead of a hard-coded match arm.
2. **Layer 2 — image-level pins (devcontainer authority)**: keep .devcontainer/Dockerfile and `tools.lock` as the authoritative inventory, but record `tools.lock` single-arch (x86_64) as a Phase 2 dual-arch item so arm64 hosts do not inherit an unpinned toolchain.
3. **Fix the proxy-boundary fragility**: for `lake`/`lean`/`rustc`/`rustfmt`, verify the elan/rustup proxy digest on the proxy path itself (before canonicalization) and the toolchain binary digest from the elan/rustup database records; do not canonicalize across the proxy boundary.
4. Document that PP5008 failures outside the pinned devcontainer are environment/fragility artifacts until Layer 1-3 land, and tie any PP5008 waiver to ADR-013's evidence policy.

## Consequences
- **Positive**: the "unpinned digest" guarantee holds for every executed tool, not four families; PP5008 behavior becomes environment-stable.
- **Negative**: more constants to maintain; docker and cargo digest constants must be version-bumped with each devcontainer change; arm64 off-line bootstrap is deferred to Phase 2.
- **Action**: extend the pin table; generate tools.lock for arm64; add conformance cases for unpinned `cargo`/`docker`/`wasm-tools` and for proxy-boundary behavior.

## References
- `crates/prismpm/src/verification.rs:254-305` (PP5008, pin table at :286-291)
- `.devcontainer/Dockerfile:54-99`, `tools.lock`, `scripts/bootstrap-verify.sh`
- VERIFICATION.md § "Bootstrap verification" (:1491-1494), ADR-005 (environment vs implementation classification)