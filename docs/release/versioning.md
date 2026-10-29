# How a version is chosen and cut

This page serves issue [#262](https://github.com/UOR-Foundation/PrismPM/issues/262).
The policy is [STABILITY.md](../../STABILITY.md) section 2; this page says how the repository enforces it and why no release bot is configured.

## The rule

Before 1.0, for the `prismpm` crate and every npm package under `sdk/npm/`:

- a version whose entry lists a breaking change is a minor bump (0.4.0 to 0.5.0);
- every other version is a patch bump (0.4.0 to 0.4.1).

This is the rule release-please calls `bump-minor-pre-major`, with `bump-patch-for-minor-pre-major` off.

## What enforces it

`node sdk/docs/changelog.mjs` reads `CHANGELOG.md` and fails, naming file and line, when an entry after 0.4.0:

- uses a header other than `### Features Added`, `### Breaking Changes`, `### Bugs Fixed`, `### Other Changes`, repeats one, or writes them out of that order;
- has no `### Breaking Changes` (a release that breaks nothing writes `None.` under it);
- lists breaking changes without being a minor bump over the entry before it, or lists none without being a patch bump;
- has no date, once it is a released version rather than `## [Unreleased]`.

The `docs generate` workflow runs it on every pull request, through `sdk/docs/test/changelog.test.mjs`, which also runs it on the real `CHANGELOG.md`.
Entries up to 0.4.0 keep the Keep a Changelog headers they were published with (`LEGACY_THROUGH` in `sdk/docs/lib/changelog.mjs`).
`release-check` reads the dated 0.4.0 entry and its `### Compatibility` policy, so those entries are not rewritten.

## What a person still does

Nothing here tags, publishes or opens a release pull request.
A release is cut as before:

1. Move the `## [Unreleased]` entry to `## [X.Y.Z] - YYYY-MM-DD` and set `Cargo.toml` and the `package.json` of each package that changed to `X.Y.Z`.
2. Run `node sdk/docs/changelog.mjs`; it refuses a wrong bump.
3. Tag. The crates and image release is the `v*` tag read by `release.yml`; `prism-code` is the `prism-code-v*` tag read by `npm-prism-code.yml`.

## Why not a release bot

Options weighed for #262:

| Option | Fits | Cost |
| --- | --- | --- |
| This check, no bot (chosen) | One rule for the crate and all four npm packages. Keeps the hand-written entries and the `release-check` reading of `CHANGELOG.md`. Cannot tag or publish. | The bump is chosen by a person and checked, not computed. |
| release-please, manifest mode with `bump-minor-pre-major` | Computes the bump and opens a release pull request for the crate and each npm package. | It writes `CHANGELOG.md` from commit subjects, and its stock output puts a `BREAKING CHANGES` heading first rather than the four headers in STABILITY.md's order, so entries need a custom notes step. The Rust workspace version (`workspace.package.version`) needs `extra-files`. Merging its release pull request creates tags, so it must be set to open pull requests only. It needs `contents` and `pull-requests` write on the token. |
| `cargo-release` or `release-plz` | Computes Rust bumps; release-plz can template the changelog. | Rust only; the npm packages stay manual. `release-plz` publishes to crates.io by default. |

If the owner wants a bot later, release-please is the one that covers both ecosystems.
Adopt it with `skip-github-release` set and a notes template for the four headers, and keep `node sdk/docs/changelog.mjs` as the check on what it writes.
