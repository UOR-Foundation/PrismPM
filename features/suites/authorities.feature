Feature: authorities

  @AU-01 @build
  Scenario: Every standards claim uses a typed immutable authority binding distinct from its executable oracle.
    Given the locked PrismPM production fixture
    When the AU-01 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-02 @build
  Scenario: Authority resolution writes a canonical reviewable lock and locked resolution rejects drift.
    Given the locked PrismPM production fixture
    When the AU-02 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-03 @build
  Scenario: Fetch verifies signatures and digests before atomically publishing immutable cached inputs.
    Given the locked PrismPM production fixture
    When the AU-03 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-04 @build
  Scenario: Authority and oracle verification reproduces from a populated cache with network access disabled.
    Given the locked PrismPM production fixture
    When the AU-04 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-05 @build
  Scenario: Oracle runners are digest-bound, resource-bounded, networkless, and report exact covered and uncovered requirements.
    Given the locked PrismPM production fixture
    When the AU-05 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-06 @build
  Scenario: Official positive and negative corpora plus Prism mutations detect stale, bypassed, changed, or always-passing oracles.
    Given the locked PrismPM production fixture
    When the AU-06 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-07 @build
  Scenario: The imported OSCAL JSON structural oracle executes complete pinned schema corpora and bounded subject-capture adversaries without claiming implemented controls or whole-standard acceptance.
    Given the locked PrismPM production fixture
    When the AU-07 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-08 @build
  Scenario: MCP 2026-07-28 is bound as an interface kind to its specification, schema, tasks extension, conformance suite and MCP Apps authorities, with the schema oracle and the server conformance oracle as separate evidence kinds, and a locked suite whose requirements file differs from the frozen one is refused.
    Given the locked PrismPM production fixture
    When the AU-08 contract is exercised
    Then its positive evidence passes and its planted defect is rejected

  @AU-09 @build
  Scenario: The MCP schema oracle and the MCP server conformance oracle execute in the networkless sandbox against positive and planted subjects, and a conformance build with one required scenario failing is rejected with the oracle's diagnostic.
    Given the locked PrismPM production fixture
    When the AU-09 contract is exercised
    Then its positive evidence passes and its planted defect is rejected
