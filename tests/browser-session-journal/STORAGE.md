# Private storage component

`session-storage.mjs` provides IndexedDB mechanics admitted by the generated
retention model: content-addressed immutable objects, exact-frontier compare-and-
swap, shared-root retention and acknowledgement after transaction completion.
It grants no identity, authority, freshness or application acceptance.

27 September 2026: three canonical-framing tests and nine real Chromium storage
journeys pass, including separate tabs, persistent browser restart, rollback,
quota failure and delayed completion acknowledgement. The fixture executes
generated retention Wasm, not a JavaScript transition substitute.

- Wasm: `92ce26f8513293215320f1aa34922ea4704c92539d63e5684bdd2c88479beddd`.
- Receipt: `c375beb263b98013dc037d6fa765039a6117349e42caf32ec2fc9b88af18be2a`.

Full retention source-owner verification, native browser-transcript replay,
host mutation checks, joint storage maxima and authenticated journal composition
remain required. Snapshots bind the reference frontier; callers must read and
hash the complete authenticated closure before executing or recovering effects.
