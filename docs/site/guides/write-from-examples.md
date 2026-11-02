---
title: Write from examples
description: Author an application from examples, intent and types, build it, and read the report.
available_in: "0.6"
---
This guide is the main way to write a Prism application. You give `build` examples of what the application does, sentences about what it must always and never do, and types for its input and output. An agent writes the model, the Lean kernel checks every sentence, and `build` fails by name on any sentence that is not proved.

This is experimental. Pass `--unstable-author` to `build`, and pin the exact version you use. Without the flag, `build` is unchanged.

## The four levels

Prism has four ways in. Pick the lowest one that does the job.

- Level 0: run someone's holo. No model. Just `pull`, `check` and `run`.
- Level 1: compose in `prism.toml`. Name the holos you depend on and the capability set. No code.
- Level 2: examples and intent. This guide. You write data and sentences, an agent writes the model.
- Level 3: write the model directly. See [Write the model](/docs/guides/write-the-model). A project with a hand-written `model/` and no `intent.md` or `examples/` builds as it always has.

## The five inputs

`build` reads five inputs. Everything else is generated.

| Input | What it holds |
|---|---|
| `prism.toml` | The manifest: name, version, where the model comes from, the capability set. |
| `examples/` | One JSON file per example: an input and its expected output. These are the holo's acceptance vectors, in the order of their file names. |
| `intent.md` | One sentence per line. Each becomes a theorem. |
| `types/` | Optional. The input and output types, in TypeScript, JSON Schema or Pydantic. |
| `prismpm.lock` | The lockfile: the SDK image digest, the model, spec and world digests, and the version of every tool in the SDK image. |

For calculator, `prism.toml` names the application:

```toml
[application]
name = "calculator"
version = "0.1.0"
description = "Adds two numbers."
```

`examples/add.json` holds the one example. The example is the bytes on the wire: a string is its UTF-8, `{"hex": "ff00"}` is those bytes, and any other JSON value is its canonical form.

```json
{"input": {"op": "add", "a": 2, "b": 3}, "output": 5}
```

`intent.md` holds two sentences. A sentence ends with a period and uses exactly one of always, never or only:

```md
add always gives the same result when a and b swap.
The result never exceeds 32 bits.
```

`types/calculator.ts` holds the input and output types. The file extension says whether it is TypeScript, JSON Schema or Pydantic:

```ts
export type Input = { op: "add"; a: number; b: number };
export type Output = number;
```

Write each sentence the way you would state the rule to a colleague. One rule per sentence. Use the operation name from your examples. A sentence is proved for every input the types allow, not only for the examples, so narrow the types when the rule needs it.

One form is also checked against your examples before any agent runs. Put the bytes in backticks: `X never returns \`error\`.`, `X always returns \`ok\` when the input contains \`add\`.`, or `X returns \`overflow\` only when the input contains \`multiply\`.`. If an example contradicts one, the build fails at once and names both.

## What build does with them

`build` runs the same steps every time.

- Reads the examples, the intent and the types, and fails if an example contradicts a sentence.
- Has an agent write the model, unless `model/` already comes from this exact spec. The agent's answer goes through the SDK image at the digest in the lockfile: the compiler and the provers live there, not on your machine. Whatever fails goes back to the agent, up to the budget.
- Requires the holo's acceptance vectors to equal your examples, and replays every one through the compiled program.
- Requires a theorem the kernel accepted for every sentence. What the agent says it proved counts for nothing.
- Writes the model to `model/` and pins its digest and the spec digest in the lockfile, last, once everything passed.

Commit `model/` with the rest of the project. The lockfile pins it, so the next build never calls the agent unless the spec changed. A reviewer can read `model/`, and `model/author.json` puts each sentence beside the statement the agent proved for it. Read that file: the kernel proves the statement, and only a reader can tell that the statement says what your sentence says.

The default agent needs `npm install @anthropic-ai/sdk` and an API key. `--author-command <path>` runs any other agent, and `--author-model <id>` chooses the model recorded in the lockfile. `--max-iterations`, `--author-timeout` and `--author-max-tokens` bound what a build may spend.

## Read the report

Add a third sentence to `intent.md`: "add never returns a negative number." Then build:

```bash
prism build --unstable-author
```

```text
Proving
  add never returns a negative number.  UNPROVED
    agent: a = -1 and b = 0 is an input the types allow, and it gives -1.
Build failed: 1 intent sentence unproved (exit 1)
```

Read it top to bottom. An unproved sentence comes with the agent's own explanation. The explanation is a lead to check, not a proof. No holo was accepted and `model/` is as it was. Exit code 1 says the build failed, not that it could not run. A build that gets to the kernel lists every sentence, each with `checked` or `UNPROVED`.

To check a holo against its project later, without an agent, run `check` with the flag. It proves the spec, the model and the holo belong together, and it prints the sentences:

```bash
prism check calculator.holo --unstable-author
```

You have two fixes. Narrow the types so `a` and `b` cannot be negative, and the sentence becomes provable. Or delete the sentence, because it was never true. There is no third option. A holo only exists when every sentence is a theorem.

## Iterate with dev

`dev` watches the inputs, rebuilds and re-checks on every save, and serves a local studio:

```bash
prism dev
```

```text
Watching examples/, intent.md, types.ts, prism.toml
Studio at http://localhost:4100  tabs: Check, Replay
Built calculator@sha256:3f9c… in 2.1s  Proof: 2 theorems checked  Level: P2
```

The Check tab shows the verdict and the reason for each line. The Replay tab shows every example and whether it still replays. Add an example, save, and watch both tabs update.

## Next steps

- [Verify and inspect](/docs/guides/verify-and-inspect)
- [Run and gate](/docs/guides/run-and-gate)
- [Write the model](/docs/guides/write-the-model)
