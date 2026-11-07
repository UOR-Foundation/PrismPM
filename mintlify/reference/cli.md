---
title: CLI reference
description: The prism-code command that ships today, then every prism verb, its flags, its exit codes and its usage.
available_in: "1.0-beta"
---

<Note>This page describes Prism 1.0-beta.1, which has not shipped. [Release status](/reference/release-status) says what works today.</Note>


Source: `sdk/docs/cli.json` 2026-10-30, the stand-in for `prism --help --json`.

`prismpm` is an alias of `prism` and is kept through every 1.x release.

## The command that ships today

Install `prism-code` with `npm install prism-code`, or run it with `npx prism-code <verb>`. Node.js 22 or later. build, verify, axioms and conformance also need Docker and a `prismpm.lock` in the current directory; `build --check` and `verify --lean` run on the hosted verifier without them.

| Verb                 | Usage                                                                                                                                                                                                                                                                                              | What it does                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `run`                | `prism-code run <app.holo> [<input>] [--input <text>] [--rm] [--port <n>] [--policy <file>] [--allow-unverified] [--capabilities <file>]`                                                                                                                                                          | Check the holo against the policy. With an input, this is one run as a container: dispatch the input, print the output and exit with the container's code, keeping the container until `rm` unless `--rm`. Without one, serve its view on a local port; a POST to `/_hologram/intent` dispatches one request. A holo that fails check is refused unless `--allow-unverified`, which warns on every start.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `rm`                 | `prism-code rm <container>...`                                                                                                                                                                                                                                                                     | Remove containers, by id or unique id prefix. A running container is refused.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `init`               | `prism-code init [<dir>] [--sdk-image <ref>]`                                                                                                                                                                                                                                                      | Scaffold a project: `prism.toml`, a language-1.2 model and its Lean workspace, and a `.gitattributes` that keeps sources LF, because LexLean refuses CRLF. Print the build tiers to run next, cheapest first. With `--sdk-image`, also copy that image's `standards.lock` and write `prismpm.lock` (needs Docker). `npm create prism-code` runs the same command.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `manifest`           | `prism-code manifest [<dir>] [--target <target>]`                                                                                                                                                                                                                                                  | Check and lint the project's `prism.toml`, with the overlay for the target merged in, and print the application, the grant and the variables.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `providers`          | `prism-code providers <verb> [<dir>] [--target <target>] [--json]`                                                                                                                                                                                                                                 | `up` runs the `prism-provider-<type>` each capability in `prism.toml` declares and prints the variables it produced, and `down` releases them. Text output names them, and `--json` also prints their values. The capability set must allow the type and each variable. Experimental.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `vectors`            | `prism-code vectors <app.holo>`                                                                                                                                                                                                                                                                    | Replay every acceptance vector the holo carries and exit 1 on a failure.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `history`            | `prism-code history <app.holo> [--output <file>]`                                                                                                                                                                                                                                                  | Write the acceptance vectors the holo carries as a history, its first.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `replay`             | `prism-code replay <history>... --holo <app.holo> [--statement <file>] [--json]`                                                                                                                                                                                                                   | Re-execute histories, or every history in a directory, against this build and compare every outcome: exit 0 unchanged, 1 diverged, 2 could not replay. `--statement` writes the `prism/replay/v1` statement, also on a divergence.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `inspect`            | `prism-code inspect <holo-or-container> [--format <path>]`                                                                                                                                                                                                                                         | Print what the holo says about itself, without running it, or a container's state, exit code, image digest and output.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `attest`             | `prism-code attest <app.holo> --predicate <file> --type <uri>`                                                                                                                                                                                                                                     | Sign an in-toto statement about the holo into its attestation section: keyless with an OIDC token, or with `--key`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `verify-attestation` | `prism-code verify-attestation <app.holo> --certificate-identity <id> --certificate-oidc-issuer <url>`                                                                                                                                                                                             | Check the holo's attestations against the signer you expect, or against `--key`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `export`             | `prism-code export <name@sha256:digest> [--out <dir>] [--tar <file>] [--project <dir>] [--json]`                                                                                                                                                                                                   | Write a release's six-file browser export or tarball, byte-checked against the build manifest, attestations beside.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `generate`           | `prism-code generate <app.holo> --out <dir> [--lang js]`                                                                                                                                                                                                                                           | Write bindings and a typed client from the holo's WIT world and model into `<dir>`, with a record of what they came from. A second run with the same inputs writes nothing.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `check`              | `prism-code check <app.holo> [--policy <file>] [--replay <file>] [--key <file>] [--certificate-identity <id>] [--certificate-oidc-issuer <url>] [--trusted-root <file>] [--json]`                                                                                                                  | Judge the holo against a policy and print its verdict: the assurance level, whether the policy passed, and every reason. `--policy` takes a Rego file, the name of a shipped policy (`pure-only`, `provenance-only`) or a JSON file. Exit 1 when the policy denies, 2 when it cannot be parsed or evaluated. A holo that carries signed provenance and proof statements reaches P1 and P2 only when `--key`, or a certificate identity and issuer, names the signer that signed them. Without a holo, the deprecated spelling of `build --check`, removed in 0.4.0.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `push`               | `prism-code push <app.holo> <reference> [--key <file>] [--keyless] [--no-attestations] [--signing-config <file>] [--store <dir>] [--plain-http] [--json]`                                                                                                                                          | Put the holo in an OCI registry or a local store as a deterministic manifest, each attestation a referrer. With `--key` or `--keyless`, also sign a statement naming the manifest, so `cosign verify-attestation` passes on the same reference.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `pull`               | `prism-code pull <reference> [<out.holo>] [--policy <file>] [--key <file>] [--certificate-identity <id>] [--certificate-identity-regexp <re>] [--certificate-oidc-issuer <url>] [--type <uri>] [--trusted-root <file>] [--entries <n>] [--timestamps <n>] [--store <dir>] [--plain-http] [--json]` | Fetch a holo and write it only after its digests, archive, verdict and attestations pass; the signer flags are those of `verify-attestation`. A reference is `registry/repo:tag`, `@sha256:`, `@blake3:` or `dir://`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `dev`                | `prism-code dev [<dir>] [--port <n>] [--no-open] [--policy <file>] [--build <command>] [--holo <path>]`                                                                                                                                                                                            | Watch the project, rebuild and re-check on every change, and serve the local studio at `http://localhost:4100`, with a Check tab and a Replay tab. The page loads nothing from outside the machine.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `setup`              | `prism-code setup [--agent <id>] [--pull] [--image <ref>] [--agents-only] [--machine-only] [--json] [--dry-run]`                                                                                                                                                                                   | Get this machine ready, then the agents. The machine rows: Node 22 or later; Docker on the PATH or, on Windows without Docker Desktop, inside the first WSL distribution whose `docker version` answers, recorded in `$PRISM_HOME/config.json` so every delegated verb spawns it; the SDK image `prismpm.lock` pins (or `--image`), pulled only with `--pull` or `--image` and otherwise deferred: `build --check` and `verify --lean` run hosted without it, and the first `build`, `verify` or `axioms` pulls it, about 5.5 GB, saying so first; a cache volume per image, mounted at `/tmp`; `* text eol=lf` in the project's `.gitattributes`; and, read only, what `build --remote github` needs: a github.com origin, a token (`GITHUB_TOKEN`, `GH_TOKEN` or `gh`) and `.github/workflows/build.yml`. Then add the MCP server `lexlean` at `https://mcp.uor.foundation/` to every agent found: Claude Code, Codex, Cursor and VS Code, the Connectors step for Claude Desktop; add the local Prism server `prism`, the `mcp` verb, beside it, Claude Desktop included; Install the `prism` skill this package ships into `~/.claude/skills` and `~/.agents/skills`. Install the `holo://` link handler and remember the preferred agent. A server already there, and a `holo://` handler another program owns, are left alone. `--agent` limits it to one agent. `--dry-run` prints every file and command and changes nothing; `--json` prints the report. |
| `mcp`                | `prism-code mcp [--project <dir>]`                                                                                                                                                                                                                                                                 | Serve the Prism MCP server on stdio, for any agent that speaks MCP. It asks six spec questions and checks each answer as it arrives, then writes the LexLean model, has Lean prove every stated property, on the hosted verifier when the SDK image is not on this machine, and builds the holo in the SDK image, downloading it first when it is missing, or on the repository's own GitHub Actions with `remote: "github"`, verified here. `setup` adds it to every agent it configures. Experimental.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `open`               | `prism-code open <link> [--dry-run]`                                                                                                                                                                                                                                                               | Hand a `holo://build?agent=<id>&q=<sentence>` link to its agent. Claude Code and Codex get a new terminal that shows the sentence and runs `claude` or `codex` with it after Enter. Cursor and VS Code get their MCP install link and the sentence on the clipboard. Claude and ChatGPT get their prompt URL. Another host, an unknown agent or parameter, or a sentence over 2000 characters exits 2.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `build`              | `prism-code build [--check] [--hosted] [--local] [--remote github] [--repo <owner/name>] [--ref <commit>] [--timeout <s>] [--allow-dirty] [--json]`                                                                                                                                                | Build the project into a holo inside the SDK image the lockfile pins. With `--remote github`, the repository's own GitHub Actions builds one commit instead, pushed to `prism/build/<sha>` so the run is that commit and no other, and this machine verifies what comes back before anything lands under `.prism/build/`: the commit and repository named in the artifact, the holo's SHA-256, the signed provenance (its subject, its repository and commit, and its signature by a workflow of the repository through GitHub's issuer), then `check` at the P0 policy unless `--policy` names one, the recorded fingerprint, and every acceptance vector replayed here. Needs the template's `build.yml` in the repository, a token (`GITHUB_TOKEN`, `GH_TOKEN` or `gh`), and the Sigstore packages or `gh` for the signature. No Docker. With `--check`, validate the model and write nothing: the full check in the image when it is on this machine, or with `--local`; otherwise `prism.toml` here and `lexlean check --all` on the hosted verifier, which skips the lockfile and SDK pins and says so. The hosted check sends the files under `src/`, unlisted but not private.                                                                                                                                                                                                                                                                            |
| `verify`             | `prism-code verify [--lean] [--hosted] [--local] [--all]`                                                                                                                                                                                                                                          | Run verify in the SDK image the lockfile pins. With `--lean`, run `lexlean verify` on the project's LexLean project instead: every theorem through Lean and leanchecker with its exact axiom policy, about a minute. It runs in the image when the image is on this machine, or with `--local`; otherwise on the hosted verifier at `https://mcp.uor.foundation/` (profile `prism-app`), which takes a project whose only source root is `src` and passes only when every entrypoint module comes back verified, and writes its record to `.lexlean/hosted/`. The hosted verify sends the files under `src/`, unlisted but not private.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `axioms`             | `prism-code axioms [--write] [--json]`                                                                                                                                                                                                                                                             | Compile the project once in the SDK image and print the axiom set Lean observes for every definition and theorem beside the set the source declares; exit 1 while any differs. With `--write`, set each `axioms` member to the observed set and format the project.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `conformance`        | `prism-code conformance`                                                                                                                                                                                                                                                                           | Run the conformance suite in the SDK image the lockfile pins.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |

| Code | Name           | Meaning                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ---- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0    | ok             | The verb did what you asked, or `check` reached a verdict the policy accepts.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 1    | vectors failed | At least one acceptance vector did not replay or the holo carries none, `replay` diverged, `verify-attestation` did not verify, `check` reached a verdict the policy rejects, or `pull` found a digest that does not match or a holo that fails verification and wrote nothing.                                                                                                                                                                                                                                                                                                                                                                                         |
| 2    | refused        | The holo, capability file or manifest cannot be read or fails its checks, an identity check failed, a guest import, the holo's world or a provider is not granted, or a provider failed; or `check` reached no verdict: not a holo, an unreadable policy or an unknown format revision; or `push` or `pull` got no answer: an unreachable registry, refused credentials, a reference that does not exist or referrers that cannot be stored; or an export's bytes differ from the build manifest; or `generate` refused to write: the guest does not conform to its world, or a file in the output directory is not one it wrote; or `replay` could not be carried out. |
| 3    | signing failed | `attest` could not sign, or the optional sigstore packages are not installed.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 10   | proof mismatch | `run` refused a holo for a proof reason. 11 is a world mismatch, 12 a determinism violation, 13 a replay error.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 64   | usage          | The arguments were wrong, or `prismpm.lock` is missing.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 69   | no Docker      | Docker is not installed or not on the PATH.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 70   | untyped        | Dispatch threw something untyped: a guest trap, or a bug in the runtime.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |

### Plugins (experimental)

An executable named `prism-<name>` on the PATH runs as `prism-code <name>`, and `prism <name>` once the CLI is renamed. On Windows the first `PATHEXT` extension found in a directory wins and names are case-insensitive. The first PATH directory that has the plugin wins, and a built-in verb always wins over a plugin.

A plugin answers `prism-<name> prism-cli-plugin-metadata` with JSON: `schemaVersion` "1", `vendor`, `version`, `shortDescription` and optional `errorHooks`, a list of verbs. A plugin that does not answer is listed as invalid and never runs. Arguments reach the plugin unchanged and its exit code is the command's.

When a verb a plugin names in `errorHooks` exits non-zero, prism-code runs `prism-<name> prism-cli-plugin-hooks` with `{"schemaVersion":"1","command","args","exitCode"}` on stdin and prints the `messages` it returns under What's next, at most ten lines. Set `PRISM_CODE_NO_HOOKS=1` to turn hooks off.

| Plugin   | Binary         | Usage               | What it does                                                                                                                                                                                                                                    |
| -------- | -------------- | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `doctor` | `prism-doctor` | `prism-code doctor` | Check Node.js, Docker, `./prismpm.lock` and the optional sigstore packages; exit 1 when a check fails. Ships in the prism-code package, and its error hook points at it when check, build, verify, conformance or export-browser exit 64 or 69. |

### Providers (experimental)

An executable named `prism-provider-<type>` on the PATH is found the way a plugin is, and is not a plugin: `prism-code provider-<type>` is not a command, and the `provider-` prefix is reserved.

| Type | Binary              | In prism.toml                                               | What it does                                                                                                                                                                            |
| ---- | ------------------- | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `kv` | `prism-provider-kv` | `provider = { type = "kv", options = { store, dir, set } }` | A local key-value store, kept as one JSON file under `.prism/kv`. Returns `URL` and `STORE`. `up` never overwrites a key, and `down` deletes the file. Ships in the prism-code package. |

## Global flags

| Flag                   | Default            | Meaning                                                      |
| ---------------------- | ------------------ | ------------------------------------------------------------ |
| `--json`               | off                | Print the result as one JSON object instead of text.         |
| `--policy <file>`      | the default policy | Policy file for `check` and `run`.                           |
| `--unstable-<feature>` | off                | Enable one experimental feature by name for this invocation. |
| `--help`               |                    | Print usage for the verb and exit 0.                         |

## Exit codes

| Code | Name          | Meaning                                                                                               |
| ---- | ------------- | ----------------------------------------------------------------------------------------------------- |
| 0    | pass          | The verb did what you asked and every check it ran passed.                                            |
| 1    | fail          | A check failed, an intent sentence is unproved, or a replay diverged.                                 |
| 2    | cannot verify | No verdict was reached: a missing attestation, an unreachable registry or an unknown format revision. |

## Verbs

Status: the prism CLI has not shipped, so no verb output is captured. Usage and flags come from the stand-in description.

### init

Scaffold a new application: `prism.toml`, `examples/`, `intent.md`, a default capability set and the lockfile.

```bash
prism init [<name>] [flags]
```

| Flag                                  | Default | Meaning                                |
| ------------------------------------- | ------- | -------------------------------------- |
| `--types <ts\|json-schema\|pydantic>` | ts      | Which type source the scaffold writes. |
| `--template <name>`                   | minimal | Starting examples and intent.          |
| `--force`                             | off     | Overwrite files that already exist.    |

```bash
prism init calculator
```

### build

Compile the model into a holo inside the pinned SDK image, record provenance and the proof attestation, and cache the result by spec digest.

```bash
prism build [flags]
```

| Flag                         | Default                              | Meaning                                                                                                         |
| ---------------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------- |
| `--provenance=<min\|max>`    | min                                  | How much the provenance records. `max` adds every resolved input and the full toolchain environment.            |
| `--no-provenance`            | off                                  | Skip provenance. Unsafe: the holo cannot pass `check` above P0.                                                 |
| `--no-cache`                 | off                                  | Rebuild even when the spec digest is unchanged.                                                                 |
| `--target <browser\|node>`   | none                                 | Apply the matching overlay, `prism.browser.toml` or `prism.node.toml`.                                          |
| `--max-iterations <n>`       | 10                                   | How many times the agent may revise the model before the build fails.                                           |
| `--author-model <id>`        | the lockfile's, then claude-opus-5-5 | The model id to pin in the lockfile when the agent runs. It changes nothing while the spec digest is unchanged. |
| `--author-command <path>`    | the bundled adapter                  | The agent process: one request document on stdin, one response document on stdout.                              |
| `--author-timeout <seconds>` | 600                                  | How long one attempt may take.                                                                                  |
| `--author-max-tokens <n>`    | 2000000                              | Stop after this many tokens across attempts.                                                                    |
| `--project <dir>`            | .                                    | The project to author.                                                                                          |

```bash
prism build
```

### check

Verify the holo's digest, provenance, proof, lockfile pins and declared world against a policy, then print a verdict with the assurance level and reasons.

```bash
prism check [<ref>] [flags]
```

| Flag              | Default            | Meaning                                                                    |
| ----------------- | ------------------ | -------------------------------------------------------------------------- |
| `--policy <file>` | the default policy | Policy to judge against.                                                   |
| `--offline`       | off                | Do not contact the transparency log or any registry. Missing data exits 2. |

```bash
prism check calculator@sha256:3f9c
```

### run

Instantiate the holo after `check` passes at the policy's level, then grant exactly the capability set it declares.

```bash
prism run <ref> [flags]
```

| Flag                 | Default            | Meaning                                                       |
| -------------------- | ------------------ | ------------------------------------------------------------- |
| `--input <json>`     | stdin              | One input to dispatch. Omit it to read inputs line by line.   |
| `--policy <file>`    | the default policy | Policy the gate judges against.                               |
| `--allow-unverified` | off                | Run even when `check` fails. Prints a warning on every start. |
| `--rm`               | off                | Remove the container when it exits. One input only.           |
| `--port <n>`         | none               | Serve the instance over HTTP instead of dispatching once.     |

```bash
prism run calculator@sha256:3f9c --input '{"op":"add","a":2,"b":3}'
```

### dev

Watch the model, rebuild and re-check on every change, and serve a local studio with Check and Replay tabs.

```bash
prism dev [flags]
```

| Flag                | Default                                      | Meaning                              |
| ------------------- | -------------------------------------------- | ------------------------------------ |
| `--port <n>`        | 4100                                         | Studio port.                         |
| `--no-open`         | off                                          | Do not open the studio in a browser. |
| `--policy <file>`   | the default policy                           | Policy the Check tab judges against. |
| `--build <command>` | `[model].build`, then `prism build --update` | The build to run, in a shell.        |
| `--holo <path>`     | the one the build reports                    | The file the build writes.           |

```bash
prism dev
```

### replay

Re-execute recorded histories against a build and fail on any divergence, reporting every one.

```bash
prism replay <history>... --holo <path> [flags]
```

| Flag                 | Default  | Meaning                                                                     |
| -------------------- | -------- | --------------------------------------------------------------------------- |
| `--holo <path>`      | required | Which holo to replay against.                                               |
| `--statement <file>` | none     | Write the `prism/replay/v1` statement there, also when the replay diverged. |

```bash
prism replay history/2026-10-23.json --holo calculator.holo
```

### push

Push a holo and its attestations to an OCI registry. The attestations travel as referrers of the holo.

```bash
prism push <ref> <registry/repo[:tag]> [flags]
```

| Flag                | Default | Meaning                                                |
| ------------------- | ------- | ------------------------------------------------------ |
| `--store <dir>`     | none    | Push to a local directory store instead of a registry. |
| `--no-attestations` | off     | Push the holo only. The pushed holo checks at P0.      |

```bash
prism push calculator@sha256:3f9c ghcr.io/acme/calculator:1.0
```

### pull

Pull a holo and its attestations from an OCI registry or a local directory store.

```bash
prism pull <registry/repo[:tag]|ref> [flags]
```

| Flag            | Default | Meaning                                                |
| --------------- | ------- | ------------------------------------------------------ |
| `--store <dir>` | none    | Pull from a local directory store.                     |
| `--check`       | off     | Run `check` on the pulled holo and exit with its code. |

```bash
prism pull ghcr.io/acme/calculator:1.0
```

### deploy

Publish the holo to a GitHub repository and GitHub Pages, serving its attestations alongside it.

```bash
prism deploy [<ref>] [flags]
```

| Flag                  | Default        | Meaning                     |
| --------------------- | -------------- | --------------------------- |
| `--repo <owner/name>` | the git remote | Repository to publish to.   |
| `--branch <name>`     | gh-pages       | Branch GitHub Pages serves. |
| `--no-pages`          | off            | Push the repository only.   |

```bash
prism deploy
```

### export

Write the six-file browser export, a tarball, or a runnable OCI image that `docker run` serves.

```bash
prism export [<ref>] [flags]
```

| Flag               | Default | Meaning                                     |
| ------------------ | ------- | ------------------------------------------- |
| `--out <dir>`      | export/ | Where the browser export is written.        |
| `--tar <file>`     | none    | Write one tarball instead of a directory.   |
| `--oci <name:tag>` | none    | Write a runnable OCI image under this name. |

```bash
prism export --oci acme/calculator:1.0
```

### inspect

Print the manifest, section digests, format revision, world and attestations of a holo without running it.

```bash
prism inspect <ref> [flags]
```

| Flag                                                           | Default | Meaning                                                                       |
| -------------------------------------------------------------- | ------- | ----------------------------------------------------------------------------- |
| `--format <manifest\|sections\|world\|attestations\|revision>` | all     | Print one part only.                                                          |
| `--audit`                                                      | off     | Print the security view: signers, trusted base, capability set, theorem list. |
| `--report <file>`                                              | none    | With `--audit`, write the report to a file.                                   |

```bash
prism inspect calculator@sha256:3f9c
```

### attest

Add a signed statement to a holo. Signing is keyless by default.

```bash
prism attest <ref> --predicate <file> --type <uri> [flags]
```

| Flag                 | Default  | Meaning                                                      |
| -------------------- | -------- | ------------------------------------------------------------ |
| `--predicate <file>` | required | JSON file holding the predicate.                             |
| `--type <uri>`       | required | Predicate type URI.                                          |
| `--key <file>`       | none     | Sign with a local key for air-gapped use instead of keyless. |

```bash
prism attest calculator@sha256:3f9c --predicate sbom.json --type https://example.com/sbom/v1
```

### verify

Check a signed statement on a holo against the signing identity you expect.

```bash
prism verify <ref> --certificate-identity <id> --certificate-oidc-issuer <url> [flags]
```

| Flag                              | Default  | Meaning                                               |
| --------------------------------- | -------- | ----------------------------------------------------- |
| `--certificate-identity <id>`     | required | Identity the statement must be signed by.             |
| `--certificate-oidc-issuer <url>` | required | Issuer that identity must come from.                  |
| `--type <uri>`                    | all      | Verify statements of one predicate type only.         |
| `--key <file>`                    | none     | Verify against a local public key instead of keyless. |

```bash
prism verify calculator@sha256:3f9c --certificate-identity ci@acme.example --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

### generate

Regenerate the JS, Rust and Python bindings and the typed client from the world. Nothing is written when nothing changed.

```bash
prism generate [flags]
```

| Flag                      | Default        | Meaning                                                                                                 |
| ------------------------- | -------------- | ------------------------------------------------------------------------------------------------------- |
| `--out <dir>`             | none, required | Output path. There is no default, so nothing lands in `node_modules` or another place you did not name. |
| `--lang <js,rust,python>` | all three      | Which bindings to write.                                                                                |

```bash
prism generate --out generated
```

### lock

Write or refresh the lockfile from the current model and inputs, without building. Pins that still resolve stay as they are, and the SDK image and toolchain pins do not move. Exits 2 when there is no lockfile or its SDK cannot be resolved.

```bash
prism lock [flags]
```

| Flag      | Default | Meaning                                                                       |
| --------- | ------- | ----------------------------------------------------------------------------- |
| `--check` | off     | Write nothing. Fail when the lockfile is stale and name the pins that differ. |

```bash
prism lock
```

### update

Move the lockfile forward and print the old and new digests of every pin that changed. Without a flag it refreshes the toolchain pins; with `--toolchain` it also pins a new SDK image. Exits 2 when the lockfile or the image cannot be resolved.

```bash
prism update [flags]
```

| Flag                        | Default                      | Meaning                                                    |
| --------------------------- | ---------------------------- | ---------------------------------------------------------- |
| `--toolchain <ref>`         | the pinned image             | SDK image to pin, as `name@sha256:digest`.                 |
| `--standards-lock <digest>` | the project's standards lock | Standards lock the new image selects. Needs `--toolchain`. |
| `--dry-run`                 | off                          | Print the change without writing the lockfile.             |

```bash
prism update --toolchain ghcr.io/acme/toolchain@sha256:1f3c --dry-run
```

## Next steps

- [Manifest reference](/reference/manifest)
- [Errors reference](/reference/errors)
- [Lockfile reference](/reference/lockfile)

Generated from sdk/docs/cli.json 2026-10-30 (stand-in for prism --help --json).
