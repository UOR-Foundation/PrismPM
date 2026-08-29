Feature: holo

  @HO-01 @build
  Scenario: Holo artifacts conform to the prismpm/holo/1 JSON schema.
    Given schemas/holo.schema.json
    When a generated model.holo is validated
    Then it conforms strictly to the schema

  @HO-02 @build
  Scenario: Holo serialization follows canonical JSON formatting rules with sorted ASCII keys.
    Given Holo DTOs in memory
    When canonical encoding runs
    Then object keys are sorted by ASCII byte order with no extra whitespace

  @HO-03 @build
  Scenario: Holo content ID is the exact SHA-256 hash of its canonical UTF-8 bytes.
    Given canonical model.holo bytes
    When the SHA-256 hash is computed
    Then it equals the declared Holo content ID

  @HO-04 @build
  Scenario: The Holo projector is a total function from LexLean semantic snapshots.
    Given a valid LexLean semantic snapshot
    When the Holo projector runs
    Then a complete Holo model is produced deterministically

  @HO-05 @build
  Scenario: Holo entity identifiers are qualified strings assigned deterministic zero-based indexes.
    Given a projected Holo model
    When runtime index assignment occurs
    Then entities receive unique, consecutive zero-based Nat indexes

  @HO-06 @build
  Scenario: Holo serialization excludes host paths, timestamps, and unstable environment data.
    Given a generated Holo document
    When document contents are inspected
    Then no timestamps, absolute paths, or host identifiers are present

  @HO-07 @build
  Scenario: Holo decoding rejects noncanonical representations and malformed values.
    Given noncanonical or invalid Holo bytes
    When Holo decoder parses input
    Then decoding fails with registered diagnostic codes

  @HO-08 @build
  Scenario: The emitter-semantics ID uniquely identifies the Holo projector inputs.
    Given model/emitter-inputs.toml
    When input files are hashed
    Then the computed tree digest equals the declared emitter-semantics ID

  @HO-09 @build
  Scenario: Shared golden vectors prove correspondence between Lean normalized models and Holo DTOs.
    Given golden test vectors
    When evaluated in Lean and Rust
    Then normalized models and validity results correspond exactly

  @HO-10 @build
  Scenario: Holo validation verifies internal cross-references before emission.
    Given an in-memory Holo model with dangling references
    When validation executes
    Then dangling references are rejected before serialization
