---
title: Errors reference
description: Every error code the runtime raises and every CLI exit code.
available_in: "0.5"
---

Source: `prism-code` 0.9.1 source and type declarations, `sdk/docs/cli.json` 2026-10-30.

## Runtime errors

Status: the typed error classes are in the prism-code source and are not in the published 0.2.0. The table lists every code the source raises.

```ts
export class PrismError extends Error {
  static readonly code: string;
  code: string;
  check?: string;
  diagnostic?: string;
  details?: unknown;
  verdict?: Verdict;
  reasons?: VerdictReason[];
  constructor(
    message: string,
    options?: { code?: string; check?: string; diagnostic?: string; details?: unknown; verdict?: Verdict; reasons?: VerdictReason[]; cause?: unknown },
  );
}

export class PrismCodeError extends PrismError {
  code: string;
  constructor(code: string, message: string, details?: unknown);
}
```

The typed errors, their codes and the verdict checks that map to them are on the error types page.

| Code                    | Raised in                                                                      |
| ----------------------- | ------------------------------------------------------------------------------ |
| `archive.blob`          | `src/archive.mjs`                                                              |
| `archive.bounds`        | `src/archive.mjs`, `src/attestations.mjs`, `src/uorc.mjs`                      |
| `archive.directory`     | `src/archive.mjs`                                                              |
| `archive.extension`     | `src/archive.mjs`                                                              |
| `archive.flags`         | `src/archive.mjs`                                                              |
| `archive.input`         | `src/archive.mjs`                                                              |
| `archive.kind`          | `src/archive.mjs`                                                              |
| `archive.magic`         | `src/archive.mjs`, `src/uorc.mjs`                                              |
| `archive.manifest`      | `src/archive.mjs`                                                              |
| `archive.metadata`      | `src/archive.mjs`                                                              |
| `archive.model`         | `src/archive.mjs`, `src/index.mjs`                                             |
| `archive.provenance`    | `src/archive.mjs`                                                              |
| `archive.version`       | `src/archive.mjs`                                                              |
| `attestation.bundle`    | `src/attestations.mjs`                                                         |
| `attestation.duplicate` | `src/attestations.mjs`                                                         |
| `attestation.envelope`  | `src/attestations.mjs`                                                         |
| `attestation.footer`    | `src/attestations.mjs`                                                         |
| `attestation.format`    | `src/attestations.mjs`                                                         |
| `attestation.input`     | `src/attestations.mjs`                                                         |
| `attestation.lines`     | `src/attestations.mjs`                                                         |
| `attestation.position`  | `src/attestations.mjs`                                                         |
| `attestation.statement` | `src/attestations.mjs`                                                         |
| `attestation.subject`   | `src/attestations.mjs`                                                         |
| `auth.unavailable`      | `src/index.mjs`                                                                |
| `axioms.probe`          | `src/axioms.mjs`                                                               |
| `axioms.project`        | `src/axioms.mjs`                                                               |
| `capabilities.invalid`  | `src/capabilities.mjs`, `src/index.mjs`, `src/networks.mjs`, `src/volumes.mjs` |
| `capabilities.kind`     | `src/capabilities.mjs`                                                         |
| `capabilities.path`     | `src/node.mjs`                                                                 |
| `client.option`         | `src/index.mjs`                                                                |
| `closure.files`         | `src/browser.mjs`, `src/closure.mjs`                                           |
| `closure.glue`          | `src/closure.mjs`                                                              |
| `closure.model`         | `src/closure.mjs`                                                              |
| `closure.provenance`    | `src/closure.mjs`                                                              |
| `container.ambiguous`   | `src/containers.mjs`                                                           |
| `container.input`       | `src/containers.mjs`                                                           |
| `container.missing`     | `src/containers.mjs`                                                           |
| `container.option`      | `src/containers.mjs`                                                           |
| `container.running`     | `src/containers.mjs`                                                           |
| `container.state`       | `src/containers.mjs`                                                           |
| `container.store`       | `src/node.mjs`                                                                 |
| `core.allocation`       | `src/closure.mjs`, `src/core.mjs`                                              |
| `core.compile`          | `src/core.mjs`                                                                 |
| `core.exports`          | `src/core.mjs`                                                                 |
| `core.request`          | `src/core.mjs`                                                                 |
| `core.response`         | `src/core.mjs`, `src/session.mjs`                                              |
| `dev.path`              | `src/dev-cli.mjs`                                                              |
| `dev.port`              | `src/dev-server.mjs`                                                           |
| `dev.project`           | `src/dev-cli.mjs`                                                              |
| `docker.missing`        | `src/host.mjs`                                                                 |
| `docker.mount`          | `src/docker.mjs`, `src/host.mjs`                                               |
| `export.directory`      | `src/export.mjs`                                                               |
| `export.exists`         | `src/export.mjs`                                                               |
| `export.reference`      | `src/export.mjs`                                                               |
| `export.release`        | `src/export.mjs`                                                               |
| `export.unsafe`         | `src/export.mjs`                                                               |
| `hosted.coverage`       | `src/hosted.mjs`                                                               |
| `hosted.layout`         | `src/hosted.mjs`                                                               |
| `hosted.unreachable`    | `src/hosted.mjs`                                                               |
| `image.ambiguous`       | `src/image-store.mjs`                                                          |
| `image.corrupt`         | `src/image-store.mjs`                                                          |
| `image.missing`         | `src/image-store.mjs`                                                          |
| `image.name`            | `src/image-store.mjs`                                                          |
| `image.store`           | `src/image-store.mjs`                                                          |
| `images.unavailable`    | `src/index.mjs`                                                                |
| `init.template`         | `src/init.mjs`                                                                 |
| `inspect.kind`          | `src/images-namespace.mjs`, `src/index.mjs`                                    |
| `inspect.selector`      | `src/inspect.mjs`                                                              |
| `load.fetch`            | `src/browser.mjs`                                                              |
| `load.path`             | `src/images-namespace.mjs`, `src/node.mjs`                                     |
| `load.source`           | `src/images-namespace.mjs`, `src/index.mjs`                                    |
| `lock.image`            | `src/docker.mjs`, `src/init.mjs`                                               |
| `manifest.env`          | `src/manifest.mjs`                                                             |
| `manifest.invalid`      | `src/manifest.mjs`                                                             |
| `manifest.overlay`      | `src/manifest.mjs`                                                             |
| `manifest.path`         | `src/node.mjs`                                                                 |
| `manifest.secret`       | `src/manifest.mjs`                                                             |
| `manifest.toml`         | `src/toml.mjs`                                                                 |
| `middleware.name`       | `src/middleware.mjs`                                                           |
| `middleware.relative`   | `src/middleware.mjs`                                                           |
| `middleware.shape`      | `src/middleware.mjs`, `src/tracing.mjs`                                        |
| `middleware.step`       | `src/middleware.mjs`                                                           |
| `mount.environment`     | `src/browser.mjs`                                                              |
| `networks.exists`       | `src/networks.mjs`                                                             |
| `networks.option`       | `src/networks.mjs`                                                             |
| `networks.protected`    | `src/networks.mjs`                                                             |
| `networks.store`        | `src/networks.mjs`                                                             |
| `networks.unavailable`  | `src/networks.mjs`                                                             |
| `open.agent`            | `src/open.mjs`                                                                 |
| `open.clipboard`        | `src/open.mjs`                                                                 |
| `open.host`             | `src/open.mjs`                                                                 |
| `open.parameter`        | `src/open.mjs`                                                                 |
| `open.sentence`         | `src/open.mjs`                                                                 |
| `open.terminal`         | `src/open.mjs`                                                                 |
| `open.ticket`           | `src/open.mjs`                                                                 |
| `open.url`              | `src/open.mjs`                                                                 |
| `provenance.archive`    | `src/provenance.mjs`                                                           |
| `provenance.duplicate`  | `src/provenance.mjs`                                                           |
| `provenance.input`      | `src/provenance.mjs`                                                           |
| `provenance.lock`       | `src/provenance.mjs`                                                           |
| `provenance.mode`       | `src/provenance.mjs`                                                           |
| `remote.api`            | `src/remote-github.mjs`                                                        |
| `remote.artifact`       | `src/remote-github.mjs`                                                        |
| `remote.attestation`    | `src/remote-github.mjs`                                                        |
| `remote.auth`           | `src/remote-github.mjs`                                                        |
| `remote.commit`         | `src/remote-github.mjs`                                                        |
| `remote.dirty`          | `src/remote-github.mjs`                                                        |
| `remote.fingerprint`    | `src/remote-github.mjs`                                                        |
| `remote.permission`     | `src/remote-github.mjs`                                                        |
| `remote.provenance`     | `src/remote-github.mjs`                                                        |
| `remote.push`           | `src/remote-github.mjs`                                                        |
| `remote.ref`            | `src/remote-github.mjs`                                                        |
| `remote.repository`     | `src/remote-github.mjs`                                                        |
| `remote.run`            | `src/remote-github.mjs`                                                        |
| `remote.signer`         | `src/remote-github.mjs`                                                        |
| `remote.subject`        | `src/remote-github.mjs`                                                        |
| `remote.timeout`        | `src/remote-github.mjs`                                                        |
| `remote.workflow`       | `src/remote-github.mjs`                                                        |
| `session.busy`          | `src/session.mjs`                                                              |
| `session.closed`        | `src/session.mjs`                                                              |
| `session.required`      | `src/index.mjs`                                                                |
| `session.unsupported`   | `src/index.mjs`                                                                |
| `setup.agent`           | `src/setup.mjs`                                                                |
| `setup.command`         | `src/setup.mjs`                                                                |
| `system.event`          | `src/system.mjs`                                                               |
| `system.events`         | `src/system.mjs`                                                               |
| `system.lock`           | `src/node.mjs`, `src/system.mjs`                                               |
| `system.option`         | `src/system.mjs`                                                               |
| `tier.project`          | `src/tiers.mjs`                                                                |
| `view.bounds`           | `src/view.mjs`                                                                 |
| `view.entry`            | `src/view.mjs`                                                                 |
| `view.magic`            | `src/view.mjs`                                                                 |
| `view.order`            | `src/view.mjs`                                                                 |
| `view.version`          | `src/view.mjs`                                                                 |

## CLI exit codes

| Code | Name          | Meaning                                                                                               |
| ---- | ------------- | ----------------------------------------------------------------------------------------------------- |
| 0    | pass          | The verb did what you asked and every check it ran passed.                                            |
| 1    | fail          | A check failed, an intent sentence is unproved, or a replay diverged.                                 |
| 2    | cannot verify | No verdict was reached: a missing attestation, an unreachable registry or an unknown format revision. |

## Next steps

- [CLI reference](/docs/reference/cli)
- [Error types](/docs/api/errors)
- [Load a holo](/docs/api/load)

Generated from prism-code 0.9.1, sdk/docs/cli.json 2026-10-30, @prism-code/spec 2026-10-23 at 168c87148ba570763a573a4b3a0d374fb7183e31.
