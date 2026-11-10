# Prism Factory

The factory decides, for every obligation of a release, whether it is already
proved, must run, is refuted, or is blocked, and in what order to run what is
left. Those decisions are a Prism model checked by the Lean kernel and exported
by LexLean to certified Rust. Everything else is a thin host. The same certified
core runs on a laptop, on any CI runner, and in a browser (GitHub Pages).

GitHub Actions is only one place the host runs. It supplies a runner, a checkout
and a branch for receipts. It decides nothing.

## Why

From 2026-10-31 to 2026-11-10, `release.yml` ran 15 times on `release/v0.4`.
None of the runs passed, and together they took about 190 hours. Each fix
invalidated every stage, because every stage was keyed on the whole tree. The
factory keys each obligation on its own declared inputs instead, and it
measures how much each change re-runs:

```text
$ prism-factory plan --rev <before>   # and again for <after>, then compare identities
48adc65b -> 45e4819b (#579, 13 files):  18 of 21 obligations change
45e4819b -> d8bf55ed (#580,  2 files):  18 of 21 obligations change
d8bf55ed -> 3666dc0f (#581, 10 files):  18 of 21 obligations change
```

The 18 obligations read the whole source. The SDK image packs the entire Git
revision into itself (`scripts/sdk-vv-inputs.mjs` `buildClosure`), and source
V&V builds that image (`vv.sh --with-sdk`). So any change, even to a script
outside the product, rebuilds the image and everything after it. The model
states this and the kernel checks it
(`eighteen_obligations_read_the_whole_source`). The next step is to split that
image so each obligation reads only what it uses (see [Next](#next)).

## Layout

| Path | What it is |
| --- | --- |
| `model/src/Factory/Engine.lex.tex` | Obligations, identity, routing, priority, and their theorems |
| `model/src/Factory/PrismPM.lex.tex` | PrismPM's release obligation graph, with theorems over it |
| `model/src/Factory/Workflow.lex.tex` | The GitHub workflow, as a projection of the model |
| `model/src/Factory.lex.tex` | The production roots `plan`, `reweigh`, `workflow` and their text protocol |
| `std/` | LexLean's standard library (`Buffer`, `Sha256`, `Text`), byte-identical to LexLean `e0b74b81` `stdlib/` |
| `certified/` | The production bundle: certified Rust packages, certificates, attestation |
| `host/` | The thin host: CLI (`prism-factory`), WebAssembly ABI, tests |
| `site/index.html` | The GitHub Pages page that runs the wasm core |
| `factory.sh`, `check.sh` | Entry points for a laptop or CI, and the `factory-model` obligation |
| `../.github/workflows/factory.yml` | Generated. `prism-factory workflow --check` refuses drift |

## What the kernel proves

- **Routing.** A passing receipt proves its identity. A hermetic failure is
  refuted and not re-run on the same inputs. A failed fetch is observed again,
  because the network is not an input. A later pass outweighs an earlier
  failure. An obligation whose upstream is not proved is blocked, even when it
  has its own receipt.
- **Priority.** Weights are a 16-level type, so out-of-range weights can't be
  represented. A failure never lowers priority and a pass never raises it.
  Routing takes no weight: priority changes only the order.
- **The PrismPM graph.** Names are distinct. Every need is declared before it
  is needed, so the graph is acyclic. Every obligation declares its inputs and
  its run. No obligation is arbitrary: each one is needed, directly or
  transitively, by `release`. The theorems also record how many obligations
  still run on the legacy Actions adapter (19) and how many read the whole
  source (18). These are the numbers to bring down.

The kernel can't evaluate `Bytes` equality, so SHA-256 and the text protocol
are held by a differential instead. `host/tests/core.rs` checks an identity
against Python's `hashlib`, byte for byte, the same way LexLean's stdlib holds
its byte functions.

## Identity

An obligation's identity is the SHA-256 (in hex) of:

```text
prism-factory/obligation/1
name <name>
kind hermetic|fetch
environment <environment>
run <run>
input <prefix>                 one line per declared input, in order
needs <name> <identity>        one line per need, in order
closure <sha256 of the closure>
```

The closure is one line `file <mode>:<object> <path>` for each tracked entry
whose path starts with a declared input, in Git tree order. Changing a file,
an input, the command, the environment or any upstream identity changes the
identity. Nothing else does.

## Running it

```sh
prism-factory/factory.sh plan              # the certified plan for HEAD
prism-factory/factory.sh run               # run every `observe` obligation of environment `host`
prism-factory/factory.sh request > r.txt   # the request, to plan offline or in the page
prism-factory/factory.sh site out/         # the Pages site: index.html + factory.wasm
```

`run` copies each obligation's closure, and nothing else, into a fresh
directory outside any repository and runs it there. An undeclared input
therefore fails the run instead of producing a false receipt. The first local
run caught exactly this: a closure inside the checkout picked up the root Cargo
workspace. Receipts go to `target/factory/receipts`, and in CI to the
`factory-receipts` branch, which is written only by pushes to `main`. The plan
is repeated until it reaches a fixed point, and a second run on the same tree
runs nothing.

The page takes a repository and a revision. It reads the tree from the GitHub
API and the receipts from the `factory-receipts` branch, and plans in the
browser in about 0.2 s. It also accepts a request file. To publish it, set
Pages to deploy from GitHub Actions and set the repository variable
`FACTORY_PAGES` to `true`.

## Changing the model

```sh
cd prism-factory/model
lexlean verify --all                                         # kernel-check every theorem
lexlean production build --request ../production-request.json
rm -rf ../certified && cp -r .lexlean/production/<result> ../certified
cd ../host && cargo test --release
./target/release/prism-factory workflow --write ../../.github/workflows/factory.yml
```

`check.sh` (the `factory-model` obligation) replays the bundle without Lean.
Where the pinned Lean is installed, it also re-verifies the model and requires
the re-export to equal `certified/`.

## Limits

- `host` obligations run with the runner's network and user. The closure is
  enforced. Network and uid isolation are not enforced yet, and the receipt
  records `network host`.
- `actions` obligations are the existing `release.yml` jobs. The factory
  identifies and routes them, but it doesn't run them yet.
- A receipt is trusted because of who can write the `factory-receipts` branch.
  Signing receipts with the release identity is a follow-up.

## Next

1. **Split the SDK image** into toolchain and oracle stages keyed only on the
   lock files and oracle sources they read, with the PrismPM source as the last
   layer. This takes the image-derived obligations off the whole source.
2. **Move each `actions` obligation to a hermetic `host` command**, one at a
   time, so that `eighteen_obligations_read_the_whole_source` and
   `nineteen_obligations_still_run_on_the_legacy_adapter` fall to zero.
3. **Ship the component through the template**, so every repository Prism is
   installed into gets its own graph and its own receipts.
