Feature: sdk

  @DK-10 @build
  Scenario: The LexLean workspace reducer executes its complete bounded state-transition corpus through freshly generated Rust and Core-Wasm; authentication, durable effects, and application acceptance remain separate obligations.
    Given the complete modeled workspace corpus and pinned production toolchains
    When fresh generated standard Rust, no_std Rust, and Core-Wasm execute every case
    Then valid transitions, every modeled rejection, and maximum-size boundaries agree

  @DK-07 @build
  Scenario: The browser cryptography host boundary signs bounded domain-separated bytes with nonextractable keys and detects changed authors, contexts, payloads, and persisted key bindings without assigning organizational authority.
    Given nonextractable Web Cryptography identities
    When bounded domain-separated records are signed and independently verified
    Then changed authors, contexts, payloads, and key bindings are rejected

  @DK-08 @build
  Scenario: The browser storage host boundary retains identity keys and content-addressed bytes across reopening and atomically rejects stale heads, partial writes, corruption, and resource-policy changes.
    Given independent browser connections to a bounded IndexedDB namespace
    When signed identity keys and atomic object-head updates are retained and reopened
    Then concurrent conflicts and corrupted or over-limit records cannot be acknowledged

  @DK-09 @build
  Scenario: The browser peer host boundary exchanges bounded ordered bytes over manually paired direct WebRTC sessions and rejects malformed signaling, framing, queue overflow, expired operations, and closed sessions without claiming peer authority or internet-wide discovery.
    Given independent browser contexts and explicit manual pairing
    When real ordered peer data channels exchange opaque bounded bytes
    Then malformed and excessive traffic, stale signaling, and closed sessions fail explicitly

  @DK-01 @build
  Scenario: One versioned SDK inventory closes over PrismPM, LexLean, Lean, lean4-prod, stdlib, oracles, adapters, and schemas.
    Given the locked PrismPM production fixture with exact platform-indexed and legacy native SDK inventories and review-required update proposals
    When the DK-01 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @DK-02 @build
  Scenario: Signed non-root amd64 and arm64 SDK images use digest-pinned bases and carry OCI, SPDX, and provenance evidence.
    Given the locked PrismPM production fixture
    When the DK-02 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @DK-03 @build
  Scenario: Native Linux archives and OCI SDK execution produce identical platform-independent outputs and diagnostics.
    Given the locked PrismPM production fixture
    When the DK-03 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @DK-04 @build
  Scenario: The complete SDK lock and explicit fetch phase permit all build and verification phases to run locked and offline.
    Given the locked PrismPM production fixture
    When the DK-04 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @DK-05 @build
  Scenario: SDK bootstrap uses the prior accepted SDK, two clean self-rebuilds, and independent formal evidence verification without a trust cycle.
    Given the locked PrismPM production fixture and closed legacy and content-compatible bootstrap evidence contracts
    When the DK-05 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @DK-06 @build
  Scenario: SDK execution rejects undeclared PATH tools, tampered executables, base drift, mutable inputs, and circular self-attestation.
    Given the locked PrismPM production fixture
    When the DK-06 contract is exercised
    Then its positive evidence passes and its planted defect is rejected
