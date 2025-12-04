# TRD-014: Graceful Error and Failure Handling

## 1. Status

**Proposed**

## 2. Context

A `cloudscanner` run may involve multiple providers, each with its own configuration and potential failure points (e.g., invalid credentials, network issues, API throttling). A failure in one part of the scan should not terminate the entire process. The tool must be resilient enough to complete as much of the scan as possible, providing partial results rather than crashing completely.

## 3. Decision

The `cloudscanner` Core Engine will be designed to handle errors gracefully and continue processing where possible.

1.  **Provider-Level Isolation:** The main discovery loop will iterate through the configured providers. If any single provider plugin returns an error during its `discover_assets()` call, the Core Engine will:
    *   Log the error clearly to `stderr` (respecting verbosity settings).
    *   Mark that provider as failed.
    *   Continue execution with the next configured provider.

2.  **Error Propagation:** Errors will be modeled as a distinct type within the application. The `discover_assets()` function in the provider ABI will return a `Result<_, Error>` type, allowing the Core Engine to robustly distinguish between success and failure.

3.  **Final Report:** The final report will clearly indicate which providers were successful and which failed, ensuring the user understands the scope of the completed scan.

## 4. Consequences

### 4.1. Advantages

*   **Increased Resilience:** The tool is more robust and useful in real-world scenarios where partial failures are common.
*   **Better User Experience:** Provides a more complete picture of the environment, even if some parts are inaccessible.
*   **Prevents cascading failures** where one small misconfiguration would render the entire tool useless.

### 4.2. Disadvantages

*   **More Complex Control Flow:** The Core Engine's discovery loop is slightly more complex, as it must handle the `Result` of each provider call.

## 5. Reference

This decision supports a core reliability requirement referenced in: [poc2.md#2.-Technical-Requirements](./poc2.md#2.-Technical-Requirements)
