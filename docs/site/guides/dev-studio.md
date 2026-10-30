---
title: Develop with the studio
description: Watch the model, rebuild and re-check on every save, and read the verdict in a local studio that works offline.
available_in: "0.6"
---
This guide shows you the loop you run while you write a model: save, build, check, read the verdict. `dev` runs it for you and shows the result in a studio in your browser.

## Start the loop

Run `dev` in the project directory:

```bash
prism dev
```

```text
calculator in /home/you/calculator
watching: prism.toml, prism.browser.toml, prism.node.toml, prismpm.toml, lexlean.toml, lexlean.lock, prismpm.lock, src/**/*.lex.tex, language/**/*.toml (not .git, node_modules, .lake, .lexlean, .prism)
build: prism build; check: policy default
replay: histories/*.json against every build
studio at http://127.0.0.1:4100/ (offline: Check and Replay tabs; GET /verdict, GET /events)
run 1  FAILED  P0 unverified  policy default  edit to verdict 38.40 s (build 38.10 s, check 0.30 s; start)
```

It watches the model sources and the project files. The model sources are the globs in `[model] watch` of `prism.toml`; without them, the `[model] source` directory, and without that, `src/**/*.lex.tex` and `language/**/*.toml`. The project files are `prism.toml`, its overlays, the toolchain files and the lockfiles. The build tree is never watched, so a build cannot trigger itself.

On every change it runs the project's build, then `check` on the holo the build wrote. The build is `[model] build` in `prism.toml` when the manifest names one, otherwise `prism build`. The verdict is the one `check` prints, with the same policy:

```bash
prism dev --policy policy
```

A burst of saves is one rebuild. A save that lands while a build is running queues exactly one more run, with every file of the new burst. The terminal prints one line per run; the studio shows the rest.

## The Check tab

The Check tab is the verdict, as `check` prints it: passed or failed, the level and its name, the policy and its minimum, the holo digest, and every reason with its check id, its registered code, its title and the level it withholds. From P2 up it also shows what the check trusts the signer for, the same four limits `check` prints.

When the build fails, the tab shows the build's output instead of a verdict. When `check` reaches no verdict, it says why: an unreadable policy, a file that is not a holo. Neither stops the loop; the next save runs it again.

## The Replay tab

The Replay tab lists the histories in `histories/`, every `.json` file directly in it, and replays them against each new build the way `replay` does. Each history shows its event count and its divergences, and each divergence shows the request, the recorded outcome and the replayed one. A build that changes behaviour shows up here before you commit it.

In a build of `prism-code` without `replay`, the tab says "replay not available in this build".

## Measure edit to verdict

Every run records the time from the first changed file of the burst to the verdict, split into build and check. The studio shows it on the verdict card and in the Runs table, and the terminal prints it on every line. The target for `dev` is that this time stays under the rebuild floor of the toolchain image: the build dominates, and the check adds well under a second.

## Offline by design

The studio is three static files served by the dev process. It loads nothing from a CDN and makes no network call. The page carries a `Content-Security-Policy` that allows only its own origin, so the browser refuses any other origin even if a file tried. Live updates arrive over Server-Sent Events from the same process, never by polling.

The same state is plain HTTP for scripts:

| Path | Returns |
| --- | --- |
| `GET /state` | Everything the studio shows, as JSON. |
| `GET /verdict` | The latest verdict, exactly as `check --json` prints it. 404 until there is one. |
| `GET /events` | A `state` event on every change, as Server-Sent Events. |
| `POST /rebuild` | Runs the loop once, now. |

## The port

The studio listens on `127.0.0.1:4100`. The port is fixed so a bookmark, a script and a teammate's instructions always reach the same studio. When the port is in use, `dev` refuses to start and exits 69 instead of picking another port. Pass `--port <n>` to choose a different one.

## Today

The same loop runs from `prism-code`, with the build the manifest names:

```bash
npx prism-code dev --port 4100 --policy policy.json
```

Without a `[model] build` in `prism.toml`, the build is `prism-code build`, which runs the toolchain image the lockfile pins and needs Docker.

## Next steps

- [Verify and inspect](/docs/guides/verify-and-inspect)
- [Run and gate](/docs/guides/run-and-gate)
- [CLI reference](/docs/reference/cli)
