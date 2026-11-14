# Changelog

## [0.2.0](https://github.com/UOR-Foundation/prismpm/compare/prism-code-verify-v0.1.0...prism-code-verify-v0.2.0) (2026-11-14)


### ⚠ BREAKING CHANGES

* **holo:** holo v5 is a UORC archive named by κ (ADR-017) ([#663](https://github.com/UOR-Foundation/prismpm/issues/663))

### Features Added

* **conformance:** the Rust runtime runs @prism-code/conformance ([#590](https://github.com/UOR-Foundation/prismpm/issues/590)) ([d9373f9](https://github.com/UOR-Foundation/prismpm/commit/d9373f952cfc5ef1524b79791346085ef60f7a9d))
* **examples:** Payment Approval, the second reference archive (Part of [#278](https://github.com/UOR-Foundation/prismpm/issues/278)) ([#418](https://github.com/UOR-Foundation/prismpm/issues/418)) ([17044e1](https://github.com/UOR-Foundation/prismpm/commit/17044e11702e30e4d02450269eb821809731a065))
* **holo:** holo v5 is a UORC archive named by κ (ADR-017) ([#663](https://github.com/UOR-Foundation/prismpm/issues/663)) ([7b48b4d](https://github.com/UOR-Foundation/prismpm/commit/7b48b4d4939a7fa42c623c4a596357384fbe1566))
* **prism-code:** canonical path, integrity fails every policy, refuse text, local by default ([#648](https://github.com/UOR-Foundation/prismpm/issues/648)) ([6b2c684](https://github.com/UOR-Foundation/prismpm/commit/6b2c6844e3df025ca6ddb25a4052695dcc75805d))
* **prism-code:** check reaches P2 from signed statements (Refs [#270](https://github.com/UOR-Foundation/prismpm/issues/270)) ([b3abbb0](https://github.com/UOR-Foundation/prismpm/commit/b3abbb0f0948a03112b4065fd49a2a5767849d38))
* **prism-code:** provider services in prism.toml ([#294](https://github.com/UOR-Foundation/prismpm/issues/294)) ([15d0f30](https://github.com/UOR-Foundation/prismpm/commit/15d0f30461845bc78a206be086590e6241b19c8e))
* **prism-code:** push and pull holos with attestations as OCI referrers ([#292](https://github.com/UOR-Foundation/prismpm/issues/292)) ([#417](https://github.com/UOR-Foundation/prismpm/issues/417)) ([116b19d](https://github.com/UOR-Foundation/prismpm/commit/116b19d1131546713b6628d76169423259a320e4))
* **prism-code:** run provider services from prism.toml ([#294](https://github.com/UOR-Foundation/prismpm/issues/294)) ([024504f](https://github.com/UOR-Foundation/prismpm/commit/024504fd1fa50196fc3b9cbd1266b9762a9c7794))
* **prism-code:** sessions, the hologram:guest/core-wasm@2 world (Part of [#629](https://github.com/UOR-Foundation/prismpm/issues/629)) ([#638](https://github.com/UOR-Foundation/prismpm/issues/638)) ([9d31247](https://github.com/UOR-Foundation/prismpm/commit/9d312473d32df18d95356f6d9b48f48e6d37fb56))
* **spec:** provider declarations in prism/manifest/1 and the providers capability kind ([#294](https://github.com/UOR-Foundation/prismpm/issues/294)) ([099b783](https://github.com/UOR-Foundation/prismpm/commit/099b783500cafa67d3f4771a0915b212616eb155))
* **verify:** histories and prism replay with the prism/replay/v1 statement (Refs [#284](https://github.com/UOR-Foundation/prismpm/issues/284)) ([#415](https://github.com/UOR-Foundation/prismpm/issues/415)) ([1466f53](https://github.com/UOR-Foundation/prismpm/commit/1466f537dbb9c2d3653ff67db9f8e2bd2c8a7e5c))
* **verify:** policy files for prism-code check, in Rego (Refs [#285](https://github.com/UOR-Foundation/prismpm/issues/285)) ([#414](https://github.com/UOR-Foundation/prismpm/issues/414)) ([cc5ff83](https://github.com/UOR-Foundation/prismpm/commit/cc5ff83cc76d626296a732c411983920d7b2b0aa))


### Bugs Fixed

* **verify:** say plainly that a built holo does not carry its proof ([#600](https://github.com/UOR-Foundation/prismpm/issues/600)) ([46b13cc](https://github.com/UOR-Foundation/prismpm/commit/46b13cc72be35af3919a654438096adcbf0cd6bd))
