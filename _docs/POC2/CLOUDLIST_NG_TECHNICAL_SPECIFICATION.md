# Cloudlist-NG: Technical Specification

## 1. Introduction & Vision

This document provides the complete technical specification for `cloudlist-ng`, the next-generation, all-in-one cloud asset discovery, policy, and remediation platform.

### 1.1. Language Justification: Rust

The decision to build `cloudlist-ng` from the ground up in Rust is a strategic one, driven by the need for a truly production-grade, high-performance, and secure platform. Go, while excellent, presents challenges that Rust is uniquely suited to solve:

*   **Memory Safety & Fearless Concurrency:** Rust's ownership model and borrow checker eliminate entire classes of bugs (null pointer dereferences, data races) at compile time. This allows us to build a highly concurrent scanning engine that is both fast and safe, without the need for a garbage collector, which can introduce unpredictable latency.
*   **Performance:** Rust provides C-level performance with high-level abstractions. For a tool that needs to process vast amounts of data from cloud APIs, this raw performance is critical for delivering results quickly.
*   **Reliable Single-Binary Deployment:** Rust compiles to a single, statically-linked native binary with no external runtime dependencies, simplifying deployment across all major platforms and environments, from a security engineer's laptop to a container in a CI/CD pipeline.
*   **Rich Ecosystem:** The Rust ecosystem, via Cargo and crates.io, provides high-quality libraries for asynchronous I/O (`tokio`), observability (`tracing`), and more, allowing us to build a robust application on a solid foundation.

### 1.2. Architectural Vision: The Unified Engine

`cloudlist-ng` fundamentally rejects the previous external orchestrator model. That model, while flexible, introduced operational complexity, multiple points of failure, and a disjointed user experience.

The vision for `cloudlist-ng` is a single, cohesive, and powerful binary that manages the entire asset management lifecycle internally:

**Discover -> Evaluate -> Remediate**

This unified architecture provides a streamlined user experience, simplifies configuration and deployment, and allows for tighter integration and better performance between the core components.

## 2. Core Architecture: A Plugin-First, Modular Design

The architecture is designed to be lean, modular, and extensible from the ground up. The core of the application knows nothing about specific clouds; all provider-specific logic is handled by plugins.

### 2.1. The `cloudlist-ng` Core Crate

The main `cloudlist-ng` binary is the central nervous system. Its responsibilities are strictly limited to:

*   **Configuration Management:** Loading and parsing the `cloudlist-ng.toml` configuration file.
*   **Plugin Management:** Discovering, loading, and managing the lifecycle of provider plugins.
*   **Orchestration:** Driving the main workflow by invoking the Discovery, Policy, and Remediation engines in the correct sequence.
*   **Observability:** Initializing logging and metrics endpoints.

### 2.2. Dynamic Provider Plugin System

A cornerstone of the `cloudlist-ng` architecture is its dynamic plugin system, ensuring the core remains decoupled and extensible.

*   **Provider as a Plugin:** All cloud providers (AWS, GCP, Azure, DigitalOcean, etc.) are implemented as self-contained, dynamically loadable libraries (e.g., `libcloudlist_provider_aws.so`, `cloudlist_provider_aws.dll`). The core application can run without any providers present and will gracefully handle the loading of any compatible plugin found in its designated plugin directory.

*   **Stable Plugin API Contract (ABI):** To ensure long-term stability and compatibility between the core application and a growing ecosystem of plugins, we will define a stable C-style Application Binary Interface (ABI).
    *   We will use the `libloading` crate in the core for dynamic library loading.
    *   The API contract will be defined in a dedicated `cloudlist-ng-api` crate.
    *   All shared structs will use `#[repr(C)]` to guarantee a stable memory layout.
    *   All functions exposed by the plugin will be marked `extern "C"`.

*   **Example Provider API Contract (`cloudlist-ng-api` crate):**

    ```rust
    // In cloudlist-ng-api crate

    use std::collections::HashMap;

    #[repr(C)]
    pub struct Asset {
        pub id: String,
        pub asset_type: String,
        pub provider: String,
        pub region: Option<String>,
        pub public_ipv4: Vec<String>,
        pub public_ipv6: Vec<String>,
        pub dns_name: Option<String>,
        pub metadata: HashMap<String, String>,
    }

    /// The primary trait defining the functionality of a provider plugin.
    /// This is for internal Rust use; the C ABI is the public contract.
    pub trait Provider {
        fn name(&self) -> &'static str;
        fn discover_assets(&self, config: &ProviderConfig) -> Result<Vec<Asset>, String>;
        // More methods for remediation would be added here.
    }

    // The ABI-stable interface exposed by the plugin .so/.dll file
    #[no_mangle]
    pub extern "C" fn _cloudlist_provider_init() -> *mut dyn Provider {
        // Plugin returns a pointer to its provider implementation
        Box::into_raw(Box::new(MyCloudProvider::new()))
    }
    ```

## 3. The Three Integrated Engines

### 3.1. Discovery Engine

*   **Function:** Orchestrated by the core, the Discovery Engine finds and loads all available provider plugins from the filesystem. It concurrently invokes the `discover_assets` method on each provider, passing the relevant configuration. It is responsible for aggregating all discovered assets into a single, unified, in-memory inventory.
*   **Data Model:** The `Asset` struct, defined in the `cloudlist-ng-api` crate, is the canonical data model. Its flexible `metadata` `HashMap` is crucial for allowing providers to return rich, resource-specific data that the Policy Engine can then evaluate.

### 3.2. Policy Engine

*   **Function:** The Policy Engine is a first-class component that evaluates a set of policies against the in-memory asset inventory.
*   **Pluggable Evaluators:** The engine will support different policy languages via a `PolicyEvaluator` trait. This allows for future extensibility without modifying the engine's core logic.
    ```rust
    // In cloudlist-ng-policy crate
    pub trait PolicyEvaluator {
        fn name(&self) -> &'static str;
        fn evaluate(&self, assets: &[Asset]) -> Vec<Violation>;
    }
    ```
*   **Native YAML Evaluator:** A simple, built-in evaluator for common checks. The YAML format will be intuitive and human-readable.
    ```yaml
    # Example policy: Prohibit public S3 buckets
    id: "aws-s3-no-public-read"
    severity: "high"
    description: "Ensures AWS S3 buckets do not allow public read access"
    target:
      provider: "aws"
      asset_type: "s3_bucket"
    rules:
      - field: "metadata.acl"
        condition: "contains"
        value: "PublicRead"
    remediation:
      mode: "dry-run"
      action: "aws_s3_set_private"
      description: "Set bucket ACL to private."
      revert:
        action: "aws_s3_set_acl"
        params:
          acl: "public-read"
    ```
*   **OPA/Rego Evaluator:** Integration with OPA (Open Policy Agent) is a priority for advanced use cases. We will investigate using a pure Rust Rego implementation like `opa-rs`. If that proves immature, we will use the official OPA Go SDK via CGo bindings, isolating it behind the `PolicyEvaluator` trait.

### 3.3. Remediation Engine

*   **Function:** The Remediation Engine acts on `Violation` objects produced by the Policy Engine. This is a critical component where safety is the absolute priority.
*   **Safety-First Modes:** The engine *must* support the following modes, configurable globally or per-policy:
    *   `dry-run` (Default): Logs the remediation action that would be taken without executing it.
    *   `auto`: Automatically executes the defined remediation action.
    *   `require-approval`: Pauses execution and waits for interactive operator approval via the CLI before proceeding with a remediation action.
*   **Remediation Safety Mechanisms:**
    *   **Concurrency Control:** A global configuration setting, `remediation.concurrency` (default: 1), will limit the number of simultaneous write actions to cloud APIs, preventing rate-limiting and controlling the blast radius of changes.
    *   **Rollback Definition:** As shown in the YAML example, policies can define a `revert` block. The Remediation Engine will not perform automatic rollbacks, but it will expose a command (`cloudlist-ng revert --run-id <id>`) to allow an operator to manually trigger the defined revert actions for a specific run.

## 4. Production-Grade & Cross-Cutting Concerns

### 4.1. Unified Configuration

Configuration will be managed via a single `cloudlist-ng.toml` file. Every parameter will be overridable by environment variables with a `CLN_` prefix (e.g., `CLN_REMEDIATION_CONCURRENCY=5`).

### 4.2. Observability

*   **Structured Logging:** We will use the `tracing` and `tracing-subscriber` crates to implement structured (JSON) logging for all events. This is non-negotiable for a modern, automatable tool.
*   **Prometheus Metrics:** The application will expose a `/metrics` endpoint (e.g., on port 9090) providing key Prometheus metrics, including `assets_discovered_total`, `policy_evaluations_total`, and `policy_violations_total{policy_id, severity}`.

### 4.3. Secure Credential Management

`cloudlist-ng` will not manage credentials directly. It will use the standard credential chain of the underlying cloud SDKs (e.g., `rusoto` or the official AWS Rust SDK). A clear separation will be maintained:
*   **Discovery Credentials:** Should be configured with read-only permissions.
*   **Remediation Credentials:** A separate configuration block will allow specifying a different profile or role to assume for write actions, enabling the principle of least privilege.

### 4.4. Stateful Policy Handling

To handle stateful policies (e.g., "delete this resource if it's been unused for 14 days") without adding the complexity of a database, we will adopt the **Stateful Tagging** pattern.
*   **Workflow Example:**
    1.  **Policy 1 (Mark):** A policy detects an unused EBS volume. Its remediation action is not to delete it, but to call the AWS API to add a tag: `cloudlist-ng:state:marked-for-deletion-timestamp` = `<utc_now>`.
    2.  **Policy 2 (Enforce):** A separate policy triggers on resources with the `marked-for-deletion-timestamp` tag. Its logic checks if `now() > timestamp + 14 days`. If true, it triggers the actual deletion remediation.
*   **Advantage:** This keeps the `cloudlist-ng` engine stateless, highly scalable, and makes the state of any resource visible directly within the cloud provider's console.

## 5. Proposed Project Structure & Roadmap

### 5.1. Rust Crate Structure

```
cloudlist-ng/
├── cloudlist-ng-core/      # The main binary, CLI logic, and orchestration
├── cloudlist-ng-api/       # The stable plugin API contract (traits, structs)
├── cloudlist-ng-discovery/ # The discovery engine logic
├── cloudlist-ng-policy/    # The policy engine and evaluators (YAML, OPA)
├── cloudlist-ng-remediation/ # The remediation engine logic
└── plugins/
    └── cloudlist-provider-aws/ # Example of a provider plugin crate
```

### 5.2. Phased Implementation Roadmap

*   **Phase 1 (Core & Discovery):**
    *   Build `cloudlist-ng-core` with configuration, logging, and the plugin loader.
    *   Define the stable API in `cloudlist-ng-api`.
    *   Implement the Discovery Engine.
    *   Implement `cloudlist-provider-aws` as the first plugin, focusing on discovering EC2 instances and S3 buckets.
    *   **Goal:** A working binary that can discover assets from AWS and print them as JSON.

*   **Phase 2 (Policy Engine):**
    *   Build the `cloudlist-ng-policy` crate with the `PolicyEvaluator` trait.
    *   Implement the native YAML evaluator.
    *   Integrate the policy engine into the core workflow.
    *   **Goal:** A binary that can discover assets and evaluate policies, reporting violations.

*   **Phase 3 (Remediation & Enterprise Features):**
    *   Build the `cloudlist-ng-remediation` crate with all safety features (`dry-run`, concurrency control).
    *   Add the `require-approval` and `revert` functionalities.
    *   Implement the Prometheus metrics endpoint.
    *   Begin integration of the OPA/Rego evaluator.
    *   **Goal:** A feature-complete, enterprise-ready platform for cloud asset discovery, policy evaluation, and safe remediation.
