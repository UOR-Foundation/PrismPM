# Cross-engine test infrastructure

Private test tooling, not application or installed-SDK acceptance. The default
remains Chromium; additional engines require explicit closed options.

Pinned browser devcontainer image:
`ghcr.io/uor-foundation/prismpm-sdk-candidate@sha256:60226bc791d4c0e5613402a6be7e63f4963d3faf7f327befcf56fc0e41d0ce21`.
Run as UID 1000 with a read-only workspace/root filesystem, no external network,
and writable temporary profile/oracle directories. Current SDK oracle files
were copied from `prismpm-dev`, not installed on the host. Both complete 8852-file
inventories match: `83d55369f5efb93d2378bf9d4e8900077b42dd667f9b723b6fefce0128444262`
(sorted `[path,kind,SHA256-or-link-target]` JSON). Node: 22.23.2; Playwright and
its selected core: 1.62.1. Exact browser versions are enforced by the helper.

```sh
node --test tests/browser-cross-engine/server.test.mjs sdk/browser/store.test.mjs sdk/browser/boundary.test.mjs scripts/browser-api-sdk-check.test.mjs
node --test tests/browser-cross-engine/store.test.mjs
```

27 September 2026: 43/43 helper/boundary/SDK-binding tests passed, followed by
15/15 matrix tests executing all four unchanged Store journeys in Chromium,
Firefox and WebKit. The default standalone Store owner still registers four
tests. Retention/journal matrix acceptance remains a separate owner.

All three engines report strict IndexedDB transactions. This is not eviction-
proof persistence: the separate ephemeral-context probe returned `persist=false`
in Chromium, awaited permission in Firefox, and lacked the persistence API in
WebKit. Bookworm's copied Noble browsers lack GTK dependencies; use this pinned
Noble environment rather than skipping Firefox/WebKit or installing host tools.
