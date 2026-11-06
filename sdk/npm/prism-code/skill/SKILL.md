---
name: prism
description: Build a new Prism app from one sentence and publish it, or load, run, inspect and verify a holo, the Prism package that carries its own proof and provenance. Use when asked to build, start or ship a Prism app, when a task mentions Prism, LexLean, a .holo file or prism-code, or when it asks whether an application can be trusted before it runs.
---

# Prism

Prism works like Docker. A model is built into a holo, an immutable package addressed by its digest, like an image. A runtime runs the holo, like a container runtime runs an image, and every run starts from a loaded, verified instance, like a container. A holo carries its own proof and provenance, so you check it before anything runs.

| Docker | Prism |
| --- | --- |
| Dockerfile | model |
| image, tag and digest | holo |
| container | instance |
| base image pinned in `FROM` | SDK image pinned in the lockfile |
| `docker run` | `prism-code run` |
| `docker inspect` | `prism-code inspect` |

## Install

```bash
npx prism-code@latest setup
```

One run adds the `lexlean` MCP server to every agent on the machine and installs this skill. Spell it `prism-code@latest`, because `npx` prefers an older `prism-code` installed in any folder above the current one. To use the package from code, `npm install prism-code`.

Use `npx prism-code` followed by a verb to run it without installing. The package ships a sample holo, `Calculator.holo`, so you can try every command below without building anything.

## Try it

<!-- generated:try:begin -->
```bash
npx prism-code --version   # print the installed version
npx prism-code inspect node_modules/prism-code/fixtures/Calculator.holo   # print what the holo says about itself, without running it
npx prism-code vectors node_modules/prism-code/fixtures/Calculator.holo   # replay its acceptance vectors; exit 0 means every one passed
```
<!-- generated:try:end -->

## Commands

Run `npx prism-code --help` for the list your installed version has. The commands below are the ones that ship today.

<!-- generated:commands:begin -->
| Command | What it does |
| --- | --- |
| `prism-code run <app.holo> [<input>] [--input <text>] [--rm] [--port <n>] [--policy <file>] [--allow-unverified] [--capabilities <file>]` | Check the holo against the policy. With an input, this is one run as a container: dispatch the input, print the output and exit with the container's code, keeping the container until `rm` unless `--rm`. Without one, serve its view on a local port; a POST to `/_hologram/intent` dispatches one request. A holo that fails check is refused unless `--allow-unverified`, which warns on every start. |
| `prism-code rm <container>...` | Remove containers, by id or unique id prefix. A running container is refused. |
| `prism-code init [<dir>] [--sdk-image <ref>]` | Scaffold a project: `prism.toml`, a language-1.2 model and its Lean workspace, and a `.gitattributes` that keeps sources LF, because LexLean refuses CRLF. Print the build tiers to run next, cheapest first. With `--sdk-image`, also copy that image's `standards.lock` and write `prismpm.lock` (needs Docker). `npm create prism-code` runs the same command. |
| `prism-code manifest [<dir>] [--target <target>]` | Check and lint the project's `prism.toml`, with the overlay for the target merged in, and print the application, the grant and the variables. |
| `prism-code providers <verb> [<dir>] [--target <target>] [--json]` | `up` runs the `prism-provider-<type>` each capability in `prism.toml` declares and prints the variables it produced, and `down` releases them. Text output names them, and `--json` also prints their values. The capability set must allow the type and each variable. Experimental. |
| `prism-code vectors <app.holo>` | Replay every acceptance vector the holo carries and exit 1 on a failure. |
| `prism-code history <app.holo> [--output <file>]` | Write the acceptance vectors the holo carries as a history, its first. |
| `prism-code replay <history>... --holo <app.holo> [--statement <file>] [--json]` | Re-execute histories, or every history in a directory, against this build and compare every outcome: exit 0 unchanged, 1 diverged, 2 could not replay. `--statement` writes the `prism/replay/v1` statement, also on a divergence. |
| `prism-code inspect <holo-or-container> [--format <path>]` | Print what the holo says about itself, without running it, or a container's state, exit code, image digest and output. |
| `prism-code attest <app.holo> --predicate <file> --type <uri>` | Sign an in-toto statement about the holo into its attestation section: keyless with an OIDC token, or with `--key`. |
| `prism-code verify-attestation <app.holo> --certificate-identity <id> --certificate-oidc-issuer <url>` | Check the holo's attestations against the signer you expect, or against `--key`. |
| `prism-code export <name@sha256:digest> [--out <dir>] [--tar <file>] [--project <dir>] [--json]` | Write a release's six-file browser export or tarball, byte-checked against the build manifest, attestations beside. |
| `prism-code generate <app.holo> --out <dir> [--lang js]` | Write bindings and a typed client from the holo's WIT world and model into `<dir>`, with a record of what they came from. A second run with the same inputs writes nothing. |
| `prism-code check <app.holo> [--policy <file>] [--replay <file>] [--key <file>] [--certificate-identity <id>] [--certificate-oidc-issuer <url>] [--trusted-root <file>] [--json]` | Judge the holo against a policy and print its verdict: the assurance level, whether the policy passed, and every reason. `--policy` takes a Rego file, the name of a shipped policy (`pure-only`, `provenance-only`) or a JSON file. Exit 1 when the policy denies, 2 when it cannot be parsed or evaluated. A holo that carries signed provenance and proof statements reaches P1 and P2 only when `--key`, or a certificate identity and issuer, names the signer that signed them. Without a holo, the deprecated spelling of `build --check`, removed in 0.4.0. |
| `prism-code push <app.holo> <reference> [--key <file>] [--keyless] [--no-attestations] [--signing-config <file>] [--store <dir>] [--plain-http] [--json]` | Put the holo in an OCI registry or a local store as a deterministic manifest, each attestation a referrer. With `--key` or `--keyless`, also sign a statement naming the manifest, so `cosign verify-attestation` passes on the same reference. |
| `prism-code pull <reference> [<out.holo>] [--policy <file>] [--key <file>] [--certificate-identity <id>] [--certificate-identity-regexp <re>] [--certificate-oidc-issuer <url>] [--type <uri>] [--trusted-root <file>] [--entries <n>] [--timestamps <n>] [--store <dir>] [--plain-http] [--json]` | Fetch a holo and write it only after its digests, archive, verdict and attestations pass; the signer flags are those of `verify-attestation`. A reference is `registry/repo:tag`, `@sha256:`, `@blake3:` or `dir://`. |
| `prism-code dev [<dir>] [--port <n>] [--no-open] [--policy <file>] [--build <command>] [--holo <path>]` | Watch the project, rebuild and re-check on every change, and serve the local studio at `http://localhost:4100`, with a Check tab and a Replay tab. The page loads nothing from outside the machine. |
| `prism-code setup [--agent <id>] [--pull] [--image <ref>] [--agents-only] [--machine-only] [--json] [--dry-run]` | Get this machine ready, then the agents. The machine rows: Node 22 or later; Docker on the PATH or, on Windows without Docker Desktop, inside the first WSL distribution whose `docker version` answers, recorded in `$PRISM_HOME/config.json` so every delegated verb spawns it; the SDK image `prismpm.lock` pins (or `--image`), pulled only with `--pull` or `--image` and otherwise deferred: `build --check` and `verify --lean` run hosted without it, and the first `build`, `verify` or `axioms` pulls it, about 5.5 GB, saying so first; a cache volume per image, mounted at `/tmp`; `* text eol=lf` in the project's `.gitattributes`; and, read only, what `build --remote github` needs: a github.com origin, a token (`GITHUB_TOKEN`, `GH_TOKEN` or `gh`) and `.github/workflows/build.yml`. Then add the MCP server `lexlean` at `https://mcp.uor.foundation/` to every agent found: Claude Code, Codex, Cursor and VS Code, the Connectors step for Claude Desktop; add the local Prism server `prism`, the `mcp` verb, beside it, Claude Desktop included; Install the `prism` skill this package ships into `~/.claude/skills` and `~/.agents/skills`. Install the `holo://` link handler and remember the preferred agent. A server already there, and a `holo://` handler another program owns, are left alone. `--agent` limits it to one agent. `--dry-run` prints every file and command and changes nothing; `--json` prints the report. |
| `prism-code mcp [--project <dir>]` | Serve the Prism MCP server on stdio, for any agent that speaks MCP. It asks six spec questions and checks each answer as it arrives, then writes the LexLean model, has Lean prove every stated property, on the hosted verifier when the SDK image is not on this machine, and builds the holo in the SDK image, downloading it first when it is missing, or on the repository's own GitHub Actions with `remote: "github"`, verified here. `setup` adds it to every agent it configures. Experimental. |
| `prism-code open <link> [--dry-run]` | Hand a `holo://build?agent=<id>&q=<sentence>` link to its agent. Claude Code and Codex get a new terminal that shows the sentence and runs `claude` or `codex` with it after Enter. Cursor and VS Code get their MCP install link and the sentence on the clipboard. Claude and ChatGPT get their prompt URL. Another host, an unknown agent or parameter, or a sentence over 2000 characters exits 2. |
| `prism-code build [--check] [--hosted] [--local] [--remote github] [--repo <owner/name>] [--ref <commit>] [--timeout <s>] [--allow-dirty] [--json]` | Build the project into a holo inside the SDK image the lockfile pins. With `--remote github`, the repository's own GitHub Actions builds one commit instead, pushed to `prism/build/<sha>` so the run is that commit and no other, and this machine verifies what comes back before anything lands under `.prism/build/`: the commit and repository named in the artifact, the holo's SHA-256, the signed provenance (its subject, its repository and commit, and its signature by a workflow of the repository through GitHub's issuer), then `check` at the P0 policy unless `--policy` names one, the recorded fingerprint, and every acceptance vector replayed here. Needs the template's `build.yml` in the repository, a token (`GITHUB_TOKEN`, `GH_TOKEN` or `gh`), and the Sigstore packages or `gh` for the signature. No Docker. With `--check`, validate the model and write nothing: the full check in the image when it is on this machine, or with `--local`; otherwise `prism.toml` here and `lexlean check --all` on the hosted verifier, which skips the lockfile and SDK pins and says so. The hosted check sends the files under `src/`, unlisted but not private. |
| `prism-code verify [--lean] [--hosted] [--local] [--all]` | Run verify in the SDK image the lockfile pins. With `--lean`, run `lexlean verify` on the project's LexLean project instead: every theorem through Lean and leanchecker with its exact axiom policy, about a minute. It runs in the image when the image is on this machine, or with `--local`; otherwise on the hosted verifier at `https://mcp.uor.foundation/` (profile `prism-app`), which takes a project whose only source root is `src` and passes only when every entrypoint module comes back verified, and writes its record to `.lexlean/hosted/`. The hosted verify sends the files under `src/`, unlisted but not private. |
| `prism-code axioms [--write] [--json]` | Compile the project once in the SDK image and print the axiom set Lean observes for every definition and theorem beside the set the source declares; exit 1 while any differs. With `--write`, set each `axioms` member to the observed set and format the project. |
| `prism-code conformance` | Run the conformance suite in the SDK image the lockfile pins. |

| Exit code | Name | Meaning |
| --- | --- | --- |
| 0 | ok | The verb did what you asked, or `check` reached a verdict the policy accepts. |
| 1 | vectors failed | At least one acceptance vector did not replay or the holo carries none, `replay` diverged, `verify-attestation` did not verify, `check` reached a verdict the policy rejects, or `pull` found a digest that does not match or a holo that fails verification and wrote nothing. |
| 2 | refused | The holo, capability file or manifest cannot be read or fails its checks, an identity check failed, a guest import, the holo's world or a provider is not granted, or a provider failed; or `check` reached no verdict: not a holo, an unreadable policy or an unknown format revision; or `push` or `pull` got no answer: an unreachable registry, refused credentials, a reference that does not exist or referrers that cannot be stored; or an export's bytes differ from the build manifest; or `generate` refused to write: the guest does not conform to its world, or a file in the output directory is not one it wrote; or `replay` could not be carried out. |
| 3 | signing failed | `attest` could not sign, or the optional sigstore packages are not installed. |
| 10 | proof mismatch | `run` refused a holo for a proof reason. 11 is a world mismatch, 12 a determinism violation, 13 a replay error. |
| 64 | usage | The arguments were wrong, or `prismpm.lock` is missing. |
| 69 | no Docker | Docker is not installed or not on the PATH. |
| 70 | untyped | Dispatch threw something untyped: a guest trap, or a bug in the runtime. |

Node.js 22 or later. build, verify, axioms and conformance also need Docker and a `prismpm.lock` in the current directory; `build --check` and `verify --lean` run on the hosted verifier without them.
<!-- generated:commands:end -->

## Load a holo from code

Three calls: `load` opens a holo and verifies its digests, `dispatch` runs one request on the instance, `acceptance` replays the examples the holo carries. The script expects `Calculator.holo` beside it; copy it from `node_modules/prism-code/fixtures/`.

<!-- generated:example:begin -->
```js
import { load } from 'prism-code';

const app = await load('./Calculator.holo');
console.log(app.name, app.identities.verified);

const response = await app.dispatch(new TextEncoder().encode('1\tadd\t20\t22'));
console.log(JSON.stringify(new TextDecoder().decode(response)));

const result = await app.acceptance();
console.log(result.passed, 'of', result.total, 'examples replayed');
```

```text
Calculator true
"1\tok\t42"
22 of 22 examples replayed
```
<!-- generated:example:end -->

## How to work with a holo

- Check before you run. `inspect` and `vectors` read a holo without trusting it. A holo that fails an identity check exits 2: stop and report it. Do not run it, and do not edit its bytes.
- Report what the output says. `inspect` prints the holo's identities, limits, provenance and view files. Do not infer a proof or a signed statement from what it leaves out; say what is and is not there.
- Read the exit code before the text. 0 is ok, 1 is a failed vector, 2 is a refused holo, 64 is a usage error, 69 is no Docker.
- `check`, `build`, `verify`, `axioms` and `conformance` run the project's SDK image, so they need Docker and a `prismpm.lock` in the current directory. `build --check` and `verify --lean` also run on the hosted verifier when the image is not here. Say so when Docker is missing instead of working around it.
- The docs say which release each feature needs. Read the release status page before you promise a feature.

## Build a new app

The user says something like "build me a Prism app that tells whether a year is a leap year". That sentence is the whole brief. Give back a live page where anyone can load the app, check its fingerprint and replay its examples.

**The guided path.** When the `prism` MCP server is connected and the app decides one answer from a line of fields (each a number or a choice from a fixed list), let it do the work on this machine. It is experimental. Call `prism_start` and follow what it returns: it asks six short questions, one at a time. Rewrite each answer into the format the question shows, say what you wrote, and pass it to `prism_answer`. When a result has an `issue`, such as a property the rules break or an example they contradict, explain it in a sentence and ask the user how to resolve it. Once the spec is complete, call `prism_build`, and call it again while it says `running`. Prism writes the model, Lean proves every property (on the hosted verifier when the SDK image is not on this machine, sending the files under `prism/src/`, unlisted but not private), and the holo is built in the SDK image, which the first build downloads (about 5.5 GB) and says so. Never edit files under `prism/` by hand. Say "proved" only for what it reports as proved; the acceptance vectors are tested. For anything else, or to publish the app, follow the steps below.

1. **Decide, then ask once.** Restate the purpose as one sentence the app guarantees. Name the repository `prism-` plus two or three lowercase words. The owner is `gh api user --jq .login`, and the repository is public, because Pages on a private repository needs a paid plan. Creating a public repository is outward facing, so ask for approval in the same line that announces the plan, and ask nothing else until the app is live or something fails.
2. **Create it from the template.**

   ```bash
   gh repo create OWNER/NAME --template UOR-Foundation/template --public
   gh api -X POST repos/OWNER/NAME/pages -f build_type=workflow
   git -c core.autocrlf=false clone https://github.com/OWNER/NAME.git
   ```

   Keep `core.autocrlf` off in the clone: CRLF breaks the byte-pinned files and LexLean refuses it.
3. **Read before you write.** Read the template's `AGENTS.md` (its rules are the gate), `README.md` and `app/src/Contract.lex.tex`, a complete small model with its theorem. Copy shapes; do not guess LexLean syntax.
4. **Write the model through the MCP server.** `lexlean_profiles` (use `prism-app`), then `lexlean_draft_assemble`, `lexlean_check` and `lexlean_format` in seconds, `lexlean_explain` for a diagnostic, and `lexlean_verify` to kernel-check the theorems. Keep the attestation ID. Check clean before you verify, and retry a 502 or a lost task after ten seconds. Copy the checked files into `app/src/`.
5. **Follow the template's order of work.** Add each capability with `just new-capability`, write its model and real examples (requests and their exact responses, which anyone can replay), make the failing test pass, then plant the defect it catches and see the gate fail. Never edit the files `template.lock` pins.
6. **Ship.** Push to `main`. The `bootstrap` workflow is the gate; the template's `pages.yml` calls `build.yml`, which builds and verifies the holo in the SDK image, signs its provenance and publishes the page. Watch both with `gh run watch`, and fix the cause of a red run instead of weakening the gate. To hold the holo on this machine without Docker, run `npx prism-code build --remote github --ref <the commit>`: it takes that run's artifact and verifies it here (see Build without Docker below).
7. **Report in three lines:** the live URL, the repository, and the fingerprint with how many examples replay, plus the attestation ID.

Rules the gate enforces that are easy to miss: a `\title` uses only words the module's glossaries define (`LLL1004`); a proof declares exactly the axioms it uses (`LLV7005` names the observed set); requests are unique, and one is exactly `guestAllocationMaximum` bytes (`PP2009`).

### Verify locally in tiers

`npx prism-code@latest setup` also readies the machine: it finds Docker, also inside WSL on Windows, and creates the build cache. It does not pull the SDK image (about 5.5 GB) unless you pass `--pull`. Until the image is here, `build --check` and `verify --lean` run on the hosted verifier at `https://mcp.uor.foundation/`, which receives the files under `src/` (unlisted, not private), and the first `build`, `verify` or `axioms` pulls the image and says so first. Then judge every edit by the cheapest gate that can refuse it:

```bash
npx prism-code build --check
npx prism-code verify --lean
npx prism-code axioms --write
npx prism-code verify
```

- `build --check` takes seconds: the model is well formed and locked. Run it after every edit. Hosted, it checks the model and `prism.toml` but not the lockfile pins, and its last line says so.
- `verify --lean` takes about a minute: every theorem through Lean and leanchecker, every axiom policy exact. Run it before the full build, never instead of it. Hosted, it passes only when every entrypoint module comes back verified, and writes its record to `.lexlean/hosted/`; `hosted.layout` means the project's shape needs `--local`.
- When `verify --lean` names an axiom policy, run `axioms --write` once. It sets every declaration's `axioms` member from what Lean observes, in one pass. Do not guess axiom lists by hand; the verify reports only the first wrong one per run.
- `verify` takes minutes: the guest, the vectors and the holo. Run it once the cheaper gates pass.
- Keep sources LF. `init` writes `* text eol=lf`; LexLean refuses a file with CRLF.
- Exit 69 means no Docker. Run `npx prism-code@latest setup` and report what its `docker` row says.

### Build without Docker: GitHub builds, this machine checks

```bash
npx prism-code build --remote github
```

The repository's own GitHub Actions builds one commit, and this machine verifies what comes back before it counts as built. Nothing here needs Docker, the SDK image or a toolchain, so it works from any machine with Node and a token, and the same run serves a phone: an edit committed on github.com or GitHub Mobile builds the app, signs its provenance and publishes the page, which verifies the holo again in the visitor's browser.

- It needs: the template's `.github/workflows/build.yml` in the repository (a new instance has it; an older one copies it from the template), a token (`GITHUB_TOKEN`, `GH_TOKEN` or `gh auth login`), and the commit pushed as the working tree stands. A tree with uncommitted changes is refused; commit, or name a commit with `--ref`.
- What happens: the commit goes to `refs/heads/prism/build/<sha>`, so the run is that commit and no other; `build.yml` runs the full verify in the SDK image and uploads `prism-build-<sha>` with the holo and its signed provenance; the ref is deleted afterwards. A run that already exists for the commit, from a push to `main` for instance, is reused with no push.
- What is checked here, in order, before anything lands under `.prism/build/`: the artifact names this commit and repository; the holo's SHA-256 is the one recorded; the provenance's subject is that digest and its build names this repository and commit; the signature is by a workflow of this repository through GitHub's issuer (prism-code's Sigstore verifier, else `gh attestation verify`, else refused); `check` passes at the P0 policy, or the one `--policy` names; the fingerprint matches; every acceptance vector replays. Report the fingerprint and the run URL it prints.
- Read the exit code: 2 is a refusal (`remote.*`, the message says which check), 1 is bytes that differ from their digest or a verdict or vector that failed, 0 is verified. A run that ended in failure is reported as a failure and nothing is written: read its log on GitHub.
- `setup`'s `github` row says what is in place: the origin repository, where a token would come from, and whether `build.yml` is there. Pages on a private repository needs a paid plan; the artifact and the run's summary need none.

## Docs

<!-- generated:docs:begin -->
- https://universal-coordinate-foundation.lovable.app/docs/llms.txt: the index of every page, written for agents
- https://universal-coordinate-foundation.lovable.app/docs/llms-full.md: every page in one file
- https://universal-coordinate-foundation.lovable.app/docs/reference/cli: every command, flag and exit code
- https://universal-coordinate-foundation.lovable.app/docs/reference/release-status: what has shipped and what is planned
- https://universal-coordinate-foundation.lovable.app/docs/guides/verify-and-inspect: check and inspect a holo
- https://universal-coordinate-foundation.lovable.app/docs/guides/agents: agents and holos
- https://universal-coordinate-foundation.lovable.app/docs/reference/holo-format: the byte layout of a holo
- Append `.md` to any page URL to get that page as Markdown.
<!-- generated:docs:end -->

## MCP

<!-- mcp:begin -->
The MCP server `lexlean` at `https://mcp.uor.foundation/` checks, formats and kernel-verifies LexLean models with no local toolchain or Docker. `npx prism-code@latest setup` adds it to Claude Code, Codex, Cursor and VS Code; in Claude Desktop, claude.ai and ChatGPT, add that URL as a custom connector. Its tools include `lexlean_profiles`, `lexlean_lexicon_search`, `lexlean_draft_assemble`, `lexlean_draft_write`, `lexlean_check`, `lexlean_format`, `lexlean_explain`, `lexlean_verify` and `lexlean_attestation`. Drafts are unlisted but not private, so put no secrets in a module.

`setup` also adds the local server `prism`, which is `prism-code mcp` on stdio, in Claude Desktop too. Its tools are `prism_start`, `prism_answer`, `prism_spec`, `prism_prove`, `prism_build`, `prism_try` and `prism_status`: the guided path under Build a new app. Proving needs neither Docker nor the SDK image: without them it runs hosted. Building needs both; the first build downloads the image. When it says Docker is missing, tell the user to run `npx prism-code@latest setup`.
<!-- mcp:end -->
