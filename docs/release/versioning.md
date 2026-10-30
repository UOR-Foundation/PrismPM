# How a version is chosen and cut

This page serves issue [#262](https://github.com/UOR-Foundation/PrismPM/issues/262).
The policy is [STABILITY.md](../../STABILITY.md) section 2; this page says how the repository enforces it.

## The rule

Before 1.0, for the `prismpm` crate and every npm package under `sdk/npm/`:

- a version whose entry lists a breaking change is a minor bump (0.4.0 to 0.5.0);
- every other version is a patch bump (0.4.0 to 0.4.1).

## The tooling

| File | What it is |
| --- | --- |
| `release-please-config.json` | `bump-minor-pre-major: true`, `bump-patch-for-minor-pre-major: false`; one package per release unit (the root, which carries the Cargo workspace version in `Cargo.toml`, and the four npm packages); changelog sections named as the four headers. |
| `.release-please-manifest.json` | The last released version of each package. Edit it by hand when a release is tagged outside release-please. |
| `.github/workflows/release-please.yml` | Manual (`workflow_dispatch`) and `skip-github-release`: opens or updates release pull requests, never tags, never publishes. |
| `node sdk/docs/changelog.mjs` | Holds `CHANGELOG.md` and each `sdk/npm/*/CHANGELOG.md` to the four headers and the bump rule. Runs in `docs generate` on every pull request. |

release-please reads the commit subjects since each package's last tag and proposes the next version and changelog entry in a pull request.
It sees only subjects in the form `type(scope): subject`; any other subject is skipped, so a breaking change under a free-form subject would be missed.
Write the pull request title that way (`feat(scope): ...`; a breaking change is `feat(scope)!: ...` or has a `BREAKING CHANGE:` footer), because a squash merge uses it as the subject.

## What the changelog check enforces

An entry after 0.4.0 in the root changelog, and every entry in a package changelog:

- uses only `### Features Added`, `### Breaking Changes`, `### Bugs Fixed`, `### Other Changes`, once each and in that order;
- has `### Breaking Changes` (`None.` when it breaks nothing);
- lists breaking changes only as a minor bump over the entry before it, and lists none only as a patch bump;
- has a date once it is a released version rather than `## [Unreleased]`.

Entries up to 0.4.0 keep the Keep a Changelog headers they were published with (`LEGACY_THROUGH` in `sdk/docs/lib/changelog.mjs`).
`release-check` reads the dated 0.4.0 entry and its `### Compatibility` policy, so those entries are not rewritten.

release-please writes a `⚠ BREAKING CHANGES` heading first.
The check fails on it and says what to do: rename it to `### Breaking Changes` and move it after `### Features Added`, by editing the release pull request.

## Cutting a release

1. Run the `release-please` workflow from the Actions tab. It opens a release pull request per package that has releasable commits.
2. On the release pull request branch, edit the entry if needed so `node sdk/docs/changelog.mjs` passes, and run `cargo update --workspace` and commit `Cargo.lock`: release-please edits the workspace version in `Cargo.toml` but cannot edit the package array of `Cargo.lock`, so `--locked` builds fail on the branch until the lock follows. Then merge it.
3. Make the release's own preparation changes. `release-check` and `release.yml` name the version they release (`0.4.0` today), so each root release still needs a preparation pull request.
4. A person tags: `vX.Y.Z` for the crates and the image (read by `release.yml`), `prism-code-vX.Y.Z` for the npm package (read by `npm-prism-code.yml`).
5. Set the package's version in `.release-please-manifest.json` to the tagged version if the release was not cut through release-please.

The three npm packages that have never been published (`@prism-code/spec`, `@prism-code/verify`, `create-prism-code`) have no tag yet.
Until each has one, release-please walks the whole history for it, so the manifest alone is not enough: set `bootstrap-sha` in the config to the commit of the first release.

## Options weighed

| Option | Fits | Cost |
| --- | --- | --- |
| release-please, manifest mode (chosen) | One tool for the crate and all four npm packages; computes the bump from commit subjects; the `Cargo.toml` versions are updated through `extra-files`. | Writes `CHANGELOG.md` from subjects, so entries are shorter than the hand-written ones; the breaking heading needs the edit above; free-form subjects are invisible to it. |
| `cargo-release` or `release-plz` | Compute Rust bumps. | Rust only; the npm packages stay manual. `release-plz` publishes to crates.io by default. |
| The changelog check alone, no bot | Keeps the hand-written entries. | A person chooses the bump and the check refuses a wrong one. |
