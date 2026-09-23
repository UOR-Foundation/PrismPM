# Portable oracle build policy

Verified source: `9ab9841fa181297fdf4a3f058da8ffc7952ab6fd`, 2026-09-20.
Execution: non-root PrismPM devcontainer, private Cargo target, offline.
Oracle image: `127.0.0.1:5000/prismpm-openid-oracle-test@sha256:db5ad1aaac7b1d847a5ec6fc2ed33688fc46dc31b449f7f5f2e7b83e70e7228d`.
Set `PRISMPM_TEST_SDK_IMAGE` to that image and `CARGO_TARGET_DIR` to the private
worktree's target; outer Cargo used jobs=1, dev debug=0, incremental=0.

```sh
cargo test --locked --offline -p prismpm --lib \
  controller::release_tests::alternate_release_verification_is_bound_to_the_selected_build \
  -- --exact --nocapture
```

The complete owner passed: 1 test, zero failures/ignored, 718.19 seconds.
It verifies both named releases, all seven required system oracles, mismatched
build/selector rejection, evidence mutations, producer deletion, and both
source-free receiver exports.

The actual oracle subprocess used jobs=2, dev debug=0, incremental=0, and offline
mode. Its cold build took approximately 162.219 seconds; the second dependency-
cached build took 17.586 seconds (250 ms observation interval). The existing
300-second deadline, source, lock, features, and oracle assertions were unchanged.
Minimum observed free disk was 15,610,929,152 bytes; available host memory was
9,363,390,464 bytes. No resource stop, skipped check, or retry occurred.

Portable boundary/environment regressions: 3/3 passed, including exact child
test-name/count/zero-skip assertions. Scoped library/test Clippy passed with
warnings denied; formatting and diff checks passed.

Retained local evidence (ignored `target/`):

| File | SHA-256 |
| --- | --- |
| `portable-oracle-owner.log` | `2d52bca724b7e5232079796bb46bf545f8cccc55dcfd19bc744fb71999dc6f0b` |
| `portable-oracle-owner.json` | `aa5588486c3d6682dd1cb784962918b8eceee7f1717b061861da77ae0da6f4b3` |
| `observe-owner.mjs` | `892267cf2f873ab025e13d205c50604df69c2c62c0cd8b0e9a0fa2a8c4143256` |

The full concurrent CI workload remains to be rerun. This gate does not accept
the installed SDK, effectful public application runtime, or Foundry deployment.
