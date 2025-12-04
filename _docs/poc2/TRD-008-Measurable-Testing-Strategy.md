# TRD-008: Measurable Testing Strategy

## 1. Status

**Proposed**

## 2. Context

Technical Requirement TR-8 mandates a "comprehensive testing strategy," but this is too abstract to be actionable. To ensure the reliability and correctness of the `cloudscanner` PoC, we need to define specific, measurable goals for our testing efforts. This provides a clear "definition of done" for quality assurance and gives us confidence that the core architecture is sound.

## 3. Decision

The `cloudscanner` testing strategy will be composed of three distinct layers, each with a mandatory, measurable goal for the PoC.

1.  **Unit Testing:**
    *   **Scope:** Individual functions and components (structs/modules) within the core engine and provider plugins. Logic for parsing, data transformation, and configuration loading are key candidates.
    *   **Method:** Standard `cargo test` framework. Cloud provider SDKs and other external dependencies will be mocked.
    *   **Measurable Goal:** The core engine components (including the Policy Engine, Discovery Engine, and Reporter) **must achieve a minimum of 80% unit test line coverage**.

2.  **Integration Testing:**
    *   **Scope:** The interaction between two or more components. For example, testing the Core Engine's ability to correctly load and communicate with a provider plugin via the ABI.
    *   **Method:** `cargo test` tests that operate on slightly larger components. For example, loading a compiled mock plugin and calling its functions.
    *   **Measurable Goal:** At least one integration test **must be implemented to verify the Plugin Loader can successfully load and execute a mock provider plugin**.

3.  **End-to-End (E2E) Testing:**
    *   **Scope:** The entire, compiled `cloudscanner` binary operating in a realistic scenario.
    *   **Method:** A test script that runs as part of the CI/CD pipeline. It will use infrastructure-as-code to provision resources in a sandboxed AWS account, run `cloudscanner`, and assert the output.
    *   **Measurable Goal:** The PoC **must include at least one end-to-end test that runs in CI**. This test will:
        1.  Create an AWS S3 bucket with public read access.
        2.  Execute the `cloudscanner` binary against the sandbox account with a policy to detect public S3 buckets.
        3.  Assert that the JSON output contains a violation for the created bucket.
        4.  Tear down the created resources.

## 4. Consequences

### 4.1. Advantages

*   **High Confidence:** Provides strong guarantees that the system works as intended, from individual functions to the complete workflow.
*   **Prevents Regressions:** A comprehensive test suite makes it safer and faster to refactor code or add new features.
*   **Clear Development Targets:** The measurable goals give developers a clear target for what constitutes "well-tested" code.
*   **Automated Quality Gate:** The CI-based E2E test acts as an automated quality gate, preventing broken builds from being released.

### 4.2. Disadvantages

*   **Infrastructure Overhead:** Requires setting up and managing a dedicated, sandboxed cloud account for E2E testing.
*   **Development Time:** Writing and maintaining high-quality tests, especially E2E tests with infrastructure setup, requires a non-trivial amount of development effort.
*   **CI Complexity:** The CI/CD pipeline becomes more complex, as it needs to handle cloud credentials and the infrastructure provisioning steps for E2E tests.

## 5. Reference

This TRD provides the concrete, measurable goals for the high-level requirement defined in: [poc2.md#3.-Technical-Requirements](./poc2.md#3.-Technical-Requirements) (TR-8).
