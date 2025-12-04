# PoC Plan: `cloudscanner` Next-Gen

## 1. Overview

This document outlines the plan for a Proof of Concept (PoC) to validate the architecture and core capabilities of `cloudscanner`, a next-generation cloud security and compliance platform. This PoC is the second iteration, building on the lessons learned from the first prototype, and is referred to as **PoC2**.

## 2. Technical Requirements

The PoC must meet the following technical requirements, which will be validated through a series of specific, measurable goals (see Section 4).

| ID | Requirement | Description |
| :--- | :--- | :--- |
| **TR-1** | **Plugin-Based Architecture** | The engine must support a plugin-based architecture for discovering resources, allowing for easy extension to new cloud providers. |
| **TR-2** | **High-Performance Language** | The core engine must be written in a modern, high-performance, and safe systems programming language. |
| **TR-3** | **Unified Engine** | The tool must function as a single, unified binary that handles the entire workflow (discover, evaluate, report) internally. |
| **TR-4** | **Human-Readable Policies** | The policy engine must support simple, human-readable rules (e.g., in YAML) for defining security and compliance checks. |
| **TR-5** | **Structured Configuration** | All configuration must be handled through a clear, structured file format like TOML. |
| **TR-6** | **Event-Driven Discovery** | The engine must have the capability to listen to real-time cloud event streams and trigger targeted, immediate scans. |
| **TR-7** | **Context-Aware Asset Graph** | The engine must build a graph of assets and their relationships to enable advanced risk analysis. |
| **TR-8** | **Comprehensive Testing** | The PoC must include a robust and measurable testing strategy, including unit, integration, and end-to-end tests. |

## 3. High-Level Architecture

The `cloudscanner` PoC will be built around a unified core engine that orchestrates a series of components, each responsible for a specific stage of the security assessment pipeline. All provider-specific logic will be isolated into dynamically loaded plugins.

### 3.1. High-Level Component Diagram

```mermaid
componentDiagram
    package "cloudscanner Core Engine" {
        [CLI] --> [Config Loader]
        [Config Loader] --> [Plugin Loader]
        [Config Loader] --> [Discovery Engine]
        [Plugin Loader] --> [Discovery Engine]
        [Discovery Engine] --> [Policy Engine]
        [Discovery Engine] --> [Asset Graph]
        [Policy Engine] --> [Reporter]
        [Asset Graph] --> [Reporter]
    }

    package "Provider Plugins (Dynamic Libraries)" {
        [AWS Provider] as AWS
        [GCP Provider] as GCP
    }

    [Plugin Loader] ..> AWS : "loads"
    [Plugin Loader] ..> GCP : "loads"
    AWS ..> [Discovery Engine] : "registers & provides assets"
    GCP ..> [Discovery Engine] : "registers & provides assets"
```

### 3.2. Core Components

*   **CLI:** The command-line interface for user interaction.
*   **Config Loader:** Reads and validates the `cloudscanner.toml` file.
*   **Plugin Loader:** Loads and manages the provider plugins (shared libraries).
*   **Discovery Engine:** Orchestrates the asset discovery process using the loaded plugins.
*   **Asset Graph:** Builds and maintains the in-memory graph of discovered assets and their relationships.
*   **Policy Engine:** Evaluates the discovered assets against user-defined policies.
*   **Reporter:** Formats the results into the desired output format, such as a **JSON list (basic mode)** or a **DOT graph (advanced mode)**.

## 4. PoC Goals & Success Criteria

The PoC will be considered successful upon the completion and demonstration of the following concrete goals. These goals are designed to be a direct, measurable validation of the technical requirements.

| Goal ID | Title | Description | Related TR |
| :--- | :--- | :--- | :--- |
| **G-1** | **AWS S3 Provider Plugin** | Implement a provider plugin for AWS that can discover S3 buckets. This plugin will be a dynamically loaded shared library. | TR-1, TR-2 |
| **G-2** | **YAML Policy for Public S3** | Implement a policy in the core engine that can identify S3 buckets that are publicly accessible. | TR-4 |
| **G-3** | **End-to-End Scan** | Demonstrate a complete, end-to-end scan using the unified engine: `discover` (AWS S3) -> `evaluate` (public S3 policy) -> `report` (JSON output). | TR-3, TR-5 |
| **G-4** | **Live Event Detection** | Demonstrate that creating a new, public S3 bucket in a test AWS account triggers an immediate, event-driven scan that flags the new resource within 60 seconds. | TR-6 |
| **G-5** | **Attack Path Visualization** | Demonstrate the construction of an asset graph from a test environment and export it to a DOT file. The graph must show a connection from an EC2 instance to an assumed IAM role to an S3 bucket. | TR-7 |
| **G-6** | **CI/CD Quality Gate** | Implement an automated end-to-end test in the CI/CD pipeline that fails the build if a known, critical misconfiguration is not detected. | TR-8 |

## 5. Implementation and Testing Strategy

This section outlines the practical steps for implementing and testing the PoC goals.

### 5.1. Localstack Test Environment for AWS

To support the development and testing of AWS provider plugins and policies (Goals G-1, G-2, G-3, G-6), a local testing environment using Localstack will be established. This provides a fast, offline, and cost-effective way to simulate AWS services.

#### Running the Test

A `docker-compose` setup is provided to orchestrate the test environment.

```bash
# Run from the project root
docker-compose -f _docs/poc2/test/test-aws-localstack.yaml up
```

This command will:

1.  Start a `localstack` container with S3 and EC2 services enabled.
2.  Build a Docker image for the `cloudlist` tool.
3.  Run the `cloudlist` container to execute scans against the `localstack` instance.

#### Status

This initial setup is complete and functional. The `docker-compose` configuration successfully starts both the `localstack` and `cloudlist` services. However, `cloudlist` currently does not discover any resources, as no resources have been created in the Localstack environment.

#### Next Steps

1.  **Create AWS Resources:** A script is needed to create dummy AWS resources (e.g., EC2 instances, S3 buckets) in the Localstack container after it starts. This will provide assets for `cloudlist` to discover.
2.  **Verify Discovery:** Once resources are created, re-run the test to confirm that `cloudlist` successfully discovers and lists them.
3.  **Expand Service Coverage:** Add more AWS services to the `SERVICES` environment variable in the `docker-compose` file and create corresponding resources to test a wider range of `cloudlist`'s capabilities.

## 6. Technical Decision Records (TRDs)

Detailed architectural and design decisions are captured in the following Technical Decision Records (TRDs), located in this directory:

*   [TRD-001-Provider-Plugin-ABI.md](./TRD-001-Provider-Plugin-ABI.md)
*   [TRD-002-Language-Choice.md](./TRD-002-Language-Choice.md)
*   [TRD-003-Unified-Engine.md](./TRD-003-Unified-Engine.md)
*   [TRD-004-Policy-Engine.md](./TRD-004-Policy-Engine.md)
*   [TRD-005-Configuration.md](./TRD-005-Configuration.md)
*   [TRD-009-Event-Driven-Discovery.md](./TRD-009-Event-Driven-Discovery.md)
*   [TRD-010-Asset-Graph.md](./TRD-010-Asset-Graph.md)
*   [TRD-008-Measurable-Testing-Strategy.md](./TRD-008-Measurable-Testing-Strategy.md)

## 7. Questions for Follow-up

This section captures important, unanswered questions that are outside the immediate scope of PoC2 but must be addressed for the production version of `cloudscanner`.

*   **How will we manage the memory consumption of the asset graph for extremely large cloud environments?** (See TRD-010 for initial thoughts on pruning and disk-based storage.)
*   **What is the long-term strategy for the Policy Engine?** While the simple YAML format is ideal for the PoC, will we eventually need the power of a more expressive language like Rego or Datalog? (See TRD-004.)
*   **How will the dynamic plugin (shared library) model work with cross-platform builds and static linking?** This is a known challenge in the Rust ecosystem and needs a clear distribution strategy.
*   **How will automated remediation be implemented safely?** The PoC focuses on discovery and evaluation, but the architecture must eventually support safe, automated remediation actions (e.g., `dry-run` modes, approval workflows).
