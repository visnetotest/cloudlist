# TRD-003: Unified Engine Architecture

## 1. Status

**Decided**

## 2. Context

The previous generation of the tool relied on an external orchestrator model. While this approach offered some flexibility, it also introduced significant operational complexity, multiple potential points of failure, and a disjointed user experience. Managing separate components for discovery, evaluation, and remediation is cumbersome and inefficient.

## 3. Decision

`cloudscanner` will be architected as a single, cohesive, and powerful binary that manages the entire asset management lifecycle internally. The core workflow is defined as a unified pipeline:

**Discover -> Evaluate -> Remediate**

This integrated approach rejects the previous external orchestrator model in favor of a streamlined, all-in-one solution.

### 3.1. Discovery-Only Mode

To maintain the utility of the original `cloudlist` tool for pure asset inventory, the engine will support a **Discovery-Only Mode**.

*   **Trigger:** This mode will be activated by default if the user does not provide a policy configuration (e.g., via a `--policy-file` flag).
*   **Workflow:** In this mode, the "Evaluate" and "Remediate" stages of the pipeline are bypassed. The workflow is reduced to:
    **Discover -> Report**
*   **Output:** The application will output the discovered assets directly in a user-specified format (e.g., JSON, list), providing a simple and efficient way to perform cloud asset inventory.

This ensures that the core discovery functionality remains a first-class feature, independent of the policy engine.

## 4. Consequences

### 4.1. Advantages

*   **Streamlined User Experience:** The entire workflow can be invoked with a single command and configured via a single file, dramatically simplifying operation.
*   **Simplified Deployment:** A single, self-contained binary is easy to deploy and manage across different environments (laptops, CI/CD, containers).
*   **Tighter Integration & Performance:** Components can pass data in-memory (e.g., from the Discovery Engine to the Policy Engine) without the overhead of serialization or intermediate storage, leading to significant performance gains.
*   **Reduced Complexity:** Eliminates the need for external scripting, cron jobs, or other tools to glue the different stages of the process together.
*   **Dual-Purpose Utility:** The tool can function as both a simple asset discovery tool and a full policy and remediation engine, depending on the user's command-line arguments.

### 4.2. Disadvantages

*   **Monolithic Nature:** The application is more monolithic compared to a microservice-style architecture. However, this is a deliberate trade-off for operational simplicity and is mitigated by the modular, plugin-based design for providers.
*   **Single Point of Failure:** As a single process, a crash in one component could halt the entire workflow. This will be mitigated through robust error handling and the inherent stability provided by Rust.

## 5. Reference

This decision is a core principle outlined in the main technical specification: [CLOUDSCANNER_TECHNICAL_SPECIFICATION.md#1.2.-Architectural-Vision:-The-Unified-Engine](./CLOUDSCANNER_TECHNICAL_SPECIFICATION.md#1.2.-Architectural-Vision:-The-Unified-Engine)
