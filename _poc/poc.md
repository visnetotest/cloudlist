# Proof of Concept (PoC) Strategy

## Strategic Context

This document outlines two parallel Proof of Concept (PoC) initiatives designed to accelerate `cloudlist`'s evolution from a discovery tool into a full-fledged Asset Intelligence Platform. These PoCs represent a dual-track strategy:

1.  **The Policy Engine PoC (Feature-Track)**: Focuses on delivering immediate, tangible value to users by adding a critical security and compliance feature to the existing architecture.
2.  **The Asset Intelligence PoC (Platform-Track)**: Focuses on a foundational architectural shift to an event-driven model, enabling a future generation of 10x capabilities and ensuring long-term scalability.

This approach allows us to ship value to users now while simultaneously investing in the strategic platform for tomorrow.

---

# PoC 1: The Policy Engine

**Objective**: To validate the core functionality of a native "Policy as Code" engine that can evaluate user-defined rules against discovered assets.

### 1. Goals

*   **Prove Technical Feasibility**: Demonstrate that we can successfully build and integrate a native policy evaluator into the `cloudlist` scan process.
*   **Validate Data Models**: Confirm that the proposed `Policy` and `PolicyReport` data models are effective and ergonomic for users.
*   **Establish a Foundation**: Build a solid, extensible foundation for future, more advanced policy features (e.g., OPA integration, auto-remediation).

### 2. Success Criteria

*   **Technical Validation**: The engine must successfully evaluate a sample set of 5 distinct policies (e.g., checking for public S3 buckets, unencrypted disks) against a corpus of 100+ mock assets, producing a 100% accurate JSON report.
*   **Performance Validation**: The policy evaluation process must add no more than a 20% time overhead to a standard `cloudlist` scan for the targeted resource types.
*   **Architectural Validation**: The final code must provide a clean, pluggable interface for the evaluator, allowing a future `OPAEvaluator` to be added with minimal refactoring of the core engine.

### 3. Key Risks & Mitigation Strategies

*   **Risk**: The native rule logic becomes overly complex and difficult to maintain.
    *   **Mitigation**: We will strictly adhere to the declarative `all`/`any` structure defined in the TRD. More complex procedural logic will be explicitly deferred to the future OPA integration phase.
*   **Risk**: Performance overhead exceeds the 20% target, making it unusable for large scans.
    *   **Mitigation**: We will conduct performance profiling early in the development process, focusing on the efficiency of asset-to-policy matching before evaluation.

### 4. Benefits

*   **For Users (Security & DevOps Teams)**: Immediately automates the tedious, error-prone process of manually checking for common misconfigurations, freeing up operators to focus on higher-value work.
*   **For the Business**: Directly reduces organizational risk by systematically and automatically identifying known-bad configurations. It provides a clear, repeatable mechanism to demonstrate compliance to auditors.

---

# PoC 2: The Asset Intelligence Platform Core

**Objective**: To validate the foundational paradigm shift from a batch-oriented tool to an event-driven, stream-processing platform.

### 1. Goals

*   **Prove Event-Driven Architecture**: Demonstrate the viability of a decoupled, microservice-based architecture using an event bus as the central nervous system.
*   **Validate Core Correlation Concept**: Build a functioning, albeit simple, "Correlation Service" to prove out the concept of unifying raw discovery events into a canonical asset identity.
*   **Establish a Scalable Pattern**: Create a foundational code structure for building future real-time stream-processing services (e.g., enrichment, policy, graph loading).

### 2. Success Criteria

*   **Technical Validation**: The `cloudlist agent` command must successfully launch and run an end-to-end event pipeline: a collector must publish `RawDiscoveryEvent`s, the `CorrelationService` must consume them and publish `CorrelatedAssetEvent`s, and a `LoggerSink` must correctly output the final, correlated events to the console.
*   **Architectural Validation**: The event bus interface and service templates must be generic. We must demonstrate that a new stream-processing service can be added to the agent without modifying the core bus or existing services.
*   **Decoupling Validation**: The collector and correlation services must run as independent, concurrent goroutines that communicate *only* through the event bus, with no direct dependencies.

### 3. Key Risks & Mitigation Strategies

*   **Risk**: The in-memory event bus is too simplistic and hides real-world distributed systems challenges (e.g., backpressure, delivery guarantees).
    *   **Mitigation**: The PoC will focus on defining clean, bus-agnostic interfaces for services. The services should not know if the underlying bus is in-memory channels or a production system like Kafka, ensuring future swappability.
*   **Risk**: The concept of a "Universal Asset ID" proves too complex to implement in a simple PoC.
    *   **Mitigation**: The correlation service will use a simple, deterministic hashing method based on key asset properties (e.g., ARN, resource ID). This validates the *flow* of correlation while deferring the *complexity* of the logic itself.

### 4. Benefits

*   **For Users (Platform & SRE Teams)**: This is the critical first step toward providing near real-time asset visibility, eliminating the problem of stale data from periodic scans.
*   **For the Business**: This PoC unlocks the path to a modern, highly scalable architecture. It is the foundational investment required to reduce Mean Time to Detect (MTTD) from hours to seconds and to enable future strategic capabilities like attack path analysis and automated remediation, creating significant market differentiation.
